//! Phase 63 Plan 02 (PADM-05 / PADM-07) — DB-Integrationstests fuer die fuenf
//! DAO-Pfade, auf denen die Personen-Detailansicht und der
//! ausprägungs-spezifische Loeschpfad aufsetzen:
//! `PersonDirectoryDao::find_directory_entry` sowie `PersonDao::delete`,
//! `::has_live_facets`, `::find_by_sales_person` und `::find_by_user`.
//!
//! Die Tests fahren gegen eine echte SQLite-in-memory-DB (`sqlx::migrate!`) und
//! gegen die echten DAO-Implementierungen — bewusst **ohne** DAO-Mock: der
//! gesamte Wert dieser Schicht liegt in der Join- und Filter-Semantik des SQL,
//! und ein gemockter DAO wuerde genau die nicht ausfuehren (Begruendung wie im
//! Kopf von `person_candidates.rs` und `person_directory_dao.rs`).
//!
//! **Die D-63-01-Lesart, die dieser Plan entschieden hat** (RESEARCH.md liess
//! sie offen, Pitfall 1 / Assumption A1): Es gilt Lesart **(b)** — eine Person
//! ist traegerlos, wenn keine `user`-Zeile und keine **lebende**
//! (`deleted IS NULL`) `sales_person`-Zeile mehr auf sie zeigt; eine
//! soft-geloeschte Sales Person zaehlt als weg, weil sie durch den
//! Verzeichnis-Join ohnehin unsichtbar ist (D-62-04). Der akzeptierte Preis:
//! diese soft-geloeschte Zeile behaelt einen `person_id`, der nach dem
//! `DELETE FROM person` auf nichts mehr aufloest. `D-4` unten ist der Beleg,
//! dass dieser Zustand konstruierbar und gewollt ist — kein Bug zum
//! „Reparieren".
//!
//! Abgedeckte Invarianten:
//!
//! - **E-1** (PADM-05): Eine verknuepfte Person liefert eine Zeile mit beiden
//!   Seiten (`find_directory_entry_liefert_verknuepfte_zeile`).
//! - **E-2**: Reines Benutzerkonto — Verkaeufer-Seite leer
//!   (`find_directory_entry_nur_benutzer`).
//! - **E-3**: Reiner Verkaeufer — Benutzer-Seite leer
//!   (`find_directory_entry_nur_verkaeufer`).
//! - **E-4** (D-62-04 / WR-02): Nur soft-geloeschte Sales Person, kein
//!   Benutzer → `None`, dieselbe Unsichtbarkeit wie in der Liste
//!   (`find_directory_entry_ohne_lebenden_traeger_ist_none`).
//! - **E-5**: Unbekannte `person_id` → `None`
//!   (`find_directory_entry_unbekannte_person_ist_none`).
//! - **E-6** (D-62-04 / WR-02): Soft-geloeschte Sales Person **mit** Benutzer →
//!   Zeile bleibt, nur die Verkaeufer-Seite ist leer
//!   (`find_directory_entry_soft_geloeschte_sales_person_laesst_benutzer_sichtbar`).
//!   Der Beweis, dass der Loesch-Filter in der `ON`- und nicht in der
//!   `WHERE`-Klausel steht — E-1 bis E-5 halten auch gegen die verschobene
//!   Klausel, dieser Test nicht.
//! - **L-1 … L-4** (D-63-01): die vier Traeger-Faelle von `has_live_facets` —
//!   nur Benutzer, nur lebende Sales Person, soft-geloeschte Sales Person ohne
//!   Benutzer, soft-geloeschte Sales Person mit Benutzer.
//! - **D-1** (PADM-07): `delete` entfernt die Zeile, ein zweiter Aufruf ist ein
//!   No-Op ohne Fehler (`delete_entfernt_person_zeile`).
//! - **D-4** (D-63-01): `delete` gelingt auch, wenn eine soft-geloeschte Sales
//!   Person noch auf die Person zeigt; die Sales-Person-Zeile bleibt mit ihrem
//!   nun haengenden `person_id` unveraendert bestehen
//!   (`delete_funktioniert_bei_haengendem_person_id`).
//! - **R-1** (D-61-03): Beide Rueckwaerts-Lookups liefern dieselbe
//!   `PersonEntity` (`find_by_sales_person_und_find_by_user_loesen_auf`).
//!
//! Harness und Seed-Helfer sind dateilokale Kopien der Gegenstuecke aus
//! `person_directory_dao.rs` — die dortigen Helfer sind privat, und ihre
//! Sichtbarkeit zu aendern ist kein Recht dieses Plans.

use std::sync::Arc;

use dao::person::{PersonDao, PersonEntity};
use dao::person_directory::{PersonDirectoryDao, PersonDirectoryRow};
use dao::TransactionDao;
use dao_impl_sqlite::person::PersonDaoImpl;
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
    TransactionDaoImpl::new(pool.clone())
        .new_transaction()
        .await
        .expect("new transaction")
}

/// Schreibende Tests muessen committen: ohne Commit rollt die Transaktion beim
/// Drop zurueck und der nachfolgende Lesepfad saehe den alten Zustand.
async fn commit(pool: &Arc<sqlx::SqlitePool>, tx: TransactionImpl) {
    TransactionDaoImpl::new(pool.clone())
        .commit(tx)
        .await
        .expect("commit");
}

async fn entry(pool: &Arc<sqlx::SqlitePool>, person_id: Uuid) -> Option<PersonDirectoryRow> {
    PersonDirectoryDaoImpl::new(pool.clone())
        .find_directory_entry(person_id, tx_of(pool).await)
        .await
        .expect("find_directory_entry")
}

async fn has_live_facets(pool: &Arc<sqlx::SqlitePool>, person_id: Uuid) -> bool {
    PersonDaoImpl::new(pool.clone())
        .has_live_facets(person_id, tx_of(pool).await)
        .await
        .expect("has_live_facets")
}

async fn find_person(pool: &Arc<sqlx::SqlitePool>, person_id: Uuid) -> Option<PersonEntity> {
    PersonDaoImpl::new(pool.clone())
        .find_by_id(person_id, tx_of(pool).await)
        .await
        .expect("find_by_id")
}

async fn delete_person(pool: &Arc<sqlx::SqlitePool>, person_id: Uuid) {
    let tx = tx_of(pool).await;
    PersonDaoImpl::new(pool.clone())
        .delete(person_id, tx.clone())
        .await
        .expect("delete");
    commit(pool, tx).await;
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

/// Soft-geloeschte Sales Person — der D-63-01-relevante Fall.
async fn seed_deleted_sales_person(pool: &sqlx::SqlitePool, id: Uuid, name: &str, person_id: Uuid) {
    seed_sales_person(
        pool,
        id,
        name,
        person_id,
        "#FFF",
        true,
        false,
        Some(DELETED_AT),
    )
    .await;
}

async fn seed_user(pool: &sqlx::SqlitePool, name: &str, person_id: Uuid) {
    sqlx::query("INSERT INTO user (name, person_id, update_process) VALUES (?1, ?2, 'seed')")
        .bind(name)
        .bind(person_id.as_bytes().to_vec())
        .execute(pool)
        .await
        .expect("seed user");
}

// ─── E-1 … E-5: find_directory_entry ────────────────────────────────────────

/// E-1 (PADM-05): Die Detailansicht liest dieselbe Rohzeile wie die Liste —
/// beide Traeger-Seiten gefuellt.
#[tokio::test]
async fn find_directory_entry_liefert_verknuepfte_zeile() {
    let pool = setup_pool().await;

    let person_id = Uuid::new_v4();
    let sales_person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_live_sales_person(pool.as_ref(), sales_person_id, "S-verknuepft", person_id).await;
    seed_user(pool.as_ref(), "u-verknuepft", person_id).await;

    let row = entry(&pool, person_id)
        .await
        .expect("die verknuepfte Person hat eine Verzeichniszeile");

    assert_eq!(person_id, row.person_id);
    assert_eq!(Some(sales_person_id), row.sales_person_id);
    assert_eq!(
        Some(Arc::<str>::from("S-verknuepft")),
        row.sales_person_name
    );
    assert_eq!(Some(Arc::<str>::from("#FFF")), row.background_color);
    assert_eq!(Some(true), row.is_paid);
    assert_eq!(Some(false), row.inactive);
    assert_eq!(Some(Arc::<str>::from("u-verknuepft")), row.username);
}

/// E-2: Reines Benutzerkonto — alle fuenf Verkaeufer-Felder gemeinsam leer.
#[tokio::test]
async fn find_directory_entry_nur_benutzer() {
    let pool = setup_pool().await;

    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_user(pool.as_ref(), "u-ohne-sales-person", person_id).await;

    let row = entry(&pool, person_id)
        .await
        .expect("ein reines Benutzerkonto hat eine Verzeichniszeile");

    assert_eq!(None, row.sales_person_id);
    assert_eq!(None, row.sales_person_name);
    assert_eq!(None, row.background_color);
    assert_eq!(None, row.is_paid);
    assert_eq!(None, row.inactive);
    assert_eq!(
        Some(Arc::<str>::from("u-ohne-sales-person")),
        row.username,
        "die Benutzer-Seite traegt die Zeile allein"
    );
}

/// E-3: Reiner Verkaeufer — Benutzer-Seite leer, Verkaeufer-Seite vollstaendig.
#[tokio::test]
async fn find_directory_entry_nur_verkaeufer() {
    let pool = setup_pool().await;

    let person_id = Uuid::new_v4();
    let sales_person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_sales_person(
        pool.as_ref(),
        sales_person_id,
        "S-ohne-Login",
        person_id,
        "#ABC",
        false,
        true,
        None,
    )
    .await;

    let row = entry(&pool, person_id)
        .await
        .expect("ein reiner Verkaeufer hat eine Verzeichniszeile");

    assert_eq!(Some(sales_person_id), row.sales_person_id);
    assert_eq!(Some(Arc::<str>::from("S-ohne-Login")), row.sales_person_name);
    assert_eq!(Some(Arc::<str>::from("#ABC")), row.background_color);
    assert_eq!(Some(false), row.is_paid);
    assert_eq!(
        Some(true),
        row.inactive,
        "inaktiv ist ein Anzeige-Zustand, kein Ausschlussgrund (D-62-04)"
    );
    assert_eq!(None, row.username);
}

/// E-4 (D-62-04 / WR-02): Eine Person ohne lebenden Traeger ist hier so
/// unsichtbar wie in der Liste. Der Anker haelt den Test nicht-vakuum: die
/// Abfrage liefert sehr wohl, nur eben nicht fuer diese Person.
#[tokio::test]
async fn find_directory_entry_ohne_lebenden_traeger_ist_none() {
    let pool = setup_pool().await;

    let p_orphan = Uuid::new_v4();
    let p_anchor = Uuid::new_v4();
    for person in [p_orphan, p_anchor] {
        seed_person(pool.as_ref(), person).await;
    }
    seed_deleted_sales_person(pool.as_ref(), Uuid::new_v4(), "S-geloescht", p_orphan).await;
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "S-lebend", p_anchor).await;

    assert!(
        entry(&pool, p_anchor).await.is_some(),
        "Vorbedingung: der lebende Anker ist sichtbar"
    );
    assert_eq!(
        None,
        entry(&pool, p_orphan).await,
        "eine nur soft-geloeschte Sales Person traegt keine Detailansicht mehr"
    );
}

/// E-5: Eine unbekannte `person_id` liefert `None` statt eines Fehlers — der
/// aufrufende Service uebersetzt das spaeter in ein 404.
#[tokio::test]
async fn find_directory_entry_unbekannte_person_ist_none() {
    let pool = setup_pool().await;

    let p_anchor = Uuid::new_v4();
    seed_person(pool.as_ref(), p_anchor).await;
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "S-lebend", p_anchor).await;

    assert_eq!(None, entry(&pool, Uuid::new_v4()).await);
}

/// E-6 (D-62-04 / WR-02) — **der Kern-Nachweis dieses Lesepfads**: Eine
/// soft-geloeschte Sales Person nimmt ihren Benutzer NICHT mit aus der
/// Detailansicht. Steht `sp.deleted IS NULL` in der `WHERE`- statt in der
/// `ON`-Klausel, matcht die geloeschte Zeile und faellt dann samt Person aus
/// dem Ergebnis — die Detailansicht eines Benutzers mit Verkaeufer-Historie
/// waere ploetzlich ein 404. Das Gegenstueck zu V-3 in
/// `person_directory_dao.rs`; ohne diesen Test haelt das Testmodul auch gegen
/// die verschobene Klausel.
#[tokio::test]
async fn find_directory_entry_soft_geloeschte_sales_person_laesst_benutzer_sichtbar() {
    let pool = setup_pool().await;

    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_deleted_sales_person(pool.as_ref(), Uuid::new_v4(), "S-geloescht", person_id).await;
    seed_user(pool.as_ref(), "u-nur-geloeschte-sales-person", person_id).await;

    let row = entry(&pool, person_id)
        .await
        .expect("der Benutzer haelt die Zeile am Leben, auch ohne lebende Sales Person");

    assert_eq!(person_id, row.person_id);
    assert_eq!(
        None, row.sales_person_id,
        "die soft-geloeschte Verkaeufer-Seite bleibt leer"
    );
    assert_eq!(None, row.sales_person_name);
    assert_eq!(None, row.inactive);
    assert_eq!(
        Some(Arc::<str>::from("u-nur-geloeschte-sales-person")),
        row.username,
        "die Benutzer-Seite bleibt sichtbar"
    );
}

// ─── L-1 … L-4: has_live_facets (D-63-01-Lesart (b)) ────────────────────────

/// L-1: Ein Benutzerkonto haelt die Person am Leben — `user` kennt keinen
/// Loesch-Zustand.
#[tokio::test]
async fn has_live_facets_nur_benutzer_ist_true() {
    let pool = setup_pool().await;

    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_user(pool.as_ref(), "u-allein", person_id).await;

    assert!(has_live_facets(&pool, person_id).await);
}

/// L-2: Eine lebende Sales Person haelt die Person am Leben.
#[tokio::test]
async fn has_live_facets_nur_lebende_sales_person_ist_true() {
    let pool = setup_pool().await;

    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_live_sales_person(pool.as_ref(), Uuid::new_v4(), "S-allein", person_id).await;

    assert!(has_live_facets(&pool, person_id).await);
}

/// L-3 — **der Kern der D-63-01-Lesart (b)**: Eine soft-geloeschte Sales Person
/// ohne Benutzerkonto zaehlt als weg. Genau dieses `false` loest den
/// Orphan-Cleanup aus. Unter Lesart (a) waere hier `true` und die `person`-Zeile
/// bliebe ewig liegen.
#[tokio::test]
async fn has_live_facets_soft_geloeschte_sales_person_ohne_benutzer_ist_false() {
    let pool = setup_pool().await;

    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_deleted_sales_person(pool.as_ref(), Uuid::new_v4(), "S-geloescht", person_id).await;

    assert!(
        !has_live_facets(&pool, person_id).await,
        "soft-geloeschte Sales Person ohne Benutzer: die Person ist traegerlos"
    );
}

/// L-4: Dieselbe soft-geloeschte Sales Person, aber mit Benutzerkonto — die
/// Person bleibt. Die Gegenprobe zu L-3: ohne sie wuerde auch eine
/// Implementierung halten, die die Benutzer-Seite gar nicht prueft.
#[tokio::test]
async fn has_live_facets_soft_geloeschte_sales_person_mit_benutzer_ist_true() {
    let pool = setup_pool().await;

    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_deleted_sales_person(pool.as_ref(), Uuid::new_v4(), "S-geloescht", person_id).await;
    seed_user(pool.as_ref(), "u-haelt-die-person", person_id).await;

    assert!(has_live_facets(&pool, person_id).await);
}

// ─── D-1, D-4: delete ───────────────────────────────────────────────────────

/// D-1 (PADM-07): Harter `DELETE`, und ein zweiter Aufruf ist ein No-Op ohne
/// Fehler — der Cleanup-Pfad darf bei Wiederholung nicht scheitern.
#[tokio::test]
async fn delete_entfernt_person_zeile() {
    let pool = setup_pool().await;

    let person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    assert!(
        find_person(&pool, person_id).await.is_some(),
        "Vorbedingung: die Person existiert"
    );

    delete_person(&pool, person_id).await;
    assert_eq!(
        None,
        find_person(&pool, person_id).await,
        "nach dem DELETE ist die Zeile weg — kein Soft-Delete-Stempel (D-60-04)"
    );

    let tx = tx_of(&pool).await;
    let second = PersonDaoImpl::new(pool.clone())
        .delete(person_id, tx.clone())
        .await;
    commit(&pool, tx).await;
    assert!(
        second.is_ok(),
        "ein zweiter delete-Aufruf ist ein No-Op ohne Fehler: {second:?}"
    );
}

/// D-4 — **der Beleg fuer die gewaehlte D-63-01-Lesart (b)**: Die Person wird
/// geloescht, obwohl eine soft-geloeschte Sales Person noch ihren `person_id`
/// traegt. Das gelingt, weil `sales_person.person_id` KEINEN Foreign Key auf
/// `person(id)` hat (Phase-60-Migration: ADD COLUMN plus UNIQUE-Index).
/// Der danach haengende Zeiger ist die dokumentierte, bewusst akzeptierte
/// Konsequenz — kein Defekt, und nichts, was ein spaeterer Aufraeum-Pass
/// „reparieren" soll.
#[tokio::test]
async fn delete_funktioniert_bei_haengendem_person_id() {
    use sqlx::Row;

    let pool = setup_pool().await;

    let person_id = Uuid::new_v4();
    let sales_person_id = Uuid::new_v4();
    seed_person(pool.as_ref(), person_id).await;
    seed_deleted_sales_person(pool.as_ref(), sales_person_id, "S-geloescht", person_id).await;

    assert!(
        !has_live_facets(&pool, person_id).await,
        "Vorbedingung: die Person gilt als traegerlos"
    );

    delete_person(&pool, person_id).await;

    assert_eq!(
        None,
        find_person(&pool, person_id).await,
        "die person-Zeile ist weg"
    );

    let row = sqlx::query("SELECT person_id, deleted FROM sales_person WHERE id = ?1")
        .bind(sales_person_id.as_bytes().to_vec())
        .fetch_one(pool.as_ref())
        .await
        .expect("die soft-geloeschte sales_person-Zeile existiert weiter");
    let dangling: Vec<u8> = row.get("person_id");
    let deleted: Option<String> = row.get("deleted");
    assert_eq!(
        person_id.as_bytes().to_vec(),
        dangling,
        "die sales_person-Zeile behaelt ihren nun haengenden person_id — \
         der akzeptierte Preis der D-63-01-Lesart (b)"
    );
    assert_eq!(
        Some(DELETED_AT.to_string()),
        deleted,
        "der Soft-Delete-Stempel der Sales Person bleibt unveraendert"
    );
}

// ─── R-1: Rueckwaerts-Lookups ───────────────────────────────────────────────

/// R-1 (D-61-03): Beide Traeger-Seiten loesen auf dieselbe Identitaet auf —
/// genau das braucht der kombinierte Anlege-Pfad, um nach dem Merge die
/// gewinnende Person zurueckgeben zu koennen.
#[tokio::test]
async fn find_by_sales_person_und_find_by_user_loesen_auf() {
    let pool = setup_pool().await;

    let person_id = Uuid::new_v4();
    let sales_person_id = Uuid::new_v4();
    let deleted_sales_person_id = Uuid::new_v4();
    let other_person_id = Uuid::new_v4();
    for person in [person_id, other_person_id] {
        seed_person(pool.as_ref(), person).await;
    }
    seed_live_sales_person(pool.as_ref(), sales_person_id, "S-verknuepft", person_id).await;
    seed_user(pool.as_ref(), "u-verknuepft", person_id).await;
    seed_deleted_sales_person(
        pool.as_ref(),
        deleted_sales_person_id,
        "S-geloescht",
        other_person_id,
    )
    .await;

    let dao = PersonDaoImpl::new(pool.clone());

    let via_sales_person = dao
        .find_by_sales_person(sales_person_id, tx_of(&pool).await)
        .await
        .expect("find_by_sales_person")
        .expect("die Sales Person loest auf ihre Person auf");
    let via_user = dao
        .find_by_user("u-verknuepft", tx_of(&pool).await)
        .await
        .expect("find_by_user")
        .expect("der Benutzer loest auf seine Person auf");

    assert_eq!(person_id, via_sales_person.id);
    assert_eq!(
        via_sales_person, via_user,
        "beide Seiten liefern dieselbe Identitaet, Feld fuer Feld"
    );
    assert_eq!(
        Some(person_id),
        find_person(&pool, person_id).await.map(|person| person.id),
        "und es ist dieselbe Zeile, die find_by_id liefert"
    );

    let deleted_resolves = dao
        .find_by_sales_person(deleted_sales_person_id, tx_of(&pool).await)
        .await
        .expect("find_by_sales_person fuer eine soft-geloeschte Sales Person");
    assert_eq!(
        Some(other_person_id),
        deleted_resolves.map(|person| person.id),
        "der Lookup bewertet nicht, er loest auf: auch eine soft-geloeschte \
         Sales Person liefert ihre Person"
    );

    assert_eq!(
        None,
        dao.find_by_user("gibt-es-nicht", tx_of(&pool).await)
            .await
            .expect("find_by_user fuer unbekannten Benutzer"),
        "ein unbekannter Benutzername liefert None statt eines Fehlers"
    );
}
