use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::booking_log::{BookingLogDao, BookingLogEntity};
use dao::DaoError;
use shifty_utils::DayOfWeek;
use sqlx::query_as;
use std::sync::Arc;
use time::PrimitiveDateTime;

struct BookingLogDb {
    name: Option<String>,
    year: Option<i64>,
    calendar_week: Option<i64>,
    day_of_week: Option<i64>,
    time_from: Option<String>,
    time_to: Option<String>,
    created: Option<String>,
    deleted: Option<String>,
    created_by: Option<String>,
    deleted_by: Option<String>,
}

impl TryFrom<&BookingLogDb> for BookingLogEntity {
    type Error = DaoError;

    fn try_from(db: &BookingLogDb) -> Result<Self, Self::Error> {
        let year = db.year.ok_or_else(|| DaoError::EnumValueNotFound("year is NULL".into()))?;
        let calendar_week = db.calendar_week.ok_or_else(|| DaoError::EnumValueNotFound("calendar_week is NULL".into()))?;
        let day_of_week_num = db.day_of_week.ok_or_else(|| DaoError::EnumValueNotFound("day_of_week is NULL".into()))?;
        let name = db.name.as_ref().ok_or_else(|| DaoError::EnumValueNotFound("name is NULL".into()))?;
        let time_from_str = db.time_from.as_ref().ok_or_else(|| DaoError::EnumValueNotFound("time_from is NULL".into()))?;
        let time_to_str = db.time_to.as_ref().ok_or_else(|| DaoError::EnumValueNotFound("time_to is NULL".into()))?;
        let created_str = db.created.as_ref().ok_or_else(|| DaoError::EnumValueNotFound("created is NULL".into()))?;
        let created_by_str = db.created_by.as_ref().ok_or_else(|| DaoError::EnumValueNotFound("created_by is NULL".into()))?;

        Ok(Self {
            year: year as u32,
            calendar_week: calendar_week as u8,
            day_of_week: DayOfWeek::from_number(day_of_week_num as u8)
                .ok_or_else(|| DaoError::InvalidDayOfWeek(day_of_week_num as u8))?,
            name: name.clone().into(),
            time_from: time::Time::parse(time_from_str, &time::format_description::well_known::Iso8601::DEFAULT)?,
            time_to: time::Time::parse(time_to_str, &time::format_description::well_known::Iso8601::DEFAULT)?,
            created: PrimitiveDateTime::parse(created_str, &time::format_description::well_known::Iso8601::DEFAULT)?,
            deleted: db
                .deleted
                .as_ref()
                .map(|d| PrimitiveDateTime::parse(d, &time::format_description::well_known::Iso8601::DEFAULT))
                .transpose()?,
            created_by: created_by_str.clone().into(),
            deleted_by: db.deleted_by.as_ref().map(|s| s.clone().into()),
        })
    }
}

pub struct BookingLogDaoImpl;

#[async_trait]
impl BookingLogDao for BookingLogDaoImpl {
    type Transaction = crate::TransactionImpl;

    async fn get_booking_logs_for_week(
        &self,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[BookingLogEntity]>, DaoError> {
        let result = query_as!(
            BookingLogDb,
            r#"
            SELECT
                name,
                year,
                calendar_week,
                day_of_week,
                time_from,
                time_to,
                created,
                deleted,
                created_by,
                deleted_by
            FROM bookings_view
            WHERE year = ? AND calendar_week = ?
            ORDER BY day_of_week, time_from
            "#,
            year,
            calendar_week
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(result
            .iter()
            .map(BookingLogEntity::try_from)
            .collect::<Result<Arc<[_]>, _>>()?)
    }
}
