use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

#[derive(Clone, Debug, PartialEq)]
pub struct BillingPeriodSalesPersonEntity {
    pub id: uuid::Uuid,
    pub billing_period_id: uuid::Uuid,
    pub sales_person_id: uuid::Uuid,

    pub value_type: Arc<str>,
    pub value_delta: f32,
    pub value_ytd_from: f32,
    pub value_ytd_to: f32,
    pub value_full_year: f32,

    pub created_at: time::PrimitiveDateTime,
    pub created_by: Arc<str>,
    pub deleted_at: Option<time::PrimitiveDateTime>,
    pub deleted_by: Option<Arc<str>>,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait BillingPeriodSalesPersonDao {
    type Transaction: crate::Transaction;

    async fn dump_all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[BillingPeriodSalesPersonEntity]>, crate::DaoError>;
    async fn create(
        &self,
        entity: &BillingPeriodSalesPersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<BillingPeriodSalesPersonEntity, crate::DaoError>;
    async fn update(
        &self,
        entity: &BillingPeriodSalesPersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<BillingPeriodSalesPersonEntity, crate::DaoError>;
    async fn clear_all(
        &self,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), crate::DaoError>;

    async fn all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[BillingPeriodSalesPersonEntity]>, crate::DaoError> {
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
    ) -> Result<Option<BillingPeriodSalesPersonEntity>, crate::DaoError> {
        self.all(tx)
            .await?
            .iter()
            .find(|bp| bp.id == id)
            .map_or(Ok(None), |bp| Ok(Some(bp.clone())))
    }

    async fn find_by_billing_period_and_sales_person(
        &self,
        billing_period_id: uuid::Uuid,
        sales_person_id: uuid::Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[BillingPeriodSalesPersonEntity]>, crate::DaoError> {
        Ok(self
            .all(tx)
            .await?
            .iter()
            .filter(|bp| {
                bp.billing_period_id == billing_period_id && bp.sales_person_id == sales_person_id
            })
            .cloned()
            .collect())
    }
}
