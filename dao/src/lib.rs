use std::{future::Future, sync::Arc};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DaoError {
    #[error("Database query error: {0}")]
    DatabaseQueryError(#[from] Box<dyn std::error::Error>),
}

pub trait HelloDao {
    fn get_hello(&self) -> impl Future<Output = Result<Arc<str>, DaoError>> + Send;
}

pub trait PermissionDao {
    fn has_privilege(
        &self,
        user: &str,
        privilege: &str,
    ) -> impl Future<Output = Result<bool, DaoError>> + Send;
}
