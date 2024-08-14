use std::sync::Arc;

use async_trait::async_trait;
use service::permission::Authentication;
use service::{Privilege, ServiceError};

pub struct PermissionServiceImpl<PermissionDao, UserService>
where
    PermissionDao: dao::PermissionDao + Send + Sync,
    UserService: service::user_service::UserService + Send + Sync,
{
    permission_dao: Arc<PermissionDao>,
    user_service: Arc<UserService>,
}
impl<PermissionDao, UserService> PermissionServiceImpl<PermissionDao, UserService>
where
    PermissionDao: dao::PermissionDao + Send + Sync,
    UserService: service::user_service::UserService + Send + Sync,
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
    UserService: service::user_service::UserService + Send + Sync,
{
    type Context = UserService::Context;

    async fn current_user_id(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Option<Arc<str>>, ServiceError> {
        match context {
            Authentication::Full => Ok(None),
            Authentication::Context(context) => {
                let current_user = self.user_service.current_user(context).await?;
                Ok(Some(current_user))
            }
        }
    }
    async fn check_permission(
        &self,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        match context {
            Authentication::Full => Ok(()),
            Authentication::Context(context) => {
                let current_user = self.user_service.current_user(context).await?;
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
    }

    async fn check_user(
        &self,
        user: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        match context {
            Authentication::Full => Ok(()),
            Authentication::Context(context) => {
                let current_user = self.user_service.current_user(context).await?;
                if current_user.as_ref() == user {
                    Ok(())
                } else {
                    Err(service::ServiceError::Forbidden)
                }
            }
        }
    }

    async fn check_only_full_authentication(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        match context {
            Authentication::Full => Ok(()),
            Authentication::Context(_) => Err(service::ServiceError::Forbidden),
        }
    }

    async fn get_privileges_for_current_user(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[Privilege]>, ServiceError> {
        match context {
            Authentication::Full => Ok(Arc::new([Privilege {
                name: "god-mode".into(),
            }])),
            Authentication::Context(context) => {
                let current_user = self.user_service.current_user(context).await?;
                Ok(self
                    .permission_dao
                    .privileges_for_user(current_user.as_ref())
                    .await?
                    .iter()
                    .map(service::Privilege::from)
                    .collect())
            }
        }
    }

    async fn create_user(
        &self,
        user: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao
            .create_user(
                &dao::UserEntity { name: user.into() },
                PERMISSION_SERVICE_PROCESS,
            )
            .await?;
        Ok(())
    }
    async fn delete_user(
        &self,
        user: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao.delete_user(user).await?;
        Ok(())
    }

    async fn user_exists(
        &self,
        user: &str,
        context: Authentication<Self::Context>,
    ) -> Result<bool, ServiceError> {
        self.check_permission("hr", context).await?;
        Ok(self
            .permission_dao
            .find_user(user)
            .await
            .map(|x| x.is_some())?)
    }

    async fn get_all_users(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[service::User]>, service::ServiceError> {
        self.check_permission("admin", context).await?;
        Ok(self
            .permission_dao
            .all_users()
            .await?
            .iter()
            .map(service::User::from)
            .collect())
    }

    async fn create_role(
        &self,
        role: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao
            .create_role(
                &dao::RoleEntity { name: role.into() },
                PERMISSION_SERVICE_PROCESS,
            )
            .await?;
        Ok(())
    }
    async fn delete_role(
        &self,
        role: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao.delete_role(role).await?;
        Ok(())
    }
    async fn get_all_roles(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[service::Role]>, service::ServiceError> {
        self.check_permission("admin", context).await?;
        Ok(self
            .permission_dao
            .all_roles()
            .await?
            .iter()
            .map(service::Role::from)
            .collect())
    }

    async fn create_privilege(
        &self,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
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

    async fn delete_privilege(
        &self,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao.delete_privilege(privilege).await?;
        Ok(())
    }
    async fn get_all_privileges(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[service::Privilege]>, service::ServiceError> {
        self.check_permission("admin", context).await?;
        Ok(self
            .permission_dao
            .all_privileges()
            .await?
            .iter()
            .map(service::Privilege::from)
            .collect())
    }

    async fn add_user_role(
        &self,
        user: &str,
        role: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao
            .add_user_role(user, role, PERMISSION_SERVICE_PROCESS)
            .await?;
        Ok(())
    }
    async fn add_role_privilege(
        &self,
        role: &str,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao
            .add_role_privilege(role, privilege, PERMISSION_SERVICE_PROCESS)
            .await?;
        Ok(())
    }
    async fn delete_role_privilege(
        &self,
        role: &str,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao
            .delete_role_privilege(role, privilege)
            .await?;
        Ok(())
    }
    async fn delete_user_role(
        &self,
        user: &str,
        role: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao.delete_user_role(user, role).await?;
        Ok(())
    }
}
