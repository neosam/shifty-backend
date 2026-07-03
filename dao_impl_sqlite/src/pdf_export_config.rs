use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    pdf_export_config::{PdfExportConfigDao, PdfExportConfigEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

/// Raw row shape aus SQLite: BLOB→Vec<u8> für UUIDs, TEXT-Timestamps als
/// String (ISO-8601). Nullable-Felder als `Option`.
#[derive(Debug)]
struct PdfExportConfigDb {
    id: Vec<u8>,
    enabled: bool,
    nextcloud_url: Option<String>,
    webdav_user: Option<String>,
    webdav_app_token: Option<String>,
    target_folder: Option<String>,
    weeks_horizon: i64,
    cron_schedule: String,
    last_success_at: Option<String>,
    last_error_at: Option<String>,
    last_error_message: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&PdfExportConfigDb> for PdfExportConfigEntity {
    type Error = DaoError;

    fn try_from(db: &PdfExportConfigDb) -> Result<Self, Self::Error> {
        Ok(PdfExportConfigEntity {
            id: Uuid::from_slice(&db.id)?,
            enabled: db.enabled,
            nextcloud_url: db.nextcloud_url.as_deref().map(Arc::from),
            webdav_user: db.webdav_user.as_deref().map(Arc::from),
            webdav_app_token: db.webdav_app_token.as_deref().map(Arc::from),
            target_folder: db.target_folder.as_deref().map(Arc::from),
            weeks_horizon: db.weeks_horizon as u32,
            cron_schedule: Arc::from(db.cron_schedule.as_str()),
            last_success_at: db
                .last_success_at
                .as_ref()
                .map(|s| PrimitiveDateTime::parse(s, &Iso8601::DATE_TIME))
                .transpose()?,
            last_error_at: db
                .last_error_at
                .as_ref()
                .map(|s| PrimitiveDateTime::parse(s, &Iso8601::DATE_TIME))
                .transpose()?,
            last_error_message: db.last_error_message.as_deref().map(Arc::from),
            version: Uuid::from_slice(&db.update_version)?,
        })
    }
}

pub struct PdfExportConfigDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl PdfExportConfigDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl PdfExportConfigDao for PdfExportConfigDaoImpl {
    type Transaction = crate::TransactionImpl;

    async fn get(&self, tx: Self::Transaction) -> Result<PdfExportConfigEntity, DaoError> {
        let row = query_as!(
            PdfExportConfigDb,
            r#"SELECT
                id,
                enabled AS "enabled!: bool",
                nextcloud_url,
                webdav_user,
                webdav_app_token,
                target_folder,
                weeks_horizon,
                cron_schedule,
                last_success_at,
                last_error_at,
                last_error_message,
                update_version
              FROM pdf_export_config
              LIMIT 1"#,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .ok_or_else(|| {
            DaoError::DatabaseQueryError(Box::new(std::io::Error::other(
                "pdf_export_config seed row missing",
            )))
        })?;

        PdfExportConfigEntity::try_from(&row)
    }

    async fn update(
        &self,
        entity: &PdfExportConfigEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let version_vec = entity.version.as_bytes().to_vec();
        let nextcloud_url = entity.nextcloud_url.as_deref();
        let webdav_user = entity.webdav_user.as_deref();
        let webdav_app_token = entity.webdav_app_token.as_deref();
        let target_folder = entity.target_folder.as_deref();
        let cron_schedule = entity.cron_schedule.as_ref();
        let weeks_horizon = entity.weeks_horizon as i64;

        query!(
            r#"UPDATE pdf_export_config
               SET enabled = ?,
                   nextcloud_url = ?,
                   webdav_user = ?,
                   webdav_app_token = ?,
                   target_folder = ?,
                   weeks_horizon = ?,
                   cron_schedule = ?,
                   update_process = ?,
                   update_version = ?
               WHERE id = ?"#,
            entity.enabled,
            nextcloud_url,
            webdav_user,
            webdav_app_token,
            target_folder,
            weeks_horizon,
            cron_schedule,
            process,
            version_vec,
            id_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }

    async fn record_success(
        &self,
        at: time::PrimitiveDateTime,
        process: &str,
        version: Uuid,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let at_str = at.format(&Iso8601::DATE_TIME).map_db_error()?;
        let version_vec = version.as_bytes().to_vec();

        query!(
            r#"UPDATE pdf_export_config
               SET last_success_at = ?,
                   last_error_at = NULL,
                   last_error_message = NULL,
                   update_process = ?,
                   update_version = ?"#,
            at_str,
            process,
            version_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }

    async fn record_error(
        &self,
        at: time::PrimitiveDateTime,
        message: &str,
        process: &str,
        version: Uuid,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let at_str = at.format(&Iso8601::DATE_TIME).map_db_error()?;
        let version_vec = version.as_bytes().to_vec();

        query!(
            r#"UPDATE pdf_export_config
               SET last_error_at = ?,
                   last_error_message = ?,
                   update_process = ?,
                   update_version = ?"#,
            at_str,
            message,
            process,
            version_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }
}
