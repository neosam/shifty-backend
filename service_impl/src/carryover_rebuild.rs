//! Phase 4 — CarryoverRebuildService implementation.
//!
//! ## Locked decision (C-Phase4-02 + RESEARCH.md Pitfall 1)
//!
//! Cycle-breaker: this Business-Logic-Tier service consumes
//! [`ReportingService`] (Read) and [`CarryoverService`] (Write). The existing
//! [`CarryoverService`] remains basic-tier and DOES NOT consume
//! [`ReportingService`] — preserving the `Reporting -> Carryover`
//! directionality from Phase 1-3.
//!
//! Variant rejected (B): inlining in `CutoverServiceImpl::run`. CutoverService
//! deps would balloon to 11 (already at 10 in Wave 1) and the rebuild logic is
//! reusable enough to deserve its own service surface.
//!
//! Variant rejected (C): promoting [`CarryoverService`] to Business-Logic. That
//! breaks the Phase 1-3 contract that CarryoverService is basic-tier; the
//! ripple would touch every existing CarryoverService caller.
//!
//! ## Invocation contract
//!
//! `rebuild_for_year(sp, year, ctx, tx)` is called by
//! `CutoverServiceImpl::run` per (sales_person_id, year) tuple from
//! `GateResult::scope_set` (Plan 04-05). The cutover Tx flips the
//! `absence_range_source_active` feature flag to `true` in the same Tx, so
//! [`ReportingService`] internally reads from `derive_hours_for_range` for
//! the three migrated absence categories (Vacation/SickLeave/UnpaidLeave).
//!
//! Permission: [`CUTOVER_ADMIN_PRIVILEGE`]. The cutover caller passes
//! `Authentication::Full`, which bypasses the per-call check; external
//! callers must hold cutover_admin.

use async_trait::async_trait;
use uuid::Uuid;

use crate::gen_service_impl;
use dao::TransactionDao;
use service::carryover::{Carryover, CarryoverService};
use service::carryover_rebuild::CarryoverRebuildService;
use service::cutover::CUTOVER_ADMIN_PRIVILEGE;
use service::permission::Authentication;
use service::reporting::ReportingService;
use service::{PermissionService, ServiceError};

/// Reporting always evaluates whole calendar years; the cutover scope works
/// per-year, so we ask for the full ISO year (week 53 covers all ISO weeks
/// including the rare 53-week years).
const FULL_YEAR_UNTIL_WEEK: u8 = 53;

gen_service_impl! {
    struct CarryoverRebuildServiceImpl: service::carryover_rebuild::CarryoverRebuildService = CarryoverRebuildServiceDeps {
        ReportingService: ReportingService<Context = Self::Context, Transaction = Self::Transaction> = reporting_service,
        CarryoverService: CarryoverService<Context = Self::Context, Transaction = Self::Transaction> = carryover_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: CarryoverRebuildServiceDeps> CarryoverRebuildService
    for CarryoverRebuildServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn rebuild_for_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Permission gate. Inside the cutover Tx the caller passes
        // `Authentication::Full` (service-internal trust); external callers
        // must hold cutover_admin.
        self.permission_service
            .check_permission(CUTOVER_ADMIN_PRIVILEGE, context.clone())
            .await?;

        // Use the caller-passed Tx (CutoverServiceImpl::run holds it open).
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // 1. Read the up-to-date employee report for (sp, year). Within the
        //    cutover Tx the feature flag `absence_range_source_active` is
        //    already `true`, so ReportingService internally pulls from
        //    `AbsenceService::derive_hours_for_range` for the three migrated
        //    absence categories. The internal call uses Authentication::Full
        //    (Backend-internal trust) per the established pattern in
        //    `service_impl/src/feature_flag.rs:36-41`.
        let report = self
            .reporting_service
            .get_report_for_employee(
                &sales_person_id,
                year,
                FULL_YEAR_UNTIL_WEEK,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;

        // 2. Extract carryover-relevant numbers from the report. Field names
        //    are pinned to the EmployeeReport struct in service/src/reporting.rs:
        //    - balance_hours: f32 — surplus/deficit of worked vs expected hours
        //    - vacation_carryover: i32 — leftover vacation entitlement that
        //      rolls over into the next year
        //    Reporting already incorporates the previous year's carryover via
        //    its own pipeline, so we directly persist these values.
        let new_carryover_hours: f32 = report.balance_hours;
        let new_vacation: i32 = report.vacation_carryover;

        // 3. Build the Carryover struct + write via CarryoverService::set_carryover
        //    (which is UPSERT-backed via CarryoverDao::upsert — verified in
        //    service_impl/src/carryover.rs). Reuse `created` from any existing
        //    row so audit history is preserved across rebuilds.
        let now_utc = time::OffsetDateTime::now_utc();
        let now = time::PrimitiveDateTime::new(now_utc.date(), now_utc.time());

        let existing = self
            .carryover_service
            .get_carryover(
                sales_person_id,
                year,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;

        let new_row = Carryover {
            sales_person_id,
            year,
            carryover_hours: new_carryover_hours,
            vacation: new_vacation,
            created: existing.as_ref().map(|c| c.created).unwrap_or(now),
            deleted: None,
            version: Uuid::new_v4(),
        };

        self.carryover_service
            .set_carryover(&new_row, Authentication::Full, Some(tx.clone()))
            .await?;

        // Do NOT commit — caller (CutoverServiceImpl::run) owns the cutover Tx
        // and decides commit-vs-rollback after the gate evaluation.
        Ok(())
    }
}
