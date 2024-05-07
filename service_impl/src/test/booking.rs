use crate::test::error_test::*;
use dao::booking::{BookingEntity, MockBookingDao};
use mockall::predicate::eq;
use service::{
    booking::Booking, clock::MockClockService, uuid_service::MockUuidService, MockPermissionService,
};
use time::{Date, Month, PrimitiveDateTime, Time};
use uuid::{uuid, Uuid};

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
        deleted: None,
        version: default_version(),
    }
}

pub struct BookingServiceDependencies {
    pub booking_dao: MockBookingDao,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
}
impl BookingServiceDependencies {
    pub fn build_service(
        self,
    ) -> BookingServiceImpl<MockBookingDao, MockPermissionService, MockClockService, MockUuidService>
    {
        BookingServiceImpl::new(
            self.booking_dao.into(),
            self.permission_service.into(),
            self.clock_service.into(),
            self.uuid_service.into(),
        )
    }
}

pub fn build_dependencies(permission: bool, role: &'static str) -> BookingServiceDependencies {
    let booking_dao = MockBookingDao::new();
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(role), eq(()))
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

    BookingServiceDependencies {
        booking_dao,
        permission_service,
        clock_service,
        uuid_service,
    }
}

#[tokio::test]
async fn test_get_all() {
    let mut deps = build_dependencies(true, "hr");
    deps.booking_dao.expect_all().returning(|| {
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
    let result = service.get_all(()).await;
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
    let deps = build_dependencies(false, "hr");
    let service = deps.build_service();
    let result = service.get_all(()).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_get() {
    let mut deps = build_dependencies(true, "hr");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_booking_entity())));
    let service = deps.build_service();
    let result = service.get(default_id(), ()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), default_booking());
}

#[tokio::test]
async fn test_get_not_found() {
    let mut deps = build_dependencies(true, "hr");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(None));
    let service = deps.build_service();
    let result = service.get(default_id(), ()).await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_get_no_permission() {
    let deps = build_dependencies(false, "hr");
    let service = deps.build_service();
    let result = service.get(default_id(), ()).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_create() {
    let mut deps = build_dependencies(true, "hr");
    deps.booking_dao
        .expect_create()
        .with(eq(default_booking_entity()), eq("booking-service"))
        .returning(|_, _| Ok(()));
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
                ..default_booking()
            },
            (),
        )
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), default_booking());
}

#[tokio::test]
async fn test_create_no_permission() {
    let deps = build_dependencies(false, "hr");
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                version: Uuid::nil(),
                ..default_booking()
            },
            (),
        )
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_create_with_id() {
    let deps = build_dependencies(true, "hr");
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                version: Uuid::nil(),
                ..default_booking()
            },
            (),
        )
        .await;
    test_zero_id_error(&result);
}

#[tokio::test]
async fn test_create_with_version() {
    let deps = build_dependencies(true, "hr");
    let service = deps.build_service();
    let result = service
        .create(
            &Booking {
                id: Uuid::nil(),
                ..default_booking()
            },
            (),
        )
        .await;
    test_zero_version_error(&result);
}

#[tokio::test]
async fn test_delete_no_permission() {
    let deps = build_dependencies(false, "hr");
    let service = deps.build_service();
    let result = service.delete(default_id(), ()).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_delete_not_found() {
    let mut deps = build_dependencies(true, "hr");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(None));
    let service = deps.build_service();
    let result = service.delete(default_id(), ()).await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_delete() {
    let mut deps = build_dependencies(true, "hr");
    deps.booking_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_booking_entity())));
    deps.booking_dao
        .expect_update()
        .with(
            eq(BookingEntity {
                deleted: Some(PrimitiveDateTime::new(
                    Date::from_calendar_date(2063, Month::April, 5).unwrap(),
                    Time::from_hms(23, 42, 0).unwrap(),
                )),
                version: alternate_version(),
                ..default_booking_entity()
            }),
            eq("booking-service"),
        )
        .returning(|_, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("booking-version"))
        .returning(|_| alternate_version());
    let service = deps.build_service();
    let result = service.delete(default_id(), ()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ());
}
