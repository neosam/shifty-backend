use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::{permission::Authentication, sales_person::SalesPerson, ServiceError};

#[automock(type Context=(); type Transaction = dao::MockTransaction;)]
#[async_trait]
pub trait SalesPersonShiftplanService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_shiftplans_for_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Vec<(Uuid, String)>, ServiceError>;

    async fn set_shiftplans_for_sales_person(
        &self,
        sales_person_id: Uuid,
        assignments: &[(Uuid, String)],
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn get_bookable_sales_persons(
        &self,
        shiftplan_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPerson]>, ServiceError>;

    async fn is_eligible(
        &self,
        sales_person_id: Uuid,
        shiftplan_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<bool, ServiceError>;
}
