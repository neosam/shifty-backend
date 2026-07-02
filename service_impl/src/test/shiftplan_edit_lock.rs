//! Phase 40 (Plan 40-03) — Wochen-Sperr-Enforcement-Matrix für
//! `ShiftplanEditServiceImpl`.
//!
//! Deckt die 6 Schreibpfade × {Locked, Open} + TOCTOU (kein Write vor Gate)
//! + delete_booking-Reihenfolge (get → gate → delete) ab. IDs aus
//! 40-VALIDATION.md (T-40-01..17) + T-40-CR01 (CR-01-Regressions-Test).
//!
//! T-40-03 (WR-01 Fix, 2026-07-02): modify_slot_single_week + non-editor →
//! Forbidden. War in der ursprünglichen Implementierung vergessen worden;
//! jetzt wiederhergestellt (vollständige Matrix: 17 nummerierte Tests).
//!
//! T-40-CR01 (CR-01 Fix, 2026-07-02): Regression für den shiftplanner-vs-
//! shiftplan.edit-Bypass-Mismatch. Belegt, dass book_slot shiftplan.edit
//! (nicht shiftplanner) als Bypass prüft — konsistent mit allen anderen Pfaden.
//!
//! Enforcement-Vertrag (D-40-02):
//!  - `assert_week_not_locked` prüft ZUERST `check_permission("shiftplan.edit")`
//!    → bei Ok Bypass (shiftplan.edit-Halter dürfen in Locked-Wochen schreiben).
//!  - sonst wird der Wochen-Status in DERSELBEN Transaktion gelesen; bei
//!    `WeekStatus::Locked` → `Err(ServiceError::WeekLocked { year, week })`.
//!  - Der Read läuft VOR jedem Write-Effekt (kein TOCTOU); delete_booking liest
//!    year/week per `get` VOR dem `delete`.
//!
//! Reuse: `build_dependencies` / `ShiftplanEditDependencies` aus dem
//! bestehenden `shiftplan_edit`-Testmodul (pub(crate)). Die Fixtures sind hier
//! lokal dupliziert, damit dieses Modul nur die beiden Plan-Dateien berührt
//! (shiftplan_edit_lock.rs + mod.rs) und nicht das bestehende Testmodul.

use std::sync::Arc;

use service::{
    booking::Booking,
    shiftplan_edit::ShiftplanEditService,
    slot::Slot,
    week_status::WeekStatus,
    ServiceError,
};
use shifty_utils::DayOfWeek;
use time::macros::datetime;
use time::{Month, Time};
use uuid::{uuid, Uuid};

use crate::test::error_test::{test_forbidden, NoneTypeExt};
use crate::test::shiftplan_edit::{build_dependencies, ShiftplanEditDependencies};

// ---------- Lokale Fixtures (spiegeln shiftplan_edit-Testmodul) ----------

fn default_sales_person_id() -> Uuid {
    uuid!("04215DFE-13C4-413C-8C66-77AC741BB5F0")
}

fn default_slot_id() -> Uuid {
    uuid!("7A7FF57A-782B-4C2E-A68B-4E2D81D79380")
}

fn default_booking_id() -> Uuid {
    uuid!("CEA260A0-112B-4970-936C-F7E529955BD0")
}

fn default_version() -> Uuid {
    uuid!("F79C462A-8D4E-42E1-8171-DB4DBD019E50")
}

/// Slot mit `version == default_version()` — matcht den vom Default-Mock
/// (`build_dependencies`) gelieferten `get_slot`-Slot, sodass modify_slot keine
/// EntityConflicts wirft.
fn monday_slot() -> Slot {
    Slot {
        id: default_slot_id(),
        day_of_week: DayOfWeek::Monday,
        from: Time::from_hms(9, 0, 0).unwrap(),
        to: Time::from_hms(17, 0, 0).unwrap(),
        min_resources: 1,
        max_paid_employees: None,
        valid_from: time::Date::from_calendar_date(2024, Month::January, 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: default_version(),
        shiftplan_id: None,
    }
}

/// Booking in 2026-W17 (year=2026, calendar_week=17). Für delete_booking liefert
/// `booking_service.get` diese Werte → assert_week_not_locked(2026, 17, ...).
fn default_booking() -> Booking {
    Booking {
        id: Uuid::nil(),
        sales_person_id: default_sales_person_id(),
        slot_id: default_slot_id(),
        calendar_week: 17,
        year: 2026,
        created: None,
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: Uuid::nil(),
    }
}

fn persisted_booking() -> Booking {
    Booking {
        id: default_booking_id(),
        version: default_version(),
        created: Some(datetime!(2026 - 04 - 20 00:00:00)),
        ..default_booking()
    }
}

// ---------- Helpers ----------

/// Überschreibt den Default-`get_week_status`-Mock (Unset) mit dem gewünschten
/// Wochen-Status. `checkpoint()` entfernt die Default-Expectation zuerst.
fn set_week_status(deps: &mut ShiftplanEditDependencies, status: WeekStatus) {
    deps.week_status_service.checkpoint();
    deps.week_status_service
        .expect_get_week_status()
        .returning(move |_, _, _, _| Ok(status.clone()));
}

fn empty_bookings() -> Arc<[Booking]> {
    Arc::from(Vec::<Booking>::new())
}

// =========================================================================
//  modify_slot (shiftplan.edit-gegatet → non-editor bekommt Forbidden VOR Gate)
// =========================================================================

/// T-40-01: modify_slot + Locked + non-shiftplanner → Err(Forbidden).
/// Die shiftplan.edit-Permission schlägt VOR dem Wochen-Sperr-Gate fehl; das
/// Gate ist hier nicht ausschlaggebend (40-VALIDATION.md-Hinweis).
#[tokio::test]
async fn t_40_01_modify_slot_locked_non_editor_forbidden() {
    let mut deps = build_dependencies(false, false); // non-editor
    set_week_status(&mut deps, WeekStatus::Locked);
    // Keine Slot-/Booking-Expectations: Forbidden feuert vor jeder Mutation.
    let service = deps.build_service();
    let result = service
        .modify_slot(&monday_slot(), 2026, 26, ().auth(), None)
        .await;
    test_forbidden(&result);
}

/// T-40-02: modify_slot + Locked + shiftplanner → Ok (Bypass via shiftplan.edit).
#[tokio::test]
async fn t_40_02_modify_slot_locked_editor_bypass_ok() {
    let mut deps = build_dependencies(true, true); // editor
    set_week_status(&mut deps, WeekStatus::Locked);
    deps.slot_service
        .expect_update_slot()
        .returning(|_, _, _| Ok(()));
    deps.slot_service
        .expect_create_slot()
        .returning(|s, _, _| Ok(s.clone()));
    deps.booking_service
        .expect_get_for_slot_id_since()
        .returning(|_, _, _, _, _| Ok(empty_bookings()));

    let service = deps.build_service();
    let result = service
        .modify_slot(&monday_slot(), 2026, 26, ().auth(), None)
        .await;
    assert!(
        result.is_ok(),
        "shiftplan.edit-Halter muss die Sperre umgehen: {result:?}"
    );
}

// =========================================================================
//  modify_slot_single_week
// =========================================================================

/// T-40-03: modify_slot_single_week + Locked + non-editor → Err(Forbidden).
/// Symmetrisch zu T-40-01 (modify_slot): shiftplan.edit schlägt VOR dem
/// Sperr-Gate fehl, sodass Forbidden der erwartete Fehler ist.
/// (Dieser Test war in der ursprünglichen Phase-40-Implementierung vergessen
/// worden; WR-01 Fix, 2026-07-02.)
#[tokio::test]
async fn t_40_03_modify_slot_single_week_locked_non_editor_forbidden() {
    let mut deps = build_dependencies(false, false); // non-editor
    set_week_status(&mut deps, WeekStatus::Locked);
    let service = deps.build_service();
    let result = service
        .modify_slot_single_week(&monday_slot(), 2026, 26, ().auth(), None)
        .await;
    test_forbidden(&result);
}

/// T-40-04: modify_slot_single_week + Locked + shiftplanner → Ok (Bypass).
#[tokio::test]
async fn t_40_04_modify_slot_single_week_locked_editor_bypass_ok() {
    let mut deps = build_dependencies(true, true);
    set_week_status(&mut deps, WeekStatus::Locked);
    deps.slot_service
        .expect_update_slot()
        .returning(|_, _, _| Ok(()));
    deps.slot_service
        .expect_create_slot()
        .returning(|s, _, _| {
            Ok(Slot {
                id: uuid!("40030000-0000-0000-0000-000000000001"),
                ..s.clone()
            })
        });
    deps.booking_service
        .expect_get_for_slot_id_since()
        .returning(|_, _, _, _, _| Ok(empty_bookings()));

    let service = deps.build_service();
    let result = service
        .modify_slot_single_week(&monday_slot(), 2026, 26, ().auth(), None)
        .await;
    assert!(
        result.is_ok(),
        "shiftplan.edit-Halter muss die Sperre umgehen: {result:?}"
    );
}

// =========================================================================
//  remove_slot
// =========================================================================

/// T-40-05: remove_slot + Locked + shiftplanner → Ok (Bypass).
#[tokio::test]
async fn t_40_05_remove_slot_locked_editor_bypass_ok() {
    let mut deps = build_dependencies(true, true);
    set_week_status(&mut deps, WeekStatus::Locked);
    deps.slot_service
        .expect_update_slot()
        .returning(|_, _, _| Ok(()));
    deps.booking_service
        .expect_get_for_slot_id_since()
        .returning(|_, _, _, _, _| Ok(empty_bookings()));

    let service = deps.build_service();
    let result = service
        .remove_slot(default_slot_id(), 2026, 26, ().auth(), None)
        .await;
    assert!(
        result.is_ok(),
        "shiftplan.edit-Halter muss die Sperre umgehen: {result:?}"
    );
}

/// T-40-06: remove_slot + Open (Planned) + shiftplanner → Ok.
#[tokio::test]
async fn t_40_06_remove_slot_open_editor_ok() {
    let mut deps = build_dependencies(true, true);
    set_week_status(&mut deps, WeekStatus::Planned);
    deps.slot_service
        .expect_update_slot()
        .returning(|_, _, _| Ok(()));
    deps.booking_service
        .expect_get_for_slot_id_since()
        .returning(|_, _, _, _, _| Ok(empty_bookings()));

    let service = deps.build_service();
    let result = service
        .remove_slot(default_slot_id(), 2026, 26, ().auth(), None)
        .await;
    assert!(result.is_ok(), "offene Woche muss durchlaufen: {result:?}");
}

// =========================================================================
//  book_slot_with_conflict_check
// =========================================================================

/// T-40-07: book_slot + Locked + non-shiftplanner (Self-Booker) →
/// Err(WeekLocked { 2026, 17 }). Kern-Enforcement WST-03.
///
/// RED gegen das 40-01-pass-through: Self-Booker läuft durch → Ok statt
/// WeekLocked.
#[tokio::test]
async fn t_40_07_book_slot_locked_non_editor_weeklocked() {
    let mut deps = build_dependencies(false, true); // non-editor, Self erlaubt
    set_week_status(&mut deps, WeekStatus::Locked);

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&default_booking(), ().auth(), None)
        .await;

    match result {
        Err(ServiceError::WeekLocked { year, week }) => {
            assert_eq!(year, 2026);
            assert_eq!(week, 17);
        }
        other => panic!("expected WeekLocked, got {other:?}"),
    }
}

/// T-40-08: book_slot + Locked + shiftplanner → Ok (Bypass).
#[tokio::test]
async fn t_40_08_book_slot_locked_editor_bypass_ok() {
    let mut deps = build_dependencies(true, false); // editor
    set_week_status(&mut deps, WeekStatus::Locked);

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&default_booking(), ().auth(), None)
        .await;
    assert!(
        result.is_ok(),
        "shiftplan.edit-Halter muss die Sperre umgehen: {result:?}"
    );
}

/// T-40-09: book_slot + Open (Planned) + non-shiftplanner → Ok.
#[tokio::test]
async fn t_40_09_book_slot_open_non_editor_ok() {
    let mut deps = build_dependencies(false, true); // non-editor, Self erlaubt
    set_week_status(&mut deps, WeekStatus::Planned);

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&default_booking(), ().auth(), None)
        .await;
    assert!(result.is_ok(), "offene Woche muss durchlaufen: {result:?}");
}

/// T-40-CR01 Regression (CR-01 Fix): book_slot + Locked + shiftplanner-only
/// (kein shiftplan.edit) → Err(WeekLocked).
///
/// Dieser Test war mit dem alten `if !is_shiftplanner`-Guard ROT (der Guard
/// übersprang den Lock-Check für jeden shiftplanner-Halter, auch ohne
/// shiftplan.edit). Nach dem CR-01 Fix (unbedingtes assert_week_not_locked)
/// ist er GRÜN, weil assert_week_not_locked intern shiftplan.edit prüft —
/// und nur bei Erfolg bypassed.
///
/// Mock-Setup: shiftplanner=Ok, shiftplan.edit=Err(Forbidden) — die beiden
/// DB-Rollen werden bewusst getrennt gemockt (separate Migrationen).
#[tokio::test]
async fn t_40_cr01_book_slot_locked_shiftplanner_no_edit_weeklocked() {
    use mockall::predicate::{always, eq};
    use service::permission::SHIFTPLANNER_PRIVILEGE;

    let mut deps = build_dependencies(false, false); // Basis: alle Perms Err
    set_week_status(&mut deps, WeekStatus::Locked);

    // Perms granular überschreiben: shiftplanner=Ok, shiftplan.edit=Err.
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .returning(|_, _| Ok(()));
    deps.permission_service
        .expect_check_permission()
        .with(eq("shiftplan.edit"), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.permission_service
        .expect_current_user_id()
        .returning(|_| Ok(Some("test-user".into())));

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&default_booking(), ().auth(), None)
        .await;

    match result {
        Err(ServiceError::WeekLocked { year, week }) => {
            assert_eq!(year, 2026, "year mismatch");
            assert_eq!(week, 17, "week mismatch");
        }
        other => panic!(
            "expected WeekLocked for shiftplanner-only caller in Locked week, got {other:?}"
        ),
    }
}

// =========================================================================
//  copy_week_with_conflict_check
// =========================================================================

/// T-40-10: copy_week + Locked Ziel + shiftplanner → Ok (Bypass).
#[tokio::test]
async fn t_40_10_copy_week_locked_target_editor_bypass_ok() {
    let mut deps = build_dependencies(true, false);
    set_week_status(&mut deps, WeekStatus::Locked);
    // Default get_for_week (leere Quelle) → keine inneren book_slot-Aufrufe.

    let service = deps.build_service();
    let result = service
        .copy_week_with_conflict_check(16, 2026, 17, 2026, ().auth(), None)
        .await;
    assert!(
        result.is_ok(),
        "shiftplan.edit-Halter muss die Ziel-Woche-Sperre umgehen: {result:?}"
    );
}

/// T-40-11: copy_week + Locked Quelle, Open Ziel + shiftplanner → Ok.
/// Nur die Ziel-Woche ist schreibrelevant; shiftplan.edit-Halter bypassen
/// ohnehin. Belegt, dass der Copy-Pfad bei Editor immer durchläuft.
#[tokio::test]
async fn t_40_11_copy_week_locked_source_open_target_editor_ok() {
    let mut deps = build_dependencies(true, false);
    set_week_status(&mut deps, WeekStatus::Locked);

    let service = deps.build_service();
    let result = service
        .copy_week_with_conflict_check(16, 2026, 17, 2026, ().auth(), None)
        .await;
    assert!(
        result.is_ok(),
        "nur die Ziel-Woche zählt; Editor bypassed: {result:?}"
    );
}

// =========================================================================
//  delete_booking (6. Schreibpfad, WST-04 — inkl. Self-Ausbuchen)
// =========================================================================

/// T-40-12: delete_booking + Locked + non-shiftplanner → Err(WeekLocked).
/// Schließt die WST-04-Bypass-Lücke (harte Sperre inkl. Self, D-40-02).
///
/// RED gegen das 40-01-pass-through: delete läuft durch → Ok statt WeekLocked.
#[tokio::test]
async fn t_40_12_delete_booking_locked_non_editor_weeklocked() {
    let mut deps = build_dependencies(false, true); // non-editor
    set_week_status(&mut deps, WeekStatus::Locked);
    deps.booking_service
        .expect_get()
        .returning(|_, _, _| Ok(persisted_booking()));
    // delete bleibt für den pass-through-RED-Lauf ungebremst erlaubt.
    deps.booking_service
        .expect_delete()
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let result = service
        .delete_booking(default_booking_id(), ().auth(), None)
        .await;

    match result {
        Err(ServiceError::WeekLocked { year, week }) => {
            assert_eq!(year, 2026);
            assert_eq!(week, 17);
        }
        other => panic!("expected WeekLocked, got {other:?}"),
    }
}

/// T-40-13: delete_booking + Locked + shiftplanner → Ok (Bypass).
#[tokio::test]
async fn t_40_13_delete_booking_locked_editor_bypass_ok() {
    let mut deps = build_dependencies(true, false); // editor
    set_week_status(&mut deps, WeekStatus::Locked);
    deps.booking_service
        .expect_get()
        .returning(|_, _, _| Ok(persisted_booking()));
    deps.booking_service
        .expect_delete()
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let result = service
        .delete_booking(default_booking_id(), ().auth(), None)
        .await;
    assert!(
        result.is_ok(),
        "shiftplan.edit-Halter muss die Sperre umgehen: {result:?}"
    );
}

/// T-40-14: delete_booking + Open (Planned) + non-shiftplanner → Ok.
#[tokio::test]
async fn t_40_14_delete_booking_open_non_editor_ok() {
    let mut deps = build_dependencies(false, true); // non-editor
    set_week_status(&mut deps, WeekStatus::Planned);
    deps.booking_service
        .expect_get()
        .returning(|_, _, _| Ok(persisted_booking()));
    deps.booking_service
        .expect_delete()
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let result = service
        .delete_booking(default_booking_id(), ().auth(), None)
        .await;
    assert!(result.is_ok(), "offene Woche muss durchlaufen: {result:?}");
}

/// T-40-15: delete_booking + nicht-existente ID → Err(EntityNotFound) VOR dem
/// Gate. Belegt die get → gate-Reihenfolge (get schlägt zuerst fehl).
#[tokio::test]
async fn t_40_15_delete_booking_nonexistent_entitynotfound_before_gate() {
    let mut deps = build_dependencies(false, true);
    // Locked würde blockieren — aber get schlägt zuerst fehl.
    set_week_status(&mut deps, WeekStatus::Locked);
    deps.booking_service
        .expect_get()
        .returning(|_, _, _| Err(ServiceError::EntityNotFound(default_booking_id())));
    // Kein expect_delete → ein delete-Aufruf würde paniken (darf nicht passieren).

    let service = deps.build_service();
    let result = service
        .delete_booking(default_booking_id(), ().auth(), None)
        .await;

    match result {
        Err(ServiceError::EntityNotFound(_)) => {}
        other => panic!("expected EntityNotFound before gate, got {other:?}"),
    }
}

// =========================================================================
//  TOCTOU + delete-Reihenfolge
// =========================================================================

/// T-40-16 (TOCTOU): book_slot + Locked + non-shiftplanner → der Write-Mock
/// (`booking_service.create`) darf NIE aufgerufen werden. Der Lock-Read läuft in
/// derselben Transaktion VOR jedem Write-Effekt.
///
/// RED gegen das 40-01-pass-through: create wird trotz Locked aufgerufen →
/// `.times(0)` schlägt fehl.
#[tokio::test]
async fn t_40_16_book_slot_locked_no_write_before_gate() {
    let mut deps = build_dependencies(false, true); // non-editor, Self erlaubt
    set_week_status(&mut deps, WeekStatus::Locked);
    // Default-Booking-Expectations entfernen und create explizit auf times(0) setzen.
    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_create()
        .times(0)
        .returning(|_, _, _| Ok(persisted_booking()));

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&default_booking(), ().auth(), None)
        .await;

    assert!(
        matches!(result, Err(ServiceError::WeekLocked { .. })),
        "erwartet WeekLocked ohne Write, got {result:?}"
    );
}

/// T-40-17 (delete-Reihenfolge): delete_booking + Locked + non-shiftplanner →
/// `booking_service.delete` darf NIE aufgerufen werden. Reihenfolge:
/// get → lock-check → (blockiert, kein delete).
///
/// RED gegen das 40-01-pass-through: delete wird trotz Locked aufgerufen →
/// `.times(0)` schlägt fehl.
#[tokio::test]
async fn t_40_17_delete_booking_locked_no_delete_before_gate() {
    let mut deps = build_dependencies(false, true); // non-editor
    set_week_status(&mut deps, WeekStatus::Locked);
    deps.booking_service
        .expect_get()
        .returning(|_, _, _| Ok(persisted_booking()));
    deps.booking_service
        .expect_delete()
        .times(0)
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let result = service
        .delete_booking(default_booking_id(), ().auth(), None)
        .await;

    assert!(
        matches!(result, Err(ServiceError::WeekLocked { .. })),
        "erwartet WeekLocked ohne delete, got {result:?}"
    );
}
