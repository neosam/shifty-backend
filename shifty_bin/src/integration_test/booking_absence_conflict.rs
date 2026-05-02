//! Wave-0-Stubs für Phase-3-Cross-Source-End-to-End-Tests.
//!
//! ALLE Tests sind aktuell `#[ignore]` + `unimplemented!()`-Body. Wave 5
//! (Final-Tests-Plan) muss:
//!  1. `#[ignore]`-Attribut entfernen.
//!  2. `TestSetup`-Setup analog `shifty_bin/src/integration_test/absence_period.rs`
//!     bauen.
//!  3. Den NEUEN konflikt-aware-Endpunkt
//!     (`ShiftplanEditService::book_slot_with_conflict_check`) gegen echte
//!     In-Memory-SQLite + REST-Layer treiben.
//!
//! Abdeckung pro Test verbindlich aus 03-VALIDATION.md:
//!  - test_double_source_two_warnings_one_booking         (BOOK-02 / Cross-Source)
//!  - test_softdeleted_absence_no_warning_no_marker       (SC4 / Pitfall-1)
//!  - test_copy_week_three_bookings_two_warnings          (BOOK-02 / D-Phase3-02)
//!  - test_shiftplan_marker_softdeleted_absence_none      (PLAN-01 + SC4 — Read-Pfad)

#[tokio::test]
#[ignore = "wave-5 target — Cross-Source: AbsencePeriod + sales_person_unavailable on same day → 2 warnings"]
async fn test_double_source_two_warnings_one_booking() {
    unimplemented!("wave-5 — see 03-PATTERNS.md § 'shifty_bin/src/integration_test/booking_absence_conflict.rs'");
}

#[tokio::test]
#[ignore = "wave-5 target — Pitfall-1 / SC4: soft-deleted AbsencePeriod produces NO warning"]
async fn test_softdeleted_absence_no_warning_no_marker() {
    unimplemented!("wave-5 — assert result.warnings.is_empty() AND ShiftplanDay.unavailable.is_none()");
}

#[tokio::test]
#[ignore = "wave-5 target — D-Phase3-02: 3 source bookings, 2 on absence days → 3 copied + 2 warnings"]
async fn test_copy_week_three_bookings_two_warnings() {
    unimplemented!("wave-5 — full-stack copy_week_with_conflict_check");
}

#[tokio::test]
#[ignore = "wave-5 target — PLAN-01 + SC4: ShiftplanDay-Marker None für soft-deleted absence"]
async fn test_shiftplan_marker_softdeleted_absence_none() {
    unimplemented!("wave-5 — get_shiftplan_week_for_sales_person check");
}
