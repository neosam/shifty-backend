use std::{future::Future, sync::Arc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Database query error: {0}")]
    DatabaseQueryError(#[from] dao::DaoError),

    #[error("Forbidden")]
    Forbidden,
}

pub trait HelloService {
    fn hello(&self) -> impl Future<Output = Result<Arc<str>, ServiceError>> + Send;
}

pub trait PermissionService {
    fn check_permission(
        &self,
        privilege: &str,
    ) -> impl Future<Output = Result<(), ServiceError>> + Send;

    fn current_user(&self) -> impl Future<Output = Result<Arc<str>, ServiceError>> + Send;
}
