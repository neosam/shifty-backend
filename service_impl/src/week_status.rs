use crate::gen_service_impl;
use async_trait::async_trait;
use dao::{week_status::WeekStatusDao, TransactionDao};
use service::{
    clock::ClockService,
    permission::Authentication,
    uuid_service::UuidService,
    week_status::{WeekStatus, WeekStatusService},
    PermissionService, ServiceError,
};

#[allow(dead_code)]
const WEEK_STATUS_SERVICE_PROCESS: &str = "week-status-service";

gen_service_impl! {
    struct WeekStatusServiceImpl: WeekStatusService = WeekStatusServiceDeps {
        WeekStatusDao: WeekStatusDao<Transaction = Self::Transaction> = week_status_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: WeekStatusServiceDeps> WeekStatusService for WeekStatusServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_week_status(
        &self,
        _year: u32,
        _calendar_week: u8,
        _context: Authentication<Self::Context>,
        _tx: Option<Self::Transaction>,
    ) -> Result<WeekStatus, ServiceError> {
        todo!("implemented in GREEN phase (Task 2)")
    }

    async fn set_week_status(
        &self,
        _year: u32,
        _calendar_week: u8,
        _status: WeekStatus,
        _context: Authentication<Self::Context>,
        _tx: Option<Self::Transaction>,
    ) -> Result<WeekStatus, ServiceError> {
        todo!("implemented in GREEN phase (Task 2)")
    }
}
