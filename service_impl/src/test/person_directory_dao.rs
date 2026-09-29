//! Phase 62 Plan 01 (PADM-01 / D-62-02 / D-62-04) — DB-Integrationstests fuer
//! den Verzeichnis-Lesepfad `PersonDirectoryDao`.
//!
//! Die Tests fahren gegen eine echte SQLite-in-memory-DB (`sqlx::migrate!`) und
//! gegen den echten `PersonDirectoryDaoImpl` — bewusst **ohne** DAO-Mock: der
//! gesamte Wert dieser Schicht liegt in der Join-Semantik des SQL, und ein
//! gemockter DAO wuerde genau die nicht ausfuehren (Begruendung wie im Kopf von
//! `person_candidates.rs`).
//!
//! Harness und Seed-Helfer sind dateilokale Kopien der Gegenstuecke aus
//! `person_candidates.rs` — die dortigen Helfer sind privat, und ihre
//! Sichtbarkeit zu aendern ist kein Recht dieses Plans.
//!
//! Abgedeckte Invarianten:
//!
//! - **V-1** (D-62-02): Alle drei Auspraegungen — verknuepfte Person, reines
//!   Benutzerkonto, reiner Verkaeufer — stehen in derselben Ergebnismenge
//!   (`verzeichnis_enthaelt_alle_drei_auspraegungen`).
//! - **V-2** (D-62-04): Eine soft-geloeschte Sales Person erscheint nicht
//!   (`soft_geloeschte_sales_person_erscheint_nicht`).
//! - **V-3** (D-62-04 / WR-02): Die soft-geloeschte Sales Person nimmt ihren
//!   Benutzer NICHT mit aus dem Verzeichnis — der Beweis, dass der
//!   Loesch-Filter in der `ON`- und nicht in der `WHERE`-Klausel steht
//!   (`soft_geloeschte_sales_person_laesst_benutzer_sichtbar`).
//! - **V-4** (D-62-04): Eine inaktive Sales Person erscheint sehr wohl, mit
//!   `inactive == Some(true)` (`inaktive_sales_person_erscheint`).
//! - **V-5** (Research Pitfall 5): Eine `person`-Zeile ohne jeden Traeger
//!   erscheint nicht (`person_ohne_traeger_erscheint_nicht`).
//! - **V-6** (Success Criterion 5): Die Rollen kommen als eine einzige
//!   Bulk-Query ueber alle Benutzer, nicht pro Zeile
//!   (`all_user_roles_liefert_alle_zuordnungen_in_einer_query`).
//!
//! V-2 und V-5 sind Abwesenheits-Behauptungen und darum bewusst gegen einen
//! zusaetzlich geseedeten, sichtbaren Anker gebaut: ohne ihn wuerden sie auch
//! gegen eine Implementierung halten, die schlicht immer leer liefert.

use std::sync::Arc;

use dao::person_directory::{PersonDirectoryDao, PersonDirectoryRow};
use dao_impl_sqlite::person_directory::PersonDirectoryDaoImpl;
use dao_impl_sqlite::{TransactionDaoImpl, TransactionImpl};
use uuid::Uuid;

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

/// Frische Transaktion fuer die DAO-Ebene (der DAO nimmt kein `Option`).
async fn tx_of(pool: &Arc<sqlx::SqlitePool>) -> TransactionImpl {
    use dao::TransactionDao;
    TransactionDaoImpl::new(pool.clone())
        .new_transaction()
        .await
        .expect("new transaction")
}

async fn directory(pool: &Arc<sqlx::SqlitePool>) -> Vec<PersonDirectoryRow> {
    PersonDirectoryDaoImpl::new(pool.clone())
        .list_directory_entries(tx_of(pool).await)
        .await
        .expect("list_directory_entries")
        .to_vec()
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

fn row_of(rows: &[PersonDirectoryRow], person_id: Uuid) -> &PersonDirectoryRow {
    rows.iter()
        .find(|row| row.person_id == person_id)
        .unwrap_or_else(|| panic!("Person {person_id} fehlt im Verzeichnis: {rows:?}"))
}

// ─── V-1 … V-5: list_directory_entries ──────────────────────────────────────

/// V-1 (D-62-02): Der Drei-Wege-Outer-Join ist die eine Lesequelle. Alle drei
/// Auspraegungen stehen nebeneinander — genau das loest die bisherigen Tabs ab.
#[tokio::test]
async fn verzeichnis_enthaelt_alle_drei_auspraegungen() {
    let pool = setup_pool().await;

    let p_linked = Uuid::new_v4();
    let p_user_only = Uuid::new_v4();
    let p_sales_only = Uuid::new_v4();
    for person in [p_linked, p_user_only, p_sales_only] {
        seed_person(pool.as_ref(), person).await;
    }

    let linked_sales_person = Uuid::new_v4();
    let lonely_sales_person = Uuid::new_v4();
    seed_live_sales_person(pool.as_ref(), linked_sales_person, "S-verknuepft", p_linked).await;
    seed_live_sales_person(
        pool.as_ref(),
        lonely_sales_person,
        "S-ohne-Login",
        p_sales_only,
    )
    .await;
    seed_user(pool.as_ref(), "u-verknuepft", p_linked).await;
    seed_user(pool.as_ref(), "u-ohne-sales-person", p_user_only).await;

    let rows = directory(&pool).await;

    assert_eq!(
        3,
        rows.len(),
        "drei Traeger-Konstellationen ergeben genau drei Zeilen: {rows:?}"
    );

    let linked = row_of(&rows, p_linked);
    assert_eq!(
        Some(linked_sales_person),
        linked.sales_person_id,
        "die verknuepfte Zeile traegt die Verkaeufer-Seite"
    );
    assert_eq!(
        Some(Arc::<str>::from("S-verknuepft")),
        linked.sales_person_name
    );
    assert_eq!(Some(Arc::<str>::from("#FFF")), linked.background_color);
    assert_eq!(Some(true), linked.is_paid);
    assert_eq!(Some(false), linked.inactive);
    assert_eq!(
        Some(Arc::<str>::from("u-verknuepft")),
        linked.username,
        "die verknuepfte Zeile traegt zusaetzlich die Benutzer-Seite"
    );

    let user_only = row_of(&rows, p_user_only);
    assert_eq!(
        None, user_only.sales_person_id,
        "ein reines Benutzerkonto hat keine Verkaeufer-Seite"
    );
    assert_eq!(None, user_only.sales_person_name);
    assert_eq!(None, user_only.background_color);
    assert_eq!(None, user_only.is_paid);
    assert_eq!(None, user_only.inactive);
    assert_eq!(
        Some(Arc::<str>::from("u-ohne-sales-person")),
        user_only.username
    );

    let sales_only = row_of(&rows, p_sales_only);
    assert_eq!(Some(lonely_sales_person), sales_only.sales_person_id);
    assert_eq!(
        Some(Arc::<str>::from("S-ohne-Login")),
        sales_only.sales_person_name
    );
    assert_eq!(
        None, sales_only.username,
        "ein reiner Verkaeufer hat keine Benutzer-Seite"
    );
}

/// V-2 (D-62-04): „Geloescht" ist Abwesenheit. Der lebende Anker beweist, dass
/// die Abfrage ueberhaupt liefert — sonst haelt der Test auch gegen eine
/// Implementierung, die immer leer zurueckgibt.
#[tokio::test]
async fn soft_geloeschte_sales_person_erscheint_nicht() {
    let pool = setup_pool().await;

    let p_deleted = Uuid::new_v4();
    let p_anchor = Uuid::new_v4();
    for person in [p_deleted, p_anchor] {
        seed_person(pool.as_ref(), person).await;
    }

    seed_sales_person(
        pool.as_ref(),
        Uuid::new_v4(),
        "S-geloescht",
        p_deleted,
        "#FFF",
        true,
        false,
        Some(DELETED_AT),
    )
    .await;
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "S-lebend", p_anchor).await;

    let rows = directory(&pool).await;

    assert_eq!(
        1,
        rows.len(),
        "nur der lebende Anker darf erscheinen: {rows:?}"
    );
    assert_eq!(
        p_anchor, rows[0].person_id,
        "die soft-geloeschte Sales Person traegt nichts mehr ins Verzeichnis"
    );
}

/// V-3 (D-62-04 / WR-02): der Kern-Nachweis dieser Schicht. Steht
/// `sp.deleted IS NULL` in der `WHERE`- statt in der `ON`-Klausel, degeneriert
/// der LEFT JOIN zum INNER JOIN und diese Zeile verschwindet mitsamt Benutzer.
#[tokio::test]
async fn soft_geloeschte_sales_person_laesst_benutzer_sichtbar() {
    let pool = setup_pool().await;

    let p5 = Uuid::new_v4();
    seed_person(pool.as_ref(), p5).await;
    seed_sales_person(
        pool.as_ref(),
        Uuid::new_v4(),
        "S5-geloescht-mit-Login",
        p5,
        "#FFF",
        true,
        false,
        Some(DELETED_AT),
    )
    .await;
    seed_user(pool.as_ref(), "u5-nur-geloeschte-sales-person", p5).await;

    let rows = directory(&pool).await;

    assert_eq!(
        1,
        rows.len(),
        "der Benutzer haelt die Zeile am Leben, auch wenn seine Sales Person \
         soft-geloescht ist: {rows:?}"
    );
    let row = row_of(&rows, p5);
    assert_eq!(
        None, row.sales_person_id,
        "die soft-geloeschte Verkaeufer-Seite bleibt leer"
    );
    assert_eq!(None, row.sales_person_name);
    assert_eq!(None, row.inactive);
    assert_eq!(
        Some(Arc::<str>::from("u5-nur-geloeschte-sales-person")),
        row.username,
        "die Benutzer-Seite bleibt sichtbar"
    );
}

/// V-4 (D-62-04): „Inaktiv" ist ein Anzeige-Zustand mit eigenem Badge, kein
/// Ausschlussgrund.
#[tokio::test]
async fn inaktive_sales_person_erscheint() {
    let pool = setup_pool().await;

    let p_inactive = Uuid::new_v4();
    seed_person(pool.as_ref(), p_inactive).await;
    let inactive_sales_person = Uuid::new_v4();
    seed_sales_person(
        pool.as_ref(),
        inactive_sales_person,
        "S-inaktiv",
        p_inactive,
        "#ABC",
        false,
        true,
        None,
    )
    .await;

    let rows = directory(&pool).await;

    assert_eq!(1, rows.len(), "die inaktive Sales Person erscheint: {rows:?}");
    let row = row_of(&rows, p_inactive);
    assert_eq!(Some(inactive_sales_person), row.sales_person_id);
    assert_eq!(
        Some(true),
        row.inactive,
        "der Inaktiv-Zustand wird durchgereicht, nicht weggefiltert"
    );
    assert_eq!(Some(false), row.is_paid);
    assert_eq!(Some(Arc::<str>::from("#ABC")), row.background_color);
}

/// V-5 (Research Pitfall 5): eine `person`-Zeile ohne jeden Traeger hat im
/// Verzeichnis nichts anzuzeigen. Der Anker haelt den Test nicht-vakuum.
#[tokio::test]
async fn person_ohne_traeger_erscheint_nicht() {
    use sqlx::Row;

    let pool = setup_pool().await;

    let p_orphan = Uuid::new_v4();
    let p_anchor = Uuid::new_v4();
    for person in [p_orphan, p_anchor] {
        seed_person(pool.as_ref(), person).await;
    }
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "S-lebend", p_anchor).await;

    let person_count: i64 = sqlx::query("SELECT COUNT(*) AS cnt FROM person")
        .fetch_one(pool.as_ref())
        .await
        .expect("count person")
        .get("cnt");
    assert_eq!(
        2, person_count,
        "Fixture-Vorbedingung: zwei person-Zeilen, eine davon traegerlos"
    );

    let rows = directory(&pool).await;

    assert_eq!(
        1,
        rows.len(),
        "nur die Person mit Traeger erscheint: {rows:?}"
    );
    assert_eq!(p_anchor, rows[0].person_id);
    assert!(
        !rows.iter().any(|row| row.person_id == p_orphan),
        "die traegerlose Person darf keine Zeile erzeugen"
    );
}

// ─── V-6: all_user_roles ────────────────────────────────────────────────────

/// V-6 (Success Criterion 5): eine Bulk-Query statt einer Abfrage pro Zeile.
#[tokio::test]
async fn all_user_roles_liefert_alle_zuordnungen_in_einer_query() {
    let pool = setup_pool().await;

    let p1 = Uuid::new_v4();
    let p2 = Uuid::new_v4();
    for person in [p1, p2] {
        seed_person(pool.as_ref(), person).await;
    }
    seed_user(pool.as_ref(), "u1", p1).await;
    seed_user(pool.as_ref(), "u2", p2).await;

    seed_user_role(pool.as_ref(), "u1", "admin").await;
    seed_user_role(pool.as_ref(), "u1", "hr").await;
    seed_user_role(pool.as_ref(), "u2", "sales").await;
    seed_user_role(pool.as_ref(), "u2", "hr").await;

    let mut pairs: Vec<(String, String)> = PersonDirectoryDaoImpl::new(pool.clone())
        .all_user_roles(tx_of(&pool).await)
        .await
        .expect("all_user_roles")
        .iter()
        .map(|row| (row.user_name.to_string(), row.role_name.to_string()))
        .collect();
    pairs.sort();

    assert_eq!(
        vec![
            ("u1".to_string(), "admin".to_string()),
            ("u1".to_string(), "hr".to_string()),
            ("u2".to_string(), "hr".to_string()),
            ("u2".to_string(), "sales".to_string()),
        ],
        pairs,
        "ein einziger Aufruf liefert alle vier Zuordnungen beider Benutzer"
    );
}
