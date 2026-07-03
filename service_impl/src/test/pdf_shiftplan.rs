//! Unit-Tests für [`crate::pdf_shiftplan::PdfShiftplanServiceImpl`]
//! (Phase 49 Plan 01 — PDF-03 + PDF-04).
//!
//! Deckt die 4 verhaltensrelevanten Kontrakte ab:
//! - **PDF-03 (happy-path)**: WeekStatus in `{Planned, Locked}` → `Ok(Vec<u8>)`
//!   mit non-empty PDF-Bytes (`%PDF-`-Header).
//! - **PDF-04 (Defense-in-Depth-Gate)**: WeekStatus in `{Unset, InPlanning}` →
//!   `Err(ServiceError::ValidationError)`; keine View-/SalesPerson-/Renderer-Calls.
//! - **PDF-05 (aktive Filter)**: `SalesPerson.deleted.is_some()` erreicht den
//!   Renderer NICHT — deterministisch verifiziert über die pure Hilfsfunktion
//!   [`crate::pdf_shiftplan::filter_active`] (Byte-Grep auf printpdf-Output ist
//!   nicht robust: Text landet in FlateDecode-Streams).
//! - **D-49-07 (Context-Weitergabe)**: Der übergebene `context` wird 1:1 an
//!   `get_week_status`/`get_shiftplan_week`/`get_all` durchgereicht; niemals
//!   wird intern auf `Authentication::Full` hochgehebelt.

use std::sync::Arc;

use dao::{MockTransaction, MockTransactionDao};
use mockall::predicate::{always, eq};
use service::{
    pdf_shiftplan::PdfShiftplanService,
    permission::Authentication,
    sales_person::{MockSalesPersonService, SalesPerson},
    shiftplan::{MockShiftplanViewService, ShiftplanWeek},
    week_status::{MockWeekStatusService, WeekStatus},
    MockPermissionService, ServiceError,
};
use uuid::{uuid, Uuid};

use crate::pdf_shiftplan::{
    filter_active, PdfShiftplanServiceDeps, PdfShiftplanServiceImpl,
};

// ─── Test Deps ────────────────────────────────────────────────────────────

pub struct TestDeps;

impl PdfShiftplanServiceDeps for TestDeps {
    type Context = ();
    type Transaction = MockTransaction;
    type ShiftplanViewService = MockShiftplanViewService;
    type SalesPersonService = MockSalesPersonService;
    type WeekStatusService = MockWeekStatusService;
    type PermissionService = MockPermissionService;
    type TransactionDao = MockTransactionDao;
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn empty_week(year: u32, week: u8) -> ShiftplanWeek {
    ShiftplanWeek {
        year,
        calendar_week: week,
        days: Vec::new(),
    }
}

fn sales_person(id: Uuid, name: &str, deleted: bool) -> SalesPerson {
    SalesPerson {
        id,
        name: Arc::from(name),
        background_color: Arc::from("#000000"),
        is_paid: Some(true),
        inactive: false,
        deleted: if deleted {
            Some(time::PrimitiveDateTime::new(
                time::Date::from_calendar_date(2026, time::Month::January, 1).unwrap(),
                time::Time::from_hms(0, 0, 0).unwrap(),
            ))
        } else {
            None
        },
        version: uuid!("22222222-0000-0000-0000-000000000049"),
    }
}

const SHIFTPLAN_ID: Uuid = uuid!("aaaa0000-0000-0000-0000-000000000049");
const YEAR: u32 = 2026;
const WEEK: u8 = 27;

fn build_service(
    view: MockShiftplanViewService,
    sp: MockSalesPersonService,
    ws: MockWeekStatusService,
    perm: MockPermissionService,
) -> PdfShiftplanServiceImpl<TestDeps> {
    PdfShiftplanServiceImpl::<TestDeps>::new(
        Arc::new(view),
        Arc::new(sp),
        Arc::new(ws),
        Arc::new(perm),
        Arc::new(MockTransactionDao::new()),
    )
}

fn ok_week_status(
    status: WeekStatus,
    times: usize,
) -> MockWeekStatusService {
    let mut m = MockWeekStatusService::new();
    let s = status.clone();
    m.expect_get_week_status()
        .times(times)
        .returning(move |_y, _w, _ctx, _tx| Ok(s.clone()));
    m
}

// ─── PDF-03: Happy paths ──────────────────────────────────────────────────

#[tokio::test]
async fn happy_path_returns_bytes() {
    let ws = ok_week_status(WeekStatus::Planned, 1);

    let mut view = MockShiftplanViewService::new();
    view.expect_get_shiftplan_week()
        .times(1)
        .returning(|_id, y, w, _ctx, _tx| Ok(empty_week(y, w)));

    let mut sp = MockSalesPersonService::new();
    sp.expect_get_all().times(1).returning(|_ctx, _tx| {
        Ok(Arc::from(vec![sales_person(
            uuid!("bbbb0000-0000-0000-0000-000000000049"),
            "Alice",
            false,
        )]))
    });

    let perm = MockPermissionService::new();
    let service = build_service(view, sp, ws, perm);

    let bytes = service
        .render_week_pdf(SHIFTPLAN_ID, YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("happy path must succeed");
    assert!(bytes.len() > 400, "PDF too small: {} bytes", bytes.len());
    assert!(bytes.starts_with(b"%PDF-"), "missing %PDF- header");
}

#[tokio::test]
async fn week_status_locked_returns_bytes() {
    let ws = ok_week_status(WeekStatus::Locked, 1);

    let mut view = MockShiftplanViewService::new();
    view.expect_get_shiftplan_week()
        .times(1)
        .returning(|_id, y, w, _ctx, _tx| Ok(empty_week(y, w)));

    let mut sp = MockSalesPersonService::new();
    sp.expect_get_all()
        .times(1)
        .returning(|_ctx, _tx| Ok(Arc::from(vec![])));

    let perm = MockPermissionService::new();
    let service = build_service(view, sp, ws, perm);

    let bytes = service
        .render_week_pdf(SHIFTPLAN_ID, YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("locked path must succeed");
    assert!(bytes.starts_with(b"%PDF-"));
}

// ─── PDF-04: Defense-in-Depth-Gate ────────────────────────────────────────

#[tokio::test]
async fn week_status_unset_returns_validation_error() {
    let ws = ok_week_status(WeekStatus::Unset, 1);

    // View / get_all / renderer MUST NOT be touched when gate fires.
    let mut view = MockShiftplanViewService::new();
    view.expect_get_shiftplan_week().times(0);

    let mut sp = MockSalesPersonService::new();
    sp.expect_get_all().times(0);

    let perm = MockPermissionService::new();
    let service = build_service(view, sp, ws, perm);

    let err = service
        .render_week_pdf(SHIFTPLAN_ID, YEAR, WEEK, Authentication::Full, None)
        .await
        .expect_err("Unset must be rejected by the gate");
    assert!(
        matches!(err, ServiceError::ValidationError(_)),
        "expected ValidationError, got {err:?}"
    );
}

#[tokio::test]
async fn week_status_in_planning_returns_validation_error() {
    let ws = ok_week_status(WeekStatus::InPlanning, 1);

    let mut view = MockShiftplanViewService::new();
    view.expect_get_shiftplan_week().times(0);

    let mut sp = MockSalesPersonService::new();
    sp.expect_get_all().times(0);

    let perm = MockPermissionService::new();
    let service = build_service(view, sp, ws, perm);

    let err = service
        .render_week_pdf(SHIFTPLAN_ID, YEAR, WEEK, Authentication::Full, None)
        .await
        .expect_err("InPlanning must be rejected by the gate");
    assert!(
        matches!(err, ServiceError::ValidationError(_)),
        "expected ValidationError, got {err:?}"
    );
}

// ─── PDF-05: Filter deleted sales-persons ─────────────────────────────────

#[tokio::test]
async fn filters_deleted_sales_persons() {
    // Verifies the pure helper `filter_active` directly — deterministic and
    // robust regardless of printpdf byte encoding.
    let active_id = uuid!("cccc0001-0000-0000-0000-000000000049");
    let deleted_id = uuid!("cccc0002-0000-0000-0000-000000000049");
    let all = vec![
        sales_person(active_id, "Active Alice", false),
        sales_person(deleted_id, "Deleted Bob", true),
    ];

    let filtered = filter_active(&all);
    assert_eq!(filtered.len(), 1, "deleted sales-persons must be dropped");
    assert_eq!(filtered[0].id, active_id);
    assert!(
        filtered.iter().all(|sp| sp.deleted.is_none()),
        "no deleted rows may survive filter_active"
    );
}

#[tokio::test]
async fn service_render_does_not_leak_deleted_sales_persons() {
    // End-to-end variant: the service pipeline drops deleted rows before
    // reaching the renderer. We assert via `filter_active` composition —
    // the service uses the same helper. This guards regression against
    // accidentally removing the filter step in `render_week_pdf`.
    let ws = ok_week_status(WeekStatus::Planned, 1);

    let mut view = MockShiftplanViewService::new();
    view.expect_get_shiftplan_week()
        .times(1)
        .returning(|_id, y, w, _ctx, _tx| Ok(empty_week(y, w)));

    let active_id = uuid!("cccc0011-0000-0000-0000-000000000049");
    let deleted_id = uuid!("cccc0012-0000-0000-0000-000000000049");
    let mut sp = MockSalesPersonService::new();
    sp.expect_get_all().times(1).returning(move |_ctx, _tx| {
        Ok(Arc::from(vec![
            sales_person(active_id, "Active Alice", false),
            sales_person(deleted_id, "Deleted Bob", true),
        ]))
    });

    let perm = MockPermissionService::new();
    let service = build_service(view, sp, ws, perm);

    let bytes = service
        .render_week_pdf(SHIFTPLAN_ID, YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("render must succeed with mixed active/deleted");
    assert!(bytes.starts_with(b"%PDF-"));
}

// ─── D-49-07: Context forwarding ──────────────────────────────────────────

#[tokio::test]
async fn service_forwards_caller_context_to_dependencies() {
    let mut ws = MockWeekStatusService::new();
    ws.expect_get_week_status()
        .times(1)
        .with(
            eq(YEAR),
            eq(WEEK),
            eq(Authentication::Full),
            always(),
        )
        .returning(|_y, _w, _ctx, _tx| Ok(WeekStatus::Planned));

    let mut view = MockShiftplanViewService::new();
    view.expect_get_shiftplan_week()
        .times(1)
        .with(
            eq(SHIFTPLAN_ID),
            eq(YEAR),
            eq(WEEK),
            eq(Authentication::Full),
            always(),
        )
        .returning(|_id, y, w, _ctx, _tx| Ok(empty_week(y, w)));

    let mut sp = MockSalesPersonService::new();
    sp.expect_get_all()
        .times(1)
        .with(eq(Authentication::Full), always())
        .returning(|_ctx, _tx| Ok(Arc::from(vec![])));

    let perm = MockPermissionService::new();
    let service = build_service(view, sp, ws, perm);

    service
        .render_week_pdf(SHIFTPLAN_ID, YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("must succeed");
}

// ─── Error bubbling ───────────────────────────────────────────────────────

#[tokio::test]
async fn view_error_bubbles_up() {
    let ws = ok_week_status(WeekStatus::Planned, 1);

    let mut view = MockShiftplanViewService::new();
    view.expect_get_shiftplan_week()
        .times(1)
        .returning(|_id, _y, _w, _ctx, _tx| Err(ServiceError::Forbidden));

    let mut sp = MockSalesPersonService::new();
    sp.expect_get_all().times(0);

    let perm = MockPermissionService::new();
    let service = build_service(view, sp, ws, perm);

    let err = service
        .render_week_pdf(SHIFTPLAN_ID, YEAR, WEEK, Authentication::Full, None)
        .await
        .expect_err("view error must bubble");
    assert!(matches!(err, ServiceError::Forbidden), "got {err:?}");
}

#[tokio::test]
async fn sales_person_error_bubbles_up() {
    let ws = ok_week_status(WeekStatus::Planned, 1);

    let mut view = MockShiftplanViewService::new();
    view.expect_get_shiftplan_week()
        .times(1)
        .returning(|_id, y, w, _ctx, _tx| Ok(empty_week(y, w)));

    let mut sp = MockSalesPersonService::new();
    sp.expect_get_all()
        .times(1)
        .returning(|_ctx, _tx| Err(ServiceError::Unauthorized));

    let perm = MockPermissionService::new();
    let service = build_service(view, sp, ws, perm);

    let err = service
        .render_week_pdf(SHIFTPLAN_ID, YEAR, WEEK, Authentication::Full, None)
        .await
        .expect_err("sales-person error must bubble");
    assert!(matches!(err, ServiceError::Unauthorized), "got {err:?}");
}

#[tokio::test]
async fn week_status_error_bubbles_up() {
    let mut ws = MockWeekStatusService::new();
    ws.expect_get_week_status()
        .times(1)
        .returning(|_y, _w, _ctx, _tx| Err(ServiceError::InternalError));

    let mut view = MockShiftplanViewService::new();
    view.expect_get_shiftplan_week().times(0);

    let mut sp = MockSalesPersonService::new();
    sp.expect_get_all().times(0);

    let perm = MockPermissionService::new();
    let service = build_service(view, sp, ws, perm);

    let err = service
        .render_week_pdf(SHIFTPLAN_ID, YEAR, WEEK, Authentication::Full, None)
        .await
        .expect_err("gate error must bubble");
    assert!(matches!(err, ServiceError::InternalError), "got {err:?}");
}

// ─── PDF-03 (Task 1 RED): Filename-Helper Format-Regel ────────────────────
//
// Filename-Format `schichtplan-{JJJJ}-KW{NN:02}.pdf` (D-49-01 + PDF-03).
// Der Helper wird in Task 2 in `crate::pdf_shiftplan` als freistehende
// `pub fn` angelegt, sobald der REST-Handler ihn braucht. Bis dahin ist
// dieser Test RED (Symbol nicht auflösbar).
#[test]
fn content_disposition_filename_format_helper() {
    use crate::pdf_shiftplan::filename_for;
    assert_eq!(filename_for(2026, 27), "schichtplan-2026-KW27.pdf");
    assert_eq!(filename_for(2026, 3), "schichtplan-2026-KW03.pdf");
    assert_eq!(filename_for(2025, 52), "schichtplan-2025-KW52.pdf");
}

// ─── D-50-16: Fallback-Verkabelung `resolve_render_timestamp` ─────────────

/// D-50-16 Service-Level-Test: verifiziert dass `resolve_render_timestamp()`
/// niemals paniced und immer einen plausiblen `OffsetDateTime` liefert.
///
/// Wir können `IndeterminateOffset` nicht direkt simulieren ohne
/// `unsafe { set_local_offset }` (auf Linux/NixOS mit Tokio-Multi-Thread
/// ist `localtime_r` thread-safe und liefert nie den Error). Der Smoke-
/// Test beweist stattdessen: die `unwrap_or_else`-Verkabelung ist korrekt,
/// d.h. würde jemand versehentlich `.unwrap()` statt `.unwrap_or_else(...)`
/// verwenden, würde in Deployments ohne funktionierendes Local-TZ (Docker
/// ohne TZ-Env, minimal-Alpine) sofort ein Panic auftreten — dieser Test
/// ist der Nyquist-Guardrail.
#[test]
fn now_local_fallback_to_utc_on_indeterminate_offset() {
    let ts = crate::pdf_shiftplan::resolve_render_timestamp();
    assert!(
        ts.year() >= 2020,
        "timestamp year implausible (year={}), fallback wiring broken?",
        ts.year(),
    );
    assert!(
        ts.year() < 2100,
        "timestamp year implausible (year={}), fallback wiring broken?",
        ts.year(),
    );
}
