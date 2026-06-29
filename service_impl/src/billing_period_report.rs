use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use service::text_template::TemplateEngine;
use tera::{Context, Tera};
use dao::TransactionDao;
use service::billing_period::{
    BillingPeriod, BillingPeriodSalesPerson, BillingPeriodService, BillingPeriodValue,
    BillingPeriodValueType,
};
use service::billing_period_report::BillingPeriodReportService;
use service::clock::ClockService;
use service::employee_work_details::EmployeeWorkDetailsService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::reporting::ReportingService;
use service::sales_person::{SalesPerson, SalesPersonService};
use service::text_template::TextTemplateService;
use service::uuid_service::UuidService;
use service::PermissionService;
use service::ServiceError;
use shifty_utils::ShiftyDate;
use time::macros::datetime;
use uuid::Uuid;

use crate::gen_service_impl;

const BILLING_PERIOD_REPORT_SERVICE: &str = "BillingPeriodReportService";

/// Schema version stamped on every newly persisted `billing_period` snapshot.
///
/// Bump by one whenever you add, remove, rename, or change the computation of
/// any persisted `value_type` on `billing_period_sales_person`. See
/// `CLAUDE.md` → "Billing Period Snapshot Schema Versioning" for the rationale
/// and the full list of trigger conditions.
///
/// History:
/// - v1/v2: pre-Phase-2 baseline (initial snapshot model + intermediate bump).
/// - v3: Phase 2 — neuer `value_type` UnpaidLeave + AbsencePeriod-derived
///   Vacation/Sick/UnpaidLeave (changed the computation that produces these
///   value_types vs. the Phase-1 extra_hours-derived computation).
/// - v4: Phase 8.3 — `AbsencePeriod.day_fraction == Half` halbiert die effektive
///   Soll-Stundenzahl pro Tag in `derive_hours_for_range`; betrifft die
///   Vacation/SickLeave/UnpaidLeave-derived value_types (hours + days). Alte
///   Snapshots (v3) haben kein `day_fraction`-Feld in den Quell-Rows (Default
///   `'full'` nach Migration), aber die Computation würde bei Re-Validation
///   andere Werte liefern, sobald ein Half-Eintrag existiert. Validator MUSS
///   daher v3-Snapshots als "older schema" markieren.
/// - v5: Phase 8.4 — Additiver Merge: Vacation/SickLeave/UnpaidLeave werden
///   nun aus BEIDEN Quellen (lebende `extra_hours` + `absence_period` via
///   `derive_hours_for_range`) summiert statt quellen-exklusiv per Flag.
///   Ältere Snapshots (v4) haben die `extra_hours`-Seite nicht mitgezählt
///   (Flag war on → nur absence_period) oder umgekehrt (Flag off → nur
///   extra_hours). Ein Validator kann v4-Snapshots nicht sicher re-validieren.
/// - v6: Phase 8.4 (Gap 2 / WR-01) — absence_period-derived Vacation/SickLeave/
///   UnpaidLeave reduzieren jetzt SYMMETRISCH zu extra_hours-Absence die
///   `expected_hours`/`balance_hours` (und damit die persistierten value_types
///   Balance + ExpectedHours). v5-Snapshots haben die absence_period-Seite nur
///   in den Display-Stunden, nicht in der Balance gezaehlt — ein Validator kann
///   v5-Balance/ExpectedHours nicht gegen die neue Computation re-validieren.
/// - v7: Bugfix (debug/vacation-hours-overcounted) — Domänenmodell korrigiert:
///   Das Per-Tag-Soll in `derive_hours_for_range` ist `expected_hours /
///   workdays_per_week` (`hours_per_day`), und pro ISO-Woche wird auf höchstens
///   `workdays_per_week` Urlaubstage gedeckelt. Die angehakten Wochentag-
///   Booleans (`has_day_of_week`) sind NUR Verfügbarkeit ("wann die Person
///   arbeiten kann"), NICHT die Zahl der Arbeitstage. Eine volle Urlaubswoche
///   ergibt damit exakt `workdays_per_week` Tage / `expected_hours` Stunden,
///   unabhängig davon, an wie vielen Wochentagen die Person verfügbar ist.
///   Alte Snapshots (v6) zählten jeden verfügbaren Tag (ohne Wochen-Deckelung)
///   und überzählten bei mehr verfügbaren Tagen als `workdays_per_week`; ein
///   Validator kann sie nicht gegen die korrigierte Computation re-validieren.
///   (Version BLEIBT 7 — v7 wurde nie deployed.)
/// - Phase 15 (committed_voluntary Zwei-Band): KEIN Bump — Achse-B-only, kein persistierter value_type berührt.
/// - v8: Bugfix (debug/report-ehrenamt-gesamtstunden) — `get_report_for_employee_range`
///   nutzt jetzt den per-Woche GEDECKELTEN `shiftplan_hours_by_week` für
///   `overall_hours`/`balance_hours`/`shiftplan_hours` statt der rohen,
///   ungedeckelten shiftplan-Summe. Bei `cap_planned_hours_to_expected = true`
///   leakte der Cap-Überlauf (auto_volunteer / Ehrenamt-Anteil) vorher in
///   `overall_hours` + `balance_hours` — und damit in die persistierten value_types
///   Balance + ExpectedHours (siehe value_delta/ytd/full_year unten). Eine
///   Neuberechnung weicht für cap-aktive Mitarbeiter mit Überlauf von v7-Snapshots
///   ab; ein Validator kann v7 nicht gegen die korrigierte Computation
///   re-validieren. (v7 war nie deployed — daher zugleich erster real deploybarer Bump.)
/// - v9: Bump 8->9 (quick-260624-ujk): Die Berechnung des persistierten value_type
///   `volunteer_hours` aendert sich — geleistete Shiftplan-Stunden in Wochen OHNE
///   EmployeeWorkDetails-Vertragszeile zaehlen jetzt als Ehrenamt (volunteer) statt
///   Soll=Ist-neutralisiert. Laut CLAUDE.md (Snapshot Schema Versioning: "Change the
///   computation that produces an existing value_type") ist ein Bump Pflicht, damit
///   Snapshot-Validatoren Schema-Drift von echten Datenfehlern unterscheiden koennen.
///   Betroffen: BillingPeriodValueType::Volunteer (und transitiv Balance/ExpectedHours
///   fuer Mitarbeiter mit Shiftplan-Stunden ohne Vertragszeile im Abrechnungszeitraum).
/// - v10: UV-05 / D-18-07 — fix: converted hours-based absences (extra_hours soft-deleted
///   -> absence_period via derive_hours_for_range) now flow into the per-week category
///   fields (vacation_hours / sick_leave_hours / unpaid_leave_hours) in `hours_per_week`.
///   As a result, `BillingPeriodValueType::VacationDays` (and sick/unpaid days via the
///   same path) change from 0 to the correct >0 value for converted entries. Snapshots
///   written under v9 persist vacation_days=0 for converted absences; re-validating them
///   against the corrected computation would show a false mismatch. Validators MUST treat
///   v9 snapshots as "older schema" and not re-validate vacation/sick/unpaid day counts.
/// - v11: Phase 25 (HOL-01/02, HCFG-01) — derive-on-read holiday auto-credit.
///   `hours_per_week` now returns derived holiday hours in `holiday_hours` and adds them
///   to `absense_hours` when the `holiday_auto_credit` toggle is configured. This changes
///   the computed values for `BillingPeriodValueType::HolidayHours` (and transitively
///   `Balance`, `ExpectedHours`) for employees whose contracts cover configured holiday
///   dates. Validators MUST treat v10 snapshots as "older schema" and skip holiday-hours
///   re-validation for those entries.
/// - v12: Phase 28 (VAC-OFFSET-01 / D-28-05) — off-by-one fix in
///   `EmployeeWorkDetails::vacation_days_for_year`. The year-START proration no longer
///   over-subtracts ~1/365 of the annual entitlement at a 1.1. contract start (it now
///   subtracts the days STRICTLY before the start, so a 1.1. start subtracts 0). Because
///   this value feeds the persisted `BillingPeriodValueType::VacationEntitlement`
///   (reporting.rs:853 <- :803), the computed entitlement changes for partial-year and
///   full-year contracts. Validators MUST treat v11 snapshots as "older schema" and skip
///   vacation-entitlement re-validation for those entries. NOTE: `VacationDays` (taken
///   vacation) is UNAFFECTED — only `VacationEntitlement` (contract aliquot) changes.
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;

gen_service_impl! {
    struct BillingPeriodReportServiceImpl: BillingPeriodReportService = BillingPeriodReportServiceDeps {
        BillingPeriodService: BillingPeriodService<Context = Self::Context, Transaction = Self::Transaction> = billing_period_service,
        ReportingService: ReportingService<Context = Self::Context, Transaction = Self::Transaction> = reporting_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Context = Self::Context, Transaction = Self::Transaction> = employee_work_details_service,
        TextTemplateService: TextTemplateService<Context = Self::Context, Transaction = Self::Transaction> = text_template_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        UuidService: UuidService = uuid_service,
        ClockService: ClockService = clock_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

impl<Deps: BillingPeriodReportServiceDeps> BillingPeriodReportServiceImpl<Deps> {
    pub async fn build_billing_period_report_for_sales_person(
        &self,
        sales_person: SalesPerson,
        start_date: ShiftyDate,
        end_date: ShiftyDate,
        context: Authentication<Deps::Context>,
        tx: Option<Deps::Transaction>,
    ) -> Result<BillingPeriodSalesPerson, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let report_start = self
            .reporting_service
            .get_report_for_employee_range(
                &sales_person.id,
                ShiftyDate::first_day_in_year(start_date.calendar_year()),
                start_date.previous_day(),
                true,
                context.clone(),
                tx.clone().into(),
            )
            .await?;
        let report_end = self
            .reporting_service
            .get_report_for_employee_range(
                &sales_person.id,
                ShiftyDate::first_day_in_year(end_date.calendar_year()),
                end_date,
                true,
                context.clone(),
                tx.clone().into(),
            )
            .await?;
        let report_end_of_year = self
            .reporting_service
            .get_report_for_employee_range(
                &sales_person.id,
                ShiftyDate::first_day_in_year(end_date.calendar_year()),
                ShiftyDate::last_day_in_year(end_date.calendar_year()),
                true,
                context.clone(),
                tx.clone().into(),
            )
            .await?;
        let report_delta = self
            .reporting_service
            .get_report_for_employee_range(
                &sales_person.id,
                start_date,
                end_date,
                false,
                context.clone(),
                tx.clone().into(),
            )
            .await?;

        let mut billing_period_values = BTreeMap::new();
        billing_period_values.insert(
            BillingPeriodValueType::Overall,
            BillingPeriodValue {
                value_delta: report_delta.overall_hours,
                value_ytd_from: report_start.overall_hours,
                value_ytd_to: report_end.overall_hours,
                value_full_year: report_end_of_year.overall_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::Balance,
            BillingPeriodValue {
                value_delta: report_delta.balance_hours,
                value_ytd_from: report_start.balance_hours,
                value_ytd_to: report_end.balance_hours,
                value_full_year: report_end_of_year.balance_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::ExpectedHours,
            BillingPeriodValue {
                value_delta: report_delta.expected_hours,
                value_ytd_from: report_start.expected_hours,
                value_ytd_to: report_end.expected_hours,
                value_full_year: report_end_of_year.expected_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::ExtraWork,
            BillingPeriodValue {
                value_delta: report_delta.extra_work_hours,
                value_ytd_from: report_start.extra_work_hours,
                value_ytd_to: report_end.extra_work_hours,
                value_full_year: report_end_of_year.extra_work_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::VacationHours,
            BillingPeriodValue {
                value_delta: report_delta.vacation_hours,
                value_ytd_from: report_start.vacation_hours,
                value_ytd_to: report_end.vacation_hours,
                value_full_year: report_end_of_year.vacation_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::SickLeave,
            BillingPeriodValue {
                value_delta: report_delta.sick_leave_hours,
                value_ytd_from: report_start.sick_leave_hours,
                value_ytd_to: report_end.sick_leave_hours,
                value_full_year: report_end_of_year.sick_leave_hours,
            },
        );
        // Ab Phase 8.4 additiv: lebende extra_hours(UnpaidLeave) PLUS
        // derive_hours_for_range-abgeleitete UnpaidLeave-Stunden, summiert.
        // Kein Flag-Branch mehr.
        billing_period_values.insert(
            BillingPeriodValueType::UnpaidLeave,
            BillingPeriodValue {
                value_delta: report_delta.unpaid_leave_hours,
                value_ytd_from: report_start.unpaid_leave_hours,
                value_ytd_to: report_end.unpaid_leave_hours,
                value_full_year: report_end_of_year.unpaid_leave_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::Holiday,
            BillingPeriodValue {
                value_delta: report_delta.holiday_hours,
                value_ytd_from: report_start.holiday_hours,
                value_ytd_to: report_end.holiday_hours,
                value_full_year: report_end_of_year.holiday_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::VacationDays,
            BillingPeriodValue {
                value_delta: report_delta.vacation_days,
                value_ytd_from: report_start.vacation_days,
                value_ytd_to: report_end.vacation_days,
                value_full_year: report_end_of_year.vacation_days,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::VacationEntitlement,
            BillingPeriodValue {
                value_delta: report_delta.vacation_entitlement,
                value_ytd_from: report_start.vacation_entitlement,
                value_ytd_to: report_end.vacation_entitlement,
                value_full_year: report_end_of_year.vacation_entitlement,
            },
        );
        if report_delta.volunteer_hours != 0.0 {
            billing_period_values.insert(
                BillingPeriodValueType::Volunteer,
                BillingPeriodValue {
                    value_delta: report_delta.volunteer_hours,
                    value_ytd_from: report_start.volunteer_hours,
                    value_ytd_to: report_end.volunteer_hours,
                    value_full_year: report_end_of_year.volunteer_hours,
                },
            );
        }
        for custom_hours in report_delta.custom_extra_hours.iter() {
            let ytd_from_value = report_start
                .custom_extra_hours
                .iter()
                .find(|ch| ch.name == custom_hours.name)
                .map_or(0.0, |ch| ch.hours);
            let ytd_to_value = report_end
                .custom_extra_hours
                .iter()
                .find(|ch| ch.name == custom_hours.name)
                .map_or(0.0, |ch| ch.hours);
            let full_year_value = report_end_of_year
                .custom_extra_hours
                .iter()
                .find(|ch| ch.name == custom_hours.name)
                .map_or(0.0, |ch| ch.hours);

            billing_period_values.insert(
                BillingPeriodValueType::CustomExtraHours(custom_hours.name.clone()),
                BillingPeriodValue {
                    value_delta: custom_hours.hours,
                    value_ytd_from: ytd_from_value,
                    value_ytd_to: ytd_to_value,
                    value_full_year: full_year_value,
                },
            );
        }

        Ok(BillingPeriodSalesPerson {
            id: Uuid::nil(),
            sales_person_id: sales_person.id,
            values: billing_period_values,
            created_at: datetime!(1970-01-01 00:00:00),
            created_by: "".into(),
            deleted_at: None,
            deleted_by: None,
        })
    }
}

#[async_trait]
impl<Deps: BillingPeriodReportServiceDeps> BillingPeriodReportService
    for BillingPeriodReportServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn build_new_billing_period(
        &self,
        end_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BillingPeriod, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let start_date = self
            .billing_period_service
            .get_latest_billing_period_end_date(context.clone(), tx.clone().into())
            .await?
            .unwrap_or(ShiftyDate::from_date(
                time::OffsetDateTime::UNIX_EPOCH.date(),
            ))
            .next_day();

        let sales_persons = self
            .sales_person_service
            .get_all(context.clone(), tx.clone().into())
            .await?;

        let mut sales_person_reports = Vec::new();
        for sales_person in sales_persons.iter() {
            // D-GATING-STYLE / CVC-10: unbezahlte Personen (is_paid=false) werden ab Phase 17
            // EmployeeWorkDetails-Records halten (rein freiwillige Helfer). Sie duerfen NICHT als
            // BillingPeriodSalesPerson-Eintraege im Snapshot erscheinen — Personen-Set-Konsistenz
            // mit get_week (year-summary) + get_reports_for_all_employees (all-employees-report).
            // KEIN value_type-Change -> KEIN CURRENT_SNAPSHOT_SCHEMA_VERSION-Bump
            // (Wert hier unveraendert; aktuelle Baseline ist 9, siehe const-Definition).
            if !sales_person.is_paid.unwrap_or(false) {
                continue;
            }
            let sales_person_report = self
                .build_billing_period_report_for_sales_person(
                    sales_person.clone(),
                    start_date,
                    end_date,
                    context.clone(),
                    tx.clone().into(),
                )
                .await?;
            sales_person_reports.push(sales_person_report);
        }

        let billing_period = BillingPeriod {
            id: Uuid::nil(),
            start_date,
            end_date,
            snapshot_schema_version: CURRENT_SNAPSHOT_SCHEMA_VERSION,
            sales_persons: sales_person_reports.into(),
            created_at: datetime!(1970-01-01 00:00:00),
            created_by: "".into(),
            deleted_at: None,
            deleted_by: None,
        };

        Ok(billing_period)
    }

    async fn build_and_persist_billing_period_report(
        &self,
        end_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Uuid, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let billing_period = self
            .build_new_billing_period(end_date, context.clone(), tx.clone().into())
            .await?;
        let billing_period_id = billing_period.id;

        self.billing_period_service
            .create_billing_period(
                &billing_period,
                BILLING_PERIOD_REPORT_SERVICE,
                context,
                tx.clone().into(),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(billing_period_id)
    }

    async fn generate_custom_report(
        &self,
        template_id: Uuid,
        billing_period_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<str>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Check HR permission
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        // Load the text template
        let text_template = self
            .text_template_service
            .get_by_id(template_id, context.clone(), tx.clone().into())
            .await?;

        // Load the billing period
        let billing_period = self
            .billing_period_service
            .get_billing_period_by_id(billing_period_id, context.clone(), tx.clone().into())
            .await?;

        // Load all sales persons and employee work details for enrichment
        let all_sales_persons = self
            .sales_person_service
            .get_all(context.clone(), tx.clone().into())
            .await?;
        let all_work_details = self
            .employee_work_details_service
            .all(context.clone(), tx.clone().into())
            .await?;

        // Build template context data as JSON
        let context_data = json!({
            "billing_period": {
                "id": billing_period.id.to_string(),
                "start_date": billing_period.start_date.to_date().to_string(),
                "end_date": billing_period.end_date.to_date().to_string(),
                "created_at": billing_period.created_at.to_string(),
                "created_by": billing_period.created_by.as_ref(),
                "sales_persons": billing_period.sales_persons.iter().map(|sp| {
                    let sales_person = all_sales_persons.iter().find(|s| s.id == sp.sales_person_id);
                    let name = sales_person.map(|s| s.name.as_ref()).unwrap_or("");
                    let is_paid = sales_person.and_then(|s| s.is_paid).unwrap_or(false);
                    let is_dynamic = all_work_details.iter()
                        .filter(|wd| wd.sales_person_id == sp.sales_person_id)
                        .any(|wd| wd.is_dynamic);
                    let sanitize = |v: f32| -> f64 {
                        let v = v as f64;
                        if v.is_nan() || v.is_infinite() { 0.0 } else { v }
                    };
                    let values_map: serde_json::Map<String, serde_json::Value> = sp.values.iter().map(|(key, value)| {
                        (key.as_str().to_string(), json!({
                            "delta": sanitize(value.value_delta),
                            "ytd_from": sanitize(value.value_ytd_from),
                            "ytd_to": sanitize(value.value_ytd_to),
                            "full_year": sanitize(value.value_full_year),
                        }))
                    }).collect();
                    json!({
                        "id": sp.id.to_string(),
                        "sales_person_id": sp.sales_person_id.to_string(),
                        "name": name,
                        "is_paid": is_paid,
                        "is_dynamic": is_dynamic,
                        "values": sp.values.iter().map(|(key, value)| {
                            json!({
                                "type": key.as_str().as_ref(),
                                "value_delta": sanitize(value.value_delta),
                                "value_ytd_from": sanitize(value.value_ytd_from),
                                "value_ytd_to": sanitize(value.value_ytd_to),
                                "value_full_year": sanitize(value.value_full_year),
                            })
                        }).collect::<Vec<_>>(),
                        "values_map": values_map,
                        "created_at": sp.created_at.to_string(),
                        "created_by": sp.created_by.as_ref(),
                    })
                }).collect::<Vec<_>>(),
            },
            "template": {
                "id": text_template.id.to_string(),
                "template_type": text_template.template_type.as_ref(),
                "created_at": text_template.created_at.map(|dt| dt.to_string()),
                "created_by": text_template.created_by.as_ref().map(|s| s.as_ref()),
            }
        });

        // Render using the appropriate engine
        let rendered = match text_template.template_engine {
            TemplateEngine::Tera => {
                let mut tera = Tera::default();
                tera.add_raw_template("custom_report", &text_template.template_text)
                    .map_err(|e| {
                        tracing::error!("Failed to parse Tera template: {}", e);
                        ServiceError::InternalError
                    })?;
                let template_context = Context::from_serialize(&context_data).map_err(|e| {
                    tracing::error!("Failed to serialize template context: {}", e);
                    ServiceError::InternalError
                })?;
                tera.render("custom_report", &template_context)
                    .map_err(|e| {
                        tracing::error!("Failed to render Tera template: {}", e);
                        ServiceError::InternalError
                    })?
            }
            TemplateEngine::MiniJinja => {
                let env = minijinja::Environment::new();
                env.render_str(&text_template.template_text, context_data)
                    .map_err(|e| {
                        tracing::error!("Failed to render MiniJinja template: {e:#}");
                        ServiceError::InternalError
                    })?
            }
        };

        self.transaction_dao.commit(tx).await?;
        Ok(rendered.into())
    }
}
