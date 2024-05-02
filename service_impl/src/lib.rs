use std::sync::Arc;

use async_trait::async_trait;

pub mod clock;
pub mod permission;
pub mod slot;
mod test;
pub mod uuid_service;

pub use permission::PermissionServiceImpl;

pub struct HelloServiceImpl<HelloDao, PermissionService>
where
    HelloDao: dao::HelloDao + Sync + Send,
    PermissionService: service::PermissionService + Sync + Send,
{
    hello_dao: Arc<HelloDao>,
    permission_service: Arc<PermissionService>,
}
impl<HelloDao, PermissionService> HelloServiceImpl<HelloDao, PermissionService>
where
    HelloDao: dao::HelloDao + Sync + Send,
    PermissionService: service::PermissionService + Sync + Send,
{
    pub fn new(hello_dao: Arc<HelloDao>, permission_service: Arc<PermissionService>) -> Self {
        Self {
            hello_dao,
            permission_service,
        }
    }
}

impl<HelloDao, PermissionService> service::HelloService
    for HelloServiceImpl<HelloDao, PermissionService>
where
    HelloDao: dao::HelloDao + Sync + Send,
    PermissionService: service::PermissionService + Sync + Send,
{
    async fn hello(&self) -> Result<Arc<str>, service::ServiceError> {
        self.permission_service.check_permission("hello").await?;
        Ok(self.hello_dao.get_hello().await?)
    }
}

pub struct UserServiceDev;

#[async_trait]
impl service::UserService for UserServiceDev {
    async fn current_user(&self) -> Result<Arc<str>, service::ServiceError> {
        Ok("DEVUSER".into())
    }
}
