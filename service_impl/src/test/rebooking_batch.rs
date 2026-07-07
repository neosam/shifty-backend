//! Phase 54 Plan 02 Task 5 — Unit-Tests fuer RebookingBatchServiceImpl.
//!
//! Testet HR-Gate, Delegation an DAO, defensive id/version/created-Belegung
//! und den UNIQUE-Konflikt-Pfad D-54-DM-01 (Pre-Check innerhalb derselben
//! Transaktion mappt auf ServiceError::EntityAlreadyExists).

use std::sync::Arc;

use dao::{
    rebooking_batch::{
        MockRebookingBatchDao, RebookingBatchEntity, RebookingBatchEntryEntity,
        RebookingBatchKind, RebookingBatchState,
    },
    MockTransaction, MockTransactionDao,
};
use mockall::predicate::always;
use service::{
    clock::MockClockService, permission::Authentication, rebooking_batch::RebookingBatchService,
    uuid_service::MockUuidService, MockPermissionService, ServiceError,
};
use uuid::{uuid, Uuid};

use crate::rebooking_batch::{RebookingBatchServiceDeps, RebookingBatchServiceImpl};
use crate::test::error_test::test_forbidden;

pub struct RebookingBatchServiceDependencies {
    pub rebooking_batch_dao: MockRebookingBatchDao,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
    pub transaction_dao: MockTransactionDao,
}

impl RebookingBatchServiceDeps for RebookingBatchServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type RebookingBatchDao = MockRebookingBatchDao;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = MockTransactionDao;
}

impl RebookingBatchServiceDependencies {
    pub fn build_service(self) -> RebookingBatchServiceImpl<RebookingBatchServiceDependencies> {
        RebookingBatchServiceImpl {
            rebooking_batch_dao: Arc::new(self.rebooking_batch_dao),
            permission_service: Arc::new(self.permission_service),
            clock_service: Arc::new(self.clock_service),
            uuid_service: Arc::new(self.uuid_service),
            transaction_dao: Arc::new(self.transaction_dao),
        }
    }
}

fn build_dependencies(permission: bool, role: &'static str) -> RebookingBatchServiceDependencies {
    let rebooking_batch_dao = MockRebookingBatchDao::new();

    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(always(), always())
        .returning(move |inner_role, context| {
            if context == Authentication::Full || (permission && inner_role == role) {
                Ok(())
            } else {
                Err(service::ServiceError::Forbidden)
            }
        });

    let mut clock_service = MockClockService::new();
    clock_service.expect_date_time_now().returning(fixed_datetime);

    let uuid_service = MockUuidService::new();

    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    RebookingBatchServiceDependencies {
        rebooking_batch_dao,
        permission_service,
        clock_service,
        uuid_service,
        transaction_dao,
    }
}

fn default_batch_id() -> Uuid {
    uuid!("6E9F1A62-4A2E-4C89-B8C3-2E9F1A624A2E")
}
fn default_version() -> Uuid {
    uuid!("7F1A2E9F-6E9F-4A2E-4C89-B8C32E9F1A62")
}
fn default_sales_person_id() -> Uuid {
    uuid!("2E9F1A62-4A2E-4C89-B8C3-2E9F1A624A2F")
}
fn fresh_batch_id() -> Uuid {
    uuid!("AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE")
}
fn fresh_version() -> Uuid {
    uuid!("11111111-2222-3333-4444-555555555555")
}
fn fixed_datetime() -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(
        time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
        time::Time::from_hms(23, 42, 0).unwrap(),
    )
}

fn existing_batch_entity() -> RebookingBatchEntity {
    RebookingBatchEntity {
        id: default_batch_id(),
        sales_person_id: default_sales_person_id(),
        iso_year: 2063,
        iso_week: 14,
        kind: RebookingBatchKind::Manual,
        state: RebookingBatchState::Pending,
        created: fixed_datetime(),
        approved: None,
        approved_by: None,
        deleted: None,
        version: default_version(),
    }
}

/// (1) find_by_id: HR-Gate durch, DAO liefert Some(entity), Ergebnis wird
/// durchgereicht.
#[tokio::test]
async fn find_by_id_returns_dao_result() {
    let mut deps = build_dependencies(true, "hr");
    deps.rebooking_batch_dao
        .expect_find_by_id()
        .returning(|_id, _tx| Ok(Some(existing_batch_entity())));

    let service = deps.build_service();
    let result = service
        .find_by_id(default_batch_id(), Authentication::Full, None)
        .await;

    let entity = result
        .expect("find_by_id should succeed")
        .expect("expected Some(entity)");
    assert_eq!(entity.id, default_batch_id());
    assert_eq!(entity.iso_year, 2063);
    assert_eq!(entity.iso_week, 14);
}

/// (2) find_by_sales_person_year_week: DAO liefert None, Service reicht None
/// nach commit durch.
#[tokio::test]
async fn find_by_sales_person_year_week_none() {
    let mut deps = build_dependencies(true, "hr");
    deps.rebooking_batch_dao
        .expect_find_by_sales_person_year_week()
        .returning(|_sp, _y, _w, _tx| Ok(None));

    let service = deps.build_service();
    let result = service
        .find_by_sales_person_year_week(
            default_sales_person_id(),
            2063,
            14,
            Authentication::Full,
            None,
        )
        .await;

    assert!(matches!(result, Ok(None)));
}

/// (3) create: kein aktiver Batch fuer Slot; UuidService/ClockService fuellen
/// id/version/created defensiv; DAO-INSERT wird angesprochen.
#[tokio::test]
async fn create_success() {
    let mut deps = build_dependencies(true, "hr");
    // Pre-Check: kein Konflikt.
    deps.rebooking_batch_dao
        .expect_find_by_sales_person_year_week()
        .returning(|_sp, _y, _w, _tx| Ok(None));

    // UuidService liefert frische Uuids fuer Batch-id, Batch-version,
    // Entry-id, Entry-version — die Reihenfolge ist implementations-abhaengig,
    // also verwenden wir einen Fallback via Standard-Uuid.
    deps.uuid_service
        .expect_new_uuid()
        .returning(|process| {
            if process.contains("batch id") {
                fresh_batch_id()
            } else if process.contains("batch version") {
                fresh_version()
            } else if process.contains("entry id") {
                uuid!("22222222-3333-4444-5555-666666666666")
            } else {
                uuid!("33333333-4444-5555-6666-777777777777")
            }
        });

    // DAO-INSERT wird aufgerufen.
    deps.rebooking_batch_dao
        .expect_create_batch_with_entries()
        .returning(|_batch, _entries, _process, _tx| Ok(()));

    let service = deps.build_service();

    // Aufrufer uebergibt Uuid::nil-Batch, damit die Fill-Logik greift.
    let batch = RebookingBatchEntity {
        id: Uuid::nil(),
        sales_person_id: default_sales_person_id(),
        iso_year: 2063,
        iso_week: 14,
        kind: RebookingBatchKind::Manual,
        state: RebookingBatchState::Pending,
        created: time::PrimitiveDateTime::MIN,
        approved: None,
        approved_by: None,
        deleted: None,
        version: Uuid::nil(),
    };
    let entry = RebookingBatchEntryEntity {
        id: Uuid::nil(),
        batch_id: Uuid::nil(),
        sales_person_id: default_sales_person_id(),
        hours: 4.0,
        balance_before: -8.0,
        voluntary_actual: 0.0,
        voluntary_committed: 4.0,
        extra_hours_out_id: None,
        extra_hours_in_id: None,
        created: time::PrimitiveDateTime::MIN,
        deleted: None,
        version: Uuid::nil(),
    };

    let result = service
        .create(&batch, &[entry], Authentication::Full, None)
        .await
        .expect("create should succeed");

    assert_eq!(result.id, fresh_batch_id(), "batch id should be filled");
    assert_eq!(
        result.version,
        fresh_version(),
        "batch version should be filled"
    );
    assert_eq!(result.created, fixed_datetime(), "created should be filled");
    assert_eq!(result.sales_person_id, default_sales_person_id());
    assert_eq!(result.iso_year, 2063);
    assert_eq!(result.iso_week, 14);
}

/// (4) create UNIQUE-Konflikt D-54-DM-01: Pre-Check findet bereits einen
/// aktiven Batch → EntityAlreadyExists; DAO-INSERT wird nie erreicht.
#[tokio::test]
async fn create_unique_conflict_maps_to_already_exists() {
    let mut deps = build_dependencies(true, "hr");
    // Pre-Check trifft den existierenden Batch — der DB-UNIQUE-Slot ist belegt.
    deps.rebooking_batch_dao
        .expect_find_by_sales_person_year_week()
        .returning(|_sp, _y, _w, _tx| Ok(Some(existing_batch_entity())));

    // expect_create_batch_with_entries wird bewusst NICHT gesetzt → mockall
    // panics, wenn die Methode dennoch aufgerufen wird.

    let service = deps.build_service();
    let new_batch = RebookingBatchEntity {
        id: Uuid::nil(),
        sales_person_id: default_sales_person_id(),
        iso_year: 2063,
        iso_week: 14,
        kind: RebookingBatchKind::HrSuggestion,
        state: RebookingBatchState::Pending,
        created: time::PrimitiveDateTime::MIN,
        approved: None,
        approved_by: None,
        deleted: None,
        version: Uuid::nil(),
    };

    let result = service
        .create(&new_batch, &[], Authentication::Full, None)
        .await;

    match result {
        Err(ServiceError::EntityAlreadyExists(id)) => {
            assert_eq!(id, default_batch_id());
        }
        other => panic!("Expected EntityAlreadyExists, got {other:?}"),
    }
}

/// (5) find_by_id ohne HR-Rolle → Forbidden; DAO wird NICHT aufgerufen
/// (expect_find_by_id ist nicht gesetzt, mockall panics on unexpected call).
#[tokio::test]
async fn find_by_id_non_hr_forbidden() {
    let deps = build_dependencies(false, "hr");
    // rebooking_batch_dao ohne expect_find_by_id — Aufruf wuerde panic.
    let service = deps.build_service();

    let result = service
        .find_by_id(default_batch_id(), Authentication::Context(()), None)
        .await;

    test_forbidden(&result);
}
