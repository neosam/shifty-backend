use std::sync::Arc;
use std::fmt::Debug;

use async_trait::async_trait;
use mockall::automock;

use crate::ServiceError;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Authentication<Context: Clone + PartialEq + Eq + Send + Sync + Debug + 'static>{
    Full,
    Context(Context),
}
impl<Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static> From<Context> for Authentication<Context> {
    fn from(context: Context) -> Self {
        Self::Context(context)
    }
}

#[automock(type Context=();)]
#[async_trait]
pub trait PermissionService {
    type Context: Clone + PartialEq + Eq + Debug + Send + Sync + 'static;

    async fn check_permission(
        &self,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;

    async fn create_user(&self, user: &str, context: Authentication<Self::Context>) -> Result<(), ServiceError>;
    async fn user_exists(&self, user: &str, context: Authentication<Self::Context>) -> Result<bool, ServiceError>;
    async fn delete_user(&self, user: &str, context: Authentication<Self::Context>) -> Result<(), ServiceError>;
    async fn get_all_users(&self, context: Authentication<Self::Context>) -> Result<Arc<[User]>, ServiceError>;

    async fn create_role(&self, role: &str, context: Authentication<Self::Context>) -> Result<(), ServiceError>;
    async fn delete_role(&self, role: &str, context: Authentication<Self::Context>) -> Result<(), ServiceError>;
    async fn get_all_roles(&self, context: Authentication<Self::Context>) -> Result<Arc<[Role]>, ServiceError>;

    async fn create_privilege(
        &self,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
    async fn delete_privilege(
        &self,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
    async fn get_all_privileges(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[Privilege]>, ServiceError>;

    async fn add_user_role(
        &self,
        user: &str,
        role: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
    async fn add_role_privilege(
        &self,
        role: &str,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
    async fn delete_role_privilege(
        &self,
        role: &str,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
    async fn delete_user_role(
        &self,
        user: &str,
        role: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
}
