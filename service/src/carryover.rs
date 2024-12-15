use crate::permission::Authentication;
use crate::ServiceError;
use async_trait::async_trait;
use mockall::automock;
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct Carryover {
    pub sales_person_id: Uuid,
    pub year: u32,
    pub carryover_hours: f32,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

impl From<&dao::carryover::CarryoverEntity> for Carryover {
    fn from(entity: &dao::carryover::CarryoverEntity) -> Self {
        Self {
            sales_person_id: entity.sales_person_id,
            year: entity.year,
            carryover_hours: entity.carryover_hours,
            created: entity.created,
            deleted: entity.deleted,
            version: entity.version,
        }
    }
}

impl TryFrom<&Carryover> for dao::carryover::CarryoverEntity {
    type Error = ServiceError;
    fn try_from(c: &Carryover) -> Result<Self, Self::Error> {
        Ok(Self {
            sales_person_id: c.sales_person_id,
            year: c.year,
            carryover_hours: c.carryover_hours,
            created: c.created,
            deleted: c.deleted,
            version: c.version,
        })
    }
}

#[automock(type Context=();)]
#[async_trait]
pub trait CarryoverService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

    async fn get_carryover(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
    ) -> Result<Option<Carryover>, ServiceError>;

    async fn set_carryover(
        &self,
        carryover: &Carryover,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
}
