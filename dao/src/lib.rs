use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use thiserror::Error;

pub mod billing_period;
pub mod billing_period_sales_person;
pub mod booking;
pub mod carryover;
pub mod custom_extra_hours;
pub mod employee_work_details;
pub mod extra_hours;
pub mod permission;
pub mod sales_person;
pub mod sales_person_unavailable;
pub mod session;
pub mod shiftplan_report;
pub mod slot;
pub mod special_day;
pub mod week_message;

pub use permission::MockPermissionDao;
pub use permission::PermissionDao;
pub use permission::PrivilegeEntity;
pub use permission::RoleEntity;
pub use permission::UserEntity;

#[derive(Error, Debug)]
pub enum DaoError {
    #[error("Database query error: {0}")]
    DatabaseQueryError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Uuid error: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Invalid day of week number: {0}")]
    InvalidDayOfWeek(u8),

    #[error("Date/Time parse error: {0}")]
    DateTimeParseError(#[from] time::error::Parse),

    #[error("Date/Time format error: {0}")]
    DateTimeFormatError(#[from] time::error::Format),

    #[error("Component range error: {0}")]
    ComponentRangeError(#[from] time::error::ComponentRange),

    #[error("Enum value not found: {0}")]
    EnumValueNotFound(Arc<str>),
}

#[automock]
#[async_trait]
pub trait HelloDao {
    async fn get_hello(&self) -> Result<Arc<str>, DaoError>;
}

#[automock]
#[async_trait]
pub trait BasicDao {
    async fn clear_all(&self) -> Result<(), DaoError>;
}

#[must_use]
pub trait Transaction: Clone + Send + Sync {}
#[derive(Clone, Debug)]
pub struct MockTransaction;
impl Transaction for MockTransaction {}

#[automock(type Transaction = MockTransaction;)]
#[async_trait]
pub trait TransactionDao {
    type Transaction: Transaction;

    async fn new_transaction(&self) -> Result<Self::Transaction, DaoError>;
    async fn use_transaction(
        &self,
        tx: Option<Self::Transaction>,
    ) -> Result<Self::Transaction, DaoError>;
    async fn commit(&self, transaction: Self::Transaction) -> Result<(), DaoError>;
}
