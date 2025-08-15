use std::sync::Arc;

use crate::{ResultDbErrorExt, TransactionImpl};
use async_trait::async_trait;
use dao::{
    billing_period::{BillingPeriodDao, BillingPeriodEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, OffsetDateTime, PrimitiveDateTime};
use uuid::Uuid;

#[derive(Debug)]
struct BillingPeriodDb {
    id: Vec<u8>,
    from_date_time: String,
    to_date_time: String,
    created: String,
    created_by: String,
    deleted: Option<String>,
    deleted_by: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&BillingPeriodDb> for BillingPeriodEntity {
    type Error = DaoError;

    fn try_from(db: &BillingPeriodDb) -> Result<Self, Self::Error> {
        let created_at = PrimitiveDateTime::parse(&db.created, &Iso8601::DATE_TIME)?;
        let deleted_at = db
            .deleted
            .as_ref()
            .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
            .transpose()?;

        let start_date = PrimitiveDateTime::parse(&db.from_date_time, &Iso8601::DATE_TIME)?.date();
        let end_date = PrimitiveDateTime::parse(&db.to_date_time, &Iso8601::DATE_TIME)?.date();

        Ok(Self {
            id: Uuid::from_slice(&db.id).unwrap(),
            start_date,
            end_date,
            created_at,
            created_by: db.created_by.as_str().into(),
            deleted_at,
            deleted_by: db.deleted_by.as_ref().map(|s| s.as_str().into()),
        })
    }
}

pub struct BillingPeriodDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl BillingPeriodDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl BillingPeriodDao for BillingPeriodDaoImpl {
    type Transaction = TransactionImpl;

    async fn dump_all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[BillingPeriodEntity]>, DaoError> {
        Ok(query_as!(
            BillingPeriodDb,
            "SELECT id, from_date_time, to_date_time, created, created_by, deleted, deleted_by, update_version FROM billing_period"
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(BillingPeriodEntity::try_from)
        .collect::<Result<Arc<[BillingPeriodEntity]>, DaoError>>()?)
    }

    async fn create(
        &self,
        entity: &BillingPeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<BillingPeriodEntity, DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let from_date_time = entity
            .start_date
            .with_hms(0, 0, 0)
            .unwrap()
            .assume_utc()
            .format(&Iso8601::DATE_TIME)
            .map_db_error()?;
        let to_date_time = entity
            .end_date
            .with_hms(23, 59, 59)
            .unwrap()
            .assume_utc()
            .format(&Iso8601::DATE_TIME)
            .map_db_error()?;
        let created = entity
            .created_at
            .format(&Iso8601::DATE_TIME)
            .map_db_error()?;
        let created_by = entity.created_by.as_ref();
        let deleted = entity
            .deleted_at
            .as_ref()
            .map(|deleted| deleted.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let deleted_by = entity.deleted_by.as_ref().map(|s| s.as_ref());
        let version_vec = Uuid::new_v4().as_bytes().to_vec();

        query!(
            "INSERT INTO billing_period (id, from_date_time, to_date_time, created, created_by, deleted, deleted_by, update_version, update_process) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id_vec,
            from_date_time,
            to_date_time,
            created,
            created_by,
            deleted,
            deleted_by,
            version_vec,
            process
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(entity.clone())
    }

    async fn update(
        &self,
        entity: &BillingPeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<BillingPeriodEntity, DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let from_date_time = entity
            .start_date
            .with_hms(0, 0, 0)
            .unwrap()
            .assume_utc()
            .format(&Iso8601::DATE_TIME)
            .map_db_error()?;
        let to_date_time = entity
            .end_date
            .with_hms(23, 59, 59)
            .unwrap()
            .assume_utc()
            .format(&Iso8601::DATE_TIME)
            .map_db_error()?;
        let deleted = entity
            .deleted_at
            .as_ref()
            .map(|deleted| deleted.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let deleted_by = entity.deleted_by.as_ref().map(|s| s.as_ref());
        let version_vec = Uuid::new_v4().as_bytes().to_vec();

        query!(
            "UPDATE billing_period SET from_date_time = ?, to_date_time = ?, deleted = ?, deleted_by = ?, update_version = ?, update_process = ? WHERE id = ?",
            from_date_time,
            to_date_time,
            deleted,
            deleted_by,
            version_vec,
            process,
            id_vec
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(entity.clone())
    }

    async fn clear_all(
        &self,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let now = OffsetDateTime::now_utc()
            .format(&Iso8601::DATE_TIME)
            .map_db_error()?;
        let version_vec = Uuid::new_v4().as_bytes().to_vec();

        query!(
            "UPDATE billing_period SET deleted = ?, deleted_by = ?, update_version = ?, update_process = ? WHERE deleted IS NULL",
            now,
            process,
            version_vec,
            process
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }
}
