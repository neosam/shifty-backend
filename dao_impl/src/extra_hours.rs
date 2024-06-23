use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    extra_hours::{ExtraHoursCategoryEntity, ExtraHoursDao, ExtraHoursEntity},
    DaoError,
};
use sqlx::query_as;
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

struct ExtraHoursDb {
    id: Vec<u8>,
    sales_person_id: Vec<u8>,
    amount: f64,

    category: String,
    description: Option<String>,
    date_time: String,
    deleted: Option<String>,
}
impl TryFrom<&ExtraHoursDb> for ExtraHoursEntity {
    type Error = DaoError;

    fn try_from(extra_hours: &ExtraHoursDb) -> Result<Self, DaoError> {
        Ok(Self {
            id: Uuid::from_slice(extra_hours.id.as_ref()).unwrap(),
            sales_person_id: Uuid::from_slice(extra_hours.sales_person_id.as_ref()).unwrap(),
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
            )
            .unwrap(),
            deleted: extra_hours
                .deleted
                .as_ref()
                .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
                .transpose()
                .unwrap(),
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
    async fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        until_week: u8,
    ) -> Result<Arc<[ExtraHoursEntity]>, crate::DaoError> {
        let id_vec = sales_person_id.as_bytes().to_vec();
        Ok(query_as!(
            ExtraHoursDb,
            "SELECT id, sales_person_id, amount, category, description, date_time, deleted FROM extra_hours WHERE sales_person_id = ? AND strftime('%Y', date_time) = ? AND strftime('%m', date_time) <= ?",
            id_vec,
            year,
            until_week,
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
        _entity: &ExtraHoursEntity,
        _process: &str,
    ) -> Result<(), crate::DaoError> {
        unimplemented!()
    }
    async fn update(
        &self,
        _entity: &ExtraHoursEntity,
        _process: &str,
    ) -> Result<(), crate::DaoError> {
        unimplemented!()
    }
    async fn delete(&self, _id: Uuid, _process: &str) -> Result<(), crate::DaoError> {
        unimplemented!()
    }
}
