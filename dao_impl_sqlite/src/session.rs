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
        let session = SessionDb {
            id: entity.id.to_string(),
            user_id: entity.user_id.to_string(),
            expires: entity.expires,
            created: entity.created,
        };
        query!(
            r"INSERT INTO session (id, user_id, expires, created) VALUES (?, ?, ?, ?)",
            session.id,
            session.user_id,
            session.expires,
            session.created,
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
            r"SELECT id, user_id, expires, created FROM session WHERE id = ?",
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
        }))
    }

    async fn delete(&self, id: &str) -> Result<(), DaoError> {
        query!(r"DELETE FROM session WHERE id = ?", id,)
            .execute(self.pool.as_ref())
            .await
            .map_db_error()?;
        Ok(())
    }
}
