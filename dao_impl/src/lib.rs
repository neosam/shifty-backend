use std::sync::Arc;

use dao::DaoError;
use sqlx::{query, SqlitePool};

pub struct HelloDaoImpl {
    pool: Arc<SqlitePool>,
}

impl HelloDaoImpl {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

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
        .map_err(|err| DaoError::DatabaseQueryError(Box::new(err)))?;
        Ok(result[0].results > 0)
    }
}
