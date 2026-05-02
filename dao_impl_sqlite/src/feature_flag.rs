use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    feature_flag::{FeatureFlagDao, FeatureFlagEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, OffsetDateTime};

#[derive(Debug)]
struct FeatureFlagDb {
    key: String,
    enabled: i64,
    description: Option<String>,
}

impl From<&FeatureFlagDb> for FeatureFlagEntity {
    fn from(db: &FeatureFlagDb) -> Self {
        FeatureFlagEntity {
            key: db.key.clone(),
            enabled: db.enabled != 0,
            description: db.description.clone(),
        }
    }
}

pub struct FeatureFlagDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl FeatureFlagDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl FeatureFlagDao for FeatureFlagDaoImpl {
    type Transaction = crate::TransactionImpl;

    async fn is_enabled(&self, key: &str, tx: Self::Transaction) -> Result<bool, DaoError> {
        let result = query!(
            r#"SELECT enabled FROM feature_flag WHERE key = ?"#,
            key,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        // Returns false for non-existent feature flags (fail-safe default,
        // analog ToggleDaoImpl::is_enabled)
        Ok(result.map(|row| row.enabled != 0).unwrap_or(false))
    }

    async fn get(
        &self,
        key: &str,
        tx: Self::Transaction,
    ) -> Result<Option<FeatureFlagEntity>, DaoError> {
        Ok(query_as!(
            FeatureFlagDb,
            r#"SELECT key, enabled, description
               FROM feature_flag
               WHERE key = ?"#,
            key,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(FeatureFlagEntity::from))
    }

    async fn set(
        &self,
        key: &str,
        enabled: bool,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let enabled_int: i64 = if enabled { 1 } else { 0 };
        let timestamp = OffsetDateTime::now_utc()
            .format(&Iso8601::DEFAULT)
            .map_err(DaoError::DateTimeFormatError)?;
        // UPDATE-only: migration must seed all known keys.
        query!(
            r#"UPDATE feature_flag
               SET enabled = ?, update_timestamp = ?, update_process = ?
               WHERE key = ?"#,
            enabled_int,
            timestamp,
            process,
            key,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }
}
