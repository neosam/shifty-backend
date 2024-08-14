use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    shiftplan_report::{ShiftplanQuickOverviewEntity, ShiftplanReportDao, ShiftplanReportEntity},
    slot::DayOfWeek,
    DaoError,
};
use sqlx::query_as;
use uuid::Uuid;

pub struct ShiftplanReportDb {
    pub sales_person_id: Vec<u8>,
    pub hours: Option<f64>,
    pub year: i64,
    pub calendar_week: i64,
    pub day_of_week: i64,
}
impl TryFrom<&ShiftplanReportDb> for ShiftplanReportEntity {
    type Error = DaoError;
    fn try_from(entity: &ShiftplanReportDb) -> Result<Self, DaoError> {
        Ok(Self {
            sales_person_id: Uuid::from_slice(entity.sales_person_id.as_ref())?,
            hours: entity.hours.unwrap_or(0.0) as f32,
            year: entity.year as u32,
            calendar_week: entity.calendar_week as u8,
            day_of_week: DayOfWeek::from_number(entity.day_of_week as u8)
                .ok_or_else(|| DaoError::InvalidDayOfWeek(entity.day_of_week as u8))?,
        })
    }
}

pub struct ShiftplanQuickOverviewDb {
    pub sales_person_id: Vec<u8>,
    pub hours: Option<f64>,
    pub year: i64,
}
impl From<&ShiftplanQuickOverviewDb> for ShiftplanQuickOverviewEntity {
    fn from(entity: &ShiftplanQuickOverviewDb) -> Self {
        Self {
            sales_person_id: Uuid::from_slice(entity.sales_person_id.as_ref()).unwrap(),
            hours: entity.hours.unwrap_or(0.0) as f32,
            year: entity.year as u32,
        }
    }
}

pub struct ShiftplanReportDaoImpl {
    pub pool: Arc<sqlx::SqlitePool>,
}
impl ShiftplanReportDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShiftplanReportDao for ShiftplanReportDaoImpl {
    async fn extract_shiftplan_report(
        &self,
        sales_person_id: Uuid,
        year: u32,
        until_week: u8,
    ) -> Result<Arc<[ShiftplanReportEntity]>, DaoError> {
        let sales_person_id_vec = sales_person_id.as_bytes().to_vec();
        Ok(query_as!(
            ShiftplanReportDb,
            r#"
                SELECT
                  sales_person.id as sales_person_id,
                  sum((STRFTIME('%H', slot.time_to) + STRFTIME('%M', slot.time_to) / 60.0) - (STRFTIME('%H', slot.time_from) + STRFTIME('%M', slot.time_from))) as hours,
                  booking.calendar_week, booking.year, slot.day_of_week
                FROM slot
                INNER JOIN booking ON (booking.slot_id = slot.id AND booking.deleted IS NULL)
                INNER JOIN sales_person ON booking.sales_person_id = sales_person.id
                WHERE sales_person.id = ?
                  AND booking.year = ?
                  AND booking.calendar_week <= ?
                GROUP BY year, calendar_week, day_of_week
                        "#,
            sales_person_id_vec,
            year,
            until_week
        ).fetch_all(self.pool.as_ref())
            .await
            .map_db_error()?
            .iter()
            .map(ShiftplanReportEntity::try_from)
            .collect::<Result<Arc<[_]>, _>>()?
        )
    }

    async fn extract_quick_shiftplan_report(
        &self,
        year: u32,
        until_week: u8,
    ) -> Result<Arc<[ShiftplanQuickOverviewEntity]>, DaoError> {
        Ok(query_as!(
            ShiftplanQuickOverviewDb,
            r#"
                SELECT
                  sales_person.id as sales_person_id,
                  sum((STRFTIME('%H', slot.time_to) + STRFTIME('%M', slot.time_to) / 60.0) - (STRFTIME('%H', slot.time_from) + STRFTIME('%M', slot.time_from))) as hours,
                  booking.year
                FROM slot
                INNER JOIN booking ON (booking.slot_id = slot.id AND booking.deleted IS NULL)
                INNER JOIN sales_person ON booking.sales_person_id = sales_person.id
                WHERE booking.year = ?
                  AND booking.calendar_week <= ?
                GROUP BY sales_person_id, year
                        "#,
            year,
            until_week
        ).fetch_all(self.pool.as_ref())
            .await
            .map_db_error()?
            .iter()
            .map(ShiftplanQuickOverviewEntity::from)
            .collect::<Arc<[_]>>()
        )
    }

    async fn extract_shiftplan_report_for_week(
        &self,
        year: u32,
        calendar_week: u8,
    ) -> Result<Arc<[ShiftplanReportEntity]>, DaoError> {
        Ok(query_as!(
            ShiftplanReportDb,
            r#"
                SELECT
                  sales_person.id as sales_person_id,
                  sum((STRFTIME('%H', slot.time_to) + STRFTIME('%M', slot.time_to) / 60.0) - (STRFTIME('%H', slot.time_from) + STRFTIME('%M', slot.time_from))) as hours,
                  booking.calendar_week, booking.year, slot.day_of_week
                FROM slot
                INNER JOIN booking ON (booking.slot_id = slot.id AND booking.deleted IS NULL)
                INNER JOIN sales_person ON booking.sales_person_id = sales_person.id
                WHERE booking.year = ?
                  AND booking.calendar_week = ?
                GROUP BY sales_person_id, year
                        "#,
            year,
            calendar_week
        ).fetch_all(self.pool.as_ref())
            .await
            .map_db_error()?
            .iter()
            .map(ShiftplanReportEntity::try_from)
            .collect::<Result<Arc<[_]>, _>>()?
        )
    }
}
