use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use thiserror::Error;

pub mod booking;
pub mod extra_hours;
pub mod permission;
pub mod sales_person;
pub mod sales_person_unavailable;
pub mod shiftplan_report;
pub mod slot;
pub mod working_hours;

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
