use crate::{permission::Authentication, shiftplan_catalog::Shiftplan, ServiceError};
use dao::Transaction;
use mockall::automock;
use shifty_utils::DayOfWeek;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ShiftplanWeek {
    pub year: u32,
    pub calendar_week: u8,
    pub days: Vec<ShiftplanDay>,
}

/// Per-Tag-Marker für die per-sales-person-Sicht (D-Phase3-10). Bei
/// Doppel-Quelle (AbsencePeriod UND ManualUnavailable am selben Tag) wird
/// `Both` gesetzt — die `absence_id`/`category` der AbsencePeriod werden
/// mitgeführt, weil sie semantisch reicher als der bloße ManualUnavailable-
/// Eintrag sind.
///
/// Granularität: pro Tag genau eine Variante. Slots bleiben sichtbar —
/// Frontend rendert die Markierung als Per-Tag-Badge zusätzlich zu den
/// Slots (D-Phase3-10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnavailabilityMarker {
    AbsencePeriod {
        absence_id: uuid::Uuid,
        category: crate::absence::AbsenceCategory,
    },
    ManualUnavailable,
    Both {
        absence_id: uuid::Uuid,
        category: crate::absence::AbsenceCategory,
    },
}

#[derive(Debug, Clone)]
pub struct ShiftplanDay {
    pub day_of_week: DayOfWeek,
    pub slots: Vec<ShiftplanSlot>,
    /// Phase-3-Marker, gesetzt nur durch die per-sales-person-Sicht
    /// (`get_shiftplan_*_for_sales_person`). Globale `get_shiftplan_*` lassen
    /// das Feld immer `None`.
    pub unavailable: Option<UnavailabilityMarker>,
}

#[derive(Debug, Clone)]
pub struct ShiftplanSlot {
    pub slot: crate::slot::Slot,
    pub bookings: Vec<ShiftplanBooking>,
    /// Phase 5 (D-04, D-05, D-09): live count of bookings whose
    /// `sales_person.is_paid == true`. Always populated regardless of whether
    /// `slot.max_paid_employees` is configured. Soft-deleted bookings and
    /// soft-deleted sales persons are excluded upstream by the DAO/service
    /// layer; absence status of the booked person is irrelevant (D-05).
    pub current_paid_count: u8,
}

#[derive(Debug, Clone)]
pub struct ShiftplanBooking {
    pub booking: crate::booking::Booking,
    pub sales_person: crate::sales_person::SalesPerson,
    pub self_added: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct PlanDayView {
    pub shiftplan: Shiftplan,
    pub slots: Vec<ShiftplanSlot>,
}

#[derive(Debug, Clone)]
pub struct ShiftplanDayAggregate {
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
    pub plans: Vec<PlanDayView>,
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait::async_trait]
pub trait ShiftplanViewService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: Transaction;

    async fn get_shiftplan_week(
        &self,
        shiftplan_id: uuid::Uuid,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ShiftplanWeek, ServiceError>;

    async fn get_shiftplan_day(
        &self,
        year: u32,
        week: u8,
        day_of_week: DayOfWeek,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ShiftplanDayAggregate, ServiceError>;

    /// Phase 3 — per-sales-person-Variante (PLAN-01).
    ///
    /// Liefert die Schichtplan-Woche, wobei jeder Tag das Feld `unavailable`
    /// gesetzt bekommt, falls für `sales_person_id` an diesem Tag eine
    /// aktive AbsencePeriod und/oder ein aktiver `sales_person_unavailable`-
    /// Eintrag existiert. Soft-deleted Einträge werden gefiltert
    /// (Pitfall 1 / SC4).
    ///
    /// Permission: HR ∨ `verify_user_is_sales_person(sales_person_id)`
    /// (D-Phase3-12).
    async fn get_shiftplan_week_for_sales_person(
        &self,
        shiftplan_id: uuid::Uuid,
        year: u32,
        week: u8,
        sales_person_id: uuid::Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ShiftplanWeek, ServiceError>;

    /// Phase 3 — per-sales-person-Day-Variante (PLAN-01). Analog zu
    /// [`Self::get_shiftplan_week_for_sales_person`], aber liefert nur den
    /// einen `day_of_week` als Aggregat über alle Shiftplans.
    ///
    /// Permission: HR ∨ `verify_user_is_sales_person(sales_person_id)`
    /// (D-Phase3-12).
    async fn get_shiftplan_day_for_sales_person(
        &self,
        year: u32,
        week: u8,
        day_of_week: DayOfWeek,
        sales_person_id: uuid::Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ShiftplanDayAggregate, ServiceError>;
}
