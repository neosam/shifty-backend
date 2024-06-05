use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

use crate::DaoError;

#[derive(Debug, PartialEq, Eq)]
pub struct UserEntity {
    pub name: Arc<str>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct RoleEntity {
    pub name: Arc<str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PrivilegeEntity {
    pub name: Arc<str>,
}

#[automock]
#[async_trait]
pub trait PermissionDao {
    async fn has_privilege(&self, user: &str, privilege: &str) -> Result<bool, DaoError>;

    async fn create_user(&self, user: &UserEntity, process: &str) -> Result<(), DaoError>;
    async fn all_users(&self) -> Result<Arc<[UserEntity]>, DaoError>;
    async fn find_user(&self, username: &str) -> Result<Option<UserEntity>, DaoError>;
    async fn delete_user(&self, username: &str) -> Result<(), DaoError>;

    async fn create_role(&self, role: &RoleEntity, process: &str) -> Result<(), DaoError>;
    async fn all_roles(&self) -> Result<Arc<[RoleEntity]>, DaoError>;
    async fn delete_role(&self, rolename: &str) -> Result<(), DaoError>;

    async fn create_privilege(
        &self,
        privilege: &PrivilegeEntity,
        process: &str,
    ) -> Result<(), DaoError>;
    async fn all_privileges(&self) -> Result<Arc<[PrivilegeEntity]>, DaoError>;
    async fn delete_privilege(&self, privilege: &str) -> Result<(), DaoError>;

    async fn add_user_role(&self, user: &str, role: &str, process: &str) -> Result<(), DaoError>;
    async fn add_role_privilege(
        &self,
        role: &str,
        privilege: &str,
        process: &str,
    ) -> Result<(), DaoError>;
    async fn delete_role_privilege(&self, role: &str, privilege: &str) -> Result<(), DaoError>;
    async fn delete_user_role(&self, user: &str, role: &str) -> Result<(), DaoError>;

    async fn privileges_for_user(&self, user: &str) -> Result<Arc<[PrivilegeEntity]>, DaoError>;
}
