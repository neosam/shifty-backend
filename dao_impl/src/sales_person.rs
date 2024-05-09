use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    sales_person::{SalesPersonDao, SalesPersonEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

pub struct SalesPersonDaoImpl {
    pub pool: Arc<sqlx::SqlitePool>,
}
impl SalesPersonDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

struct SalesPersonDb {
    id: Vec<u8>,
    name: String,
    inactive: bool,
    deleted: Option<String>,
    update_version: Vec<u8>,
}
impl TryFrom<&SalesPersonDb> for SalesPersonEntity {
    type Error = DaoError;
    fn try_from(sales_person: &SalesPersonDb) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from_slice(sales_person.id.as_ref()).unwrap(),
            name: sales_person.name.as_str().into(),
            inactive: sales_person.inactive,
            deleted: sales_person
                .deleted
                .as_ref()
                .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&sales_person.update_version).unwrap(),
        })
    }
}

#[async_trait]
impl SalesPersonDao for SalesPersonDaoImpl {
    async fn all(&self) -> Result<Arc<[SalesPersonEntity]>, DaoError> {
        Ok(query_as!(
            SalesPersonDb,
            "SELECT id, name, inactive, deleted, update_version FROM sales_person WHERE deleted IS NULL"
        )
            .fetch_all(self.pool.as_ref())
            .await
            .map_db_error()?
            .iter()
            .map(SalesPersonEntity::try_from)
            .collect::<Result<Arc<[SalesPersonEntity]>, DaoError>>()?
        )
    }
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SalesPersonEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            SalesPersonDb,
            "SELECT id, name, inactive, deleted, update_version FROM sales_person WHERE id = ?",
            id_vec
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_db_error()?
        .as_ref()
        .map(SalesPersonEntity::try_from)
        .transpose()?)
    }
    async fn create(&self, entity: &SalesPersonEntity, process: &str) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let name = entity.name.as_ref();
        let inactive = entity.inactive;
        let deleted = entity.deleted.as_ref().map(|deleted| deleted.to_string());
        query!("INSERT INTO sales_person (id, name, inactive, deleted, update_version, update_process) VALUES (?, ?, ?, ?, ?, ?)", id, name, inactive, deleted, version, process)
            .execute(self.pool.as_ref())
            .await
            .map_db_error()?;
        Ok(())
    }
    async fn update(&self, entity: &SalesPersonEntity, process: &str) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let name = entity.name.as_ref();
        let inactive = entity.inactive;
        let deleted = entity.deleted.as_ref().map(|deleted| deleted.to_string());
        query!("UPDATE sales_person SET name = ?, inactive = ?, deleted = ?, update_version = ?, update_process = ? WHERE id = ?", name, inactive, deleted, version, process, id)
            .execute(self.pool.as_ref())
            .await
            .map_db_error()?;
        Ok(())
    }
}
