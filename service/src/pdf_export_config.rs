//! PDF-Export-Config-Domain für Phase 48 (EXP-02/EXP-03).
//!
//! Stellt das Service-Trait [`PdfExportConfigService`] sowie die Domain-
//! Struktur [`PdfExportConfig`] und das Update-DTO [`PdfExportConfigUpdate`]
//! bereit.
//!
//! Tier-Klassifizierung: **Basic-Service (Entity-Manager)** (D-48-BASIC).
//! Der Service hängt AUSSCHLIESSLICH von DAO + Permission + Clock + Uuid +
//! Transaction ab — KEIN Domain-Service als Dependency (Service-Tier-
//! Konvention).
//!
//! Permissionsmodell (D-48-ADMIN): `get`/`update` sind admin-gated
//! (`check_permission("admin", …)`); ein nicht-admin Aufrufer erhält
//! [`ServiceError::Forbidden`]. `record_success`/`record_error` sind KEIN
//! admin-Public-API — sie werden AUSSCHLIESSLICH vom Cron-Task in Plan
//! 48-04 mit `Authentication::Full` aufgerufen.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::pdf_export_config::PdfExportConfigEntity;
use mockall::automock;
use uuid::Uuid;

use crate::{permission::Authentication, ServiceError};

/// Vollständige PDF-Export-Konfiguration (inkl. Status-Feldern). Interne
/// Domain-Struktur — beim Übergang in das REST-DTO (`PdfExportConfigTO`) wird
/// `webdav_app_token` maskiert (T-48-02).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PdfExportConfig {
    pub id: Uuid,
    pub enabled: bool,
    pub nextcloud_url: Option<Arc<str>>,
    pub webdav_user: Option<Arc<str>>,
    pub webdav_app_token: Option<Arc<str>>,
    pub target_folder: Option<Arc<str>>,
    pub weeks_horizon: u32,
    pub cron_schedule: Arc<str>,
    pub last_success_at: Option<time::PrimitiveDateTime>,
    pub last_error_at: Option<time::PrimitiveDateTime>,
    pub last_error_message: Option<Arc<str>>,
    pub version: Uuid,
}

impl From<&PdfExportConfigEntity> for PdfExportConfig {
    fn from(entity: &PdfExportConfigEntity) -> Self {
        Self {
            id: entity.id,
            enabled: entity.enabled,
            nextcloud_url: entity.nextcloud_url.clone(),
            webdav_user: entity.webdav_user.clone(),
            webdav_app_token: entity.webdav_app_token.clone(),
            target_folder: entity.target_folder.clone(),
            weeks_horizon: entity.weeks_horizon,
            cron_schedule: entity.cron_schedule.clone(),
            last_success_at: entity.last_success_at,
            last_error_at: entity.last_error_at,
            last_error_message: entity.last_error_message.clone(),
            version: entity.version,
        }
    }
}

/// Update-DTO vom Admin-REST-Handler. Enthält NUR admin-editierbare Felder
/// (KEINE Status-Felder — die persistiert der Scheduler getrennt).
///
/// `webdav_app_token`-Semantik (per D-48-REST):
/// - `None`  → bestehenden Token-Wert behalten (leer speichern in der UI)
/// - `Some(v)` → neuen Wert setzen
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PdfExportConfigUpdate {
    pub enabled: bool,
    pub nextcloud_url: Option<Arc<str>>,
    pub webdav_user: Option<Arc<str>>,
    pub webdav_app_token: Option<Arc<str>>,
    pub target_folder: Option<Arc<str>>,
    pub weeks_horizon: u32,
    pub cron_schedule: Arc<str>,
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait PdfExportConfigService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Admin-gated (D-48-ADMIN). Non-Admin → [`ServiceError::Forbidden`].
    async fn get(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<PdfExportConfig, ServiceError>;

    /// Admin-gated (D-48-ADMIN). Merged `webdav_app_token`-Semantik gemäß
    /// [`PdfExportConfigUpdate`]-Doc.
    async fn update(
        &self,
        update: PdfExportConfigUpdate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<PdfExportConfig, ServiceError>;

    /// Vom Scheduler (Plan 48-04) mit [`Authentication::Full`] aufgerufen;
    /// KEIN admin-Public-API-Pfad.
    async fn record_success(
        &self,
        at: time::PrimitiveDateTime,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    /// Analog zu [`record_success`] — nur Full-Auth (Scheduler).
    async fn record_error(
        &self,
        at: time::PrimitiveDateTime,
        message: Arc<str>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
