use std::sync::Arc;

use dao::{
    week_status::{MockWeekStatusDao, WeekStatusEntity, WeekStatusKind},
    MockTransaction, MockTransactionDao,
};
use mockall::predicate::{always, eq};
use service::{
    clock::MockClockService,
    permission::Authentication,
    uuid_service::MockUuidService,
    week_status::{WeekStatus, WeekStatusService},
    MockPermissionService,
};
use uuid::{uuid, Uuid};

use crate::test::error_test::test_forbidden;
use crate::week_status::{WeekStatusServiceDeps, WeekStatusServiceImpl};

const YEAR: u32 = 2026;
const WEEK: u8 = 11;
const PROCESS: &str = "week-status-service";

fn default_id() -> Uuid {
    uuid!("11111111-2222-3333-4444-555555555555")
}
fn new_id() -> Uuid {
    uuid!("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
}
fn default_version() -> Uuid {
    uuid!("99999999-8888-7777-6666-555555555555")
}
fn new_version() -> Uuid {
    uuid!("12121212-3434-5656-7878-909090909090")
}

fn fixed_datetime() -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(
        time::Date::from_calendar_date(2026, time::Month::March, 15).unwrap(),
        time::Time::from_hms(10, 0, 0).unwrap(),
    )
}

fn existing_entity(status: WeekStatusKind) -> WeekStatusEntity {
    WeekStatusEntity {
        id: default_id(),
        year: YEAR,
        calendar_week: WEEK,
        status,
        created: fixed_datetime(),
        deleted: None,
        version: default_version(),
    }
}

pub struct WeekStatusServiceDependencies {
    pub week_status_dao: MockWeekStatusDao,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
    pub transaction_dao: MockTransactionDao,
}

impl WeekStatusServiceDeps for WeekStatusServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type WeekStatusDao = MockWeekStatusDao;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = MockTransactionDao;
}

impl WeekStatusServiceDependencies {
    pub fn build_service(self) -> WeekStatusServiceImpl<WeekStatusServiceDependencies> {
        WeekStatusServiceImpl {
            week_status_dao: Arc::new(self.week_status_dao),
            permission_service: Arc::new(self.permission_service),
            clock_service: Arc::new(self.clock_service),
            uuid_service: Arc::new(self.uuid_service),
            transaction_dao: Arc::new(self.transaction_dao),
        }
    }
}

/// `permission = true` grants SHIFTPLANNER_PRIVILEGE; `false` denies it. The
/// transaction/clock mocks are always wired (they may or may not be called).
fn build_dependencies(permission: bool) -> WeekStatusServiceDependencies {
    let week_status_dao = MockWeekStatusDao::new();

    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(always(), always())
        .returning(move |_role, context| {
            if context == Authentication::Full || permission {
                Ok(())
            } else {
                Err(service::ServiceError::Forbidden)
            }
        });

    let mut clock_service = MockClockService::new();
    clock_service
        .expect_date_time_now()
        .returning(fixed_datetime);

    let uuid_service = MockUuidService::new();

    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    WeekStatusServiceDependencies {
        week_status_dao,
        permission_service,
        clock_service,
        uuid_service,
        transaction_dao,
    }
}

// --- ISO week correctness (pure `time` proof, no service/DAO) -----------------
// The service derives (year, week) from `date.to_iso_week_date().0`, never from
// `date.year()`. These 5 mandatory KW-53 / year-boundary cases (D-39-11) pin the
// ground-truth ISO-week behaviour the service must match.
mod iso_week {
    fn iso_year_week(d: time::Date) -> (u32, u8) {
        let (y, w, _) = d.to_iso_week_date();
        (y as u32, w)
    }

    #[test]
    fn case_2021_01_01_is_2020_w53() {
        assert_eq!(iso_year_week(time::macros::date!(2021 - 01 - 01)), (2020, 53));
    }

    #[test]
    fn case_2020_12_28_is_2020_w53() {
        assert_eq!(iso_year_week(time::macros::date!(2020 - 12 - 28)), (2020, 53));
    }

    #[test]
    fn case_2025_12_29_is_2026_w1() {
        assert_eq!(iso_year_week(time::macros::date!(2025 - 12 - 29)), (2026, 1));
    }

    #[test]
    fn case_2025_12_28_is_2025_w52() {
        assert_eq!(iso_year_week(time::macros::date!(2025 - 12 - 28)), (2025, 52));
    }

    #[test]
    fn case_2026_03_15_is_2026_w11() {
        assert_eq!(iso_year_week(time::macros::date!(2026 - 03 - 15)), (2026, 11));
    }
}

// --- Permission gate (T-39-01) ------------------------------------------------

#[tokio::test]
async fn test_set_permission_denied_no_dao_write() {
    // permission denied -> Forbidden, and NO dao create/update/delete is set as an
    // expectation, so any DAO write would panic the mock (proves gate is first).
    let dependencies = build_dependencies(false);
    let service = dependencies.build_service();

    let result = service
        .set_week_status(YEAR, WEEK, WeekStatus::Locked, ().into(), None)
        .await;
    test_forbidden(&result);
}

// --- Upsert / soft-delete semantics (D-39-04) ---------------------------------

#[tokio::test]
async fn test_set_unset_soft_deletes_existing() {
    let mut dependencies = build_dependencies(true);
    dependencies
        .week_status_dao
        .expect_find_by_year_and_week()
        .with(eq(YEAR), eq(WEEK), always())
        .returning(|_, _, _| Ok(Some(existing_entity(WeekStatusKind::InPlanning))));
    dependencies
        .week_status_dao
        .expect_delete()
        .with(eq(default_id()), eq(PROCESS), always())
        .times(1)
        .returning(|_, _, _| Ok(()));
    let service = dependencies.build_service();

    let result = service
        .set_week_status(YEAR, WEEK, WeekStatus::Unset, ().into(), None)
        .await
        .unwrap();
    assert_eq!(result, WeekStatus::Unset);
}

#[tokio::test]
async fn test_set_unset_noop_when_absent() {
    // find -> None + Unset -> neither delete nor create/update (no dao expectations).
    let mut dependencies = build_dependencies(true);
    dependencies
        .week_status_dao
        .expect_find_by_year_and_week()
        .with(eq(YEAR), eq(WEEK), always())
        .returning(|_, _, _| Ok(None));
    let service = dependencies.build_service();

    let result = service
        .set_week_status(YEAR, WEEK, WeekStatus::Unset, ().into(), None)
        .await
        .unwrap();
    assert_eq!(result, WeekStatus::Unset);
}

#[tokio::test]
async fn test_set_creates_when_absent() {
    let mut dependencies = build_dependencies(true);
    dependencies
        .week_status_dao
        .expect_find_by_year_and_week()
        .with(eq(YEAR), eq(WEEK), always())
        .returning(|_, _, _| Ok(None));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq(format!("{PROCESS}::create id")))
        .returning(|_| new_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq(format!("{PROCESS}::create version")))
        .returning(|_| new_version());
    dependencies
        .week_status_dao
        .expect_create()
        .with(
            eq(WeekStatusEntity {
                id: new_id(),
                year: YEAR,
                calendar_week: WEEK,
                status: WeekStatusKind::Locked,
                created: fixed_datetime(),
                deleted: None,
                version: new_version(),
            }),
            eq(PROCESS),
            always(),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));
    let service = dependencies.build_service();

    let result = service
        .set_week_status(YEAR, WEEK, WeekStatus::Locked, ().into(), None)
        .await
        .unwrap();
    assert_eq!(result, WeekStatus::Locked);
}

#[tokio::test]
async fn test_set_updates_when_present() {
    let mut dependencies = build_dependencies(true);
    dependencies
        .week_status_dao
        .expect_find_by_year_and_week()
        .with(eq(YEAR), eq(WEEK), always())
        .returning(|_, _, _| Ok(Some(existing_entity(WeekStatusKind::InPlanning))));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq(format!("{PROCESS}::update version")))
        .returning(|_| new_version());
    dependencies
        .week_status_dao
        .expect_update()
        .with(
            eq(WeekStatusEntity {
                id: default_id(),
                year: YEAR,
                calendar_week: WEEK,
                status: WeekStatusKind::Planned,
                created: fixed_datetime(),
                deleted: None,
                version: new_version(),
            }),
            eq(PROCESS),
            always(),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));
    let service = dependencies.build_service();

    let result = service
        .set_week_status(YEAR, WEEK, WeekStatus::Planned, ().into(), None)
        .await
        .unwrap();
    assert_eq!(result, WeekStatus::Planned);
}

// --- Free transitions (D-39-02) -----------------------------------------------

/// Every transition is allowed, including Locked -> InPlanning and Locked -> Unset
/// (no separate unlock gate). Each leg builds a fresh service (mocks are single-shot).
#[tokio::test]
async fn test_transitions_free() {
    // InPlanning -> Planned (update)
    {
        let mut deps = build_dependencies(true);
        deps.week_status_dao
            .expect_find_by_year_and_week()
            .returning(|_, _, _| Ok(Some(existing_entity(WeekStatusKind::InPlanning))));
        deps.uuid_service
            .expect_new_uuid()
            .returning(|_| new_version());
        deps.week_status_dao
            .expect_update()
            .returning(|_, _, _| Ok(()));
        let service = deps.build_service();
        assert_eq!(
            service
                .set_week_status(YEAR, WEEK, WeekStatus::Planned, ().into(), None)
                .await
                .unwrap(),
            WeekStatus::Planned
        );
    }
    // Planned -> Locked (update)
    {
        let mut deps = build_dependencies(true);
        deps.week_status_dao
            .expect_find_by_year_and_week()
            .returning(|_, _, _| Ok(Some(existing_entity(WeekStatusKind::Planned))));
        deps.uuid_service
            .expect_new_uuid()
            .returning(|_| new_version());
        deps.week_status_dao
            .expect_update()
            .returning(|_, _, _| Ok(()));
        let service = deps.build_service();
        assert_eq!(
            service
                .set_week_status(YEAR, WEEK, WeekStatus::Locked, ().into(), None)
                .await
                .unwrap(),
            WeekStatus::Locked
        );
    }
    // Locked -> Unset (soft-delete, no unlock gate)
    {
        let mut deps = build_dependencies(true);
        deps.week_status_dao
            .expect_find_by_year_and_week()
            .returning(|_, _, _| Ok(Some(existing_entity(WeekStatusKind::Locked))));
        deps.week_status_dao
            .expect_delete()
            .returning(|_, _, _| Ok(()));
        let service = deps.build_service();
        assert_eq!(
            service
                .set_week_status(YEAR, WEEK, WeekStatus::Unset, ().into(), None)
                .await
                .unwrap(),
            WeekStatus::Unset
        );
    }
    // Locked -> InPlanning (update; proves unlock is a free transition)
    {
        let mut deps = build_dependencies(true);
        deps.week_status_dao
            .expect_find_by_year_and_week()
            .returning(|_, _, _| Ok(Some(existing_entity(WeekStatusKind::Locked))));
        deps.uuid_service
            .expect_new_uuid()
            .returning(|_| new_version());
        deps.week_status_dao
            .expect_update()
            .returning(|_, _, _| Ok(()));
        let service = deps.build_service();
        assert_eq!(
            service
                .set_week_status(YEAR, WEEK, WeekStatus::InPlanning, ().into(), None)
                .await
                .unwrap(),
            WeekStatus::InPlanning
        );
    }
    // Unset -> InPlanning (create from absence)
    {
        let mut deps = build_dependencies(true);
        deps.week_status_dao
            .expect_find_by_year_and_week()
            .returning(|_, _, _| Ok(None));
        deps.uuid_service.expect_new_uuid().returning(|_| new_id());
        deps.week_status_dao
            .expect_create()
            .returning(|_, _, _| Ok(()));
        let service = deps.build_service();
        assert_eq!(
            service
                .set_week_status(YEAR, WEEK, WeekStatus::InPlanning, ().into(), None)
                .await
                .unwrap(),
            WeekStatus::InPlanning
        );
    }
}

// --- get_week_status mapping (D-39-04) ----------------------------------------

#[tokio::test]
async fn test_get_returns_unset_when_absent() {
    let mut dependencies = build_dependencies(true);
    dependencies
        .week_status_dao
        .expect_find_by_year_and_week()
        .with(eq(YEAR), eq(WEEK), always())
        .returning(|_, _, _| Ok(None));
    let service = dependencies.build_service();

    let result = service
        .get_week_status(YEAR, WEEK, ().into(), None)
        .await
        .unwrap();
    assert_eq!(result, WeekStatus::Unset);
}

#[tokio::test]
async fn test_get_maps_kind() {
    let mut dependencies = build_dependencies(true);
    dependencies
        .week_status_dao
        .expect_find_by_year_and_week()
        .with(eq(YEAR), eq(WEEK), always())
        .returning(|_, _, _| Ok(Some(existing_entity(WeekStatusKind::Locked))));
    let service = dependencies.build_service();

    let result = service
        .get_week_status(YEAR, WEEK, ().into(), None)
        .await
        .unwrap();
    assert_eq!(result, WeekStatus::Locked);
}
