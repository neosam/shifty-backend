use std::sync::Arc;

use async_trait::async_trait;
use dao::session::SessionEntity;
use mockall::automock;

use crate::ServiceError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub id: Arc<str>,
    pub user_id: Arc<str>,
    pub expires: i64,
    pub created: i64,
}

impl From<&SessionEntity> for Session {
    fn from(session: &SessionEntity) -> Self {
        Self {
            id: session.id.clone(),
            user_id: session.user_id.clone(),
            expires: session.expires,
            created: session.created,
        }
    }
}

impl From<&Session> for SessionEntity {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id.clone(),
            user_id: session.user_id.clone(),
            expires: session.expires,
            created: session.created,
        }
    }
}

#[automock(type Context=();)]
#[async_trait]
pub trait SessionService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;

    async fn new_session_for_user(&self, user_id: &str) -> Result<Session, ServiceError>;
    async fn invalidate_user_session(&self, id: &str) -> Result<(), ServiceError>;
    async fn verify_user_session(&self, id: &str) -> Result<Option<Session>, ServiceError>;
}
