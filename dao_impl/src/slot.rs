use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use dao::{
    slot::{DayOfWeek, SlotEntity},
    DaoError,
};
use sqlx::{query, SqlitePool};
use time::{
    format_description::well_known::Iso8601, macros::format_description, Date, PrimitiveDateTime,
    Time,
};
use uuid::Uuid;

use crate::{ResultDbErrorExt, TransactionImpl};

pub struct SlotDaoImpl {
    _pool: Arc<SqlitePool>,
}
impl SlotDaoImpl {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl dao::slot::SlotDao for SlotDaoImpl {
    type Transaction = TransactionImpl;

    async fn get_slots(&self, tx: Self::Transaction) -> Result<Arc<[SlotEntity]>, DaoError> {
        let result = query!(r"SELECT id, day_of_week, time_from, time_to, min_resources, valid_from, valid_to, deleted, update_version FROM slot WHERE deleted IS NULL")
            .fetch_all(tx.tx.lock().await.as_mut())
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
                    min_resources: row.min_resources as u8,
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

    async fn get_slot(
        &self,
        id: &Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<SlotEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        let result = query!(r"SELECT id, day_of_week, time_from, time_to, min_resources, valid_from, valid_to, deleted, update_version FROM slot WHERE id = ?", id_vec)
            .fetch_optional(tx.tx.lock().await.as_mut())
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
                    min_resources: row.min_resources as u8,
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

    async fn get_slots_for_week(
        &self,
        year: u32,
        week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[SlotEntity]>, DaoError> {
        let monday = Date::from_iso_week_date(year as i32, week, time::Weekday::Monday)?;
        let sunday = Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday)?;
        let monday_str = monday.format(&Iso8601::DATE)?;
        let sunday_str = sunday.format(&Iso8601::DATE)?;
        let result = query!(r"
                SELECT id, day_of_week, time_from, time_to, min_resources, valid_from, valid_to, deleted, update_version 
                FROM slot 
                WHERE deleted IS NULL
                AND valid_from <= ?
                AND (valid_to IS NULL OR valid_to >= ?)", sunday_str, monday_str)
            .fetch_all(tx.tx.lock().await.as_mut())
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
                    min_resources: row.min_resources as u8,
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

    async fn create_slot(
        &self,
        slot: &SlotEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let time_format = format_description!("[hour]:[minute]:[second].0");
        let id_vec = slot.id.as_bytes().to_vec();
        let version_vec = slot.version.as_bytes().to_vec();
        let day_of_week = slot.day_of_week.to_number();
        let from = slot.from.format(&time_format)?;
        let to = slot.to.format(&time_format)?;
        let valid_from = slot.valid_from.to_string();
        let valid_to = slot.valid_to.map(|valid_to| valid_to.to_string());
        let deleted = slot.deleted.as_ref().map(|deleted| deleted.to_string());
        let min_resources = slot.min_resources;
        query!("INSERT INTO slot (id, day_of_week, time_from, time_to, valid_from, valid_to, deleted, update_version, update_process, min_resources) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id_vec,
            day_of_week,
            from,
            to,
            valid_from,
            valid_to,
            deleted,
            version_vec,
            process,
            min_resources,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn update_slot(
        &self,
        slot: &SlotEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
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
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }
}
