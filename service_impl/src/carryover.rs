use crate::gen_service_impl;
use async_trait::async_trait;
use dao::{carryover::CarryoverDao, TransactionDao};
use service::{
    carryover::{Carryover, CarryoverService},
    permission::Authentication,
    ServiceError,
};
use uuid::Uuid;

// If you need any particular process name constant, define here:
const CARRYOVER_SERVICE_PROCESS: &str = "carryover-service";

gen_service_impl! {
    struct CarryoverServiceImpl: service::carryover::CarryoverService = CarryoverServiceDeps {
        CarryoverDao: dao::carryover::CarryoverDao<Transaction = Self::Transaction> = carryover_dao,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

// Implement the trait methods for CarryoverService:
#[async_trait]
impl<Deps: CarryoverServiceDeps> CarryoverService for CarryoverServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_carryover(
        &self,
        sales_person_id: Uuid,
        year: u32,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<Carryover>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self
            .carryover_dao
            .find_by_sales_person_id_and_year(sales_person_id, year, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result.map(|e| (&e).into()))
    }

    async fn set_carryover(
        &self,
        carryover: &Carryover,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entity = carryover.try_into()?;
        self.carryover_dao
            .upsert(&entity, CARRYOVER_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
