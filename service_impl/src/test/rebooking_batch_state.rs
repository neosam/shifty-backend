//! Phase 55 Plan 01 (HR-ALERT-03 / HR-ALERT-04) — Integration-Tests fuer den
//! F5-Modell-B-Lifecycle des Basic-Tier `RebookingBatchService`:
//!
//! - `create_pending_suggestion` = pending-Batch-PRODUCER (INSERT + UNIQUE-Claim).
//! - `find_pending_for_sales_person` = Reader fuer `pending_rebooking_id` (55-03).
//! - `update_state` = race-sichere Decide-Transition (pending→approved|rejected)
//!   via state-conditional UPDATE (`WHERE state='pending'`) + `rows_affected()==1`.
//!
//! Diese Tests fahren gegen eine echte SQLite-in-memory DB (`sqlx::migrate!`) und
//! den echten `RebookingBatchDaoImpl` — kein Mocking der DB (Muster:
//! `pdf_export_config::integration`), damit das genuin neue DB-Muster
//! (Optimistic-State-Transition + UNIQUE-Slot-Claim) end-to-end nachgewiesen wird.
//!
//! Conflict-Variante (gepinnt): Der Race-/Kollisions-Verlierer erhaelt
//! `ServiceError::EntityAlreadyExists` (→ spaeter HTTP 409, Pitfall 12). Da es
//! keine dedizierte `ServiceError::Conflict`-Variante gibt, ist
//! `EntityAlreadyExists` die kanonische 409-Variante fuer BEIDE Konflikt-Faelle:
//! den belegten Slot (Double-Claim) UND den bereits entschiedenen Batch
//! (Double-Approve / rows_affected==0). Diese Wahl wird in Task 2/3 implementiert.

use std::sync::Arc;

use dao::rebooking_batch::{
    RebookingBatchEntity, RebookingBatchEntryEntity, RebookingBatchKind, RebookingBatchState,
};
use dao_impl_sqlite::rebooking_batch::RebookingBatchDaoImpl;
use dao_impl_sqlite::{TransactionDaoImpl, TransactionImpl};
use service::clock::MockClockService;
use service::permission::Authentication;
use service::rebooking_batch::RebookingBatchService;
use service::uuid_service::MockUuidService;
use service::{MockPermissionService, ServiceError};
use uuid::{uuid, Uuid};

use crate::rebooking_batch::{RebookingBatchServiceDeps, RebookingBatchServiceImpl};

// ─── Harness ────────────────────────────────────────────────────────────────

async fn setup_pool() -> Arc<sqlx::SqlitePool> {
    let pool = Arc::new(
        sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Could not connect to in-memory SQLite"),
    );
    sqlx::migrate!("./../migrations/sqlite")
        .run(pool.as_ref())
        .await
        .expect("Could not run migrations");
    pool
}

struct IntegrationDeps;
impl RebookingBatchServiceDeps for IntegrationDeps {
    type Context = ();
    type Transaction = TransactionImpl;
    type RebookingBatchDao = RebookingBatchDaoImpl;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = TransactionDaoImpl;
}

/// Permission-Mock: `Authentication::Full` passiert das HR-Gate; jeder andere
/// Context erhaelt `Forbidden`. So laesst sich der Non-HR-Pfad ohne echten
/// Permission-DAO testen.
fn build_perm(hr: bool) -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(move |_privilege, context| {
            if context == Authentication::Full || hr {
                Ok(())
            } else {
                Err(service::ServiceError::Forbidden)
            }
        });
    permission_service
}

fn fixed_datetime() -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(
        time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
        time::Time::from_hms(23, 42, 0).unwrap(),
    )
}

fn sp_id() -> Uuid {
    uuid!("2E9F1A62-4A2E-4C89-B8C3-2E9F1A624A2F")
}

/// Baut einen Service mit echter DB (Full-Auth), UuidService liefert stets
/// dieselbe Uuid — fuer die Producer-Tests unerheblich (die id wird zwar von
/// `create_pending_suggestion` erzeugt, aber der Test liest sie zurueck).
fn build_service(
    pool: Arc<sqlx::SqlitePool>,
    perm: MockPermissionService,
) -> RebookingBatchServiceImpl<IntegrationDeps> {
    let mut clock_service = MockClockService::new();
    clock_service.expect_date_time_now().returning(fixed_datetime);

    let mut uuid_service = MockUuidService::new();
    // Distinkte Uuids je Aufruf, damit id != version und mehrere Claims
    // unterscheidbar sind.
    uuid_service.expect_new_uuid().returning(|_process| Uuid::new_v4());

    RebookingBatchServiceImpl::<IntegrationDeps> {
        rebooking_batch_dao: Arc::new(RebookingBatchDaoImpl::new(pool.clone())),
        permission_service: Arc::new(perm),
        clock_service: Arc::new(clock_service),
        uuid_service: Arc::new(uuid_service),
        transaction_dao: Arc::new(TransactionDaoImpl::new(pool.clone())),
    }
}

/// Legt via Phase-54-`create` einen Manual-Batch an, um den UNIQUE-Slot
/// vorzubelegen (fuer den Kollisions-Test).
async fn seed_manual_batch(
    service: &RebookingBatchServiceImpl<IntegrationDeps>,
    iso_year: u32,
    iso_week: u8,
) {
    let batch = RebookingBatchEntity {
        id: Uuid::nil(),
        sales_person_id: sp_id(),
        iso_year,
        iso_week,
        kind: RebookingBatchKind::Manual,
        state: RebookingBatchState::Approved,
        created: time::PrimitiveDateTime::MIN,
        approved: None,
        approved_by: None,
        deleted: None,
        version: Uuid::nil(),
    };
    let entries: Vec<RebookingBatchEntryEntity> = vec![];
    service
        .create(&batch, &entries, Authentication::Full, None)
        .await
        .expect("seed manual batch should succeed");
}

// ─── Tests ────────────────────────────────────────────────────────────────

/// F5-Modell-B Producer (HR-ALERT-04): create_pending_suggestion claimt den
/// leeren Slot und der erzeugte pending-Batch ist danach via find_pending
/// auffindbar.
#[tokio::test]
async fn create_pending_suggestion_claims_slot() {
    let pool = setup_pool().await;
    let service = build_service(pool, build_perm(true));

    let created = service
        .create_pending_suggestion(sp_id(), 2063, 14, Authentication::Full, None)
        .await
        .expect("create_pending_suggestion should claim empty slot");

    assert_eq!(created.kind, RebookingBatchKind::HrSuggestion);
    assert_eq!(created.state, RebookingBatchState::Pending);
    assert_eq!(created.sales_person_id, sp_id());
    assert_eq!(created.iso_year, 2063);
    assert_eq!(created.iso_week, 14);

    let found = service
        .find_pending_for_sales_person(sp_id(), 2063, 14, Authentication::Full, None)
        .await
        .expect("find_pending should succeed")
        .expect("expected the just-created pending batch");
    assert_eq!(found.id, created.id);
    assert_eq!(found.state, RebookingBatchState::Pending);
}

/// UNIQUE-Partial-Index greift: ein bereits belegter Slot (Manual-Batch) laesst
/// keinen zweiten Claim zu → EntityAlreadyExists (kein Read-then-INSERT).
#[tokio::test]
async fn create_pending_suggestion_on_taken_slot_is_conflict() {
    let pool = setup_pool().await;
    let service = build_service(pool, build_perm(true));

    seed_manual_batch(&service, 2063, 20).await;

    let result = service
        .create_pending_suggestion(sp_id(), 2063, 20, Authentication::Full, None)
        .await;

    assert!(
        matches!(result, Err(ServiceError::EntityAlreadyExists(_))),
        "expected EntityAlreadyExists on taken slot, got {result:?}"
    );
}

/// Approve: create_pending_suggestion → update_state(Approved) → find_by_id
/// zeigt state=Approved, approved-Timestamp + approved_by gesetzt.
#[tokio::test]
async fn approve_transitions_pending_to_approved() {
    let pool = setup_pool().await;
    let service = build_service(pool, build_perm(true));

    let created = service
        .create_pending_suggestion(sp_id(), 2063, 15, Authentication::Full, None)
        .await
        .expect("create pending");

    service
        .update_state(
            created.id,
            RebookingBatchState::Approved,
            Some(Arc::from("hr-user")),
            Authentication::Full,
            None,
        )
        .await
        .expect("approve should succeed");

    let after = service
        .find_by_id(created.id, Authentication::Full, None)
        .await
        .expect("find_by_id")
        .expect("batch still present");
    assert_eq!(after.state, RebookingBatchState::Approved);
    assert!(after.approved.is_some(), "approved timestamp must be set");
    assert_eq!(after.approved_by.as_deref(), Some("hr-user"));
}

/// Reject: update_state(Rejected) → state=Rejected.
#[tokio::test]
async fn reject_transitions_pending_to_rejected() {
    let pool = setup_pool().await;
    let service = build_service(pool, build_perm(true));

    let created = service
        .create_pending_suggestion(sp_id(), 2063, 16, Authentication::Full, None)
        .await
        .expect("create pending");

    service
        .update_state(
            created.id,
            RebookingBatchState::Rejected,
            Some(Arc::from("hr-user")),
            Authentication::Full,
            None,
        )
        .await
        .expect("reject should succeed");

    let after = service
        .find_by_id(created.id, Authentication::Full, None)
        .await
        .expect("find_by_id")
        .expect("batch still present");
    assert_eq!(after.state, RebookingBatchState::Rejected);
}

/// Double-Approve (T-55-01): der erste Approve transitioniert; der zweite auf
/// denselben id liefert Conflict (rows_affected==0) — der Batch bleibt Approved.
#[tokio::test]
async fn double_approve_second_is_conflict() {
    let pool = setup_pool().await;
    let service = build_service(pool, build_perm(true));

    let created = service
        .create_pending_suggestion(sp_id(), 2063, 17, Authentication::Full, None)
        .await
        .expect("create pending");

    service
        .update_state(
            created.id,
            RebookingBatchState::Approved,
            Some(Arc::from("hr-user")),
            Authentication::Full,
            None,
        )
        .await
        .expect("first approve should succeed");

    let second = service
        .update_state(
            created.id,
            RebookingBatchState::Approved,
            Some(Arc::from("hr-user-2")),
            Authentication::Full,
            None,
        )
        .await;
    assert!(
        matches!(second, Err(ServiceError::EntityAlreadyExists(_))),
        "second approve must be conflict, got {second:?}"
    );

    // Batch bleibt Approved (kein doppelter Wechsel, approved_by unveraendert).
    let after = service
        .find_by_id(created.id, Authentication::Full, None)
        .await
        .expect("find_by_id")
        .expect("batch present");
    assert_eq!(after.state, RebookingBatchState::Approved);
    assert_eq!(after.approved_by.as_deref(), Some("hr-user"));
}

/// Ein bereits entschiedener Batch (Approved) laesst sich nicht auf Rejected
/// aendern — der `state='pending'`-Guard greift → Conflict.
#[tokio::test]
async fn update_state_on_already_decided_is_conflict() {
    let pool = setup_pool().await;
    let service = build_service(pool, build_perm(true));

    let created = service
        .create_pending_suggestion(sp_id(), 2063, 18, Authentication::Full, None)
        .await
        .expect("create pending");

    service
        .update_state(
            created.id,
            RebookingBatchState::Approved,
            Some(Arc::from("hr-user")),
            Authentication::Full,
            None,
        )
        .await
        .expect("approve");

    let result = service
        .update_state(
            created.id,
            RebookingBatchState::Rejected,
            Some(Arc::from("hr-user")),
            Authentication::Full,
            None,
        )
        .await;
    assert!(
        matches!(result, Err(ServiceError::EntityAlreadyExists(_))),
        "already-decided batch must stay put, got {result:?}"
    );
}

/// find_pending liefert nur Pending: nach Approve desselben Batches ist der Slot
/// nicht mehr pending → None; leerer Slot → None.
#[tokio::test]
async fn find_pending_returns_only_pending() {
    let pool = setup_pool().await;
    let service = build_service(pool, build_perm(true));

    // Leerer Slot → None.
    let empty = service
        .find_pending_for_sales_person(sp_id(), 2063, 19, Authentication::Full, None)
        .await
        .expect("find_pending on empty slot");
    assert!(empty.is_none(), "empty slot must yield None");

    let created = service
        .create_pending_suggestion(sp_id(), 2063, 19, Authentication::Full, None)
        .await
        .expect("create pending");

    let pending = service
        .find_pending_for_sales_person(sp_id(), 2063, 19, Authentication::Full, None)
        .await
        .expect("find_pending")
        .expect("pending present");
    assert_eq!(pending.id, created.id);

    // Nach Approve: kein Pending mehr im Slot.
    service
        .update_state(
            created.id,
            RebookingBatchState::Approved,
            Some(Arc::from("hr-user")),
            Authentication::Full,
            None,
        )
        .await
        .expect("approve");

    let after = service
        .find_pending_for_sales_person(sp_id(), 2063, 19, Authentication::Full, None)
        .await
        .expect("find_pending after approve");
    assert!(after.is_none(), "approved slot must not be pending");
}

/// HR-Gate (T-55-02): Non-HR-Aufrufer auf create_pending_suggestion UND
/// update_state → je ServiceError::Forbidden. Das Gate greift vor jedem
/// Datenabruf.
#[tokio::test]
async fn create_and_update_state_are_hr_gated() {
    let pool = setup_pool().await;
    let service = build_service(pool, build_perm(false));

    let create_res = service
        .create_pending_suggestion(sp_id(), 2063, 21, Authentication::Context(()), None)
        .await;
    assert!(
        matches!(create_res, Err(ServiceError::Forbidden)),
        "non-HR create_pending_suggestion must be Forbidden, got {create_res:?}"
    );

    let update_res = service
        .update_state(
            Uuid::new_v4(),
            RebookingBatchState::Approved,
            Some(Arc::from("nope")),
            Authentication::Context(()),
            None,
        )
        .await;
    assert!(
        matches!(update_res, Err(ServiceError::Forbidden)),
        "non-HR update_state must be Forbidden, got {update_res:?}"
    );

    let find_res = service
        .find_pending_for_sales_person(sp_id(), 2063, 21, Authentication::Context(()), None)
        .await;
    assert!(
        matches!(find_res, Err(ServiceError::Forbidden)),
        "non-HR find_pending must be Forbidden, got {find_res:?}"
    );
}
