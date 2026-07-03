use std::sync::Arc;

use crate::DaoError;
use mockall::automock;
use uuid::Uuid;

/// Persistenz-Modell für die Nextcloud-PDF-Export-Konfiguration
/// (Phase 48, EXP-02/EXP-03, D-48-CONFIG). Single-Row-Pattern: exakt eine
/// aktive Zeile mit fixer UUID als PK — analog `paid_limit_config` /
/// `holiday_stichtag_config`.
///
/// `webdav_app_token` liegt bewusst KLARTEXT in der DB (D-48-01, Ops-
/// Entscheidung); der Service maskiert ihn in der HTTP-Response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PdfExportConfigEntity {
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

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait PdfExportConfigDao {
    type Transaction: crate::Transaction;

    /// Liest die einzige seed-persistierte Konfig-Zeile. Bricht ab, wenn die
    /// Seed-Row fehlt (defensive; die Migration seedet immer).
    async fn get(&self, tx: Self::Transaction) -> Result<PdfExportConfigEntity, DaoError>;

    /// Überschreibt alle admin-editierbaren Felder (enabled, URL, User, Token,
    /// Zielordner, Wochen-Horizont, Cron). Status-Felder (`last_success_at`,
    /// `last_error_at`, `last_error_message`) bleiben unangetastet — die
    /// setzt der Scheduler über [`record_success`] / [`record_error`].
    async fn update(
        &self,
        entity: &PdfExportConfigEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    /// Vom Scheduler nach einem erfolgreichen Lauf aufgerufen: setzt
    /// `last_success_at`, clearet `last_error_at` und `last_error_message`.
    async fn record_success(
        &self,
        at: time::PrimitiveDateTime,
        process: &str,
        version: Uuid,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    /// Vom Scheduler nach 3× fehlgeschlagenem Retry aufgerufen: setzt
    /// `last_error_at` und `last_error_message`; `last_success_at` bleibt
    /// unverändert.
    async fn record_error(
        &self,
        at: time::PrimitiveDateTime,
        message: &str,
        process: &str,
        version: Uuid,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
