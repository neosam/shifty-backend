//! Domain-Warnings für Phase-3 (Booking ⇄ Absence ⇄ ManualUnavailable Cross-Source-Konflikte).
//!
//! `Warning` ist Erfolgs-Pfad — sie wird in den Wrapper-Result-Structs
//! `BookingCreateResult` / `CopyWeekResult` (in `service::shiftplan_edit`) und
//! `AbsencePeriodCreateResult` (in `service::absence`) propagiert. KEIN
//! `ServiceError`-Pfad. KEIN ValidationFailureItem (das wäre 422; Warnings
//! sind 200/201 mit Liste).
//!
//! Granularität (D-Phase3-15): eine Warning pro betroffenem Booking-Tag.

use shifty_utils::DayOfWeek;
use time::Date;
use uuid::Uuid;

use crate::absence::AbsenceCategory;

/// Cross-Source-Konflikt-Warning. Vier Varianten, jede trägt nur die für die
/// jeweilige Quelle relevanten Felder. Frontend rendert eine Liste.
///
/// Stable per D-Phase3-14 — die 5. Variante `ManualUnavailableOnAbsenceDay`
/// ist deferred (D-Phase3-17, Folgephase).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Warning {
    /// Beim Anlegen eines Bookings auf einem Tag, der durch eine
    /// AbsencePeriod abgedeckt ist (Reverse-Warning, BOOK-02).
    BookingOnAbsenceDay {
        booking_id: Uuid,
        date: Date,
        absence_id: Uuid,
        category: AbsenceCategory,
    },
    /// Beim Anlegen eines Bookings auf einem Tag, der durch
    /// `sales_person_unavailable` abgedeckt ist (Reverse-Warning, BOOK-02).
    BookingOnUnavailableDay {
        booking_id: Uuid,
        year: u32,
        week: u8,
        day_of_week: DayOfWeek,
    },
    /// Beim Anlegen einer AbsencePeriod, die ein bestehendes Booking
    /// überlappt (Forward-Warning, BOOK-01).
    AbsenceOverlapsBooking {
        absence_id: Uuid,
        booking_id: Uuid,
        date: Date,
    },
    /// Beim Anlegen einer AbsencePeriod, die einen bestehenden manuellen
    /// `sales_person_unavailable`-Eintrag überdeckt (Forward-Warning,
    /// BOOK-01, D-Phase3-16: KEIN Auto-Cleanup).
    AbsenceOverlapsManualUnavailable {
        absence_id: Uuid,
        unavailable_id: Uuid,
    },
    /// Phase 5 (D-08): emittiert beim Anlegen eines Bookings über
    /// `ShiftplanEditService::book_slot_with_conflict_check`, wenn der
    /// Ziel-Slot ein konfiguriertes `max_paid_employees`-Limit hat und der
    /// resultierende Live-Count der bezahlten Mitarbeiter:innen in dieser
    /// (year, week, slot)-Kombination das Limit STRIKT übersteigt
    /// (`current_paid_count > max_paid_employees`, D-06).
    ///
    /// Die Buchung wird trotzdem persistiert (D-07) — die Warning ist
    /// rein informativ. NULL `max_paid_employees` triggert NICHT (D-15).
    /// Übersetzung der Variant-Bedeutung passiert im Frontend (en/de/cs);
    /// das Backend trägt nur strukturierte Daten.
    PaidEmployeeLimitExceeded {
        slot_id: Uuid,
        booking_id: Uuid,
        year: u32,
        week: u8,
        current_paid_count: u8,
        max_paid_employees: u8,
    },
}
