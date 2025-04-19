use std::sync::Arc;

use crate::ResultDbErrorExt;
use async_trait::async_trait;
use dao::{
    custom_extra_hours::{CustomExtraHoursDao, CustomExtraHoursEntity},
    DaoError,
};
use sqlx::{query, query_as, QueryBuilder, Sqlite};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use tracing::info;
use uuid::Uuid;

#[derive(Debug)]
struct CustomExtraHoursDb {
    id: Vec<u8>,
    name: String,
    description: Option<String>,
    modifies_balance: i64,

    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

#[derive(Debug)]
struct CustomExtraHoursSalesPersonMappingDb {
    custom_extra_hours_id: Vec<u8>,
    sales_person_id: Vec<u8>,
}

fn combine(
    custom_hours: Arc<[CustomExtraHoursDb]>,
    sales_person_mapping: Arc<[CustomExtraHoursSalesPersonMappingDb]>,
) -> Arc<[CustomExtraHoursEntity]> {
    let mut result = Vec::new();

    for custom_hours in custom_hours.iter() {
        let id = Uuid::from_slice(&custom_hours.id).unwrap();
        let name = Arc::from(custom_hours.name.as_str());
        let description = custom_hours
            .description
            .as_ref()
            .map(|d| Arc::from(d.as_str()));
        let modifies_balance = custom_hours.modifies_balance != 0;

        let created = PrimitiveDateTime::parse(&custom_hours.created, &Iso8601::DEFAULT).unwrap();

        let deleted = match &custom_hours.deleted {
            Some(deleted) => Some(PrimitiveDateTime::parse(deleted, &Iso8601::DEFAULT).unwrap()),
            None => None,
        };

        let version = Uuid::from_slice(&custom_hours.update_version).unwrap();

        let assigned_sales_person_ids: Vec<Uuid> = sales_person_mapping
            .iter()
            .filter(|mapping| mapping.custom_extra_hours_id == custom_hours.id)
            .map(|mapping| Uuid::from_slice(&mapping.sales_person_id).unwrap())
            .collect();

        result.push(CustomExtraHoursEntity {
            id,
            name,
            description,
            modifies_balance,
            assigned_sales_person_ids: Arc::from(assigned_sales_person_ids),
            created,
            deleted,
            version,
        });
    }

    Arc::from(result)
}

pub struct CustomExtraHoursDaoImpl;

#[async_trait]
impl CustomExtraHoursDao for CustomExtraHoursDaoImpl {
    type Transaction = crate::TransactionImpl;

    /// Returns everything, including deleted items.
    async fn dump(&self, tx: Self::Transaction) -> Result<Arc<[CustomExtraHoursEntity]>, DaoError> {
        info!("Dump all data from custom_extra_hours");
        let custom_hours = query_as!(
            CustomExtraHoursDb,
            r#"
            SELECT id, name, description, modifies_balance, created, deleted, update_version
            FROM custom_extra_hours
            "#
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        let sales_person_mapping = query_as!(
            CustomExtraHoursSalesPersonMappingDb,
            r#"
            SELECT custom_extra_hours_id, sales_person_id
            FROM custom_extra_hours_sales_person
            "#
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(combine(
            Arc::from(custom_hours),
            Arc::from(sales_person_mapping),
        ))
    }

    /// Create a new entity
    async fn create(
        &self,
        entity: &CustomExtraHoursEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let name = entity.name.as_ref();
        let description = entity.description.as_ref().map(|d| d.as_ref());
        let modifies_balance = if entity.modifies_balance { 1 } else { 0 };
        let created_str = entity.created.format(&Iso8601::DEFAULT).map_db_error()?;
        let version = entity.version.as_bytes().to_vec();

        info!("Running query to create custom extra hours");

        query!(
            r#"
            INSERT INTO custom_extra_hours (id, name, description, modifies_balance, created, deleted, update_version, update_process)
            VALUES (?, ?, ?, ?, ?, NULL, ?, ?)
            "#,
            id,
            name,
            description,
            modifies_balance,
            created_str,
            version,
            process
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        info!("Assign sales_person_ids to custom extra hours");

        for sales_person_id in entity.assigned_sales_person_ids.iter() {
            let sales_person_id_vec = sales_person_id.as_bytes().to_vec();
            query!(
                r#"
                INSERT INTO custom_extra_hours_sales_person (custom_extra_hours_id, sales_person_id, created, update_process)
                VALUES (?, ?, ?, ?)
                "#,
                id,
                sales_person_id_vec,
                created_str,
                process
            )
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        }

        Ok(())
    }

    /// Upates an entity
    async fn update(
        &self,
        entity: CustomExtraHoursEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let name = entity.name.as_ref();
        let description = entity.description.as_ref().map(|d| d.as_ref());
        let modifies_balance = if entity.modifies_balance { 1 } else { 0 };
        let created_str = entity.created.format(&Iso8601::DEFAULT).map_db_error()?;
        let version = entity.version.as_bytes().to_vec();

        query!(
            r#"
            UPDATE custom_extra_hours
            SET name = ?, description = ?, modifies_balance = ?, created = ?, deleted = NULL, update_version = ?, update_process = ?
            WHERE id = ?
            "#,
            name,
            description,
            modifies_balance,
            created_str,
            version,
            process,
            id
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        // Insert all sales_person_ids
        for sales_person_id in entity.assigned_sales_person_ids.iter() {
            let sales_person_id_vec = sales_person_id.as_bytes().to_vec();
            query!(
                r#"
                INSERT OR IGNORE INTO custom_extra_hours_sales_person (custom_extra_hours_id, sales_person_id, created, update_process)
                VALUES (?, ?, ?, ?)
                "#,
                id,
                sales_person_id_vec,
                created_str,
                process
            )
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        }

        // Remove all sales_person_ids that are not in the new list
        let mut remove_query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("DELETE FROM custom_extra_hours_sales_person WHERE custom_extra_hours_id = ? AND sales_person_id NOT IN (");
        let mut separated = remove_query_builder.separated(",");
        for sales_person_id in entity.assigned_sales_person_ids.iter() {
            let sales_person_id_vec = sales_person_id.as_bytes().to_vec();
            separated.push_bind(sales_person_id_vec);
        }
        separated.push_unseparated(")");

        let query = remove_query_builder.push_bind(id).build();
        query
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;

        Ok(())
    }
}
