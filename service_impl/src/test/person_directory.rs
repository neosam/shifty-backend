//! Phase 62 Plan 03 (PADM-01 / PADM-06 / PADM-08) — Tests fuer den
//! Business-Logic-Aggregat-Service `PersonDirectoryService`.
//!
//! Die Tests fahren gegen eine echte SQLite-in-memory-DB (`sqlx::migrate!`) und
//! gegen den echten `PersonDirectoryDaoImpl` — bewusst **ohne** DAO-Mock: das
//! Verhalten, das hier zu beweisen ist (Klassifikation, Anzeigename,
//! Sortierung, Rollen-Zuordnung), haengt an der tatsaechlichen Zeilenmenge des
//! Drei-Wege-Outer-Joins. Gemockt ist nur das Gate (`MockPermissionService`) —
//! es ist die einzige Abhaengigkeit, deren Antwort der Test variieren muss.
//!
//! Harness und Seed-Helfer sind dateilokale Kopien der Gegenstuecke aus
//! `person_candidates.rs` / `person_directory_dao.rs`; die dortigen Helfer sind
//! privat, und ihre Sichtbarkeit zu aendern ist kein Recht dieses Plans.
//!
//! Abgedeckte Invarianten:
//!
//! - **S-1** (PADM-01, D-62-01): Alle drei Auspraegungen stehen nebeneinander,
//!   jede mit gesetztem `kind`
//!   (`liste_enthaelt_alle_drei_auspraegungen`).
//! - **S-2** (PADM-06): Ein reines Benutzerkonto traegt den Benutzernamen als
//!   Anzeigenamen (`display_name_reines_benutzerkonto_ist_username`).
//! - **S-3** (D-62-03): Die verknuepfte Zeile traegt den Verkaeufernamen als
//!   Anzeigenamen UND den Benutzernamen im eigenen Feld
//!   (`display_name_verknuepft_ist_verkaeufername`).
//! - **S-4** (D-62-03): Der reine Verkaeufer hat kein Benutzernamen-Feld, aber
//!   einen gefuellten Anzeigenamen
//!   (`username_bei_reinem_verkaeufer_ist_leer`).
//! - **S-5**: Die Liste ist case-insensitive nach Anzeigename sortiert
//!   (`liste_ist_case_insensitive_nach_display_name_sortiert`). `zora`,
//!   `Anton`, `mia` ergeben `Anton`, `mia`, `zora` — eine ASCII-Sortierung auf
//!   der Rohform wuerde `Anton`, `mia`, `zora` zwar zufaellig auch liefern,
//!   darum traegt die Fixture zusaetzlich einen Grossbuchstaben-Nachzuegler.
//! - **S-6** (D-62-06): Rollen landen beim richtigen Benutzer; der reine
//!   Verkaeufer hat eine leere Rollenliste
//!   (`rollen_werden_dem_richtigen_benutzer_zugeordnet`).
//! - **S-7** (D-62-04): Soft-geloeschte Verkaeufer erscheinen nicht, inaktive
//!   schon (`soft_geloeschter_verkaeufer_erscheint_nicht_inaktiver_schon`).
//! - **S-8** (PADM-08, D-62-08, T-62-01/T-62-05): Ohne Admin-Privileg liefert
//!   `list` einen Fehler — **keine** leere Liste, **keine** redigierten Felder
//!   (`list_ohne_admin_privileg_ist_forbidden`).
//!
//! Der Permission-Mock antwortet nur auf das Literal `"admin"` positiv. Ein
//! Service, der hier versehentlich das HR-Privileg des Basic-Tier
//! `PersonService` uebernimmt (RESEARCH.md Pitfall 4), faellt damit in **jedem**
//! Test dieser Datei um, nicht nur im Gate-Test.

use std::sync::Arc;

use dao_impl_sqlite::person_directory::PersonDirectoryDaoImpl;
use dao_impl_sqlite::{TransactionDaoImpl, TransactionImpl};
use service::permission::Authentication;
use service::person_directory::{PersonDirectoryService, PersonEntry, PersonKind};
use service::{MockPermissionService, ServiceError};
use uuid::Uuid;

use crate::person_directory::{PersonDirectoryServiceDeps, PersonDirectoryServiceImpl};

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
impl PersonDirectoryServiceDeps for IntegrationDeps {
    type Context = ();
    type Transaction = TransactionImpl;
    type PersonDirectoryDao = PersonDirectoryDaoImpl;
    type PermissionService = MockPermissionService;
    type TransactionDao = TransactionDaoImpl;
}

/// Permission-Mock fuer D-62-08. Erfolgreich **nur** beim Literal `"admin"` und
/// nur, wenn der Test das Privileg zugesteht. Damit ist das erwartete Gate Teil
/// jeder Assertion dieser Datei und nicht nur eines Grep-Kriteriums.
fn build_perm(admin: bool) -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(move |privilege, _context| {
            if admin && privilege == "admin" {
                Ok(())
            } else {
                Err(ServiceError::Forbidden)
            }
        });
    permission_service
}

fn build_service(
    pool: Arc<sqlx::SqlitePool>,
    perm: MockPermissionService,
) -> PersonDirectoryServiceImpl<IntegrationDeps> {
    PersonDirectoryServiceImpl::<IntegrationDeps> {
        person_directory_dao: Arc::new(PersonDirectoryDaoImpl::new(pool.clone())),
        permission_service: Arc::new(perm),
        transaction_dao: Arc::new(TransactionDaoImpl::new(pool.clone())),
    }
}

/// Verzeichnis als Admin — der Normalpfad aller Tests ausser S-8.
async fn directory(pool: &Arc<sqlx::SqlitePool>) -> Vec<PersonEntry> {
    build_service(pool.clone(), build_perm(true))
        .list(Authentication::Context(()), None)
        .await
        .expect("list should succeed for admin")
        .to_vec()
}

fn entry_of(entries: &[PersonEntry], person_id: Uuid) -> &PersonEntry {
    entries
        .iter()
        .find(|entry| entry.person_id == person_id)
        .unwrap_or_else(|| panic!("Person {person_id} fehlt im Verzeichnis: {entries:?}"))
}

// ─── Seed-Helfer ────────────────────────────────────────────────────────────

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
    background_color: &str,
    is_paid: bool,
    inactive: bool,
    deleted: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO sales_person \
         (id, name, background_color, is_paid, inactive, deleted, person_id, \
          update_process, update_version) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'seed', ?8)",
    )
    .bind(id.as_bytes().to_vec())
    .bind(name)
    .bind(background_color)
    .bind(is_paid)
    .bind(inactive)
    .bind(deleted)
    .bind(person_id.as_bytes().to_vec())
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(pool)
    .await
    .expect("seed sales_person");
}

/// Lebende, bezahlte, aktive Sales Person — der Normalfall.
async fn seed_live_sales_person(pool: &sqlx::SqlitePool, id: Uuid, name: &str, person_id: Uuid) {
    seed_sales_person(pool, id, name, person_id, "#FFF", true, false, None).await;
}

const DELETED_AT: &str = "2026-02-01T00:00:00.000000000Z";

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

// ─── Fixture: alle drei Auspraegungen ───────────────────────────────────────

const LINKED_SALES_PERSON_NAME: &str = "Bea Verknuepft";
const LINKED_USER: &str = "bea";
const LONELY_USER: &str = "carl";
const LONELY_SALES_PERSON_NAME: &str = "Dana Ohne Login";

struct ThreeKinds {
    linked: Uuid,
    user_only: Uuid,
    sales_only: Uuid,
    linked_sales_person_id: Uuid,
    lonely_sales_person_id: Uuid,
}

async fn seed_three_kinds(pool: &sqlx::SqlitePool) -> ThreeKinds {
    let linked = Uuid::new_v4();
    let user_only = Uuid::new_v4();
    let sales_only = Uuid::new_v4();
    for person in [linked, user_only, sales_only] {
        seed_person(pool, person).await;
    }

    let linked_sales_person_id = Uuid::new_v4();
    let lonely_sales_person_id = Uuid::new_v4();
    seed_live_sales_person(pool, linked_sales_person_id, LINKED_SALES_PERSON_NAME, linked).await;
    seed_live_sales_person(
        pool,
        lonely_sales_person_id,
        LONELY_SALES_PERSON_NAME,
        sales_only,
    )
    .await;
    seed_user(pool, LINKED_USER, linked).await;
    seed_user(pool, LONELY_USER, user_only).await;

    ThreeKinds {
        linked,
        user_only,
        sales_only,
        linked_sales_person_id,
        lonely_sales_person_id,
    }
}

// ─── S-1 (PADM-01, D-62-01) ─────────────────────────────────────────────────

/// Genau das loest die Tabs ab: verknuepfte Person, reines Benutzerkonto und
/// reiner Verkaeufer stehen in **einer** Liste, jeder mit gesetzter
/// Auspraegung.
#[tokio::test]
async fn liste_enthaelt_alle_drei_auspraegungen() {
    let pool = setup_pool().await;
    let fixture = seed_three_kinds(pool.as_ref()).await;

    let entries = directory(&pool).await;

    assert_eq!(
        3,
        entries.len(),
        "drei Traeger-Konstellationen ergeben genau drei Eintraege: {entries:?}"
    );
    assert_eq!(
        PersonKind::Linked,
        entry_of(&entries, fixture.linked).kind,
        "Person mit Verkaeufer UND Benutzerkonto ist Linked"
    );
    assert_eq!(
        PersonKind::UserOnly,
        entry_of(&entries, fixture.user_only).kind,
        "Person mit nur einem Benutzerkonto ist UserOnly"
    );
    assert_eq!(
        PersonKind::SalesPersonOnly,
        entry_of(&entries, fixture.sales_only).kind,
        "Person mit nur einem Verkaeufer ist SalesPersonOnly"
    );

    assert_eq!(
        Some(fixture.linked_sales_person_id),
        entry_of(&entries, fixture.linked).sales_person_id,
    );
    assert_eq!(
        Some(fixture.lonely_sales_person_id),
        entry_of(&entries, fixture.sales_only).sales_person_id,
    );
    assert_eq!(
        None,
        entry_of(&entries, fixture.user_only).sales_person_id,
        "ein reines Benutzerkonto traegt keine Verkaeufer-Seite"
    );
}

// ─── S-2 (PADM-06) ──────────────────────────────────────────────────────────

/// PADM-06: Der Anzeigename kommt fertig aus dem Service. Ein reines
/// Benutzerkonto hat keinen Verkaeufernamen — dann traegt der Benutzername die
/// Zeile. Leer darf er nie sein.
#[tokio::test]
async fn display_name_reines_benutzerkonto_ist_username() {
    let pool = setup_pool().await;
    let fixture = seed_three_kinds(pool.as_ref()).await;

    let entries = directory(&pool).await;
    let entry = entry_of(&entries, fixture.user_only);

    assert_eq!(
        Arc::<str>::from(LONELY_USER),
        entry.display_name,
        "ohne Verkaeufer-Seite ist der Benutzername der Anzeigename"
    );
    assert!(
        !entry.display_name.is_empty(),
        "der Anzeigename ist nie leer (PADM-06)"
    );
    assert_eq!(Some(Arc::<str>::from(LONELY_USER)), entry.username);
}

// ─── S-3 (D-62-03) ──────────────────────────────────────────────────────────

/// D-62-03: Bei der verknuepften Zeile gewinnt der Verkaeufername; der
/// Benutzername bleibt daneben sichtbar, er wird nicht verdraengt.
#[tokio::test]
async fn display_name_verknuepft_ist_verkaeufername() {
    let pool = setup_pool().await;
    let fixture = seed_three_kinds(pool.as_ref()).await;

    let entries = directory(&pool).await;
    let entry = entry_of(&entries, fixture.linked);

    assert_eq!(
        Arc::<str>::from(LINKED_SALES_PERSON_NAME),
        entry.display_name,
        "bei einer verknuepften Person gewinnt der Verkaeufername"
    );
    assert_eq!(
        Some(Arc::<str>::from(LINKED_USER)),
        entry.username,
        "der Benutzername bleibt in seinem eigenen Feld erhalten"
    );
    assert_ne!(
        Some(entry.display_name.clone()),
        entry.username,
        "Anzeigename und Benutzername sind hier verschiedene Werte"
    );
    assert!(!entry.display_name.is_empty());
}

// ─── S-4 (D-62-03) ──────────────────────────────────────────────────────────

/// D-62-03: Ein reiner Verkaeufer hat kein Benutzerkonto — `username` ist leer,
/// der Anzeigename aber trotzdem gefuellt.
#[tokio::test]
async fn username_bei_reinem_verkaeufer_ist_leer() {
    let pool = setup_pool().await;
    let fixture = seed_three_kinds(pool.as_ref()).await;

    let entries = directory(&pool).await;
    let entry = entry_of(&entries, fixture.sales_only);

    assert_eq!(
        None, entry.username,
        "ein reiner Verkaeufer traegt kein Benutzernamen-Feld"
    );
    assert_eq!(
        Arc::<str>::from(LONELY_SALES_PERSON_NAME),
        entry.display_name,
        "der Verkaeufername traegt die Zeile"
    );
    assert!(!entry.display_name.is_empty());
}

// ─── S-5: Sortierung ────────────────────────────────────────────────────────

/// Die Sortierung gehoert in den Service (62-01 liefert unsortiert) und ist
/// case-insensitive: `Anton` steht vor `mia`, obwohl `A` < `m` in ASCII ohnehin
/// gilt — entscheidend ist `Zacharias` NACH `mia`, was eine naive
/// ASCII-Sortierung (`Z` = 0x5A < `m` = 0x6D) falsch machen wuerde.
#[tokio::test]
async fn liste_ist_case_insensitive_nach_display_name_sortiert() {
    let pool = setup_pool().await;

    let p_zora = Uuid::new_v4();
    let p_anton = Uuid::new_v4();
    let p_mia = Uuid::new_v4();
    let p_zacharias = Uuid::new_v4();
    for person in [p_zora, p_anton, p_mia, p_zacharias] {
        seed_person(pool.as_ref(), person).await;
    }

    // Einfuege-Reihenfolge bewusst unsortiert.
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "zora", p_zora).await;
    seed_user(pool.as_ref(), "Anton", p_anton).await;
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "mia", p_mia).await;
    seed_user(pool.as_ref(), "Zacharias", p_zacharias).await;

    let display_names: Vec<String> = directory(&pool)
        .await
        .iter()
        .map(|entry| entry.display_name.to_string())
        .collect();

    assert_eq!(
        vec![
            "Anton".to_string(),
            "mia".to_string(),
            "Zacharias".to_string(),
            "zora".to_string(),
        ],
        display_names,
        "case-insensitive nach Anzeigename sortiert"
    );
}

// ─── S-6 (D-62-06) ──────────────────────────────────────────────────────────

/// D-62-06: Die Rollen kommen als eine Bulk-Query und werden im Service
/// zugeordnet. Eine falsche Gruppierung (z.B. alle Rollen an alle) faellt hier
/// auf; der reine Verkaeufer hat keinen Benutzer und darum keine Rollen.
#[tokio::test]
async fn rollen_werden_dem_richtigen_benutzer_zugeordnet() {
    let pool = setup_pool().await;
    let fixture = seed_three_kinds(pool.as_ref()).await;

    seed_user_role(pool.as_ref(), LINKED_USER, "hr").await;
    seed_user_role(pool.as_ref(), LINKED_USER, "sales").await;
    seed_user_role(pool.as_ref(), LONELY_USER, "admin").await;

    let entries = directory(&pool).await;

    let mut linked_roles: Vec<String> = entry_of(&entries, fixture.linked)
        .roles
        .iter()
        .map(|role| role.to_string())
        .collect();
    linked_roles.sort();
    assert_eq!(
        vec!["hr".to_string(), "sales".to_string()],
        linked_roles,
        "{LINKED_USER} traegt genau seine eigenen Rollen"
    );

    let lonely_roles: Vec<String> = entry_of(&entries, fixture.user_only)
        .roles
        .iter()
        .map(|role| role.to_string())
        .collect();
    assert_eq!(
        vec!["admin".to_string()],
        lonely_roles,
        "{LONELY_USER} traegt genau seine eigene Rolle, nicht die der anderen"
    );

    assert!(
        entry_of(&entries, fixture.sales_only).roles.is_empty(),
        "ein reiner Verkaeufer hat kein Benutzerkonto und darum keine Rollen"
    );
}

// ─── S-7 (D-62-04) ──────────────────────────────────────────────────────────

/// D-62-04: „Geloescht" ist Abwesenheit, „inaktiv" ein Anzeige-Zustand. Der
/// inaktive Verkaeufer ist zugleich der sichtbare Anker, ohne den die
/// Abwesenheits-Behauptung auch gegen eine Immer-leer-Implementierung hielte.
#[tokio::test]
async fn soft_geloeschter_verkaeufer_erscheint_nicht_inaktiver_schon() {
    let pool = setup_pool().await;

    let p_deleted = Uuid::new_v4();
    let p_inactive = Uuid::new_v4();
    for person in [p_deleted, p_inactive] {
        seed_person(pool.as_ref(), person).await;
    }

    seed_sales_person(
        pool.as_ref(),
        Uuid::new_v4(),
        "Geloescht",
        p_deleted,
        "#FFF",
        true,
        false,
        Some(DELETED_AT),
    )
    .await;
    seed_sales_person(
        pool.as_ref(),
        Uuid::new_v4(),
        "Inaktiv",
        p_inactive,
        "#FFF",
        true,
        true,
        None,
    )
    .await;

    let entries = directory(&pool).await;

    assert_eq!(
        1,
        entries.len(),
        "nur der inaktive Verkaeufer bleibt uebrig: {entries:?}"
    );
    assert!(
        !entries.iter().any(|entry| entry.person_id == p_deleted),
        "ein soft-geloeschter Verkaeufer erscheint nicht im Verzeichnis"
    );

    let inactive = entry_of(&entries, p_inactive);
    assert_eq!(
        Some(true),
        inactive.inactive,
        "der inaktive Verkaeufer erscheint, mit gesetztem Inaktiv-Kennzeichen"
    );
    assert_eq!(PersonKind::SalesPersonOnly, inactive.kind);
}

// ─── S-8 (PADM-08, D-62-08, T-62-01 / T-62-05) ──────────────────────────────

/// PADM-08 / D-62-08: Ohne Admin-Privileg gibt es einen Fehler — keine leere
/// Liste und keine redigierten Felder. Eine leere Liste waere von einem leeren
/// Verzeichnis ununterscheidbar; der Aufrufer muss die Ablehnung sehen.
#[tokio::test]
async fn list_ohne_admin_privileg_ist_forbidden() {
    let pool = setup_pool().await;
    seed_three_kinds(pool.as_ref()).await;

    let service = build_service(pool.clone(), build_perm(false));
    let result = service.list(Authentication::Context(()), None).await;

    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "Nicht-Admin muss Forbidden erhalten, bekam {result:?}"
    );

    // Gegenprobe: dieselbe Fixture liefert als Admin sehr wohl Daten — die
    // Ablehnung oben ist also das Gate und nicht ein leeres Verzeichnis.
    assert_eq!(
        3,
        directory(&pool).await.len(),
        "als Admin liefert dieselbe Fixture drei Eintraege"
    );
}
