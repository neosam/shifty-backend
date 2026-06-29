use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    vacation_entitlement_offset::{VacationEntitlementOffsetDao, VacationEntitlementOffsetEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

#[derive(Debug)]
struct VacationEntitlementOffsetDb {
    id: Vec<u8>,
    sales_person_id: Vec<u8>,
    year: i64,
    offset_days: i64,
    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&VacationEntitlementOffsetDb> for VacationEntitlementOffsetEntity {
    type Error = DaoError;

    fn try_from(db: &VacationEntitlementOffsetDb) -> Result<Self, Self::Error> {
        Ok(VacationEntitlementOffsetEntity {
            id: Uuid::from_slice(&db.id)?,
            sales_person_id: Uuid::from_slice(&db.sales_person_id)?,
            year: db.year as u32,
            offset_days: db.offset_days as i32,
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

pub struct VacationEntitlementOffsetDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl VacationEntitlementOffsetDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl VacationEntitlementOffsetDao for VacationEntitlementOffsetDaoImpl {
    type Transaction = crate::TransactionImpl;

    async fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<Option<VacationEntitlementOffsetEntity>, DaoError> {
        let sales_person_id_vec = sales_person_id.as_bytes().to_vec();
        Ok(query_as!(
            VacationEntitlementOffsetDb,
            r#"SELECT id, sales_person_id, year, offset_days, created, deleted, update_version
               FROM vacation_entitlement_offset
               WHERE sales_person_id = ? AND year = ? AND deleted IS NULL"#,
            sales_person_id_vec,
            year,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(VacationEntitlementOffsetEntity::try_from)
        .transpose()?)
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<VacationEntitlementOffsetEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            VacationEntitlementOffsetDb,
            r#"SELECT id, sales_person_id, year, offset_days, created, deleted, update_version
               FROM vacation_entitlement_offset
               WHERE id = ?"#,
            id_vec,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(VacationEntitlementOffsetEntity::try_from)
        .transpose()?)
    }

    async fn create(
        &self,
        entity: &VacationEntitlementOffsetEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let sales_person_id_vec = entity.sales_person_id.as_bytes().to_vec();
        let created_str = entity.created.format(&Iso8601::DATE_TIME).map_db_error()?;
        let deleted_str = entity
            .deleted
            .map(|del| del.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let version_vec = entity.version.as_bytes().to_vec();

        query!(
            r#"INSERT INTO vacation_entitlement_offset (id, sales_person_id, year, offset_days, created, deleted, update_process, update_version)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
            id_vec,
            sales_person_id_vec,
            entity.year,
            entity.offset_days,
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
        entity: &VacationEntitlementOffsetEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let deleted_str = entity
            .deleted
            .map(|del| del.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let version_vec = entity.version.as_bytes().to_vec();

        query!(
            r#"UPDATE vacation_entitlement_offset
               SET offset_days = ?, deleted = ?, update_process = ?, update_version = ?
               WHERE id = ?"#,
            entity.offset_days,
            deleted_str,
            process,
            version_vec,
            id_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }
}
