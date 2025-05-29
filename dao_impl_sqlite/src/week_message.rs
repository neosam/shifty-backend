use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    week_message::{WeekMessageDao, WeekMessageEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

#[derive(Debug)]
struct WeekMessageDb {
    id: Vec<u8>,
    year: i64,
    calendar_week: i64,
    message: String,
    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&WeekMessageDb> for WeekMessageEntity {
    type Error = DaoError;

    fn try_from(db: &WeekMessageDb) -> Result<Self, Self::Error> {
        Ok(WeekMessageEntity {
            id: Uuid::from_slice(&db.id)?,
            year: db.year as u32,
            calendar_week: db.calendar_week as u8,
            message: db.message.clone(),
            created: PrimitiveDateTime::parse(&db.created, &Iso8601::DATE_TIME)?,
            deleted: db
                .deleted
                .as_ref()
                .map(|del| PrimitiveDateTime::parse(del, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&db.update_version)?,
        })
    }
}

pub struct WeekMessageDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl WeekMessageDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl WeekMessageDao for WeekMessageDaoImpl {
    type Transaction = crate::TransactionImpl;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<WeekMessageEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            WeekMessageDb,
            r#"SELECT id, year, calendar_week, message, created, deleted, update_version
               FROM week_message
               WHERE id = ? AND deleted IS NULL"#,
            id_vec,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(WeekMessageEntity::try_from)
        .transpose()?)
    }

    async fn find_by_year_and_week(
        &self,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Option<WeekMessageEntity>, DaoError> {
        Ok(query_as!(
            WeekMessageDb,
            r#"SELECT id, year, calendar_week, message, created, deleted, update_version
               FROM week_message
               WHERE year = ? AND calendar_week = ? AND deleted IS NULL"#,
            year,
            calendar_week,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(WeekMessageEntity::try_from)
        .transpose()?)
    }

    async fn find_by_year(
        &self,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<Vec<WeekMessageEntity>, DaoError> {
        let rows = query_as!(
            WeekMessageDb,
            r#"SELECT id, year, calendar_week, message, created, deleted, update_version
               FROM week_message
               WHERE year = ? AND deleted IS NULL
               ORDER BY calendar_week"#,
            year,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        let mut entities = Vec::new();
        for row in &rows {
            entities.push(WeekMessageEntity::try_from(row)?);
        }
        Ok(entities)
    }

    async fn create(
        &self,
        entity: &WeekMessageEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let created_str = entity.created.format(&Iso8601::DATE_TIME).map_db_error()?;
        let deleted_str = entity
            .deleted
            .map(|del| del.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let version_vec = entity.version.as_bytes().to_vec();

        query!(
            r#"INSERT INTO week_message (id, year, calendar_week, message, created, deleted, update_process, update_version)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
            id_vec,
            entity.year,
            entity.calendar_week,
            entity.message,
            created_str,
            deleted_str,
            process,
            version_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }

    async fn update(
        &self,
        entity: &WeekMessageEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let created_str = entity.created.format(&Iso8601::DATE_TIME).map_db_error()?;
        let deleted_str = entity
            .deleted
            .map(|del| del.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let version_vec = entity.version.as_bytes().to_vec();

        query!(
            r#"UPDATE week_message 
               SET message = ?, created = ?, deleted = ?, update_process = ?, update_version = ?
               WHERE id = ?"#,
            entity.message,
            created_str,
            deleted_str,
            process,
            version_vec,
            id_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }

    async fn delete(&self, id: Uuid, process: &str, tx: Self::Transaction) -> Result<(), DaoError> {
        let id_vec = id.as_bytes().to_vec();
        let now_str = time::OffsetDateTime::now_utc()
            .format(&Iso8601::DATE_TIME)
            .map_db_error()?;

        query!(
            r#"UPDATE week_message 
               SET deleted = ?, update_process = ?
               WHERE id = ?"#,
            now_str,
            process,
            id_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }
}
