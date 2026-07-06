use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use shifty_utils::{DayOfWeek, ShiftyDate, ShiftyDateUtilsError};
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

use crate::{permission::Authentication, ServiceError};

#[derive(Clone, Debug, PartialEq)]
pub struct ShiftplanReportDay {
    pub sales_person_id: Uuid,
    pub hours: f32,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
}

impl ShiftplanReportDay {
    pub fn to_date(&self) -> Result<ShiftyDate, ShiftyDateUtilsError> {
        ShiftyDate::new(self.year, self.calendar_week, self.day_of_week)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShiftplanQuickOverview {
    pub sales_person_id: Uuid,
    pub hours: f32,
    pub year: u32,
}

#[automock(type Context=(); type Transaction = MockTransaction;)]
#[async_trait]
pub trait ShiftplanReportService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn extract_shiftplan_report(
        &self,
        sales_person_id: Uuid,
        from_date: ShiftyDate,
        to_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanReportDay]>, ServiceError>;

    async fn extract_quick_shiftplan_report(
        &self,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanQuickOverview]>, ServiceError>;

    async fn extract_shiftplan_report_for_week(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanReportDay]>, ServiceError>;

    /// Phase 52 Follow-up #3 — ISO-Wochenjahr-Batch-Variante zu
    /// [`extract_shiftplan_report_for_week`].
    ///
    /// Semantisch äquivalent zum Aufsummieren aller `_for_week(year, w)`-Rufe
    /// für `w ∈ 1..=weeks_in_year(year)`. Liefert alle Report-Tage des ISO-
    /// Jahres in **einem** DAO-Roundtrip; Aggregation, Clip und Stichtag-Gate
    /// laufen im Service-Layer analog zur Wochenvariante.
    ///
    /// **ISO-Wochenjahr, nicht Kalender-Jahr:** die DB-Spalte `booking.year`
    /// speichert das ISO-Wochenjahr. Ein Booking am 2027-01-01 (Fr = ISO-2026-W53-Fri)
    /// liegt in `iso_year=2026`. Ersetzt das alte `extract_shiftplan_report_for_year`
    /// (aus Phase 52 Wave 3), das die SpecialDay-Liste kalendarisch geladen und
    /// dann per ISO-Wochenjahr gefiltert hat — mit dem Ergebnis, dass Feiertage
    /// an KW 1 / KW 53 nicht auf Bookings dieser Wochen wirkten.
    ///
    /// Rückgabe-Rows tragen `year`, `calendar_week` und `day_of_week` — der
    /// Consumer kann pro Woche weiterfiltern.
    async fn extract_shiftplan_report_for_iso_year(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanReportDay]>, ServiceError>;
}
