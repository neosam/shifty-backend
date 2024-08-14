use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    extra_hours::{ExtraHoursCategoryEntity, ExtraHoursDao, ExtraHoursEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

struct ExtraHoursDb {
    id: Vec<u8>,
    sales_person_id: Vec<u8>,
    amount: f64,

    category: String,
    description: Option<String>,
    date_time: String,
    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}
impl TryFrom<&ExtraHoursDb> for ExtraHoursEntity {
    type Error = DaoError;

    fn try_from(extra_hours: &ExtraHoursDb) -> Result<Self, DaoError> {
        Ok(Self {
            id: Uuid::from_slice(extra_hours.id.as_ref())?,
            sales_person_id: Uuid::from_slice(extra_hours.sales_person_id.as_ref())?,
            amount: extra_hours.amount as f32,
            category: match extra_hours.category.as_str() {
                "ExtraWork" => ExtraHoursCategoryEntity::ExtraWork,
                "Vacation" => ExtraHoursCategoryEntity::Vacation,
                "SickLeave" => ExtraHoursCategoryEntity::SickLeave,
                "Holiday" => ExtraHoursCategoryEntity::Holiday,
                value @ _ => return Err(DaoError::EnumValueNotFound(value.into())),
            },
            description: extra_hours
                .description
                .clone()
                .unwrap_or_else(|| String::new())
                .as_str()
                .into(),
            date_time: PrimitiveDateTime::parse(
                extra_hours.date_time.as_str(),
                &Iso8601::DATE_TIME,
            )?,
            created: PrimitiveDateTime::parse(extra_hours.created.as_str(), &Iso8601::DATE_TIME)?,
            deleted: extra_hours
                .deleted
                .as_ref()
                .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&extra_hours.update_version)?,
        })
    }
}

pub struct ExtraHoursDaoImpl {
    pub pool: Arc<sqlx::SqlitePool>,
}
impl ExtraHoursDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExtraHoursDao for ExtraHoursDaoImpl {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ExtraHoursEntity>, crate::DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            ExtraHoursDb,
            "SELECT id, sales_person_id, amount, category, description, date_time, created, deleted, update_version FROM extra_hours WHERE id = ? AND deleted IS NULL",
            id_vec,
        ).fetch_optional(self.pool.as_ref())
            .await
            .map_db_error()?
            .as_ref()
            .map(ExtraHoursEntity::try_from)
            .transpose()?)
    }

    async fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
    ) -> Result<Arc<[ExtraHoursEntity]>, crate::DaoError> {
        let id_vec = sales_person_id.as_bytes().to_vec();
        Ok(query_as!(
            ExtraHoursDb,
            "SELECT id, sales_person_id, amount, category, description, date_time, created, deleted, update_version FROM extra_hours WHERE sales_person_id = ? AND CAST(strftime('%Y', date_time) AS INTEGER) = ? AND deleted IS NULL",
            id_vec,
            year,
        ).fetch_all(self.pool.as_ref())
            .await
            .map_db_error()?
            .iter()
            .map(ExtraHoursEntity::try_from)
            .collect::<Result<Arc<[_]>, _>>()?
            .into())
    }

    async fn find_by_week(
        &self,
        calendar_week: u8,
        year: u32,
    ) -> Result<Arc<[ExtraHoursEntity]>, crate::DaoError> {
        let monday = time::PrimitiveDateTime::new(
            time::Date::from_iso_week_date(year as i32, calendar_week, time::Weekday::Monday)?,
            time::Time::MIDNIGHT,
        );
        let next_monday = monday + time::Duration::weeks(1);
        let monday_str = monday.format(&Iso8601::DATE_TIME)?;
        let next_monday_str = next_monday.format(&Iso8601::DATE_TIME)?;
        Ok(query_as!(
            ExtraHoursDb,
            "SELECT id, sales_person_id, amount, category, description, date_time, created, deleted, update_version FROM extra_hours WHERE date_time >= ? and date_time < ? AND deleted IS NULL",
            monday_str,
            next_monday_str,
        ).fetch_all(self.pool.as_ref())
            .await
            .map_db_error()?
            .iter()
            .map(ExtraHoursEntity::try_from)
            .collect::<Result<Arc<[_]>, _>>()?
            .into())
    }

    async fn create(
        &self,
        entity: &ExtraHoursEntity,
        process: &str,
    ) -> Result<(), crate::DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let sales_person_id_vec = entity.sales_person_id.as_bytes().to_vec();
        let category = match entity.category {
            ExtraHoursCategoryEntity::ExtraWork => "ExtraWork",
            ExtraHoursCategoryEntity::Vacation => "Vacation",
            ExtraHoursCategoryEntity::SickLeave => "SickLeave",
            ExtraHoursCategoryEntity::Holiday => "Holiday",
        };
        let description = entity.description.as_ref();
        let date_time = entity.date_time.format(&Iso8601::DATE_TIME)?;
        let created = entity.created.format(&Iso8601::DATE_TIME)?;
        let deleted = entity
            .deleted
            .map(|deleted| deleted.format(&Iso8601::DATE_TIME))
            .transpose()?;
        let version_vec = entity.version.as_bytes().to_vec();
        query!(
            "INSERT INTO extra_hours (id, sales_person_id, amount, category, description, date_time, created, deleted, update_process, update_version) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id_vec,
            sales_person_id_vec,
            entity.amount,
            category,
            description,
            date_time,
            created,
            deleted,
            process,
            version_vec,
        ).execute(self.pool.as_ref())
            .await
            .map_db_error()?;
        Ok(())
    }
    async fn update(
        &self,
        entity: &ExtraHoursEntity,
        process: &str,
    ) -> Result<(), crate::DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let version_vec = entity.version.as_bytes().to_vec();
        let delete = entity
            .deleted
            .map(|date_time| date_time.format(&Iso8601::DATE_TIME))
            .transpose()?;
        query!(
            "UPDATE extra_hours SET deleted = ?, update_version = ?, update_process = ? WHERE id = ?",
            delete,
            version_vec,
            process,
            id_vec,
        )
            .execute(self.pool.as_ref())
            .await
            .map_db_error()?;
        Ok(())
    }
    async fn delete(&self, _id: Uuid, _process: &str) -> Result<(), crate::DaoError> {
        unimplemented!()
    }
}
