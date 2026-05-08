//! Service-Impl der Vacation-Balance-Domain (Phase 8).
//!
//! Tier-Klassifizierung: **Business-Logic-Service** (D-04 in
//! `08-CONTEXT.md`). Der Service kombiniert Cross-Entity-Daten:
//! - `EmployeeWorkDetailsService::find_by_sales_person_id` →
//!   `vacation_days_for_year` (aliquoter Jahresanspruch pro aktiven
//!   Vertrag, summiert),
//! - `CarryoverService::get_carryover` → `Carryover.vacation` (Übertrag
//!   in Tagen, `i32`),
//! - `AbsenceService::find_by_sales_person` → `AbsencePeriod`s der
//!   Kategorie `Vacation`, getrennt nach `used` (`to_date < today`) und
//!   `planned` (`from_date >= today`).
//!
//! Permissionsmodell:
//! - `get(sales_person_id, year, ...)`: HR ∨ self via
//!   `tokio::join!(check_permission(HR), verify_user_is_sales_person)`
//!   plus `.or()` (T-8-AUTH-01, T-8-IDOR-01).
//! - `get_team(year, ...)`: HR-only (T-8-AUTH-02).
//!
//! Tag-Berechnung (Plan 08-02 — keine Special-Day-Subtraktion in dieser
//! ersten Iteration; A5-Note in 08-RESEARCH.md): pro Vacation-Periode
//! `(to_date - from_date).whole_days() + 1`. Periodeen werden auf das
//! angefragte Jahr beschnitten, indem Tag-Iteration ungenutzt bleibt;
//! statt dessen wird der Schnitt der Range mit `[year-01-01, year-12-31]`
//! gebildet.
//!
//! Carryover-Year-Semantik: `get_carryover(sp_id, year)` liefert das
//! year-Snapshot (Konvention im Repo, vgl. `service_impl/src/carryover.rs`
//! und `service_impl/src/test/carryover.rs`). Wir reichen `year` direkt
//! durch.

use std::sync::Arc;

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    absence::{AbsenceCategory, AbsenceService},
    carryover::CarryoverService,
    clock::ClockService,
    employee_work_details::EmployeeWorkDetailsService,
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

/// Tag-Anzahl einer Periode, beschnitten auf das angefragte Jahr.
///
/// Beide Seiten inklusive (`from..=to`). Liegt die Periode komplett
/// außerhalb des Jahres, wird `0` zurückgegeben.
fn days_in_year_for_period(from: Date, to: Date, year: u32) -> u32 {
    // Build year boundaries; year is u32 from u8/year-of-the-future input,
    // but `Date::from_calendar_date` takes i32. Cast is safe for v1.3 timeframes.
    let year_start = match Date::from_calendar_date(year as i32, Month::January, 1) {
        Ok(d) => d,
        Err(_) => return 0,
    };
    let year_end = match Date::from_calendar_date(year as i32, Month::December, 31) {
        Ok(d) => d,
        Err(_) => return 0,
    };
    let lo = if from > year_start { from } else { year_start };
    let hi = if to < year_end { to } else { year_end };
    if lo > hi {
        return 0;
    }
    // (hi - lo).whole_days() + 1, both inclusive.
    let span = (hi - lo).whole_days();
    if span < 0 {
        0
    } else {
        (span as u32) + 1
    }
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

        // Vacation-Periodeen über AbsenceService — Authentication::Full,
        // weil Outer-Permission bereits geprüft ist.
        let absences = self
            .absence_service
            .find_by_sales_person(sales_person_id, Authentication::Full, Some(tx.clone()))
            .await?;

        let mut used_days: f32 = 0.0;
        let mut planned_days: f32 = 0.0;
        for ap in absences.iter() {
            if ap.deleted.is_some() {
                continue;
            }
            if ap.category != AbsenceCategory::Vacation {
                continue;
            }
            let days = days_in_year_for_period(ap.from_date, ap.to_date, year);
            if days == 0 {
                continue;
            }
            if ap.to_date < today {
                used_days += days as f32;
            } else if ap.from_date > today {
                planned_days += days as f32;
            } else {
                // Aktive Periode (today ∈ [from, to]) — der bereits
                // vergangene Anteil zählt zu used, der zukünftige zu
                // planned. Wir splitten auf today als Stichtag.
                let used_split =
                    days_in_year_for_period(ap.from_date, today, year) as f32;
                let planned_split = (days as f32 - used_split).max(0.0);
                used_days += used_split;
                planned_days += planned_split;
            }
        }

        // Vertragsanspruch — alle Verträge, die das Jahr berühren,
        // beitragen mit `vacation_days_for_year(year)` (liefert 0.0 für
        // nicht überlappende Jahre).
        let work_details = self
            .employee_work_details_service
            .find_by_sales_person_id(sales_person_id, Authentication::Full, Some(tx.clone()))
            .await?;
        let entitled_days: f32 = work_details
            .iter()
            .filter(|wd| wd.deleted.is_none())
            .map(|wd| wd.vacation_days_for_year(year))
            .sum();

        // Carryover — Method heißt `get_carryover` und liefert
        // `Option<Carryover>`. Field heißt `vacation: i32`.
        let carryover_opt = self
            .carryover_service
            .get_carryover(sales_person_id, year, Authentication::Full, Some(tx))
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
