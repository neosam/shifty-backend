use crate::test::error_test::*;
use dao::{MockTransaction, MockTransactionDao};
use service::{
    booking::{Booking, MockBookingService},
    permission::MockPermissionService,
    sales_person::{MockSalesPersonService, SalesPerson},
    shiftplan::ShiftplanViewService,
    shiftplan_catalog::{MockShiftplanService, Shiftplan},
    slot::{MockSlotService, Slot},
    special_days::{MockSpecialDayService, SpecialDay, SpecialDayType},
};
use shifty_utils::DayOfWeek;
use std::collections::HashMap;
use std::sync::Arc;
use time::{Date, Month, Time};
use uuid::{uuid, Uuid};

use crate::shiftplan::{build_shiftplan_day, ShiftplanViewServiceDeps, ShiftplanViewServiceImpl};

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
        shiftplan_id: None,
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

pub struct ShiftplanViewServiceDependencies {
    pub slot_service: MockSlotService,
    pub booking_service: MockBookingService,
    pub sales_person_service: MockSalesPersonService,
    pub special_day_service: MockSpecialDayService,
    pub shiftplan_service: MockShiftplanService,
    pub permission_service: MockPermissionService,
    pub transaction_dao: MockTransactionDao,
}
impl ShiftplanViewServiceDeps for ShiftplanViewServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type SlotService = MockSlotService;
    type BookingService = MockBookingService;
    type SalesPersonService = MockSalesPersonService;
    type SpecialDayService = MockSpecialDayService;
    type ShiftplanService = MockShiftplanService;
    type PermissionService = MockPermissionService;
    type TransactionDao = MockTransactionDao;
}

impl ShiftplanViewServiceDependencies {
    pub fn build_service(self) -> ShiftplanViewServiceImpl<ShiftplanViewServiceDependencies> {
        ShiftplanViewServiceImpl {
            slot_service: self.slot_service.into(),
            booking_service: self.booking_service.into(),
            sales_person_service: self.sales_person_service.into(),
            special_day_service: self.special_day_service.into(),
            shiftplan_service: self.shiftplan_service.into(),
            permission_service: self.permission_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

pub fn build_dependencies() -> ShiftplanViewServiceDependencies {
    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slots_for_week()
        .returning(|_, _, _, _, _| Ok(Arc::new([default_slot()])));

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

    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| Ok(Arc::new([])));

    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Err(service::ServiceError::Forbidden));

    let shiftplan_service = MockShiftplanService::new();

    ShiftplanViewServiceDependencies {
        slot_service,
        booking_service,
        sales_person_service,
        special_day_service,
        shiftplan_service,
        permission_service,
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

    let result = service.get_shiftplan_week(Uuid::nil(), 2024, 3, ().auth(), None).await;
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
        .expect_get_slots_for_week()
        .returning(|_, _, _, _, _| Err(service::ServiceError::Forbidden));

    // Set up booking service expectations since it gets called after slot service
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));

    let service = deps.build_service();
    let result = service.get_shiftplan_week(Uuid::nil(), 2024, 3, ().auth(), None).await;
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
    let result = service.get_shiftplan_week(Uuid::nil(), 2024, 0, ().auth(), None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_shiftplan_week_with_special_days() {
    let mut deps = build_dependencies();

    // Set up booking service expectations
    deps.booking_service.checkpoint();

    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));

    // Set up special days - a holiday on Monday and short day on Tuesday
    deps.special_day_service.checkpoint();

    deps.special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| {
            Ok(Arc::new([
                SpecialDay {
                    id: Uuid::new_v4(),
                    year: 2024,
                    calendar_week: 3,
                    day_of_week: DayOfWeek::Monday,
                    day_type: service::special_days::SpecialDayType::Holiday,
                    time_of_day: None,
                    created: None,
                    deleted: None,
                    version: Uuid::new_v4(),
                },
                SpecialDay {
                    id: Uuid::new_v4(),
                    year: 2024,
                    calendar_week: 3,
                    day_of_week: DayOfWeek::Tuesday,
                    day_type: service::special_days::SpecialDayType::ShortDay,
                    time_of_day: Some(Time::from_hms(14, 0, 0).unwrap()),
                    created: None,
                    deleted: None,
                    version: Uuid::new_v4(),
                },
            ]))
        });

    let service = deps.build_service();

    let result = service.get_shiftplan_week(Uuid::nil(), 2024, 3, ().auth(), None).await;
    assert!(result.is_ok());

    let shiftplan = result.unwrap();

    // Monday should have no slots due to holiday
    let monday = &shiftplan.days[0];
    assert!(matches!(monday.day_of_week, DayOfWeek::Monday));
    assert_eq!(monday.slots.len(), 0);

    // Tuesday should only have slots ending before 14:00
    let tuesday = &shiftplan.days[1];
    assert!(matches!(tuesday.day_of_week, DayOfWeek::Tuesday));
    assert!(tuesday
        .slots
        .iter()
        .all(|slot| slot.slot.to <= Time::from_hms(14, 0, 0).unwrap()));
}

// --- Unit tests for build_shiftplan_day ---

fn default_booking(slot_id: Uuid, sales_person_id: Uuid) -> Booking {
    Booking {
        id: Uuid::new_v4(),
        sales_person_id,
        slot_id,
        calendar_week: 3,
        year: 2024,
        created: None,
        deleted: None,
        created_by: Some("user1".into()),
        deleted_by: None,
        version: Uuid::new_v4(),
    }
}

fn slot_with_day_and_time(day: DayOfWeek, from_h: u8, to_h: u8) -> Slot {
    Slot {
        id: Uuid::new_v4(),
        day_of_week: day,
        from: Time::from_hms(from_h, 0, 0).unwrap(),
        to: Time::from_hms(to_h, 0, 0).unwrap(),
        min_resources: 1,
        valid_from: Date::from_calendar_date(2024, Month::January, 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: Uuid::new_v4(),
        shiftplan_id: None,
    }
}

#[test]
fn test_build_shiftplan_day_filters_by_day_and_assigns_bookings() {
    let monday_slot = slot_with_day_and_time(DayOfWeek::Monday, 9, 17);
    let tuesday_slot = slot_with_day_and_time(DayOfWeek::Tuesday, 9, 17);
    let sp = default_sales_person();
    let booking = default_booking(monday_slot.id, sp.id);

    let slots = vec![monday_slot.clone(), tuesday_slot];
    let bookings = vec![booking];
    let sales_persons = vec![sp];

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &slots,
        &bookings,
        &sales_persons,
        &[],
        None,
    )
    .unwrap();

    assert_eq!(result.day_of_week, DayOfWeek::Monday);
    assert_eq!(result.slots.len(), 1);
    assert_eq!(result.slots[0].slot.id, monday_slot.id);
    assert_eq!(result.slots[0].bookings.len(), 1);
}

#[test]
fn test_build_shiftplan_day_excludes_all_on_holiday() {
    let slot = slot_with_day_and_time(DayOfWeek::Monday, 9, 17);
    let sp = default_sales_person();
    let holiday = SpecialDay {
        id: Uuid::new_v4(),
        year: 2024,
        calendar_week: 3,
        day_of_week: DayOfWeek::Monday,
        day_type: SpecialDayType::Holiday,
        time_of_day: None,
        created: None,
        deleted: None,
        version: Uuid::new_v4(),
    };

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[slot],
        &[],
        &[sp],
        &[holiday],
        None,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 0);
}

#[test]
fn test_build_shiftplan_day_filters_short_day() {
    let early_slot = slot_with_day_and_time(DayOfWeek::Monday, 9, 12);
    let late_slot = slot_with_day_and_time(DayOfWeek::Monday, 14, 18);
    let sp = default_sales_person();
    let short_day = SpecialDay {
        id: Uuid::new_v4(),
        year: 2024,
        calendar_week: 3,
        day_of_week: DayOfWeek::Monday,
        day_type: SpecialDayType::ShortDay,
        time_of_day: Some(Time::from_hms(14, 0, 0).unwrap()),
        created: None,
        deleted: None,
        version: Uuid::new_v4(),
    };

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[early_slot.clone(), late_slot],
        &[],
        &[sp],
        &[short_day],
        None,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 1);
    assert_eq!(result.slots[0].slot.id, early_slot.id);
}

#[test]
fn test_build_shiftplan_day_self_added_with_assignments() {
    let slot = slot_with_day_and_time(DayOfWeek::Monday, 9, 17);
    let sp = default_sales_person();
    let booking = default_booking(slot.id, sp.id);

    let mut assignments = HashMap::new();
    assignments.insert(sp.id, Arc::<str>::from("user1"));

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[slot],
        &[booking],
        &[sp],
        &[],
        Some(&assignments),
    )
    .unwrap();

    assert_eq!(result.slots[0].bookings[0].self_added, Some(true));
}

#[test]
fn test_build_shiftplan_day_self_added_none_without_assignments() {
    let slot = slot_with_day_and_time(DayOfWeek::Monday, 9, 17);
    let sp = default_sales_person();
    let booking = default_booking(slot.id, sp.id);

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[slot],
        &[booking],
        &[sp],
        &[],
        None,
    )
    .unwrap();

    assert_eq!(result.slots[0].bookings[0].self_added, None);
}

#[test]
fn test_build_shiftplan_day_sorts_slots_by_from_time() {
    let late_slot = slot_with_day_and_time(DayOfWeek::Monday, 14, 18);
    let early_slot = slot_with_day_and_time(DayOfWeek::Monday, 8, 12);
    let sp = default_sales_person();

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[late_slot.clone(), early_slot.clone()],
        &[],
        &[sp],
        &[],
        None,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 2);
    assert_eq!(result.slots[0].slot.id, early_slot.id);
    assert_eq!(result.slots[1].slot.id, late_slot.id);
}

// --- Service tests for get_shiftplan_day ---

fn default_shiftplan(name: &str) -> Shiftplan {
    Shiftplan {
        id: Uuid::new_v4(),
        name: name.into(),
        is_planning: false,
        deleted: None,
        version: Uuid::new_v4(),
    }
}

#[tokio::test]
async fn test_get_shiftplan_day_aggregates_all_plans() {
    let plan_a = default_shiftplan("Morning");
    let plan_b = default_shiftplan("Evening");
    let plan_a_id = plan_a.id;
    let plan_b_id = plan_b.id;

    let slot_a = Slot {
        shiftplan_id: Some(plan_a_id),
        ..slot_with_day_and_time(DayOfWeek::Monday, 8, 12)
    };
    let slot_b = Slot {
        shiftplan_id: Some(plan_b_id),
        ..slot_with_day_and_time(DayOfWeek::Monday, 14, 18)
    };
    let slot_a_clone = slot_a.clone();
    let slot_b_clone = slot_b.clone();

    let mut deps = build_dependencies();

    // Override slot service to return different slots per plan
    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slots_for_week()
        .returning(move |_, _, shiftplan_id, _, _| {
            if shiftplan_id == plan_a_id {
                Ok(Arc::new([slot_a_clone.clone()]))
            } else if shiftplan_id == plan_b_id {
                Ok(Arc::new([slot_b_clone.clone()]))
            } else {
                Ok(Arc::new([]))
            }
        });

    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));

    let plans = Arc::new([plan_a, plan_b]);
    deps.shiftplan_service
        .expect_get_all()
        .returning(move |_, _| Ok(plans.clone()));

    let service = deps.build_service();
    let result = service
        .get_shiftplan_day(2024, 3, DayOfWeek::Monday, ().auth(), None)
        .await;

    assert!(result.is_ok());
    let aggregate = result.unwrap();
    assert_eq!(aggregate.year, 2024);
    assert_eq!(aggregate.calendar_week, 3);
    assert_eq!(aggregate.day_of_week, DayOfWeek::Monday);
    assert_eq!(aggregate.plans.len(), 2);
    assert_eq!(aggregate.plans[0].slots.len(), 1);
    assert_eq!(aggregate.plans[0].slots[0].slot.id, slot_a.id);
    assert_eq!(aggregate.plans[1].slots.len(), 1);
    assert_eq!(aggregate.plans[1].slots[0].slot.id, slot_b.id);
}

#[tokio::test]
async fn test_get_shiftplan_day_invalid_week() {
    let mut deps = build_dependencies();
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));
    let service = deps.build_service();

    let result = service
        .get_shiftplan_day(2024, 0, DayOfWeek::Monday, ().auth(), None)
        .await;
    assert!(result.is_err());
}
