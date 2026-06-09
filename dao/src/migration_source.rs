//! Phase 8.5 (D-04) — Backlink `extra_hours -> absence_period`, befreit von
//! Cutover-Run-Semantik.
//!
//! Dieses Modul ersetzt den Zugriff auf `absence_period_migration_source` ueber
//! `CutoverDao::upsert_migration_source`. Es bleibt erhalten wenn in Phase 8.6
//! der `CutoverDao` vollstaendig subtraktiv geloescht wird.

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::DaoError;

/// Persistierte Backlink-Zeile: ordnet eine `extra_hours`-Row einer
/// `absence_period` zu. Befreit von `cutover_run_id` (Phase 8.5, D-04).
#[derive(Clone, Debug, PartialEq)]
pub struct MigrationSourceRow {
    pub extra_hours_id: Uuid,
    pub absence_period_id: Uuid,
    pub migrated_at: time::PrimitiveDateTime,
}

/// DAO-Trait fuer den Backlink-Speicher (`absence_period_migration_source`).
///
/// Semantik identisch zur ehemaligen `CutoverDao::upsert_migration_source`,
/// aber ohne `cutover_run_id`-Kopplung. Phase 8.6 kann `CutoverDao` vollstaendig
/// subtraktiv loeschen — dieser Trait und seine Impl ueberleben.
#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait MigrationSourceDao {
    type Transaction: crate::Transaction;

    /// UPSERT (INSERT ... ON CONFLICT(extra_hours_id) DO NOTHING) in
    /// `absence_period_migration_source`. Idempotent: bereits vorhandene
    /// Eintraege werden nicht ueberschrieben.
    async fn upsert_migration_source(
        &self,
        row: &MigrationSourceRow,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    /// Lookup: gibt den Backlink-Eintrag zu einer `extra_hours_id` zurueck.
    /// `None` falls kein Backlink existiert (extra_hours nicht konvertiert).
    async fn find_by_extra_hours_id(
        &self,
        extra_hours_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<MigrationSourceRow>, DaoError>;
}
