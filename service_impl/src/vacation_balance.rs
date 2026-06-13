//! Service-Impl der Vacation-Balance-Domain (Phase 8).
//!
//! Tier-Klassifizierung: **Business-Logic-Service** (D-04 in
//! `08-CONTEXT.md`). Der Service kombiniert Cross-Entity-Daten:
//! - `EmployeeWorkDetailsService::find_by_sales_person_id` →
//!   `vacation_days_for_year` (aliquoter Jahresanspruch pro aktiven
//!   Vertrag, summiert),
//! - `CarryoverService::get_carryover` → `Carryover.vacation` (Übertrag
//!   in Tagen, `i32`),
//! - `AbsenceService::derive_hours_for_range` → pro-Tag aufgelöste
//!   Vacation-Stunden für das angefragte Jahr, getrennt nach `used`
//!   (`date <= today`) und `planned` (`date > today`).
//!
//! Permissionsmodell:
//! - `get(sales_person_id, year, ...)`: HR ∨ self via
//!   `tokio::join!(check_permission(HR), verify_user_is_sales_person)`
//!   plus `.or()` (T-8-AUTH-01, T-8-IDOR-01).
//! - `get_team(year, ...)`: HR-only (T-8-AUTH-02).
//!
//! Tag-Berechnung (stundenbasiert, konsistent mit `ReportingService` —
//! Decision 2026-06-12, ersetzt die naive Kalendertag-Zählung aus Plan 08-02):
//! Jeder Vacation-Tag des Jahres wird über `derive_hours_for_range` vertraglich
//! zu effektiven Stunden aufgelöst (Workdays, Feiertage und Halbtage via
//! `day_fraction` bereits berücksichtigt). Die Summe wird durch das
//! `hours_per_day` des aktiven Vertrags geteilt (Modell A: ein `hours_per_day`
//! pro Jahr, siehe [`representative_hours_per_day`]). Die Beschneidung auf das
//! Jahr erfolgt implizit über den `[year-01-01, year-12-31]`-Range.
//!
//! Carryover-Year-Semantik: Ein `Carryover`-Eintrag mit `year = Y` speichert
//! den Ende-von-Jahr-Y-Saldo, der in Jahr Y+1 eingebracht wird. Um den
//! Übertrag in `year` zu erhalten, muss also `get_carryover(sp_id, year - 1)`
//! aufgerufen werden — identisch zum Aufruf in `ReportingService::get_employee`
//! (line 603-616 dort). Fehler in der ursprünglichen Implementierung: `year`
//! wurde direkt weitergereicht, was den Ende-von-`year`-Saldo (→ nächstes Jahr)
//! statt den Saldo aus dem Vorjahr lieferte.

use std::sync::Arc;

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    absence::{AbsenceCategory, AbsenceService},
    carryover::CarryoverService,
    clock::ClockService,
    employee_work_details::{EmployeeWorkDetails, EmployeeWorkDetailsService},
    permission::{Authentication, HR_PRIVILEGE},
    sales_person::SalesPersonService,
    vacation_balance::{VacationBalance, VacationBalanceService},
    PermissionService, ServiceError,
};
use time::{Date, Month};
use tokio::join;
use uuid::Uuid;

use crate::gen_service_impl;

gen_service_impl! {
    struct VacationBalanceServiceImpl: VacationBalanceService = VacationBalanceServiceDeps {
        AbsenceService: AbsenceService<Context = Self::Context, Transaction = Self::Transaction> = absence_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Context = Self::Context, Transaction = Self::Transaction> = employee_work_details_service,
        CarryoverService: CarryoverService<Context = Self::Context, Transaction = Self::Transaction> = carryover_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

/// `hours_per_day` des für `year` repräsentativen Vertrags: der jüngste (nach
/// Vertragsbeginn) nicht-gelöschte Vertrag, dessen Jahresspanne `year` berührt.
///
/// Modell A (Decision 2026-06-12): ein `hours_per_day` pro Jahr für die
/// Stunden→Tage-Umrechnung — exakt bei einem Vertrag/Jahr, approximativ bei
/// Vertragswechsel mit abweichendem `hours_per_day` mitten im Jahr. Konsistent
/// mit `ReportingService`, der pro Gruppe ebenfalls ein `hours_per_day` nutzt.
fn representative_hours_per_day(work_details: &[EmployeeWorkDetails], year: u32) -> f32 {
    work_details
        .iter()
        .filter(|wd| wd.deleted.is_none())
        .filter(|wd| {
            let from_year = wd.from_date().map(|d| d.calendar_year()).unwrap_or(u32::MAX);
            let to_year = wd.to_date().map(|d| d.calendar_year()).unwrap_or(u32::MIN);
            from_year <= year && year <= to_year
        })
        .max_by_key(|wd| (wd.from_year, wd.from_calendar_week))
        // hours_per_active_weekday (statt hours_per_day): muss mit dem Per-Tag-
        // Soll übereinstimmen, das derive_hours_for_range zum Aufbau von
        // used_hours/planned_hours verwendet. Sonst driftet die Stunden→Tage-
        // Umrechnung (used_days = used_hours / hours_per_day) bei divergierendem
        // workdays_per_week (Bug: vacation-hours-overcounted).
        .map(|wd| wd.hours_per_active_weekday())
        .unwrap_or(0.0)
}

#[async_trait]
impl<Deps: VacationBalanceServiceDeps> VacationBalanceService for VacationBalanceServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<VacationBalance, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Permission HR ∨ self (analog absence.rs:110-119).
        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;

        let balance = self
            .compute_balance(sales_person_id, year, tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(balance)
    }

    async fn get_team(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[VacationBalance]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // HR-only (T-8-AUTH-02).
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let sales_persons = self
            .sales_person_service
            .get_all_paid(Authentication::Full, Some(tx.clone()))
            .await?;

        let mut balances: Vec<VacationBalance> = Vec::with_capacity(sales_persons.len());
        for sp in sales_persons.iter() {
            let balance = self.compute_balance(sp.id, year, tx.clone()).await?;
            balances.push(balance);
        }

        self.transaction_dao.commit(tx).await?;
        Ok(balances.into())
    }
}

impl<Deps: VacationBalanceServiceDeps> VacationBalanceServiceImpl<Deps> {
    /// Berechnet das Resturlaubs-Aggregat ohne Permission-Check und ohne
    /// commit. Aufrufer (`get`/`get_team`) haben Permission bereits geprüft
    /// und übergeben einen aktiven `tx`. Innere Service-Calls laufen mit
    /// `Authentication::Full`.
    async fn compute_balance(
        &self,
        sales_person_id: Uuid,
        year: u32,
        tx: <Deps as VacationBalanceServiceDeps>::Transaction,
    ) -> Result<VacationBalance, ServiceError> {
        let today = self.clock_service.date_now();

        // Vertragsanspruch — alle Verträge, die das Jahr berühren, beitragen
        // mit `vacation_days_for_year(year)` (liefert 0.0 für nicht
        // überlappende Jahre). Zuerst geladen, weil wir `hours_per_day` für die
        // Stunden→Tage-Umrechnung der Used/Planned-Tage brauchen.
        let work_details = self
            .employee_work_details_service
            .find_by_sales_person_id(sales_person_id, Authentication::Full, Some(tx.clone()))
            .await?;
        let entitled_days: f32 = work_details
            .iter()
            .filter(|wd| wd.deleted.is_none())
            .map(|wd| wd.vacation_days_for_year(year))
            .sum();

        // Stundenbasierte Used/Planned-Tage (konsistent mit ReportingService):
        // jeden Vacation-Tag des Jahres vertraglich zu effektiven Stunden
        // auflösen (Workdays, Feiertage, Halbtage via day_fraction sind in
        // `derive_hours_for_range` bereits berücksichtigt) und durch
        // `hours_per_day` des aktiven Vertrags teilen. Ersetzt die frühere
        // naive Kalendertag-Zählung (zählte Wochenenden/Feiertage/Halbtage
        // falsch). Conflict-Resolution (Sick > Vacation > Unpaid) liefert pro
        // Tag genau eine Kategorie — Vacation wird hier herausgefiltert.
        let hours_per_day = representative_hours_per_day(&work_details, year);
        let mut used_hours: f32 = 0.0;
        let mut planned_hours: f32 = 0.0;
        if let (Ok(year_start), Ok(year_end)) = (
            Date::from_calendar_date(year as i32, Month::January, 1),
            Date::from_calendar_date(year as i32, Month::December, 31),
        ) {
            let resolved = self
                .absence_service
                .derive_hours_for_range(
                    year_start,
                    year_end,
                    sales_person_id,
                    Authentication::Full,
                    Some(tx.clone()),
                )
                .await?;
            for (date, resolved_day) in resolved.iter() {
                if resolved_day.category != AbsenceCategory::Vacation {
                    continue;
                }
                // `today` selbst zählt zu used (aktive Periode splittet am
                // Stichtag: [from, today] used, (today, to] planned).
                if *date <= today {
                    used_hours += resolved_day.hours;
                } else {
                    planned_hours += resolved_day.hours;
                }
            }
        }
        let (used_days, planned_days) = if hours_per_day > 0.0 {
            (used_hours / hours_per_day, planned_hours / hours_per_day)
        } else {
            (0.0, 0.0)
        };

        // Carryover — Ein Carryover-Eintrag mit year=Y speichert den
        // Ende-von-Jahr-Y-Saldo (Übertrag in Jahr Y+1). Um den Übertrag
        // EINGEHEND in `year` zu lesen, wird year-1 abgefragt —
        // konsistent mit ReportingService::get_employee (Z. 603-616 dort).
        let carryover_opt = self
            .carryover_service
            .get_carryover(sales_person_id, year - 1, Authentication::Full, Some(tx))
            .await?;
        let carryover_days: i32 = carryover_opt
            .filter(|c| c.deleted.is_none())
            .map(|c| c.vacation)
            .unwrap_or(0);

        let remaining_days =
            entitled_days + carryover_days as f32 - (used_days + planned_days);

        Ok(VacationBalance {
            sales_person_id,
            year,
            entitled_days,
            carryover_days,
            used_days,
            planned_days,
            remaining_days,
        })
    }
}
