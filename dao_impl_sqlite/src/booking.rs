use std::sync::Arc;

use crate::{ResultDbErrorExt, TransactionImpl};
use async_trait::async_trait;
use dao::{
    booking::{BookingDao, BookingEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

#[derive(Debug)]
struct BookingDb {
    id: Vec<u8>,
    sales_person_id: Vec<u8>,
    slot_id: Vec<u8>,
    calendar_week: i64,
    year: i64,
    created: String,
    deleted: Option<String>,
    created_by: Option<String>,
    deleted_by: Option<String>,
    update_version: Vec<u8>,
}
impl TryFrom<&BookingDb> for BookingEntity {
    type Error = DaoError;
    fn try_from(booking: &BookingDb) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from_slice(booking.id.as_ref()).unwrap(),
            sales_person_id: Uuid::from_slice(booking.sales_person_id.as_ref()).unwrap(),
            slot_id: Uuid::from_slice(booking.slot_id.as_ref()).unwrap(),
            calendar_week: booking.calendar_week as i32,
            year: booking.year as u32,
            created: PrimitiveDateTime::parse(&booking.created, &Iso8601::DATE_TIME)?,
            deleted: booking
                .deleted
                .as_ref()
                .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
                .transpose()?,
            created_by: booking.created_by.as_ref().map(|s| s.as_str().into()),
            deleted_by: booking.deleted_by.as_ref().map(|s| s.as_str().into()),
            version: Uuid::from_slice(&booking.update_version).unwrap(),
        })
    }
}

pub struct BookingDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}
impl BookingDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl BookingDao for BookingDaoImpl {
    type Transaction = TransactionImpl;

    async fn all(&self, tx: Self::Transaction) -> Result<Arc<[BookingEntity]>, DaoError> {
        Ok(query_as!(
            BookingDb,
            "SELECT id, sales_person_id, slot_id, calendar_week, year, created, deleted, created_by, deleted_by, update_version FROM booking WHERE deleted IS NULL"
        )
            .fetch_all(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?
            .iter()
            .map(BookingEntity::try_from)
            .collect::<Result<Arc<[BookingEntity]>, DaoError>>()?
        )
    }
    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<BookingEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            BookingDb,
            "SELECT id, sales_person_id, slot_id, calendar_week, year, created, deleted, created_by, deleted_by, update_version FROM booking WHERE id = ?",
            id_vec,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(BookingEntity::try_from)
        .transpose()?)
    }

    async fn find_by_slot_id_from(
        &self,
        slot_id: Uuid,
        year: u32,
        week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[BookingEntity]>, DaoError> {
        let slot_id_vec = slot_id.as_bytes().to_vec();
        let until = year * 100 + week as u32;
        Ok(query_as!(
            BookingDb,
            "SELECT id, sales_person_id, slot_id, calendar_week, year, created, deleted, created_by, deleted_by, update_version FROM booking WHERE slot_id = ? AND year * 100 + calendar_week >= ? AND deleted IS NULL",
            slot_id_vec,
            until,
        )
        //.fetch_all(self.pool.as_ref())
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(BookingEntity::try_from)
        .collect::<Result<Arc<[BookingEntity]>, DaoError>>()?
        )
    }

    async fn find_by_booking_data(
        &self,
        sales_person_id: Uuid,
        slot_id: Uuid,
        calendar_week: i32,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<Option<BookingEntity>, DaoError> {
        let sales_person_id_vec = sales_person_id.as_bytes().to_vec();
        let slot_id_vec = slot_id.as_bytes().to_vec();
        Ok(query_as!(
            BookingDb,
            "SELECT id, sales_person_id, slot_id, calendar_week, year, created, deleted, created_by, deleted_by, update_version FROM booking WHERE sales_person_id = ? AND slot_id = ? AND calendar_week = ? AND year = ? AND deleted IS NULL",
            sales_person_id_vec,
            slot_id_vec,
            calendar_week,
            year,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(BookingEntity::try_from)
        .transpose()?)
    }

    async fn find_by_week(
        &self,
        calendar_week: u8,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<Arc<[BookingEntity]>, DaoError> {
        Ok(query_as!(
            BookingDb,
            "SELECT id, sales_person_id, slot_id, calendar_week, year, created, deleted, created_by, deleted_by, update_version FROM booking WHERE calendar_week = ? AND year = ? AND deleted IS NULL",
            calendar_week,
            year,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(BookingEntity::try_from)
        .collect::<Result<Arc<[BookingEntity]>, DaoError>>()?
        )
    }

    async fn create(
        &self,
        entity: &BookingEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let sales_person_id_vec = entity.sales_person_id.as_bytes().to_vec();
        let slot_id_vec = entity.slot_id.as_bytes().to_vec();
        let created = entity.created.format(&Iso8601::DATE_TIME).map_db_error()?;
        let deleted = entity
            .deleted
            .as_ref()
            .map(|deleted| deleted.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let created_by = entity.created_by.as_ref().map(|s| s.as_ref());
        let deleted_by = entity.deleted_by.as_ref().map(|s| s.as_ref());
        let version_vec = entity.version.as_bytes().to_vec();
        query!("INSERT INTO booking (id, sales_person_id, slot_id, calendar_week, year, created, deleted, created_by, deleted_by, update_version, update_process) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id_vec, sales_person_id_vec, slot_id_vec, entity.calendar_week, entity.year, created, deleted, created_by, deleted_by, version_vec, process
        ).execute(tx.tx.lock().await.as_mut()).await.map_db_error()?;
        Ok(())
    }
    async fn update(
        &self,
        entity: &BookingEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let version_vec = entity.version.as_bytes().to_vec();
        let deleted = entity
            .deleted
            .as_ref()
            .map(|deleted| deleted.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let deleted_by = entity.deleted_by.as_ref().map(|s| s.as_ref());
        query!(
            "UPDATE booking SET deleted = ?, deleted_by = ?, update_version = ?, update_process = ? WHERE id = ?",
            deleted,
            deleted_by,
            version_vec,
            process,
            id_vec
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }
}
