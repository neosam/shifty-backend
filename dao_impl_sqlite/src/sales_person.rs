use std::collections::HashMap;
use std::sync::Arc;

use crate::{ResultDbErrorExt, TransactionImpl};
use async_trait::async_trait;
use dao::{
    sales_person::{SalesPersonDao, SalesPersonEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

pub struct SalesPersonDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}
impl SalesPersonDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

struct SalesPersonDb {
    id: Vec<u8>,
    name: String,
    background_color: String,
    is_paid: bool,
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
            background_color: sales_person.background_color.as_str().into(),
            is_paid: sales_person.is_paid,
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
    type Transaction = TransactionImpl;

    async fn all(&self, tx: Self::Transaction) -> Result<Arc<[SalesPersonEntity]>, DaoError> {
        Ok(query_as!(
            SalesPersonDb,
            "SELECT id, name, background_color, is_paid, inactive, deleted, update_version FROM sales_person WHERE deleted IS NULL"
        )
            .fetch_all(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?
            .iter()
            .map(SalesPersonEntity::try_from)
            .collect::<Result<Arc<[SalesPersonEntity]>, DaoError>>()?
        )
    }

    async fn all_paid(&self, tx: Self::Transaction) -> Result<Arc<[SalesPersonEntity]>, DaoError> {
        Ok(query_as!(
            SalesPersonDb,
            "SELECT id, name, background_color, is_paid, inactive, deleted, update_version FROM sales_person WHERE deleted IS NULL AND is_paid = 1"
        )
            .fetch_all(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?
            .iter()
            .map(SalesPersonEntity::try_from)
            .collect::<Result<Arc<[SalesPersonEntity]>, DaoError>>()?
        )
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<SalesPersonEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            SalesPersonDb,
            "SELECT id, name, background_color, is_paid, inactive, deleted, update_version FROM sales_person WHERE id = ?",
            id_vec
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(SalesPersonEntity::try_from)
        .transpose()?)
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        tx: Self::Transaction,
    ) -> Result<Option<SalesPersonEntity>, DaoError> {
        Ok(query_as!(
            SalesPersonDb,
            "SELECT sp.id, sp.name, sp.background_color, sp.is_paid, sp.inactive, sp.deleted, sp.update_version FROM sales_person sp JOIN sales_person_user spu ON sp.id = spu.sales_person_id WHERE spu.user_id = ?",
            user_id
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(SalesPersonEntity::try_from)
        .transpose()?)
    }

    async fn create(
        &self,
        entity: &SalesPersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let name = entity.name.as_ref();
        let background_color = entity.background_color.as_ref();
        let is_paid = entity.is_paid;
        let inactive = entity.inactive;
        let deleted = entity.deleted.as_ref().map(|deleted| deleted.to_string());
        query!("INSERT INTO sales_person (id, name, background_color, is_paid, inactive, deleted, update_version, update_process) VALUES (?, ?, ?, ?, ?, ?, ?, ?)", id, name, background_color, is_paid, inactive, deleted, version, process)
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        Ok(())
    }
    async fn update(
        &self,
        entity: &SalesPersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let name = entity.name.as_ref();
        let background_color = entity.background_color.as_ref();
        let is_paid = entity.is_paid;
        let inactive = entity.inactive;
        let deleted = entity.deleted.as_ref().map(|deleted| deleted.to_string());
        query!("UPDATE sales_person SET name = ?, background_color = ?, is_paid = ?, inactive = ?, deleted = ?, update_version = ?, update_process = ? WHERE id = ?", name, background_color, is_paid, inactive, deleted, version, process, id)
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        Ok(())
    }

    async fn assign_to_user(
        &self,
        sales_person_id: Uuid,
        user_id: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let sales_person_id = sales_person_id.as_bytes().to_vec();
        query!("INSERT INTO sales_person_user (user_id, sales_person_id, update_process) VALUES (?, ?, ?)", user_id, sales_person_id, process)
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        Ok(())
    }

    async fn discard_assigned_user(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let sales_person_id = sales_person_id.as_bytes().to_vec();
        query!(
            "DELETE FROM sales_person_user WHERE sales_person_id = ?",
            sales_person_id
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn get_assigned_user(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<Arc<str>>, DaoError> {
        let sales_person_id = sales_person_id.as_bytes().to_vec();
        Ok(query!(
            "SELECT user_id FROM sales_person_user WHERE sales_person_id = ?",
            sales_person_id
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .map(|result| result.user_id.into()))
    }

    async fn get_all_user_assignments(
        &self,
        tx: Self::Transaction,
    ) -> Result<HashMap<Uuid, Arc<str>>, DaoError> {
        let rows = query!(
            "SELECT sales_person_id, user_id FROM sales_person_user"
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        rows.iter()
            .map(|row| {
                let sales_person_id = Uuid::from_slice(&row.sales_person_id)?;
                let user_id: Arc<str> = row.user_id.clone().into();
                Ok((sales_person_id, user_id))
            })
            .collect::<Result<HashMap<_, _>, DaoError>>()
    }

    async fn find_sales_person_by_user_id(
        &self,
        user_id: &str,
        tx: Self::Transaction,
    ) -> Result<Option<SalesPersonEntity>, DaoError> {
        Ok(query_as!(
            SalesPersonDb,
            "SELECT sp.id, sp.name, sp.background_color, sp.is_paid, sp.inactive, sp.deleted, sp.update_version FROM sales_person sp JOIN sales_person_user spu ON sp.id = spu.sales_person_id WHERE spu.user_id = ?",
            user_id
        )
            .fetch_optional(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?
            .as_ref()
            .map(SalesPersonEntity::try_from)
            .transpose()?
        )
    }
}
