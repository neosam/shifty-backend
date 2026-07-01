use std::sync::Arc;

use crate::special_days::SpecialDayServiceImpl;
use crate::test::error_test::{
    test_forbidden, test_not_found, test_validation_error, test_zero_id_error,
    test_zero_version_error,
};
use dao::special_day::{MockSpecialDayDao, SpecialDayEntity, SpecialDayTypeEntity};
use mockall::predicate::{always, eq};
use service::{
    clock::MockClockService,
    permission::SHIFTPLANNER_PRIVILEGE,
    special_days::{SpecialDay, SpecialDayService, SpecialDayType},
    uuid_service::MockUuidService,
    MockPermissionService, ServiceError, ValidationFailureItem,
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

fn default_version() -> Uuid {
    uuid!("11111111-2222-3333-4444-555555555555")
}

fn fixed_created() -> PrimitiveDateTime {
    PrimitiveDateTime::new(
        Date::from_calendar_date(2026, Month::January, 1).unwrap(),
        Time::from_hms(0, 0, 0).unwrap(),
    )
}

/// A concrete non-nil id for the "existing" row in replacement tests so that the
/// "keeps existing id" assertion is meaningful (distinct from the nil id the
/// client POSTs on create).
fn existing_id() -> Uuid {
    uuid!("B0B0B0B0-0000-0000-0000-000000000001")
}

/// Version uuid assigned by the service on the replacement path.
fn replace_version() -> Uuid {
    uuid!("CCCCCCCC-0000-0000-0000-000000000002")
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

/// Build an existing active entity for (2026, W1, Monday) with a given id, day_type,
/// and time_of_day — used in directional switch tests (SDF-01, D-09).
fn make_existing_entity(
    id: Uuid,
    day_type: SpecialDayTypeEntity,
    time_of_day: Option<Time>,
) -> SpecialDayEntity {
    SpecialDayEntity {
        id,
        year: 2026,
        calendar_week: 1,
        day_of_week: DayOfWeek::Monday,
        day_type,
        time_of_day,
        created: fixed_created(),
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

/// SPD-01, WR-05: create happy-path assigns a fresh id/version, stamps `created`
/// from the clock, and invokes `dao.create` once with the expected entity.
#[tokio::test]
async fn test_create_success() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Ok(()));
    let mut clock = MockClockService::new();
    clock.expect_date_time_now().returning(fixed_created);
    let mut uuid = MockUuidService::new();
    uuid.expect_new_uuid()
        .with(eq("special-day-service::create id"))
        .times(1)
        .returning(|_| default_id());
    uuid.expect_new_uuid()
        .with(eq("special-day-service::create version"))
        .times(1)
        .returning(|_| default_version());
    let mut dao = MockSpecialDayDao::new();
    dao.expect_find_by_week()
        .with(eq(2026u32), eq(1u8))
        .times(1)
        .returning(|_, _| Ok(Arc::from([])));
    let expected = SpecialDayEntity {
        id: default_id(),
        year: 2026,
        calendar_week: 1,
        day_of_week: DayOfWeek::Monday,
        day_type: SpecialDayTypeEntity::Holiday,
        time_of_day: None,
        created: fixed_created(),
        deleted: None,
        version: default_version(),
    };
    dao.expect_create()
        .withf(move |entity, process| {
            entity == &expected && process == "special-days-service::create"
        })
        .times(1)
        .returning(|_, _| Ok(()));
    let svc = SpecialDayServiceImpl::new(
        Arc::new(dao),
        Arc::new(permission),
        Arc::new(clock),
        Arc::new(uuid),
    );
    let result = svc.create(&minimal_special_day(), ().into()).await;
    assert!(result.is_ok(), "create must succeed: {:?}", result);
    let created = result.unwrap();
    assert_eq!(created.id, default_id());
    assert_eq!(created.version, default_version());
    assert!(!created.id.is_nil(), "id must be assigned");
    assert!(!created.version.is_nil(), "version must be assigned");
    assert_eq!(created.created, Some(fixed_created()));
}

/// SPD-01, WR-05: create rejects a non-nil version with VersionSetOnCreate.
#[tokio::test]
async fn test_create_rejects_nonnil_version() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Ok(()));
    let mut clock = MockClockService::new();
    clock.expect_date_time_now().returning(fixed_created);
    let svc = SpecialDayServiceImpl::new(
        Arc::new(MockSpecialDayDao::new()),
        Arc::new(permission),
        Arc::new(clock),
        Arc::new(MockUuidService::new()),
    );
    let day_with_version = SpecialDay {
        version: default_version(),
        ..minimal_special_day()
    };
    let result = svc.create(&day_with_version, ().into()).await;
    test_zero_version_error(&result);
}

/// SDF-01, D-01: create replaces an existing same-date entry instead of returning
/// ValidationError(Duplicate) — the duplicate guard is superseded by replacement semantics.
/// The replacement path calls dao.update once (keeping the existing id) and never calls
/// dao.create.
#[tokio::test]
async fn test_create_replaces_same_date_entry() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Ok(()));
    let mut clock = MockClockService::new();
    clock.expect_date_time_now().returning(fixed_created);
    let mut uuid = MockUuidService::new();
    uuid.expect_new_uuid()
        .with(eq("special-day-service::replace version"))
        .times(1)
        .returning(|_| replace_version());
    let mut dao = MockSpecialDayDao::new();
    // An existing Monday/2026/W1 Holiday entry (same date as minimal_special_day()).
    dao.expect_find_by_week()
        .with(eq(2026u32), eq(1u8))
        .times(1)
        .returning(|_, _| Ok(Arc::from([make_entity()])));
    // Replacement: update is called once with the existing entity's id; create is never called.
    dao.expect_update()
        .withf(|entity, process| {
            entity.id == Uuid::nil()
                && entity.day_type == SpecialDayTypeEntity::Holiday
                && entity.time_of_day.is_none()
                && entity.deleted.is_none()
                && process == "special-days-service::replace"
        })
        .times(1)
        .returning(|_, _| Ok(()));
    dao.expect_create().times(0);
    let svc = SpecialDayServiceImpl::new(
        Arc::new(dao),
        Arc::new(permission),
        Arc::new(clock),
        Arc::new(uuid),
    );
    let result = svc.create(&minimal_special_day(), ().into()).await;
    assert!(
        result.is_ok(),
        "same-date entry must be replaced, not rejected: {:?}",
        result
    );
}

/// WR-03: create rejects a ShortDay without a time_of_day.
#[tokio::test]
async fn test_create_rejects_shortday_without_time() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Ok(()));
    let svc = make_service(MockSpecialDayDao::new(), permission);
    let short_day = SpecialDay {
        day_type: SpecialDayType::ShortDay,
        time_of_day: None,
        ..minimal_special_day()
    };
    let result = svc.create(&short_day, ().into()).await;
    test_validation_error(
        &result,
        &ValidationFailureItem::InvalidValue("time_of_day is required for a ShortDay".into()),
        1,
    );
}

/// WR-03: create rejects a calendar_week outside the valid range for the year.
#[tokio::test]
async fn test_create_rejects_calendar_week_out_of_range() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Ok(()));
    let svc = make_service(MockSpecialDayDao::new(), permission);
    let bad_week = SpecialDay {
        calendar_week: 60,
        ..minimal_special_day()
    };
    let result = svc.create(&bad_week, ().into()).await;
    match &result {
        Err(ServiceError::ValidationError(items)) => assert_eq!(items.len(), 1),
        other => panic!("expected validation error, got {:?}", other),
    }
}

/// SPD-02, WR-05: get_by_week delegates to find_by_week and maps entities.
#[tokio::test]
async fn test_get_by_week_delegates_and_maps() {
    let mut dao = MockSpecialDayDao::new();
    dao.expect_find_by_week()
        .with(eq(2026u32), eq(1u8))
        .times(1)
        .returning(|_, _| Ok(Arc::from([make_entity()])));
    let svc = make_service(dao, MockPermissionService::new());
    let result = svc.get_by_week(2026, 1, ().into()).await;
    assert!(result.is_ok(), "get_by_week must succeed: {:?}", result);
    let days = result.unwrap();
    assert_eq!(days.len(), 1, "expected 1 special day");
    assert_eq!(days[0].year, 2026);
    assert_eq!(days[0].calendar_week, 1);
    assert_eq!(days[0].day_of_week, DayOfWeek::Monday);
    assert_eq!(days[0].day_type, SpecialDayType::Holiday);
}

/// SDF-01, D-01, D-04, D-09: create replaces an existing Holiday at (2026, W1, Monday)
/// with a ShortDay by calling dao.update once with the existing id and new day_type=ShortDay
/// and the supplied time_of_day — no ValidationError, no dao.create call.
#[tokio::test]
async fn test_create_switches_holiday_to_shortday() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Ok(()));
    let mut clock = MockClockService::new();
    clock.expect_date_time_now().returning(fixed_created);
    let mut uuid = MockUuidService::new();
    uuid.expect_new_uuid()
        .with(eq("special-day-service::replace version"))
        .times(1)
        .returning(|_| replace_version());
    let existing = make_existing_entity(existing_id(), SpecialDayTypeEntity::Holiday, None);
    let time_12 = Time::from_hms(12, 0, 0).unwrap();
    let expected_updated = SpecialDayEntity {
        id: existing_id(),
        year: 2026,
        calendar_week: 1,
        day_of_week: DayOfWeek::Monday,
        day_type: SpecialDayTypeEntity::ShortDay,
        time_of_day: Some(time_12),
        created: fixed_created(),
        deleted: None,
        version: replace_version(),
    };
    let mut dao = MockSpecialDayDao::new();
    dao.expect_find_by_week()
        .with(eq(2026u32), eq(1u8))
        .times(1)
        .returning(move |_, _| Ok(Arc::from([existing.clone()])));
    dao.expect_update()
        .withf(move |entity, process| {
            *entity == expected_updated && process == "special-days-service::replace"
        })
        .times(1)
        .returning(|_, _| Ok(()));
    dao.expect_create().times(0);
    let svc = SpecialDayServiceImpl::new(
        Arc::new(dao),
        Arc::new(permission),
        Arc::new(clock),
        Arc::new(uuid),
    );
    let input = SpecialDay {
        day_type: SpecialDayType::ShortDay,
        time_of_day: Some(time_12),
        ..minimal_special_day()
    };
    let result = svc.create(&input, ().into()).await;
    assert!(
        result.is_ok(),
        "Holiday->ShortDay switch must succeed: {:?}",
        result
    );
    let created = result.unwrap();
    assert_eq!(
        created.id,
        existing_id(),
        "returned id must be the existing entity id"
    );
    assert_eq!(created.day_type, SpecialDayType::ShortDay);
    assert_eq!(created.time_of_day, Some(time_12));
}

/// SDF-01, D-01, D-04, D-09: create replaces an existing ShortDay at (2026, W1, Monday)
/// with a Holiday by calling dao.update once with the existing id, day_type=Holiday and
/// time_of_day normalized to None — no ValidationError, no dao.create call.
#[tokio::test]
async fn test_create_switches_shortday_to_holiday() {
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .with(eq(SHIFTPLANNER_PRIVILEGE), always())
        .times(1)
        .returning(|_, _| Ok(()));
    let mut clock = MockClockService::new();
    clock.expect_date_time_now().returning(fixed_created);
    let mut uuid = MockUuidService::new();
    uuid.expect_new_uuid()
        .with(eq("special-day-service::replace version"))
        .times(1)
        .returning(|_| replace_version());
    let time_12 = Time::from_hms(12, 0, 0).unwrap();
    let existing =
        make_existing_entity(existing_id(), SpecialDayTypeEntity::ShortDay, Some(time_12));
    let expected_updated = SpecialDayEntity {
        id: existing_id(),
        year: 2026,
        calendar_week: 1,
        day_of_week: DayOfWeek::Monday,
        day_type: SpecialDayTypeEntity::Holiday,
        time_of_day: None,
        created: fixed_created(),
        deleted: None,
        version: replace_version(),
    };
    let mut dao = MockSpecialDayDao::new();
    dao.expect_find_by_week()
        .with(eq(2026u32), eq(1u8))
        .times(1)
        .returning(move |_, _| Ok(Arc::from([existing.clone()])));
    dao.expect_update()
        .withf(move |entity, process| {
            *entity == expected_updated && process == "special-days-service::replace"
        })
        .times(1)
        .returning(|_, _| Ok(()));
    dao.expect_create().times(0);
    let svc = SpecialDayServiceImpl::new(
        Arc::new(dao),
        Arc::new(permission),
        Arc::new(clock),
        Arc::new(uuid),
    );
    let input = SpecialDay {
        day_type: SpecialDayType::Holiday,
        time_of_day: None,
        ..minimal_special_day()
    };
    let result = svc.create(&input, ().into()).await;
    assert!(
        result.is_ok(),
        "ShortDay->Holiday switch must succeed: {:?}",
        result
    );
    let created = result.unwrap();
    assert_eq!(
        created.id,
        existing_id(),
        "returned id must be the existing entity id"
    );
    assert_eq!(created.day_type, SpecialDayType::Holiday);
    assert!(
        created.time_of_day.is_none(),
        "Holiday must have no time_of_day"
    );
}
