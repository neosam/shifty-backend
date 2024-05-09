use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    slot::{DayOfWeek, SlotEntity},
    DaoError,
};
use sqlx::{query, SqlitePool};
use time::{format_description::well_known::Iso8601, Date, PrimitiveDateTime, Time};
use uuid::Uuid;

use crate::ResultDbErrorExt;

pub struct SlotDaoImpl {
    pool: Arc<SqlitePool>,
}
impl SlotDaoImpl {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl dao::slot::SlotDao for SlotDaoImpl {
    async fn get_slots(&self) -> Result<Arc<[SlotEntity]>, DaoError> {
        let result = query!(r"SELECT id, day_of_week, time_from, time_to, valid_from, valid_to, deleted, update_version FROM slot WHERE deleted IS NULL")
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|err| DaoError::DatabaseQueryError(Box::new(err)))?;
        result
            .iter()
            .map(|row| {
                Ok(SlotEntity {
                    id: Uuid::from_slice(row.id.as_ref())?,
                    day_of_week: DayOfWeek::from_number(row.day_of_week as u8)
                        .ok_or(DaoError::InvalidDayOfWeek(row.day_of_week as u8))?,
                    from: Time::parse(&row.time_from, &Iso8601::TIME)?,
                    to: Time::parse(&row.time_to, &Iso8601::TIME)?,
                    valid_from: Date::parse(&row.valid_from, &Iso8601::DATE)?,
                    valid_to: row
                        .valid_to
                        .as_ref()
                        .map(|valid_to| Date::parse(valid_to, &Iso8601::DATE))
                        .transpose()?,
                    deleted: row
                        .deleted
                        .as_ref()
                        .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE))
                        .transpose()?,
                    version: Uuid::from_slice(&row.update_version)?,
                })
            })
            .collect()
    }
    async fn get_slot(&self, id: &Uuid) -> Result<Option<SlotEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        let result = query!(r"SELECT id, day_of_week, time_from, time_to, valid_from, valid_to, deleted, update_version FROM slot WHERE id = ?", id_vec)
            .fetch_optional(self.pool.as_ref())
            .await
            .map_err(|err| DaoError::DatabaseQueryError(Box::new(err)))?;
        result
            .map(|row| {
                Ok(SlotEntity {
                    id: Uuid::from_slice(row.id.as_ref())?,
                    day_of_week: DayOfWeek::from_number(row.day_of_week as u8)
                        .ok_or(DaoError::InvalidDayOfWeek(row.day_of_week as u8))?,
                    from: Time::parse(&row.time_from, &Iso8601::TIME)?,
                    to: Time::parse(&row.time_to, &Iso8601::TIME)?,
                    valid_from: Date::parse(&row.valid_from, &Iso8601::DATE)?,
                    valid_to: row
                        .valid_to
                        .as_ref()
                        .map(|valid_to| Date::parse(valid_to, &Iso8601::DATE))
                        .transpose()?,
                    deleted: row
                        .deleted
                        .as_ref()
                        .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE))
                        .transpose()?,
                    version: Uuid::from_slice(&row.update_version)?,
                })
            })
            .transpose()
    }
    async fn create_slot(&self, slot: &SlotEntity, process: &str) -> Result<(), DaoError> {
        let id_vec = slot.id.as_bytes().to_vec();
        let version_vec = slot.version.as_bytes().to_vec();
        let day_of_week = slot.day_of_week.to_number();
        let from = slot.from.to_string();
        let to = slot.to.to_string();
        let valid_from = slot.valid_from.to_string();
        let valid_to = slot.valid_to.map(|valid_to| valid_to.to_string());
        let deleted = slot.deleted.as_ref().map(|deleted| deleted.to_string());
        query!("INSERT INTO slot (id, day_of_week, time_from, time_to, valid_from, valid_to, deleted, update_version, update_process) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id_vec,
            day_of_week,
            from,
            to,
            valid_from,
            valid_to,
            deleted,
            version_vec,
            process,
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn update_slot(&self, slot: &SlotEntity, process: &str) -> Result<(), DaoError> {
        let id_vec = slot.id.as_bytes().to_vec();
        let version_vec = slot.version.as_bytes().to_vec();
        let valid_to = slot.valid_to.map(|valid_to| valid_to.to_string());
        let deleted = slot.deleted.as_ref().map(|deleted| deleted.to_string());
        query!("UPDATE slot SET valid_to = ?, deleted = ?, update_version = ?, update_process = ? WHERE id = ?",
            valid_to,
            deleted,
            version_vec,
            process,
            id_vec,
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }
}
