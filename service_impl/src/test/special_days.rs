use std::sync::Arc;

use crate::special_days::SpecialDayServiceImpl;
use crate::test::error_test::{test_forbidden, test_not_found, test_zero_id_error};
use dao::special_day::{MockSpecialDayDao, SpecialDayEntity, SpecialDayTypeEntity};
use mockall::predicate::{always, eq};
use service::{
    clock::MockClockService,
    permission::SHIFTPLANNER_PRIVILEGE,
    special_days::{SpecialDay, SpecialDayService, SpecialDayType},
    uuid_service::MockUuidService,
    MockPermissionService, ServiceError,
};
use shifty_utils::DayOfWeek;
use time::{Date, Month, PrimitiveDateTime, Time};
use uuid::{uuid, Uuid};

fn make_service(
    dao: MockSpecialDayDao,
    permission: MockPermissionService,
) -> SpecialDayServiceImpl<
    MockSpecialDayDao,
    MockPermissionService,
    MockClockService,
    MockUuidService,
> {
    SpecialDayServiceImpl::new(
        Arc::new(dao),
        Arc::new(permission),
        Arc::new(MockClockService::new()),
        Arc::new(MockUuidService::new()),
    )
}

fn default_id() -> Uuid {
    uuid!("682DA62E-20CB-49D9-A2A7-3F53C6842405")
}

fn make_entity() -> SpecialDayEntity {
    SpecialDayEntity {
        id: Uuid::nil(),
        year: 2026,
        calendar_week: 1,
        day_of_week: DayOfWeek::Monday,
        day_type: SpecialDayTypeEntity::Holiday,
        time_of_day: None,
        created: PrimitiveDateTime::new(
            Date::from_calendar_date(2026, Month::January, 1).unwrap(),
            Time::from_hms(0, 0, 0).unwrap(),
        ),
        deleted: None,
        version: Uuid::nil(),
    }
}

fn minimal_special_day() -> SpecialDay {
    SpecialDay {
        id: Uuid::nil(),
        year: 2026,
        calendar_week: 1,
        day_of_week: DayOfWeek::Monday,
        day_type: SpecialDayType::Holiday,
        time_of_day: None,
        created: None,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// SPD-02, D-33-05: get_by_year delegates to find_by_year and maps entities.
#[tokio::test]
async fn test_get_by_year_delegates_and_maps() {
    let mut dao = MockSpecialDayDao::new();
    dao.expect_find_by_year()
        .with(eq(2026u32))
        .times(1)
        .returning(|_| Ok(Arc::from([make_entity()])));
    let svc = make_service(dao, MockPermissionService::new());
    let result = svc.get_by_year(2026, ().into()).await;
    assert!(result.is_ok(), "get_by_year must succeed: {:?}", result);
    let days = result.unwrap();
    assert_eq!(days.len(), 1, "expected 1 special day");
    assert_eq!(days[0].year, 2026);
    assert_eq!(days[0].calendar_week, 1);
    assert_eq!(days[0].day_of_week, DayOfWeek::Monday);
    assert_eq!(days[0].day_type, SpecialDayType::Holiday);
}

/// D-33-01, SPD-04: create is shiftplanner-gated.
#[tokio::test]
async fn test_create_forbidden_without_shiftplanner() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Err(ServiceError::Forbidden));
    let svc = make_service(MockSpecialDayDao::new(), permission);
    let result = svc.create(&minimal_special_day(), ().into()).await;
    test_forbidden(&result);
}

/// SPD-01: create rejects a non-nil id with IdSetOnCreate.
#[tokio::test]
async fn test_create_rejects_nonnil_id() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Ok(()));
    let mut clock = MockClockService::new();
    clock.expect_date_time_now().returning(|| {
        PrimitiveDateTime::new(
            Date::from_calendar_date(2026, Month::January, 1).unwrap(),
            Time::from_hms(0, 0, 0).unwrap(),
        )
    });
    let svc = SpecialDayServiceImpl::new(
        Arc::new(MockSpecialDayDao::new()),
        Arc::new(permission),
        Arc::new(clock),
        Arc::new(MockUuidService::new()),
    );
    let day_with_id = SpecialDay {
        id: default_id(),
        ..minimal_special_day()
    };
    let result = svc.create(&day_with_id, ().into()).await;
    test_zero_id_error(&result);
}

/// D-33-01, SPD-04: delete is shiftplanner-gated.
#[tokio::test]
async fn test_delete_forbidden_without_shiftplanner() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Err(ServiceError::Forbidden));
    let svc = make_service(MockSpecialDayDao::new(), permission);
    let result = svc.delete(default_id(), ().into()).await;
    test_forbidden(&result);
}

/// SPD-03: delete of a missing id returns EntityNotFound.
#[tokio::test]
async fn test_delete_not_found() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Ok(()));
    let mut dao = MockSpecialDayDao::new();
    dao.expect_find_by_id()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(None));
    let svc = make_service(dao, permission);
    let result = svc.delete(default_id(), ().into()).await;
    test_not_found(&result, &default_id());
}
