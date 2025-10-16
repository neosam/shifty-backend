use crate::gen_service_impl;
use std::sync::Arc;

use async_trait::async_trait;
use dao::user_invitation::{UserInvitationDao, UserInvitationEntity};
use dao::{PermissionDao, TransactionDao, UserEntity};
use service::clock::ClockService;
use service::permission::Authentication;
use service::user_invitation::{UserInvitation, UserInvitationService};
use service::uuid_service::UuidService;
use service::{PermissionService, ServiceError};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

gen_service_impl! {
    struct UserInvitationServiceImpl: service::user_invitation::UserInvitationService = UserInvitationServiceDeps {
        UserInvitationDao: dao::user_invitation::UserInvitationDao = user_invitation_dao,
        PermissionDao: dao::PermissionDao = permission_dao,
        PermissionService: service::PermissionService<Context = Self::Context> = permission_service,
        UuidService: service::uuid_service::UuidService = uuid_service,
        ClockService: service::clock::ClockService = clock_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

const USER_INVITATION_SERVICE_PROCESS: &str = "user-invitation-service";

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
        };

        self.user_invitation_dao.create_invitation(&entity).await?;

        self.transaction_dao.commit(tx).await?;

        Ok(UserInvitation {
            id: entity.id,
            username: entity.username.to_string(),
            token: entity.token,
            expiration_date: entity.expiration_date,
            created_date: entity.created_date,
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

        // Delete the token after successful validation
        self.user_invitation_dao.delete_by_token(token).await?;

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
}
