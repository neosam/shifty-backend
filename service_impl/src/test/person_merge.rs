//! Phase 61 Plan 04 (PERS-05 / D-61-03, D-61-04, D-61-05, P-61-B) —
//! Integrationstests fuer **Verknuepfen als Merge** und **Loesen als Split**.
//!
//! `SalesPersonDaoImpl::assign_to_user` verschmilzt seit diesem Plan die beiden
//! Personen der zu verknuepfenden Seiten zu **einer** und loescht die
//! absorbierte `person`-Zeile (D-61-03). `discard_assigned_user` spaltet sie
//! wieder auf: die Sales Person behaelt ihre Identitaet, der Benutzer bekommt
//! eine frisch angelegte (D-61-05).
//!
//! Die Tests fahren gegen eine echte SQLite-in-memory-DB (`sqlx::migrate!`) und
//! gegen die echte DAO-Implementierung — bewusst **ohne** DAO-Mock: Merge und
//! Split sind reines SQL auf einer Transaktion, ein gemockter DAO wuerde weder
//! die Konflikt-`SELECT`s noch die beiden vollstaendigen UNIQUE-Indizes
//! ausfuehren und damit nichts beweisen (Muster: `person_creation.rs`,
//! `person_self_gate.rs`).
//!
//! Abgedeckte Invarianten (Task 1 — Merge):
//!
//! - **M-1** (D-61-03): Zwei freie Objekte ergeben genau **eine** gemeinsame
//!   Person; die absorbierte `person`-Zeile ist geloescht
//!   (`merge_zweier_freier_objekte_ergibt_eine_person`).
//! - **M-2**: Ein zweiter Merge desselben Paars ist ein No-Op
//!   (`merge_ist_idempotent_wenn_bereits_verknuepft`).
//! - **M-3** (D-61-04 / T-61-11 / T-61-12): Ziel-Benutzer haengt bereits an
//!   einer anderen Sales Person → `DaoError::EntityAlreadyExists` **ohne**
//!   jeden Schreibvorgang
//!   (`konflikt_ziel_benutzer_haengt_an_anderer_sales_person`).
//! - **M-4** (D-61-04, Gegenfall): Die Sales Person haengt bereits an einem
//!   anderen Benutzer → ebenfalls `EntityAlreadyExists`
//!   (`konflikt_sales_person_haengt_an_anderem_benutzer`).
//! - **M-5** (D-61-04, Abgrenzung zur verworfenen Alternative): Beide Seiten
//!   tragen **schon** je eine eigene Person (Bestandsdaten nach dem
//!   Phase-60-Backfill), aber jede nur die eigene Seite — der Merge geht durch
//!   (`komplementaeres_verschmelzen_ist_erlaubt`).
//! - **M-6** (SC4 / T-61-15): Ueber `SalesPersonServiceImpl::set_user` wird der
//!   Konflikt zu `ServiceError::EntityAlreadyExists` und ist von
//!   `ServiceError::EntityConflicts` unterscheidbar
//!   (`konflikt_wird_als_409_sichtbar`).

use std::sync::Arc;

use dao::sales_person::SalesPersonDao;
use dao::{DaoError, TransactionDao};
use dao_impl_sqlite::sales_person::SalesPersonDaoImpl;
use dao_impl_sqlite::{TransactionDaoImpl, TransactionImpl};
use service::clock::MockClockService;
use service::permission::Authentication;
use service::sales_person::SalesPersonService;
use service::uuid_service::MockUuidService;
use service::{MockPermissionService, ServiceError};
use uuid::Uuid;

use crate::sales_person::{SalesPersonServiceDeps, SalesPersonServiceImpl};

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

/// Frische Transaktion fuer die DAO-Ebene (der DAO nimmt — anders als der
/// Service — kein `Option<Transaction>`).
async fn tx_of(pool: &Arc<sqlx::SqlitePool>) -> TransactionImpl {
    TransactionDaoImpl::new(pool.clone())
        .new_transaction()
        .await
        .expect("new transaction")
}

async fn commit(pool: &Arc<sqlx::SqlitePool>, tx: TransactionImpl) {
    TransactionDaoImpl::new(pool.clone())
        .commit(tx)
        .await
        .expect("commit");
}

async fn rollback(pool: &Arc<sqlx::SqlitePool>, tx: TransactionImpl) {
    TransactionDaoImpl::new(pool.clone())
        .rollback(tx)
        .await
        .expect("rollback");
}

fn dao(pool: &Arc<sqlx::SqlitePool>) -> SalesPersonDaoImpl {
    SalesPersonDaoImpl::new(pool.clone())
}

struct IntegrationDeps;
impl SalesPersonServiceDeps for IntegrationDeps {
    type Context = ();
    type Transaction = TransactionImpl;
    type SalesPersonDao = SalesPersonDaoImpl;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = TransactionDaoImpl;
}

/// Permission-Mock: jeder Context passiert das HR-Gate. Die Gate-Semantik
/// selbst ist in `test::sales_person` abgedeckt; hier geht es ausschliesslich
/// um den Merge-/Split-Pfad (Muster `person_candidates::build_perm`).
fn build_perm() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_privilege, _context| Ok(()));
    permission_service
}

fn service(pool: &Arc<sqlx::SqlitePool>) -> SalesPersonServiceImpl<IntegrationDeps> {
    let mut clock_service = MockClockService::new();
    clock_service.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, time::Month::September, 17).unwrap(),
            time::Time::from_hms(8, 0, 0).unwrap(),
        )
    });

    let mut uuid_service = MockUuidService::new();
    uuid_service
        .expect_new_uuid()
        .returning(|_process| Uuid::new_v4());

    SalesPersonServiceImpl::<IntegrationDeps> {
        sales_person_dao: Arc::new(SalesPersonDaoImpl::new(pool.clone())),
        permission_service: Arc::new(build_perm()),
        clock_service: Arc::new(clock_service),
        uuid_service: Arc::new(uuid_service),
        transaction_dao: Arc::new(TransactionDaoImpl::new(pool.clone())),
    }
}

// ─── Fixture-Helfer ─────────────────────────────────────────────────────────

async fn seed_person(pool: &sqlx::SqlitePool, id: Uuid) {
    sqlx::query(
        "INSERT INTO person (id, created, update_process, update_version) \
         VALUES (?1, '2026-09-17T00:00:00.000000000Z', 'seed', ?2)",
    )
    .bind(id.as_bytes().to_vec())
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(pool)
    .await
    .expect("seed person");
}

async fn seed_sales_person(pool: &sqlx::SqlitePool, id: Uuid, name: &str, person_id: Option<Uuid>) {
    sqlx::query(
        "INSERT INTO sales_person (id, name, inactive, deleted, update_process, update_version, person_id) \
         VALUES (?1, ?2, 0, NULL, 'seed', ?3, ?4)",
    )
    .bind(id.as_bytes().to_vec())
    .bind(name)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .bind(person_id.map(|id| id.as_bytes().to_vec()))
    .execute(pool)
    .await
    .expect("seed sales_person");
}

async fn seed_user(pool: &sqlx::SqlitePool, name: &str, person_id: Option<Uuid>) {
    sqlx::query("INSERT INTO user (name, update_process, person_id) VALUES (?1, 'seed', ?2)")
        .bind(name)
        .bind(person_id.map(|id| id.as_bytes().to_vec()))
        .execute(pool)
        .await
        .expect("seed user");
}

/// Legt eine Sales Person **mit** eigener Person an und liefert die Person-ID.
async fn seed_sales_person_with_person(pool: &sqlx::SqlitePool, id: Uuid, name: &str) -> Uuid {
    let person_id = Uuid::new_v4();
    seed_person(pool, person_id).await;
    seed_sales_person(pool, id, name, Some(person_id)).await;
    person_id
}

/// Legt einen Benutzer **mit** eigener Person an und liefert die Person-ID.
async fn seed_user_with_person(pool: &sqlx::SqlitePool, name: &str) -> Uuid {
    let person_id = Uuid::new_v4();
    seed_person(pool, person_id).await;
    seed_user(pool, name, Some(person_id)).await;
    person_id
}

async fn person_of_sales_person(pool: &sqlx::SqlitePool, id: Uuid) -> Option<Uuid> {
    let row: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT person_id FROM sales_person WHERE id = ?1")
            .bind(id.as_bytes().to_vec())
            .fetch_one(pool)
            .await
            .expect("read sales_person.person_id");
    row.map(|bytes| Uuid::from_slice(&bytes).expect("uuid"))
}

async fn person_of_user(pool: &sqlx::SqlitePool, name: &str) -> Option<Uuid> {
    let row: Option<Vec<u8>> = sqlx::query_scalar("SELECT person_id FROM user WHERE name = ?1")
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("read user.person_id");
    row.map(|bytes| Uuid::from_slice(&bytes).expect("uuid"))
}

async fn count_persons(pool: &sqlx::SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM person")
        .fetch_one(pool)
        .await
        .expect("count person")
}

async fn person_exists(pool: &sqlx::SqlitePool, id: Uuid) -> bool {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM person WHERE id = ?1")
        .bind(id.as_bytes().to_vec())
        .fetch_one(pool)
        .await
        .expect("count person by id");
    count > 0
}

/// Fuehrt `assign_to_user` auf einer frischen Transaktion aus und committet nur
/// im Erfolgsfall — genau wie der Service es tut.
async fn assign(
    pool: &Arc<sqlx::SqlitePool>,
    sales_person_id: Uuid,
    user_id: &str,
) -> Result<(), DaoError> {
    let tx = tx_of(pool).await;
    let result = dao(pool)
        .assign_to_user(sales_person_id, user_id, "test-merge", tx.clone())
        .await;
    match &result {
        Ok(()) => commit(pool, tx).await,
        Err(_) => rollback(pool, tx).await,
    }
    result
}

// ─── M-1 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn merge_zweier_freier_objekte_ergibt_eine_person() {
    let pool = setup_pool().await;
    let s1 = Uuid::new_v4();
    let p1 = seed_sales_person_with_person(pool.as_ref(), s1, "S1").await;
    let p2 = seed_user_with_person(pool.as_ref(), "U1").await;

    let persons_before = count_persons(pool.as_ref()).await;
    assign(&pool, s1, "U1").await.expect("merge geht durch");

    assert_eq!(
        person_of_sales_person(pool.as_ref(), s1).await,
        Some(p1),
        "M-1: die Sales Person behaelt ihre Identitaet"
    );
    assert_eq!(
        person_of_user(pool.as_ref(), "U1").await,
        Some(p1),
        "M-1: der Benutzer zeigt nach dem Merge auf dieselbe Person"
    );
    assert!(
        !person_exists(pool.as_ref(), p2).await,
        "M-1 (D-61-03): die absorbierte person-Zeile ist geloescht"
    );
    assert_eq!(
        count_persons(pool.as_ref()).await,
        persons_before - 1,
        "M-1: aus zwei Personen ist genau eine geworden"
    );
}

// ─── M-2 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn merge_ist_idempotent_wenn_bereits_verknuepft() {
    let pool = setup_pool().await;
    let s1 = Uuid::new_v4();
    let p1 = seed_sales_person_with_person(pool.as_ref(), s1, "S1").await;
    seed_user_with_person(pool.as_ref(), "U1").await;

    assign(&pool, s1, "U1").await.expect("erster Merge");
    let persons_after_first = count_persons(pool.as_ref()).await;

    assign(&pool, s1, "U1")
        .await
        .expect("M-2: der zweite Merge ist ein No-Op, kein Fehler");

    assert_eq!(
        count_persons(pool.as_ref()).await,
        persons_after_first,
        "M-2: der zweite Merge aendert die person-Zeilenzahl nicht"
    );
    assert_eq!(person_of_sales_person(pool.as_ref(), s1).await, Some(p1));
    assert_eq!(person_of_user(pool.as_ref(), "U1").await, Some(p1));
}

// ─── M-3 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn konflikt_ziel_benutzer_haengt_an_anderer_sales_person() {
    let pool = setup_pool().await;
    let s1 = Uuid::new_v4();
    let s2 = Uuid::new_v4();
    let p1 = seed_sales_person_with_person(pool.as_ref(), s1, "S1").await;
    let p2 = seed_sales_person_with_person(pool.as_ref(), s2, "S2").await;
    // U2 haengt bereits an S2 — beide tragen P2.
    seed_user(pool.as_ref(), "U2", Some(p2)).await;

    let persons_before = count_persons(pool.as_ref()).await;

    let err = assign(&pool, s1, "U2")
        .await
        .expect_err("M-3: echte Mehrdeutigkeit wird abgelehnt");
    assert!(
        matches!(err, DaoError::EntityAlreadyExists),
        "M-3: erwartet EntityAlreadyExists, war {err:?}"
    );

    // T-61-11/T-61-12: im Konfliktfall darf NICHTS geschrieben worden sein.
    assert_eq!(person_of_sales_person(pool.as_ref(), s1).await, Some(p1));
    assert_eq!(person_of_sales_person(pool.as_ref(), s2).await, Some(p2));
    assert_eq!(person_of_user(pool.as_ref(), "U2").await, Some(p2));
    assert_eq!(count_persons(pool.as_ref()).await, persons_before);
}

// ─── M-4 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn konflikt_sales_person_haengt_an_anderem_benutzer() {
    let pool = setup_pool().await;
    let s3 = Uuid::new_v4();
    let p3 = seed_sales_person_with_person(pool.as_ref(), s3, "S3").await;
    // U3 haengt bereits an S3.
    seed_user(pool.as_ref(), "U3", Some(p3)).await;
    let p4 = seed_user_with_person(pool.as_ref(), "U4").await;

    let persons_before = count_persons(pool.as_ref()).await;

    let err = assign(&pool, s3, "U4")
        .await
        .expect_err("M-4: P3 truege nach dem Merge zwei Benutzer");
    assert!(
        matches!(err, DaoError::EntityAlreadyExists),
        "M-4: erwartet EntityAlreadyExists, war {err:?}"
    );

    assert_eq!(person_of_sales_person(pool.as_ref(), s3).await, Some(p3));
    assert_eq!(person_of_user(pool.as_ref(), "U3").await, Some(p3));
    assert_eq!(person_of_user(pool.as_ref(), "U4").await, Some(p4));
    assert_eq!(count_persons(pool.as_ref()).await, persons_before);
}

// ─── M-5 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn komplementaeres_verschmelzen_ist_erlaubt() {
    let pool = setup_pool().await;
    // Bestandsdaten nach dem Phase-60-Backfill: JEDE Zeile traegt eine Person,
    // aber keine Person traegt beide Seiten. D-61-04 sagt: das ist KEIN
    // Konflikt — 409 gilt nur bei echter Mehrdeutigkeit (eine dritte Zeile).
    let s = Uuid::new_v4();
    let p_s = seed_sales_person_with_person(pool.as_ref(), s, "Bestand S").await;
    let p_u = seed_user_with_person(pool.as_ref(), "bestand-u").await;
    // Unbeteiligte Nachbarzeilen, die beim Merge nicht mitgezogen werden duerfen.
    let s_nachbar = Uuid::new_v4();
    let p_nachbar = seed_sales_person_with_person(pool.as_ref(), s_nachbar, "Nachbar").await;
    let p_nachbar_user = seed_user_with_person(pool.as_ref(), "nachbar-u").await;

    let persons_before = count_persons(pool.as_ref()).await;

    assign(&pool, s, "bestand-u")
        .await
        .expect("M-5: komplementaeres Verschmelzen geht durch — kein 409");

    assert_eq!(person_of_sales_person(pool.as_ref(), s).await, Some(p_s));
    assert_eq!(person_of_user(pool.as_ref(), "bestand-u").await, Some(p_s));
    assert!(!person_exists(pool.as_ref(), p_u).await);
    assert_eq!(count_persons(pool.as_ref()).await, persons_before - 1);

    // Nachbarn unberuehrt.
    assert_eq!(
        person_of_sales_person(pool.as_ref(), s_nachbar).await,
        Some(p_nachbar)
    );
    assert_eq!(
        person_of_user(pool.as_ref(), "nachbar-u").await,
        Some(p_nachbar_user)
    );
}

// ─── M-6 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn konflikt_wird_als_409_sichtbar() {
    let pool = setup_pool().await;
    let s1 = Uuid::new_v4();
    let s2 = Uuid::new_v4();
    seed_sales_person_with_person(pool.as_ref(), s1, "S1").await;
    let p2 = seed_sales_person_with_person(pool.as_ref(), s2, "S2").await;
    seed_user(pool.as_ref(), "U2", Some(p2)).await;

    let err = service(&pool)
        .set_user(s1, Some("U2".into()), Authentication::Full, None)
        .await
        .expect_err("M-6: der Konflikt erreicht den Aufrufer");

    // SC4: die Variante muss von EntityConflicts unterscheidbar sein, sonst
    // ist der 409 vom Buchungs-Konflikt nicht trennbar (T-61-15).
    assert!(
        !matches!(err, ServiceError::EntityConflicts(..)),
        "M-6: EntityConflicts waere die falsche Variante, war {err:?}"
    );
    match err {
        ServiceError::EntityAlreadyExists(id) => assert_eq!(
            id, s1,
            "M-6: der 409 nennt die Sales Person, auf die sich der Versuch bezog"
        ),
        other => panic!("M-6: erwartet EntityAlreadyExists, war {other:?}"),
    }
}
