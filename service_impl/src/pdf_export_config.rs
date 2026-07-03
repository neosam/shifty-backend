//! Basic-Tier Implementation von [`PdfExportConfigService`] (Phase 48,
//! EXP-02/EXP-03, D-48-BASIC).
//!
//! - `get` / `update` sind admin-gated (`ADMIN_PRIVILEGE`, D-48-ADMIN).
//! - `record_success` / `record_error` sind Scheduler-only
//!   (`Authentication::Full`) und laufen ohne admin-Check, weil der Scheduler
//!   sich intern authentisiert.
//! - Der `webdav_app_token`-Merge (leer = keep, gesetzt = replace) passiert
//!   HIER, damit der REST-Handler den Merge nicht kennt.

use std::sync::Arc;

use crate::gen_service_impl;

use async_trait::async_trait;
use dao::{pdf_export_config::PdfExportConfigDao, TransactionDao};
use service::{
    clock::ClockService,
    pdf_export_config::{PdfExportConfig, PdfExportConfigService, PdfExportConfigUpdate},
    permission::Authentication,
    uuid_service::UuidService,
    PermissionService, ServiceError,
};

/// Admin-Privilege für die PDF-Export-Konfiguration (D-48-ADMIN). Nutzt die
/// existierende `admin`-Rolle (kein separates Privileg) — analog anderen
/// admin-only Endpoints im System.
const ADMIN_PRIVILEGE: &str = "admin";

const PROCESS_UPDATE: &str = "pdf-export-config-service::update";
const PROCESS_RECORD_SUCCESS: &str = "pdf-export-config-service::record_success";
const PROCESS_RECORD_ERROR: &str = "pdf-export-config-service::record_error";

gen_service_impl! {
    struct PdfExportConfigServiceImpl: PdfExportConfigService = PdfExportConfigServiceDeps {
        PdfExportConfigDao: PdfExportConfigDao<Transaction = Self::Transaction> = pdf_export_config_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: PdfExportConfigServiceDeps> PdfExportConfigService for PdfExportConfigServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<PdfExportConfig, ServiceError> {
        self.permission_service
            .check_permission(ADMIN_PRIVILEGE, context)
            .await?;
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entity = self.pdf_export_config_dao.get(tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok((&entity).into())
    }

    async fn update(
        &self,
        update: PdfExportConfigUpdate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<PdfExportConfig, ServiceError> {
        self.permission_service
            .check_permission(ADMIN_PRIVILEGE, context)
            .await?;
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let current = self.pdf_export_config_dao.get(tx.clone()).await?;

        // Token-Merge-Semantik (D-48-REST): None ⇒ keep, Some(v) ⇒ set.
        let merged_token: Option<Arc<str>> = update
            .webdav_app_token
            .or_else(|| current.webdav_app_token.clone());

        let new_version = self
            .uuid_service
            .new_uuid("pdf-export-config-service::update version");

        let entity = dao::pdf_export_config::PdfExportConfigEntity {
            id: current.id,
            enabled: update.enabled,
            nextcloud_url: update.nextcloud_url,
            webdav_user: update.webdav_user,
            webdav_app_token: merged_token,
            target_folder: update.target_folder,
            weeks_horizon: update.weeks_horizon,
            cron_schedule: update.cron_schedule,
            // Status-Felder bleiben unverändert — die verwaltet der Scheduler.
            last_success_at: current.last_success_at,
            last_error_at: current.last_error_at,
            last_error_message: current.last_error_message,
            version: new_version,
        };

        self.pdf_export_config_dao
            .update(&entity, PROCESS_UPDATE, tx.clone())
            .await?;

        // Read-after-write: die persistierte Row als Antwort zurückliefern.
        let after = self.pdf_export_config_dao.get(tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok((&after).into())
    }

    async fn record_success(
        &self,
        at: time::PrimitiveDateTime,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Scheduler-only: erlaubt AUSSCHLIESSLICH Full-Auth (D-48-ADMIN).
        self.permission_service
            .check_only_full_authentication(context)
            .await?;
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let version = self
            .uuid_service
            .new_uuid("pdf-export-config-service::record_success version");
        self.pdf_export_config_dao
            .record_success(at, PROCESS_RECORD_SUCCESS, version, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn record_error(
        &self,
        at: time::PrimitiveDateTime,
        message: Arc<str>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        self.permission_service
            .check_only_full_authentication(context)
            .await?;
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let version = self
            .uuid_service
            .new_uuid("pdf-export-config-service::record_error version");
        self.pdf_export_config_dao
            .record_error(at, message.as_ref(), PROCESS_RECORD_ERROR, version, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
