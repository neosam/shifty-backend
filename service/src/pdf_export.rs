//! PDF-Export Scheduler-Trait (Phase 48 EXP-01 + EXP-03).
//!
//! Business-Logic-Tier per Service-Tier-Konvention: kombiniert
//! [`crate::pdf_export_config::PdfExportConfigService`] (Basic) +
//! [`crate::shiftplan::ShiftplanViewService`] (Read-Aggregat) +
//! [`crate::shiftplan_catalog::ShiftplanService`] +
//! [`crate::sales_person::SalesPersonService`] + pure Rendering (48-02) +
//! WebDAV-Upload (48-03).
//!
//! Der Scheduler kapselt die Cron-Schleife (`tokio-cron-scheduler`),
//! Retry-Persistenz (`record_success` / `record_error` via
//! [`crate::pdf_export_config::PdfExportConfigService`]) und einen optionalen
//! „Jetzt exportieren"-Trigger. Die Cron-Loop läuft mit
//! [`crate::permission::Authentication::Full`]; der REST-Handler
//! `POST /pdf-export-config/trigger` prüft das admin-Privileg VOR dem
//! `spawn`.

use std::fmt::Debug;

use async_trait::async_trait;
use mockall::automock;

use crate::{permission::Authentication, ServiceError};

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait PdfExportScheduler {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Beim App-Boot: Lädt die aktuelle Config und registriert den Cron-Job.
    /// Wenn `enabled=false` steht der Job dormant (Registration aber ohne
    /// aktiven Trigger).
    async fn start(&self) -> Result<(), ServiceError>;

    /// Nach `PUT /pdf-export-config`: Alt-Job entfernen, neuen Cron aus der
    /// aktuellen Config registrieren. Erwartet keinen User-Context.
    async fn reload_from_db(&self) -> Result<(), ServiceError>;

    /// Löst genau EINEN sofortigen Export-Lauf synchron aus. Wird vom
    /// Cron-Trigger (mit `Authentication::Full`) und vom REST-Handler
    /// `POST /pdf-export-config/trigger` (nach admin-check, dann spawn)
    /// aufgerufen.
    ///
    /// Idempotent bzgl. Nextcloud-Filenames (overwrite via PUT). Fehler
    /// werden IM Scheduler persistiert (`record_error`), nicht nach oben
    /// gereicht — der Return-Type ist `Ok(())` außer bei Auth-Fehlern.
    async fn run_once_now(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
}
