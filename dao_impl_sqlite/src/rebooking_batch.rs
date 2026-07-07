//! Phase 54 (VOL-ACCT / D-54-DM-01): SQLite-Implementierung des
//! Basic-Tier `RebookingBatchDao`. Konsumenten folgen ab Phase 55/56.

use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    rebooking_batch::{
        RebookingBatchDao, RebookingBatchEntity, RebookingBatchEntryEntity, RebookingBatchKind,
        RebookingBatchState,
    },
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

use crate::{ResultDbErrorExt, TransactionImpl};

// ---------------------------------------------------------------------------
// Row-Layout
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct RebookingBatchDb {
    id: Vec<u8>,
    sales_person_id: Vec<u8>,
    iso_year: i64,
    iso_week: i64,
    kind: String,
    state: String,
    created: String,
    approved: Option<String>,
    approved_by: Option<String>,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&RebookingBatchDb> for RebookingBatchEntity {
    type Error = DaoError;

    fn try_from(db: &RebookingBatchDb) -> Result<Self, Self::Error> {
        Ok(RebookingBatchEntity {
            id: Uuid::from_slice(&db.id)?,
            sales_person_id: Uuid::from_slice(&db.sales_person_id)?,
            iso_year: db.iso_year as u32,
            iso_week: db.iso_week as u8,
            kind: str_to_kind(&db.kind)?,
            state: str_to_state(&db.state)?,
            created: PrimitiveDateTime::parse(&db.created, &Iso8601::DATE_TIME)?,
            approved: db
                .approved
                .as_ref()
                .map(|s| PrimitiveDateTime::parse(s, &Iso8601::DATE_TIME))
                .transpose()?,
            approved_by: db.approved_by.as_ref().map(|s| Arc::<str>::from(s.as_str())),
            deleted: db
                .deleted
                .as_ref()
                .map(|s| PrimitiveDateTime::parse(s, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&db.update_version)?,
        })
    }
}

#[derive(Debug)]
struct RebookingBatchEntryDb {
    id: Vec<u8>,
    batch_id: Vec<u8>,
    sales_person_id: Vec<u8>,
    hours: f64,
    balance_before: f64,
    voluntary_actual: f64,
    voluntary_committed: f64,
    extra_hours_out_id: Option<Vec<u8>>,
    extra_hours_in_id: Option<Vec<u8>>,
    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&RebookingBatchEntryDb> for RebookingBatchEntryEntity {
    type Error = DaoError;

    fn try_from(db: &RebookingBatchEntryDb) -> Result<Self, Self::Error> {
        Ok(RebookingBatchEntryEntity {
            id: Uuid::from_slice(&db.id)?,
            batch_id: Uuid::from_slice(&db.batch_id)?,
            sales_person_id: Uuid::from_slice(&db.sales_person_id)?,
            hours: db.hours as f32,
            balance_before: db.balance_before as f32,
            voluntary_actual: db.voluntary_actual as f32,
            voluntary_committed: db.voluntary_committed as f32,
            extra_hours_out_id: db
                .extra_hours_out_id
                .as_ref()
                .map(|v| Uuid::from_slice(v))
                .transpose()?,
            extra_hours_in_id: db
                .extra_hours_in_id
                .as_ref()
                .map(|v| Uuid::from_slice(v))
                .transpose()?,
            created: PrimitiveDateTime::parse(&db.created, &Iso8601::DATE_TIME)?,
            deleted: db
                .deleted
                .as_ref()
                .map(|s| PrimitiveDateTime::parse(s, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&db.update_version)?,
        })
    }
}

// ---------------------------------------------------------------------------
// Enum <-> String Konversionen
// ---------------------------------------------------------------------------

fn kind_to_str(kind: &RebookingBatchKind) -> &'static str {
    match kind {
        RebookingBatchKind::Manual => "manual",
        RebookingBatchKind::HrSuggestion => "hr_suggestion",
        RebookingBatchKind::AutoCron => "auto_cron",
        RebookingBatchKind::AutoCronBackfill => "auto_cron_backfill",
    }
}

fn str_to_kind(s: &str) -> Result<RebookingBatchKind, DaoError> {
    match s {
        "manual" => Ok(RebookingBatchKind::Manual),
        "hr_suggestion" => Ok(RebookingBatchKind::HrSuggestion),
        "auto_cron" => Ok(RebookingBatchKind::AutoCron),
        "auto_cron_backfill" => Ok(RebookingBatchKind::AutoCronBackfill),
        other => Err(DaoError::EnumValueNotFound(other.into())),
    }
}

fn state_to_str(state: &RebookingBatchState) -> &'static str {
    match state {
        RebookingBatchState::Pending => "pending",
        RebookingBatchState::Approved => "approved",
        RebookingBatchState::Rejected => "rejected",
        RebookingBatchState::SkippedLocked => "skipped_locked",
    }
}

fn str_to_state(s: &str) -> Result<RebookingBatchState, DaoError> {
    match s {
        "pending" => Ok(RebookingBatchState::Pending),
        "approved" => Ok(RebookingBatchState::Approved),
        "rejected" => Ok(RebookingBatchState::Rejected),
        "skipped_locked" => Ok(RebookingBatchState::SkippedLocked),
        other => Err(DaoError::EnumValueNotFound(other.into())),
    }
}

// ---------------------------------------------------------------------------
// DAO-Impl
// ---------------------------------------------------------------------------

pub struct RebookingBatchDaoImpl {
    pub pool: Arc<sqlx::SqlitePool>,
}

impl RebookingBatchDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RebookingBatchDao for RebookingBatchDaoImpl {
    type Transaction = TransactionImpl;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<RebookingBatchEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            RebookingBatchDb,
            r#"SELECT id, sales_person_id, iso_year, iso_week, kind, state, created,
                      approved, approved_by, deleted, update_version
               FROM rebooking_batch
               WHERE id = ? AND deleted IS NULL"#,
            id_vec,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(RebookingBatchEntity::try_from)
        .transpose()?)
    }

    async fn find_by_sales_person_year_week(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        tx: Self::Transaction,
    ) -> Result<Option<RebookingBatchEntity>, DaoError> {
        let sp_vec = sales_person_id.as_bytes().to_vec();
        Ok(query_as!(
            RebookingBatchDb,
            r#"SELECT id, sales_person_id, iso_year, iso_week, kind, state, created,
                      approved, approved_by, deleted, update_version
               FROM rebooking_batch
               WHERE sales_person_id = ? AND iso_year = ? AND iso_week = ?
                 AND deleted IS NULL"#,
            sp_vec,
            iso_year,
            iso_week,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(RebookingBatchEntity::try_from)
        .transpose()?)
    }

    async fn create_batch_with_entries(
        &self,
        batch: &RebookingBatchEntity,
        entries: &[RebookingBatchEntryEntity],
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = batch.id.as_bytes().to_vec();
        let sp_vec = batch.sales_person_id.as_bytes().to_vec();
        let kind_str = kind_to_str(&batch.kind);
        let state_str = state_to_str(&batch.state);
        let created_str = batch.created.format(&Iso8601::DATE_TIME).map_db_error()?;
        let approved_str = batch
            .approved
            .map(|d| d.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let approved_by_str = batch.approved_by.as_ref().map(|s| s.to_string());
        let deleted_str = batch
            .deleted
            .map(|d| d.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let version_vec = batch.version.as_bytes().to_vec();

        query!(
            r#"INSERT INTO rebooking_batch
                   (id, sales_person_id, iso_year, iso_week, kind, state, created,
                    approved, approved_by, deleted, update_process, update_version)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            id_vec,
            sp_vec,
            batch.iso_year,
            batch.iso_week,
            kind_str,
            state_str,
            created_str,
            approved_str,
            approved_by_str,
            deleted_str,
            process,
            version_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        for entry in entries {
            let e_id = entry.id.as_bytes().to_vec();
            let e_batch = entry.batch_id.as_bytes().to_vec();
            let e_sp = entry.sales_person_id.as_bytes().to_vec();
            let e_hours = entry.hours as f64;
            let e_balance = entry.balance_before as f64;
            let e_va = entry.voluntary_actual as f64;
            let e_vc = entry.voluntary_committed as f64;
            let e_out = entry.extra_hours_out_id.map(|u| u.as_bytes().to_vec());
            let e_in = entry.extra_hours_in_id.map(|u| u.as_bytes().to_vec());
            let e_created = entry.created.format(&Iso8601::DATE_TIME).map_db_error()?;
            let e_deleted = entry
                .deleted
                .map(|d| d.format(&Iso8601::DATE_TIME))
                .transpose()
                .map_db_error()?;
            let e_version = entry.version.as_bytes().to_vec();

            query!(
                r#"INSERT INTO rebooking_batch_entry
                       (id, batch_id, sales_person_id, hours, balance_before,
                        voluntary_actual, voluntary_committed,
                        extra_hours_out_id, extra_hours_in_id,
                        created, deleted, update_process, update_version)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                e_id,
                e_batch,
                e_sp,
                e_hours,
                e_balance,
                e_va,
                e_vc,
                e_out,
                e_in,
                e_created,
                e_deleted,
                process,
                e_version,
            )
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        }

        Ok(())
    }

    async fn list_entries_for_batch(
        &self,
        batch_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[RebookingBatchEntryEntity]>, DaoError> {
        let batch_vec = batch_id.as_bytes().to_vec();
        Ok(query_as!(
            RebookingBatchEntryDb,
            r#"SELECT id, batch_id, sales_person_id, hours, balance_before,
                      voluntary_actual, voluntary_committed,
                      extra_hours_out_id, extra_hours_in_id,
                      created, deleted, update_version
               FROM rebooking_batch_entry
               WHERE batch_id = ? AND deleted IS NULL"#,
            batch_vec,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(RebookingBatchEntryEntity::try_from)
        .collect::<Result<Arc<[_]>, _>>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_roundtrip() {
        for k in [
            RebookingBatchKind::Manual,
            RebookingBatchKind::HrSuggestion,
            RebookingBatchKind::AutoCron,
            RebookingBatchKind::AutoCronBackfill,
        ] {
            assert_eq!(str_to_kind(kind_to_str(&k)).unwrap(), k);
        }
    }

    #[test]
    fn state_roundtrip() {
        for s in [
            RebookingBatchState::Pending,
            RebookingBatchState::Approved,
            RebookingBatchState::Rejected,
            RebookingBatchState::SkippedLocked,
        ] {
            assert_eq!(str_to_state(state_to_str(&s)).unwrap(), s);
        }
    }

    #[test]
    fn unknown_kind_is_enum_value_not_found() {
        match str_to_kind("bogus") {
            Err(DaoError::EnumValueNotFound(v)) => assert_eq!(&*v, "bogus"),
            other => panic!("expected EnumValueNotFound(\"bogus\"), got {other:?}"),
        }
    }

    #[test]
    fn unknown_state_is_enum_value_not_found() {
        match str_to_state("bogus") {
            Err(DaoError::EnumValueNotFound(v)) => assert_eq!(&*v, "bogus"),
            other => panic!("expected EnumValueNotFound(\"bogus\"), got {other:?}"),
        }
    }
}
