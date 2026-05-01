use std::sync::Arc;

use crate::{ResultDbErrorExt, TransactionImpl};
use async_trait::async_trait;
use dao::{
    absence::{AbsenceCategoryEntity, AbsenceDao, AbsencePeriodEntity},
    DaoError,
};
use shifty_utils::DateRange;
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, Date, PrimitiveDateTime};
use uuid::Uuid;

struct AbsencePeriodDb {
    id: Vec<u8>,
    logical_id: Vec<u8>,
    sales_person_id: Vec<u8>,
    category: String,
    from_date: String,
    to_date: String,
    description: Option<String>,
    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&AbsencePeriodDb> for AbsencePeriodEntity {
    type Error = DaoError;

    fn try_from(row: &AbsencePeriodDb) -> Result<Self, DaoError> {
        Ok(Self {
            id: Uuid::from_slice(row.id.as_ref())?,
            logical_id: Uuid::from_slice(row.logical_id.as_ref())?,
            sales_person_id: Uuid::from_slice(row.sales_person_id.as_ref())?,
            category: match row.category.as_str() {
                "Vacation" => AbsenceCategoryEntity::Vacation,
                "SickLeave" => AbsenceCategoryEntity::SickLeave,
                "UnpaidLeave" => AbsenceCategoryEntity::UnpaidLeave,
                value => return Err(DaoError::EnumValueNotFound(value.into())),
            },
            from_date: Date::parse(row.from_date.as_str(), &Iso8601::DATE)?,
            to_date: Date::parse(row.to_date.as_str(), &Iso8601::DATE)?,
            description: row
                .description
                .clone()
                .unwrap_or_default()
                .as_str()
                .into(),
            created: PrimitiveDateTime::parse(row.created.as_str(), &Iso8601::DATE_TIME)?,
            deleted: row
                .deleted
                .as_ref()
                .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&row.update_version)?,
        })
    }
}

fn category_to_str(c: &AbsenceCategoryEntity) -> &'static str {
    match c {
        AbsenceCategoryEntity::Vacation => "Vacation",
        AbsenceCategoryEntity::SickLeave => "SickLeave",
        AbsenceCategoryEntity::UnpaidLeave => "UnpaidLeave",
    }
}

pub struct AbsenceDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl AbsenceDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl AbsenceDao for AbsenceDaoImpl {
    type Transaction = TransactionImpl;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<AbsencePeriodEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            AbsencePeriodDb,
            "SELECT id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_version FROM absence_period WHERE id = ? AND deleted IS NULL",
            id_vec,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(AbsencePeriodEntity::try_from)
        .transpose()?)
    }

    async fn find_by_logical_id(
        &self,
        logical_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<AbsencePeriodEntity>, DaoError> {
        let logical_id_vec = logical_id.as_bytes().to_vec();
        Ok(query_as!(
            AbsencePeriodDb,
            "SELECT id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_version FROM absence_period WHERE logical_id = ? AND deleted IS NULL",
            logical_id_vec,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(AbsencePeriodEntity::try_from)
        .transpose()?)
    }

    async fn find_by_sales_person(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[AbsencePeriodEntity]>, DaoError> {
        let sp_vec = sales_person_id.as_bytes().to_vec();
        Ok(query_as!(
            AbsencePeriodDb,
            "SELECT id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_version FROM absence_period WHERE sales_person_id = ? AND deleted IS NULL ORDER BY from_date",
            sp_vec,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(AbsencePeriodEntity::try_from)
        .collect::<Result<Arc<[_]>, _>>()?)
    }

    async fn find_all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[AbsencePeriodEntity]>, DaoError> {
        Ok(query_as!(
            AbsencePeriodDb,
            "SELECT id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_version FROM absence_period WHERE deleted IS NULL ORDER BY sales_person_id, from_date",
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(AbsencePeriodEntity::try_from)
        .collect::<Result<Arc<[_]>, _>>()?)
    }

    async fn find_overlapping(
        &self,
        sales_person_id: Uuid,
        category: AbsenceCategoryEntity,
        range: DateRange,
        exclude_logical_id: Option<Uuid>,
        tx: Self::Transaction,
    ) -> Result<Arc<[AbsencePeriodEntity]>, DaoError> {
        let sp_vec = sales_person_id.as_bytes().to_vec();
        let category_str = category_to_str(&category);
        // ISO-8601 YYYY-MM-DD; lex-sort == date-sort.
        let from_str = range.from().format(&Iso8601::DATE)?;
        let to_str = range.to().format(&Iso8601::DATE)?;

        let rows = match exclude_logical_id {
            Some(exclude) => {
                let exclude_vec = exclude.as_bytes().to_vec();
                query_as!(
                    AbsencePeriodDb,
                    "SELECT id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_version FROM absence_period WHERE sales_person_id = ? AND category = ? AND from_date <= ? AND to_date >= ? AND logical_id != ? AND deleted IS NULL",
                    sp_vec,
                    category_str,
                    to_str,
                    from_str,
                    exclude_vec,
                )
                .fetch_all(tx.tx.lock().await.as_mut())
                .await
                .map_db_error()?
            }
            None => {
                query_as!(
                    AbsencePeriodDb,
                    "SELECT id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_version FROM absence_period WHERE sales_person_id = ? AND category = ? AND from_date <= ? AND to_date >= ? AND deleted IS NULL",
                    sp_vec,
                    category_str,
                    to_str,
                    from_str,
                )
                .fetch_all(tx.tx.lock().await.as_mut())
                .await
                .map_db_error()?
            }
        };

        Ok(rows
            .iter()
            .map(AbsencePeriodEntity::try_from)
            .collect::<Result<Arc<[_]>, _>>()?)
    }

    async fn create(
        &self,
        entity: &AbsencePeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let logical_id_vec = entity.logical_id.as_bytes().to_vec();
        let sp_vec = entity.sales_person_id.as_bytes().to_vec();
        let category_str = category_to_str(&entity.category);
        let from_str = entity.from_date.format(&Iso8601::DATE)?;
        let to_str = entity.to_date.format(&Iso8601::DATE)?;
        let description = entity.description.as_ref();
        let created = entity.created.format(&Iso8601::DATE_TIME)?;
        let deleted = entity
            .deleted
            .map(|dt| dt.format(&Iso8601::DATE_TIME))
            .transpose()?;
        let version_vec = entity.version.as_bytes().to_vec();
        query!(
            "INSERT INTO absence_period (id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_process, update_version) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id_vec,
            logical_id_vec,
            sp_vec,
            category_str,
            from_str,
            to_str,
            description,
            created,
            deleted,
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
        entity: &AbsencePeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let version_vec = entity.version.as_bytes().to_vec();
        let delete = entity
            .deleted
            .map(|dt| dt.format(&Iso8601::DATE_TIME))
            .transpose()?;
        query!(
            "UPDATE absence_period SET deleted = ?, update_version = ?, update_process = ? WHERE id = ?",
            delete,
            version_vec,
            process,
            id_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }
}
