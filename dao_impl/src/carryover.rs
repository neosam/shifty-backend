use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    carryover::{CarryoverDao, CarryoverEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

#[derive(Debug)]
struct CarryoverDb {
    sales_person_id: Vec<u8>,
    year: i64,
    carryover_hours: f64,
    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&CarryoverDb> for CarryoverEntity {
    type Error = DaoError;

    fn try_from(db: &CarryoverDb) -> Result<Self, Self::Error> {
        Ok(CarryoverEntity {
            sales_person_id: Uuid::from_slice(&db.sales_person_id)?,
            year: db.year as u32,
            carryover_hours: db.carryover_hours as f32,
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

pub struct CarryoverDaoImpl {
    pub pool: Arc<sqlx::SqlitePool>,
}

impl CarryoverDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CarryoverDao for CarryoverDaoImpl {
    async fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
    ) -> Result<Option<CarryoverEntity>, DaoError> {
        let sales_person_id_vec = sales_person_id.as_bytes().to_vec();
        Ok(query_as!(
            CarryoverDb,
            r#"SELECT sales_person_id, year, carryover_hours, created, deleted, update_version
               FROM employee_yearly_carryover
               WHERE sales_person_id = ? AND year = ? AND deleted IS NULL"#,
            sales_person_id_vec,
            year,
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_db_error()?
        .as_ref()
        .map(CarryoverEntity::try_from)
        .transpose()?)
    }

    async fn upsert(&self, entity: &CarryoverEntity, process: &str) -> Result<(), DaoError> {
        let sales_person_id_vec = entity.sales_person_id.as_bytes().to_vec();
        let created_str = entity.created.format(&Iso8601::DATE_TIME).map_db_error()?;
        let deleted_str = entity
            .deleted
            .map(|del| del.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let version_vec = entity.version.as_bytes().to_vec();

        query!(
            r#"INSERT INTO employee_yearly_carryover (sales_person_id, year, carryover_hours, created, deleted, update_process, update_version)
               VALUES (?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(sales_person_id, year) DO UPDATE SET carryover_hours=excluded.carryover_hours, deleted=excluded.deleted, update_process=excluded.update_process, update_version=excluded.update_version"#,
            sales_person_id_vec,
            entity.year,
            entity.carryover_hours,
            created_str,
            deleted_str,
            process,
            version_vec,
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;

        Ok(())
    }
}
