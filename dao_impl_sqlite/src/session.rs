use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    session::{SessionDao, SessionEntity},
    DaoError,
};
use sqlx::{query, query_as, SqlitePool};

use crate::ResultDbErrorExt;

struct SessionDb {
    id: String,
    user_id: String,
    expires: i64,
    created: i64,
    impersonate_user_id: Option<String>,
}

pub struct SessionDaoImpl {
    pool: Arc<SqlitePool>,
}

impl SessionDaoImpl {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionDao for SessionDaoImpl {
    async fn create(&self, entity: &SessionEntity) -> Result<(), DaoError> {
        let id = entity.id.to_string();
        let user_id = entity.user_id.to_string();
        let expires = entity.expires;
        let created = entity.created;
        query!(
            r"INSERT INTO session (id, user_id, expires, created) VALUES (?, ?, ?, ?)",
            id,
            user_id,
            expires,
            created,
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<SessionEntity>, DaoError> {
        let id = id.to_string();
        let session = query_as!(
            SessionDb,
            r"SELECT id, user_id, expires, created, impersonate_user_id FROM session WHERE id = ?",
            id
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(session.map(|session| SessionEntity {
            id: Arc::from(session.id),
            user_id: Arc::from(session.user_id),
            expires: session.expires,
            created: session.created,
            impersonate_user_id: session.impersonate_user_id.map(Arc::from),
        }))
    }

    async fn delete(&self, id: &str) -> Result<(), DaoError> {
        query!(r"DELETE FROM session WHERE id = ?", id,)
            .execute(self.pool.as_ref())
            .await
            .map_db_error()?;
        Ok(())
    }

    async fn update_impersonate(
        &self,
        session_id: &str,
        impersonate_user_id: Option<Arc<str>>,
    ) -> Result<(), DaoError> {
        let session_id = session_id.to_string();
        let impersonate = impersonate_user_id.map(|s| s.to_string());
        query!(
            r"UPDATE session SET impersonate_user_id = ? WHERE id = ?",
            impersonate,
            session_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }
}
