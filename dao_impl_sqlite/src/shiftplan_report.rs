use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    shiftplan_report::{ShiftplanReportDao, ShiftplanReportRawRow},
    DaoError,
};
use shifty_utils::DayOfWeek;
use sqlx::query_as;
use time::{format_description::well_known::Iso8601, Time};
use uuid::Uuid;

/// DB-Row-Struct für die drei raw-row Queries (Phase 51 Chain D, D-51-08).
///
/// Der SQL-Query liefert je Booking eine Row (kein `SUM`, kein `GROUP BY`).
/// Der Service-Layer aggregiert + clippt + gatet.
///
/// `time_from`/`time_to` liegen in SQLite als TEXT im `HH:MM:SS`-Format
/// vor (siehe `slot.rs:40-41`) und werden hier über `Time::parse` in
/// `time::Time` gehoben — nicht via `sqlx::Decode`, weil `time::Time`
/// ohne extra Feature-Gate keinen sqlx-Decoder hat.
pub struct ShiftplanReportRawRowDb {
    pub sales_person_id: Vec<u8>,
    pub booking_id: Vec<u8>,
    pub year: i64,
    pub calendar_week: i64,
    pub day_of_week: i64,
    pub time_from: String,
    pub time_to: String,
}

impl TryFrom<&ShiftplanReportRawRowDb> for ShiftplanReportRawRow {
    type Error = DaoError;
    fn try_from(entity: &ShiftplanReportRawRowDb) -> Result<Self, DaoError> {
        Ok(Self {
            sales_person_id: Uuid::from_slice(entity.sales_person_id.as_ref())?,
            booking_id: Uuid::from_slice(entity.booking_id.as_ref())?,
            year: entity.year as u32,
            calendar_week: entity.calendar_week as u8,
            day_of_week: DayOfWeek::from_number(entity.day_of_week as u8)
                .ok_or(DaoError::InvalidDayOfWeek(entity.day_of_week as u8))?,
            time_from: Time::parse(&entity.time_from, &Iso8601::TIME)?,
            time_to: Time::parse(&entity.time_to, &Iso8601::TIME)?,
        })
    }
}

pub struct ShiftplanReportDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}
impl ShiftplanReportDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl ShiftplanReportDao for ShiftplanReportDaoImpl {
    type Transaction = crate::TransactionImpl;

    async fn extract_raw_shiftplan_report(
        &self,
        sales_person_id: Uuid,
        from_year: u32,
        from_week: u8,
        to_year: u32,
        to_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[ShiftplanReportRawRow]>, DaoError> {
        let sales_person_id_vec = sales_person_id.as_bytes().to_vec();
        Ok(query_as!(
            ShiftplanReportRawRowDb,
            r#"
                SELECT
                  sales_person.id as sales_person_id,
                  booking.id as booking_id,
                  booking.year,
                  booking.calendar_week,
                  slot.day_of_week,
                  slot.time_from,
                  slot.time_to
                FROM slot
                INNER JOIN booking ON (booking.slot_id = slot.id AND booking.deleted IS NULL)
                INNER JOIN sales_person ON booking.sales_person_id = sales_person.id
                LEFT JOIN shiftplan ON slot.shiftplan_id = shiftplan.id
                WHERE sales_person.id = ?
                  AND booking.year * 100 + booking.calendar_week >= ? * 100 + ?
                  AND booking.year * 100 + booking.calendar_week <= ? * 100 + ?
                  AND (shiftplan.is_planning = 0 OR shiftplan.is_planning IS NULL)
                        "#,
            sales_person_id_vec,
            from_year,
            from_week,
            to_year,
            to_week,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(ShiftplanReportRawRow::try_from)
        .collect::<Result<Arc<[_]>, _>>()?)
    }

    async fn extract_raw_quick_shiftplan_report(
        &self,
        year: u32,
        until_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[ShiftplanReportRawRow]>, DaoError> {
        Ok(query_as!(
            ShiftplanReportRawRowDb,
            r#"
                SELECT
                  sales_person.id as sales_person_id,
                  booking.id as booking_id,
                  booking.year,
                  booking.calendar_week,
                  slot.day_of_week,
                  slot.time_from,
                  slot.time_to
                FROM slot
                INNER JOIN booking ON (booking.slot_id = slot.id AND booking.deleted IS NULL)
                INNER JOIN sales_person ON booking.sales_person_id = sales_person.id
                LEFT JOIN shiftplan ON slot.shiftplan_id = shiftplan.id
                WHERE booking.year = ?
                  AND booking.calendar_week <= ?
                  AND (shiftplan.is_planning = 0 OR shiftplan.is_planning IS NULL)
                        "#,
            year,
            until_week
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(ShiftplanReportRawRow::try_from)
        .collect::<Result<Arc<[_]>, _>>()?)
    }

    async fn extract_raw_shiftplan_report_for_week(
        &self,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[ShiftplanReportRawRow]>, DaoError> {
        Ok(query_as!(
            ShiftplanReportRawRowDb,
            r#"
                SELECT
                  sales_person.id as sales_person_id,
                  booking.id as booking_id,
                  booking.year,
                  booking.calendar_week,
                  slot.day_of_week,
                  slot.time_from,
                  slot.time_to
                FROM slot
                INNER JOIN booking ON (booking.slot_id = slot.id AND booking.deleted IS NULL)
                INNER JOIN sales_person ON booking.sales_person_id = sales_person.id
                LEFT JOIN shiftplan ON slot.shiftplan_id = shiftplan.id
                WHERE booking.year = ?
                  AND booking.calendar_week = ?
                  AND (shiftplan.is_planning = 0 OR shiftplan.is_planning IS NULL)
                        "#,
            year,
            calendar_week
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(ShiftplanReportRawRow::try_from)
        .collect::<Result<Arc<[_]>, _>>()?)
    }
}
