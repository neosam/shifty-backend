use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{permission::Authentication, ServiceError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvitationStatus {
    Valid,          // Not redeemed and not expired
    Expired,        // Not redeemed but past expiration date  
    Redeemed,       // Already used (has redeemed_at and active session)
    SessionRevoked, // Session was revoked (has session_revoked_at)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserInvitation {
    pub id: Uuid,
    pub username: String,
    pub token: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub expiration_date: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub created_date: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub redeemed_at: Option<OffsetDateTime>,
    pub status: InvitationStatus,
}

#[automock(type Transaction = (); type Context = ();)]
#[async_trait]
pub trait UserInvitationService {
    type Transaction: Clone + Send + Sync;
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

    async fn generate_invitation(
        &self,
        username: &str,
        expiration_hours: i64,
        tx: Option<Self::Transaction>,
        auth: Authentication<Self::Context>,
    ) -> Result<UserInvitation, ServiceError>;

    async fn validate_and_consume_token(
        &self,
        token: &Uuid,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<str>, ServiceError>;

    async fn mark_token_redeemed(
        &self,
        token: &Uuid,
        session_id: &str,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn find_invitation_by_session(
        &self,
        session_id: &str,
        tx: Option<Self::Transaction>,
        auth: Authentication<Self::Context>,
    ) -> Result<Option<UserInvitation>, ServiceError>;

    async fn list_invitations_for_user(
        &self,
        username: &str,
        tx: Option<Self::Transaction>,
        auth: Authentication<Self::Context>,
    ) -> Result<Vec<UserInvitation>, ServiceError>;

    async fn revoke_invitation(
        &self,
        id: &Uuid,
        tx: Option<Self::Transaction>,
        auth: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;

    async fn cleanup_expired_invitations(
        &self,
        tx: Option<Self::Transaction>,
    ) -> Result<u64, ServiceError>;

    async fn revoke_session_for_invitation(
        &self,
        invitation_id: &Uuid,
        tx: Option<Self::Transaction>,
        auth: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
}