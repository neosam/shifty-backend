//! Business-Logic-Tier Implementation von
//! [`service::pdf_shiftplan::PdfShiftplanService`] (Phase 49 PDF-03/PDF-04).
//!
//! Assembler-Service für den Wochen-PDF-Download. Kombiniert
//! [`service::week_status::WeekStatusService`] (Gate) +
//! [`service::shiftplan::ShiftplanViewService`] (View) +
//! [`service::sales_person::SalesPersonService`] (Filter `deleted.is_none()`)
//! + [`crate::pdf_render::render_shiftplan_week_pdf`] (Rendering).
//!
//! Beide Aufrufer (REST-Handler Plan 02, Scheduler-Refactor Plan 03) gehen
//! durch [`PdfShiftplanServiceImpl::render_week_pdf`]. DRY-Kern.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    pdf_shiftplan::PdfShiftplanService,
    permission::Authentication,
    sales_person::{SalesPerson, SalesPersonService},
    shiftplan::ShiftplanViewService,
    week_status::{WeekStatus, WeekStatusService},
    PermissionService, ServiceError, ValidationFailureItem,
};
use uuid::Uuid;

use crate::gen_service_impl;
#[allow(unused_imports)]
use crate::pdf_render;

gen_service_impl! {
    struct PdfShiftplanServiceImpl: service::pdf_shiftplan::PdfShiftplanService = PdfShiftplanServiceDeps {
        ShiftplanViewService: ShiftplanViewService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = shiftplan_view_service,
        SalesPersonService: SalesPersonService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = sales_person_service,
        WeekStatusService: WeekStatusService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = week_status_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

impl<Deps: PdfShiftplanServiceDeps> PdfShiftplanServiceImpl<Deps> {
    pub fn new(
        shiftplan_view_service: Arc<Deps::ShiftplanViewService>,
        sales_person_service: Arc<Deps::SalesPersonService>,
        week_status_service: Arc<Deps::WeekStatusService>,
        permission_service: Arc<Deps::PermissionService>,
        transaction_dao: Arc<Deps::TransactionDao>,
    ) -> Self {
        Self {
            shiftplan_view_service,
            sales_person_service,
            week_status_service,
            permission_service,
            transaction_dao,
        }
    }
}

impl<Deps: PdfShiftplanServiceDeps> Debug for PdfShiftplanServiceImpl<Deps> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PdfShiftplanServiceImpl").finish_non_exhaustive()
    }
}

/// Hilfs-Reiner-Filter: aktive SalesPersons (nicht soft-deleted). Als eigene
/// pure fn extrahiert, damit der Filter-Test (`filters_deleted_sales_persons`)
/// deterministisch ohne printpdf-Byte-Grep prüfen kann (siehe PLAN-01
/// Alternative-Assertion in Task 2).
#[allow(dead_code)]
pub(crate) fn filter_active(sales_persons: &[SalesPerson]) -> Vec<SalesPerson> {
    sales_persons
        .iter()
        .filter(|sp| sp.deleted.is_none())
        .cloned()
        .collect()
}

#[async_trait]
impl<Deps: PdfShiftplanServiceDeps + 'static> PdfShiftplanService for PdfShiftplanServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn render_week_pdf(
        &self,
        _shiftplan_id: Uuid,
        _year: u32,
        _calendar_week: u8,
        _context: Authentication<Self::Context>,
        _tx: Option<Self::Transaction>,
    ) -> Result<Vec<u8>, ServiceError> {
        // RED-Phase skeleton — Task 2 (GREEN) will implement the assemble path.
        // Reference the types so unused-import lints don't fire on the stub.
        let _ = (
            &self.shiftplan_view_service,
            &self.sales_person_service,
            &self.week_status_service,
            &self.permission_service,
            &self.transaction_dao,
        );
        let _ = (WeekStatus::Planned, ValidationFailureItem::Duplicate);
        Err(ServiceError::InternalError)
    }
}
