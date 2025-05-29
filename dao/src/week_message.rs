use crate::DaoError;
use mockall::automock;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct WeekMessageEntity {
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub message: String,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait WeekMessageDao {
    type Transaction: crate::Transaction;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<WeekMessageEntity>, DaoError>;

    async fn find_by_year_and_week(
        &self,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Option<WeekMessageEntity>, DaoError>;

    async fn find_by_year(
        &self,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<Vec<WeekMessageEntity>, DaoError>;

    async fn create(
        &self,
        entity: &WeekMessageEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn update(
        &self,
        entity: &WeekMessageEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn delete(&self, id: Uuid, process: &str, tx: Self::Transaction) -> Result<(), DaoError>;
}
