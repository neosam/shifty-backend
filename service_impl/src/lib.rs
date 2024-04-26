use std::sync::Arc;

use async_trait::async_trait;

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

pub struct PermissionServiceImpl<PermissionDao, UserService>
where
    PermissionDao: dao::PermissionDao + Send + Sync,
    UserService: service::UserService + Send + Sync,
{
    permission_dao: Arc<PermissionDao>,
    user_service: Arc<UserService>,
}
impl<PermissionDao, UserService> PermissionServiceImpl<PermissionDao, UserService>
where
    PermissionDao: dao::PermissionDao + Send + Sync,
    UserService: service::UserService + Send + Sync,
{
    pub fn new(permission_dao: Arc<PermissionDao>, user_service: Arc<UserService>) -> Self {
        Self {
            permission_dao,
            user_service,
        }
    }
}

#[async_trait]
impl<PermissionDao, UserService> service::PermissionService
    for PermissionServiceImpl<PermissionDao, UserService>
where
    PermissionDao: dao::PermissionDao + Send + Sync,
    UserService: service::UserService + Send + Sync,
{
    async fn check_permission(&self, privilege: &str) -> Result<(), service::ServiceError> {
        let current_user = self.user_service.current_user().await?;
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
}

pub struct UserServiceDev;

#[async_trait]
impl service::UserService for UserServiceDev {
    async fn current_user(&self) -> Result<Arc<str>, service::ServiceError> {
        Ok("DEVUSER".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::eq;
    use service::{HelloService, MockPermissionService, PermissionService};
    use tokio;

    #[tokio::test]
    async fn test_get_hello_successful() {
        let mut hello_dao = dao::MockHelloDao::new();
        hello_dao
            .expect_get_hello()
            .times(1)
            .returning(|| Ok("Hello, world!".into()));
        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .times(1)
            .returning(|_| Ok(()));

        let hello_service =
            HelloServiceImpl::new(Arc::new(hello_dao), Arc::new(permission_service));
        assert_eq!(
            "Hello, world!",
            hello_service.hello().await.unwrap().as_ref()
        );
    }

    #[tokio::test]
    async fn test_get_hello_no_permission() {
        let hello_dao = dao::MockHelloDao::new();

        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .times(1)
            .returning(|_| Err(service::ServiceError::Forbidden));

        let hello_service =
            HelloServiceImpl::new(Arc::new(hello_dao), Arc::new(permission_service));
        if let Err(service::ServiceError::Forbidden) = hello_service.hello().await {
            // All good
        } else {
            panic!("Expected forbidden error");
        }
    }

    #[tokio::test]
    async fn test_check_permission() {
        let mut permission_dao = dao::MockPermissionDao::new();
        permission_dao
            .expect_has_privilege()
            .with(eq("DEVUSER"), eq("hello"))
            .returning(|_, _| Ok(true));

        let mut user_service = service::MockUserService::new();
        user_service
            .expect_current_user()
            .returning(|| Ok("DEVUSER".into()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        let result = permission_service.check_permission("hello").await;
        result.expect("Expected successful authorization");
    }

    #[tokio::test]
    async fn test_check_permission_denied() {
        let mut permission_dao = dao::MockPermissionDao::new();
        permission_dao
            .expect_has_privilege()
            .with(eq("DEVUSER"), eq("hello"))
            .returning(|_, _| Ok(false));

        let mut user_service = service::MockUserService::new();
        user_service
            .expect_current_user()
            .returning(|| Ok("DEVUSER".into()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        let result = permission_service.check_permission("hello").await;
        if let Err(service::ServiceError::Forbidden) = result {
            // All good
        } else {
            panic!("Expected forbidden error");
        }
    }

    #[tokio::test]
    async fn test_user_service_dev() {
        use service::UserService;
        let user_service = UserServiceDev;
        assert_eq!(
            "DEVUSER",
            user_service.current_user().await.unwrap().as_ref()
        );
    }
}
