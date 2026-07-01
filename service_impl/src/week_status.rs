use crate::gen_service_impl;
use async_trait::async_trait;
use dao::{
    week_status::{WeekStatusDao, WeekStatusEntity, WeekStatusKind},
    TransactionDao,
};
use service::{
    clock::ClockService,
    permission::{Authentication, SHIFTPLANNER_PRIVILEGE},
    uuid_service::UuidService,
    week_status::{WeekStatus, WeekStatusService},
    PermissionService, ServiceError,
};

const WEEK_STATUS_SERVICE_PROCESS: &str = "week-status-service";

/// Map a persistable `WeekStatus` to its DAO discriminant. `Unset` never reaches
/// this function (it is handled by the soft-delete branch) — returns `None` so the
/// caller can treat it as a structural impossibility.
fn to_kind(status: &WeekStatus) -> Option<WeekStatusKind> {
    match status {
        WeekStatus::Unset => None,
        WeekStatus::InPlanning => Some(WeekStatusKind::InPlanning),
        WeekStatus::Planned => Some(WeekStatusKind::Planned),
        WeekStatus::Locked => Some(WeekStatusKind::Locked),
    }
}

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
        year: u32,
        calendar_week: u8,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeekStatus, ServiceError> {
        // No permission gate: status is not sensitive, all roles may read (T-39-03).
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let existing = self
            .week_status_dao
            .find_by_year_and_week(year, calendar_week, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;

        Ok(existing
            .map(|e| e.status.into())
            .unwrap_or(WeekStatus::Unset))
    }

    async fn set_week_status(
        &self,
        year: u32,
        calendar_week: u8,
        status: WeekStatus,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeekStatus, ServiceError> {
        // Permission gate FIRST — before any DAO access (D-39-01, T-39-01).
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        // find + write in the SAME transaction (no TOCTOU, T-39-04).
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let existing = self
            .week_status_dao
            .find_by_year_and_week(year, calendar_week, tx.clone())
            .await?;

        match to_kind(&status) {
            // Unset == row absence (D-39-04): soft-delete the active row, else no-op.
            None => {
                if let Some(existing) = existing {
                    self.week_status_dao
                        .delete(existing.id, WEEK_STATUS_SERVICE_PROCESS, tx.clone())
                        .await?;
                }
            }
            // Free transitions (D-39-02): upsert without transition validation.
            Some(kind) => match existing {
                Some(existing) => {
                    let entity = WeekStatusEntity {
                        status: kind,
                        version: self.uuid_service.new_uuid(&format!(
                            "{WEEK_STATUS_SERVICE_PROCESS}::update version"
                        )),
                        ..existing
                    };
                    self.week_status_dao
                        .update(&entity, WEEK_STATUS_SERVICE_PROCESS, tx.clone())
                        .await?;
                }
                None => {
                    let entity = WeekStatusEntity {
                        id: self
                            .uuid_service
                            .new_uuid(&format!("{WEEK_STATUS_SERVICE_PROCESS}::create id")),
                        year,
                        calendar_week,
                        status: kind,
                        created: self.clock_service.date_time_now(),
                        deleted: None,
                        version: self
                            .uuid_service
                            .new_uuid(&format!("{WEEK_STATUS_SERVICE_PROCESS}::create version")),
                    };
                    self.week_status_dao
                        .create(&entity, WEEK_STATUS_SERVICE_PROCESS, tx.clone())
                        .await?;
                }
            },
        }

        self.transaction_dao.commit(tx).await?;
        Ok(status)
    }
}
