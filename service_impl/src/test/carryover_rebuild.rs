//! Phase 4 — CarryoverRebuildService service-level tests.
//!
//! Wave 1 implements:
//!   - 1 forbidden-permission test (rebuild_forbidden_for_unprivileged):
//!     verifies the CUTOVER_ADMIN_PRIVILEGE gate at the top of
//!     rebuild_for_year and proves Reporting + Carryover side-services are
//!     never touched on a forbidden call.

use std::sync::Arc;

use mockall::predicate::{always, eq};
use uuid::Uuid;

use dao::{MockTransaction, MockTransactionDao};
use service::carryover::MockCarryoverService;
use service::carryover_rebuild::CarryoverRebuildService;
use service::cutover::CUTOVER_ADMIN_PRIVILEGE;
use service::permission::{Authentication, MockPermissionService};
use service::reporting::MockReportingService;
use service::ServiceError;

use crate::carryover_rebuild::{CarryoverRebuildServiceDeps, CarryoverRebuildServiceImpl};

// ----------------------------------------------------------------------------
// Test harness — multi-mock dependency injection
// ----------------------------------------------------------------------------

struct CarryoverRebuildDependencies {
    reporting_service: MockReportingService,
    carryover_service: MockCarryoverService,
    permission_service: MockPermissionService,
}

impl CarryoverRebuildServiceDeps for CarryoverRebuildDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type ReportingService = MockReportingService;
    type CarryoverService = MockCarryoverService;
    type PermissionService = MockPermissionService;
    type TransactionDao = MockTransactionDao;
}

impl CarryoverRebuildDependencies {
    fn new() -> Self {
        Self {
            reporting_service: MockReportingService::new(),
            carryover_service: MockCarryoverService::new(),
            permission_service: MockPermissionService::new(),
        }
    }

    fn build_service(
        self,
        transaction_dao: MockTransactionDao,
    ) -> CarryoverRebuildServiceImpl<CarryoverRebuildDependencies> {
        CarryoverRebuildServiceImpl {
            reporting_service: Arc::new(self.reporting_service),
            carryover_service: Arc::new(self.carryover_service),
            permission_service: Arc::new(self.permission_service),
            transaction_dao: Arc::new(transaction_dao),
        }
    }
}

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[tokio::test]
async fn rebuild_forbidden_for_unprivileged() {
    // Arrange: PermissionService rejects CUTOVER_ADMIN_PRIVILEGE.
    let mut deps = CarryoverRebuildDependencies::new();
    deps.permission_service
        .expect_check_permission()
        .with(eq(CUTOVER_ADMIN_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Err(ServiceError::Forbidden));

    // Reporting + Carryover MUST NOT be called when the permission gate fails.
    deps.reporting_service
        .expect_get_report_for_employee()
        .times(0);
    deps.carryover_service.expect_get_carryover().times(0);
    deps.carryover_service.expect_set_carryover().times(0);

    // The Tx must NOT be opened either — the gate fires before
    // transaction_dao.use_transaction.
    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao.expect_use_transaction().times(0);
    transaction_dao.expect_commit().times(0);
    transaction_dao.expect_rollback().times(0);

    let service = deps.build_service(transaction_dao);

    // Act
    let result = service
        .rebuild_for_year(
            Uuid::new_v4(),
            2024,
            Authentication::Context(()),
            None,
        )
        .await;

    // Assert: Forbidden propagates verbatim.
    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "expected Err(ServiceError::Forbidden), got {:?}",
        result
    );
}
