use crate::DaoError;
use mockall::automock;
use uuid::Uuid;

/// Persisted KW-status discriminant. Deliberately WITHOUT an `Unset` variant:
/// on the DAO layer `Unset` == row absence (D-39-04). The persisted variant is
/// named `WeekStatusKind` (not `None`) to avoid Option-shadowing (D-39-03).
#[derive(Clone, Debug, PartialEq)]
pub enum WeekStatusKind {
    InPlanning,
    Planned,
    Locked,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeekStatusEntity {
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub status: WeekStatusKind,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait WeekStatusDao {
    type Transaction: crate::Transaction;

    async fn find_by_year_and_week(
        &self,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Option<WeekStatusEntity>, DaoError>;

    async fn create(
        &self,
        entity: &WeekStatusEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn update(
        &self,
        entity: &WeekStatusEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn delete(&self, id: Uuid, process: &str, tx: Self::Transaction) -> Result<(), DaoError>;
}
