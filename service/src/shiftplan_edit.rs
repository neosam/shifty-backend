use crate::booking::Booking;
use crate::permission::Authentication;
use crate::slot::Slot;
use crate::ServiceError;
use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use std::sync::Arc;
use uuid::Uuid;

/// Wrapper-Result für [`ShiftplanEditService::book_slot_with_conflict_check`].
///
/// Enthält das persistierte Booking + alle Cross-Source-Warnings, die für
/// diesen Tag detektiert wurden (D-Phase3-01, D-Phase3-15: eine Warning pro
/// betroffenem Booking-Tag; KEINE De-Dup zwischen AbsencePeriod- und
/// ManualUnavailable-Quellen).
#[derive(Debug, Clone)]
pub struct BookingCreateResult {
    pub booking: Booking,
    pub warnings: Arc<[crate::warning::Warning]>,
}

/// Wrapper-Result für [`ShiftplanEditService::copy_week_with_conflict_check`].
///
/// Aggregiert pro kopiertem Booking alle Warnings (D-Phase3-02). Inner-Loop
/// ruft pro Source-Booking [`ShiftplanEditService::book_slot_with_conflict_check`]
/// und akkumuliert dessen Warnings; KEINE De-Dup über Bookings hinweg
/// (D-Phase3-15).
#[derive(Debug, Clone)]
pub struct CopyWeekResult {
    pub copied_bookings: Arc<[Booking]>,
    pub warnings: Arc<[crate::warning::Warning]>,
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait ShiftplanEditService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction + std::fmt::Debug + Clone + Send + Sync + 'static;

    async fn modify_slot(
        &self,
        slot: &Slot,
        change_year: u32,
        change_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Slot, ServiceError>;

    /// Ändert die Werte eines Slots als einmalige Ausnahme für **genau eine Kalenderwoche**
    /// (D-35-01 Approach B: 3-Segment-Split + Re-Merge).
    ///
    /// Erzeugt drei Slot-Versionen:
    /// - Segment 1: Original mit valid_to = Sonntag KW-1 (oder delete_slot wenn erste KW)
    /// - Segment 2: Ausnahme-Woche (Mon KW → Son KW) mit neuen Werten aus `slot`
    /// - Segment 3: Wiederherstellung ab Montag KW+1 mit Original-Werten
    ///
    /// Buchungen ab change_week werden partitioniert:
    /// `calendar_week == change_week` → Segment 2; sonst → Segment 3 (D-35-03).
    ///
    /// Permission: `shiftplan.edit` (D-35-06). Alles in EINER Transaktion (D-35-04).
    async fn modify_slot_single_week(
        &self,
        slot: &Slot,
        change_year: u32,
        change_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Slot, ServiceError>;

    async fn remove_slot(
        &self,
        slot: Uuid,
        change_year: u32,
        change_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn update_carryover(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn update_carryover_all_employees(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn add_vacation(
        &self,
        sales_person_id: Uuid,
        from: time::Date,
        to: time::Date,
        description: Arc<str>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    /// Phase 3 — konflikt-aware Booking-Persist (BOOK-02).
    ///
    /// Persistiert `booking` via Basic-[`crate::booking::BookingService::create`]
    /// und produziert Cross-Source-Warnings:
    /// [`crate::warning::Warning::BookingOnAbsenceDay`] pro überlappender
    /// AbsencePeriod, [`crate::warning::Warning::BookingOnUnavailableDay`]
    /// falls ein `sales_person_unavailable`-Eintrag den Tag abdeckt
    /// (D-Phase3-06).
    ///
    /// Permission: HR ∨ `verify_user_is_sales_person(booking.sales_person_id)`
    /// (D-Phase3-12). Soft-deleted AbsencePeriods werden im DAO-Layer
    /// gefiltert (Pitfall 1 / SC4); soft-deleted ManualUnavailables werden
    /// hier client-seitig ignoriert.
    async fn book_slot_with_conflict_check(
        &self,
        booking: &Booking,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BookingCreateResult, ServiceError>;

    /// Phase 3 — konflikt-aware copy_week (BOOK-02 / D-Phase3-02).
    ///
    /// Iteriert die Bookings der Quell-Woche, ruft pro Source-Booking
    /// intern [`Self::book_slot_with_conflict_check`] und aggregiert ALLE
    /// Warnings (D-Phase3-15: KEINE De-Dup). Liefert das Set der kopierten
    /// Bookings + die akkumulierten Warnings.
    ///
    /// Permission: `shiftplan.edit` (HR/SHIFTPLANNER) — bulk-Operation auf
    /// Schichtplan-Ebene, analog zu `modify_slot`.
    async fn copy_week_with_conflict_check(
        &self,
        from_calendar_week: u8,
        from_year: u32,
        to_calendar_week: u8,
        to_year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CopyWeekResult, ServiceError>;

    /// Phase 40 (WST-04) — Lösch-Pfad mit Wochen-Sperre-Gate.
    ///
    /// Ersetzt den direkten `BookingService::delete`-Aufruf im DELETE-Handler,
    /// damit auch das Ausbuchen der Wochen-Sperre unterliegt (D-40-02). Lädt das
    /// Booking (für year/calendar_week), prüft die Sperre und delegiert dann an
    /// die Basic-Tier-`BookingService::delete` (erhält die Shiftplanner-∨-Self-
    /// Permission). Reihenfolge: get → assert_week_not_locked → delete.
    async fn delete_booking(
        &self,
        booking_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
