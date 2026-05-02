//! Wave-0-Stubs für Phase-3-Reverse-Warning-Tests.
//!
//! ALLE Tests sind aktuell `#[ignore]` + `unimplemented!()`-Body. Wave 3
//! muss pro Test:
//!  1. `#[ignore]`-Attribut ENTFERNEN.
//!  2. `unimplemented!()` durch echten Mock-DI-Setup + Assertions ersetzen
//!     (Pattern aus `service_impl/src/test/booking.rs:113-192` +
//!      `service_impl/src/test/shiftplan.rs:59-100`).
//!
//! Abdeckung pro Test verbindlich aus 03-VALIDATION.md:
//!  - test_book_slot_warning_on_absence_day        (BOOK-02 / D-Phase3-14 BookingOnAbsenceDay)
//!  - test_book_slot_warning_on_manual_unavailable (BOOK-02 / D-Phase3-14 BookingOnUnavailableDay)
//!  - test_book_slot_no_warning_when_softdeleted_absence (SC4 / Pitfall-1)
//!  - test_copy_week_aggregates_warnings           (BOOK-02 / D-Phase3-02, D-Phase3-15: KEINE De-Dup)
//!  - test_book_slot_with_conflict_check_forbidden (D-Phase3-12 HR ∨ self)
//!  - test_copy_week_with_conflict_check_forbidden (D-Phase3-12 HR ∨ self)

#[tokio::test]
#[ignore = "wave-3 target — Warning enum + ShiftplanEditService::book_slot_with_conflict_check"]
async fn test_book_slot_warning_on_absence_day() {
    unimplemented!("wave-3 — see 03-PATTERNS.md § 'service_impl/src/test/shiftplan_edit.rs'");
}

#[tokio::test]
#[ignore = "wave-3 target — Warning enum + sales_person_unavailable lookup"]
async fn test_book_slot_warning_on_manual_unavailable() {
    unimplemented!("wave-3 — see 03-PATTERNS.md");
}

#[tokio::test]
#[ignore = "wave-3 target — Pitfall-1 / SC4: soft-deleted AbsencePeriod MUST NOT trigger warning"]
async fn test_book_slot_no_warning_when_softdeleted_absence() {
    unimplemented!("wave-3 — Mock returns empty Vec (DAO filtert deleted IS NULL)");
}

#[tokio::test]
#[ignore = "wave-3 target — D-Phase3-02 + D-Phase3-15: aggregate warnings, KEINE De-Dup"]
async fn test_copy_week_aggregates_warnings() {
    unimplemented!("wave-3 — 3 source bookings, 2 on absence days → 2 warnings + 3 copied");
}

#[tokio::test]
#[ignore = "wave-3 target — D-Phase3-12 Permission HR ∨ self"]
async fn test_book_slot_with_conflict_check_forbidden() {
    unimplemented!("wave-3 — both permission probes return Forbidden");
}

#[tokio::test]
#[ignore = "wave-3 target — D-Phase3-12 Permission HR ∨ self"]
async fn test_copy_week_with_conflict_check_forbidden() {
    unimplemented!("wave-3 — both permission probes return Forbidden");
}
