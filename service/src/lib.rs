use async_trait::async_trait;
use mockall::automock;
use std::{future::Future, sync::Arc};
use thiserror::Error;

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

#[derive(Debug, PartialEq, Eq)]
pub struct User {
    pub name: Arc<str>,
}
impl From<&dao::UserEntity> for User {
    fn from(user: &dao::UserEntity) -> Self {
        Self {
            name: user.name.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Role {
    pub name: Arc<str>,
}
impl From<&dao::RoleEntity> for Role {
    fn from(role: &dao::RoleEntity) -> Self {
        Self {
            name: role.name.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Privilege {
    pub name: Arc<str>,
}
impl From<&dao::PrivilegeEntity> for Privilege {
    fn from(privilege: &dao::PrivilegeEntity) -> Self {
        Self {
            name: privilege.name.clone(),
        }
    }
}

#[automock]
#[async_trait]
pub trait PermissionService {
    async fn check_permission(&self, privilege: &str) -> Result<(), ServiceError>;

    async fn create_user(&self, user: &str) -> Result<(), ServiceError>;
    async fn delete_user(&self, user: &str) -> Result<(), ServiceError>;
    async fn get_all_users(&self) -> Result<Arc<[User]>, ServiceError>;

    async fn create_role(&self, role: &str) -> Result<(), ServiceError>;
    async fn delete_role(&self, role: &str) -> Result<(), ServiceError>;
    async fn get_all_roles(&self) -> Result<Arc<[Role]>, ServiceError>;

    async fn create_privilege(&self, privilege: &str) -> Result<(), ServiceError>;
    async fn delete_privilege(&self, privilege: &str) -> Result<(), ServiceError>;
    async fn get_all_privileges(&self) -> Result<Arc<[Privilege]>, ServiceError>;

    async fn add_user_role(&self, user: &str, role: &str) -> Result<(), ServiceError>;
    async fn add_role_privilege(&self, role: &str, privilege: &str) -> Result<(), ServiceError>;
    async fn delete_role_privilege(&self, role: &str, privilege: &str) -> Result<(), ServiceError>;
    async fn delete_user_role(&self, user: &str, role: &str) -> Result<(), ServiceError>;
}

#[automock]
#[async_trait]
pub trait UserService {
    async fn current_user(&self) -> Result<Arc<str>, ServiceError>;
}
