//! Phase 51 Chain D — `ShiftplanReportService` Aggregation + Clip + Gate + Snapshot-Immunity.
//!
//! Diese Datei pinnt den Rust-Layer-Refactor aus D-51-08:
//! Der DAO liefert Roh-Rows pro Booking (`ShiftplanReportRawRow`), der Service
//! aggregiert nach `(sales_person_id, year, week, day_of_week)` und wendet
//! `shortday_gate::clip_slot_for_week` (Slot::clip_to + Stichtag-Gate) pro Row
//! an. Testfälle A–G decken:
//!
//! - A: Clip greift (14:00–15:00, Cutoff 14:30, Gate aktiv → 0,5h).
//! - B: Gate inaktiv (active_from = None → 1,0h Legacy).
//! - C: Slot komplett hinter Cutoff (D-04 Zeile 3 → 0h).
//! - D: Stichtag-Grenzfall (booking_date == active_from ⇔ inklusiv).
//! - E: Zwei Bookings dieselbe Slot-Zeit → summiert (SHC-05 kein Rewrite).
//! - F: Multi-Wochen-Range mit gemischtem Gate.
//! - G: `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt bei 12 (Snapshot-Immunität).

use std::sync::Arc;

use dao::shiftplan_report::{MockShiftplanReportDao, ShiftplanReportRawRow};
use service::permission::Authentication;
use service::shiftplan_report::ShiftplanReportService;
use service::special_days::{MockSpecialDayService, SpecialDay, SpecialDayType};
use service::toggle::MockToggleService;
use shifty_utils::{DayOfWeek, ShiftyDate};
use time::macros::datetime;
use uuid::Uuid;

use crate::shiftplan_report::{ShiftplanReportServiceDeps, ShiftplanReportServiceImpl};

// ─── TestDeps ─────────────────────────────────────────────────────────────────

struct TestDeps;

impl ShiftplanReportServiceDeps for TestDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type ShiftplanReportDao = MockShiftplanReportDao;
    type SpecialDayService = MockSpecialDayService;
    type ToggleService = MockToggleService;
    type TransactionDao = dao::MockTransactionDao;
}

// ─── Fixture-Konstanten ───────────────────────────────────────────────────────

const YEAR: u32 = 2026;
/// 2026-W31 (Mo = 2026-07-27 ISO).
const WEEK: u8 = 31;

// ─── Fixture-Helpers ──────────────────────────────────────────────────────────

fn raw_row(
    sales_person_id: Uuid,
    year: u32,
    week: u8,
    dow: DayOfWeek,
    from: time::Time,
    to: time::Time,
) -> ShiftplanReportRawRow {
    ShiftplanReportRawRow {
        sales_person_id,
        booking_id: Uuid::new_v4(),
        year,
        calendar_week: week,
        day_of_week: dow,
        time_from: from,
        time_to: to,
    }
}

fn shortday(dow: DayOfWeek, cutoff_at: time::Time, year: u32, week: u8) -> SpecialDay {
    SpecialDay {
        id: Uuid::nil(),
        year,
        calendar_week: week,
        day_of_week: dow,
        day_type: SpecialDayType::ShortDay,
        time_of_day: Some(cutoff_at),
        created: Some(datetime!(2026 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}

/// Baut einen Service, der `extract_raw_shiftplan_report_for_week` mit `rows` beantwortet
/// und für die Ziel-Woche `WEEK` die `special_days`-Liste ausliefert. Toggle-Wert konfigurierbar.
fn build_service_for_week(
    rows: Vec<ShiftplanReportRawRow>,
    special_days_for_target: Vec<SpecialDay>,
    toggle_active_from: Option<&'static str>,
) -> ShiftplanReportServiceImpl<TestDeps> {
    let rows_arc: Arc<[ShiftplanReportRawRow]> = Arc::from(rows);
    let mut shiftplan_report_dao = MockShiftplanReportDao::new();
    let rows_clone = rows_arc.clone();
    shiftplan_report_dao
        .expect_extract_raw_shiftplan_report_for_week()
        .returning(move |year, week, _| {
            if year == YEAR && week == WEEK {
                Ok(rows_clone.clone())
            } else {
                Ok(Arc::from(Vec::<ShiftplanReportRawRow>::new()))
            }
        });

    let sd_clone = special_days_for_target.clone();
    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(move |year, week, _| {
            if year == YEAR && week == WEEK {
                Ok(Arc::from(sd_clone.clone()))
            } else {
                Ok(Arc::from(Vec::<SpecialDay>::new()))
            }
        });

    let toggle_val: Option<String> = toggle_active_from.map(String::from);
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(move |_, _, _| Ok(toggle_val.clone().map(Arc::from)));

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    ShiftplanReportServiceImpl::<TestDeps> {
        shiftplan_report_dao: Arc::new(shiftplan_report_dao),
        special_day_service: Arc::new(special_day_service),
        toggle_service: Arc::new(toggle_service),
        transaction_dao: Arc::new(transaction_dao),
    }
}

// ─── Test A — Clip greift ────────────────────────────────────────────────────

/// D-51-06 Chain D + D-04 Zeile 4:
/// Slot Mo 14:00–15:00, ShortDay Mo 14:30, Gate aktiv → 0,5h.
#[tokio::test]
async fn test_extract_for_week_clips_at_shortday() {
    let sp = Uuid::new_v4();
    let row = raw_row(
        sp,
        YEAR,
        WEEK,
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let sd = shortday(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 30, 0).unwrap(),
        YEAR,
        WEEK,
    );
    let service = build_service_for_week(vec![row], vec![sd], Some("2026-07-01"));

    let out = service
        .extract_shiftplan_report_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("extract must succeed");

    assert_eq!(out.len(), 1);
    let day = &out[0];
    assert_eq!(day.sales_person_id, sp);
    assert!(
        approx(day.hours, 0.5),
        "Chain D + D-04 Zeile 4: geclipptes 14:00–14:30 = 0,5h, got {}",
        day.hours
    );
}

// ─── Test B — Gate inaktiv → kein Clip ──────────────────────────────────────

/// D-51-07: `active_from = None` → Legacy (ungeclippt 1,0h).
#[tokio::test]
async fn test_extract_for_week_ungated_no_clip() {
    let sp = Uuid::new_v4();
    let row = raw_row(
        sp,
        YEAR,
        WEEK,
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let sd = shortday(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 30, 0).unwrap(),
        YEAR,
        WEEK,
    );
    let service = build_service_for_week(vec![row], vec![sd], None);

    let out = service
        .extract_shiftplan_report_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("extract must succeed");

    assert_eq!(out.len(), 1);
    assert!(
        approx(out[0].hours, 1.0),
        "D-51-07 Gate off: volle 1,0h, got {}",
        out[0].hours
    );
}

// ─── Test C — Slot komplett hinter Cutoff → 0h ──────────────────────────────

/// D-04 Zeile 3 / SHC-05:
/// Slot Mo 15:00–16:00, Cutoff 14:30, Gate aktiv → Slot komplett weg.
/// Row liefert 0h (aggregations-Bucket ist zwar präsent, aber Hours = 0).
#[tokio::test]
async fn test_extract_for_week_post_cutoff_row_zero() {
    let sp = Uuid::new_v4();
    let row = raw_row(
        sp,
        YEAR,
        WEEK,
        DayOfWeek::Monday,
        time::Time::from_hms(15, 0, 0).unwrap(),
        time::Time::from_hms(16, 0, 0).unwrap(),
    );
    let sd = shortday(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 30, 0).unwrap(),
        YEAR,
        WEEK,
    );
    let service = build_service_for_week(vec![row], vec![sd], Some("2026-07-01"));

    let out = service
        .extract_shiftplan_report_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("extract must succeed");

    // Aggregations-Bucket ist präsent (Row wird aggregiert, trägt 0h bei) —
    // Test akzeptiert beides: leerer Output ODER Bucket mit hours=0.
    let hours = out.iter().map(|d| d.hours).sum::<f32>();
    assert!(
        approx(hours, 0.0),
        "D-04 Zeile 3: Post-Cutoff-Row trägt 0h bei, got {}",
        hours
    );
}

// ─── Test D — Stichtag-Grenzfall (booking_date == active_from) ─────────────

/// D-51-07 SHC-06 Grenzfall:
/// 2026-07-27 ist ISO-Mo Woche 31.
/// - active_from = 2026-07-27 → Gate greift (inklusiv) → 0,5h.
/// - active_from = 2026-07-28 → Gate greift NICHT → 1,0h.
#[tokio::test]
async fn test_extract_for_week_stichtag_boundary_inclusive() {
    let sp = Uuid::new_v4();
    let make_row = || {
        raw_row(
            sp,
            YEAR,
            WEEK,
            DayOfWeek::Monday, // 2026-07-27
            time::Time::from_hms(14, 0, 0).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
    };
    let make_sd = || {
        shortday(
            DayOfWeek::Monday,
            time::Time::from_hms(14, 30, 0).unwrap(),
            YEAR,
            WEEK,
        )
    };

    // active_from == booking_date → clip (inclusive).
    let svc_on = build_service_for_week(vec![make_row()], vec![make_sd()], Some("2026-07-27"));
    let out_on = svc_on
        .extract_shiftplan_report_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("extract must succeed");
    assert!(
        approx(out_on[0].hours, 0.5),
        "SHC-06 inklusiv am Stichtag: 0,5h, got {}",
        out_on[0].hours
    );

    // active_from = booking_date + 1 (Di) → gate für Mo inaktiv (booking_date < active_from).
    let svc_off = build_service_for_week(vec![make_row()], vec![make_sd()], Some("2026-07-28"));
    let out_off = svc_off
        .extract_shiftplan_report_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("extract must succeed");
    assert!(
        approx(out_off[0].hours, 1.0),
        "Vortag: 1,0h, got {}",
        out_off[0].hours
    );
}

// ─── Test E — Aggregation über mehrere Bookings (SHC-05) ────────────────────

/// SHC-05 / D-51-08:
/// Zwei separate Bookings am selben Mo 14:00–15:00 (verschiedene booking_id).
/// Beweis: DAO liefert 2 Rows (Booking-ID pro Row), Rust summiert.
/// Ohne Clip: 2×1,0 = 2,0h. Mit Clip: 2×0,5 = 1,0h.
#[tokio::test]
async fn test_extract_for_week_aggregates_multiple_bookings() {
    let sp = Uuid::new_v4();
    let make_row = || {
        raw_row(
            sp,
            YEAR,
            WEEK,
            DayOfWeek::Monday,
            time::Time::from_hms(14, 0, 0).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
    };
    let rows = vec![make_row(), make_row()];
    let sd = shortday(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 30, 0).unwrap(),
        YEAR,
        WEEK,
    );

    let service_clipped = build_service_for_week(rows.clone(), vec![sd.clone()], Some("2026-07-01"));
    let out_clipped = service_clipped
        .extract_shiftplan_report_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("extract must succeed");
    assert_eq!(out_clipped.len(), 1, "Ein einziger Aggregations-Bucket pro (person, dow)");
    assert!(
        approx(out_clipped[0].hours, 1.0),
        "Zwei Rows × 0,5h clipped = 1,0h, got {}",
        out_clipped[0].hours
    );

    let service_raw = build_service_for_week(rows, vec![sd], None);
    let out_raw = service_raw
        .extract_shiftplan_report_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("extract must succeed");
    assert!(
        approx(out_raw[0].hours, 2.0),
        "Zwei Rows × 1,0h ungeclippt = 2,0h (Gate off), got {}",
        out_raw[0].hours
    );
}

// ─── Test F — Multi-Wochen-Range (Gate-Mischung) ────────────────────────────

/// D-51-07 pro-Woche-Gate:
/// Range über zwei aufeinanderfolgende Wochen (W30 + W31).
/// active_from = 2026-07-27 (Mo W31) → W30 (2026-07-20 bis 2026-07-26) NICHT geclippt,
/// W31 GECLIPPT.
#[tokio::test]
async fn test_extract_range_multi_week_gate_mix() {
    let sp = Uuid::new_v4();
    let row_w30 = raw_row(
        sp,
        YEAR,
        30,
        DayOfWeek::Monday, // 2026-07-20
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let row_w31 = raw_row(
        sp,
        YEAR,
        WEEK, // 31
        DayOfWeek::Monday, // 2026-07-27
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let sd_w30 = shortday(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 30, 0).unwrap(),
        YEAR,
        30,
    );
    let sd_w31 = shortday(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 30, 0).unwrap(),
        YEAR,
        31,
    );

    // Custom-Setup für Range mit zwei Wochen SpecialDays.
    let rows_arc: Arc<[ShiftplanReportRawRow]> = Arc::from(vec![row_w30, row_w31]);
    let mut shiftplan_report_dao = MockShiftplanReportDao::new();
    let rows_clone = rows_arc.clone();
    shiftplan_report_dao
        .expect_extract_raw_shiftplan_report()
        .returning(move |_, _, _, _, _, _| Ok(rows_clone.clone()));

    let mut special_day_service = MockSpecialDayService::new();
    let sd_w30_c = sd_w30.clone();
    let sd_w31_c = sd_w31.clone();
    special_day_service
        .expect_get_by_week()
        .returning(move |_year, week, _| match week {
            30 => Ok(Arc::from(vec![sd_w30_c.clone()])),
            31 => Ok(Arc::from(vec![sd_w31_c.clone()])),
            _ => Ok(Arc::from(Vec::<SpecialDay>::new())),
        });

    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(Some(Arc::from("2026-07-27".to_string()))));

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    let service = ShiftplanReportServiceImpl::<TestDeps> {
        shiftplan_report_dao: Arc::new(shiftplan_report_dao),
        special_day_service: Arc::new(special_day_service),
        toggle_service: Arc::new(toggle_service),
        transaction_dao: Arc::new(transaction_dao),
    };

    let from_date = ShiftyDate::new(YEAR, 30, DayOfWeek::Monday).unwrap();
    let to_date = ShiftyDate::new(YEAR, 31, DayOfWeek::Sunday).unwrap();
    let out = service
        .extract_shiftplan_report(sp, from_date, to_date, Authentication::Full, None)
        .await
        .expect("extract must succeed");

    // Sortiert nach year, week, day_of_week (Service sortiert deterministisch).
    assert_eq!(out.len(), 2);
    let w30 = out.iter().find(|d| d.calendar_week == 30).unwrap();
    let w31 = out.iter().find(|d| d.calendar_week == 31).unwrap();
    assert!(
        approx(w30.hours, 1.0),
        "W30 vor Stichtag: ungeclippt 1,0h, got {}",
        w30.hours
    );
    assert!(
        approx(w31.hours, 0.5),
        "W31 ab Stichtag: geclippt 0,5h, got {}",
        w31.hours
    );
}

// ─── Test G — Snapshot-Immunität ───────────────────────────────────────────

/// D-03 / Chain D:
/// Chain-D-Refactor darf `CURRENT_SNAPSHOT_SCHEMA_VERSION` NICHT bumpen.
/// Der Snapshot-Reader liest persistierte Rows unverändert weiter.
#[test]
fn test_snapshot_schema_version_unchanged() {
    use crate::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION;
    assert_eq!(
        CURRENT_SNAPSHOT_SCHEMA_VERSION, 12,
        "Phase 51 Chain D darf die Snapshot-Version NICHT bumpen (D-03 Snapshot-Immunität)"
    );
}

// ─── Gap-Closure Regression (Chain D) ───────────────────────────────────────

/// Wenn `ToggleService::get_toggle_value` `ServiceError::Unauthorized` liefert,
/// darf `extract_shiftplan_report_for_week` NICHT mit 401 durchschlagen.
/// Statt dessen: Gate inaktiv (Legacy) → Slot bleibt ungeklippt → volle
/// Stunden im Aggregat.
///
/// **Live-Symptom vor Fix:** `GET /report/week/{year}/{week}` gab 401 zurück,
/// weil das REST-Handler-context via mock-auth oder als `Authentication::Full`
/// durchgereicht wurde und der `ToggleService.current_user_id → None →
/// Unauthorized` zurücklieferte. Der zentrale
/// `shortday_gate::read_active_from`-Helper fängt das jetzt in ALLEN drei
/// `ShiftplanReportService`-Methoden ab (nicht mehr nur in
/// `extract_shiftplan_report`).
#[tokio::test]
async fn test_extract_for_week_tolerates_toggle_unauthorized() {
    // Slot Mo 14:00–15:00 mit ShortDay-Cutoff 14:30. Bei aktivem Gate:
    // 0,5h. Bei Legacy (Unauthorized → None): volle 1,0h.
    let sp = Uuid::new_v4();
    let row = raw_row(
        sp,
        YEAR,
        WEEK,
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let sd = shortday(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 30, 0).unwrap(),
        YEAR,
        WEEK,
    );

    // Custom-Setup wie `build_service_for_week`, aber Toggle liefert Unauthorized.
    let rows_arc: Arc<[ShiftplanReportRawRow]> = Arc::from(vec![row]);
    let mut shiftplan_report_dao = MockShiftplanReportDao::new();
    let rows_clone = rows_arc.clone();
    shiftplan_report_dao
        .expect_extract_raw_shiftplan_report_for_week()
        .returning(move |year, week, _| {
            if year == YEAR && week == WEEK {
                Ok(rows_clone.clone())
            } else {
                Ok(Arc::from(Vec::<ShiftplanReportRawRow>::new()))
            }
        });

    let sd_clone = vec![sd];
    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(move |year, week, _| {
            if year == YEAR && week == WEEK {
                Ok(Arc::from(sd_clone.clone()))
            } else {
                Ok(Arc::from(Vec::<SpecialDay>::new()))
            }
        });

    // Kernstück des Regression-Guards: Toggle → Unauthorized.
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Err(service::ServiceError::Unauthorized));

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    let service = ShiftplanReportServiceImpl::<TestDeps> {
        shiftplan_report_dao: Arc::new(shiftplan_report_dao),
        special_day_service: Arc::new(special_day_service),
        toggle_service: Arc::new(toggle_service),
        transaction_dao: Arc::new(transaction_dao),
    };

    let out = service
        .extract_shiftplan_report_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect(
            "Unauthorized-Toleranz: extract_shiftplan_report_for_week muss Ok liefern (Legacy off, kein 401)",
        );

    assert_eq!(out.len(), 1);
    assert!(
        approx(out[0].hours, 1.0),
        "Legacy off: 14:00–15:00 ungeklippt = 1.0h, got {}",
        out[0].hours
    );
}

// ─── Gap-Closure Phase 51 (Live-Symptom): Full-Auth honoriert Toggle ────────
//
// Live-Symptom (Milestone v2.4, 2026-07-04):
// - User setzt `shortday_slot_clipping_active_from = 2026-06-28`.
// - `GET /report/week/2026/27` schlägt bis in Chain D durch mit
//   `Authentication::Full`.
// - `ToggleService::get_toggle_value(name, Full, tx)` warf vor Fix
//   `Unauthorized`, weil `PermissionService::current_user_id(Full) → None`
//   den `user_id.is_none()` Guard triggerte.
// - `shortday_gate::read_active_from` fing das defensiv als `Ok(None)` ab →
//   Modern-Modus mit `active_from = None` → **kein Clip** → volle 1,0h in der
//   Balance statt der konfigurierten 0,5h (Cutoff 14:30 im 14:00–15:00-Slot).
//
// Fix (`toggle.rs`): `Authentication::Full` überspringt den `current_user_id`-
// Guard und lässt den Toggle-Wert durch. Dieser Test PINNT das: mit einem
// Toggle-Wert und `Full` MUSS die aggregierte Stunde 0,5h sein, nicht 1,0h.
#[tokio::test]
async fn test_extract_for_week_honors_toggle_under_full_auth() {
    // Slot Mo 14:00–15:00, ShortDay-Cutoff 14:30, active_from = 2026-07-01
    // (< 2026-07-27 = W31-Mo) → Gate aktiv → 0,5h.
    let sp = Uuid::new_v4();
    let row = raw_row(
        sp,
        YEAR,
        WEEK,
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let sd = shortday(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 30, 0).unwrap(),
        YEAR,
        WEEK,
    );
    // WICHTIG: Toggle liefert den echten Wert (kein Unauthorized). Das
    // simuliert das Verhalten NACH dem toggle.rs-Fix.
    let service = build_service_for_week(vec![row], vec![sd], Some("2026-07-01"));

    let out = service
        .extract_shiftplan_report_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("extract must succeed under Authentication::Full");

    assert_eq!(out.len(), 1);
    assert!(
        approx(out[0].hours, 0.5),
        "Full-Auth honoriert Toggle: Gate aktiv → 0,5h Clip (Phase 51 Gap-Closure Live-Symptom), got {}",
        out[0].hours
    );
}
