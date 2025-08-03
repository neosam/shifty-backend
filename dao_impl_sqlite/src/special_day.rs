use std::sync::Arc;

use crate::DaoError;
use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::special_day::{SpecialDayDao, SpecialDayEntity, SpecialDayTypeEntity};
use shifty_utils::DayOfWeek;
use sqlx::query_as;
use time::macros::format_description;
use time::{format_description::well_known::Iso8601, PrimitiveDateTime, Time};
use uuid::Uuid;

struct SpecialDayDb {
    id: Vec<u8>,
    year: i64,
    calendar_week: i64,
    day_of_week: i64,
    day_type: String,
    time_of_day: Option<String>,
    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&SpecialDayDb> for SpecialDayEntity {
    type Error = DaoError;

    fn try_from(entity: &SpecialDayDb) -> Result<Self, Self::Error> {
        let time_format = format_description!("[hour]:[minute]:[second]");
        Ok(Self {
            id: Uuid::from_slice(&entity.id)?,
            year: entity.year as u32,
            calendar_week: entity.calendar_week as u8,
            day_of_week: DayOfWeek::from_number(entity.day_of_week as u8)
                .ok_or(DaoError::InvalidDayOfWeek(entity.day_of_week as u8))?,
            day_type: match entity.day_type.as_str() {
                "Holiday" => SpecialDayTypeEntity::Holiday,
                "ShortDay" => SpecialDayTypeEntity::ShortDay,
                value @ _ => return Err(DaoError::EnumValueNotFound(value.into())),
            },
            time_of_day: entity
                .time_of_day
                .as_ref()
                .map(|time_of_day| Time::parse(&time_of_day, &time_format))
                .transpose()?,
            created: PrimitiveDateTime::parse(&entity.created, &Iso8601::DATE_TIME)?,
            deleted: entity
                .deleted
                .as_ref()
                .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&entity.update_version).unwrap(),
        })
    }
}

pub struct SpecialDayDaoImpl {
    pub pool: Arc<sqlx::SqlitePool>,
}
impl SpecialDayDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SpecialDayDao for SpecialDayDaoImpl {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SpecialDayEntity>, DaoError> {
        let id = id.as_bytes().to_vec();
        Ok(query_as!(
            SpecialDayDb,
            r#"
            SELECT id, year, calendar_week, day_of_week, day_type, time_of_day, created, deleted, update_version
            FROM special_day
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_db_error()?
        .as_ref()
        .map(SpecialDayEntity::try_from)
        .transpose()?)
    }
    async fn find_by_week(
        &self,
        year: u32,
        calendar_week: u8,
    ) -> Result<Arc<[SpecialDayEntity]>, DaoError> {
        Ok(query_as!(
            SpecialDayDb,
            r#"
            SELECT id, year, calendar_week, day_of_week, day_type, time_of_day, created, deleted, update_version
            FROM special_day
            WHERE year = ? AND calendar_week = ? AND deleted IS NULL
            "#,
            year,
            calendar_week
        )
        .fetch_all(&*self.pool)
        .await
        .map_db_error()?
        .iter()
        .map(SpecialDayEntity::try_from)
        .collect::<Result<_, _>>()?)
    }
    async fn create(&self, entity: &SpecialDayEntity, process: &str) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let year = entity.year as i64;
        let calendar_week = entity.calendar_week as i64;
        let day_of_week = entity.day_of_week.to_number() as i64;
        let day_type = match entity.day_type {
            SpecialDayTypeEntity::Holiday => "Holiday",
            SpecialDayTypeEntity::ShortDay => "ShortDay",
        }
        .to_string();
        let time_format = format_description!("[hour]:[minute]:[second]");
        let time_of_day = entity
            .time_of_day
            .as_ref()
            .map(|time_of_day| time_of_day.format(&time_format))
            .transpose()?;
        let created = entity.created.format(&Iso8601::DATE_TIME).map_db_error()?;
        query_as!(
            SpecialDayDb,
            r#"
            INSERT INTO special_day (id, year, calendar_week, day_of_week, day_type, time_of_day, created, update_version, update_process)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            year,
            calendar_week,
            day_of_week,
            day_type,
            time_of_day,
            created,
            version,
            process,
        ).execute(&*self.pool)
            .await
            .map_db_error()?;
        Ok(())
    }
    async fn update(&self, entity: &SpecialDayEntity, process: &str) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let deleted = entity
            .deleted
            .as_ref()
            .map(|deleted| deleted.format(&Iso8601::DATE_TIME))
            .transpose()?;
        query_as!(
            SpecialDayDb,
            r#"
            UPDATE special_day
            SET deleted = ?, update_version = ?, update_process = ?
            WHERE id = ?
            "#,
            deleted,
            version,
            process,
            id,
        )
        .execute(&*self.pool)
        .await
        .map_db_error()?;
        Ok(())
    }
}
