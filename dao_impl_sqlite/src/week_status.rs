use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    week_status::{WeekStatusDao, WeekStatusEntity, WeekStatusKind},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

#[derive(Debug)]
struct WeekStatusDb {
    id: Vec<u8>,
    year: i64,
    calendar_week: i64,
    status: String,
    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&WeekStatusDb> for WeekStatusEntity {
    type Error = DaoError;

    fn try_from(db: &WeekStatusDb) -> Result<Self, Self::Error> {
        Ok(WeekStatusEntity {
            id: Uuid::from_slice(&db.id)?,
            year: db.year as u32,
            calendar_week: db.calendar_week as u8,
            status: match db.status.as_str() {
                "InPlanning" => WeekStatusKind::InPlanning,
                "Planned" => WeekStatusKind::Planned,
                "Locked" => WeekStatusKind::Locked,
                value => return Err(DaoError::EnumValueNotFound(value.into())),
            },
            created: PrimitiveDateTime::parse(&db.created, &Iso8601::DATE_TIME)?,
            deleted: db
                .deleted
                .as_ref()
                .map(|del| PrimitiveDateTime::parse(del, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&db.update_version)?,
        })
    }
}

/// Serialize the persisted discriminant. Explicit match (no `.to_string()`);
/// `WeekStatusKind` structurally has no `Unset` variant, so `Unset` can never
/// be written (D-39-04).
fn status_to_str(status: &WeekStatusKind) -> &'static str {
    match status {
        WeekStatusKind::InPlanning => "InPlanning",
        WeekStatusKind::Planned => "Planned",
        WeekStatusKind::Locked => "Locked",
    }
}

pub struct WeekStatusDaoImpl {
    pub pool: Arc<sqlx::SqlitePool>,
}

impl WeekStatusDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WeekStatusDao for WeekStatusDaoImpl {
    type Transaction = crate::TransactionImpl;

    async fn find_by_year_and_week(
        &self,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Option<WeekStatusEntity>, DaoError> {
        Ok(query_as!(
            WeekStatusDb,
            r#"SELECT id, year, calendar_week, status, created, deleted, update_version
               FROM week_status
               WHERE year = ? AND calendar_week = ? AND deleted IS NULL"#,
            year,
            calendar_week,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(WeekStatusEntity::try_from)
        .transpose()?)
    }

    async fn create(
        &self,
        entity: &WeekStatusEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let status_str = status_to_str(&entity.status);
        let created_str = entity.created.format(&Iso8601::DATE_TIME).map_db_error()?;
        let deleted_str = entity
            .deleted
            .map(|del| del.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let version_vec = entity.version.as_bytes().to_vec();

        query!(
            r#"INSERT INTO week_status (id, year, calendar_week, status, created, deleted, update_process, update_version)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
            id_vec,
            entity.year,
            entity.calendar_week,
            status_str,
            created_str,
            deleted_str,
            process,
            version_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }

    async fn update(
        &self,
        entity: &WeekStatusEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let status_str = status_to_str(&entity.status);
        let version_vec = entity.version.as_bytes().to_vec();

        query!(
            r#"UPDATE week_status
               SET status = ?, update_process = ?, update_version = ?
               WHERE id = ?"#,
            status_str,
            process,
            version_vec,
            id_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }

    async fn delete(&self, id: Uuid, process: &str, tx: Self::Transaction) -> Result<(), DaoError> {
        let id_vec = id.as_bytes().to_vec();
        let now_str = time::OffsetDateTime::now_utc()
            .format(&Iso8601::DATE_TIME)
            .map_db_error()?;

        query!(
            r#"UPDATE week_status
               SET deleted = ?, update_process = ?
               WHERE id = ?"#,
            now_str,
            process,
            id_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_db(status: &str) -> WeekStatusDb {
        WeekStatusDb {
            id: Uuid::nil().as_bytes().to_vec(),
            year: 2026,
            calendar_week: 27,
            status: status.to_string(),
            created: "2026-07-02T00:00:00".to_string(),
            deleted: None,
            update_version: Uuid::nil().as_bytes().to_vec(),
        }
    }

    #[test]
    fn unknown_discriminant() {
        let db = sample_db("Bogus");
        let result = WeekStatusEntity::try_from(&db);
        match result {
            Err(DaoError::EnumValueNotFound(value)) => assert_eq!(&*value, "Bogus"),
            other => panic!("expected EnumValueNotFound(\"Bogus\"), got {other:?}"),
        }
    }

    #[test]
    fn roundtrip_discriminant() {
        assert_eq!(
            WeekStatusEntity::try_from(&sample_db("InPlanning"))
                .unwrap()
                .status,
            WeekStatusKind::InPlanning
        );
        assert_eq!(
            WeekStatusEntity::try_from(&sample_db("Planned"))
                .unwrap()
                .status,
            WeekStatusKind::Planned
        );
        assert_eq!(
            WeekStatusEntity::try_from(&sample_db("Locked"))
                .unwrap()
                .status,
            WeekStatusKind::Locked
        );
    }
}
