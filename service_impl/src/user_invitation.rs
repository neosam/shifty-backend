use crate::gen_service_impl;
use std::sync::Arc;

use async_trait::async_trait;
use dao::user_invitation::{UserInvitationDao, UserInvitationEntity};
use dao::{PermissionDao, TransactionDao, UserEntity};
use service::permission::Authentication;
use service::session::SessionService;
use service::user_invitation::{InvitationStatus, UserInvitation, UserInvitationService};
use service::uuid_service::UuidService;
use service::{PermissionService, ServiceError};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

gen_service_impl! {
    struct UserInvitationServiceImpl: service::user_invitation::UserInvitationService = UserInvitationServiceDeps {
        UserInvitationDao: dao::user_invitation::UserInvitationDao = user_invitation_dao,
        PermissionDao: dao::PermissionDao = permission_dao,
        PermissionService: service::PermissionService<Context = Self::Context> = permission_service,
        SessionService: service::session::SessionService<Context = Self::Context> = session_service,
        UuidService: service::uuid_service::UuidService = uuid_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

const USER_INVITATION_SERVICE_PROCESS: &str = "user-invitation-service";

fn compute_invitation_status(entity: &UserInvitationEntity) -> InvitationStatus {
    if entity.session_revoked_at.is_some() {
        InvitationStatus::SessionRevoked
    } else if entity.redeemed_at.is_some() {
        InvitationStatus::Redeemed
    } else if entity.expiration_date < OffsetDateTime::now_utc() {
        InvitationStatus::Expired
    } else {
        InvitationStatus::Valid
    }
}

#[async_trait]
impl<Deps: UserInvitationServiceDeps> UserInvitationService for UserInvitationServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn generate_invitation(
        &self,
        username: &str,
        expiration_hours: i64,
        tx: Option<Self::Transaction>,
        auth: Authentication<Self::Context>,
    ) -> Result<UserInvitation, ServiceError> {
        self.permission_service
            .check_permission("admin", auth)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        let id = self.uuid_service.new_uuid("user-invitation-id");
        let token = self.uuid_service.new_uuid("user-invitation-token");
        let now = OffsetDateTime::now_utc();
        let expiration_date = now + Duration::hours(expiration_hours);

        let entity = UserInvitationEntity {
            id,
            username: Arc::from(username),
            token,
            expiration_date,
            created_date: now,
            update_process: Arc::from(USER_INVITATION_SERVICE_PROCESS),
            redeemed_at: None,
            session_id: None,
            session_revoked_at: None,
        };

        self.user_invitation_dao.create_invitation(&entity).await?;

        self.transaction_dao.commit(tx).await?;

        Ok(UserInvitation {
            id: entity.id,
            username: entity.username.to_string(),
            token: entity.token,
            expiration_date: entity.expiration_date,
            created_date: entity.created_date,
            redeemed_at: entity.redeemed_at,
            status: compute_invitation_status(&entity),
        })
    }

    async fn validate_and_consume_token(
        &self,
        token: &Uuid,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<str>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let invitation = self
            .user_invitation_dao
            .find_by_token(token)
            .await?
            .ok_or_else(|| {
                ServiceError::EntityNotFoundGeneric("Invalid invitation token".into())
            })?;

        // Check if already redeemed
        if invitation.session_id.is_some() {
            return Err(ServiceError::EntityNotFoundGeneric(
                "Invitation token has already been used".into(),
            ));
        }

        let now = OffsetDateTime::now_utc();
        if invitation.expiration_date < now {
            return Err(ServiceError::EntityNotFoundGeneric(
                "Invitation token has expired".into(),
            ));
        }

        // Ensure user exists in the system
        if self
            .permission_dao
            .find_user(invitation.username.as_ref())
            .await?
            .is_none()
        {
            // Create the user if they don't exist
            let user_entity = UserEntity {
                name: invitation.username.clone(),
            };
            self.permission_dao
                .create_user(&user_entity, USER_INVITATION_SERVICE_PROCESS)
                .await?;
        }

        // Note: We no longer delete the token here - it will be marked as redeemed
        // after the session is created

        self.transaction_dao.commit(tx).await?;

        Ok(invitation.username)
    }

    async fn list_invitations_for_user(
        &self,
        username: &str,
        tx: Option<Self::Transaction>,
        auth: Authentication<Self::Context>,
    ) -> Result<Vec<UserInvitation>, ServiceError> {
        self.permission_service
            .check_permission("admin", auth)
            .await?;

        let _tx = self.transaction_dao.use_transaction(tx).await?;

        let entities = self.user_invitation_dao.find_by_username(username).await?;

        let invitations = entities
            .into_iter()
            .map(|entity| UserInvitation {
                id: entity.id,
                username: entity.username.to_string(),
                token: entity.token,
                expiration_date: entity.expiration_date,
                created_date: entity.created_date,
                redeemed_at: entity.redeemed_at,
                status: compute_invitation_status(&entity),
            })
            .collect();

        Ok(invitations)
    }

    async fn revoke_invitation(
        &self,
        id: &Uuid,
        tx: Option<Self::Transaction>,
        auth: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        self.permission_service
            .check_permission("admin", auth)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        self.user_invitation_dao.delete_by_id(id).await?;

        self.transaction_dao.commit(tx).await?;

        Ok(())
    }

    async fn mark_token_redeemed(
        &self,
        token: &Uuid,
        session_id: &str,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        self.user_invitation_dao.mark_as_redeemed(token, session_id).await?;

        self.transaction_dao.commit(tx).await?;

        Ok(())
    }

    async fn find_invitation_by_session(
        &self,
        session_id: &str,
        tx: Option<Self::Transaction>,
        auth: Authentication<Self::Context>,
    ) -> Result<Option<UserInvitation>, ServiceError> {
        self.permission_service
            .check_permission("admin", auth)
            .await?;

        let _tx = self.transaction_dao.use_transaction(tx).await?;

        let entity = self.user_invitation_dao.find_by_session_id(session_id).await?;

        Ok(entity.map(|e| UserInvitation {
            id: e.id,
            username: e.username.to_string(),
            token: e.token,
            expiration_date: e.expiration_date,
            created_date: e.created_date,
            redeemed_at: e.redeemed_at,
            status: compute_invitation_status(&e),
        }))
    }

    async fn cleanup_expired_invitations(
        &self,
        tx: Option<Self::Transaction>,
    ) -> Result<u64, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let now = OffsetDateTime::now_utc();
        let deleted_count = self.user_invitation_dao.delete_expired(&now).await?;

        self.transaction_dao.commit(tx).await?;

        Ok(deleted_count)
    }

    async fn revoke_session_for_invitation(
        &self,
        invitation_id: &Uuid,
        tx: Option<Self::Transaction>,
        auth: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        // Check admin permission
        self.permission_service
            .check_permission("admin", auth)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Find the invitation by ID
        let invitation = self.user_invitation_dao
            .find_by_id(invitation_id)
            .await?
            .ok_or_else(|| ServiceError::EntityNotFoundGeneric("Invitation not found".into()))?;

        // Check if the invitation has an associated session
        if let Some(session_id) = invitation.session_id.as_ref() {
            // Invalidate the session
            self.session_service.invalidate_user_session(session_id).await?;
            
            // Mark the invitation as session revoked
            self.user_invitation_dao.mark_session_revoked(invitation_id).await?;
            
            self.transaction_dao.commit(tx).await?;
            Ok(())
        } else {
            Err(ServiceError::EntityNotFoundGeneric(
                "No session associated with this invitation".into(),
            ))
        }
    }
}
