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
use service::person_detail::PersonDetailService;
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
