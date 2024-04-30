use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use thiserror::Error;

mod permission;

pub use permission::MockPermissionDao;
pub use permission::PermissionDao;
pub use permission::PrivilegeEntity;
pub use permission::RoleEntity;
pub use permission::UserEntity;

#[derive(Error, Debug)]
pub enum DaoError {
    #[error("Database query error: {0}")]
    DatabaseQueryError(#[from] Box<dyn std::error::Error>),
}

#[automock]
#[async_trait]
pub trait HelloDao {
    async fn get_hello(&self) -> Result<Arc<str>, DaoError>;
}
