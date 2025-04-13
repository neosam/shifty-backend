use std::sync::Arc;

use async_trait::async_trait;
use dao::{BasicDao, DaoError, PrivilegeEntity, RoleEntity, Transaction};
use sqlx::{query, query_as, SqlitePool};
use tokio::sync::Mutex;

pub mod booking;
pub mod carryover;
pub mod custom_extra_hours;
pub mod employee_work_details;
pub mod extra_hours;
pub mod sales_person;
pub mod sales_person_unavailable;
pub mod session;
pub mod shiftplan_report;
pub mod slot;
pub mod special_day;

pub trait ResultDbErrorExt<T, E> {
    fn map_db_error(self) -> Result<T, DaoError>;
}
impl<T, E: std::error::Error + Send + Sync + 'static> ResultDbErrorExt<T, E> for Result<T, E> {
    fn map_db_error(self) -> Result<T, DaoError> {
        self.map_err(|err| DaoError::DatabaseQueryError(Box::new(err)))
    }
}

pub struct HelloDaoImpl {
    pool: Arc<SqlitePool>,
}

impl HelloDaoImpl {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl dao::HelloDao for HelloDaoImpl {
    async fn get_hello(&self) -> Result<Arc<str>, dao::DaoError> {
        let result = query!(r"SELECT 'Hello, world!' as message")
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|err| DaoError::DatabaseQueryError(Box::new(err)))?;
        let message: Arc<str> = result[0].message.clone().into();
        Ok(message)
    }
}

pub struct PermissionDaoImpl {
    pool: Arc<SqlitePool>,
}
impl PermissionDaoImpl {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}
#[async_trait]
impl dao::PermissionDao for PermissionDaoImpl {
    async fn has_privilege(&self, user: &str, privilege: &str) -> Result<bool, dao::DaoError> {
        let result = query!(
            r"SELECT count(*) as results FROM user 
                                 INNER JOIN user_role ON user.name = user_role.user_name 
                                 INNER JOIN role ON user_role.role_name = role.name 
                                 INNER JOIN role_privilege ON role.name = role_privilege.role_name 
                                 WHERE role_privilege.privilege_name = ? AND user.name = ?",
            privilege,
            user,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(result[0].results > 0)
    }

    async fn create_user(&self, user: &dao::UserEntity, process: &str) -> Result<(), DaoError> {
        let name = user.name.as_ref();
        query!(
            r"INSERT INTO user (name, update_process) VALUES (?, ?)",
            name,
            process
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }
    async fn all_users(&self) -> Result<Arc<[dao::UserEntity]>, DaoError> {
        Ok(query_as!(dao::UserEntity, r"SELECT name FROM user")
            .fetch_all(self.pool.as_ref())
            .await
            .map(Arc::<[dao::UserEntity]>::from)
            .map_err(|err| DaoError::DatabaseQueryError(Box::new(err)))?)
    }
    async fn delete_user(&self, username: &str) -> Result<(), DaoError> {
        query!(r"DELETE FROM user WHERE name = ?", username)
            .execute(self.pool.as_ref())
            .await
            .map_db_error()?;
        Ok(())
    }
    async fn find_user(&self, username: &str) -> Result<Option<dao::UserEntity>, DaoError> {
        let result = query!(r"SELECT name FROM user WHERE name = ?", username)
            .fetch_optional(self.pool.as_ref())
            .await
            .map_db_error()?;
        Ok(result.map(|row| dao::UserEntity {
            name: row.name.clone().into(),
        }))
    }

    async fn create_role(&self, role: &dao::RoleEntity, process: &str) -> Result<(), DaoError> {
        let name = role.name.as_ref();
        query!(
            "INSERT INTO role (name, update_process) VALUES (?, ?)",
            name,
            process
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }
    async fn all_roles(&self) -> Result<Arc<[dao::RoleEntity]>, DaoError> {
        Ok(query_as!(dao::RoleEntity, r"SELECT name FROM role")
            .fetch_all(self.pool.as_ref())
            .await
            .map(Arc::<[dao::RoleEntity]>::from)
            .map_db_error()?)
    }
    async fn delete_role(&self, rolename: &str) -> Result<(), DaoError> {
        query!(r"DELETE FROM role WHERE name = ?", rolename)
            .execute(self.pool.as_ref())
            .await
            .map_db_error()?;
        Ok(())
    }

    async fn create_privilege(
        &self,
        privilege: &dao::PrivilegeEntity,
        process: &str,
    ) -> Result<(), DaoError> {
        let name = privilege.name.as_ref();
        query!(
            r"INSERT INTO privilege (name, update_process) VALUES (?, ?)",
            name,
            process,
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }
    async fn all_privileges(&self) -> Result<Arc<[dao::PrivilegeEntity]>, DaoError> {
        Ok(
            query_as!(dao::PrivilegeEntity, r"SELECT name FROM privilege")
                .fetch_all(self.pool.as_ref())
                .await
                .map(Arc::<[dao::PrivilegeEntity]>::from)
                .map_db_error()?,
        )
    }
    async fn delete_privilege(&self, privilege: &str) -> Result<(), DaoError> {
        query!(r"DELETE FROM privilege WHERE name = ?", privilege)
            .execute(self.pool.as_ref())
            .await
            .map_db_error()?;
        Ok(())
    }

    async fn add_user_role(&self, user: &str, role: &str, process: &str) -> Result<(), DaoError> {
        query!(
            r"INSERT INTO user_role (user_name, role_name, update_process) VALUES (?, ?, ?)",
            user,
            role,
            process,
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }
    async fn add_role_privilege(
        &self,
        role: &str,
        privilege: &str,
        process: &str,
    ) -> Result<(), DaoError> {
        query!(
            r"INSERT INTO role_privilege (role_name, privilege_name, update_process) VALUES (?, ?, ?)",
            role,
            privilege,
            process,
        ).execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }
    async fn delete_role_privilege(&self, role: &str, privilege: &str) -> Result<(), DaoError> {
        query!(
            r"DELETE FROM role_privilege WHERE role_name = ? AND privilege_name = ?",
            role,
            privilege
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }
    async fn delete_user_role(&self, user: &str, role: &str) -> Result<(), DaoError> {
        query!(
            r"DELETE FROM user_role WHERE user_name = ? AND role_name = ?",
            user,
            role,
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn privileges_for_user(&self, user: &str) -> Result<Arc<[PrivilegeEntity]>, DaoError> {
        Ok(
            query_as!(PrivilegeEntity, r"SELECT privilege.name FROM user 
                                 INNER JOIN user_role ON user.name = user_role.user_name 
                                 INNER JOIN role ON user_role.role_name = role.name 
                                 INNER JOIN role_privilege ON role.name = role_privilege.role_name 
                                 INNER JOIN privilege ON role_privilege.privilege_name = privilege.name 
                                 WHERE user.name = ?",
                                 user
            )
            .fetch_all(self.pool.as_ref())
            .await
            .map(Arc::<[PrivilegeEntity]>::from)
            .map_db_error()?,
        )
    }

    async fn roles_for_user(&self, user: &str) -> Result<Arc<[RoleEntity]>, DaoError> {
        Ok(query_as!(
            RoleEntity,
            r"SELECT user_role.role_name as name FROM user_role 
                                 WHERE user_role.user_name = ?",
            user
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map(Arc::<[RoleEntity]>::from)
        .map_db_error()?)
    }
}

pub struct BasicDaoImpl {
    pool: Arc<SqlitePool>,
}
impl BasicDaoImpl {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BasicDao for BasicDaoImpl {
    async fn clear_all(&self) -> Result<(), DaoError> {
        query!(
            r"
                DELETE FROM booking;
                DELETE FROM sales_person_user;
                DELETE FROM sales_person_unavailable;
                DELETE FROM extra_hours;
                DELETE FROM employee_work_details;
                DELETE FROM sales_person;
                "
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TransactionImpl {
    tx: Arc<Mutex<sqlx::Transaction<'static, sqlx::Sqlite>>>,
}

impl Transaction for TransactionImpl {}

pub struct TransactionDaoImpl {
    pool: Arc<SqlitePool>,
}
impl TransactionDaoImpl {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}
#[async_trait]
impl dao::TransactionDao for TransactionDaoImpl {
    type Transaction = TransactionImpl;

    async fn new_transaction(&self) -> Result<Self::Transaction, DaoError> {
        let tx = self.pool.begin().await.map_db_error()?;
        Ok(TransactionImpl {
            tx: Arc::new(tx.into()),
        })
    }

    async fn use_transaction(
        &self,
        tx: Option<Self::Transaction>,
    ) -> Result<Self::Transaction, DaoError> {
        match tx {
            Some(tx) => Ok(tx),
            None => self.new_transaction().await,
        }
    }

    async fn commit(&self, transaction: Self::Transaction) -> Result<(), DaoError> {
        if let Some(tx) = Arc::into_inner(transaction.tx) {
            tx.into_inner().commit().await.map_db_error()?;
        }
        Ok(())
    }
}
