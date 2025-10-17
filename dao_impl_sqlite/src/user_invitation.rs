use std::sync::Arc;

use async_trait::async_trait;
use dao::user_invitation::{UserInvitationDao, UserInvitationEntity};
use dao::DaoError;
use sqlx::{query_as, SqlitePool};
use time::format_description::well_known::Iso8601;
use time::{OffsetDateTime, PrimitiveDateTime};
use uuid::Uuid;
use crate::ResultDbErrorExt;

pub struct UserInvitationDaoImpl {
    connection_pool: Arc<SqlitePool>,
}

impl UserInvitationDaoImpl {
    pub fn new(connection_pool: Arc<SqlitePool>) -> Self {
        Self { connection_pool }
    }
}

struct UserInvitationDb {
    id: String,
    username: String,
    token: String,
    expiration_date: String,
    created_date: String,
    update_process: String,
    redeemed_at: Option<String>,
    session_id: Option<String>,
}

impl TryFrom<&UserInvitationDb> for UserInvitationEntity {
    type Error = DaoError;

    fn try_from(db: &UserInvitationDb) -> Result<Self, Self::Error> {
        // Parse dates with fallback pattern like other DAOs
        let expiration_date = OffsetDateTime::parse(&db.expiration_date, &Iso8601::DATE_TIME)
            .or_else(|_| {
                PrimitiveDateTime::parse(&db.expiration_date, &Iso8601::DATE_TIME)
                    .map(|pdt| pdt.assume_utc())
            })?;
            
        let created_date = OffsetDateTime::parse(&db.created_date, &Iso8601::DATE_TIME)
            .or_else(|_| {
                PrimitiveDateTime::parse(&db.created_date, &Iso8601::DATE_TIME)
                    .map(|pdt| pdt.assume_utc())
            })?;
            
        let redeemed_at = db.redeemed_at
            .as_ref()
            .map(|date_str| {
                OffsetDateTime::parse(date_str, &Iso8601::DATE_TIME)
                    .or_else(|_| {
                        PrimitiveDateTime::parse(date_str, &Iso8601::DATE_TIME)
                            .map(|pdt| pdt.assume_utc())
                    })
            })
            .transpose()?;

        Ok(Self {
            id: db.id.parse()?,
            username: Arc::from(db.username.as_str()),
            token: db.token.parse()?,
            expiration_date,
            created_date,
            update_process: Arc::from(db.update_process.as_str()),
            redeemed_at,
            session_id: db.session_id.as_ref().map(|s| Arc::from(s.as_str())),
        })
    }
}

#[async_trait]
impl UserInvitationDao for UserInvitationDaoImpl {
    async fn create_invitation(&self, invitation: &UserInvitationEntity) -> Result<(), DaoError> {
        let id_str = invitation.id.to_string();
        let username_str = invitation.username.to_string();
        let token_str = invitation.token.to_string();
        let update_process_str = invitation.update_process.to_string();
        let expiration_date_str = invitation.expiration_date.format(&Iso8601::DATE_TIME).map_db_error()?;
        let created_date_str = invitation.created_date.format(&Iso8601::DATE_TIME).map_db_error()?;
        let redeemed_at_str = invitation.redeemed_at
            .map(|dt| dt.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let session_id_str = invitation.session_id.as_ref().map(|s| s.to_string());

        sqlx::query!(
            r#"
            INSERT INTO user_invitation (
                id, username, token, expiration_date, created_date, update_process, redeemed_at, session_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id_str,
            username_str,
            token_str,
            expiration_date_str,
            created_date_str,
            update_process_str,
            redeemed_at_str,
            session_id_str
        )
        .execute(&*self.connection_pool)
        .await
        .map_db_error()?;

        Ok(())
    }

    async fn find_by_token(&self, token: &Uuid) -> Result<Option<UserInvitationEntity>, DaoError> {
        let token_str = token.to_string();

        Ok(query_as!(
            UserInvitationDb,
            r#"
            SELECT 
                id, username, token, expiration_date, 
                created_date, update_process, redeemed_at, session_id
            FROM user_invitation
            WHERE token = ?
            "#,
            token_str
        )
        .fetch_optional(&*self.connection_pool)
        .await
        .map_db_error()?
        .as_ref()
        .map(UserInvitationEntity::try_from)
        .transpose()?)
    }

    async fn mark_as_redeemed(&self, token: &Uuid, session_id: &str) -> Result<(), DaoError> {
        let token_str = token.to_string();
        let redeemed_at = OffsetDateTime::now_utc().format(&Iso8601::DATE_TIME).map_db_error()?;

        sqlx::query!(
            r#"
            UPDATE user_invitation
            SET redeemed_at = ?, session_id = ?
            WHERE token = ?
            "#,
            redeemed_at,
            session_id,
            token_str
        )
        .execute(&*self.connection_pool)
        .await
        .map_db_error()?;

        Ok(())
    }

    async fn delete_by_token(&self, token: &Uuid) -> Result<(), DaoError> {
        let token_str = token.to_string();

        sqlx::query!(
            r#"
            DELETE FROM user_invitation
            WHERE token = ?
            "#,
            token_str
        )
        .execute(&*self.connection_pool)
        .await
        .map_db_error()?;

        Ok(())
    }

    async fn delete_expired(&self, current_time: &OffsetDateTime) -> Result<u64, DaoError> {
        let current_time_str = current_time.format(&Iso8601::DATE_TIME).map_db_error()?;

        let result = sqlx::query!(
            r#"
            DELETE FROM user_invitation
            WHERE expiration_date < ?
            "#,
            current_time_str
        )
        .execute(&*self.connection_pool)
        .await
        .map_db_error()?;

        Ok(result.rows_affected())
    }

    async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Vec<UserInvitationEntity>, DaoError> {
        let rows = query_as!(
            UserInvitationDb,
            r#"
            SELECT 
                id, username, token, expiration_date, 
                created_date, update_process, redeemed_at, session_id
            FROM user_invitation
            WHERE username = ?
            ORDER BY created_date DESC
            "#,
            username
        )
        .fetch_all(&*self.connection_pool)
        .await
        .map_db_error()?;

        rows.iter()
            .map(UserInvitationEntity::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn find_by_session_id(&self, session_id: &str) -> Result<Option<UserInvitationEntity>, DaoError> {
        Ok(query_as!(
            UserInvitationDb,
            r#"
            SELECT 
                id, username, token, expiration_date, 
                created_date, update_process, redeemed_at, session_id
            FROM user_invitation
            WHERE session_id = ?
            "#,
            session_id
        )
        .fetch_optional(&*self.connection_pool)
        .await
        .map_db_error()?
        .as_ref()
        .map(UserInvitationEntity::try_from)
        .transpose()?)
    }

    async fn delete_by_id(&self, id: &Uuid) -> Result<(), DaoError> {
        let id_str = id.to_string();

        sqlx::query!(
            r#"
            DELETE FROM user_invitation
            WHERE id = ?
            "#,
            id_str
        )
        .execute(&*self.connection_pool)
        .await
        .map_db_error()?;

        Ok(())
    }
}
