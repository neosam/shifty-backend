//! Phase 63 Plan 04 (PADM-04 / PADM-05 / PADM-06 / PADM-08) — Tests fuer den
//! Lesepfad des Business-Logic-Aggregats `PersonDetailService`.
//!
//! Zwei Testarten, jede fuer das, was sie allein beweisen kann:
//!
//! - **Mock-Tests** (`MockPersonDirectoryDao`, `MockPersonService`,
//!   `MockTransactionDao`) fuer die Gate-Faelle und fuer die Aufruf-Zahl. Nur
//!   ein Mock kann belegen, dass bei fehlendem Privileg **gar kein** DAO
//!   angefasst wird (`.times(0)`), und nur ein Mock kann festhalten, dass der
//!   Detail-Lesepfad jede DAO-Methode **genau einmal** aufruft (`.times(1)`,
//!   T-63-04-A, Prohibition „kein N+1").
//! - **Integrationstests** gegen eine echte SQLite-in-memory-DB mit den echten
//!   DAO-Implementierungen fuer die drei Auspraegungen, den Not-Found-Fall und
//!   die beiden Kandidatenlisten. Klassifikation, Anzeigename und
//!   Anti-Join-Semantik haengen an der tatsaechlichen Zeilenmenge; ein
//!   gemockter DAO wuerde davon nichts ausfuehren.
//!
//! Der Permission-Mock antwortet nur auf das Literal `"admin"` positiv. Ein
//! Service, der hier versehentlich das `HR_PRIVILEGE` der Basic-Tier-Services
//! uebernaehme (RESEARCH.md Pitfall 4), faellt damit in **jedem** Test dieser
//! Datei um, nicht nur im Gate-Test.
//!
//! Abgedeckte Invarianten:
//!
//! - **M-1** (T-63-04-A, PADM-08): `get_detail` ohne Admin-Privileg liefert
//!   `Forbidden`, und weder `use_transaction` noch ein DAO wurde aufgerufen.
//! - **M-2** (Prohibition „kein N+1"): `get_detail` ruft
//!   `find_directory_entry` und `all_user_roles` je genau einmal auf.
//! - **M-3 / M-4** (T-63-04-B): Beide Kandidatenmethoden gaten auf `admin`,
//!   bevor `PersonService` ueberhaupt angefasst wird.
//! - **I-1** (PADM-05, D-62-01): Verknuepfte Person → `Linked`, Verkaeufername
//!   als Anzeigename, Benutzername und `sales_person_id` gesetzt.
//! - **I-2**: Reines Benutzerkonto → `UserOnly`, `sales_person_id == None`.
//! - **I-3**: Reiner Verkaeufer → `SalesPersonOnly`, `username == None`.
//! - **I-4**: Unbekannte `person_id` → `EntityNotFound`.
//! - **I-5** (D-62-04): Traegerlos gewordene Person (nur soft-geloeschte Sales
//!   Person) → `EntityNotFound`, nicht eine halbleere Antwort.
//! - **I-6** (D-62-06): Die Rollen der Zeile sind genau die des eigenen
//!   Benutzers — fremde Rollen lecken nicht herein.
//! - **I-7** (PADM-04, SC-4): Eine bereits verknuepfte Sales Person erscheint
//!   nicht unter den Kandidaten.
//! - **I-8** (PADM-04, SC-4): Ein bereits verknuepfter Benutzer erscheint nicht
//!   unter den Kandidaten.

use std::sync::Arc;

use dao::person::MockPersonDao;
use dao::person_directory::{MockPersonDirectoryDao, PersonDirectoryRow, UserRoleRow};
use dao::{MockPermissionDao, MockTransaction, MockTransactionDao};
use dao_impl_sqlite::person::PersonDaoImpl;
use dao_impl_sqlite::person_directory::PersonDirectoryDaoImpl;
use dao_impl_sqlite::{PermissionDaoImpl, TransactionDaoImpl, TransactionImpl};
use service::clock::MockClockService;
use service::permission::Authentication;
use service::person::MockPersonService;
use service::person_detail::{CreatePersonEntry, PersonDetailService, PersonFacet};
use service::person_directory::{PersonEntry, PersonKind};
use service::sales_person::{MockSalesPersonService, SalesPersonService};
use service::uuid_service::MockUuidService;
use service::{MockPermissionService, ServiceError};
use uuid::Uuid;

use crate::person::{PersonServiceDeps, PersonServiceImpl};
use crate::person_detail::{PersonDetailServiceDeps, PersonDetailServiceImpl};
use crate::sales_person::{SalesPersonServiceDeps, SalesPersonServiceImpl};

// ─── Permission-Mock (beide Testarten) ──────────────────────────────────────

/// Erfolgreich **nur** beim Literal `"admin"` und nur, wenn der Test das
/// Privileg zugesteht. `Authentication::Full` passiert immer — so kann der
/// innere, HR-gegatete `PersonService` seinen eigenen Mock benutzen, ohne dass
/// das aeussere Admin-Gate weich wird.
fn build_perm(admin: bool) -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(move |privilege, context| {
            if context == Authentication::Full || (admin && privilege == "admin") {
                Ok(())
            } else {
                Err(ServiceError::Forbidden)
            }
        });
    permission_service
}

// ─── Mock-Harness (M-1 … M-4) ───────────────────────────────────────────────

struct MockDeps;
impl PersonDetailServiceDeps for MockDeps {
    type Context = ();
    type Transaction = MockTransaction;
    type PersonDirectoryDao = MockPersonDirectoryDao;
    type PersonDao = MockPersonDao;
    type PermissionDao = MockPermissionDao;
    type SalesPersonDao = dao::sales_person::MockSalesPersonDao;
    type PersonService = MockPersonService;
    type SalesPersonService = MockSalesPersonService;
    type PermissionService = MockPermissionService;
    type TransactionDao = MockTransactionDao;
}

fn build_mock_transaction_dao() -> MockTransactionDao {
    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));
    transaction_dao
}

/// `MockTransactionDao`, dessen `use_transaction` **nie** aufgerufen werden
/// darf — der Nachweis, dass das Gate vor jedem DB-Kontakt liegt.
fn build_untouched_transaction_dao() -> MockTransactionDao {
    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao.expect_use_transaction().times(0);
    transaction_dao.expect_commit().times(0);
    transaction_dao
}

#[allow(clippy::too_many_arguments)]
fn build_mock_service(
    person_directory_dao: MockPersonDirectoryDao,
    person_service: MockPersonService,
    permission_service: MockPermissionService,
    transaction_dao: MockTransactionDao,
) -> PersonDetailServiceImpl<MockDeps> {
    PersonDetailServiceImpl::<MockDeps> {
        person_directory_dao: Arc::new(person_directory_dao),
        person_dao: Arc::new(MockPersonDao::new()),
        permission_dao: Arc::new(MockPermissionDao::new()),
        sales_person_dao: Arc::new(dao::sales_person::MockSalesPersonDao::new()),
        person_service: Arc::new(person_service),
        sales_person_service: Arc::new(MockSalesPersonService::new()),
        permission_service: Arc::new(permission_service),
        transaction_dao: Arc::new(transaction_dao),
    }
}

fn linked_row(person_id: Uuid, sales_person_id: Uuid) -> PersonDirectoryRow {
    PersonDirectoryRow {
        person_id,
        sales_person_id: Some(sales_person_id),
        sales_person_name: Some("Bea Verknuepft".into()),
        background_color: Some("#FFF".into()),
        is_paid: Some(true),
        inactive: Some(false),
        username: Some("bea".into()),
    }
}

/// M-1 (T-63-04-A / PADM-08): Ohne Admin-Privileg wird **kein** DAO angefasst
/// und **keine** Transaktion geoeffnet. Ein Service, der das Gate hinter
/// `use_transaction` schoebe, liesse hier eine Transaktion offen.
#[tokio::test]
async fn get_detail_ohne_admin_privileg_ist_forbidden() {
    let mut person_directory_dao = MockPersonDirectoryDao::new();
    person_directory_dao.expect_find_directory_entry().times(0);
    person_directory_dao.expect_all_user_roles().times(0);

    let service = build_mock_service(
        person_directory_dao,
        MockPersonService::new(),
        build_perm(false),
        build_untouched_transaction_dao(),
    );

    let result = service
        .get_detail(Uuid::new_v4(), Authentication::Context(()), None)
        .await;

    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "ohne Admin-Privileg muss get_detail Forbidden liefern: {result:?}",
    );
}

/// M-2 (Prohibition „kein N+1"): Der Detail-Lesepfad fragt jede DAO-Methode
/// **genau einmal** — eine begrenzte Zahl Abfragen fuer EINE Person.
#[tokio::test]
async fn get_detail_fragt_jede_dao_methode_genau_einmal() {
    let person_id = Uuid::new_v4();
    let sales_person_id = Uuid::new_v4();

    let mut person_directory_dao = MockPersonDirectoryDao::new();
    person_directory_dao
        .expect_find_directory_entry()
        .times(1)
        .returning(move |_person_id, _tx| Ok(Some(linked_row(person_id, sales_person_id))));
    person_directory_dao
        .expect_all_user_roles()
        .times(1)
        .returning(|_tx| {
            Ok(vec![UserRoleRow {
                user_name: "bea".into(),
                role_name: "hr".into(),
            }]
            .into())
        });

    let service = build_mock_service(
        person_directory_dao,
        MockPersonService::new(),
        build_perm(true),
        build_mock_transaction_dao(),
    );

    let entry = service
        .get_detail(person_id, Authentication::Context(()), None)
        .await
        .expect("get_detail should succeed for admin");

    assert_eq!(PersonKind::Linked, entry.kind);
    assert_eq!(Arc::<str>::from("Bea Verknuepft"), entry.display_name);
    assert_eq!(vec![Arc::<str>::from("hr")], entry.roles.to_vec());
}

/// M-3 (T-63-04-B): Die Kandidatenliste der Verkaeufer gatet auf `admin`,
/// bevor `PersonService` ueberhaupt angefasst wird. `Authentication::Full`
/// darf erst NACH dem Gate erreichbar sein.
#[tokio::test]
async fn unlinked_sales_person_candidates_ohne_admin_privileg_ist_forbidden() {
    let mut person_service = MockPersonService::new();
    person_service.expect_unlinked_sales_persons().times(0);

    let service = build_mock_service(
        MockPersonDirectoryDao::new(),
        person_service,
        build_perm(false),
        build_untouched_transaction_dao(),
    );

    let result = service
        .unlinked_sales_person_candidates(Authentication::Context(()), None)
        .await;

    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "ohne Admin-Privileg darf die Kandidatenliste nichts preisgeben: {result:?}",
    );
}

/// M-4 (T-63-04-B): dasselbe fuer die Benutzer-Kandidaten.
#[tokio::test]
async fn unlinked_user_candidates_ohne_admin_privileg_ist_forbidden() {
    let mut person_service = MockPersonService::new();
    person_service.expect_unlinked_users().times(0);

    let service = build_mock_service(
        MockPersonDirectoryDao::new(),
        person_service,
        build_perm(false),
        build_untouched_transaction_dao(),
    );

    let result = service
        .unlinked_user_candidates(Authentication::Context(()), None)
        .await;

    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "ohne Admin-Privileg darf die Kandidatenliste nichts preisgeben: {result:?}",
    );
}

// ─── Integrations-Harness (I-1 … I-8) ───────────────────────────────────────

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

struct PersonIntegrationDeps;
impl PersonServiceDeps for PersonIntegrationDeps {
    type Context = ();
    type Transaction = TransactionImpl;
    type PersonDao = PersonDaoImpl;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = TransactionDaoImpl;
}

struct SalesPersonIntegrationDeps;
impl SalesPersonServiceDeps for SalesPersonIntegrationDeps {
    type Context = ();
    type Transaction = TransactionImpl;
    type SalesPersonDao = dao_impl_sqlite::sales_person::SalesPersonDaoImpl;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = TransactionDaoImpl;
}

struct IntegrationDeps;
impl PersonDetailServiceDeps for IntegrationDeps {
    type Context = ();
    type Transaction = TransactionImpl;
    type PersonDirectoryDao = PersonDirectoryDaoImpl;
    type PersonDao = PersonDaoImpl;
    type PermissionDao = PermissionDaoImpl;
    type SalesPersonDao = dao_impl_sqlite::sales_person::SalesPersonDaoImpl;
    type PersonService = PersonServiceImpl<PersonIntegrationDeps>;
    type SalesPersonService = SalesPersonServiceImpl<SalesPersonIntegrationDeps>;
    type PermissionService = MockPermissionService;
    type TransactionDao = TransactionDaoImpl;
}

fn fixed_datetime() -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(
        time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
        time::Time::from_hms(23, 42, 0).unwrap(),
    )
}

fn build_clock() -> MockClockService {
    let mut clock_service = MockClockService::new();
    clock_service.expect_date_time_now().returning(fixed_datetime);
    clock_service
}

fn build_uuid() -> MockUuidService {
    let mut uuid_service = MockUuidService::new();
    uuid_service
        .expect_new_uuid()
        .returning(|_process| Uuid::new_v4());
    uuid_service
}

fn build_service(
    pool: Arc<sqlx::SqlitePool>,
    admin: bool,
) -> PersonDetailServiceImpl<IntegrationDeps> {
    // Der innere `PersonService` ist HR-gegated und bekommt einen eigenen
    // Permission-Mock. Der aeussere Service ruft ihn mit
    // `Authentication::Full` — erst nach seinem eigenen Admin-Gate.
    let person_service = PersonServiceImpl::<PersonIntegrationDeps> {
        person_dao: Arc::new(PersonDaoImpl::new(pool.clone())),
        permission_service: Arc::new(build_perm(false)),
        clock_service: Arc::new(build_clock()),
        uuid_service: Arc::new(build_uuid()),
        transaction_dao: Arc::new(TransactionDaoImpl::new(pool.clone())),
    };

    let sales_person_service = SalesPersonServiceImpl::<SalesPersonIntegrationDeps> {
        sales_person_dao: Arc::new(dao_impl_sqlite::sales_person::SalesPersonDaoImpl::new(
            pool.clone(),
        )),
        permission_service: Arc::new(build_perm(false)),
        clock_service: Arc::new(build_clock()),
        uuid_service: Arc::new(build_uuid()),
        transaction_dao: Arc::new(TransactionDaoImpl::new(pool.clone())),
    };

    PersonDetailServiceImpl::<IntegrationDeps> {
        person_directory_dao: Arc::new(PersonDirectoryDaoImpl::new(pool.clone())),
        person_dao: Arc::new(PersonDaoImpl::new(pool.clone())),
        permission_dao: Arc::new(PermissionDaoImpl::new(pool.clone())),
        sales_person_dao: Arc::new(dao_impl_sqlite::sales_person::SalesPersonDaoImpl::new(
            pool.clone(),
        )),
        person_service: Arc::new(person_service),
        sales_person_service: Arc::new(sales_person_service),
        permission_service: Arc::new(build_perm(admin)),
        transaction_dao: Arc::new(TransactionDaoImpl::new(pool.clone())),
    }
}

async fn detail(pool: &Arc<sqlx::SqlitePool>, person_id: Uuid) -> PersonEntry {
    build_service(pool.clone(), true)
        .get_detail(person_id, Authentication::Context(()), None)
        .await
        .expect("get_detail should succeed for admin")
}

// ─── Seed-Helfer ────────────────────────────────────────────────────────────
//
// Dateilokale Kopien der Gegenstuecke aus `person_directory.rs` /
// `person_detail_dao.rs` — die dortigen Helfer sind privat, und ihre
// Sichtbarkeit zu aendern ist kein Recht dieses Plans.

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

#[allow(clippy::too_many_arguments)]
async fn seed_sales_person(
    pool: &sqlx::SqlitePool,
    id: Uuid,
    name: &str,
    person_id: Uuid,
    deleted: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO sales_person \
         (id, name, background_color, is_paid, inactive, deleted, person_id, \
          update_process, update_version) \
         VALUES (?1, ?2, '#FFF', true, false, ?3, ?4, 'seed', ?5)",
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

async fn seed_live_sales_person(pool: &sqlx::SqlitePool, id: Uuid, name: &str, person_id: Uuid) {
    seed_sales_person(pool, id, name, person_id, None).await;
}

const DELETED_AT: &str = "2026-02-01T00:00:00.000000000Z";

async fn seed_deleted_sales_person(pool: &sqlx::SqlitePool, id: Uuid, name: &str, person_id: Uuid) {
    seed_sales_person(pool, id, name, person_id, Some(DELETED_AT)).await;
}

async fn seed_user(pool: &sqlx::SqlitePool, name: &str, person_id: Uuid) {
    sqlx::query("INSERT INTO user (name, person_id, update_process) VALUES (?1, ?2, 'seed')")
        .bind(name)
        .bind(person_id.as_bytes().to_vec())
        .execute(pool)
        .await
        .expect("seed user");
}

/// `admin`, `sales` und `hr` legen bereits die Migrationen als Rollen an —
/// hier wird nur die Zuordnung geschrieben.
async fn seed_user_role(pool: &sqlx::SqlitePool, user_name: &str, role_name: &str) {
    sqlx::query(
        "INSERT INTO user_role (user_name, role_name, update_process) VALUES (?1, ?2, 'seed')",
    )
    .bind(user_name)
    .bind(role_name)
    .execute(pool)
    .await
    .expect("seed user_role");
}

// ─── I-1 … I-6: get_detail ──────────────────────────────────────────────────

/// I-1 (PADM-05, PADM-06, D-62-01/D-62-03): Die Detailansicht einer
/// verknuepften Person traegt dieselben fertigen Felder wie die Listenzeile.
#[tokio::test]
async fn get_detail_verknuepfte_person_ist_linked() {
    let pool = setup_pool().await;
    let person_id = Uuid::new_v4();
    let sales_person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_live_sales_person(pool.as_ref(), sales_person_id, "Bea Verknuepft", person_id).await;
    seed_user(pool.as_ref(), "bea", person_id).await;

    let entry = detail(&pool, person_id).await;

    assert_eq!(person_id, entry.person_id);
    assert_eq!(PersonKind::Linked, entry.kind);
    assert_eq!(
        Arc::<str>::from("Bea Verknuepft"),
        entry.display_name,
        "der Verkaeufername gewinnt gegen den Benutzernamen (D-62-03)",
    );
    assert_eq!(Some(Arc::<str>::from("bea")), entry.username);
    assert_eq!(Some(sales_person_id), entry.sales_person_id);
    assert_eq!(Some(Arc::<str>::from("#FFF")), entry.background_color);
    assert_eq!(Some(true), entry.is_paid);
    assert_eq!(Some(false), entry.inactive);
}

/// I-2 (PADM-06): Ein reines Benutzerkonto traegt den Benutzernamen als
/// Anzeigenamen und hat keine Verkaeufer-Seite.
#[tokio::test]
async fn get_detail_nur_benutzer_ist_user_only() {
    let pool = setup_pool().await;
    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_user(pool.as_ref(), "carl", person_id).await;

    let entry = detail(&pool, person_id).await;

    assert_eq!(PersonKind::UserOnly, entry.kind);
    assert_eq!(Arc::<str>::from("carl"), entry.display_name);
    assert_eq!(Some(Arc::<str>::from("carl")), entry.username);
    assert_eq!(None, entry.sales_person_id);
    assert_eq!(None, entry.background_color);
    assert_eq!(None, entry.is_paid);
    assert_eq!(None, entry.inactive);
}

/// I-3 (D-62-03): Ein reiner Verkaeufer hat kein Benutzernamen-Feld, aber einen
/// gefuellten Anzeigenamen.
#[tokio::test]
async fn get_detail_nur_verkaeufer_ist_sales_person_only() {
    let pool = setup_pool().await;
    let person_id = Uuid::new_v4();
    let sales_person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_live_sales_person(pool.as_ref(), sales_person_id, "Dana Ohne Login", person_id).await;

    let entry = detail(&pool, person_id).await;

    assert_eq!(PersonKind::SalesPersonOnly, entry.kind);
    assert_eq!(Arc::<str>::from("Dana Ohne Login"), entry.display_name);
    assert_eq!(None, entry.username);
    assert_eq!(Some(sales_person_id), entry.sales_person_id);
    assert!(
        entry.roles.is_empty(),
        "ohne Benutzerkonto gibt es keine Rollen: {:?}",
        entry.roles,
    );
}

/// I-4: Eine unbekannte `person_id` ist `EntityNotFound` — nicht eine leere
/// oder halb gefuellte Antwort.
#[tokio::test]
async fn get_detail_unbekannte_person_ist_entity_not_found() {
    let pool = setup_pool().await;
    let unbekannt = Uuid::new_v4();

    let result = build_service(pool.clone(), true)
        .get_detail(unbekannt, Authentication::Context(()), None)
        .await;

    assert!(
        matches!(result, Err(ServiceError::EntityNotFound(id)) if id == unbekannt),
        "unbekannte person_id muss EntityNotFound liefern: {result:?}",
    );
}

/// I-5 (D-62-04): Eine traegerlos gewordene Person — nur eine soft-geloeschte
/// Sales Person zeigt noch auf sie — ist in der Detailansicht genauso
/// unsichtbar wie in der Liste.
#[tokio::test]
async fn get_detail_traegerlose_person_ist_entity_not_found() {
    let pool = setup_pool().await;
    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_deleted_sales_person(pool.as_ref(), Uuid::new_v4(), "Emil Geloescht", person_id).await;

    let result = build_service(pool.clone(), true)
        .get_detail(person_id, Authentication::Context(()), None)
        .await;

    assert!(
        matches!(result, Err(ServiceError::EntityNotFound(id)) if id == person_id),
        "eine traegerlos gewordene Person darf keine halbleere Antwort liefern: {result:?}",
    );
}

/// I-6 (D-62-06): Die Zeile traegt genau die Rollen ihres eigenen Benutzers.
/// Der naheliegende Kurzschluss — „es ist ja nur eine Person, also alle
/// gelesenen Rollen an sie haengen" — faellt hier um: `carl` hat `admin`, und
/// die Antwort fuer `bea` muss trotzdem nur `hr` enthalten. Per Mutation
/// verifiziert.
#[tokio::test]
async fn get_detail_traegt_nur_die_eigenen_rollen() {
    let pool = setup_pool().await;
    let bea = Uuid::new_v4();
    let carl = Uuid::new_v4();
    seed_person(pool.as_ref(), bea).await;
    seed_person(pool.as_ref(), carl).await;
    seed_user(pool.as_ref(), "bea", bea).await;
    seed_user(pool.as_ref(), "carl", carl).await;
    seed_user_role(pool.as_ref(), "bea", "hr").await;
    seed_user_role(pool.as_ref(), "carl", "admin").await;

    let entry = detail(&pool, bea).await;

    assert_eq!(
        vec![Arc::<str>::from("hr")],
        entry.roles.to_vec(),
        "nur die Rollen des eigenen Benutzers",
    );
}

// ─── I-7 / I-8: Kandidatenlisten ────────────────────────────────────────────

/// I-7 (PADM-04, SC-4): Eine bereits verknuepfte Sales Person ist kein
/// Kandidat fuer „Login hinzufuegen"; eine unverknuepfte schon.
#[tokio::test]
async fn unlinked_sales_person_candidates_blendet_verknuepfte_aus() {
    let pool = setup_pool().await;
    let verknuepft = Uuid::new_v4();
    let frei = Uuid::new_v4();
    seed_person(pool.as_ref(), verknuepft).await;
    seed_person(pool.as_ref(), frei).await;
    let verknuepfte_sales_person = Uuid::new_v4();
    let freie_sales_person = Uuid::new_v4();
    seed_live_sales_person(
        pool.as_ref(),
        verknuepfte_sales_person,
        "Bea Verknuepft",
        verknuepft,
    )
    .await;
    seed_user(pool.as_ref(), "bea", verknuepft).await;
    seed_live_sales_person(pool.as_ref(), freie_sales_person, "Dana Ohne Login", frei).await;

    let candidates = build_service(pool.clone(), true)
        .unlinked_sales_person_candidates(Authentication::Context(()), None)
        .await
        .expect("candidates should succeed for admin");

    let ids: Vec<Uuid> = candidates.iter().map(|sp| sp.id).collect();
    assert!(
        ids.contains(&freie_sales_person),
        "die unverknuepfte Sales Person muss Kandidat sein: {ids:?}",
    );
    assert!(
        !ids.contains(&verknuepfte_sales_person),
        "eine bereits verknuepfte Sales Person darf nicht erscheinen (SC-4): {ids:?}",
    );
}

/// I-8 (PADM-04, SC-4): Ein bereits verknuepfter Benutzer ist kein Kandidat
/// fuer „Person hinzufuegen"; ein unverknuepfter schon.
#[tokio::test]
async fn unlinked_user_candidates_blendet_verknuepfte_aus() {
    let pool = setup_pool().await;
    let verknuepft = Uuid::new_v4();
    let frei = Uuid::new_v4();
    seed_person(pool.as_ref(), verknuepft).await;
    seed_person(pool.as_ref(), frei).await;
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "Bea Verknuepft", verknuepft).await;
    seed_user(pool.as_ref(), "bea", verknuepft).await;
    seed_user(pool.as_ref(), "carl", frei).await;

    let candidates = build_service(pool.clone(), true)
        .unlinked_user_candidates(Authentication::Context(()), None)
        .await
        .expect("candidates should succeed for admin");

    let names: Vec<Arc<str>> = candidates.to_vec();
    assert!(
        names.contains(&Arc::<str>::from("carl")),
        "der unverknuepfte Benutzer muss Kandidat sein: {names:?}",
    );
    assert!(
        !names.contains(&Arc::<str>::from("bea")),
        "ein bereits verknuepfter Benutzer darf nicht erscheinen (SC-4): {names:?}",
    );
}

/// Der `SalesPersonService` ist Dependency fuer den Schreibpfad in Plan 63-05
/// und wird vom Lesepfad nicht aufgerufen. Dieser Test haelt fest, dass die
/// Integrations-Verdrahtung ihn trotzdem konstruierbar macht — sonst faellt der
/// Fehler erst in 63-05 auf, in einem fremden Plan.
#[tokio::test]
async fn sales_person_service_dependency_ist_verdrahtet() {
    let pool = setup_pool().await;
    let service = build_service(pool.clone(), true);

    let all = service
        .sales_person_service
        .get_all(Authentication::Full, None)
        .await
        .expect("der verdrahtete SalesPersonService ist benutzbar");

    assert!(all.is_empty(), "frische Datenbank hat keine Sales Persons");
}

// ─── C-1 … C-8: create_combined (Plan 63-05 / PADM-02 / SC-2) ───────────────

fn create_request(
    display_name: &str,
    wants_user: bool,
    wants_sales_person: bool,
    username: Option<&str>,
) -> CreatePersonEntry {
    CreatePersonEntry {
        display_name: display_name.into(),
        wants_user,
        wants_sales_person,
        username: username.map(Arc::<str>::from),
    }
}

async fn count(pool: &sqlx::SqlitePool, sql: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(sql)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|err| panic!("count query failed ({sql}): {err}"))
}

fn uuid_from_blob(bytes: Vec<u8>) -> Uuid {
    Uuid::from_slice(&bytes).expect("person_id ist ein 16-Byte-Blob")
}

async fn person_of_user(pool: &sqlx::SqlitePool, name: &str) -> Option<Uuid> {
    sqlx::query_scalar::<_, Option<Vec<u8>>>("SELECT person_id FROM user WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await
        .expect("person_id of user")
        .flatten()
        .map(uuid_from_blob)
}

async fn person_of_sales_person(pool: &sqlx::SqlitePool, name: &str) -> Option<Uuid> {
    sqlx::query_scalar::<_, Option<Vec<u8>>>("SELECT person_id FROM sales_person WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await
        .expect("person_id of sales_person")
        .flatten()
        .map(uuid_from_blob)
}

async fn create_combined(
    pool: &Arc<sqlx::SqlitePool>,
    admin: bool,
    request: &CreatePersonEntry,
) -> Result<Uuid, ServiceError> {
    build_service(pool.clone(), admin)
        .create_combined(request, Authentication::Context(()), None)
        .await
}

/// C-1 (SC-2, PADM-02): Beide Umschalter an → genau ein Benutzer, genau eine
/// Sales Person, **eine** gemeinsame Person. Die Rueckgabe ist genau diese
/// `person_id` — das Ziel der Weiterleitung in die Detailansicht.
#[tokio::test]
async fn create_combined_beide_auspraegungen_teilen_eine_person() {
    let pool = setup_pool().await;

    let person_id = create_combined(&pool, true, &create_request("Bea Neu", true, true, Some("bea")))
        .await
        .expect("create_combined should succeed for admin");

    assert_eq!(
        Some(person_id),
        person_of_user(pool.as_ref(), "bea").await,
        "der Benutzer haengt an der zurueckgegebenen Person",
    );
    assert_eq!(
        Some(person_id),
        person_of_sales_person(pool.as_ref(), "Bea Neu").await,
        "die Sales Person haengt an derselben Person (SC-2)",
    );
    assert_eq!(
        1,
        count(pool.as_ref(), "SELECT COUNT(*) FROM person").await,
        "genau EINE person-Zeile — kein zweites Anlegen mit anschliessendem Zusammenfuehren",
    );
    assert_eq!(1, count(pool.as_ref(), "SELECT COUNT(*) FROM user").await);
    assert_eq!(
        1,
        count(pool.as_ref(), "SELECT COUNT(*) FROM sales_person").await
    );

    let entry = detail(&pool, person_id).await;
    assert_eq!(PersonKind::Linked, entry.kind);
    assert_eq!(Arc::<str>::from("Bea Neu"), entry.display_name);
}

/// C-2 (SC-2): Nur der Verkaeufer-Umschalter → keine `user`-Zeile.
#[tokio::test]
async fn create_combined_nur_verkaeufer_legt_keinen_benutzer_an() {
    let pool = setup_pool().await;

    let person_id = create_combined(
        &pool,
        true,
        &create_request("Dana Ohne Login", false, true, None),
    )
    .await
    .expect("create_combined should succeed for admin");

    assert_eq!(
        Some(person_id),
        person_of_sales_person(pool.as_ref(), "Dana Ohne Login").await,
    );
    assert_eq!(
        0,
        count(pool.as_ref(), "SELECT COUNT(*) FROM user").await,
        "ohne gesetzten Umschalter entsteht KEIN Benutzerkonto",
    );

    let entry = detail(&pool, person_id).await;
    assert_eq!(PersonKind::SalesPersonOnly, entry.kind);
}

/// C-3 (SC-2): Nur der Benutzer-Umschalter → keine `sales_person`-Zeile. Die
/// Identitaet vergibt das DAO (D-61-01) und wird rueckwaerts aufgeloest.
#[tokio::test]
async fn create_combined_nur_benutzer_legt_keine_sales_person_an() {
    let pool = setup_pool().await;

    let person_id = create_combined(&pool, true, &create_request("egal", true, false, Some("carl")))
        .await
        .expect("create_combined should succeed for admin");

    assert_eq!(Some(person_id), person_of_user(pool.as_ref(), "carl").await);
    assert_eq!(
        0,
        count(pool.as_ref(), "SELECT COUNT(*) FROM sales_person").await,
        "ohne gesetzten Umschalter entsteht KEINE Sales Person",
    );

    let entry = detail(&pool, person_id).await;
    assert_eq!(PersonKind::UserOnly, entry.kind);
    assert_eq!(Arc::<str>::from("carl"), entry.display_name);
}

/// C-4 (D-63-07, T-63-05-B, ASVS V5): Ein leerer Auswahl-Satz wird
/// **serverseitig** abgewiesen — die Regel ist keine reine Client-Absicherung.
/// Und sie faellt vor jedem Schreibzugriff.
#[tokio::test]
async fn create_combined_ohne_auswahl_ist_validation_error() {
    let pool = setup_pool().await;

    let result = create_combined(&pool, true, &create_request("Niemand", false, false, None)).await;

    assert!(
        matches!(result, Err(ServiceError::ValidationError(_))),
        "ohne einen einzigen Umschalter muss der Server ablehnen: {result:?}",
    );
    assert_eq!(
        0,
        count(pool.as_ref(), "SELECT COUNT(*) FROM person").await,
        "die Ablehnung darf nichts geschrieben haben",
    );
}

/// C-5 (D-63-07): `wants_user` ohne brauchbaren Benutzernamen ist ungueltig —
/// fehlend wie auch nur aus Leerzeichen bestehend.
#[tokio::test]
async fn create_combined_ohne_benutzernamen_ist_validation_error() {
    let pool = setup_pool().await;

    let fehlend = create_combined(&pool, true, &create_request("Bea", true, true, None)).await;
    assert!(
        matches!(fehlend, Err(ServiceError::ValidationError(_))),
        "fehlender Benutzername muss abgelehnt werden: {fehlend:?}",
    );

    let leer = create_combined(&pool, true, &create_request("Bea", true, true, Some("   "))).await;
    assert!(
        matches!(leer, Err(ServiceError::ValidationError(_))),
        "ein Benutzername aus Leerzeichen muss abgelehnt werden: {leer:?}",
    );

    assert_eq!(
        0,
        count(pool.as_ref(), "SELECT COUNT(*) FROM person").await,
        "keine der beiden Ablehnungen darf eine halbe Anlage hinterlassen",
    );
}

/// C-6 (Gegenstueck zu C-5): `wants_sales_person` ohne Anzeigenamen ist
/// ungueltig — ein namenloser Verkaeufer waere im Verzeichnis unauffindbar.
#[tokio::test]
async fn create_combined_ohne_anzeigenamen_ist_validation_error() {
    let pool = setup_pool().await;

    let result = create_combined(&pool, true, &create_request("  ", false, true, None)).await;

    assert!(
        matches!(result, Err(ServiceError::ValidationError(_))),
        "ein leerer Anzeigename muss abgelehnt werden: {result:?}",
    );
    assert_eq!(
        0,
        count(pool.as_ref(), "SELECT COUNT(*) FROM sales_person").await
    );
}

/// C-7 (T-63-05-A, PADM-08): Ohne Admin-Privileg wird nichts geschrieben.
#[tokio::test]
async fn create_combined_ohne_admin_privileg_ist_forbidden() {
    let pool = setup_pool().await;

    let result = create_combined(&pool, false, &create_request("Bea Neu", true, true, Some("bea"))).await;

    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "ohne Admin-Privileg muss create_combined Forbidden liefern: {result:?}",
    );
    assert_eq!(
        0,
        count(pool.as_ref(), "SELECT COUNT(*) FROM person").await,
        "das Gate liegt vor jedem Schreibzugriff",
    );
}

/// C-8 (SC-2, T-63-05-C — der Atomaritaets-Nachweis): Schlaegt die
/// Benutzer-Anlage fehl, weil der Name schon vergeben ist, bleibt **keine**
/// Sales Person und **keine** neue Person zurueck. Ohne die geteilte
/// Transaktion aus Plan 63-01 waere die Sales Person hier bereits geschrieben
/// und nicht mehr rueckrollbar.
#[tokio::test]
async fn create_combined_teilfehler_laesst_keine_haelfte_zurueck() {
    let pool = setup_pool().await;
    let bestehend = Uuid::new_v4();
    seed_person(pool.as_ref(), bestehend).await;
    seed_user(pool.as_ref(), "bea", bestehend).await;

    let result = create_combined(&pool, true, &create_request("Bea Neu", true, true, Some("bea"))).await;

    assert!(
        result.is_err(),
        "ein bereits vergebener Benutzername muss scheitern: {result:?}",
    );
    assert_eq!(
        0,
        count(
            pool.as_ref(),
            "SELECT COUNT(*) FROM sales_person WHERE name = 'Bea Neu'"
        )
        .await,
        "die bereits geschriebene Sales Person muss mit zurueckgerollt sein (SC-2)",
    );
    assert_eq!(
        1,
        count(pool.as_ref(), "SELECT COUNT(*) FROM person").await,
        "es darf keine neue person-Zeile uebrig bleiben",
    );
    assert_eq!(
        1,
        count(pool.as_ref(), "SELECT COUNT(*) FROM user").await,
        "der bestehende Benutzer bleibt unveraendert",
    );
}

// ─── A-1 … A-7 / G-1 … G-3: add_facet und get_detail_by_sales_person ────────
//                            (Plan 63-05 / PADM-03 / SC-3 / SC-4)

async fn add_facet(
    pool: &Arc<sqlx::SqlitePool>,
    admin: bool,
    person_id: Uuid,
    facet: PersonFacet,
    display_name: Option<&str>,
    username: Option<&str>,
) -> Result<Uuid, ServiceError> {
    build_service(pool.clone(), admin)
        .add_facet(
            person_id,
            facet,
            display_name.map(Arc::<str>::from),
            username.map(Arc::<str>::from),
            Authentication::Context(()),
            None,
        )
        .await
}

async fn person_exists(pool: &sqlx::SqlitePool, person_id: Uuid) -> bool {
    sqlx::query_scalar::<_, Vec<u8>>("SELECT id FROM person WHERE id = ?")
        .bind(person_id.as_bytes().to_vec())
        .fetch_optional(pool)
        .await
        .expect("person lookup")
        .is_some()
}

/// Version und Name der Sales Person — der Nachweis, dass das bereits
/// vorhandene Objekt beim Zuschalten der Gegenseite **unveraendert** bleibt
/// (SC-3).
async fn sales_person_fingerprint(pool: &sqlx::SqlitePool, id: Uuid) -> (String, Vec<u8>) {
    sqlx::query_as::<_, (String, Vec<u8>)>(
        "SELECT name, update_version FROM sales_person WHERE id = ?",
    )
    .bind(id.as_bytes().to_vec())
    .fetch_one(pool)
    .await
    .expect("sales person fingerprint")
}

/// A-1 (PADM-03, SC-3): „Login hinzufuegen" auf einem reinen Verkaeufer-
/// Eintrag. Der Benutzer entsteht auf derselben Person, die Sales Person
/// bleibt unangetastet, und die zurueckgegebene id ist die unveraenderte
/// Eingabe — dieser Pfad fuehrt keine Identitaeten zusammen.
#[tokio::test]
async fn add_facet_benutzer_auf_verkaeufer_eintrag_laesst_verkaeufer_unveraendert() {
    let pool = setup_pool().await;
    let person_id = Uuid::new_v4();
    let sales_person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_live_sales_person(pool.as_ref(), sales_person_id, "Dana Ohne Login", person_id).await;
    let vorher = sales_person_fingerprint(pool.as_ref(), sales_person_id).await;

    let resultierend = add_facet(
        &pool,
        true,
        person_id,
        PersonFacet::User,
        None,
        Some("dana"),
    )
    .await
    .expect("add_facet should succeed for admin");

    assert_eq!(
        person_id, resultierend,
        "ohne Zusammenfuehren behaelt der Eintrag seine person_id",
    );
    assert_eq!(Some(person_id), person_of_user(pool.as_ref(), "dana").await);
    assert_eq!(
        vorher,
        sales_person_fingerprint(pool.as_ref(), sales_person_id).await,
        "die vorhandene Sales Person bleibt unveraendert — gleicher Name, gleiche Version (SC-3)",
    );

    let entry = detail(&pool, resultierend).await;
    assert_eq!(PersonKind::Linked, entry.kind);
}

/// A-2 (PADM-03, SC-3, D-61-03): „Person hinzufuegen" auf einem reinen
/// Benutzer-Eintrag. Hier fuehrt der bestehende Schreibpfad zwei Identitaeten
/// zusammen, und **die Sales Person gewinnt** — der Eintrag wechselt seine
/// `person_id`. Der Test prueft beide Seiten: die Rueckgabe ist NICHT die
/// Eingabe, und die urspruengliche Person existiert nicht mehr.
#[tokio::test]
async fn add_facet_verkaeufer_auf_benutzer_eintrag_liefert_neue_person_id() {
    let pool = setup_pool().await;
    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_user(pool.as_ref(), "carl", person_id).await;

    let resultierend = add_facet(
        &pool,
        true,
        person_id,
        PersonFacet::SalesPerson,
        Some("Carl Verkauf"),
        None,
    )
    .await
    .expect("add_facet should succeed for admin");

    assert_ne!(
        person_id, resultierend,
        "der Merge laesst die Sales Person gewinnen — die person_id wechselt (D-61-03)",
    );
    assert!(
        !person_exists(pool.as_ref(), person_id).await,
        "die absorbierte Person darf nicht zurueckbleiben",
    );
    assert_eq!(
        Some(resultierend),
        person_of_sales_person(pool.as_ref(), "Carl Verkauf").await,
    );
    assert_eq!(
        Some(resultierend),
        person_of_user(pool.as_ref(), "carl").await,
        "der Benutzer ist auf die gewinnende Person gewandert",
    );
    assert_eq!(
        1,
        count(pool.as_ref(), "SELECT COUNT(*) FROM person").await,
        "genau eine Person bleibt uebrig",
    );

    let entry = detail(&pool, resultierend).await;
    assert_eq!(PersonKind::Linked, entry.kind);
    assert_eq!(Arc::<str>::from("Carl Verkauf"), entry.display_name);
}

/// A-3 (D-61-04): Die Benutzer-Seite ist bereits besetzt → 409-Vertrag, kein
/// Schreibzugriff.
#[tokio::test]
async fn add_facet_auf_besetzte_benutzer_seite_ist_entity_already_exists() {
    let pool = setup_pool().await;
    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "Bea Verknuepft", person_id).await;
    seed_user(pool.as_ref(), "bea", person_id).await;

    let result = add_facet(
        &pool,
        true,
        person_id,
        PersonFacet::User,
        None,
        Some("zweitkonto"),
    )
    .await;

    assert!(
        matches!(result, Err(ServiceError::EntityAlreadyExists(id)) if id == person_id),
        "eine besetzte Seite darf nicht stillschweigend ueberschrieben werden: {result:?}",
    );
    assert_eq!(
        1,
        count(pool.as_ref(), "SELECT COUNT(*) FROM user").await,
        "die Ablehnung darf keinen zweiten Benutzer angelegt haben",
    );
}

/// A-4 (D-61-04, Gegenstueck zu A-3): Die Verkaeufer-Seite ist bereits besetzt.
#[tokio::test]
async fn add_facet_auf_besetzte_verkaeufer_seite_ist_entity_already_exists() {
    let pool = setup_pool().await;
    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "Bea Verknuepft", person_id).await;
    seed_user(pool.as_ref(), "bea", person_id).await;

    let result = add_facet(
        &pool,
        true,
        person_id,
        PersonFacet::SalesPerson,
        Some("Zweiter Verkaeufer"),
        None,
    )
    .await;

    assert!(
        matches!(result, Err(ServiceError::EntityAlreadyExists(id)) if id == person_id),
        "eine besetzte Seite darf nicht stillschweigend ueberschrieben werden: {result:?}",
    );
    assert_eq!(
        1,
        count(pool.as_ref(), "SELECT COUNT(*) FROM sales_person").await,
        "die Ablehnung darf keine zweite Sales Person angelegt haben",
    );
}

/// A-5: Fehlendes Pflichtfeld je Seite — Benutzername bei `User`,
/// Anzeigename bei `SalesPerson`.
#[tokio::test]
async fn add_facet_ohne_pflichtfeld_ist_validation_error() {
    let pool = setup_pool().await;
    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "Dana Ohne Login", person_id).await;

    let ohne_benutzername = add_facet(&pool, true, person_id, PersonFacet::User, None, None).await;
    assert!(
        matches!(ohne_benutzername, Err(ServiceError::ValidationError(_))),
        "User ohne Benutzernamen muss abgelehnt werden: {ohne_benutzername:?}",
    );

    let leerer_anzeigename = add_facet(
        &pool,
        true,
        person_id,
        PersonFacet::SalesPerson,
        Some("  "),
        None,
    )
    .await;
    assert!(
        matches!(leerer_anzeigename, Err(ServiceError::ValidationError(_))),
        "SalesPerson ohne Anzeigenamen muss abgelehnt werden: {leerer_anzeigename:?}",
    );

    assert_eq!(0, count(pool.as_ref(), "SELECT COUNT(*) FROM user").await);
}

/// A-6: Eine unbekannte `person_id` ist `EntityNotFound` — dieselbe
/// Sichtbarkeitsregel wie im Lesepfad.
#[tokio::test]
async fn add_facet_unbekannte_person_ist_entity_not_found() {
    let pool = setup_pool().await;
    let unbekannt = Uuid::new_v4();

    let result = add_facet(
        &pool,
        true,
        unbekannt,
        PersonFacet::User,
        None,
        Some("niemand"),
    )
    .await;

    assert!(
        matches!(result, Err(ServiceError::EntityNotFound(id)) if id == unbekannt),
        "unbekannte person_id muss EntityNotFound liefern: {result:?}",
    );
    assert_eq!(0, count(pool.as_ref(), "SELECT COUNT(*) FROM user").await);
}

/// A-7 (T-63-05-A, PADM-08): Ohne Admin-Privileg wird nichts geschrieben.
#[tokio::test]
async fn add_facet_ohne_admin_privileg_ist_forbidden() {
    let pool = setup_pool().await;
    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "Dana Ohne Login", person_id).await;

    let result = add_facet(
        &pool,
        false,
        person_id,
        PersonFacet::User,
        None,
        Some("dana"),
    )
    .await;

    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "ohne Admin-Privileg muss add_facet Forbidden liefern: {result:?}",
    );
    assert_eq!(0, count(pool.as_ref(), "SELECT COUNT(*) FROM user").await);
}

/// G-1 (SC-4): Ueber die `sales_person_id` kommt genau dieselbe Zeile heraus
/// wie ueber die `person_id` — der Weg zurueck in die Detailansicht, nachdem
/// ein bestehender Verkaeufer verknuepft wurde und die Identitaet gewechselt
/// hat.
#[tokio::test]
async fn get_detail_by_sales_person_liefert_dieselbe_zeile_wie_get_detail() {
    let pool = setup_pool().await;
    let person_id = Uuid::new_v4();
    let sales_person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_live_sales_person(pool.as_ref(), sales_person_id, "Bea Verknuepft", person_id).await;
    seed_user(pool.as_ref(), "bea", person_id).await;
    seed_user_role(pool.as_ref(), "bea", "hr").await;

    let ueber_person = detail(&pool, person_id).await;
    let ueber_sales_person = build_service(pool.clone(), true)
        .get_detail_by_sales_person(sales_person_id, Authentication::Context(()), None)
        .await
        .expect("get_detail_by_sales_person should succeed for admin");

    assert_eq!(ueber_person, ueber_sales_person);
    assert_eq!(person_id, ueber_sales_person.person_id);
}

/// G-2: Eine Sales Person ohne aufloesbare Person ist
/// `EntityNotFoundGeneric` — die Id steht als Text drin, weil der Fehler sich
/// nicht auf eine `person_id` beziehen kann.
#[tokio::test]
async fn get_detail_by_sales_person_ohne_aufloesbare_person_ist_entity_not_found_generic() {
    let pool = setup_pool().await;

    let result = build_service(pool.clone(), true)
        .get_detail_by_sales_person(Uuid::new_v4(), Authentication::Context(()), None)
        .await;

    assert!(
        matches!(result, Err(ServiceError::EntityNotFoundGeneric(_))),
        "eine unbekannte sales_person_id muss EntityNotFoundGeneric liefern: {result:?}",
    );
}

/// G-3 (T-63-05-A): Auch dieser Lesepfad gatet auf `admin`.
#[tokio::test]
async fn get_detail_by_sales_person_ohne_admin_privileg_ist_forbidden() {
    let pool = setup_pool().await;

    let result = build_service(pool.clone(), false)
        .get_detail_by_sales_person(Uuid::new_v4(), Authentication::Context(()), None)
        .await;

    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "ohne Admin-Privileg darf auch dieser Lesepfad nichts preisgeben: {result:?}",
    );
}
