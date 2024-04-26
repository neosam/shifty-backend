use std::sync::Arc;

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

pub struct PermissionServiceImpl<PermissionDao>
where
    PermissionDao: dao::PermissionDao + Send + Sync,
{
    permission_dao: Arc<PermissionDao>,
}
impl<PermissionDao> PermissionServiceImpl<PermissionDao>
where
    PermissionDao: dao::PermissionDao + Send + Sync,
{
    pub fn new(permission_dao: Arc<PermissionDao>) -> Self {
        Self { permission_dao }
    }
}

impl<PermissionDao> service::PermissionService for PermissionServiceImpl<PermissionDao>
where
    PermissionDao: dao::PermissionDao + Send + Sync,
{
    async fn check_permission(&self, privilege: &str) -> Result<(), service::ServiceError> {
        let current_user = self.current_user().await?;
        if self
            .permission_dao
            .has_privilege(current_user.as_ref(), privilege)
            .await?
        {
            Ok(())
        } else {
            Err(service::ServiceError::Forbidden)
        }
    }

    async fn current_user(&self) -> Result<Arc<str>, service::ServiceError> {
        Ok("DEVUSER".into())
    }
}
