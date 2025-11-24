use crate::gen_service_impl;
use async_trait::async_trait;
use dao::booking_log::BookingLogDao;
use dao::TransactionDao;
use service::booking_log::{BookingLog, BookingLogService};
use service::permission::{Authentication, PermissionService, SHIFTPLANNER_PRIVILEGE};
use service::ServiceError;
use std::sync::Arc;

gen_service_impl! {
    struct BookingLogServiceImpl: service::booking_log::BookingLogService = BookingLogServiceDeps {
        BookingLogDao: dao::booking_log::BookingLogDao<Transaction = Self::Transaction> = booking_log_dao,
        PermissionService: service::permission::PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: BookingLogServiceDeps> BookingLogService for BookingLogServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_booking_logs_for_week(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[BookingLog]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Check SHIFTPLANNER permission
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        // Fetch data from DAO
        let entities = self
            .booking_log_dao
            .get_booking_logs_for_week(year, calendar_week, tx.clone())
            .await?;

        // Convert DAO entities to service domain objects
        let ret = Ok(entities
            .iter()
            .map(|entity| BookingLog {
                year: entity.year,
                calendar_week: entity.calendar_week,
                day_of_week: entity.day_of_week,
                name: entity.name.clone(),
                time_from: entity.time_from,
                time_to: entity.time_to,
                created: entity.created,
                deleted: entity.deleted,
                created_by: entity.created_by.clone(),
                deleted_by: entity.deleted_by.clone(),
            })
            .collect::<Vec<_>>()
            .into());

        self.transaction_dao.commit(tx).await?;
        ret
    }
}
