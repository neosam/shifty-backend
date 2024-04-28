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

const PERMISSION_SERVICE_PROCESS: &str = "permission-service";

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

    async fn create_user(&self, user: &str) -> Result<(), service::ServiceError> {
        self.check_permission("admin").await?;
        self.permission_dao
            .create_user(
                &dao::UserEntity { name: user.into() },
                PERMISSION_SERVICE_PROCESS,
            )
            .await?;
        Ok(())
    }
    async fn delete_user(&self, user: &str) -> Result<(), service::ServiceError> {
        self.check_permission("admin").await?;
        self.permission_dao.delete_user(user).await?;
        Ok(())
    }

    async fn get_all_users(&self) -> Result<Arc<[service::User]>, service::ServiceError> {
        self.check_permission("admin").await?;
        Ok(self
            .permission_dao
            .all_users()
            .await?
            .into_iter()
            .map(service::User::from)
            .collect())
    }

    async fn create_role(&self, role: &str) -> Result<(), service::ServiceError> {
        self.check_permission("admin").await?;
        self.permission_dao
            .create_role(
                &dao::RoleEntity { name: role.into() },
                PERMISSION_SERVICE_PROCESS,
            )
            .await?;
        Ok(())
    }
    async fn delete_role(&self, role: &str) -> Result<(), service::ServiceError> {
        self.check_permission("admin").await?;
        self.permission_dao.delete_role(role).await?;
        Ok(())
    }
    async fn get_all_roles(&self) -> Result<Arc<[service::Role]>, service::ServiceError> {
        self.check_permission("admin").await?;
        Ok(self
            .permission_dao
            .all_roles()
            .await?
            .iter()
            .map(service::Role::from)
            .collect())
    }

    async fn create_privilege(&self, privilege: &str) -> Result<(), service::ServiceError> {
        self.check_permission("admin").await?;
        self.permission_dao
            .create_privilege(
                &dao::PrivilegeEntity {
                    name: privilege.into(),
                },
                PERMISSION_SERVICE_PROCESS,
            )
            .await?;
        Ok(())
    }

    async fn delete_privilege(&self, privilege: &str) -> Result<(), service::ServiceError> {
        self.check_permission("admin").await?;
        self.permission_dao.delete_privilege(privilege).await?;
        Ok(())
    }
    async fn get_all_privileges(&self) -> Result<Arc<[service::Privilege]>, service::ServiceError> {
        self.check_permission("admin").await?;
        Ok(self
            .permission_dao
            .all_privileges()
            .await?
            .iter()
            .map(service::Privilege::from)
            .collect())
    }

    async fn add_user_role(&self, user: &str, role: &str) -> Result<(), service::ServiceError> {
        self.check_permission("admin").await?;
        self.permission_dao
            .add_user_role(user, role, PERMISSION_SERVICE_PROCESS)
            .await?;
        Ok(())
    }
    async fn add_role_privilege(
        &self,
        role: &str,
        privilege: &str,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin").await?;
        self.permission_dao
            .add_role_privilege(role, privilege, PERMISSION_SERVICE_PROCESS)
            .await?;
        Ok(())
    }
    async fn delete_role_privilege(
        &self,
        role: &str,
        privilege: &str,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin").await?;
        self.permission_dao
            .delete_role_privilege(role, privilege)
            .await?;
        Ok(())
    }
    async fn delete_user_role(&self, user: &str, role: &str) -> Result<(), service::ServiceError> {
        self.check_permission("admin").await?;
        self.permission_dao.delete_user_role(user, role).await?;
        Ok(())
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

    fn test_forbidden<T>(result: &Result<T, service::ServiceError>) {
        if let Err(service::ServiceError::Forbidden) = result {
            // All good
        } else {
            panic!("Expected forbidden error");
        }
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
        test_forbidden(&hello_service.hello().await);
    }

    fn generate_dependencies_mocks_permission(
        grant: bool,
        privilege: &'static str,
    ) -> (dao::MockPermissionDao, service::MockUserService) {
        let mut permission_dao = dao::MockPermissionDao::new();
        permission_dao
            .expect_has_privilege()
            .with(eq("DEVUSER"), eq(privilege))
            .returning(move |_, _| Ok(grant));

        let mut user_service = service::MockUserService::new();
        user_service
            .expect_current_user()
            .returning(|| Ok("DEVUSER".into()));
        (permission_dao, user_service)
    }

    #[tokio::test]
    async fn test_check_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(true, "hello");

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        let result = permission_service.check_permission("hello").await;
        result.expect("Expected successful authorization");
    }

    #[tokio::test]
    async fn test_check_permission_denied() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "hello");

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        let result = permission_service.check_permission("hello").await;
        test_forbidden(&result);
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

    #[tokio::test]
    async fn test_create_user() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao
            .expect_create_user()
            .with(
                eq(dao::UserEntity {
                    name: "testuser".into(),
                }),
                eq("permission-service"),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        permission_service
            .create_user("testuser")
            .await
            .expect("Extected successful user creation");
    }

    #[tokio::test]
    async fn test_create_user_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(&permission_service.create_user("testuser").await);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao
            .expect_delete_user()
            .with(eq("testuser"))
            .times(1)
            .returning(|_| Ok(()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

        permission_service
            .delete_user("testuser")
            .await
            .expect("Expected successful delete");
    }
    #[tokio::test]
    async fn test_delete_user_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(&permission_service.delete_user("testuser").await);
    }

    #[tokio::test]
    async fn test_create_role() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao
            .expect_create_role()
            .with(
                eq(dao::RoleEntity {
                    name: "testrole".into(),
                }),
                eq("permission-service"),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        permission_service
            .create_role("testrole")
            .await
            .expect("Extected successful role creation");
    }

    #[tokio::test]
    async fn test_create_role_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(&permission_service.create_role("testrole").await);
    }

    #[tokio::test]
    async fn test_delete_role() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao
            .expect_delete_role()
            .with(eq("testrole"))
            .times(1)
            .returning(|_| Ok(()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

        permission_service
            .delete_role("testrole")
            .await
            .expect("Expected successful delete");
    }

    #[tokio::test]
    async fn test_delete_role_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(&permission_service.delete_role("testrole").await);
    }

    #[tokio::test]
    async fn test_create_privilege() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao
            .expect_create_privilege()
            .with(
                eq(dao::PrivilegeEntity {
                    name: "testprivilege".into(),
                }),
                eq("permission-service"),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

        permission_service
            .create_privilege("testprivilege")
            .await
            .expect("Extected successful privilege creation");
    }
    #[tokio::test]
    async fn test_create_privilege_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(&permission_service.create_privilege("testprivilege").await);
    }

    #[tokio::test]
    async fn test_delete_privilege() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao
            .expect_delete_privilege()
            .with(eq("testprivilege"))
            .times(1)
            .returning(|_| Ok(()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

        permission_service
            .delete_privilege("testprivilege")
            .await
            .expect("Expected successful delete");
    }

    #[tokio::test]
    async fn test_delete_privilege_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(&permission_service.delete_privilege("testprivilege").await);
    }

    #[tokio::test]
    async fn test_add_user_role() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao
            .expect_add_user_role()
            .with(eq("testuser"), eq("testrole"), eq("permission-service"))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

        permission_service
            .add_user_role("testuser", "testrole")
            .await
            .expect("Extected successful user role creation");
    }

    #[tokio::test]
    async fn test_add_user_role_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(
            &permission_service
                .add_user_role("testuser", "testrole")
                .await,
        );
    }

    #[tokio::test]
    async fn test_add_role_privilege() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao
            .expect_add_role_privilege()
            .with(
                eq("testrole"),
                eq("testprivilege"),
                eq("permission-service"),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

        permission_service
            .add_role_privilege("testrole", "testprivilege")
            .await
            .expect("Extected successful role privilege creation");
    }

    #[tokio::test]
    async fn test_add_role_privilege_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(
            &permission_service
                .add_role_privilege("testrole", "testprivilege")
                .await,
        );
    }

    #[tokio::test]
    async fn test_delete_role_privilege() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao
            .expect_delete_role_privilege()
            .with(eq("testrole"), eq("testprivilege"))
            .times(1)
            .returning(|_, _| Ok(()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

        permission_service
            .delete_role_privilege("testrole", "testprivilege")
            .await
            .expect("Extected successful role privilege deletion");
    }

    #[tokio::test]
    async fn test_delete_role_privilege_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(
            &permission_service
                .delete_role_privilege("testrole", "testprivilege")
                .await,
        );
    }

    #[tokio::test]
    async fn test_delete_user_role() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao
            .expect_delete_user_role()
            .with(eq("testuser"), eq("testrole"))
            .times(1)
            .returning(|_, _| Ok(()));

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

        permission_service
            .delete_user_role("testuser", "testrole")
            .await
            .expect("Extected successful user role deletion");
    }

    #[tokio::test]
    async fn test_delete_user_role_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(
            &permission_service
                .delete_user_role("testuser", "testrole")
                .await,
        );
    }

    #[tokio::test]
    async fn test_all_roles() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao.expect_all_roles().times(1).returning(|| {
            Ok(Arc::new([
                dao::RoleEntity {
                    name: "testrole".into(),
                },
                dao::RoleEntity {
                    name: "testrole2".into(),
                },
            ]))
        });

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

        let all_roles = permission_service
            .get_all_roles()
            .await
            .expect("Expected roles successfully");
        assert_eq!(all_roles.len(), 2);
        assert_eq!(all_roles[0].name.as_ref(), "testrole");
        assert_eq!(all_roles[1].name.as_ref(), "testrole2");
    }

    #[tokio::test]
    async fn test_all_roles_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(&permission_service.get_all_roles().await);
    }

    #[tokio::test]
    async fn test_all_users() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao.expect_all_users().times(1).returning(|| {
            Ok(Arc::new([
                dao::UserEntity {
                    name: "testuser".into(),
                },
                dao::UserEntity {
                    name: "testuser2".into(),
                },
            ]))
        });

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

        let all_users = permission_service
            .get_all_users()
            .await
            .expect("Expected users successfully");

        assert_eq!(all_users.len(), 2);
        assert_eq!(all_users[0].name.as_ref(), "testuser");
        assert_eq!(all_users[1].name.as_ref(), "testuser2");
    }

    #[tokio::test]
    async fn test_all_users_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(&permission_service.get_all_users().await);
    }

    #[tokio::test]
    async fn test_all_privileges() {
        let (mut permission_dao, user_service) =
            generate_dependencies_mocks_permission(true, "admin");
        permission_dao
            .expect_all_privileges()
            .times(1)
            .returning(|| {
                Ok(Arc::new([
                    dao::PrivilegeEntity {
                        name: "testprivilege".into(),
                    },
                    dao::PrivilegeEntity {
                        name: "testprivilege2".into(),
                    },
                ]))
            });

        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

        let all_privileges = permission_service
            .get_all_privileges()
            .await
            .expect("Expected privileges successfully");

        assert_eq!(all_privileges.len(), 2);
        assert_eq!(all_privileges[0].name.as_ref(), "testprivilege");
        assert_eq!(all_privileges[1].name.as_ref(), "testprivilege2");
    }

    #[tokio::test]
    async fn test_all_privileges_without_permission() {
        let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
        let permission_service =
            PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
        test_forbidden(&permission_service.get_all_privileges().await);
    }
}
