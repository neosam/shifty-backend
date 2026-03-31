use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::gen_service_impl;
use dao::sales_person_shiftplan::SalesPersonShiftplanDao;
use dao::TransactionDao;
use service::{
    permission::{Authentication, PermissionService, SHIFTPLANNER_PRIVILEGE},
    sales_person::{SalesPerson, SalesPersonService},
    sales_person_shiftplan::SalesPersonShiftplanService,
    ServiceError,
};

const PROCESS: &str = "sales-person-shiftplan-service";

gen_service_impl! {
    struct SalesPersonShiftplanServiceImpl: service::sales_person_shiftplan::SalesPersonShiftplanService = SalesPersonShiftplanServiceDeps {
        SalesPersonShiftplanDao: dao::sales_person_shiftplan::SalesPersonShiftplanDao<Transaction = Self::Transaction> = sales_person_shiftplan_dao,
        SalesPersonService: service::sales_person::SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        PermissionService: service::permission::PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

#[async_trait]
impl<Deps: SalesPersonShiftplanServiceDeps> SalesPersonShiftplanService
    for SalesPersonShiftplanServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_shiftplans_for_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Vec<Uuid>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;
        let result = self
            .sales_person_shiftplan_dao
            .get_by_sales_person(sales_person_id, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn set_shiftplans_for_sales_person(
        &self,
        sales_person_id: Uuid,
        shiftplan_ids: &[Uuid],
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;
        if !self
            .sales_person_service
            .exists(sales_person_id, Authentication::Full, tx.clone().into())
            .await?
        {
            return Err(ServiceError::EntityNotFound(sales_person_id));
        }
        self.sales_person_shiftplan_dao
            .set_for_sales_person(sales_person_id, shiftplan_ids, PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn get_bookable_sales_persons(
        &self,
        shiftplan_id: Uuid,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPerson]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let all_persons = self
            .sales_person_service
            .get_all(Authentication::Full, tx.clone().into())
            .await?;

        let mut bookable = Vec::new();
        for person in all_persons.iter() {
            if person.inactive {
                continue;
            }
            let has_assignments = self
                .sales_person_shiftplan_dao
                .has_any_assignment(person.id, tx.clone())
                .await?;
            if !has_assignments {
                // Permissive: no assignments means eligible for all plans
                bookable.push(person.clone());
            } else {
                let is_assigned = self
                    .sales_person_shiftplan_dao
                    .is_assigned(person.id, shiftplan_id, tx.clone())
                    .await?;
                if is_assigned {
                    bookable.push(person.clone());
                }
            }
        }

        self.transaction_dao.commit(tx).await?;
        Ok(bookable.into())
    }

    async fn is_eligible(
        &self,
        sales_person_id: Uuid,
        shiftplan_id: Uuid,
        tx: Option<Self::Transaction>,
    ) -> Result<bool, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let has_assignments = self
            .sales_person_shiftplan_dao
            .has_any_assignment(sales_person_id, tx.clone())
            .await?;

        if !has_assignments {
            self.transaction_dao.commit(tx).await?;
            return Ok(true);
        }

        let is_assigned = self
            .sales_person_shiftplan_dao
            .is_assigned(sales_person_id, shiftplan_id, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(is_assigned)
    }
}
