//! Tests für [`PdfExportSchedulerImpl`] (Phase 48 Plan 04 EXP-01/EXP-03,
//! refactored in Phase 49 Plan 03).
//!
//! Der WebDAV-Upload wird via [`MockWebDavUpload`] gemockt — der Scheduler
//! kennt nur das Trait [`WebDavUpload`], daher reine unit-Tests ohne
//! wiremock-Server für die 6 behavior-Tests.
//!
//! ## Phase 49 Refactor (D-49-08)
//!
//! Der Scheduler delegiert das PDF-Assemble jetzt an
//! [`service::pdf_shiftplan::PdfShiftplanService::render_week_pdf`]. Die
//! frueheren `MockShiftplanViewService` + `MockSalesPersonService`-Expectations
//! sind entfernt und durch `MockPdfShiftplanService::expect_render_week_pdf`
//! ersetzt. Neue Tests decken die Q1-Verhaltensaenderung
//! (`ValidationError` → per-Week-Skip via `record_error`) und den
//! D-49-07-Aufrufer-Kontext (`Authentication::Full`) ab.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use dao::{MockTransaction, MockTransactionDao};
use mockall::predicate::always;
use service::{
    clock::MockClockService,
    pdf_export::PdfExportScheduler,
    pdf_export_config::{MockPdfExportConfigService, PdfExportConfig},
    pdf_shiftplan::MockPdfShiftplanService,
    permission::Authentication,
    shiftplan_catalog::{MockShiftplanService, Shiftplan},
    MockPermissionService, ServiceError, ValidationFailureItem,
};
use uuid::{uuid, Uuid};

use crate::pdf_export_scheduler::{
    PdfExportSchedulerDeps, PdfExportSchedulerImpl, WebDavUploadFactory,
};
use crate::webdav_client::{MockWebDavUpload, WebDavError, WebDavUpload};

// ─── Test Deps ───────────────────────────────────────────────────────────

pub struct TestDeps;

impl PdfExportSchedulerDeps for TestDeps {
    type Context = ();
    type Transaction = MockTransaction;
    type PdfExportConfigService = MockPdfExportConfigService;
    type PdfShiftplanService = MockPdfShiftplanService;
    type ShiftplanService = MockShiftplanService;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type TransactionDao = MockTransactionDao;
}

/// Test-Factory: liefert immer den vorkonfigurierten [`MockWebDavUpload`].
struct FixedFactory {
    upload: Arc<dyn WebDavUpload>,
}

impl WebDavUploadFactory for FixedFactory {
    fn build(
        &self,
        _base_url: &str,
        _user: &str,
        _app_token: &str,
    ) -> Result<Arc<dyn WebDavUpload>, WebDavError> {
        Ok(self.upload.clone())
    }
}

fn base_config(enabled: bool) -> PdfExportConfig {
    PdfExportConfig {
        id: uuid!("00000000-0000-0000-0000-000000000048"),
        enabled,
        nextcloud_url: Some(Arc::from("https://cloud.example.com/remote.php/dav/files/tester")),
        webdav_user: Some(Arc::from("tester")),
        webdav_app_token: Some(Arc::from("app-token-xyz")),
        target_folder: Some(Arc::from("Schichtplaene")),
        weeks_horizon: 2,
        cron_schedule: Arc::from("0 6 * * 1"),
        last_success_at: None,
        last_error_at: None,
        last_error_message: None,
        version: uuid!("00000000-0000-0000-0000-0000ffff0048"),
    }
}

fn active_shiftplan(id: Uuid) -> Shiftplan {
    Shiftplan {
        id,
        name: Arc::from("Planung"),
        is_planning: true,
        deleted: None,
        version: uuid!("11111111-0000-0000-0000-000000000048"),
    }
}

/// Baue einen sinnvollen minimalen PDF-Body (>400 Bytes) fuer Byte-Length-
/// Assertions in den Upload-Erwartungen — der Scheduler leitet die Bytes
/// 1:1 an den WebDAV-Upload weiter.
fn fake_pdf_bytes() -> Vec<u8> {
    let mut v = Vec::with_capacity(512);
    v.extend_from_slice(b"%PDF-1.4\n");
    v.extend_from_slice(&[b'x'; 500]);
    v
}

fn build_scheduler(
    cfg_service: MockPdfExportConfigService,
    pdf_shiftplan_service: MockPdfShiftplanService,
    shiftplan_service: MockShiftplanService,
    perm_service: MockPermissionService,
    clock_service: MockClockService,
    upload_factory: Arc<dyn WebDavUploadFactory>,
) -> PdfExportSchedulerImpl<TestDeps> {
    PdfExportSchedulerImpl::<TestDeps>::new(
        Arc::new(cfg_service),
        Arc::new(pdf_shiftplan_service),
        Arc::new(shiftplan_service),
        Arc::new(perm_service),
        Arc::new(clock_service),
        Arc::new(MockTransactionDao::new()),
        upload_factory,
    )
}

fn full_auth_permission() -> MockPermissionService {
    let mut m = MockPermissionService::new();
    m.expect_check_only_full_authentication()
        .with(always())
        .returning(|context| {
            if context == Authentication::Full {
                Ok(())
            } else {
                Err(service::ServiceError::Forbidden)
            }
        });
    m
}

// ─── Test 1: disabled config skips run ──────────────────────────────────

#[tokio::test]
async fn disabled_config_skips_run() {
    let mut cfg = MockPdfExportConfigService::new();
    cfg.expect_get()
        .returning(|_ctx, _tx| Ok(base_config(false)));
    // record_success and record_error MUST NOT be called
    cfg.expect_record_success().times(0);
    cfg.expect_record_error().times(0);

    let pdf_svc = MockPdfShiftplanService::new();
    let sh = MockShiftplanService::new();
    let clock = MockClockService::new();
    let upload = MockWebDavUpload::new();
    let factory = Arc::new(FixedFactory {
        upload: Arc::new(upload),
    });
    let perm = full_auth_permission();

    let scheduler = build_scheduler(cfg, pdf_svc, sh, perm, clock, factory);
    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("run must not fail when disabled");
}

// ─── Test 2: incomplete config records error ────────────────────────────

#[tokio::test]
async fn incomplete_config_records_error() {
    let mut cfg = MockPdfExportConfigService::new();
    cfg.expect_get().returning(|_ctx, _tx| {
        let mut c = base_config(true);
        c.webdav_app_token = None;
        Ok(c)
    });
    let recorded = Arc::new(AtomicUsize::new(0));
    let r_clone = recorded.clone();
    cfg.expect_record_error()
        .times(1)
        .returning(move |_at, msg, _ctx, _tx| {
            assert!(
                msg.as_ref().contains("unvollständig"),
                "expected 'unvollständig' in message, got: {msg}"
            );
            r_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
    cfg.expect_record_success().times(0);

    let pdf_svc = MockPdfShiftplanService::new();
    let sh = MockShiftplanService::new();
    let mut clock = MockClockService::new();
    clock.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, time::Month::June, 1).unwrap(),
            time::Time::from_hms(6, 0, 0).unwrap(),
        )
    });

    let upload = MockWebDavUpload::new();
    let factory = Arc::new(FixedFactory {
        upload: Arc::new(upload),
    });
    let perm = full_auth_permission();

    let scheduler = build_scheduler(cfg, pdf_svc, sh, perm, clock, factory);
    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("run must be Ok even when config incomplete");
    assert_eq!(recorded.load(Ordering::SeqCst), 1);
}

// ─── Test 3: happy path renders horizon and uploads ─────────────────────

#[tokio::test]
async fn happy_path_renders_horizon_and_uploads() {
    let shiftplan_id = uuid!("aaaa1111-0000-0000-0000-000000000048");

    let mut cfg = MockPdfExportConfigService::new();
    cfg.expect_get().returning(move |_ctx, _tx| {
        let mut c = base_config(true);
        c.weeks_horizon = 2;
        Ok(c)
    });
    let success_count = Arc::new(AtomicUsize::new(0));
    let sc = success_count.clone();
    cfg.expect_record_success()
        .times(1)
        .returning(move |_at, _ctx, _tx| {
            sc.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
    cfg.expect_record_error().times(0);

    let mut pdf_svc = MockPdfShiftplanService::new();
    pdf_svc
        .expect_render_week_pdf()
        .times(2)
        .returning(|_id, _y, _w, _ctx, _tx| Ok(fake_pdf_bytes()));

    let mut sh = MockShiftplanService::new();
    sh.expect_get_all().returning(move |_ctx, _tx| {
        Ok(Arc::from(vec![active_shiftplan(shiftplan_id)]))
    });

    let mut clock = MockClockService::new();
    clock.expect_date_now().returning(|| {
        // 2026-07-01 is a Wednesday in ISO week 27.
        time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap()
    });
    clock.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap(),
            time::Time::from_hms(6, 0, 0).unwrap(),
        )
    });

    let mut upload = MockWebDavUpload::new();
    let uploaded_paths = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let up_clone = uploaded_paths.clone();
    upload
        .expect_upload_file()
        .times(2)
        .returning(move |folder, filename, bytes| {
            assert_eq!(folder, "Schichtplaene");
            assert!(bytes.len() > 400, "PDF bytes too short: {}", bytes.len());
            up_clone
                .lock()
                .unwrap()
                .push(format!("{folder}/{filename}"));
            Ok(())
        });
    let factory = Arc::new(FixedFactory {
        upload: Arc::new(upload),
    });

    let perm = full_auth_permission();
    let scheduler = build_scheduler(cfg, pdf_svc, sh, perm, clock, factory);
    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("happy-path run must succeed");
    assert_eq!(success_count.load(Ordering::SeqCst), 1);

    let paths = uploaded_paths.lock().unwrap();
    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0], "Schichtplaene/schichtplan-2026-KW27.pdf");
    assert_eq!(paths[1], "Schichtplaene/schichtplan-2026-KW28.pdf");
}

// ─── Test 4: transient webdav after 3 retries records error ─────────────

#[tokio::test]
async fn webdav_transient_fail_after_3_retries_records_error() {
    let shiftplan_id = uuid!("aaaa2222-0000-0000-0000-000000000048");

    let mut cfg = MockPdfExportConfigService::new();
    cfg.expect_get().returning(move |_ctx, _tx| {
        let mut c = base_config(true);
        c.weeks_horizon = 2;
        Ok(c)
    });
    let error_count = Arc::new(AtomicUsize::new(0));
    let ec = error_count.clone();
    cfg.expect_record_error()
        .times(1)
        .returning(move |_at, msg, _ctx, _tx| {
            assert!(msg.as_ref().contains("transient"), "expected 'transient' in msg, got: {msg}");
            assert!(msg.as_ref().contains("KW27"), "expected 'KW27' in msg, got: {msg}");
            ec.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
    cfg.expect_record_success().times(0);

    let mut pdf_svc = MockPdfShiftplanService::new();
    // Only the first week is attempted (we bail on first upload failure).
    pdf_svc
        .expect_render_week_pdf()
        .times(1)
        .returning(|_id, _y, _w, _ctx, _tx| Ok(fake_pdf_bytes()));

    let mut sh = MockShiftplanService::new();
    sh.expect_get_all().returning(move |_ctx, _tx| {
        Ok(Arc::from(vec![active_shiftplan(shiftplan_id)]))
    });

    let mut clock = MockClockService::new();
    clock.expect_date_now().returning(|| {
        time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap()
    });
    clock.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap(),
            time::Time::from_hms(6, 0, 0).unwrap(),
        )
    });

    let mut upload = MockWebDavUpload::new();
    upload
        .expect_upload_file()
        .times(1)
        .returning(|_folder, _filename, _bytes| {
            Err(WebDavError::Transient {
                attempts: 3,
                reason: Arc::from("simulated 503 x3"),
            })
        });
    let factory = Arc::new(FixedFactory {
        upload: Arc::new(upload),
    });

    let perm = full_auth_permission();
    let scheduler = build_scheduler(cfg, pdf_svc, sh, perm, clock, factory);
    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("run must be Ok even after transient upload failure");
    assert_eq!(error_count.load(Ordering::SeqCst), 1);
}

// ─── Test 5: permanent 401 records error immediately ────────────────────

#[tokio::test]
async fn permanent_401_records_error_immediately() {
    let shiftplan_id = uuid!("aaaa3333-0000-0000-0000-000000000048");

    let mut cfg = MockPdfExportConfigService::new();
    cfg.expect_get().returning(move |_ctx, _tx| {
        let mut c = base_config(true);
        c.weeks_horizon = 1;
        Ok(c)
    });
    let ec = Arc::new(AtomicUsize::new(0));
    let ec_clone = ec.clone();
    cfg.expect_record_error()
        .times(1)
        .returning(move |_at, msg, _ctx, _tx| {
            assert!(msg.as_ref().contains("401"), "expected '401' in msg, got: {msg}");
            ec_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
    cfg.expect_record_success().times(0);

    let mut pdf_svc = MockPdfShiftplanService::new();
    pdf_svc
        .expect_render_week_pdf()
        .times(1)
        .returning(|_id, _y, _w, _ctx, _tx| Ok(fake_pdf_bytes()));

    let mut sh = MockShiftplanService::new();
    sh.expect_get_all().returning(move |_ctx, _tx| {
        Ok(Arc::from(vec![active_shiftplan(shiftplan_id)]))
    });

    let mut clock = MockClockService::new();
    clock.expect_date_now().returning(|| {
        time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap()
    });
    clock.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap(),
            time::Time::from_hms(6, 0, 0).unwrap(),
        )
    });

    let mut upload = MockWebDavUpload::new();
    upload
        .expect_upload_file()
        .times(1)
        .returning(|_f, _n, _b| {
            Err(WebDavError::Permanent {
                status: 401,
                body: Arc::from("unauthorized"),
            })
        });
    let factory = Arc::new(FixedFactory {
        upload: Arc::new(upload),
    });

    let perm = full_auth_permission();
    let scheduler = build_scheduler(cfg, pdf_svc, sh, perm, clock, factory);
    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("run must be Ok even after permanent 401");
    assert_eq!(ec.load(Ordering::SeqCst), 1);
}

// ─── Test 6: year-week wraps correctly (2026-KW53 → 2027-KW01) ──────────

#[tokio::test]
async fn year_week_wraps_correctly() {
    let shiftplan_id = uuid!("aaaa4444-0000-0000-0000-000000000048");

    let mut cfg = MockPdfExportConfigService::new();
    cfg.expect_get().returning(move |_ctx, _tx| {
        let mut c = base_config(true);
        c.weeks_horizon = 2;
        Ok(c)
    });
    cfg.expect_record_success()
        .times(1)
        .returning(|_at, _ctx, _tx| Ok(()));
    cfg.expect_record_error().times(0);

    let requested = Arc::new(std::sync::Mutex::new(Vec::<(u32, u8)>::new()));
    let req_clone = requested.clone();
    let mut pdf_svc = MockPdfShiftplanService::new();
    pdf_svc
        .expect_render_week_pdf()
        .times(2)
        .returning(move |_id, year, week, _ctx, _tx| {
            req_clone.lock().unwrap().push((year, week));
            Ok(fake_pdf_bytes())
        });

    let mut sh = MockShiftplanService::new();
    sh.expect_get_all().returning(move |_ctx, _tx| {
        Ok(Arc::from(vec![active_shiftplan(shiftplan_id)]))
    });

    let mut clock = MockClockService::new();
    // 2026-12-31 is in ISO week 53 (2026 has 53 ISO weeks).
    clock.expect_date_now().returning(|| {
        time::Date::from_calendar_date(2026, time::Month::December, 31).unwrap()
    });
    clock.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, time::Month::December, 31).unwrap(),
            time::Time::from_hms(6, 0, 0).unwrap(),
        )
    });

    let mut upload = MockWebDavUpload::new();
    upload
        .expect_upload_file()
        .times(2)
        .returning(|_f, _n, _b| Ok(()));
    let factory = Arc::new(FixedFactory {
        upload: Arc::new(upload),
    });

    let perm = full_auth_permission();
    let scheduler = build_scheduler(cfg, pdf_svc, sh, perm, clock, factory);
    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("year-wrap run must succeed");

    let req = requested.lock().unwrap();
    assert_eq!(req.len(), 2);
    // First week must be 2026-53 (from 2026-12-31 ISO date), second must
    // wrap to 2027-1.
    assert_eq!(req[0], (2026, 53));
    assert_eq!(req[1], (2027, 1));
}

// ─── Test 7: scheduler calls PdfShiftplanService with Authentication::Full ───

/// D-49-07 (Scheduler-Kontext): Der Scheduler ruft `render_week_pdf` mit
/// `Authentication::Full` — er ist der trusted caller, keine User-Session
/// im Cron-Callback. Der `.withf(...)`-Predicate matcht nur, wenn der
/// context-Parameter exakt `Full` ist; jede andere Variante liesse die
/// Erwartung fehlschlagen.
#[tokio::test]
async fn scheduler_calls_pdf_shiftplan_service_with_full_auth() {
    let shiftplan_id = uuid!("aaaa5555-0000-0000-0000-000000000048");

    let mut cfg = MockPdfExportConfigService::new();
    cfg.expect_get().returning(move |_ctx, _tx| {
        let mut c = base_config(true);
        c.weeks_horizon = 1;
        Ok(c)
    });
    cfg.expect_record_success()
        .times(1)
        .returning(|_at, _ctx, _tx| Ok(()));
    cfg.expect_record_error().times(0);

    let mut pdf_svc = MockPdfShiftplanService::new();
    pdf_svc
        .expect_render_week_pdf()
        .times(1)
        .withf(|_id, _y, _w, ctx, _tx| matches!(ctx, Authentication::Full))
        .returning(|_id, _y, _w, _ctx, _tx| Ok(fake_pdf_bytes()));

    let mut sh = MockShiftplanService::new();
    sh.expect_get_all().returning(move |_ctx, _tx| {
        Ok(Arc::from(vec![active_shiftplan(shiftplan_id)]))
    });

    let mut clock = MockClockService::new();
    clock.expect_date_now().returning(|| {
        time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap()
    });
    clock.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap(),
            time::Time::from_hms(6, 0, 0).unwrap(),
        )
    });

    let mut upload = MockWebDavUpload::new();
    upload
        .expect_upload_file()
        .times(1)
        .returning(|_f, _n, _b| Ok(()));
    let factory = Arc::new(FixedFactory {
        upload: Arc::new(upload),
    });

    let perm = full_auth_permission();
    let scheduler = build_scheduler(cfg, pdf_svc, sh, perm, clock, factory);
    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("full-auth-context run must succeed");
}

// ─── Test 8: scheduler skips week on ValidationError (Q1 semantic) ──────

/// Q1-Verhalten (D-49-08): Wenn der PdfShiftplanService fuer eine Woche
/// einen `ServiceError::ValidationError` liefert (z.B. weil WeekStatus nicht
/// in {Planned, Locked} ist), MUSS der Scheduler diesen als per-Week-Skip
/// behandeln — `record_error` einmal aufrufen und dann `Ok(())` returnen.
/// Kein Panic, kein Fehler-Boot, kein Abbruch der gesamten Cron-Run
/// (der bestehende `return Ok(())`-Pfad wird respektiert).
#[tokio::test]
async fn scheduler_skips_week_on_validation_error() {
    let shiftplan_id = uuid!("aaaa6666-0000-0000-0000-000000000048");

    let mut cfg = MockPdfExportConfigService::new();
    cfg.expect_get().returning(move |_ctx, _tx| {
        let mut c = base_config(true);
        c.weeks_horizon = 1;
        Ok(c)
    });
    let error_count = Arc::new(AtomicUsize::new(0));
    let ec = error_count.clone();
    cfg.expect_record_error()
        .times(1)
        .returning(move |_at, msg, _ctx, _tx| {
            assert!(
                msg.as_ref().contains("KW27"),
                "expected 'KW27' in msg, got: {msg}"
            );
            assert!(
                msg.as_ref().contains("Assemble") || msg.as_ref().contains("Render"),
                "expected assemble/render marker in msg, got: {msg}"
            );
            ec.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
    cfg.expect_record_success().times(0);

    let mut pdf_svc = MockPdfShiftplanService::new();
    pdf_svc
        .expect_render_week_pdf()
        .times(1)
        .returning(|_id, _y, _w, _ctx, _tx| {
            Err(ServiceError::ValidationError(Arc::from([
                ValidationFailureItem::InvalidValue(Arc::from(
                    "Woche KW27/2026 ist im Status Unset — kein Download",
                )),
            ])))
        });

    let mut sh = MockShiftplanService::new();
    sh.expect_get_all().returning(move |_ctx, _tx| {
        Ok(Arc::from(vec![active_shiftplan(shiftplan_id)]))
    });

    let mut clock = MockClockService::new();
    clock.expect_date_now().returning(|| {
        time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap()
    });
    clock.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap(),
            time::Time::from_hms(6, 0, 0).unwrap(),
        )
    });

    // Upload MUST NOT be called for a skipped week.
    let mut upload = MockWebDavUpload::new();
    upload.expect_upload_file().times(0);
    let factory = Arc::new(FixedFactory {
        upload: Arc::new(upload),
    });

    let perm = full_auth_permission();
    let scheduler = build_scheduler(cfg, pdf_svc, sh, perm, clock, factory);
    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("run must be Ok even when service returns ValidationError");
    assert_eq!(error_count.load(Ordering::SeqCst), 1);
}

// ═════════════════════════════════════════════════════════════════════════
// End-to-End Integration Test: boot_trigger_reload_flow
// ═════════════════════════════════════════════════════════════════════════
//
// Der Scheduler bindet mit `Transaction = Self::Transaction` alle
// Domain-Deps an einen einzigen Transaction-Type. Mocks (`#[automock]`)
// verwenden `MockTransaction`, echte Services `TransactionImpl` — Mischbetrieb
// ist damit typ-inkompatibel. Der End-to-End-Test bleibt daher mock-basiert
// für die Domain-Deps und verifiziert stattdessen alle relevanten
// end-to-end-Aspekte:
//   1. Config aus einem stateful Mock (emuliert PdfExportConfigService).
//   2. run_once_now durchläuft die volle Sequenz (get → render → upload).
//   3. Upload landet mit korrektem Filename + Body-Länge beim Mock.
//   4. record_success wird nach erfolgreichem Lauf aufgerufen (Persistenz-
//      Sichtbarkeit).
// Die echte SQLite-Persistenz von `last_success_at` ist bereits in
// `test::pdf_export_config::record_success_and_record_error_persist` gedeckt.

#[tokio::test]
async fn boot_trigger_reload_flow() {
    let shiftplan_id = uuid!("cccc0001-0000-0000-0000-000000000048");

    // Config-Mock: liefert enabled=true mit vollständiger Konfig; zählt
    // record_success.
    let mut cfg = MockPdfExportConfigService::new();
    cfg.expect_get().returning(move |_ctx, _tx| {
        let mut c = base_config(true);
        c.weeks_horizon = 1;
        c.target_folder = Some(Arc::from("Schichtplaene"));
        Ok(c)
    });
    let recorded_success = Arc::new(AtomicUsize::new(0));
    let rs = recorded_success.clone();
    cfg.expect_record_success()
        .times(1)
        .returning(move |_at, _ctx, _tx| {
            rs.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
    cfg.expect_record_error().times(0);

    // Sammle Uploads
    let uploads = Arc::new(std::sync::Mutex::new(Vec::<(String, String, usize)>::new()));
    let up_clone = uploads.clone();
    let mut upload = MockWebDavUpload::new();
    upload
        .expect_upload_file()
        .times(1)
        .returning(move |folder, filename, bytes| {
            up_clone
                .lock()
                .unwrap()
                .push((folder.to_string(), filename.to_string(), bytes.len()));
            Ok(())
        });
    let factory = Arc::new(FixedFactory {
        upload: Arc::new(upload),
    });

    let mut pdf_svc = MockPdfShiftplanService::new();
    pdf_svc
        .expect_render_week_pdf()
        .times(1)
        .returning(|_id, _y, _w, _ctx, _tx| Ok(fake_pdf_bytes()));
    let mut sh = MockShiftplanService::new();
    sh.expect_get_all().returning(move |_ctx, _tx| {
        Ok(Arc::from(vec![active_shiftplan(shiftplan_id)]))
    });

    let mut clock = MockClockService::new();
    clock.expect_date_now().returning(|| {
        time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap()
    });
    clock.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap(),
            time::Time::from_hms(6, 0, 0).unwrap(),
        )
    });

    let perm = full_auth_permission();

    let scheduler = build_scheduler(cfg, pdf_svc, sh, perm, clock, factory);
    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("e2e run must succeed");

    assert_eq!(recorded_success.load(Ordering::SeqCst), 1);
    let ups = uploads.lock().unwrap();
    assert_eq!(ups.len(), 1);
    let (folder, filename, body_len) = &ups[0];
    assert_eq!(folder, "Schichtplaene");
    // ISO week of 2026-07-01 is 27.
    assert_eq!(filename, "schichtplan-2026-KW27.pdf");
    // A minimal empty-week PDF must still exceed the 400-byte floor
    // (header + xref + trailer + metadata).
    assert!(*body_len > 400, "PDF too short: {body_len}");
}
