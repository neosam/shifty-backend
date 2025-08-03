use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    sales_person_unavailable::{SalesPersonUnavailableDao, SalesPersonUnavailableEntity},
    DaoError,
};
use shifty_utils::DayOfWeek;
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

struct SalesPersonUnavailableDb {
    id: Vec<u8>,
    sales_person_id: Vec<u8>,
    year: i64,
    calendar_week: i64,
    day_of_week: i64,
    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&SalesPersonUnavailableDb> for SalesPersonUnavailableEntity {
    type Error = DaoError;

    fn try_from(entity: &SalesPersonUnavailableDb) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from_slice(&entity.id)?,
            sales_person_id: Uuid::from_slice(&entity.sales_person_id)?,
            year: entity.year as u32,
            calendar_week: entity.calendar_week as u8,
            day_of_week: DayOfWeek::from_number(entity.day_of_week as u8)
                .ok_or(DaoError::InvalidDayOfWeek(entity.day_of_week as u8))?,
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

pub struct SalesPersonUnavailableDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}
impl SalesPersonUnavailableDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl SalesPersonUnavailableDao for SalesPersonUnavailableDaoImpl {
    type Transaction = crate::TransactionImpl;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<SalesPersonUnavailableEntity>, DaoError> {
        let id = id.as_bytes().to_vec();
        Ok(query_as!(
            SalesPersonUnavailableDb,
            "SELECT id, sales_person_id, year, calendar_week, day_of_week, created, deleted, update_version FROM sales_person_unavailable WHERE id = ?",
            id
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(SalesPersonUnavailableEntity::try_from)
        .transpose()?)
    }

    async fn find_all_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[SalesPersonUnavailableEntity]>, DaoError> {
        let id = sales_person_id.as_bytes().to_vec();
        query_as!(
            SalesPersonUnavailableDb,
            "SELECT id, sales_person_id, year, calendar_week, day_of_week, created, deleted, update_version FROM sales_person_unavailable WHERE sales_person_id = ? AND deleted IS NULL", id 
        ).fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(SalesPersonUnavailableEntity::try_from)
        .collect::<Result<Arc<[SalesPersonUnavailableEntity]>, DaoError>>()
    }
    async fn find_by_week_and_sales_person_id(
        &self,
        sales_person_id: Uuid,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[SalesPersonUnavailableEntity]>, DaoError> {
        let id = sales_person_id.as_bytes().to_vec();
        query_as!(
            SalesPersonUnavailableDb,
            "SELECT id, sales_person_id, year, calendar_week, day_of_week, created, deleted, update_version FROM sales_person_unavailable WHERE sales_person_id = ? AND year = ? AND calendar_week = ? AND deleted IS NULL",
            id,
            year,
            calendar_week
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(SalesPersonUnavailableEntity::try_from)
        .collect::<Result<Arc<[SalesPersonUnavailableEntity]>, DaoError>>()
    }

    async fn find_by_week(
        &self,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[SalesPersonUnavailableEntity]>, DaoError> {
        query_as!(
            SalesPersonUnavailableDb,
            "SELECT id, sales_person_id, year, calendar_week, day_of_week, created, deleted, update_version FROM sales_person_unavailable WHERE year = ? AND calendar_week = ? AND deleted IS NULL",
            year,
            calendar_week
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(SalesPersonUnavailableEntity::try_from)
        .collect::<Result<Arc<[SalesPersonUnavailableEntity]>, DaoError>>()
    }

    async fn create(
        &self,
        entity: &SalesPersonUnavailableEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let sales_person_id = entity.sales_person_id.as_bytes().to_vec();
        let day_of_week = entity.day_of_week.to_number() as i64;
        let created = entity.created.format(&Iso8601::DATE_TIME).map_db_error()?;
        let deleted = entity.deleted.as_ref().map(|deleted| deleted.to_string());
        query!(
            r"INSERT INTO sales_person_unavailable (id, sales_person_id, year, calendar_week, day_of_week, created, deleted, update_version, update_process) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id,
            sales_person_id,
            entity.year,
            entity.calendar_week,
            day_of_week,
            created,
            deleted,
            version,
            process,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn update(
        &self,
        entity: &SalesPersonUnavailableEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let deleted = entity.deleted.as_ref().map(|deleted| deleted.to_string());
        query!(
            r"UPDATE sales_person_unavailable SET deleted = ?, update_version = ?, update_process = ? WHERE id = ?",
            deleted,
            version,
            process,
            id,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }
}
