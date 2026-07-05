use crate::test::error_test::*;
use dao::{MockTransaction, MockTransactionDao};
use service::{
    absence::{AbsencePeriod, MockAbsenceService},
    booking::{Booking, MockBookingService},
    permission::MockPermissionService,
    sales_person::{MockSalesPersonService, SalesPerson},
    sales_person_unavailable::{MockSalesPersonUnavailableService, SalesPersonUnavailable},
    shiftplan::ShiftplanViewService,
    shiftplan_catalog::{MockShiftplanService, Shiftplan},
    slot::{MockSlotService, Slot},
    special_days::{MockSpecialDayService, SpecialDay, SpecialDayType},
    toggle::MockToggleService,
};
use shifty_utils::DayOfWeek;
use std::collections::{HashMap, HashSet};
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
        // Phase 5 (D-09): default fixture has no paid-employee limit. Tests
        // that exercise the Phase-5 paid-cap path build slot variants via
        // `Slot { max_paid_employees: Some(N), ..default_slot() }`.
        max_paid_employees: None,
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
    // Phase-3 per-sales-person-Pfade (Plan 03-04 / D-Phase3-09):
    pub absence_service: MockAbsenceService,
    pub sales_person_unavailable_service: MockSalesPersonUnavailableService,
    // Phase 51 (D-51-07): Stichtag-Gate für ShortDay-Slot-Kürzung.
    pub toggle_service: MockToggleService,
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
    type AbsenceService = MockAbsenceService;
    type SalesPersonUnavailableService = MockSalesPersonUnavailableService;
    type ToggleService = MockToggleService;
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
            absence_service: self.absence_service.into(),
            sales_person_unavailable_service: self.sales_person_unavailable_service.into(),
            toggle_service: self.toggle_service.into(),
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
    // D-24-03 fix: get_all_paid is called with Authentication::Full in all
    // shiftplan view service methods to obtain the ungated paid-person set.
    // Default: return the default_sales_person (who is paid) so tests that
    // don't override this still behave consistently.
    sales_person_service
        .expect_get_all_paid()
        .returning(|_, _| Ok(Arc::new([default_sales_person()])));
    // Phase-3: verify_user_is_sales_person läuft per `tokio::join!` parallel
    // zur HR-Probe — Default Forbidden, Tests die HR ∨ self prüfen müssen
    // diesen Mock NICHT überschreiben (HR-Grant trifft via .or() den Erfolg);
    // forbidden-Tests können beide Probes lokal auf Forbidden setzen.
    sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(service::ServiceError::Forbidden));
    // SHIFTPLANNER-Privileg-Check (in get_shiftplan_*-Bodies) löst, wenn
    // grant'd, einen `get_all_user_assignments`-Call aus — Default leere
    // HashMap, damit Tests, die HR auf Ok setzen (und damit SHIFTPLANNER
    // implizit auch grant'd, weil die Mock-`expect_check_permission` keinen
    // Privilege-Filter setzt), nicht panicken.
    sales_person_service
        .expect_get_all_user_assignments()
        .returning(|_, _| Ok(HashMap::new()));

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

    // Phase-3-Defaults: leere AbsencePeriods + leere ManualUnavailables.
    // Globalsicht-Tests rufen diese Services nie; per-sales-person-Tests
    // überschreiben sie lokal.
    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_by_sales_person()
        .returning(|_, _, _| Ok(Arc::from(Vec::<AbsencePeriod>::new())));

    let mut sales_person_unavailable_service = MockSalesPersonUnavailableService::new();
    sales_person_unavailable_service
        .expect_get_by_week_for_sales_person()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<SalesPersonUnavailable>::new())));

    // Phase 51 (D-51-07) default: Toggle nicht gesetzt → `parse_active_from`
    // liefert `None` → Legacy-Verhalten (nie clippen). Tests, die das Gate
    // aktivieren wollen (`test_get_shiftplan_week_with_special_days` +
    // die neuen `effective_to`-Tests), überschreiben diesen Mock lokal.
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));

    ShiftplanViewServiceDependencies {
        slot_service,
        booking_service,
        sales_person_service,
        special_day_service,
        shiftplan_service,
        permission_service,
        transaction_dao,
        absence_service,
        sales_person_unavailable_service,
        toggle_service,
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
    // Phase 51 (D-51-06 Chain B): Bug-Fix + Feature — der Test verifizierte
    // vorher das Filter-Anti-Pattern (`slot.to > cutoff → continue`) und ist
    // jetzt auf die Clip-Semantik umgestellt. Zusätzlich wird das Stichtag-Gate
    // (D-51-07) aktiviert, damit die Kürzung überhaupt greift.
    let mut deps = build_dependencies();

    // Explizite Tuesday-Fixture: ein überlappender Slot (12:00–15:00), ein
    // vollständig hinter Cutoff liegender Slot (15:00–17:00) sowie ein
    // Monday-Slot (der wegen des Holiday-Special-Day wegfällt).
    let monday_slot = slot_with_day_and_time(DayOfWeek::Monday, 9, 17);
    let tuesday_overlap = slot_with_day_and_time(DayOfWeek::Tuesday, 12, 15);
    let tuesday_after = slot_with_day_and_time(DayOfWeek::Tuesday, 15, 17);
    let tuesday_overlap_id = tuesday_overlap.id;

    deps.slot_service.checkpoint();
    let slot_fixture: Arc<[Slot]> = Arc::from(vec![
        monday_slot,
        tuesday_overlap,
        tuesday_after,
    ]);
    deps.slot_service
        .expect_get_slots_for_week()
        .returning(move |_, _, _, _, _| Ok(slot_fixture.clone()));

    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));

    // Special days: holiday on Monday, short day (cutoff 14:00) on Tuesday.
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

    // Stichtag-Gate aktivieren: Stichtag lange vor Test-Woche 2024-W3
    // (2024-01-15 .. 2024-01-21), damit `should_clip` `true` liefert.
    deps.toggle_service.checkpoint();
    deps.toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(Some("2020-01-01".into())));

    let service = deps.build_service();

    let result = service.get_shiftplan_week(Uuid::nil(), 2024, 3, ().auth(), None).await;
    assert!(result.is_ok());

    let shiftplan = result.unwrap();

    // Monday should have no slots due to holiday.
    let monday = &shiftplan.days[0];
    assert!(matches!(monday.day_of_week, DayOfWeek::Monday));
    assert_eq!(monday.slots.len(), 0);

    // Tuesday: der 15:00–17:00-Slot fehlt (D-04 Zeile 3, komplett hinter
    // Cutoff). Der überlappende 12:00–15:00-Slot wurde geclippt: `slot.to`
    // bleibt roh bei 15:00 (D-51-09), `effective_to == 14:00` (Cutoff).
    let tuesday = &shiftplan.days[1];
    assert!(matches!(tuesday.day_of_week, DayOfWeek::Tuesday));
    assert_eq!(
        tuesday.slots.len(),
        1,
        "expected exactly the overlapping slot (12:00–15:00), got {} slots",
        tuesday.slots.len()
    );
    let clipped = &tuesday.slots[0];
    assert_eq!(clipped.slot.id, tuesday_overlap_id);
    // D-51-09: `slot.to` bleibt roh (bidirektionale DTO-Regel).
    assert_eq!(
        clipped.slot.to,
        Time::from_hms(15, 0, 0).unwrap(),
        "slot.to must stay raw (D-51-09)"
    );
    // D-04 Zeile 4: `effective_to` == Cutoff (14:00).
    assert_eq!(
        clipped.effective_to,
        Time::from_hms(14, 0, 0).unwrap(),
        "effective_to must equal the ShortDay cutoff (D-04 Zeile 4)"
    );
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
        // Phase 5 (D-09): default fixture has no paid-employee limit.
        max_paid_employees: None,
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

    let paid_ids: HashSet<Uuid> = sales_persons
        .iter()
        .filter(|sp| sp.is_paid.unwrap_or(false))
        .map(|sp| sp.id)
        .collect();

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &slots,
        &bookings,
        &sales_persons,
        &[],
        None,
        &paid_ids,
        2024,
        3,
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

    let paid_ids: HashSet<Uuid> = HashSet::new();

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[slot],
        &[],
        &[sp],
        &[holiday],
        None,
        &paid_ids,
        2024,
        3,
        None,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 0);
}

#[test]
fn test_build_shiftplan_day_filters_short_day() {
    // Phase 51 (D-51-07): mit aktivem Stichtag-Gate greift die ShortDay-
    // Kürzung. Der Late-Slot (14:00–18:00) liegt komplett hinter dem Cutoff
    // (14:00) → wird aus dem Vec weggelassen (D-04 Zeile 3). Der Early-Slot
    // (09:00–12:00) endet weit vor dem Cutoff → unverändert (D-04 Zeile 1).
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

    let paid_ids: HashSet<Uuid> = HashSet::new();

    // active_from = Stichtag lange vor 2024-W3 → Gate aktiv.
    let active_from = Some(Date::from_calendar_date(2020, Month::January, 1).unwrap());

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[early_slot.clone(), late_slot],
        &[],
        &[sp],
        &[short_day],
        None,
        &paid_ids,
        2024,
        3,
        active_from,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 1);
    assert_eq!(result.slots[0].slot.id, early_slot.id);
    // Early slot endet vor dem Cutoff → effective_to bleibt raw.
    assert_eq!(
        result.slots[0].effective_to,
        Time::from_hms(12, 0, 0).unwrap()
    );
}

#[test]
fn test_build_shiftplan_day_self_added_with_assignments() {
    let slot = slot_with_day_and_time(DayOfWeek::Monday, 9, 17);
    let sp = default_sales_person();
    let booking = default_booking(slot.id, sp.id);

    let mut assignments = HashMap::new();
    assignments.insert(sp.id, Arc::<str>::from("user1"));

    let paid_ids: HashSet<Uuid> = HashSet::new();

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[slot],
        &[booking],
        &[sp],
        &[],
        Some(&assignments),
        &paid_ids,
        2024,
        3,
        None,
    )
    .unwrap();

    assert_eq!(result.slots[0].bookings[0].self_added, Some(true));
}

#[test]
fn test_build_shiftplan_day_self_added_none_without_assignments() {
    let slot = slot_with_day_and_time(DayOfWeek::Monday, 9, 17);
    let sp = default_sales_person();
    let booking = default_booking(slot.id, sp.id);

    let paid_ids: HashSet<Uuid> = HashSet::new();

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[slot],
        &[booking],
        &[sp],
        &[],
        None,
        &paid_ids,
        2024,
        3,
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

    let paid_ids: HashSet<Uuid> = HashSet::new();

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[late_slot.clone(), early_slot.clone()],
        &[],
        &[sp],
        &[],
        None,
        &paid_ids,
        2024,
        3,
        None,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 2);
    assert_eq!(result.slots[0].slot.id, early_slot.id);
    assert_eq!(result.slots[1].slot.id, late_slot.id);
}

// --- Phase-5 read-aggregation tests for current_paid_count
//     (D-04: count where sales_person.is_paid == true; D-05: absence
//     status irrelevant; D-09: always populated even without limit). ---

fn unpaid_sales_person(id: Uuid, name: &str) -> SalesPerson {
    SalesPerson {
        id,
        name: name.into(),
        background_color: "#00FF00".into(),
        is_paid: Some(false),
        inactive: false,
        deleted: None,
        version: Uuid::new_v4(),
    }
}

fn paid_sales_person(id: Uuid, name: &str) -> SalesPerson {
    SalesPerson {
        id,
        name: name.into(),
        background_color: "#0000FF".into(),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::new_v4(),
    }
}

#[test]
fn test_shiftplan_week_emits_current_paid_count_zero_when_no_paid() {
    // D-04: bookings exist, but no booked sales_person has is_paid == true →
    // current_paid_count == 0.
    let slot = slot_with_day_and_time(DayOfWeek::Monday, 9, 17);
    let sp_a = unpaid_sales_person(Uuid::new_v4(), "A");
    let sp_b = unpaid_sales_person(Uuid::new_v4(), "B");
    let booking_a = default_booking(slot.id, sp_a.id);
    let booking_b = default_booking(slot.id, sp_b.id);

    // No one is paid — empty set.
    let paid_ids: HashSet<Uuid> = HashSet::new();

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[slot],
        &[booking_a, booking_b],
        &[sp_a, sp_b],
        &[],
        None,
        &paid_ids,
        2024,
        3,
        None,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 1);
    assert_eq!(result.slots[0].bookings.len(), 2);
    assert_eq!(result.slots[0].current_paid_count, 0);
}

#[test]
fn test_shiftplan_week_emits_current_paid_count_mixed() {
    // D-04: mix of paid (2) and unpaid (1) bookings → count = 2.
    let slot = slot_with_day_and_time(DayOfWeek::Monday, 9, 17);
    let paid_a = paid_sales_person(Uuid::new_v4(), "Paid A");
    let paid_b = paid_sales_person(Uuid::new_v4(), "Paid B");
    let unpaid = unpaid_sales_person(Uuid::new_v4(), "Unpaid");
    let booking_paid_a = default_booking(slot.id, paid_a.id);
    let booking_paid_b = default_booking(slot.id, paid_b.id);
    let booking_unpaid = default_booking(slot.id, unpaid.id);

    let paid_ids: HashSet<Uuid> = [paid_a.id, paid_b.id].into_iter().collect();

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[slot],
        &[booking_paid_a, booking_paid_b, booking_unpaid],
        &[paid_a, paid_b, unpaid],
        &[],
        None,
        &paid_ids,
        2024,
        3,
        None,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 1);
    assert_eq!(result.slots[0].bookings.len(), 3);
    assert_eq!(result.slots[0].current_paid_count, 2);
}

#[test]
fn test_shiftplan_week_emits_current_paid_count_with_no_limit() {
    // D-09: current_paid_count is always populated regardless of whether
    // slot.max_paid_employees is configured. Slot here has no limit
    // (None), one paid booking → count == 1.
    let slot = Slot {
        max_paid_employees: None,
        ..slot_with_day_and_time(DayOfWeek::Monday, 9, 17)
    };
    let paid = paid_sales_person(Uuid::new_v4(), "Paid");
    let booking = default_booking(slot.id, paid.id);

    let paid_ids: HashSet<Uuid> = [paid.id].into_iter().collect();

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[slot],
        &[booking],
        &[paid],
        &[],
        None,
        &paid_ids,
        2024,
        3,
        None,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 1);
    assert!(result.slots[0].slot.max_paid_employees.is_none());
    assert_eq!(result.slots[0].current_paid_count, 1);
}

#[test]
fn test_shiftplan_week_paid_in_absence_still_counts() {
    // D-05: absence status of the booked person is irrelevant — anyone
    // booked counts. `build_shiftplan_day` does not consult the absence
    // service at all; this test proves the count purely depends on
    // `sales_person.is_paid` and the booking's existence. The presence of
    // an absence period in the test fixture is informational — it must NOT
    // suppress the count.
    let slot = Slot {
        max_paid_employees: Some(1),
        ..slot_with_day_and_time(DayOfWeek::Monday, 9, 17)
    };
    let paid = paid_sales_person(Uuid::new_v4(), "Paid In Absence");
    let booking = default_booking(slot.id, paid.id);

    // Construct an absence period for `paid` that overlaps the booking
    // week. `build_shiftplan_day` ignores the absence stream by design
    // (D-05) — it is not even an input parameter.
    let _absence_overlap = AbsencePeriod {
        id: uuid!("AB000000-0000-0000-0000-000000000099"),
        sales_person_id: paid.id,
        category: AbsenceCategory::Vacation,
        from_date: date!(2024 - 01 - 15),
        to_date: date!(2024 - 01 - 19),
        description: "Booked-while-on-vacation".into(),
        created: Some(datetime!(2024 - 01 - 01 12:00:00)),
        deleted: None,
        version: uuid!("AB000000-0000-0000-0000-000000000098"),
        day_fraction: DayFraction::Full,
    };

    let paid_ids: HashSet<Uuid> = [paid.id].into_iter().collect();

    let result = build_shiftplan_day(
        DayOfWeek::Monday,
        &[slot],
        &[booking],
        &[paid],
        &[],
        None,
        &paid_ids,
        2024,
        3,
        None,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 1);
    // D-05 enforced: the paid booking is counted even though the person
    // is in an active absence period.
    assert_eq!(result.slots[0].current_paid_count, 1);
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

// ---- Phase-3 per-sales-person-Tests (Plan 03-04 Wave 3) ----

use service::absence::{AbsenceCategory, DayFraction};
use service::shiftplan::UnavailabilityMarker;
use time::macros::{date, datetime};

/// 2024-W3 Monday — `time::Date::from_iso_week_date(2024, 3, Monday)` =
/// 2024-01-15.
fn absence_period_w3_monday() -> AbsencePeriod {
    AbsencePeriod {
        id: uuid!("AB000000-0000-0000-0000-000000000001"),
        sales_person_id: default_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: date!(2024 - 01 - 15),
        to_date: date!(2024 - 01 - 19),
        description: "Urlaub".into(),
        created: Some(datetime!(2024 - 01 - 01 12:00:00)),
        deleted: None,
        version: uuid!("CC000000-0000-0000-0000-000000000099"),
        day_fraction: DayFraction::Full,
    }
}

fn manual_unavailable_w3_monday() -> SalesPersonUnavailable {
    SalesPersonUnavailable {
        id: uuid!("CC000000-0000-0000-0000-000000000010"),
        sales_person_id: default_sales_person_id(),
        year: 2024,
        calendar_week: 3,
        day_of_week: DayOfWeek::Monday,
        created: Some(datetime!(2024 - 01 - 01 12:00:00)),
        deleted: None,
        version: uuid!("CC000000-0000-0000-0000-000000000100"),
    }
}

/// Test 2: AbsencePeriod-Mock-Hit on Monday → Monday.unavailable ==
/// Some(AbsencePeriod{..}).
#[tokio::test]
async fn test_get_shiftplan_week_for_sales_person_marker_absence_only() {
    let mut deps = build_dependencies();
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));

    // HR grant via permission_service.
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    deps.absence_service.checkpoint();
    deps.absence_service
        .expect_find_by_sales_person()
        .returning(|_, _, _| Ok(Arc::from(vec![absence_period_w3_monday()])));

    let service = deps.build_service();
    let result = service
        .get_shiftplan_week_for_sales_person(
            Uuid::nil(),
            2024,
            3,
            default_sales_person_id(),
            ().auth(),
            None,
        )
        .await
        .expect("get_shiftplan_week_for_sales_person should succeed");

    let monday = &result.days[0];
    assert_eq!(monday.day_of_week, DayOfWeek::Monday);
    match &monday.unavailable {
        Some(UnavailabilityMarker::AbsencePeriod {
            absence_id,
            category,
        }) => {
            assert_eq!(*absence_id, absence_period_w3_monday().id);
            assert_eq!(*category, AbsenceCategory::Vacation);
        }
        other => panic!("expected AbsencePeriod marker, got {other:?}"),
    }
    // Andere Tage haben keinen Marker (Tuesday-Friday auch von der Range
    // umfasst, aber nicht Sa/So).
    let saturday = &result.days[5];
    let sunday = &result.days[6];
    assert!(saturday.unavailable.is_none());
    assert!(sunday.unavailable.is_none());
}

/// Plan 03-06 Test 2b: ManualUnavailable-only on Monday → Monday.unavailable
/// == Some(ManualUnavailable). Komplementär zu `marker_absence_only`.
#[tokio::test]
async fn test_get_shiftplan_week_for_sales_person_marker_manual_only() {
    let mut deps = build_dependencies();
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));

    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    deps.sales_person_unavailable_service.checkpoint();
    deps.sales_person_unavailable_service
        .expect_get_by_week_for_sales_person()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![manual_unavailable_w3_monday()])));

    let service = deps.build_service();
    let result = service
        .get_shiftplan_week_for_sales_person(
            Uuid::nil(),
            2024,
            3,
            default_sales_person_id(),
            ().auth(),
            None,
        )
        .await
        .expect("get_shiftplan_week_for_sales_person should succeed");

    let monday = &result.days[0];
    match &monday.unavailable {
        Some(UnavailabilityMarker::ManualUnavailable) => {}
        other => panic!("expected ManualUnavailable marker, got {other:?}"),
    }
    // Tuesday-Sunday haben keine Markierung.
    for d in result.days.iter().skip(1) {
        assert!(
            d.unavailable.is_none(),
            "expected no marker on {:?}, got {:?}",
            d.day_of_week,
            d.unavailable
        );
    }
}

/// Test 3: Beide Quellen aktiv auf Monday → UnavailabilityMarker::Both
/// (D-Phase3-10).
#[tokio::test]
async fn test_get_shiftplan_week_for_sales_person_marker_both() {
    let mut deps = build_dependencies();
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));

    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    deps.absence_service.checkpoint();
    deps.absence_service
        .expect_find_by_sales_person()
        .returning(|_, _, _| Ok(Arc::from(vec![absence_period_w3_monday()])));

    deps.sales_person_unavailable_service.checkpoint();
    deps.sales_person_unavailable_service
        .expect_get_by_week_for_sales_person()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![manual_unavailable_w3_monday()])));

    let service = deps.build_service();
    let result = service
        .get_shiftplan_week_for_sales_person(
            Uuid::nil(),
            2024,
            3,
            default_sales_person_id(),
            ().auth(),
            None,
        )
        .await
        .expect("get_shiftplan_week_for_sales_person should succeed");

    let monday = &result.days[0];
    match &monday.unavailable {
        Some(UnavailabilityMarker::Both {
            absence_id,
            category,
        }) => {
            assert_eq!(*absence_id, absence_period_w3_monday().id);
            assert_eq!(*category, AbsenceCategory::Vacation);
        }
        other => panic!("expected Both marker, got {other:?}"),
    }
}

/// Test 4: Soft-deleted AbsencePeriod (`deleted.is_some()`) wird NICHT als
/// Marker gesetzt (Pitfall-1 / SC4).
#[tokio::test]
async fn test_get_shiftplan_week_for_sales_person_softdeleted_absence_no_marker() {
    let mut deps = build_dependencies();
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));

    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    deps.absence_service.checkpoint();
    deps.absence_service
        .expect_find_by_sales_person()
        .returning(|_, _, _| {
            Ok(Arc::from(vec![AbsencePeriod {
                deleted: Some(datetime!(2024 - 01 - 02 09:00:00)),
                ..absence_period_w3_monday()
            }]))
        });

    let service = deps.build_service();
    let result = service
        .get_shiftplan_week_for_sales_person(
            Uuid::nil(),
            2024,
            3,
            default_sales_person_id(),
            ().auth(),
            None,
        )
        .await
        .expect("get_shiftplan_week_for_sales_person should succeed");

    for day in result.days.iter() {
        assert!(
            day.unavailable.is_none(),
            "soft-deleted AbsencePeriod must not produce marker, got {:?} on {:?}",
            day.unavailable,
            day.day_of_week
        );
    }
}

#[tokio::test]
async fn test_get_shiftplan_week_for_sales_person_forbidden() {
    // permission default Forbidden + verify default unset → Mock returns
    // Err on verify_user_is_sales_person.
    let mut deps = build_dependencies();
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::new([default_sales_person()])));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(service::ServiceError::Forbidden));

    let service = deps.build_service();
    let result = service
        .get_shiftplan_week_for_sales_person(
            Uuid::nil(),
            2024,
            3,
            default_sales_person_id(),
            ().auth(),
            None,
        )
        .await;
    test_forbidden(&result);
}

/// Regression test for D-24-03: a non-HR caller sees `is_paid = None` on all
/// SalesPerson records returned by `get_all`, but `current_paid_count` must
/// still be > 0 because the service fetches the paid-person set via the
/// ungated `get_all_paid(Authentication::Full, ...)`.
///
/// **Fails against old code** (where count used `sb.sales_person.is_paid` →
/// always 0 for non-HR). **Passes with the fix** (count uses
/// `paid_sales_person_ids.contains(id)` from `get_all_paid`).
#[tokio::test]
async fn test_current_paid_count_correct_for_non_hr_caller() {
    // Slot with max_paid_employees configured so the paid-cap path is
    // clearly exercised.
    let slot_id = uuid!("AAAAAAAA-0000-0000-0000-000000000001");
    let paid_sp_id = uuid!("BBBBBBBB-0000-0000-0000-000000000001");
    let unpaid_sp_id = uuid!("CCCCCCCC-0000-0000-0000-000000000001");

    let slot = Slot {
        id: slot_id,
        day_of_week: DayOfWeek::Monday,
        from: time::Time::from_hms(9, 0, 0).unwrap(),
        to: time::Time::from_hms(17, 0, 0).unwrap(),
        min_resources: 1,
        max_paid_employees: Some(1),
        valid_from: Date::from_calendar_date(2024, Month::January, 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: Uuid::new_v4(),
        shiftplan_id: None,
    };

    // The paid sales person — as returned by `get_all` for a NON-HR caller
    // `is_paid` is scrubbed to None.
    let paid_sp_gated = SalesPerson {
        id: paid_sp_id,
        name: "Paid Person".into(),
        background_color: "#0000FF".into(),
        is_paid: None, // scrubbed — simulates non-HR response
        inactive: false,
        deleted: None,
        version: Uuid::new_v4(),
    };
    // The unpaid sales person — also has is_paid: None for non-HR callers.
    let unpaid_sp_gated = SalesPerson {
        id: unpaid_sp_id,
        name: "Unpaid Person".into(),
        background_color: "#00FF00".into(),
        is_paid: None, // scrubbed
        inactive: false,
        deleted: None,
        version: Uuid::new_v4(),
    };
    // What `get_all_paid` returns via Authentication::Full — only the truly
    // paid person, with is_paid visible (ungated path).
    let paid_sp_ungated = SalesPerson {
        is_paid: Some(true),
        ..paid_sp_gated.clone()
    };

    // Two bookings: one paid (paid_sp_id), one unpaid (unpaid_sp_id).
    let booking_paid = Booking {
        id: uuid!("DDDDDDDD-0000-0000-0000-000000000001"),
        sales_person_id: paid_sp_id,
        slot_id,
        calendar_week: 3,
        year: 2024,
        created: None,
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: Uuid::new_v4(),
    };
    let booking_unpaid = Booking {
        id: uuid!("DDDDDDDD-0000-0000-0000-000000000002"),
        sales_person_id: unpaid_sp_id,
        slot_id,
        calendar_week: 3,
        year: 2024,
        created: None,
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: Uuid::new_v4(),
    };

    let mut deps = build_dependencies();

    // Slot service returns our slot.
    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slots_for_week()
        .returning(move |_, _, _, _, _| Ok(Arc::new([slot.clone()])));

    // Booking service returns both bookings.
    deps.booking_service
        .expect_get_for_week()
        .returning(move |_, _, _, _| Ok(Arc::new([booking_paid.clone(), booking_unpaid.clone()])));

    // Simulated non-HR get_all: is_paid is scrubbed (None) on both persons.
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_get_all()
        .returning(move |_, _| {
            Ok(Arc::new([paid_sp_gated.clone(), unpaid_sp_gated.clone()]))
        });
    // get_all_paid via Authentication::Full returns only the actually-paid
    // person, with is_paid visible.
    deps.sales_person_service
        .expect_get_all_paid()
        .returning(move |_, _| Ok(Arc::new([paid_sp_ungated.clone()])));
    deps.sales_person_service
        .expect_get_all_user_assignments()
        .returning(|_, _| Ok(HashMap::new()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(service::ServiceError::Forbidden));

    let service = deps.build_service();
    let result = service
        .get_shiftplan_week(Uuid::nil(), 2024, 3, ().auth(), None)
        .await
        .expect("get_shiftplan_week should succeed");

    let monday = &result.days[0];
    assert_eq!(monday.day_of_week, DayOfWeek::Monday);
    assert_eq!(monday.slots.len(), 1, "expected one slot on Monday");

    let slot_view = &monday.slots[0];
    assert_eq!(
        slot_view.bookings.len(),
        2,
        "expected two bookings in the slot"
    );

    // D-24-03 regression: even though get_all returned is_paid=None for both
    // persons, current_paid_count must equal 1 (the one truly paid booking).
    assert_eq!(
        slot_view.current_paid_count, 1,
        "D-24-03 regression: current_paid_count should be 1 for a non-HR caller \
         with 1 paid booking, but got {}",
        slot_view.current_paid_count
    );
}

// ---- Phase-51 Chain B: Stichtag-Gate + effective_to (D-51-06, D-51-07, D-51-09) ----

/// D-51-07 + Gap-Closure (Chain B Legacy-Modus):
/// Vor Stichtag (`booking_date < active_from`) greift die moderne Kürzung nicht,
/// aber der Legacy-Filter (Pre-Phase-51) greift: ein Overlap-Slot mit
/// `slot.to > cutoff` wird verworfen (Chain B / C ist Anzeige- + Aggregat-
/// Ebene; Chain A' / Chain D bleiben in Modern-Mode → dort Keep(raw)).
///
/// Fixture: ISO-Woche 2026-W31 → Tuesday = 2026-07-28. Stichtag 2026-08-01.
/// Slot 12:00–15:00, Cutoff 14:00 → `slot.to > cutoff` → Drop.
#[test]
fn test_build_shiftplan_day_before_stichtag_legacy_drops_overlap() {
    let overlap_slot = slot_with_day_and_time(DayOfWeek::Tuesday, 12, 15);
    let sp = default_sales_person();
    let short_day = SpecialDay {
        id: Uuid::new_v4(),
        year: 2026,
        calendar_week: 31,
        day_of_week: DayOfWeek::Tuesday,
        day_type: SpecialDayType::ShortDay,
        time_of_day: Some(Time::from_hms(14, 0, 0).unwrap()),
        created: None,
        deleted: None,
        version: Uuid::new_v4(),
    };
    let paid_ids: HashSet<Uuid> = HashSet::new();
    let active_from = Some(Date::from_calendar_date(2026, Month::August, 1).unwrap());

    let result = build_shiftplan_day(
        DayOfWeek::Tuesday,
        std::slice::from_ref(&overlap_slot),
        &[],
        &[sp],
        &[short_day],
        None,
        &paid_ids,
        2026,
        31,
        active_from,
    )
    .unwrap();

    assert!(
        result.slots.is_empty(),
        "Gap-Closure Chain B: Overlap-Slot vor Stichtag muss über Legacy-Filter \
         gedroppt werden (slot.to > cutoff), got {} slots",
        result.slots.len()
    );
}

/// D-51-07 + Gap-Closure Companion:
/// Vor Stichtag + Slot der spätestens am Cutoff endet (`slot.to <= cutoff`) →
/// bleibt roh. Beweis dass der Legacy-Filter granular ist und nicht alle
/// ShortDay-Slots hart droppt.
#[test]
fn test_build_shiftplan_day_before_stichtag_legacy_keeps_pre_cutoff() {
    // Slot Tuesday 09:00–12:00, ShortDay-Cutoff 14:00 → slot.to (12:00) < cutoff.
    let early_slot = slot_with_day_and_time(DayOfWeek::Tuesday, 9, 12);
    let sp = default_sales_person();
    let short_day = SpecialDay {
        id: Uuid::new_v4(),
        year: 2026,
        calendar_week: 31,
        day_of_week: DayOfWeek::Tuesday,
        day_type: SpecialDayType::ShortDay,
        time_of_day: Some(Time::from_hms(14, 0, 0).unwrap()),
        created: None,
        deleted: None,
        version: Uuid::new_v4(),
    };
    let paid_ids: HashSet<Uuid> = HashSet::new();
    let active_from = Some(Date::from_calendar_date(2026, Month::August, 1).unwrap());

    let result = build_shiftplan_day(
        DayOfWeek::Tuesday,
        std::slice::from_ref(&early_slot),
        &[],
        &[sp],
        &[short_day],
        None,
        &paid_ids,
        2026,
        31,
        active_from,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 1);
    assert_eq!(
        result.slots[0].effective_to,
        Time::from_hms(12, 0, 0).unwrap(),
        "vor Stichtag + slot.to<=cutoff: Legacy-Keep → effective_to == raw slot.to"
    );
}

/// D-51-07 (inklusiv am Stichtag) + D-04 Zeile 4: Am Stichtag greift die
/// Kürzung. Overlap-Slot 12:00–15:00, Cutoff 14:00 → `effective_to == 14:00`.
///
/// Fixture: ISO-Woche 2026-W31, Saturday = 2026-08-01 (== Stichtag).
#[test]
fn test_build_shiftplan_day_effective_to_clipped_at_stichtag() {
    let overlap_slot = slot_with_day_and_time(DayOfWeek::Saturday, 12, 15);
    let sp = default_sales_person();
    let short_day = SpecialDay {
        id: Uuid::new_v4(),
        year: 2026,
        calendar_week: 31,
        day_of_week: DayOfWeek::Saturday,
        day_type: SpecialDayType::ShortDay,
        time_of_day: Some(Time::from_hms(14, 0, 0).unwrap()),
        created: None,
        deleted: None,
        version: Uuid::new_v4(),
    };
    let paid_ids: HashSet<Uuid> = HashSet::new();
    let active_from = Some(Date::from_calendar_date(2026, Month::August, 1).unwrap());

    let result = build_shiftplan_day(
        DayOfWeek::Saturday,
        std::slice::from_ref(&overlap_slot),
        &[],
        &[sp],
        &[short_day],
        None,
        &paid_ids,
        2026,
        31,
        active_from,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 1);
    // D-51-09: slot.to bleibt roh.
    assert_eq!(
        result.slots[0].slot.to,
        Time::from_hms(15, 0, 0).unwrap(),
        "slot.to must stay raw at stichtag (D-51-09)"
    );
    // D-04 Zeile 4: effective_to == cutoff.
    assert_eq!(
        result.slots[0].effective_to,
        Time::from_hms(14, 0, 0).unwrap(),
        "am Stichtag: effective_to == cutoff (D-04 Zeile 4)"
    );
}

/// D-51-07 + Gap-Closure (Chain B Legacy-Filter): `active_from = None`
/// → moderne Kürzung aus, aber Legacy-Filter greift wenn ShortDay existiert
/// UND `slot.to > cutoff`.
///
/// Vor Gap-Closure war die Erwartung: "Slot bleibt roh". Nach Gap-Closure
/// (siehe P51 SUMMARY): der historische Filter (Pre-Phase-51) läuft für
/// Chain B/C weiter, d.h. der Overlap-Slot wird gedroppt.
#[test]
fn test_build_shiftplan_day_none_active_from_legacy_drop() {
    let overlap_slot = slot_with_day_and_time(DayOfWeek::Tuesday, 12, 15);
    let sp = default_sales_person();
    let short_day = SpecialDay {
        id: Uuid::new_v4(),
        year: 2026,
        calendar_week: 31,
        day_of_week: DayOfWeek::Tuesday,
        day_type: SpecialDayType::ShortDay,
        time_of_day: Some(Time::from_hms(14, 0, 0).unwrap()),
        created: None,
        deleted: None,
        version: Uuid::new_v4(),
    };
    let paid_ids: HashSet<Uuid> = HashSet::new();

    let result = build_shiftplan_day(
        DayOfWeek::Tuesday,
        std::slice::from_ref(&overlap_slot),
        &[],
        &[sp],
        &[short_day],
        None,
        &paid_ids,
        2026,
        31,
        None, // Kein Stichtag → Gate aus, aber Legacy-Filter aktiv.
    )
    .unwrap();

    assert!(
        result.slots.is_empty(),
        "Gap-Closure Chain B: kein Stichtag + ShortDay + slot.to > cutoff \
         → Legacy-Drop, got {} slots",
        result.slots.len()
    );
}

/// D-51-07 + Gap-Closure Companion: `active_from = None` + kein ShortDay
/// für den Wochentag → Slot bleibt roh. Beweis dass ohne ShortDay-Zeile der
/// Legacy-Filter neutral bleibt.
#[test]
fn test_build_shiftplan_day_none_active_from_no_shortday_keeps_raw() {
    let overlap_slot = slot_with_day_and_time(DayOfWeek::Tuesday, 12, 15);
    let sp = default_sales_person();
    let paid_ids: HashSet<Uuid> = HashSet::new();

    let result = build_shiftplan_day(
        DayOfWeek::Tuesday,
        std::slice::from_ref(&overlap_slot),
        &[],
        &[sp],
        &[], // Kein ShortDay.
        None,
        &paid_ids,
        2026,
        31,
        None, // Kein Stichtag.
    )
    .unwrap();

    assert_eq!(result.slots.len(), 1);
    assert_eq!(
        result.slots[0].effective_to,
        Time::from_hms(15, 0, 0).unwrap(),
        "Kein ShortDay + kein Stichtag: Slot bleibt raw"
    );
}

/// D-51-09: Wenn geclippt, mutiert nur `effective_to` — `slot.to` bleibt
/// persistenz-treu. Beweist die bidirektionale-DTO-Invariante.
#[test]
fn test_build_shiftplan_day_slot_field_stays_raw_when_clipped() {
    let overlap_slot = slot_with_day_and_time(DayOfWeek::Tuesday, 12, 15);
    let sp = default_sales_person();
    let short_day = SpecialDay {
        id: Uuid::new_v4(),
        year: 2026,
        calendar_week: 31,
        day_of_week: DayOfWeek::Tuesday,
        day_type: SpecialDayType::ShortDay,
        time_of_day: Some(Time::from_hms(14, 0, 0).unwrap()),
        created: None,
        deleted: None,
        version: Uuid::new_v4(),
    };
    let paid_ids: HashSet<Uuid> = HashSet::new();
    let active_from = Some(Date::from_calendar_date(2020, Month::January, 1).unwrap());

    let result = build_shiftplan_day(
        DayOfWeek::Tuesday,
        std::slice::from_ref(&overlap_slot),
        &[],
        &[sp],
        &[short_day],
        None,
        &paid_ids,
        2026,
        31,
        active_from,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 1);
    let view = &result.slots[0];
    // slot.to bleibt raw (SlotTO ist bidirektional, siehe P07).
    assert_eq!(view.slot.to, Time::from_hms(15, 0, 0).unwrap());
    // effective_to trägt den gekürzten Anzeige-Wert.
    assert_eq!(view.effective_to, Time::from_hms(14, 0, 0).unwrap());
    // Alle anderen Slot-Felder unverändert.
    assert_eq!(view.slot.from, overlap_slot.from);
    assert_eq!(view.slot.id, overlap_slot.id);
    assert_eq!(view.slot.day_of_week, overlap_slot.day_of_week);
}

/// SHC-05: Auf einem teil-geclippten Slot bleiben existierende Bookings
/// unverändert im `ShiftplanSlot.bookings`-Vec — kein Rewrite, keine
/// Filterung anhand der Booking-Uhrzeit (Bookings haben keine — sie hängen
/// am Slot als Ganzes).
#[test]
fn test_build_shiftplan_day_preserves_bookings_on_clipped_slot() {
    let overlap_slot = slot_with_day_and_time(DayOfWeek::Tuesday, 12, 15);
    let sp = default_sales_person();
    let booking = default_booking(overlap_slot.id, sp.id);
    let short_day = SpecialDay {
        id: Uuid::new_v4(),
        year: 2026,
        calendar_week: 31,
        day_of_week: DayOfWeek::Tuesday,
        day_type: SpecialDayType::ShortDay,
        time_of_day: Some(Time::from_hms(14, 0, 0).unwrap()),
        created: None,
        deleted: None,
        version: Uuid::new_v4(),
    };
    let paid_ids: HashSet<Uuid> = [sp.id].into_iter().collect();
    let active_from = Some(Date::from_calendar_date(2020, Month::January, 1).unwrap());

    let result = build_shiftplan_day(
        DayOfWeek::Tuesday,
        &[overlap_slot],
        std::slice::from_ref(&booking),
        &[sp],
        &[short_day],
        None,
        &paid_ids,
        2026,
        31,
        active_from,
    )
    .unwrap();

    assert_eq!(result.slots.len(), 1);
    let view = &result.slots[0];
    // Booking überlebt die Kürzung unverändert.
    assert_eq!(view.bookings.len(), 1);
    assert_eq!(view.bookings[0].booking.id, booking.id);
    // effective_to zeigt weiterhin den gekürzten Wert.
    assert_eq!(view.effective_to, Time::from_hms(14, 0, 0).unwrap());
    // Paid-Count wird nicht durch das Clipping verändert.
    assert_eq!(view.current_paid_count, 1);
}

/// Gap-Closure Regression (Chain B Shiftplan):
///
/// Wenn `ToggleService::get_toggle_value` `ServiceError::Unauthorized` liefert,
/// darf `get_shiftplan_week` NICHT mit 401 durchschlagen. Statt dessen: Gate
/// inaktiv (Legacy) → Chain B/C wenden den Pre-Phase-51-Legacy-Filter an
/// (siehe Gap-Closure Phase 51). Der Fixture-Slot 9:00–17:00 mit ShortDay-Cutoff
/// 12:00 wird deshalb gedroppt (`slot.to > cutoff`) statt geclippt.
///
/// Regression-Guard für den zentralen `shortday_gate::read_active_from`-Fallback.
#[tokio::test]
async fn test_get_shiftplan_week_tolerates_toggle_unauthorized() {
    let mut deps = build_dependencies();

    // Permission grant (sonst Forbidden vor Toggle-Aufruf).
    deps.permission_service.checkpoint();
    deps.permission_service = MockPermissionService::new();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::new([])));

    // ShortDay-Konfiguration ist da, würde ohne Gap-Closure clippen. Erwartet:
    // Toggle → Unauthorized → None → Slot bleibt roh (9:00–17:00).
    deps.special_day_service.checkpoint();
    deps.special_day_service = MockSpecialDayService::new();
    deps.special_day_service.expect_get_by_week().returning(|_, _, _| {
        Ok(Arc::new([SpecialDay {
            id: Uuid::nil(),
            year: 2024,
            calendar_week: 3,
            day_of_week: DayOfWeek::Monday,
            day_type: SpecialDayType::ShortDay,
            time_of_day: Some(Time::from_hms(12, 0, 0).unwrap()),
            created: None,
            deleted: None,
            version: Uuid::nil(),
        }]))
    });
    deps.toggle_service.checkpoint();
    deps.toggle_service = MockToggleService::new();
    deps.toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Err(service::ServiceError::Unauthorized));

    let service = deps.build_service();
    let shiftplan = service
        .get_shiftplan_week(Uuid::nil(), 2024, 3, ().auth(), None)
        .await
        .expect("Unauthorized-Toleranz: get_shiftplan_week muss Ok liefern (Legacy off)");

    // Gap-Closure Chain B Legacy: Unauthorized → active_from = None (kein 401),
    // aber der historische Legacy-Filter droppt Slots mit `slot.to > cutoff`.
    // Slot 9:00–17:00 mit Cutoff 12:00 → Drop.
    let monday = &shiftplan.days[0];
    assert!(
        monday.slots.is_empty(),
        "Gap-Closure: Unauthorized → Legacy-Filter → Overlap-Slot gedroppt, \
         got {} slots",
        monday.slots.len()
    );
}
