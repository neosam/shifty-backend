use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::DaoError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInvitationEntity {
    pub id: Uuid,
    pub username: Arc<str>,
    pub token: Uuid,
    pub expiration_date: OffsetDateTime,
    pub created_date: OffsetDateTime,
    pub update_process: Arc<str>,
}

#[automock]
#[async_trait]
pub trait UserInvitationDao {
    async fn create_invitation(&self, invitation: &UserInvitationEntity) -> Result<(), DaoError>;

    async fn find_by_token(&self, token: &Uuid) -> Result<Option<UserInvitationEntity>, DaoError>;

    async fn delete_by_token(&self, token: &Uuid) -> Result<(), DaoError>;

    async fn delete_expired(&self, current_time: &OffsetDateTime) -> Result<u64, DaoError>;

    async fn find_by_username(&self, username: &str)
        -> Result<Vec<UserInvitationEntity>, DaoError>;

    async fn delete_by_id(&self, id: &Uuid) -> Result<(), DaoError>;
}
