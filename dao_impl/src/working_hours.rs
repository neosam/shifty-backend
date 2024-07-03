use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    working_hours::{WorkingHoursDao, WorkingHoursEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

pub struct WorkingHoursDb {
    pub id: Vec<u8>,
    pub sales_person_id: Vec<u8>,
    pub expected_hours: f64,
    pub from_calendar_week: i64,
    pub from_year: i64,
    pub to_calendar_week: i64,
    pub to_year: i64,
    pub workdays_per_week: i64,
    pub days_per_week: i64,
    pub created: String,
    pub deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&WorkingHoursDb> for WorkingHoursEntity {
    type Error = DaoError;

    fn try_from(working_hours: &WorkingHoursDb) -> Result<Self, DaoError> {
        Ok(Self {
            id: Uuid::from_slice(working_hours.id.as_ref())?,
            sales_person_id: Uuid::from_slice(working_hours.sales_person_id.as_ref()).unwrap(),
            expected_hours: working_hours.expected_hours as f32,
            from_calendar_week: working_hours.from_calendar_week as u8,
            from_year: working_hours.from_year as u32,
            to_calendar_week: working_hours.to_calendar_week as u8,
            to_year: working_hours.to_year as u32,
            workdays_per_week: working_hours.workdays_per_week as u8,
            days_per_week: working_hours.days_per_week as u8,
            created: PrimitiveDateTime::parse(working_hours.created.as_str(), &Iso8601::DATE_TIME)?,
            deleted: working_hours
                .deleted
                .as_ref()
                .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&working_hours.update_version)?,
        })
    }
}

pub struct WorkingHoursDaoImpl {
    pub pool: Arc<sqlx::SqlitePool>,
}

impl WorkingHoursDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkingHoursDao for WorkingHoursDaoImpl {
    async fn all(&self) -> Result<Arc<[WorkingHoursEntity]>, DaoError> {
        query_as!(
            WorkingHoursDb,
            r#"
            SELECT
                id,
                sales_person_id,
                expected_hours,
                from_calendar_week,
                from_year,
                to_calendar_week,
                to_year,
                workdays_per_week,
                days_per_week,
                created,
                deleted,
                update_version
            FROM
                working_hours
            "#
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_db_error()?
        .iter()
        .map(WorkingHoursEntity::try_from)
        .collect::<Result<_, _>>()
    }

    async fn find_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
    ) -> Result<Arc<[WorkingHoursEntity]>, DaoError> {
        let id_vec = sales_person_id.as_bytes().to_vec();
        query_as!(
            WorkingHoursDb,
            r#"
            SELECT
                id,
                sales_person_id,
                expected_hours,
                from_calendar_week,
                from_year,
                to_calendar_week,
                to_year,
                workdays_per_week,
                days_per_week,
                created,
                deleted,
                update_version
            FROM
                working_hours
            WHERE
                sales_person_id = ?
            "#,
            id_vec
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_db_error()?
        .iter()
        .map(WorkingHoursEntity::try_from)
        .collect::<Result<_, _>>()
    }
    async fn create(&self, entity: &WorkingHoursEntity, process: &str) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let sales_person_id = entity.sales_person_id.as_bytes().to_vec();
        let expected_hours = entity.expected_hours as f64;
        let from_calendar_week = entity.from_calendar_week as i64;
        let from_year = entity.from_year as i64;
        let to_calendar_week = entity.to_calendar_week as i64;
        let to_year = entity.to_year as i64;
        let workdays_per_week = entity.workdays_per_week as i64;
        let created = entity.created.format(&Iso8601::DATE_TIME)?;
        let version = entity.id.as_bytes().to_vec();
        query!(
            r#"
            INSERT INTO working_hours (
                id,
                sales_person_id,
                expected_hours,
                from_calendar_week,
                from_year,
                to_calendar_week,
                to_year,
                workdays_per_week,
                created,
                update_process,
                update_version
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            sales_person_id,
            expected_hours,
            from_calendar_week,
            from_year,
            to_calendar_week,
            to_year,
            workdays_per_week,
            created,
            process,
            version,
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn update(&self, entity: &WorkingHoursEntity, process: &str) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let deleted = entity.deleted.as_ref().map(|deleted| deleted.to_string());
        query!(
            r#"
            UPDATE working_hours SET
                deleted = ?,
                update_process = ?
            WHERE
                id = ?
            "#,
            deleted,
            process,
            id
        )
        .execute(self.pool.as_ref())
        .await
        .map_db_error()?;
        Ok(())
    }
}
