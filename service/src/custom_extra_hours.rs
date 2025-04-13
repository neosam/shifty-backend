use std::sync::Arc;

use crate::{permission::Authentication, ServiceError};
use async_trait::async_trait;
use dao::custom_extra_hours::CustomExtraHoursEntity;
use mockall::automock;
use std::fmt::Debug;
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomExtraHours {
    pub id: Uuid,
    pub name: Arc<str>,
    pub description: Option<Arc<str>>,
    pub modifies_balance: bool,
    pub assigned_sales_person_ids: Arc<[Uuid]>,
    pub created: Option<PrimitiveDateTime>,
    pub deleted: Option<PrimitiveDateTime>,
    pub version: Uuid,
}

impl From<&CustomExtraHoursEntity> for CustomExtraHours {
    fn from(entity: &CustomExtraHoursEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name.clone(),
            description: entity.description.clone(),
            modifies_balance: entity.modifies_balance,
            assigned_sales_person_ids: entity.assigned_sales_person_ids.clone(),
            created: Some(entity.created),
            deleted: entity.deleted,
            version: entity.version,
        }
    }
}
shifty_utils::derive_from_reference!(CustomExtraHoursEntity, CustomExtraHours);

impl TryFrom<&CustomExtraHours> for CustomExtraHoursEntity {
    type Error = crate::ServiceError;

    fn try_from(entity: &CustomExtraHours) -> Result<Self, Self::Error> {
        Ok(Self {
            id: entity.id,
            name: entity.name.clone(),
            description: entity.description.clone(),
            modifies_balance: entity.modifies_balance,
            assigned_sales_person_ids: entity.assigned_sales_person_ids.clone(),
            created: entity
                .created
                .ok_or_else(|| crate::ServiceError::InternalError)?,
            deleted: entity.deleted,
            version: entity.version,
        })
    }
}
shifty_utils::derive_try_from_reference!(CustomExtraHours, CustomExtraHoursEntity, ServiceError);

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait CustomExtraHoursService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[CustomExtraHours]>, crate::ServiceError>;

    async fn get_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CustomExtraHours, crate::ServiceError>;

    async fn get_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[CustomExtraHours]>, crate::ServiceError>;

    async fn create(
        &self,
        entity: &CustomExtraHours,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CustomExtraHours, crate::ServiceError>;

    async fn update(
        &self,
        entity: &CustomExtraHours,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CustomExtraHours, crate::ServiceError>;

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), crate::ServiceError>;
}
