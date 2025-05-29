use crate::{block::BlockServiceImpl, test::error_test::*};
use dao::{MockTransaction, MockTransactionDao};
use mockall::predicate::{always, eq};
use service::block::BlockService;
use service::booking::Booking;
use service::clock::MockClockService;
use service::ical::MockIcalService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan::MockShiftplanService;
use service::slot::{DayOfWeek, MockSlotService, Slot};
use service::ServiceError;
use service::{booking::MockBookingService, sales_person::SalesPerson};
use time::macros::date;
use time::{Date, Month, PrimitiveDateTime, Time};
use uuid::{uuid, Uuid};

/// Same pattern as `BookingServiceDeps`; the macro `gen_service_impl!`
/// creates `BlockServiceDeps`, but for testing we define our own struct
/// to hold mocks.
pub struct BlockServiceDependencies {
    pub booking_service: MockBookingService,
    pub slot_service: MockSlotService,
    pub sales_person_service: MockSalesPersonService,
    pub ical_service: MockIcalService,
    pub clock_service: MockClockService,
    pub transaction_dao: MockTransactionDao,
    pub shiftplan_service: MockShiftplanService,
}

impl crate::block::BlockServiceDeps for BlockServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type BookingService = MockBookingService;
    type SlotService = MockSlotService;
    type SalesPersonService = MockSalesPersonService;
    type IcalService = MockIcalService;
    type ClockService = MockClockService;
    type TransactionDao = MockTransactionDao;
    type ShiftplanService = MockShiftplanService;
    // If you also want to enforce permission checks here, you can add:
    // type PermissionService = MockPermissionService;
}

impl BlockServiceDependencies {
    /// Build the actual `BlockServiceImpl` from the dependencies.
    pub fn build_service(self) -> BlockServiceImpl<BlockServiceDependencies> {
        BlockServiceImpl {
            booking_service: self.booking_service.into(),
            slot_service: self.slot_service.into(),
            sales_person_service: self.sales_person_service.into(),
            ical_service: self.ical_service.into(),
            clock_service: self.clock_service.into(),
            transaction_dao: self.transaction_dao.into(),
            shiftplan_service: self.shiftplan_service.into(),
        }
    }
}

/// Some helper constants/methods for default IDs, etc.
pub fn default_sales_person_id() -> Uuid {
    uuid!("e12b19a2-7ec7-41fb-9090-94d699635894")
}
pub fn default_slot_id() -> Uuid {
    uuid!("a8757606-56bd-4456-9baf-058c0bd19cb4")
}
pub fn second_slot_id() -> Uuid {
    uuid!("96426f59-61cf-485e-a28b-54ddef7e0c5b")
}
pub fn default_booking_id() -> Uuid {
    uuid!("522c46c6-1062-4ce2-8fdf-c9530dcc7fc2")
}
pub fn second_booking_id() -> Uuid {
    uuid!("84e761e7-4c32-4da3-a1e2-908c657938ac")
}

/// Default `SalesPerson` used for tests.
pub fn default_sales_person() -> SalesPerson {
    SalesPerson {
        id: default_sales_person_id(),
        name: "Default SalesPerson".into(),
        background_color: "#FFFFFF".into(),
        is_paid: None,
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Default `Slot` with Monday 09:00-10:00 for example.
pub fn default_slot() -> Slot {
    Slot {
        id: default_slot_id(),
        day_of_week: DayOfWeek::Monday,
        from: Time::from_hms(9, 0, 0).unwrap(),
        to: Time::from_hms(10, 0, 0).unwrap(),
        min_resources: 1,
        valid_from: Date::from_calendar_date(2024, Month::January, 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// A second `Slot` that starts at 10:00 if you want to chain consecutively.
pub fn second_slot() -> Slot {
    Slot {
        id: second_slot_id(),
        day_of_week: DayOfWeek::Monday,
        from: Time::from_hms(10, 0, 0).unwrap(),
        to: Time::from_hms(11, 0, 0).unwrap(),
        min_resources: 1,
        valid_from: Date::from_calendar_date(2024, Month::January, 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Default `Booking` that references the default slot.
pub fn default_booking() -> Booking {
    Booking {
        id: default_booking_id(),
        sales_person_id: default_sales_person_id(),
        slot_id: default_slot_id(),
        calendar_week: 3,
        year: 2025,
        created: Some(PrimitiveDateTime::new(
            Date::from_calendar_date(2025, Month::January, 1).unwrap(),
            Time::from_hms(0, 0, 0).unwrap(),
        )),
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: Uuid::nil(),
    }
}

/// A second `Booking` referencing the second slot for chaining.
pub fn second_booking() -> Booking {
    Booking {
        id: second_booking_id(),
        sales_person_id: default_sales_person_id(),
        slot_id: second_slot_id(),
        calendar_week: 3,
        year: 2025,
        created: Some(PrimitiveDateTime::new(
            Date::from_calendar_date(2025, Month::January, 1).unwrap(),
            Time::from_hms(0, 0, 0).unwrap(),
        )),
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: Uuid::nil(),
    }
}

/// Build dependencies with or without permission to emulate the approach
/// in your booking tests. If you're not checking permissions here, you can ignore it.
pub fn build_dependencies() -> BlockServiceDependencies {
    // 1) booking_service mock
    let booking_service = MockBookingService::new();

    // 2) slot_service mock
    let slot_service = MockSlotService::new();

    // 3) sales_person_service mock
    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(default_sales_person()));

    // 4) transaction_dao mock
    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    let ical_service = MockIcalService::new();

    let shiftplan_service = MockShiftplanService::new();

    let mut clock_service = MockClockService::new();
    clock_service
        .expect_date_now()
        .returning(|| date!(2025 - 01 - 01));

    BlockServiceDependencies {
        booking_service,
        slot_service,
        sales_person_service,
        ical_service,
        clock_service,
        transaction_dao,
        shiftplan_service,
    }
}

/// Example test: no bookings means no blocks.
#[tokio::test]
async fn test_get_blocks_no_bookings() {
    let mut deps = build_dependencies();
    deps.booking_service
        .expect_get_for_week()
        .with(eq(3), eq(2025), always(), always())
        .returning(|_, _, _, _| Ok(vec![].into()));
    let service = deps.build_service();
    let result = service
        .get_blocks_for_sales_person_week(default_sales_person_id(), 2025, 3, ().auth(), None)
        .await;

    assert!(result.is_ok(), "Expected Ok result");
    let blocks = result.unwrap();
    assert!(blocks.is_empty(), "Expected no blocks if no bookings");
}

/// Example test: we have consecutive bookings that should merge into one block.
#[tokio::test]
async fn test_get_blocks_consecutive_bookings() {
    let mut deps = build_dependencies();

    // 1) Mock `get_for_week` to return our two test bookings (back-to-back).
    deps.booking_service
        .expect_get_for_week()
        .with(eq(3), eq(2025), always(), always())
        .returning(|_, _, _, _| Ok(vec![default_booking(), second_booking()].into()));

    // 2) Mock `get_slot` so the first booking references [9:00 - 10:00],
    //    the second references [10:00 - 11:00].
    deps.slot_service
        .expect_get_slot()
        .times(2)
        .returning(|slot_id, _, _| {
            if *slot_id == default_slot_id() {
                Ok(default_slot())
            } else {
                Ok(second_slot())
            }
        });

    let service = deps.build_service();
    let result = service
        .get_blocks_for_sales_person_week(default_sales_person_id(), 2025, 3, ().auth(), None)
        .await;

    assert!(result.is_ok(), "Expected Ok result");
    let blocks = result.unwrap();
    assert_eq!(blocks.len(), 1, "Expected exactly one merged block");

    // Check that the block covers 9:00-11:00
    let block = &blocks[0];
    assert_eq!(
        block.sales_person.as_ref().unwrap().id,
        default_sales_person_id()
    );
    assert_eq!(block.from, Time::from_hms(9, 0, 0).unwrap());
    assert_eq!(block.to, Time::from_hms(11, 0, 0).unwrap());
    assert_eq!(block.bookings.len(), 2, "Should merge both bookings");
    assert_eq!(block.slots.len(), 2, "Should merge both slots");
}

/// Example test: if user has no permission, we get forbidden.
#[tokio::test]
async fn test_get_blocks_forbidden() {
    let mut deps = build_dependencies();
    deps.booking_service
        .expect_get_for_week()
        .with(eq(3), eq(2025), always(), always())
        .returning(|_, _, _, _| Err(ServiceError::Forbidden));
    let service = deps.build_service();
    let result = service
        .get_blocks_for_sales_person_week(default_sales_person_id(), 2025, 3, ().auth(), None)
        .await;

    // The booking tests call `test_forbidden`, so do the same:
    test_forbidden(&result);
}

/// Example test: if we have two non-consecutive bookings, they become two blocks.
#[tokio::test]
async fn test_get_blocks_non_consecutive_bookings() {
    let mut deps = build_dependencies();

    // We'll create two bookings with big time gaps:
    //  Booking1 => Monday 09:00-10:00
    //  Booking2 => Monday 11:00-12:00
    let booking1 = Booking {
        id: default_booking_id(),
        sales_person_id: default_sales_person_id(),
        slot_id: default_slot_id(), // we'll say it's 09:00-10:00
        calendar_week: 3,
        year: 2025,
        created: None,
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: Uuid::nil(),
    };
    let booking2 = Booking {
        id: second_booking_id(),
        sales_person_id: default_sales_person_id(),
        slot_id: second_slot_id(), // we'll override to 11:00–12:00 below
        calendar_week: 3,
        year: 2025,
        created: None,
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: Uuid::nil(),
    };

    // Mock to return these two bookings
    deps.booking_service
        .expect_get_for_week()
        .with(eq(3), eq(2025), eq(().auth()), always())
        .returning(move |_, _, _, _| Ok(vec![booking1.clone(), booking2.clone()].into()));

    // Mock slot service so:
    //   default_slot_id() => 09:00–10:00
    //   second_slot_id() => 11:00–12:00
    let mut slot_service = MockSlotService::new();
    slot_service.expect_get_slot().returning(|slot_id, _, _| {
        if *slot_id == default_slot_id() {
            let mut s = default_slot();
            s.from = Time::from_hms(9, 0, 0).unwrap();
            s.to = Time::from_hms(10, 0, 0).unwrap();
            Ok(s)
        } else {
            let mut s = default_slot();
            s.id = second_slot_id();
            s.from = Time::from_hms(11, 0, 0).unwrap();
            s.to = Time::from_hms(12, 0, 0).unwrap();
            Ok(s)
        }
    });
    deps.slot_service = slot_service;

    let service = deps.build_service();
    let result = service
        .get_blocks_for_sales_person_week(default_sales_person_id(), 2025, 3, ().auth(), None)
        .await;

    assert!(result.is_ok(), "Expected Ok result");
    let blocks = result.unwrap();
    assert_eq!(
        blocks.len(),
        2,
        "Expected two separate blocks (non-consecutive)"
    );

    // First block: 09:00-10:00
    assert_eq!(blocks[0].from, Time::from_hms(9, 0, 0).unwrap());
    assert_eq!(blocks[0].to, Time::from_hms(10, 0, 0).unwrap());

    // Second block: 11:00-12:00
    assert_eq!(blocks[1].from, Time::from_hms(11, 0, 0).unwrap());
    assert_eq!(blocks[1].to, Time::from_hms(12, 0, 0).unwrap());
}

// If you have other edge cases (e.g., cross-day or cross-week merges),
// add more tests accordingly.
