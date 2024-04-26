use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use thiserror::Error;

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

#[automock]
#[async_trait]
pub trait PermissionDao {
    async fn has_privilege(&self, user: &str, privilege: &str) -> Result<bool, DaoError>;
}
