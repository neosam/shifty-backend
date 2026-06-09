//! Phase 8.5 (D-04) — SQLite-Implementierung von `dao::migration_source::MigrationSourceDao`.
//!
//! Schreibt und liest den Backlink `extra_hours -> absence_period` in der
//! Tabelle `absence_period_migration_source`, befreit von `cutover_run_id`.
//! Uebernimmt das INSERT...ON CONFLICT(extra_hours_id) DO NOTHING-Muster aus
//! der ehemaligen `CutoverDaoImpl::upsert_migration_source`.

use std::sync::Arc;

use async_trait::async_trait;
use dao::migration_source::{MigrationSourceDao, MigrationSourceRow};
use dao::DaoError;
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

use crate::{ResultDbErrorExt, TransactionImpl};

struct MigrationSourceDb {
    extra_hours_id: Vec<u8>,
    absence_period_id: Vec<u8>,
    migrated_at: String,
}

impl TryFrom<&MigrationSourceDb> for MigrationSourceRow {
    type Error = DaoError;

    fn try_from(row: &MigrationSourceDb) -> Result<Self, DaoError> {
        Ok(Self {
            extra_hours_id: Uuid::from_slice(row.extra_hours_id.as_ref())?,
            absence_period_id: Uuid::from_slice(row.absence_period_id.as_ref())?,
            migrated_at: PrimitiveDateTime::parse(row.migrated_at.as_str(), &Iso8601::DATE_TIME)?,
        })
    }
}

pub struct MigrationSourceDaoImpl {
    _pool: Arc<sqlx::Pool<sqlx::Sqlite>>,
}

impl MigrationSourceDaoImpl {
    pub fn new(pool: Arc<sqlx::Pool<sqlx::Sqlite>>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl MigrationSourceDao for MigrationSourceDaoImpl {
    type Transaction = TransactionImpl;

    async fn upsert_migration_source(
        &self,
        row: &MigrationSourceRow,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let extra_hours_id = row.extra_hours_id.as_bytes().to_vec();
        let absence_period_id = row.absence_period_id.as_bytes().to_vec();
        let migrated_at = row.migrated_at.format(&Iso8601::DATE_TIME)?;
        sqlx::query!(
            r#"INSERT INTO absence_period_migration_source
               (extra_hours_id, absence_period_id, migrated_at)
               VALUES (?, ?, ?)
               ON CONFLICT(extra_hours_id) DO NOTHING"#,
            extra_hours_id,
            absence_period_id,
            migrated_at,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn find_by_extra_hours_id(
        &self,
        extra_hours_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<MigrationSourceRow>, DaoError> {
        let id_vec = extra_hours_id.as_bytes().to_vec();
        let row = sqlx::query_as!(
            MigrationSourceDb,
            r#"SELECT
                extra_hours_id    AS "extra_hours_id!: Vec<u8>",
                absence_period_id AS "absence_period_id!: Vec<u8>",
                migrated_at       AS "migrated_at!: String"
               FROM absence_period_migration_source
               WHERE extra_hours_id = ?"#,
            id_vec,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(row.as_ref().map(MigrationSourceRow::try_from).transpose()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransactionDaoImpl;
    use dao::TransactionDao;
    use std::sync::Arc;
    use time::macros::datetime;
    use uuid::Uuid;

    async fn setup_pool() -> Arc<sqlx::SqlitePool> {
        let pool = Arc::new(
            sqlx::SqlitePool::connect("sqlite::memory:")
                .await
                .expect("Could not connect to in-memory SQLite"),
        );
        sqlx::migrate!("./../migrations/sqlite")
            .run(pool.as_ref())
            .await
            .expect("Could not run migrations");
        pool
    }

    #[tokio::test]
    async fn migration_source_roundtrip_without_cutover_run_id() {
        let pool = setup_pool().await;
        let dao = MigrationSourceDaoImpl::new(pool.clone());
        let tx_dao = TransactionDaoImpl::new(pool.clone());

        let extra_hours_id = Uuid::new_v4();
        let absence_period_id = Uuid::new_v4();
        let migrated_at = datetime!(2026-06-09 12:00:00);

        let row = MigrationSourceRow {
            extra_hours_id,
            absence_period_id,
            migrated_at,
        };

        // WRITE
        let tx = tx_dao.new_transaction().await.expect("tx");
        dao.upsert_migration_source(&row, tx.clone())
            .await
            .expect("upsert should succeed");
        tx_dao.commit(tx).await.expect("commit");

        // READ BACK
        let tx2 = tx_dao.new_transaction().await.expect("tx2");
        let found = dao
            .find_by_extra_hours_id(extra_hours_id, tx2.clone())
            .await
            .expect("find should succeed");
        tx_dao.commit(tx2).await.expect("commit2");

        let found = found.expect("row should be present");
        assert_eq!(found.extra_hours_id, extra_hours_id);
        assert_eq!(found.absence_period_id, absence_period_id);
        assert_eq!(found.migrated_at, migrated_at);
        // Implizit: MigrationSourceRow hat kein cutover_run_id-Feld (Compiler-Garantie)
    }

    #[tokio::test]
    async fn migration_source_upsert_idempotent_do_nothing() {
        let pool = setup_pool().await;
        let dao = MigrationSourceDaoImpl::new(pool.clone());
        let tx_dao = TransactionDaoImpl::new(pool.clone());

        let extra_hours_id = Uuid::new_v4();
        let absence_period_id_first = Uuid::new_v4();
        let absence_period_id_second = Uuid::new_v4();
        let migrated_at = datetime!(2026-06-09 12:00:00);

        let row1 = MigrationSourceRow {
            extra_hours_id,
            absence_period_id: absence_period_id_first,
            migrated_at,
        };
        let row2 = MigrationSourceRow {
            extra_hours_id,
            absence_period_id: absence_period_id_second,
            migrated_at,
        };

        // Erster UPSERT
        let tx = tx_dao.new_transaction().await.expect("tx");
        dao.upsert_migration_source(&row1, tx.clone())
            .await
            .expect("first upsert");
        tx_dao.commit(tx).await.expect("commit");

        // Zweiter UPSERT (DO NOTHING — soll Eintrag nicht ueberschreiben)
        let tx2 = tx_dao.new_transaction().await.expect("tx2");
        dao.upsert_migration_source(&row2, tx2.clone())
            .await
            .expect("second upsert should not error");
        tx_dao.commit(tx2).await.expect("commit2");

        // Lesen: erster absence_period_id muss erhalten bleiben
        let tx3 = tx_dao.new_transaction().await.expect("tx3");
        let found = dao
            .find_by_extra_hours_id(extra_hours_id, tx3.clone())
            .await
            .expect("find");
        tx_dao.commit(tx3).await.expect("commit3");

        let found = found.expect("row should be present");
        assert_eq!(found.absence_period_id, absence_period_id_first,
            "DO NOTHING: second upsert should not overwrite first entry");
    }

    #[tokio::test]
    async fn migration_source_find_returns_none_for_unknown_id() {
        let pool = setup_pool().await;
        let dao = MigrationSourceDaoImpl::new(pool.clone());
        let tx_dao = TransactionDaoImpl::new(pool.clone());

        let unknown_id = Uuid::new_v4();
        let tx = tx_dao.new_transaction().await.expect("tx");
        let result = dao
            .find_by_extra_hours_id(unknown_id, tx.clone())
            .await
            .expect("find should not error");
        tx_dao.commit(tx).await.expect("commit");

        assert!(result.is_none(), "unknown extra_hours_id should return None");
    }
}
