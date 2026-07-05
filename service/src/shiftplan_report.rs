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

    /// Phase 52 (WOP-01, D-52-06) — Jahres-Batch-Variante zu
    /// [`extract_shiftplan_report_for_week`].
    ///
    /// Semantisch äquivalent zum Aufsummieren aller `_for_week(year, w)`-Rufe
    /// für `w ∈ 1..=53`. Liefert alle Report-Tage des Jahres in **einem**
    /// DAO-Roundtrip; Aggregation, Clip und Stichtag-Gate laufen im
    /// Service-Layer analog zur Wochenvariante.
    ///
    /// Rückgabe-Rows tragen `year`, `calendar_week` und `day_of_week` — der
    /// Consumer kann pro Woche weiterfiltern.
    async fn extract_shiftplan_report_for_year(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanReportDay]>, ServiceError>;
}
