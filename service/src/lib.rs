use async_trait::async_trait;
use mockall::automock;
use std::{future::Future, sync::Arc};
use thiserror::Error;

pub mod permission;

pub use permission::MockPermissionService;
pub use permission::PermissionService;
pub use permission::Privilege;
pub use permission::Role;
pub use permission::User;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Database query error: {0}")]
    DatabaseQueryError(#[from] dao::DaoError),

    #[error("Forbidden")]
    Forbidden,
}

#[automock]
pub trait HelloService {
    fn hello(&self) -> impl Future<Output = Result<Arc<str>, ServiceError>> + Send;
}

#[automock]
#[async_trait]
pub trait UserService {
    async fn current_user(&self) -> Result<Arc<str>, ServiceError>;
}
