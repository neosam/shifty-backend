use std::sync::Arc;

use crate::{ResultDbErrorExt, TransactionImpl};
use async_trait::async_trait;
use dao::{
    billing_period_sales_person::{BillingPeriodSalesPersonDao, BillingPeriodSalesPersonEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, OffsetDateTime, PrimitiveDateTime};
use uuid::Uuid;

#[derive(Debug)]
struct BillingPeriodSalesPersonDb {
    id: Vec<u8>,
    billing_period_id: Vec<u8>,
    sales_person_id: Vec<u8>,
    value_type: String,
    value_delta: f64,
    value_ytd_from: f64,
    value_ytd_to: f64,
    value_full_year: f64,
    created_at: String,
    created_by: String,
    deleted_at: Option<String>,
    deleted_by: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&BillingPeriodSalesPersonDb> for BillingPeriodSalesPersonEntity {
    type Error = DaoError;

    fn try_from(db: &BillingPeriodSalesPersonDb) -> Result<Self, Self::Error> {
        let created_at = PrimitiveDateTime::parse(&db.created_at, &Iso8601::DATE_TIME)?;
        let deleted_at = db
            .deleted_at
            .as_ref()
            .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
            .transpose()?;

        Ok(Self {
            id: Uuid::from_slice(&db.id).unwrap(),
            billing_period_id: Uuid::from_slice(&db.billing_period_id).unwrap(),
            sales_person_id: Uuid::from_slice(&db.sales_person_id).unwrap(),
            value_type: db.value_type.as_str().into(),
            value_delta: db.value_delta as f32,
            value_ytd_from: db.value_ytd_from as f32,
            value_ytd_to: db.value_ytd_to as f32,
            value_full_year: db.value_full_year as f32,
            created_at,
            created_by: db.created_by.as_str().into(),
            deleted_at,
            deleted_by: db.deleted_by.as_ref().map(|s| s.as_str().into()),
        })
    }
}

pub struct BillingPeriodSalesPersonDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl BillingPeriodSalesPersonDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl BillingPeriodSalesPersonDao for BillingPeriodSalesPersonDaoImpl {
    type Transaction = TransactionImpl;

    async fn dump_all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[BillingPeriodSalesPersonEntity]>, DaoError> {
        Ok(query_as!(
            BillingPeriodSalesPersonDb,
            "SELECT id, billing_period_id, sales_person_id, value_type, value_delta, value_ytd_from, value_ytd_to, value_full_year, created_at, created_by, deleted_at, deleted_by, update_version FROM billing_period_sales_person"
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(BillingPeriodSalesPersonEntity::try_from)
        .collect::<Result<Arc<[BillingPeriodSalesPersonEntity]>, DaoError>>()?)
    }

    async fn create(
        &self,
        entity: &BillingPeriodSalesPersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<BillingPeriodSalesPersonEntity, DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let billing_period_id_vec = entity.billing_period_id.as_bytes().to_vec();
        let sales_person_id_vec = entity.sales_person_id.as_bytes().to_vec();
        let value_type = entity.value_type.as_ref();
        let created_at = entity
            .created_at
            .format(&Iso8601::DATE_TIME)
            .map_db_error()?;
        let created_by = entity.created_by.as_ref();
        let deleted_at = entity
            .deleted_at
            .as_ref()
            .map(|deleted| deleted.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let deleted_by = entity.deleted_by.as_ref().map(|s| s.as_ref());
        let version_vec = Uuid::new_v4().as_bytes().to_vec();
        let value_delta = entity.value_delta as f64;
        let value_ytd_from = entity.value_ytd_from as f64;
        let value_ytd_to = entity.value_ytd_to as f64;
        let value_full_year = entity.value_full_year as f64;

        query!(
            "INSERT INTO billing_period_sales_person (id, billing_period_id, sales_person_id, value_type, value_delta, value_ytd_from, value_ytd_to, value_full_year, created_at, created_by, deleted_at, deleted_by, update_version, update_process) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id_vec,
            billing_period_id_vec,
            sales_person_id_vec,
            value_type,
            value_delta,
            value_ytd_from,
            value_ytd_to,
            value_full_year,
            created_at,
            created_by,
            deleted_at,
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
        entity: &BillingPeriodSalesPersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<BillingPeriodSalesPersonEntity, DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let value_type = entity.value_type.as_ref();
        let deleted_at = entity
            .deleted_at
            .as_ref()
            .map(|deleted| deleted.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let deleted_by = entity.deleted_by.as_ref().map(|s| s.as_ref());
        let version_vec = Uuid::new_v4().as_bytes().to_vec();
        let value_delta = entity.value_delta as f64;
        let value_ytd_from = entity.value_ytd_from as f64;
        let value_ytd_to = entity.value_ytd_to as f64;
        let value_full_year = entity.value_full_year as f64;

        query!(
            "UPDATE billing_period_sales_person SET value_type = ?, value_delta = ?, value_ytd_from = ?, value_ytd_to = ?, value_full_year = ?, deleted_at = ?, deleted_by = ?, update_version = ?, update_process = ? WHERE id = ?",
            value_type,
            value_delta,
            value_ytd_from,
            value_ytd_to,
            value_full_year,
            deleted_at,
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
            "UPDATE billing_period_sales_person SET deleted_at = ?, deleted_by = ?, update_version = ?, update_process = ? WHERE deleted_at IS NULL",
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
