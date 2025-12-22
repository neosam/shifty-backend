use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillingPeriodEntity {
    pub id: uuid::Uuid,
    pub start_date: time::Date,
    pub end_date: time::Date,

    pub created_at: time::PrimitiveDateTime,
    pub created_by: Arc<str>,
    pub deleted_at: Option<time::PrimitiveDateTime>,
    pub deleted_by: Option<Arc<str>>,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait BillingPeriodDao {
    type Transaction: crate::Transaction;

    async fn dump_all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[BillingPeriodEntity]>, crate::DaoError>;
    async fn create(
        &self,
        entity: &BillingPeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<BillingPeriodEntity, crate::DaoError>;
    async fn update(
        &self,
        entity: &BillingPeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<BillingPeriodEntity, crate::DaoError>;
    async fn clear_all(
        &self,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), crate::DaoError>;

    async fn all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[BillingPeriodEntity]>, crate::DaoError> {
        Ok(self
            .dump_all(tx)
            .await?
            .iter()
            .filter(|bp| bp.deleted_at.is_none())
            .cloned()
            .collect())
    }

    async fn find_by_id(
        &self,
        id: uuid::Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<BillingPeriodEntity>, crate::DaoError> {
        self.all(tx)
            .await?
            .iter()
            .find(|bp| bp.id == id)
            .map_or(Ok(None), |bp| Ok(Some(bp.clone())))
    }

    async fn find_latest_end_date(
        &self,
        tx: Self::Transaction,
    ) -> Result<Option<time::Date>, crate::DaoError> {
        self.all(tx)
            .await?
            .iter()
            .map(|bp| bp.end_date)
            .max()
            .map_or(Ok(None), |date| Ok(Some(date)))
    }
}
