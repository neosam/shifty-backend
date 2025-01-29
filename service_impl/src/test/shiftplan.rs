use crate::test::error_test::*;
use dao::{MockTransaction, MockTransactionDao};
use service::{
    booking::MockBookingService,
    sales_person::{MockSalesPersonService, SalesPerson},
    shiftplan::ShiftplanService,
    slot::{DayOfWeek, MockSlotService, Slot},
};
use std::sync::Arc;
use time::{Date, Month, Time};
use uuid::{uuid, Uuid};

use crate::shiftplan::{ShiftplanServiceDeps, ShiftplanServiceImpl};

pub fn default_slot_id() -> Uuid {
    uuid!("7A7FF57A-782B-4C2E-A68B-4E2D81D79380")
}

pub fn default_sales_person_id() -> Uuid {
    uuid!("04215DFE-13C4-413C-8C66-77AC741BB5F0")
}

pub fn default_slot_version() -> Uuid {
    uuid!("F79C462A-8D4E-42E1-8171-DB4DBD019E50")
}

pub fn default_slot() -> Slot {
    Slot {
        id: default_slot_id(),
        day_of_week: DayOfWeek::Monday,
        from: Time::from_hms(9, 0, 0).unwrap(),
        to: Time::from_hms(17, 0, 0).unwrap(),
        min_resources: 1,
        valid_from: Date::from_calendar_date(2024, Month::January, 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: default_slot_version(),
    }
}

pub fn default_sales_person() -> SalesPerson {
    SalesPerson {
        id: default_sales_person_id(),
        name: "Test Sales Person".into(),
        background_color: "#FF0000".into(),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::new_v4(),
    }
}

pub struct ShiftplanServiceDependencies {
    pub slot_service: MockSlotService,
    pub booking_service: MockBookingService,
    pub sales_person_service: MockSalesPersonService,
    pub transaction_dao: MockTransactionDao,
}

impl ShiftplanServiceDeps for ShiftplanServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type SlotService = MockSlotService;
    type BookingService = MockBookingService;
    type SalesPersonService = MockSalesPersonService;
    type TransactionDao = MockTransactionDao;
}

impl ShiftplanServiceDependencies {
    pub fn build_service(self) -> ShiftplanServiceImpl<ShiftplanServiceDependencies> {
        ShiftplanServiceImpl {
            slot_service: self.slot_service.into(),
            booking_service: self.booking_service.into(),
            sales_person_service: self.sales_person_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

pub fn build_dependencies() -> ShiftplanServiceDependencies {
    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slots()
        .returning(|_, _| Ok(Arc::new([default_slot()])));

    let booking_service = MockBookingService::new();

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::new([default_sales_person()])));

    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    ShiftplanServiceDependencies {
        slot_service,
        booking_service,
        sales_person_service,
        transaction_dao,
    }
}

#[tokio::test]
async fn test_get_shiftplan_week() {
    let mut deps = build_dependencies();

    // Set up booking service expectations
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));

    let deps = deps;
    let service = deps.build_service();

    let result = service.get_shiftplan_week(2024, 3, ().auth(), None).await;
    assert!(result.is_ok());

    let shiftplan = result.unwrap();
    assert_eq!(shiftplan.year, 2024);
    assert_eq!(shiftplan.calendar_week, 3);
    assert_eq!(shiftplan.days.len(), 7);

    // Verify first day (Monday)
    let monday = &shiftplan.days[0];
    assert!(matches!(monday.day_of_week, DayOfWeek::Monday));
    assert_eq!(monday.slots.len(), 1);

    // Verify slot details
    let slot = &monday.slots[0];
    assert_eq!(slot.slot, default_slot());
    assert!(slot.bookings.is_empty());
}

#[tokio::test]
async fn test_get_shiftplan_week_no_permission() {
    let mut deps = build_dependencies();

    // Override slot service to return forbidden error
    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slots()
        .returning(|_, _| Err(service::ServiceError::Forbidden));

    // Set up booking service expectations since it gets called after slot service
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));

    let service = deps.build_service();
    let result = service.get_shiftplan_week(2024, 3, ().auth(), None).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_get_shiftplan_week_invalid_week() {
    let mut deps = build_dependencies();
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));
    let service = deps.build_service();

    // Week 0 is invalid
    let result = service.get_shiftplan_week(2024, 0, ().auth(), None).await;
    assert!(result.is_err());
}
