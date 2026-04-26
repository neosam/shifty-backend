use crate::employee_work_details::{
    EmployeeWorkDetailsServiceDeps, EmployeeWorkDetailsServiceImpl,
};
use crate::test::error_test::NoneTypeExt;
use dao::{
    employee_work_details::{EmployeeWorkDetailsEntity, MockEmployeeWorkDetailsDao},
    MockTransaction, MockTransactionDao,
};
use mockall::predicate::{always, function};
use service::{
    clock::MockClockService,
    employee_work_details::{EmployeeWorkDetails, EmployeeWorkDetailsService},
    sales_person::MockSalesPersonService,
    uuid_service::MockUuidService,
    MockPermissionService,
};
use shifty_utils::DayOfWeek;
use time::macros::datetime;
use uuid::Uuid;

struct Deps {
    employee_work_details_dao: MockEmployeeWorkDetailsDao,
    sales_person_service: MockSalesPersonService,
    permission_service: MockPermissionService,
    clock_service: MockClockService,
    uuid_service: MockUuidService,
    transaction_dao: MockTransactionDao,
}

impl EmployeeWorkDetailsServiceDeps for Deps {
    type Context = ();
    type Transaction = MockTransaction;
    type EmployeeWorkDetailsDao = MockEmployeeWorkDetailsDao;
    type SalesPersonService = MockSalesPersonService;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = MockTransactionDao;
}

impl Deps {
    fn build(self) -> EmployeeWorkDetailsServiceImpl<Deps> {
        EmployeeWorkDetailsServiceImpl {
            employee_work_details_dao: self.employee_work_details_dao.into(),
            sales_person_service: self.sales_person_service.into(),
            permission_service: self.permission_service.into(),
            clock_service: self.clock_service.into(),
            uuid_service: self.uuid_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

fn entity_with_cap(id: Uuid, version: Uuid, cap: bool) -> EmployeeWorkDetailsEntity {
    EmployeeWorkDetailsEntity {
        id,
        sales_person_id: Uuid::new_v4(),
        expected_hours: 5.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2024,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 52,
        to_year: 2030,
        workdays_per_week: 3,
        is_dynamic: false,
        cap_planned_hours_to_expected: cap,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: false,
        friday: false,
        saturday: false,
        sunday: false,
        vacation_days: 15,
        created: datetime!(2024-01-01 10:00:00),
        deleted: None,
        version,
    }
}

/// Regression test for the bug where the service-layer `update()` selectively
/// copied input fields onto the loaded entity but forgot
/// `cap_planned_hours_to_expected`, causing the DAO to write back the old
/// (false) value even though the SQL UPDATE included the column.
#[tokio::test]
async fn update_propagates_cap_planned_hours_flag_to_dao() {
    let id = Uuid::new_v4();
    let version = Uuid::new_v4();
    let next_version = Uuid::new_v4();

    let mut dao = MockEmployeeWorkDetailsDao::new();
    // Existing record has cap = false (the persisted state before the update).
    dao.expect_find_by_id()
        .returning(move |_, _| Ok(Some(entity_with_cap(id, version, false))));
    // The crucial assertion: the entity passed into update() must reflect the
    // input value (true), not the stale loaded value.
    dao.expect_update()
        .with(
            function(|e: &EmployeeWorkDetailsEntity| e.cap_planned_hours_to_expected),
            always(),
            always(),
        )
        .returning(|_, _, _| Ok(()));

    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    let mut uuid_service = MockUuidService::new();
    uuid_service
        .expect_new_uuid()
        .returning(move |_| next_version);

    let deps = Deps {
        employee_work_details_dao: dao,
        sales_person_service: MockSalesPersonService::new(),
        permission_service,
        clock_service: MockClockService::new(),
        uuid_service,
        transaction_dao,
    };
    let service = deps.build();

    // Input from REST: caller flips cap to true.
    let mut input = EmployeeWorkDetails::from(&entity_with_cap(id, version, true));
    input.cap_planned_hours_to_expected = true;

    let result = service.update(&input, ().auth(), None).await;
    assert!(result.is_ok(), "update failed: {:?}", result.err());
    let returned = result.unwrap();
    assert!(
        returned.cap_planned_hours_to_expected,
        "returned entity must reflect the new cap value"
    );
}
