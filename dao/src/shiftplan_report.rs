use async_trait::async_trait;
use mockall::automock;
use shifty_utils::DayOfWeek;
use std::sync::Arc;
use uuid::Uuid;

use crate::DaoError;

// Phase 51 Chain D (D-51-08): Die alten SUM-Aggregat-Entities
// `ShiftplanReportEntity` + `ShiftplanQuickOverviewEntity` wurden entfernt.
// Der DAO liefert jetzt ausschließlich Roh-Zeilen (`ShiftplanReportRawRow`),
// der Service aggregiert + clippt + gatet in Rust (Chain D).

/// Roh-Zeile pro Booking+Slot (Phase 51 Chain D — D-51-08).
///
/// Wird von den `extract_raw_*`-Methoden geliefert (kein SQL-`SUM`, kein
/// `GROUP BY`). Der Service-Layer aggregiert + wendet `Slot::clip_to` +
/// `shortday_gate::should_clip` an, bevor er die Report-DTOs baut.
#[derive(Clone, Debug, PartialEq)]
pub struct ShiftplanReportRawRow {
    pub sales_person_id: Uuid,
    pub booking_id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
    pub time_from: time::Time,
    pub time_to: time::Time,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait ShiftplanReportDao {
    type Transaction: crate::Transaction;

    /// Roh-Zeilen pro Booking im Range (Phase 51 Chain D).
    /// Service-Layer aggregiert + clippt + gatet in Rust (D-51-08).
    async fn extract_raw_shiftplan_report(
        &self,
        sales_person_id: Uuid,
        from_year: u32,
        from_week: u8,
        to_year: u32,
        to_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[ShiftplanReportRawRow]>, DaoError>;

    /// Roh-Zeilen pro Booking bis `until_week` (Phase 51 Chain D).
    /// Service-Layer aggregiert pro Jahr nach Clip + Gate.
    async fn extract_raw_quick_shiftplan_report(
        &self,
        year: u32,
        until_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[ShiftplanReportRawRow]>, DaoError>;

    /// Roh-Zeilen pro Booking für eine konkrete Woche (Phase 51 Chain D).
    async fn extract_raw_shiftplan_report_for_week(
        &self,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[ShiftplanReportRawRow]>, DaoError>;
}
