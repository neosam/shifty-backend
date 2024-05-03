use std::sync::Arc;

use async_trait::async_trait;

pub mod clock;
pub mod permission;
pub mod slot;
mod test;
pub mod uuid_service;

pub use permission::PermissionServiceImpl;

pub struct UserServiceDev;

#[async_trait]
impl service::user_service::UserService for UserServiceDev {
    type Context = ();

    async fn current_user(
        &self,
        _context: Self::Context,
    ) -> Result<Arc<str>, service::ServiceError> {
        Ok("DEVUSER".into())
    }
}
