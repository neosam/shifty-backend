use std::sync::Arc;

use async_trait::async_trait;
use dao::DaoError;
use sqlx::{query, query_as, SqlitePool};
use uuid::Uuid;

use crate::ResultDbErrorExt;

struct ShiftplanIdRow {
    shiftplan_id: Vec<u8>,
}

struct SalesPersonIdRow {
    sales_person_id: Vec<u8>,
}

pub struct SalesPersonShiftplanDaoImpl {
    pub _pool: Arc<SqlitePool>,
}

impl SalesPersonShiftplanDaoImpl {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl dao::sales_person_shiftplan::SalesPersonShiftplanDao for SalesPersonShiftplanDaoImpl {
    type Transaction = crate::TransactionImpl;

    async fn get_by_sales_person(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Vec<Uuid>, DaoError> {
        let id_vec = sales_person_id.as_bytes().to_vec();
        let rows = query_as!(
            ShiftplanIdRow,
            r"SELECT shiftplan_id FROM sales_person_shiftplan WHERE sales_person_id = ?",
            id_vec
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        rows.iter()
            .map(|row| Uuid::from_slice(&row.shiftplan_id).map_err(DaoError::from))
            .collect()
    }

    async fn get_by_shiftplan(
        &self,
        shiftplan_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Vec<Uuid>, DaoError> {
        let id_vec = shiftplan_id.as_bytes().to_vec();
        let rows = query_as!(
            SalesPersonIdRow,
            r"SELECT sales_person_id FROM sales_person_shiftplan WHERE shiftplan_id = ?",
            id_vec
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        rows.iter()
            .map(|row| Uuid::from_slice(&row.sales_person_id).map_err(DaoError::from))
            .collect()
    }

    async fn set_for_sales_person(
        &self,
        sales_person_id: Uuid,
        shiftplan_ids: &[Uuid],
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let sp_id_vec = sales_person_id.as_bytes().to_vec();

        // Delete all existing assignments
        query!(
            r"DELETE FROM sales_person_shiftplan WHERE sales_person_id = ?",
            sp_id_vec
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        // Insert new assignments
        for shiftplan_id in shiftplan_ids {
            let plan_id_vec = shiftplan_id.as_bytes().to_vec();
            query!(
                r"INSERT INTO sales_person_shiftplan (sales_person_id, shiftplan_id, update_process) VALUES (?, ?, ?)",
                sp_id_vec,
                plan_id_vec,
                process
            )
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        }

        Ok(())
    }

    async fn has_any_assignment(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<bool, DaoError> {
        let id_vec = sales_person_id.as_bytes().to_vec();
        let result = query!(
            r"SELECT count(*) as cnt FROM sales_person_shiftplan WHERE sales_person_id = ?",
            id_vec
        )
        .fetch_one(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(result.cnt > 0)
    }

    async fn is_assigned(
        &self,
        sales_person_id: Uuid,
        shiftplan_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<bool, DaoError> {
        let sp_id_vec = sales_person_id.as_bytes().to_vec();
        let plan_id_vec = shiftplan_id.as_bytes().to_vec();
        let result = query!(
            r"SELECT count(*) as cnt FROM sales_person_shiftplan WHERE sales_person_id = ? AND shiftplan_id = ?",
            sp_id_vec,
            plan_id_vec
        )
        .fetch_one(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(result.cnt > 0)
    }
}
