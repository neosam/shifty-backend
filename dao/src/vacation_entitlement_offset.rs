use crate::DaoError;
use mockall::automock;
use uuid::Uuid;

/// Per-(sales_person, year) signed vacation-entitlement offset (Phase 28,
/// VAC-OFFSET-01 / D-28-01). Soft-delete aggregate: at most one active row
/// (`deleted IS NULL`) per (sales_person_id, year), enforced by a partial
/// unique index. Structurally mirrors `CarryoverEntity` but carries an
/// explicit `id` PK so REST can address a row directly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VacationEntitlementOffsetEntity {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub year: u32,
    pub offset_days: i32,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait VacationEntitlementOffsetDao {
    type Transaction: crate::Transaction;

    /// Returns the active (non-soft-deleted) offset row for the given
    /// person+year, if any.
    async fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<Option<VacationEntitlementOffsetEntity>, DaoError>;

    /// Returns the offset row with the given id, if any (regardless of
    /// soft-delete state).
    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<VacationEntitlementOffsetEntity>, DaoError>;

    async fn create(
        &self,
        entity: &VacationEntitlementOffsetEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn update(
        &self,
        entity: &VacationEntitlementOffsetEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
