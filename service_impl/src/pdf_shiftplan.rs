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
//!
//! ## Reihenfolge im Assemble-Path
//!
//! 1. **WeekStatus-Gate** — kein Aufwand vor der Auth/Status-Prüfung.
//! 2. **ShiftplanView** — der teure Read.
//! 3. **SalesPersons + Filter** — `deleted.is_none()` (D-49-05, PDF-05).
//! 4. **Pure Renderer** — `pdf_render::render_shiftplan_week_pdf`.
//!
//! Der `context` wird 1:1 an alle konsumierten Services weitergereicht
//! (D-49-07); niemals wird intern auf `Authentication::Full` hochgehebelt.

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

/// Pure Filter-Helfer: aktive SalesPersons (nicht soft-deleted). Als eigene
/// Funktion extrahiert, damit die PDF-05-Assertion deterministisch ohne
/// printpdf-Byte-Grep testbar ist (Text landet in FlateDecode-Streams).
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
        shiftplan_id: Uuid,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Vec<u8>, ServiceError> {
        // 1) WeekStatus-Gate — Defense-in-Depth. Nur Planned/Locked freigegeben
        //    (D-49-06). Kein View/get_all-Call bei Ablehnung → zero side effects.
        let status = self
            .week_status_service
            .get_week_status(year, calendar_week, context.clone(), tx.clone())
            .await?;
        if !matches!(status, WeekStatus::Planned | WeekStatus::Locked) {
            return Err(ServiceError::ValidationError(Arc::from([
                ValidationFailureItem::InvalidValue(Arc::from(format!(
                    "Woche KW{calendar_week:02}/{year} ist im Status {status:?} — kein Download"
                ))),
            ])));
        }

        // 2) View-Read (D-49-07: caller-context weitergereicht).
        let week_view = self
            .shiftplan_view_service
            .get_shiftplan_week(shiftplan_id, year, calendar_week, context.clone(), tx.clone())
            .await?;

        // 3) SalesPersons + aktive-Filter (D-49-05, PDF-05).
        let all_sales_persons = self
            .sales_person_service
            .get_all(context, tx)
            .await?;
        let active_sales_persons = filter_active(&all_sales_persons);

        // 4) Pure Renderer.
        pdf_render::render_shiftplan_week_pdf(
            &week_view,
            &active_sales_persons,
            year,
            calendar_week,
        )
    }
}
