use crate::{booking::BookingServiceDeps, test::error_test::*};
use dao::{
    booking::{BookingEntity, MockBookingDao},
    MockTransaction, MockTransactionDao,
};
use mockall::predicate::{always, eq};
use service::{
    booking::Booking, clock::MockClockService, permission::Authentication,
    sales_person::MockSalesPersonService, slot::MockSlotService, uuid_service::MockUuidService,
    MockPermissionService, ValidationFailureItem,
};
use time::{Date, Month, PrimitiveDateTime, Time};
use uuid::{uuid, Uuid};

use super::error_test::NoneTypeExt;
use crate::booking::BookingServiceImpl;
use service::booking::BookingService;

pub fn default_id() -> Uuid {
    uuid!("CEA260A0-112B-4970-936C-F7E529955BD0")
}
pub fn alternate_id() -> Uuid {
    uuid!("CEA260A0-112B-4970-936C-F7E529955BD1")
}
pub fn default_version() -> Uuid {
    uuid!("F79C462A-8D4E-42E1-8171-DB4DBD019E50")
}
pub fn alternate_version() -> Uuid {
    uuid!("F79C462A-8D4E-42E1-8171-DB4DBD019E51")
}
pub fn default_sales_person_id() -> Uuid {
    uuid!("04215DFE-13C4-413C-8C66-77AC741BB5F0")
}
pub fn default_slot_id() -> Uuid {
    uuid!("7A7FF57A-782B-4C2E-A68B-4E2D81D79380")
}

pub fn default_booking() -> Booking {
    Booking {
        id: default_id(),
        sales_person_id: default_sales_person_id(),
        slot_id: default_slot_id(),
        calendar_week: 3,
        year: 2024,
        created: Some(PrimitiveDateTime::new(
            Date::from_calendar_date(2024, Month::January, 1).unwrap(),
            Time::from_hms(0, 0, 0).unwrap(),
        )),
        deleted: None,
        version: default_version(),
    }
}

pub fn default_booking_entity() -> BookingEntity {
    BookingEntity {
        id: default_id(),
        sales_person_id: default_sales_person_id(),
        slot_id: default_slot_id(),
        calendar_week: 3,
        year: 2024,
        created: PrimitiveDateTime::new(
            Date::from_calendar_date(2024, Month::January, 1).unwrap(),
            Time::from_hms(0, 0, 0).unwrap(),
        ),
        deleted: None,
        version: default_version(),
    }
}

pub struct BookingServiceDependencies {
    pub booking_dao: MockBookingDao,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
    pub sales_person_service: MockSalesPersonService,
    pub slot_service: MockSlotService,
    pub transaction_dao: MockTransactionDao,
}
impl BookingServiceDeps for BookingServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type BookingDao = MockBookingDao;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type SalesPersonService = MockSalesPersonService;
    type SlotService = MockSlotService;
    type TransactionDao = MockTransactionDao;
}
impl BookingServiceDependencies {
    pub fn build_service(self) -> BookingServiceImpl<BookingServiceDependencies> {
        BookingServiceImpl {
            booking_dao: self.booking_dao.into(),
            permission_service: self.permission_service.into(),
            clock_service: self.clock_service.into(),
            uuid_service: self.uuid_service.into(),
            sales_person_service: self.sales_person_service.into(),
            slot_service: self.slot_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

pub fn build_dependencies(permission: bool, role: &'static str) -> BookingServiceDependencies {
    let mut booking_dao = MockBookingDao::new();
    booking_dao
        .expect_find_by_booking_data()
        .returning(|_, _, _, _, _| Ok(None));
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(role), eq(().auth()))
        .returning(move |_, _| {
            if permission {
                Ok(())
            } else {
                Err(service::ServiceError::Forbidden)
            }
        });
    permission_service
        .expect_check_permission()
        .returning(move |_, _| Err(service::ServiceError::Forbidden));
    let mut clock_service = MockClockService::new();
    clock_service
        .expect_time_now()
        .returning(|| time::Time::from_hms(23, 42, 0).unwrap());
    clock_service
        .expect_date_now()
        .returning(|| time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap());
    clock_service.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
            time::Time::from_hms(23, 42, 0).unwrap(),
        )
    });
    let uuid_service = MockUuidService::new();

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_exists()
        .returning(|_, _, _| Ok(true));
    let mut slot_service = MockSlotService::new();
    slot_service.expect_exists().returning(|_, _, _| Ok(true));

    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    BookingServiceDependencies {
        booking_dao,
        permission_service,
        clock_service,
        uuid_service,
        sales_person_service,
        slot_service,
        transaction_dao,
    }
}

#[tokio::test]
async fn test_get_all() {
    let mut deps = build_dependencies(true, "shiftplanner");
    deps.booking_dao.expect_all().returning(|_| {
        Ok([
            default_booking_entity(),
            BookingEntity {
                id: alternate_id(),
                ..default_booking_entity()
            },
        ]
        .into())
    });
    let service = deps.build_service();
    let result = service.get_all(().auth(), None).await;
    dbg!(&result);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], default_booking());
    assert_eq!(
        result[1],
        Booking {
            id: alternate_id(),
            ..default_booking()
        }
    );
}

#[tokio::test]
async fn test_get_all_no_permission() {
    let deps = build_dependencies(false, "shiftplanner");
    let service = deps.build_service();
    let result = service.get_all(().auth(), None).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_get() {
    let mut deps = build_dependencies(true, "shiftplanner");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_booking_entity())));
    let service = deps.build_service();
    let result = service.get(default_id(), ().auth(), None).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), default_booking());
}

#[tokio::test]
async fn test_get_not_found() {
    let mut deps = build_dependencies(true, "shiftplanner");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(None));
    let service = deps.build_service();
    let result = service.get(default_id(), ().auth(), None).await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_get_no_permission() {
    let deps = build_dependencies(false, "shiftplanner");
    let service = deps.build_service();
    let result = service.get(default_id(), ().auth(), None).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_get_for_week() {
    let mut deps = build_dependencies(true, "sales");
    deps.booking_dao
        .expect_find_by_week()
        .with(eq(3), eq(2024), always())
        .returning(|_, _, _| Ok([default_booking_entity()].into()));
    let service = deps.build_service();
    let result = service.get_for_week(3, 2024, ().auth(), None).await;
    assert_eq!(result.unwrap(), [default_booking()].into());
}

#[tokio::test]
async fn test_get_for_week_shiftplanner_role() {
    let mut deps = build_dependencies(true, "shiftplanner");
    deps.booking_dao
        .expect_find_by_week()
        .with(eq(3), eq(2024), always())
        .returning(|_, _, _| Ok([default_booking_entity()].into()));
    let service = deps.build_service();
    let result = service.get_for_week(3, 2024, ().auth(), None).await;
    assert_eq!(result.unwrap(), [default_booking()].into());
}

#[tokio::test]
async fn test_get_for_week_no_permission() {
    let deps = build_dependencies(false, "shiftplanner");
    let service = deps.build_service();
    let result = service.get_for_week(3, 2024, ().auth(), None).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_create() {
    let mut deps = build_dependencies(true, "shiftplanner");
    deps.booking_dao
        .expect_create()
        .with(
            eq(BookingEntity {
                created: generate_default_datetime(),
                ..default_booking_entity()
            }),
            eq("booking-service"),
            always(),
        )
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-id"))
        .returning(|_| default_id());
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-version"))
        .returning(|_| default_version());
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                version: Uuid::nil(),
                created: None,
                ..default_booking()
            },
            ().auth(),
            None,
        )
        .await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Booking {
            created: Some(generate_default_datetime()),
            ..default_booking()
        }
    );
}

#[tokio::test]
async fn test_create_sales_user() {
    let mut deps = build_dependencies(true, "sales");
    deps.booking_dao
        .expect_create()
        .with(
            eq(BookingEntity {
                created: generate_default_datetime(),
                ..default_booking_entity()
            }),
            eq("booking-service"),
            always(),
        )
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-id"))
        .returning(|_| default_id());
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-version"))
        .returning(|_| default_version());
    deps.sales_person_service
        .expect_get_assigned_user()
        .with(eq(default_sales_person_id()), always(), always())
        .returning(|_, _, _| Ok(Some("TESTUSER".into())));
    deps.permission_service
        .expect_check_user()
        .with(eq("TESTUSER"), always())
        .returning(|_, _| Ok(()));
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                version: Uuid::nil(),
                created: None,
                ..default_booking()
            },
            ().auth(),
            None,
        )
        .await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Booking {
            created: Some(generate_default_datetime()),
            ..default_booking()
        }
    );
}

#[tokio::test]
async fn test_create_sales_user_not_exist() {
    let mut deps = build_dependencies(true, "sales");
    deps.booking_dao
        .expect_create()
        .with(
            eq(BookingEntity {
                created: generate_default_datetime(),
                ..default_booking_entity()
            }),
            eq("booking-service"),
            always(),
        )
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-id"))
        .returning(|_| default_id());
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-version"))
        .returning(|_| default_version());
    deps.sales_person_service
        .expect_get_assigned_user()
        .with(eq(default_sales_person_id()), always(), always())
        .returning(|_, _, _| Ok(Some("TESTUSER".into())));
    deps.permission_service
        .expect_check_user()
        .with(eq("TESTUSER"), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                version: Uuid::nil(),
                created: None,
                ..default_booking()
            },
            ().auth(),
            None,
        )
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_create_sales_user_no_permission() {
    let mut deps = build_dependencies(true, "sales");
    deps.booking_dao
        .expect_create()
        .with(
            eq(BookingEntity {
                created: generate_default_datetime(),
                ..default_booking_entity()
            }),
            eq("booking-service"),
            always(),
        )
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-id"))
        .returning(|_| default_id());
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-version"))
        .returning(|_| default_version());
    deps.sales_person_service
        .expect_get_assigned_user()
        .with(eq(default_sales_person_id()), always(), always())
        .returning(|_, _, _| Ok(None));
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                version: Uuid::nil(),
                created: None,
                ..default_booking()
            },
            ().auth(),
            None,
        )
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_create_no_permission() {
    let deps = build_dependencies(false, "shiftplanner");
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                version: Uuid::nil(),
                ..default_booking()
            },
            ().auth(),
            None,
        )
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_create_with_id() {
    let deps = build_dependencies(true, "shiftplanner");
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                version: Uuid::nil(),
                ..default_booking()
            },
            ().auth(),
            None,
        )
        .await;
    test_zero_id_error(&result);
}

#[tokio::test]
async fn test_create_with_version() {
    let deps = build_dependencies(true, "shiftplanner");
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                ..default_booking()
            },
            ().auth(),
            None,
        )
        .await;
    test_zero_version_error(&result);
}

#[tokio::test]
async fn test_create_with_created_fail() {
    let deps = build_dependencies(true, "shiftplanner");
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                version: Uuid::nil(),
                ..default_booking()
            },
            ().auth(),
            None,
        )
        .await;
    test_validation_error(
        &result,
        &ValidationFailureItem::InvalidValue("created".into()),
        1,
    );
}

#[tokio::test]
async fn test_create_sales_person_does_not_exist() {
    let mut deps = build_dependencies(true, "shiftplanner");
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_exists()
        .with(
            eq(default_sales_person_id()),
            eq(Authentication::Full),
            always(),
        )
        .returning(|_, _, _| Ok(false));
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                version: Uuid::nil(),
                created: None,
                ..default_booking()
            },
            ().auth(),
            None,
        )
        .await;
    dbg!(&result);
    test_validation_error(
        &result,
        &ValidationFailureItem::IdDoesNotExist("sales_person_id".into(), default_sales_person_id()),
        1,
    );
}

#[tokio::test]
async fn test_create_booking_data_already_exists() {
    let mut deps = build_dependencies(true, "shiftplanner");
    deps.booking_dao.checkpoint();
    deps.booking_dao
        .expect_find_by_booking_data()
        .with(
            eq(default_sales_person_id()),
            eq(default_slot_id()),
            eq(3),
            eq(2024),
            always(),
        )
        .returning(|_, _, _, _, _| Ok(Some(default_booking_entity())));
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                version: Uuid::nil(),
                created: None,
                ..default_booking()
            },
            ().auth(),
            None,
        )
        .await;
    test_validation_error(&result, &ValidationFailureItem::Duplicate, 1);
}

#[tokio::test]
async fn test_create_slot_does_not_exist() {
    let mut deps = build_dependencies(true, "shiftplanner");
    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_exists()
        .with(eq(default_slot_id()), eq(Authentication::Full), always())
        .returning(|_, _, _| Ok(false));
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                version: Uuid::nil(),
                created: None,
                ..default_booking()
            },
            ().auth(),
            None,
        )
        .await;
    test_validation_error(
        &result,
        &ValidationFailureItem::IdDoesNotExist("slot_id".into(), default_slot_id()),
        1,
    );
}

#[tokio::test]
async fn test_delete_no_permission() {
    let mut deps = build_dependencies(false, "shiftplanner");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_booking_entity())));
    let service = deps.build_service();
    let result = service.delete(default_id(), ().auth(), None).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_delete_not_found() {
    let mut deps = build_dependencies(true, "shiftplanner");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(None));
    let service = deps.build_service();
    let result = service.delete(default_id(), ().auth(), None).await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_delete() {
    let mut deps = build_dependencies(true, "shiftplanner");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_booking_entity())));
    deps.booking_dao
        .expect_update()
        .with(
            eq(BookingEntity {
                deleted: Some(generate_default_datetime()),
                version: alternate_version(),
                ..default_booking_entity()
            }),
            eq("booking-service"),
            always(),
        )
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-version"))
        .returning(|_| alternate_version());
    let service = deps.build_service();
    let result = service.delete(default_id(), ().auth(), None).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ());
}

#[tokio::test]
async fn test_delete_sales_user() {
    let mut deps = build_dependencies(true, "sales");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_booking_entity())));
    deps.booking_dao
        .expect_update()
        .with(
            eq(BookingEntity {
                deleted: Some(generate_default_datetime()),
                version: alternate_version(),
                ..default_booking_entity()
            }),
            eq("booking-service"),
            always(),
        )
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-version"))
        .returning(|_| alternate_version());
    deps.sales_person_service
        .expect_get_assigned_user()
        .with(eq(default_sales_person_id()), always(), always())
        .returning(|_, _, _| Ok(Some("TESTUSER".into())));
    deps.permission_service
        .expect_check_user()
        .with(eq("TESTUSER"), always())
        .returning(|_, _| Ok(()));
    let service = deps.build_service();
    let result = service.delete(default_id(), ().auth(), None).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ());
}

#[tokio::test]
async fn test_delete_sales_user_not_exists() {
    let mut deps = build_dependencies(true, "sales");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_booking_entity())));
    deps.booking_dao
        .expect_update()
        .with(
            eq(BookingEntity {
                deleted: Some(generate_default_datetime()),
                version: alternate_version(),
                ..default_booking_entity()
            }),
            eq("booking-service"),
            always(),
        )
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-version"))
        .returning(|_| alternate_version());
    deps.sales_person_service
        .expect_get_assigned_user()
        .with(eq(default_sales_person_id()), always(), always())
        .returning(|_, _, _| Ok(None));
    let service = deps.build_service();
    let result = service.delete(default_id(), ().auth(), None).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_delete_sales_user_not_allowed() {
    let mut deps = build_dependencies(true, "sales");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_booking_entity())));
    deps.booking_dao
        .expect_update()
        .with(
            eq(BookingEntity {
                deleted: Some(generate_default_datetime()),
                version: alternate_version(),
                ..default_booking_entity()
            }),
            eq("booking-service"),
            always(),
        )
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-version"))
        .returning(|_| alternate_version());
    deps.sales_person_service
        .expect_get_assigned_user()
        .with(eq(default_sales_person_id()), always(), always())
        .returning(|_, _, _| Ok(Some("TESTUSER".into())));
    deps.permission_service
        .expect_check_user()
        .with(eq("TESTUSER"), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    let service = deps.build_service();
    let result = service.delete(default_id(), ().auth(), None).await;
    test_forbidden(&result);
}
