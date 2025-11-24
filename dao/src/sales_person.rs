use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::DaoError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SalesPersonEntity {
    pub id: Uuid,
    pub name: Arc<str>,
    pub background_color: Arc<str>,
    pub is_paid: bool,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub inactive: bool,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait SalesPersonDao {
    type Transaction: crate::Transaction;

    async fn all(&self, tx: Self::Transaction) -> Result<Arc<[SalesPersonEntity]>, DaoError>;
    async fn all_paid(&self, tx: Self::Transaction) -> Result<Arc<[SalesPersonEntity]>, DaoError>;
    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<SalesPersonEntity>, DaoError>;
    async fn find_by_user(
        &self,
        user_id: &str,
        tx: Self::Transaction,
    ) -> Result<Option<SalesPersonEntity>, DaoError>;
    async fn create(
        &self,
        entity: &SalesPersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    async fn update(
        &self,
        entity: &SalesPersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    async fn get_assigned_user(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<Arc<str>>, DaoError>;
    async fn get_all_user_assignments(
        &self,
        tx: Self::Transaction,
    ) -> Result<HashMap<Uuid, Arc<str>>, DaoError>;
    async fn assign_to_user(
        &self,
        sales_person_id: Uuid,
        user_id: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    async fn discard_assigned_user(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    async fn find_sales_person_by_user_id(
        &self,
        user_id: &str,
        tx: Self::Transaction,
    ) -> Result<Option<SalesPersonEntity>, DaoError>;
}
