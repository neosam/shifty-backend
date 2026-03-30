use std::sync::Arc;

use async_trait::async_trait;
use dao::{shiftplan::ShiftplanEntity, DaoError};
use sqlx::{query, query_as};
use time::format_description::well_known::Iso8601;
use uuid::Uuid;

use crate::ResultDbErrorExt;

pub struct ShiftplanDb {
    pub id: Vec<u8>,
    pub name: String,
    pub is_planning: i64,
    pub deleted: Option<String>,
    pub update_version: Vec<u8>,
}

impl TryFrom<&ShiftplanDb> for ShiftplanEntity {
    type Error = DaoError;
    fn try_from(db: &ShiftplanDb) -> Result<Self, DaoError> {
        Ok(Self {
            id: Uuid::from_slice(&db.id)?,
            name: db.name.as_str().into(),
            is_planning: db.is_planning != 0,
            deleted: db
                .deleted
                .as_ref()
                .map(|d| time::PrimitiveDateTime::parse(d, &Iso8601::DATE))
                .transpose()?,
            version: Uuid::from_slice(&db.update_version)?,
        })
    }
}

pub struct ShiftplanDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl ShiftplanDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl dao::shiftplan::ShiftplanDao for ShiftplanDaoImpl {
    type Transaction = crate::TransactionImpl;

    async fn all(&self, tx: Self::Transaction) -> Result<Arc<[ShiftplanEntity]>, DaoError> {
        Ok(query_as!(
            ShiftplanDb,
            r"SELECT id, name, is_planning, deleted, update_version FROM shiftplan WHERE deleted IS NULL"
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(ShiftplanEntity::try_from)
        .collect::<Result<Arc<[_]>, _>>()?)
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<ShiftplanEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        let result = query_as!(
            ShiftplanDb,
            r"SELECT id, name, is_planning, deleted, update_version FROM shiftplan WHERE id = ? AND deleted IS NULL",
            id_vec
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        result
            .as_ref()
            .map(ShiftplanEntity::try_from)
            .transpose()
    }

    async fn create(
        &self,
        entity: &ShiftplanEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let version_vec = entity.version.as_bytes().to_vec();
        let name = entity.name.as_ref();
        let is_planning = entity.is_planning as i32;
        query!(
            "INSERT INTO shiftplan (id, name, is_planning, deleted, update_process, update_version) VALUES (?, ?, ?, NULL, ?, ?)",
            id_vec,
            name,
            is_planning,
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
        entity: &ShiftplanEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let version_vec = entity.version.as_bytes().to_vec();
        let name = entity.name.as_ref();
        let is_planning = entity.is_planning as i32;
        let deleted = entity.deleted.as_ref().map(|d| d.to_string());
        query!(
            "UPDATE shiftplan SET name = ?, is_planning = ?, deleted = ?, update_process = ?, update_version = ? WHERE id = ?",
            name,
            is_planning,
            deleted,
            process,
            version_vec,
            id_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }
}
