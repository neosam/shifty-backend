//! Phase 61 Plan 01 (PERS-05 / D-61-02) — Integrationstests fuer die beiden
//! „unverknuepfte Kandidaten"-Lesepfade des Basic-Tier `PersonDao` sowie fuer
//! den daraufliegenden `PersonService`.
//!
//! Die Tests fahren gegen eine echte SQLite-in-memory-DB (`sqlx::migrate!`) und
//! gegen den echten `PersonDaoImpl` — bewusst **ohne** DAO-Mock: beide Lesepfade
//! sind reines SQL, ein gemockter DAO wuerde die Anti-Join-Semantik gar nicht
//! ausfuehren und damit nichts beweisen (Muster: `rebooking_batch_state.rs`,
//! `person_backfill.rs`).
//!
//! Abgedeckte Invarianten:
//!
//! - **K-1** (D-61-02): Eine Sales Person, deren Person keinen Benutzer traegt,
//!   ist Kandidat (`sales_person_ohne_user_ist_kandidat`).
//! - **K-2**: Eine Sales Person, deren Person einen Benutzer traegt, ist kein
//!   Kandidat (`verknuepfte_sales_person_ist_kein_kandidat`).
//! - **K-3**: Eine soft-geloeschte Sales Person ist kein Kandidat, auch wenn
//!   ihre Person keinen Benutzer traegt — `sp.deleted IS NULL`
//!   (`soft_geloeschte_sales_person_ist_kein_kandidat`).
//! - **K-4** (D-61-02): Ein Benutzer, dessen Person keine Sales Person traegt,
//!   ist Kandidat (`user_ohne_sales_person_ist_kandidat`).
//! - **K-5**: Ein verknuepfter Benutzer ist kein Kandidat
//!   (`verknuepfter_user_ist_kein_kandidat`).
//! - **K-6** (der entscheidende D-61-02-Nachweis): Jede Fixture-Zeile traegt ein
//!   nicht-NULL `person_id`; beide Kandidaten-Mengen sind trotzdem nicht leer.
//!   Eine `WHERE person_id IS NULL`-Implementierung wuerde hier leer liefern
//!   (`kandidat_trotz_gesetzter_person_id`).
//!
//! Ergaenzt in Task 2 (Service-Ebene, `MockPermissionService` als Gate):
//!
//! - **K-7** (T-61-05): `create` vergibt id/version/created und ist ueber `get`
//!   wiederauffindbar (`create_vergibt_id_version_und_created`).
//! - **K-8** (T-61-05): `create` ohne HR-Privileg liefert `Forbidden` und legt
//!   keine `person`-Zeile an (`create_ohne_hr_privileg_ist_forbidden`).
//! - **K-9** (T-61-05): `unlinked_sales_persons` ohne HR-Privileg liefert
//!   `Forbidden` (`unlinked_sales_persons_gated`).
//! - **K-10**: `unlinked_sales_persons` liefert HR-seitig dieselbe Menge wie der
//!   DAO-Lesepfad (`unlinked_sales_persons_liefert_kandidaten`).
//!
//! Ergaenzt in Phase 62 Plan 01 Task 1 (D-62-05 / Review-Finding WR-02):
//!
//! - **K-11** (D-62-05 / WR-02): Ein Benutzer, dessen einzige Sales Person
//!   soft-geloescht ist, ist wieder Kandidat fuer „Person hinzufuegen"
//!   (`user_mit_nur_soft_geloeschter_sales_person_ist_kandidat`). Vor dem Fix
//!   fiel er aus BEIDEN Kandidatenlisten: die Sales Person war wegen
//!   `sp.deleted IS NULL` kein „Login hinzufuegen"-Kandidat, und der Benutzer
//!   hatte formal einen Join-Partner. K-11 ist die Gegenprobe zu K-3 und haelt
//!   fest, dass der Loesch-Filter in der `ON`- und nicht in der `WHERE`-Klausel
//!   steht — in `WHERE` degeneriert der LEFT JOIN zum INNER JOIN und K-4 faellt.

use std::sync::Arc;

use dao::person::PersonDao;
use dao_impl_sqlite::person::PersonDaoImpl;
use dao_impl_sqlite::{TransactionDaoImpl, TransactionImpl};
use service::clock::MockClockService;
use service::permission::Authentication;
use service::person::PersonService;
use service::uuid_service::MockUuidService;
use service::{MockPermissionService, ServiceError};
use uuid::Uuid;

use crate::person::{PersonServiceDeps, PersonServiceImpl};

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
impl PersonServiceDeps for IntegrationDeps {
    type Context = ();
    type Transaction = TransactionImpl;
    type PersonDao = PersonDaoImpl;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = TransactionDaoImpl;
}

/// Permission-Mock: `Authentication::Full` passiert das HR-Gate; jeder andere
/// Context erhaelt `Forbidden`. So laesst sich der Non-HR-Pfad ohne echten
/// Permission-DAO testen (Muster `rebooking_batch_state::build_perm`).
fn build_perm(hr: bool) -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(move |_privilege, context| {
            if context == Authentication::Full || hr {
                Ok(())
            } else {
                Err(ServiceError::Forbidden)
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

fn build_service(
    pool: Arc<sqlx::SqlitePool>,
    perm: MockPermissionService,
) -> PersonServiceImpl<IntegrationDeps> {
    let mut clock_service = MockClockService::new();
    clock_service.expect_date_time_now().returning(fixed_datetime);

    let mut uuid_service = MockUuidService::new();
    uuid_service
        .expect_new_uuid()
        .returning(|_process| Uuid::new_v4());

    PersonServiceImpl::<IntegrationDeps> {
        person_dao: Arc::new(PersonDaoImpl::new(pool.clone())),
        permission_service: Arc::new(perm),
        clock_service: Arc::new(clock_service),
        uuid_service: Arc::new(uuid_service),
        transaction_dao: Arc::new(TransactionDaoImpl::new(pool.clone())),
    }
}

/// Frische Transaktion fuer die DAO-Ebene (der DAO nimmt — anders als der
/// Service — kein `Option<Transaction>`).
async fn tx_of(pool: &Arc<sqlx::SqlitePool>) -> TransactionImpl {
    use dao::TransactionDao;
    TransactionDaoImpl::new(pool.clone())
        .new_transaction()
        .await
        .expect("new transaction")
}

// ─── Fixtures ───────────────────────────────────────────────────────────────

struct Fixtures {
    /// S1: Sales Person mit Person P1, KEIN Benutzer → Kandidat.
    unlinked_sales_person: Uuid,
    /// S2: Sales Person mit Person P2, Benutzer U2 auf derselben Person.
    linked_sales_person: Uuid,
    /// S3: soft-geloescht, Person P3 ohne Benutzer → trotzdem kein Kandidat.
    deleted_sales_person: Uuid,
    /// S5: soft-geloescht, Person P5 traegt zusaetzlich Benutzer U5. Die Sales
    /// Person ist kein Kandidat (K-3), U5 muss aber einer sein (K-11 / WR-02).
    deleted_sales_person_with_user: Uuid,
}

const LINKED_USER: &str = "u2-verknuepft";
const UNLINKED_USER: &str = "u4-ohne-sales-person";
/// K-11 (D-62-05 / WR-02): Benutzer, dessen einzige Sales Person soft-geloescht
/// ist. Er traegt formal einen Join-Partner, darf aber trotzdem Kandidat sein.
const DELETED_SALES_PERSON_USER: &str = "u5-nur-geloeschte-sales-person";

async fn seed_person(pool: &sqlx::SqlitePool, id: Uuid) {
    sqlx::query(
        "INSERT INTO person (id, created, update_process, update_version) \
         VALUES (?1, '2026-01-01T00:00:00.000000000Z', 'seed', ?2)",
    )
    .bind(id.as_bytes().to_vec())
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(pool)
    .await
    .expect("seed person");
}

async fn seed_sales_person(
    pool: &sqlx::SqlitePool,
    id: Uuid,
    name: &str,
    person_id: Uuid,
    deleted: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO sales_person \
         (id, name, inactive, deleted, person_id, update_process, update_version) \
         VALUES (?1, ?2, 0, ?3, ?4, 'seed', ?5)",
    )
    .bind(id.as_bytes().to_vec())
    .bind(name)
    .bind(deleted)
    .bind(person_id.as_bytes().to_vec())
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(pool)
    .await
    .expect("seed sales_person");
}

async fn seed_user(pool: &sqlx::SqlitePool, name: &str, person_id: Uuid) {
    sqlx::query("INSERT INTO user (name, person_id, update_process) VALUES (?1, ?2, 'seed')")
        .bind(name)
        .bind(person_id.as_bytes().to_vec())
        .execute(pool)
        .await
        .expect("seed user");
}

/// Seedet alle vier Auspraegungen. Entscheidend fuer K-6: **jede** Zeile traegt
/// ein nicht-NULL `person_id` — der Zustand nach dem Phase-60-Backfill
/// (D-61-01). Eine `person_id IS NULL`-Implementierung der Lesepfade wuerde
/// gegen diese Fixture leer liefern.
async fn seed_fixtures(pool: &sqlx::SqlitePool) -> Fixtures {
    let p1 = Uuid::new_v4();
    let p2 = Uuid::new_v4();
    let p3 = Uuid::new_v4();
    let p4 = Uuid::new_v4();
    let p5 = Uuid::new_v4();

    for person in [p1, p2, p3, p4, p5] {
        seed_person(pool, person).await;
    }

    let unlinked_sales_person = Uuid::new_v4();
    let linked_sales_person = Uuid::new_v4();
    let deleted_sales_person = Uuid::new_v4();
    let deleted_sales_person_with_user = Uuid::new_v4();

    seed_sales_person(pool, unlinked_sales_person, "S1 ohne Login", p1, None).await;
    seed_sales_person(pool, linked_sales_person, "S2 verknuepft", p2, None).await;
    seed_sales_person(
        pool,
        deleted_sales_person,
        "S3 geloescht",
        p3,
        Some("2026-02-01T00:00:00.000000000Z"),
    )
    .await;
    seed_sales_person(
        pool,
        deleted_sales_person_with_user,
        "S5 geloescht mit Login",
        p5,
        Some("2026-02-01T00:00:00.000000000Z"),
    )
    .await;

    seed_user(pool, LINKED_USER, p2).await;
    seed_user(pool, UNLINKED_USER, p4).await;
    seed_user(pool, DELETED_SALES_PERSON_USER, p5).await;

    Fixtures {
        unlinked_sales_person,
        linked_sales_person,
        deleted_sales_person,
        deleted_sales_person_with_user,
    }
}

async fn candidate_sales_person_ids(pool: &Arc<sqlx::SqlitePool>) -> Vec<Uuid> {
    let dao = PersonDaoImpl::new(pool.clone());
    dao.sales_persons_without_user(tx_of(pool).await)
        .await
        .expect("sales_persons_without_user")
        .iter()
        .map(|entity| entity.id)
        .collect()
}

async fn candidate_user_names(pool: &Arc<sqlx::SqlitePool>) -> Vec<String> {
    let dao = PersonDaoImpl::new(pool.clone());
    dao.users_without_sales_person(tx_of(pool).await)
        .await
        .expect("users_without_sales_person")
        .iter()
        .map(|name| name.to_string())
        .collect()
}

async fn count_persons(pool: &sqlx::SqlitePool) -> i64 {
    use sqlx::Row;
    sqlx::query("SELECT COUNT(*) AS cnt FROM person")
        .fetch_one(pool)
        .await
        .expect("count person")
        .get::<i64, _>("cnt")
}

// ─── K-1 … K-6: DAO-Lesepfade (Anti-Join, D-61-02) ──────────────────────────

#[tokio::test]
async fn sales_person_ohne_user_ist_kandidat() {
    let pool = setup_pool().await;
    let fixtures = seed_fixtures(pool.as_ref()).await;

    let candidates = candidate_sales_person_ids(&pool).await;

    assert!(
        candidates.contains(&fixtures.unlinked_sales_person),
        "S1 traegt eine Person ohne Benutzer und muss Kandidat sein, war aber nicht in {candidates:?}"
    );
}

#[tokio::test]
async fn verknuepfte_sales_person_ist_kein_kandidat() {
    let pool = setup_pool().await;
    let fixtures = seed_fixtures(pool.as_ref()).await;

    let candidates = candidate_sales_person_ids(&pool).await;

    assert!(
        !candidates.contains(&fixtures.linked_sales_person),
        "S2 teilt sich ihre Person mit Benutzer {LINKED_USER} und darf kein Kandidat sein"
    );
}

#[tokio::test]
async fn soft_geloeschte_sales_person_ist_kein_kandidat() {
    let pool = setup_pool().await;
    let fixtures = seed_fixtures(pool.as_ref()).await;

    let candidates = candidate_sales_person_ids(&pool).await;

    assert!(
        !candidates.contains(&fixtures.deleted_sales_person),
        "S3 ist soft-geloescht (sp.deleted IS NULL-Filter) und darf kein Kandidat sein"
    );
}

#[tokio::test]
async fn user_ohne_sales_person_ist_kandidat() {
    let pool = setup_pool().await;
    seed_fixtures(pool.as_ref()).await;

    let candidates = candidate_user_names(&pool).await;

    assert!(
        candidates.iter().any(|name| name == UNLINKED_USER),
        "U4 traegt eine Person ohne Sales Person und muss Kandidat sein, war aber nicht in {candidates:?}"
    );
}

#[tokio::test]
async fn verknuepfter_user_ist_kein_kandidat() {
    let pool = setup_pool().await;
    seed_fixtures(pool.as_ref()).await;

    let candidates = candidate_user_names(&pool).await;

    assert!(
        !candidates.iter().any(|name| name == LINKED_USER),
        "U2 teilt sich seine Person mit einer Sales Person und darf kein Kandidat sein"
    );
}

/// K-11 (D-62-05 / Review-Finding WR-02): U5 teilt sich seine Person mit einer
/// **soft-geloeschten** Sales Person. Die Sales Person ist wegen K-3 kein
/// „Login hinzufuegen"-Kandidat; waere U5 auch kein „Person hinzufuegen"-
/// Kandidat, liesse er sich ueber den Picker nie wieder verknuepfen. Der Fix
/// zieht `sp.deleted IS NULL` in die `ON`-Klausel des LEFT JOIN.
#[tokio::test]
async fn user_mit_nur_soft_geloeschter_sales_person_ist_kandidat() {
    let pool = setup_pool().await;
    let fixtures = seed_fixtures(pool.as_ref()).await;

    let candidates = candidate_user_names(&pool).await;

    assert!(
        candidates
            .iter()
            .any(|name| name == DELETED_SALES_PERSON_USER),
        "U5 teilt sich seine Person nur mit der soft-geloeschten Sales Person \
         {} und muss darum Kandidat sein, war aber nicht in {candidates:?}",
        fixtures.deleted_sales_person_with_user
    );
}

/// D-61-02 im Kern: nach dem Phase-60-Backfill traegt JEDE Zeile eine Person.
/// „Unverknuepft" ist die Abwesenheit des Join-Partners, nicht die Abwesenheit
/// eines Werts. Eine `WHERE person_id IS NULL`-Implementierung faellt hier um.
#[tokio::test]
async fn kandidat_trotz_gesetzter_person_id() {
    use sqlx::Row;

    let pool = setup_pool().await;
    seed_fixtures(pool.as_ref()).await;

    let sales_person_nulls: i64 =
        sqlx::query("SELECT COUNT(*) AS cnt FROM sales_person WHERE person_id IS NULL")
            .fetch_one(pool.as_ref())
            .await
            .expect("count sales_person null")
            .get("cnt");
    let user_nulls: i64 = sqlx::query("SELECT COUNT(*) AS cnt FROM user WHERE person_id IS NULL")
        .fetch_one(pool.as_ref())
        .await
        .expect("count user null")
        .get("cnt");

    assert_eq!(
        0, sales_person_nulls,
        "Fixture-Vorbedingung: keine sales_person-Zeile ohne person_id"
    );
    assert_eq!(
        0, user_nulls,
        "Fixture-Vorbedingung: keine user-Zeile ohne person_id"
    );

    assert!(
        !candidate_sales_person_ids(&pool).await.is_empty(),
        "Anti-Join muss Kandidaten liefern, obwohl jede Zeile ein person_id traegt"
    );
    assert!(
        !candidate_user_names(&pool).await.is_empty(),
        "Anti-Join muss Kandidaten liefern, obwohl jede Zeile ein person_id traegt"
    );
}

// ─── K-7 … K-10: Service-Ebene (HR-Gate, T-61-05) ───────────────────────────

#[tokio::test]
async fn create_vergibt_id_version_und_created() {
    let pool = setup_pool().await;
    let service = build_service(pool.clone(), build_perm(false));

    let person = service
        .create(Authentication::Full, None)
        .await
        .expect("create should succeed for HR");

    assert_ne!(Uuid::nil(), person.id, "create muss eine id vergeben");
    assert_ne!(
        Uuid::nil(),
        person.version,
        "create muss eine version vergeben"
    );
    assert_eq!(
        fixed_datetime(),
        person.created,
        "created kommt aus dem ClockService"
    );

    let found = service
        .get(person.id, Authentication::Full, None)
        .await
        .expect("get should succeed for HR");
    assert_eq!(
        Some(person.clone()),
        found,
        "die angelegte Person muss ueber get(id) wiederauffindbar sein"
    );
}

#[tokio::test]
async fn create_ohne_hr_privileg_ist_forbidden() {
    let pool = setup_pool().await;
    let service = build_service(pool.clone(), build_perm(false));

    let before = count_persons(pool.as_ref()).await;

    let result = service.create(Authentication::Context(()), None).await;

    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "Nicht-HR muss Forbidden erhalten, bekam {result:?}"
    );
    assert_eq!(
        before,
        count_persons(pool.as_ref()).await,
        "ein abgelehnter create darf keine person-Zeile hinterlassen"
    );
}

#[tokio::test]
async fn unlinked_sales_persons_gated() {
    let pool = setup_pool().await;
    seed_fixtures(pool.as_ref()).await;
    let service = build_service(pool.clone(), build_perm(false));

    let sales_persons = service
        .unlinked_sales_persons(Authentication::Context(()), None)
        .await;
    assert!(
        matches!(sales_persons, Err(ServiceError::Forbidden)),
        "unlinked_sales_persons muss HR-gated sein, bekam {sales_persons:?}"
    );

    let users = service
        .unlinked_users(Authentication::Context(()), None)
        .await;
    assert!(
        matches!(users, Err(ServiceError::Forbidden)),
        "unlinked_users muss HR-gated sein, bekam {users:?}"
    );
}

#[tokio::test]
async fn unlinked_sales_persons_liefert_kandidaten() {
    let pool = setup_pool().await;
    let fixtures = seed_fixtures(pool.as_ref()).await;
    let service = build_service(pool.clone(), build_perm(false));

    let mut service_ids: Vec<Uuid> = service
        .unlinked_sales_persons(Authentication::Full, None)
        .await
        .expect("unlinked_sales_persons should succeed for HR")
        .iter()
        .map(|sales_person| sales_person.id)
        .collect();
    let mut dao_ids = candidate_sales_person_ids(&pool).await;
    service_ids.sort();
    dao_ids.sort();

    assert_eq!(
        dao_ids, service_ids,
        "Service-Lesepfad muss dieselbe Menge liefern wie der DAO-Lesepfad"
    );
    assert_eq!(
        vec![fixtures.unlinked_sales_person],
        service_ids,
        "genau S1 ist Kandidat (S2 verknuepft, S3 soft-geloescht)"
    );

    let mut user_names: Vec<String> = service
        .unlinked_users(Authentication::Full, None)
        .await
        .expect("unlinked_users should succeed for HR")
        .iter()
        .map(|name| name.to_string())
        .collect();
    user_names.sort();
    let mut expected_users = vec![
        UNLINKED_USER.to_string(),
        DELETED_SALES_PERSON_USER.to_string(),
    ];
    expected_users.sort();
    assert_eq!(
        expected_users, user_names,
        "genau U4 (gar keine Sales Person) und U5 (nur eine soft-geloeschte, \
         D-62-05 / WR-02) sind Benutzer-Kandidaten; U2 ist verknuepft"
    );
}
