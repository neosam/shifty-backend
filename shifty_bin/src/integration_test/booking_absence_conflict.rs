//! End-to-End-Integrationstests für Phase 3 (Booking ⇄ Absence ⇄ ManualUnavailable).
//!
//! Pflicht-Coverage (aus 03-VALIDATION.md):
//! - test_double_source_two_warnings_one_booking         (BOOK-02 / Cross-Source)
//! - test_softdeleted_absence_no_warning_no_marker       (SC4 / Pitfall-1 — full-stack)
//! - test_copy_week_three_bookings_two_warnings          (BOOK-02 / D-Phase3-02)
//! - test_shiftplan_marker_softdeleted_absence_none      (PLAN-01 + SC4 — Read-Pfad)
//!
//! Pattern: TestSetup-In-Memory-SQLite + RestStateDef-Service-Calls (analog
//! `shifty_bin/src/integration_test/absence_period.rs`).

use rest::RestStateDef;
use service::{
    absence::{AbsenceCategory, AbsencePeriod, AbsenceService},
    booking::{Booking, BookingService},
    permission::Authentication,
    sales_person::{SalesPerson, SalesPersonService},
    sales_person_unavailable::{SalesPersonUnavailable, SalesPersonUnavailableService},
    shiftplan::ShiftplanViewService,
    shiftplan_catalog::{Shiftplan, ShiftplanService},
    shiftplan_edit::ShiftplanEditService,
    slot::{Slot, SlotService},
    warning::Warning,
};
use shifty_utils::DayOfWeek;
use time::macros::date;
use time::Time;
use uuid::Uuid;

use crate::integration_test::TestSetup;

// ---- Helpers -------------------------------------------------------------

async fn create_sales_person(test_setup: &TestSetup, name: &str) -> SalesPerson {
    test_setup
        .rest_state
        .sales_person_service()
        .create(
            &SalesPerson {
                id: Uuid::nil(),
                version: Uuid::nil(),
                name: name.into(),
                background_color: "#000000".into(),
                inactive: false,
                is_paid: Some(true),
                deleted: None,
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap()
}

async fn create_shiftplan(test_setup: &TestSetup, name: &str) -> Shiftplan {
    test_setup
        .rest_state
        .shiftplan_service()
        .create(
            &Shiftplan {
                id: Uuid::nil(),
                name: name.into(),
                is_planning: false,
                deleted: None,
                version: Uuid::nil(),
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap()
}

async fn create_monday_slot(test_setup: &TestSetup, shiftplan_id: Option<Uuid>) -> Slot {
    test_setup
        .rest_state
        .slot_service()
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                day_of_week: DayOfWeek::Monday,
                from: Time::from_hms(9, 0, 0).unwrap(),
                to: Time::from_hms(17, 0, 0).unwrap(),
                min_resources: 1,
                valid_from: date!(2024 - 01 - 01),
                valid_to: None,
                deleted: None,
                version: Uuid::nil(),
                shiftplan_id,
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap()
}

async fn create_absence_period_at(
    test_setup: &TestSetup,
    sales_person_id: Uuid,
    from_date: time::Date,
    to_date: time::Date,
) -> AbsencePeriod {
    test_setup
        .rest_state
        .absence_service()
        .create(
            &AbsencePeriod {
                id: Uuid::nil(),
                sales_person_id,
                category: AbsenceCategory::Vacation,
                from_date,
                to_date,
                description: "Phase-3 Cross-Source-Test".into(),
                created: None,
                deleted: None,
                version: Uuid::nil(),
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap()
        .absence
}

async fn create_sales_person_unavailable_at(
    test_setup: &TestSetup,
    sales_person_id: Uuid,
    year: u32,
    calendar_week: u8,
    day_of_week: DayOfWeek,
) -> SalesPersonUnavailable {
    test_setup
        .rest_state
        .sales_person_unavailable_service()
        .create(
            &SalesPersonUnavailable {
                id: Uuid::nil(),
                sales_person_id,
                year,
                calendar_week,
                day_of_week,
                created: None,
                deleted: None,
                version: Uuid::nil(),
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap()
}

fn make_booking(sales_person_id: Uuid, slot_id: Uuid, year: u32, calendar_week: u8) -> Booking {
    Booking {
        id: Uuid::nil(),
        sales_person_id,
        slot_id,
        calendar_week: calendar_week as i32,
        year,
        created: None,
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: Uuid::nil(),
    }
}

// ---- Tests --------------------------------------------------------------

/// BOOK-02 / Cross-Source: ein Tag mit beiden Quellen → ZWEI Warnings, EIN Booking.
/// D-Phase3-15: KEINE De-Dup.
#[tokio::test]
async fn test_double_source_two_warnings_one_booking() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "DoubleSource").await;
    let plan = create_shiftplan(&test_setup, "Phase3-Plan").await;
    let slot = create_monday_slot(&test_setup, Some(plan.id)).await;

    // 2026-W17 Mon = 2026-04-20.
    let _ap = create_absence_period_at(
        &test_setup,
        sp.id,
        date!(2026 - 04 - 20),
        date!(2026 - 04 - 24),
    )
    .await;
    let _mu = create_sales_person_unavailable_at(&test_setup, sp.id, 2026, 17, DayOfWeek::Monday).await;

    let booking = make_booking(sp.id, slot.id, 2026, 17);
    let result = test_setup
        .rest_state
        .shiftplan_edit_service()
        .book_slot_with_conflict_check(&booking, Authentication::Full, None)
        .await
        .expect("book_slot_with_conflict_check should succeed");

    assert_eq!(
        result.warnings.len(),
        2,
        "expected 2 cross-source warnings (one per source, NO de-dup), got {:?}",
        result.warnings
    );
    let has_absence_warning = result
        .warnings
        .iter()
        .any(|w| matches!(w, Warning::BookingOnAbsenceDay { .. }));
    let has_unavailable_warning = result
        .warnings
        .iter()
        .any(|w| matches!(w, Warning::BookingOnUnavailableDay { .. }));
    assert!(has_absence_warning, "missing BookingOnAbsenceDay");
    assert!(has_unavailable_warning, "missing BookingOnUnavailableDay");

    // Booking persistiert.
    let bookings = test_setup
        .rest_state
        .booking_service()
        .get_for_week(17, 2026, Authentication::Full, None)
        .await
        .expect("get_for_week ok");
    assert_eq!(bookings.len(), 1, "exactly one booking persisted");
    assert_eq!(bookings[0].id, result.booking.id);
}

/// SC4 / Pitfall-1 — full-stack:
/// soft-deleted AbsencePeriod produziert KEINE Warning beim Booking-Anlegen.
#[tokio::test]
async fn test_softdeleted_absence_no_warning_no_marker() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Pitfall1").await;
    let plan = create_shiftplan(&test_setup, "Phase3-Plan").await;
    let slot = create_monday_slot(&test_setup, Some(plan.id)).await;

    let ap = create_absence_period_at(
        &test_setup,
        sp.id,
        date!(2026 - 04 - 20),
        date!(2026 - 04 - 24),
    )
    .await;
    // soft-delete via service
    test_setup
        .rest_state
        .absence_service()
        .delete(ap.id, Authentication::Full, None)
        .await
        .expect("delete soft-deletes");

    // Booking-Pfad: keine Warning trotz früherer (jetzt deleted) Absence.
    let booking = make_booking(sp.id, slot.id, 2026, 17);
    let result = test_setup
        .rest_state
        .shiftplan_edit_service()
        .book_slot_with_conflict_check(&booking, Authentication::Full, None)
        .await
        .expect("book_slot_with_conflict_check should succeed");
    assert!(
        result.warnings.is_empty(),
        "soft-deleted absence MUST NOT trigger warning (SC4), got {:?}",
        result.warnings
    );
}

/// BOOK-02 / D-Phase3-02 — full-stack copy_week:
/// 3 Source-Bookings, 2 davon auf einer AbsencePeriod-Range, copy_week
/// überträgt alle 3 in die Target-Woche und produziert 2 Warnings (KEINE De-Dup).
#[tokio::test]
async fn test_copy_week_three_bookings_two_warnings() {
    let test_setup = TestSetup::new().await;
    let sp_a = create_sales_person(&test_setup, "PersonA").await;
    let sp_b = create_sales_person(&test_setup, "PersonB").await;
    let sp_c = create_sales_person(&test_setup, "PersonC").await;
    let plan = create_shiftplan(&test_setup, "Phase3-Plan-Copy").await;
    let slot = create_monday_slot(&test_setup, Some(plan.id)).await;

    // AbsencePeriod nur für sp_a + sp_b in der TARGET-Woche W17 Mon.
    let _ap_a = create_absence_period_at(
        &test_setup,
        sp_a.id,
        date!(2026 - 04 - 20),
        date!(2026 - 04 - 21),
    )
    .await;
    let _ap_b = create_absence_period_at(
        &test_setup,
        sp_b.id,
        date!(2026 - 04 - 20),
        date!(2026 - 04 - 21),
    )
    .await;

    // 3 Source-Bookings in W16 (= 2026-04-13 Monday) — sp_a, sp_b, sp_c.
    for sp_id in [sp_a.id, sp_b.id, sp_c.id] {
        test_setup
            .rest_state
            .booking_service()
            .create(
                &make_booking(sp_id, slot.id, 2026, 16),
                Authentication::Full,
                None,
            )
            .await
            .expect("source booking create");
    }

    // copy_week W16 -> W17.
    let result = test_setup
        .rest_state
        .shiftplan_edit_service()
        .copy_week_with_conflict_check(16, 2026, 17, 2026, Authentication::Full, None)
        .await
        .expect("copy_week_with_conflict_check should succeed");

    assert_eq!(
        result.copied_bookings.len(),
        3,
        "expected 3 copied bookings, got {:?}",
        result.copied_bookings.len()
    );

    let absence_warnings: Vec<&Warning> = result
        .warnings
        .iter()
        .filter(|w| matches!(w, Warning::BookingOnAbsenceDay { .. }))
        .collect();
    assert_eq!(
        absence_warnings.len(),
        2,
        "expected 2 absence-day warnings (sp_a + sp_b in target week, KEINE De-Dup), got {:?}",
        result.warnings
    );

    // Target-Woche enthält 3 Bookings.
    let target_bookings = test_setup
        .rest_state
        .booking_service()
        .get_for_week(17, 2026, Authentication::Full, None)
        .await
        .expect("get_for_week target");
    assert_eq!(target_bookings.len(), 3);
}

/// PLAN-01 + SC4 — Read-Pfad:
/// soft-deleted AbsencePeriod produziert KEINEN ShiftplanDay-Marker.
#[tokio::test]
async fn test_shiftplan_marker_softdeleted_absence_none() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "ReadPathSC4").await;
    let plan = create_shiftplan(&test_setup, "Phase3-Plan-Read").await;
    let _slot = create_monday_slot(&test_setup, Some(plan.id)).await;

    let ap = create_absence_period_at(
        &test_setup,
        sp.id,
        date!(2026 - 04 - 20),
        date!(2026 - 04 - 24),
    )
    .await;
    test_setup
        .rest_state
        .absence_service()
        .delete(ap.id, Authentication::Full, None)
        .await
        .expect("delete soft-deletes");

    let week = test_setup
        .rest_state
        .shiftplan_view_service()
        .get_shiftplan_week_for_sales_person(
            plan.id,
            2026,
            17,
            sp.id,
            Authentication::Full,
            None,
        )
        .await
        .expect("get_shiftplan_week_for_sales_person ok");

    for day in week.days.iter() {
        assert!(
            day.unavailable.is_none(),
            "soft-deleted absence MUST NOT produce shiftplan marker (SC4), \
             got {:?} on {:?}",
            day.unavailable,
            day.day_of_week
        );
    }

    // Sanity: 7 days returned.
    assert_eq!(week.days.len(), 7);
}
