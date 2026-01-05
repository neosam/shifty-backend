use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    toggle::{ToggleDao, ToggleEntity, ToggleGroupEntity},
    DaoError,
};
use sqlx::{query, query_as};

#[derive(Debug)]
struct ToggleDb {
    name: String,
    enabled: i64,
    description: Option<String>,
}

impl From<&ToggleDb> for ToggleEntity {
    fn from(db: &ToggleDb) -> Self {
        ToggleEntity {
            name: db.name.clone(),
            enabled: db.enabled != 0,
            description: db.description.clone(),
        }
    }
}

#[derive(Debug)]
struct ToggleGroupDb {
    name: String,
    description: Option<String>,
}

impl From<&ToggleGroupDb> for ToggleGroupEntity {
    fn from(db: &ToggleGroupDb) -> Self {
        ToggleGroupEntity {
            name: db.name.clone(),
            description: db.description.clone(),
        }
    }
}

pub struct ToggleDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl ToggleDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl ToggleDao for ToggleDaoImpl {
    type Transaction = crate::TransactionImpl;

    async fn create_toggle(
        &self,
        toggle: &ToggleEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let enabled: i64 = if toggle.enabled { 1 } else { 0 };
        query!(
            r#"INSERT INTO toggle (name, enabled, description, update_process)
               VALUES (?, ?, ?, ?)"#,
            toggle.name,
            enabled,
            toggle.description,
            process,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn get_toggle(
        &self,
        name: &str,
        tx: Self::Transaction,
    ) -> Result<Option<ToggleEntity>, DaoError> {
        Ok(query_as!(
            ToggleDb,
            r#"SELECT name, enabled, description
               FROM toggle
               WHERE name = ?"#,
            name,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(ToggleEntity::from))
    }

    async fn get_all_toggles(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[ToggleEntity]>, DaoError> {
        let rows = query_as!(
            ToggleDb,
            r#"SELECT name, enabled, description
               FROM toggle
               ORDER BY name"#,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(rows.iter().map(ToggleEntity::from).collect())
    }

    async fn update_toggle(
        &self,
        toggle: &ToggleEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let enabled: i64 = if toggle.enabled { 1 } else { 0 };
        query!(
            r#"UPDATE toggle
               SET enabled = ?, description = ?, update_process = ?
               WHERE name = ?"#,
            enabled,
            toggle.description,
            process,
            toggle.name,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn delete_toggle(
        &self,
        name: &str,
        _process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        // First remove from all groups
        query!(
            r#"DELETE FROM toggle_group_toggle WHERE toggle_name = ?"#,
            name,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        // Then delete the toggle itself
        query!(r#"DELETE FROM toggle WHERE name = ?"#, name,)
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        Ok(())
    }

    async fn is_enabled(&self, name: &str, tx: Self::Transaction) -> Result<bool, DaoError> {
        let result = query!(
            r#"SELECT enabled FROM toggle WHERE name = ?"#,
            name,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        // Returns false for non-existent toggles (fail-safe default)
        Ok(result.map(|row| row.enabled != 0).unwrap_or(false))
    }

    async fn create_toggle_group(
        &self,
        group: &ToggleGroupEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        query!(
            r#"INSERT INTO toggle_group (name, description, update_process)
               VALUES (?, ?, ?)"#,
            group.name,
            group.description,
            process,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn get_toggle_group(
        &self,
        name: &str,
        tx: Self::Transaction,
    ) -> Result<Option<ToggleGroupEntity>, DaoError> {
        Ok(query_as!(
            ToggleGroupDb,
            r#"SELECT name, description
               FROM toggle_group
               WHERE name = ?"#,
            name,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(ToggleGroupEntity::from))
    }

    async fn get_all_toggle_groups(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[ToggleGroupEntity]>, DaoError> {
        let rows = query_as!(
            ToggleGroupDb,
            r#"SELECT name, description
               FROM toggle_group
               ORDER BY name"#,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(rows.iter().map(ToggleGroupEntity::from).collect())
    }

    async fn delete_toggle_group(
        &self,
        name: &str,
        _process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        // First remove all toggle associations
        query!(
            r#"DELETE FROM toggle_group_toggle WHERE toggle_group_name = ?"#,
            name,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        // Then delete the group itself
        query!(r#"DELETE FROM toggle_group WHERE name = ?"#, name,)
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        Ok(())
    }

    async fn add_toggle_to_group(
        &self,
        group: &str,
        toggle: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        query!(
            r#"INSERT INTO toggle_group_toggle (toggle_group_name, toggle_name, update_process)
               VALUES (?, ?, ?)"#,
            group,
            toggle,
            process,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn remove_toggle_from_group(
        &self,
        group: &str,
        toggle: &str,
        _process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        query!(
            r#"DELETE FROM toggle_group_toggle
               WHERE toggle_group_name = ? AND toggle_name = ?"#,
            group,
            toggle,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn get_toggles_in_group(
        &self,
        group: &str,
        tx: Self::Transaction,
    ) -> Result<Arc<[ToggleEntity]>, DaoError> {
        let rows = query_as!(
            ToggleDb,
            r#"SELECT t.name, t.enabled, t.description
               FROM toggle t
               INNER JOIN toggle_group_toggle tgt ON t.name = tgt.toggle_name
               WHERE tgt.toggle_group_name = ?
               ORDER BY t.name"#,
            group,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(rows.iter().map(ToggleEntity::from).collect())
    }

    async fn get_groups_for_toggle(
        &self,
        toggle: &str,
        tx: Self::Transaction,
    ) -> Result<Arc<[ToggleGroupEntity]>, DaoError> {
        let rows = query_as!(
            ToggleGroupDb,
            r#"SELECT tg.name, tg.description
               FROM toggle_group tg
               INNER JOIN toggle_group_toggle tgt ON tg.name = tgt.toggle_group_name
               WHERE tgt.toggle_name = ?
               ORDER BY tg.name"#,
            toggle,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(rows.iter().map(ToggleGroupEntity::from).collect())
    }

    async fn enable_group(
        &self,
        group: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        query!(
            r#"UPDATE toggle
               SET enabled = 1, update_process = ?
               WHERE name IN (
                   SELECT toggle_name FROM toggle_group_toggle WHERE toggle_group_name = ?
               )"#,
            process,
            group,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn disable_group(
        &self,
        group: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        query!(
            r#"UPDATE toggle
               SET enabled = 0, update_process = ?
               WHERE name IN (
                   SELECT toggle_name FROM toggle_group_toggle WHERE toggle_group_name = ?
               )"#,
            process,
            group,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }
}
