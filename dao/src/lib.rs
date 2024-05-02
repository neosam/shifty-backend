use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use thiserror::Error;

pub mod permission;
pub mod slot;

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
}

#[automock]
#[async_trait]
pub trait HelloDao {
    async fn get_hello(&self) -> Result<Arc<str>, DaoError>;
}
