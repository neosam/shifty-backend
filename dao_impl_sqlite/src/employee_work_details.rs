use std::sync::Arc;

use crate::{ResultDbErrorExt, TransactionImpl};
use async_trait::async_trait;
use dao::{
    employee_work_details::{EmployeeWorkDetailsDao, EmployeeWorkDetailsEntity},
    DaoError,
};
use shifty_utils::DayOfWeek;
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

pub struct EmployeeWorkDetailsDb {
    pub id: Vec<u8>,
    pub sales_person_id: Vec<u8>,
    pub expected_hours: f64,
    pub from_day_of_week: i64,
    pub from_calendar_week: i64,
    pub from_year: i64,
    pub to_day_of_week: i64,
    pub to_calendar_week: i64,
    pub to_year: i64,
    pub workdays_per_week: i64,
    pub is_dynamic: i64,

    pub monday: i64,
    pub tuesday: i64,
    pub wednesday: i64,
    pub thursday: i64,
    pub friday: i64,
    pub saturday: i64,
    pub sunday: i64,

    pub vacation_days: i64,

    pub created: String,
    pub deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&EmployeeWorkDetailsDb> for EmployeeWorkDetailsEntity {
    type Error = DaoError;

    fn try_from(working_hours: &EmployeeWorkDetailsDb) -> Result<Self, DaoError> {
        Ok(Self {
            id: Uuid::from_slice(working_hours.id.as_ref())?,
            sales_person_id: Uuid::from_slice(working_hours.sales_person_id.as_ref()).unwrap(),
            expected_hours: working_hours.expected_hours as f32,
            from_day_of_week: DayOfWeek::from_number(working_hours.from_day_of_week as u8).ok_or(
                DaoError::InvalidDayOfWeek(working_hours.from_day_of_week as u8),
            )?,
            from_calendar_week: working_hours.from_calendar_week as u8,
            from_year: working_hours.from_year as u32,
            to_day_of_week: DayOfWeek::from_number(working_hours.to_day_of_week as u8).ok_or(
                DaoError::InvalidDayOfWeek(working_hours.to_day_of_week as u8),
            )?,
            to_calendar_week: working_hours.to_calendar_week as u8,
            to_year: working_hours.to_year as u32,
            workdays_per_week: working_hours.workdays_per_week as u8,
            is_dynamic: working_hours.is_dynamic != 0,

            monday: working_hours.monday != 0,
            tuesday: working_hours.tuesday != 0,
            wednesday: working_hours.wednesday != 0,
            thursday: working_hours.thursday != 0,
            friday: working_hours.friday != 0,
            saturday: working_hours.saturday != 0,
            sunday: working_hours.sunday != 0,

            vacation_days: working_hours.vacation_days as u8,

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

pub struct EmployeeWorkDetailsDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl EmployeeWorkDetailsDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl EmployeeWorkDetailsDao for EmployeeWorkDetailsDaoImpl {
    type Transaction = TransactionImpl;

    async fn all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[EmployeeWorkDetailsEntity]>, DaoError> {
        query_as!(
            EmployeeWorkDetailsDb,
            r#"
            SELECT
                id,
                sales_person_id,
                expected_hours,
                from_day_of_week,
                from_calendar_week,
                from_year,
                to_day_of_week,
                to_calendar_week,
                to_year,
                workdays_per_week,
                is_dynamic,
                
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
                sunday,

                vacation_days,

                created,
                deleted,
                update_version
            FROM
                employee_work_details
            WHERE
                deleted IS NULL
            "#
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(EmployeeWorkDetailsEntity::try_from)
        .collect::<Result<_, _>>()
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<EmployeeWorkDetailsEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        query_as!(
            EmployeeWorkDetailsDb,
            r#"
            SELECT
                id,
                sales_person_id,
                expected_hours,
                from_day_of_week,
                from_calendar_week,
                from_year,
                to_day_of_week,
                to_calendar_week,
                to_year,
                workdays_per_week,
                is_dynamic,
                
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
                sunday,

                vacation_days,

                created,
                deleted,
                update_version
            FROM
                employee_work_details
            WHERE
                id = ?
                and deleted IS NULL
            "#,
            id_vec
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(EmployeeWorkDetailsEntity::try_from)
        .transpose()
    }

    async fn find_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[EmployeeWorkDetailsEntity]>, DaoError> {
        let id_vec = sales_person_id.as_bytes().to_vec();
        query_as!(
            EmployeeWorkDetailsDb,
            r#"
            SELECT
                id,
                sales_person_id,
                expected_hours,
                from_day_of_week,
                from_calendar_week,
                from_year,
                to_day_of_week,
                to_calendar_week,
                to_year,
                workdays_per_week,
                is_dynamic,
                
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
                sunday,

                vacation_days,



                created,
                deleted,
                update_version
            FROM
                employee_work_details
            WHERE
                sales_person_id = ?
                and deleted IS NULL
            "#,
            id_vec
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(EmployeeWorkDetailsEntity::try_from)
        .collect::<Result<_, _>>()
    }

    async fn find_for_week(
        &self,
        calenar_week: u8,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<Arc<[EmployeeWorkDetailsEntity]>, DaoError> {
        query_as!(
            EmployeeWorkDetailsDb,
            r#"
            SELECT
                id,
                sales_person_id,
                expected_hours,
                from_day_of_week,
                from_calendar_week,
                from_year,
                to_day_of_week,
                to_calendar_week,
                to_year,
                workdays_per_week,
                is_dynamic,
                
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
                sunday,

                vacation_days,

                created,
                deleted,
                update_version
            FROM
                employee_work_details
            WHERE
                deleted IS NULL
                AND
                (from_year * 100 + from_calendar_week) <= (? * 100 + ?)
                AND (to_year * 100 + to_calendar_week) >= (? * 100 + ?)
            "#,
            year,
            calenar_week,
            year,
            calenar_week,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(EmployeeWorkDetailsEntity::try_from)
        .collect::<Result<_, _>>()
    }

    async fn create(
        &self,
        entity: &EmployeeWorkDetailsEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let sales_person_id = entity.sales_person_id.as_bytes().to_vec();
        let expected_hours = entity.expected_hours as f64;
        let from_day_of_week = entity.from_day_of_week.to_number() as i64;
        let from_calendar_week = entity.from_calendar_week as i64;
        let from_year = entity.from_year as i64;
        let to_day_of_week = entity.to_day_of_week.to_number() as i64;
        let to_calendar_week = entity.to_calendar_week as i64;
        let to_year = entity.to_year as i64;
        let is_dynamic = entity.is_dynamic as i64;
        let monday = entity.monday as i64;
        let tuesday = entity.tuesday as i64;
        let wednesday = entity.wednesday as i64;
        let thursday = entity.thursday as i64;
        let friday = entity.friday as i64;
        let saturday = entity.saturday as i64;
        let sunday = entity.sunday as i64;
        let vacation_days = entity.vacation_days as i64;
        let workdays_per_week = entity.workdays_per_week as i64;
        let created = entity.created.format(&Iso8601::DATE_TIME)?;
        let version = entity.id.as_bytes().to_vec();
        query!(
            r#"
            INSERT INTO employee_work_details (
                id,
                sales_person_id,
                expected_hours,
                from_day_of_week,
                from_calendar_week,
                from_year,
                to_day_of_week,
                to_calendar_week,
                to_year,
                workdays_per_week,
                is_dynamic,
                
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
                sunday,

                vacation_days,

                created,
                update_process,
                update_version
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? , ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            sales_person_id,
            expected_hours,
            from_day_of_week,
            from_calendar_week,
            from_year,
            to_day_of_week,
            to_calendar_week,
            to_year,
            workdays_per_week,
            is_dynamic,
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            sunday,
            vacation_days,
            created,
            process,
            version,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn update(
        &self,
        entity: &EmployeeWorkDetailsEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let deleted = entity
            .deleted
            .as_ref()
            .map(|deleted| deleted.format(&Iso8601::DATE_TIME))
            .transpose()
            .map_db_error()?;
        let to_year = entity.to_year as i64;
        let to_calendar_week = entity.to_calendar_week as i64;
        let to_day_of_week = entity.to_day_of_week.to_number() as i64;
        let expected_hours = entity.expected_hours as i64;
        let vacation_days = entity.vacation_days as i64;
        let workdays_per_week = entity.workdays_per_week as i64;
        let is_dynamic = entity.is_dynamic as i64;
        query!(
            r#"
            UPDATE employee_work_details SET
                deleted = ?,
                update_process = ?,
                to_year = ?,
                to_calendar_week = ?,
                to_day_of_week = ?,
                expected_hours = ?,
                vacation_days = ?,
                workdays_per_week = ?,
                is_dynamic = ?
            WHERE
                id = ?
            "#,
            deleted,
            process,
            to_year,
            to_calendar_week,
            to_day_of_week,
            expected_hours,
            vacation_days,
            workdays_per_week,
            is_dynamic,
            id
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }
}
