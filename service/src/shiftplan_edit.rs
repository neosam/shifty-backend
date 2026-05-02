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
}
