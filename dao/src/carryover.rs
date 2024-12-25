use crate::DaoError;
use mockall::automock;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct CarryoverEntity {
    pub sales_person_id: Uuid,
    pub year: u32,
    pub carryover_hours: f32,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock]
#[async_trait::async_trait]
pub trait CarryoverDao {
    async fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
    ) -> Result<Option<CarryoverEntity>, DaoError>;

    async fn upsert(&self, entity: &CarryoverEntity, process: &str) -> Result<(), DaoError>;
}