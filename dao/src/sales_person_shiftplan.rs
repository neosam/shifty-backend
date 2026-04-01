use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::DaoError;

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait SalesPersonShiftplanDao {
    type Transaction: crate::Transaction;

    async fn get_by_sales_person(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Vec<(Uuid, String)>, DaoError>;

    async fn get_by_shiftplan(
        &self,
        shiftplan_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Vec<Uuid>, DaoError>;

    async fn set_for_sales_person(
        &self,
        sales_person_id: Uuid,
        assignments: &[(Uuid, String)],
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn has_any_assignment(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<bool, DaoError>;

    async fn is_assigned(
        &self,
        sales_person_id: Uuid,
        shiftplan_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<bool, DaoError>;

    async fn get_permission_level(
        &self,
        sales_person_id: Uuid,
        shiftplan_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<String>, DaoError>;
}
