//! PDF-Shiftplan-Service (Phase 49 PDF-03/PDF-04).
//!
//! Business-Logic-Tier per Service-Tier-Konvention: kombiniert
//! [`crate::shiftplan::ShiftplanViewService`] (Read-Aggregat) +
//! [`crate::sales_person::SalesPersonService`] +
//! [`crate::week_status::WeekStatusService`] + pure Rendering
//! (`service_impl::pdf_render`) zu einer einzigen Assemble-Stelle.
//!
//! Beide zukünftigen Aufrufer — REST-Handler (`GET /shiftplan/{id}/week/{y}/{w}/pdf`, Plan 02)
//! und Scheduler-Refactor (Plan 03) — gehen durch [`PdfShiftplanService::render_week_pdf`].
//!
//! ## Defense-in-Depth: WeekStatus-Gate
//!
//! Der Service prüft `WeekStatusService::get_week_status` und returned
//! [`ServiceError::ValidationError`] wenn der Status NICHT in
//! `{Planned, Locked}` liegt (D-49-06). Der REST-Handler wird denselben
//! Check als 409-Conflict-Pre-Check machen; das Service-Gate deckt
//! Race-Windows und Direct-Impl-Aufrufer (Scheduler) mit ab.
//!
//! ## Context-Weitergabe
//!
//! Der Service leitet den vom Aufrufer übergebenen `context` an alle
//! konsumierten Services weiter (D-49-07); niemals wird intern auf
//! [`crate::permission::Authentication::Full`] hochgehebelt.

use std::fmt::Debug;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::{permission::Authentication, ServiceError};

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait PdfShiftplanService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Rendert die angegebene ISO-Woche des Shiftplans als PDF-Bytes.
    ///
    /// Ablauf (D-49-06):
    /// 1. `WeekStatusService::get_week_status(year, calendar_week, context, tx)` —
    ///    Gate: nur `Planned` und `Locked` sind erlaubt; andernfalls
    ///    [`ServiceError::ValidationError`].
    /// 2. `ShiftplanViewService::get_shiftplan_week(shiftplan_id, year, calendar_week, ...)`.
    /// 3. `SalesPersonService::get_all(...)` und Filter auf `deleted.is_none()`.
    /// 4. `service_impl::pdf_render::render_shiftplan_week_pdf(...)` → `Vec<u8>`.
    ///
    /// Alle Aufrufe reichen `context` weiter (D-49-07); interne Fehler
    /// bubblen unverändert per `?` hoch.
    async fn render_week_pdf(
        &self,
        shiftplan_id: Uuid,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Vec<u8>, ServiceError>;
}
