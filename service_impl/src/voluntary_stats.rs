//! Phase 54 Plan 03 + Gap-Closure G1 (Plan 54-07 + 54-09-Ist-Fix) —
//! Business-Logic-Tier VoluntaryStatsService (VOL-STAT-01/02, VOL-ACCT-01/02).
//!
//! Ist-Aggregat: der bereits aggregierte `EmployeeReport::volunteer_hours`
//! aus `ReportingService::get_report_for_employee_range` — dieser Wert
//! entspricht dem OVERALL-"Ehrenamt"-Feld auf der Employee-Detail-Seite und
//! deckt beide Erfassungsquellen (manuelle VolunteerWork-ExtraHours +
//! Shiftplan-Cap-Ueberlauf `auto_volunteer_hours` + no_contract-Shiftplan-
//! Stunden) konsistent ab.
//!
//! Soll-Aggregat: `committed_voluntary_target_in_range` — tages-basiert
//! pro-rata (D-F2-01).
//! Contract-Weeks-Nenner: `contract_weeks_count_in_range` — D-F1-01
//! (expected_hours=0 zaehlt MIT).
//!
//! HR-Gate an erster Stelle; Non-HR-Aufrufer erhalten alle Felder als
//! None (API-Level-Redaktion, Praezedenz VAC-OFFSET-01 v1.8, kein 403).
//! Cross-Service-Calls verwenden `Authentication::Full` (Bypass, Auth
//! wurde bereits am Einstieg geprueft — Praezedenz
//! `reference_toggle_service_full_context_bypass`).
//!
//! Anmerkung Phase 54 Marker-Strukturen (rebooking_batch, source-Spalte):
//! In Phase 54 als Datenmodell-Vorbereitung angelegt, aber NOCH NICHT in
//! diesem Ist-Aggregat aktiv — der Rebooking-neutralitaets-Filter wird
//! ab Phase 55 im ReportingService selbst greifen und dann automatisch
//! auch diese Kette treffen.

use crate::gen_service_impl;
use crate::reporting::{committed_voluntary_target_in_range, contract_weeks_count_in_range};
use async_trait::async_trait;
use dao::TransactionDao;
use service::absence::AbsenceService;
use service::employee_work_details::EmployeeWorkDetailsService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::reporting::ReportingService;
use service::sales_person::SalesPersonService;
use service::voluntary_stats::{VoluntaryStats, VoluntaryStatsService};
use service::{PermissionService, ServiceError};
use shifty_utils::ShiftyDate;
use uuid::Uuid;

gen_service_impl! {
    struct VoluntaryStatsServiceImpl: VoluntaryStatsService = VoluntaryStatsServiceDeps {
        ReportingService: ReportingService<Transaction = Self::Transaction, Context = Self::Context> = reporting_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Transaction = Self::Transaction, Context = Self::Context> = employee_work_details_service,
        SalesPersonService: SalesPersonService<Transaction = Self::Transaction, Context = Self::Context> = sales_person_service,
        // Phase 54.5 (D-54.5-03): AbsenceService fuer whole-week-out
        // Soll-Aggregation. Business-Logic → Business-Logic-Konsum, kein
        // Zyklus (AbsenceService konsumiert keinen VoluntaryStatsService).
        // Load erfolgt mit `Authentication::Full` als Cross-Service-Bypass
        // (HR-Gate wurde bereits am Einstieg verifiziert; Praezedenz
        // `reference_toggle_service_full_context_bypass`).
        AbsenceService: AbsenceService<Transaction = Self::Transaction, Context = Self::Context> = absence_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: VoluntaryStatsServiceDeps> VoluntaryStatsService for VoluntaryStatsServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_voluntary_stats(
        &self,
        sales_person_id: Uuid,
        from_date: ShiftyDate,
        to_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<VoluntaryStats, ServiceError> {
        // HR-Gate ZUERST — kein Datenabruf vor Auth (Praezedenz Phase 41 D-AVG-05).
        let is_hr = self
            .permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await
            .is_ok();

        // Non-HR: alle Felder None (API-Level-Redaktion, keine 403).
        if !is_hr {
            return Ok(VoluntaryStats {
                ist_per_contract_week: None,
                ist_total: None,
                soll_total: None,
                delta: None,
                contract_weeks: None,
                ist_per_soll_pct: None,
            });
        }

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // SalesPerson-Existenzpruefung mit internem Full-Context
        // (Cross-Service-Bypass, Auth wurde oben verifiziert).
        let _sp = self
            .sales_person_service
            .get(sales_person_id, Authentication::Full, Some(tx.clone()))
            .await?;

        // Ist-Aggregat: der aggregierte `volunteer_hours`-Wert aus dem
        // Range-Report. Deckt manuelle VolunteerWork-ExtraHours PLUS
        // Shiftplan-Cap-Ueberlauf PLUS no_contract-Shiftplan-Stunden ab —
        // konsistent zum OVERALL-"Ehrenamt"-Wert auf der UI. carryover=false,
        // weil `volunteer_hours` nicht carryover-abhaengig ist (nur
        // `balance_hours` beruecksichtigt carryover — hier irrelevant).
        let report = self
            .reporting_service
            .get_report_for_employee_range(
                &sales_person_id,
                from_date,
                to_date,
                false,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let ist_total = report.volunteer_hours;

        // EmployeeWorkDetails des SalesPerson fuer Soll + Contract-Weeks.
        let working_hours = self
            .employee_work_details_service
            .find_by_sales_person_id(sales_person_id, Authentication::Full, Some(tx.clone()))
            .await?;
        let working_hours_vec: Vec<_> = working_hours.iter().cloned().collect();

        // Phase 54.5 (D-54.5-03): Absence-Perioden des SalesPerson fuer die
        // whole-week-out-Angleichung an Pfad A (Weekly-Anzeige). Load nur
        // im HR-Path (Non-HR ist oben schon retourniert). Cross-Service-
        // Bypass mit `Authentication::Full`, konsistent zu den anderen
        // internen Loads dieser fn.
        let absences = self
            .absence_service
            .find_by_sales_person(sales_person_id, Authentication::Full, Some(tx.clone()))
            .await?;
        let absences_vec: Vec<_> = absences.iter().cloned().collect();

        self.transaction_dao.commit(tx).await?;

        let soll_total = committed_voluntary_target_in_range(
            &working_hours_vec,
            from_date,
            to_date,
            &absences_vec,
        );
        let contract_weeks = contract_weeks_count_in_range(
            &working_hours_vec,
            from_date,
            to_date,
            &absences_vec,
        );
        let ist_per_contract_week = if contract_weeks == 0 {
            0.0
        } else {
            ist_total / contract_weeks as f32
        };
        let delta = ist_total - soll_total;
        // Erfuellungsgrad: None wenn kein Soll (Nicht-Freiwillige oder Range
        // komplett in Absence-Wochen → Division-by-zero-Guard). Sonst
        // ist/soll * 100 (kann >100 sein bei Ist > Soll).
        let ist_per_soll_pct = if soll_total.abs() < 1e-6 {
            None
        } else {
            Some((ist_total / soll_total) * 100.0)
        };

        Ok(VoluntaryStats {
            ist_per_contract_week: Some(ist_per_contract_week),
            ist_total: Some(ist_total),
            soll_total: Some(soll_total),
            delta: Some(delta),
            contract_weeks: Some(contract_weeks),
            ist_per_soll_pct,
        })
    }
}
