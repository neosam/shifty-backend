//! Phase 60 Plan 01 (PERS-01 / PERS-02 / PERS-03) — Integrationstests fuer die
//! Migration `20260916000000_create-person-backfill.sql`.
//!
//! Die Tests fahren gegen eine echte SQLite-in-memory-DB (`sqlx::migrate!`) und
//! gegen rohes SQL — bewusst ohne Service/DAO, weil Phase 60 laut D-60-10 keinen
//! Applikationscode anfasst (ein `PersonDao` entsteht erst in Phase 61).
//!
//! Abgedeckte Invarianten:
//!
//! - **I-1** (PERS-01/SC1): Nach dem Backfill traegt jede `sales_person`-Zeile —
//!   auch die soft-geloeschte — und jede `user`-Zeile ein nicht-leeres
//!   `person_id` (`full_identity_coverage`).
//! - **I-2** (PERS-01/SC1): Beide Seiten eines `sales_person_user`-Paars zeigen
//!   auf dieselbe `person.id` (`full_identity_coverage`).
//! - **I-3** (PERS-01/SC1): Die Zaehlprobe
//!   `COUNT(person) == COUNT(sales_person) + COUNT(user) - COUNT(sales_person_user)`
//!   haelt (`count_identity_holds`).
//! - **I-3-negativ** (D-60-07): Eine verletzte Zaehlprobe bricht den Lauf mit
//!   einem Datenbankfehler ab (`migration_aborts_on_count_mismatch`).
//! - **I-4** (PERS-02/SC2): Ein zweiter Lauf des Backfill-Abschnitts ist ein
//!   beweisbarer No-Op — Zeilenzahl UND jede Zuordnung identisch
//!   (`apply_twice_is_noop`).
//! - **I-5** (PERS-03/SC3): Eine soft-geloeschte Sales Person mit
//!   `billing_period_sales_person`-Zeile bekommt ein `person_id`
//!   (`soft_deleted_sales_person_gets_person_id`).
//! - **I-8** (D-60-03): HR-/Admin-`user` ohne Sales Person und freiwillige
//!   `sales_person` ohne `user` bekommen je eine eigene Person; die Gegenseite
//!   wird NICHT automatisch mit angelegt (`hr_user_without_sales_person`,
//!   `volunteer_without_user`).
//! - **RESEARCH Frage 4:** Jeder erzeugte `person.id`-Wert ist mit
//!   `Uuid::from_slice()` parsebar — Vorab-Absicherung fuer den `PersonDao` in
//!   Phase 61 (`person_id_parses_as_uuid`).

use std::sync::Arc;

use sqlx::Row;
use uuid::Uuid;

// ─── Harness ────────────────────────────────────────────────────────────────

/// Die Migrationsdatei im Klartext. Der Apply-twice-Nachweis (D-60-08) schneidet
/// aus ihr den markierten Backfill-Abschnitt heraus und fuehrt genau diesen ein
/// zweites Mal aus.
const MIGRATION_SQL: &str =
    include_str!("../../../migrations/sqlite/20260916000000_create-person-backfill.sql");

const BACKFILL_START: &str = "-- BACKFILL START";
const BACKFILL_END: &str = "-- BACKFILL END";

const HR_USER: &str = "hr-admin-ohne-sales-person";
const LINKED_USER: &str = "verknuepfter-benutzer";

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

/// Schneidet die Region zwischen `-- BACKFILL START` und `-- BACKFILL END` aus
/// der Migrationsdatei heraus.
///
/// Der volle Datei-Replay scheidet aus: SQLite kennt fuer
/// `ALTER TABLE ... ADD COLUMN` keine Existenz-Guard-Klausel, ein zweiter Lauf
/// der kompletten Datei wuerde also strukturell scheitern statt die Idempotenz
/// des Backfills zu pruefen.
fn backfill_section() -> String {
    let (_, rest) = MIGRATION_SQL.split_once(BACKFILL_START).unwrap_or_else(|| {
        panic!("Marker '{BACKFILL_START}' fehlt in der Migrationsdatei — der Apply-twice-Nachweis (D-60-08) waere damit stumm abgeschaltet")
    });
    let (section, _) = rest.split_once(BACKFILL_END).unwrap_or_else(|| {
        panic!("Marker '{BACKFILL_END}' fehlt in der Migrationsdatei — der Apply-twice-Nachweis (D-60-08) waere damit stumm abgeschaltet")
    });
    section.to_string()
}

/// Fuehrt den Backfill-Abschnitt aus und reicht das `Result` unveraendert
/// durch — der Negativtest zu D-60-07 braucht den `Err`-Fall.
///
/// `sqlx::migrate!().run()` ein zweites Mal aufzurufen funktioniert nicht: der
/// Ledger `_sqlx_migrations` ueberspringt die bereits angewandte Datei.
async fn run_backfill_section(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
    let section = backfill_section();
    if section.trim().is_empty() {
        // Ein leerer Markerbereich ist definitionsgemaess ein No-Op;
        // `sqlx::raw_sql` wuerde einen leeren Skript-String ablehnen.
        return Ok(());
    }
    sqlx::raw_sql(section.as_str()).execute(pool).await?;
    Ok(())
}

// ─── Fixtures ───────────────────────────────────────────────────────────────

struct Fixtures {
    soft_deleted_sales_person: Uuid,
    volunteer_sales_person: Uuid,
    linked_sales_person: Uuid,
}

async fn seed_sales_person(
    pool: &sqlx::SqlitePool,
    id: Uuid,
    name: &str,
    deleted: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO sales_person (id, name, inactive, deleted, update_process, update_version) \
         VALUES (?1, ?2, 0, ?3, 'seed', ?4)",
    )
    .bind(id.as_bytes().to_vec())
    .bind(name)
    .bind(deleted)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(pool)
    .await
    .expect("seed sales_person");
}

async fn seed_user(pool: &sqlx::SqlitePool, name: &str) {
    sqlx::query("INSERT INTO user (name, update_process) VALUES (?1, 'seed')")
        .bind(name)
        .execute(pool)
        .await
        .expect("seed user");
}

async fn seed_link(pool: &sqlx::SqlitePool, sales_person_id: Uuid, user_name: &str) {
    sqlx::query(
        "INSERT INTO sales_person_user (sales_person_id, user_id, update_process) \
         VALUES (?1, ?2, 'seed')",
    )
    .bind(sales_person_id.as_bytes().to_vec())
    .bind(user_name)
    .execute(pool)
    .await
    .expect("seed sales_person_user");
}

/// Legt eine `billing_period`- plus `billing_period_sales_person`-Zeile fuer die
/// PERS-03-Fixture an: der write-once-Snapshot ist genau der Grund, warum eine
/// soft-geloeschte Sales Person ihre Identitaet braucht.
async fn seed_billing_period_row(pool: &sqlx::SqlitePool, sales_person_id: Uuid) {
    let billing_period_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO billing_period \
         (id, from_date_time, to_date_time, created, created_by, deleted, deleted_by, update_version, update_process) \
         VALUES (?1, '2025-01-01T00:00:00', '2025-01-31T23:59:59', '2025-02-01T00:00:00', 'seed', NULL, NULL, ?2, 'seed')",
    )
    .bind(billing_period_id.as_bytes().to_vec())
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(pool)
    .await
    .expect("seed billing_period");

    sqlx::query(
        "INSERT INTO billing_period_sales_person \
         (id, billing_period_id, sales_person_id, value_type, value_delta, value_ytd_from, value_ytd_to, value_full_year, created_at, created_by, deleted_at, deleted_by, update_version, update_process) \
         VALUES (?1, ?2, ?3, 'BalanceHours', 0.0, 0.0, 0.0, 0.0, '2025-02-01T00:00:00', 'seed', NULL, NULL, ?4, 'seed')",
    )
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .bind(billing_period_id.as_bytes().to_vec())
    .bind(sales_person_id.as_bytes().to_vec())
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(pool)
    .await
    .expect("seed billing_period_sales_person");
}

/// Seedet alle vier Auspraegungen aus 60-CONTEXT.md — es gibt keine
/// Mengenannahme ueber die Produktionsdaten, jede Auspraegung braucht eine
/// eigene Fixture.
async fn seed_fixtures(pool: &sqlx::SqlitePool) -> Fixtures {
    let soft_deleted_sales_person = Uuid::new_v4();
    let volunteer_sales_person = Uuid::new_v4();
    let linked_sales_person = Uuid::new_v4();

    // (a) soft-geloeschte Sales Person mit Abrechnungs-Snapshot-Zeile
    seed_sales_person(
        pool,
        soft_deleted_sales_person,
        "Soft-geloeschte Verkaeuferin",
        Some("2025-01-01T00:00:00"),
    )
    .await;
    seed_billing_period_row(pool, soft_deleted_sales_person).await;

    // (b) HR-/Admin-Benutzer ohne Sales Person
    seed_user(pool, HR_USER).await;

    // (c) Freiwillige ohne Login
    seed_sales_person(pool, volunteer_sales_person, "Freiwillige ohne Login", None).await;

    // (d) ueber sales_person_user verknuepftes Paar
    seed_sales_person(pool, linked_sales_person, "Verknuepfte Verkaeuferin", None).await;
    seed_user(pool, LINKED_USER).await;
    seed_link(pool, linked_sales_person, LINKED_USER).await;

    Fixtures {
        soft_deleted_sales_person,
        volunteer_sales_person,
        linked_sales_person,
    }
}

// ─── Leser ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    person_count: i64,
    sales_person_rows: Vec<(Vec<u8>, Option<Vec<u8>>)>,
    user_rows: Vec<(String, Option<Vec<u8>>)>,
}

async fn count(pool: &sqlx::SqlitePool, sql: &str) -> i64 {
    sqlx::query(sql)
        .fetch_one(pool)
        .await
        .expect("count query")
        .get::<i64, _>(0)
}

async fn snapshot(pool: &sqlx::SqlitePool) -> Snapshot {
    let person_count = count(pool, "SELECT COUNT(*) FROM person").await;

    let sales_person_rows = sqlx::query("SELECT id, person_id FROM sales_person ORDER BY id")
        .fetch_all(pool)
        .await
        .expect("read sales_person")
        .into_iter()
        .map(|row| {
            (
                row.get::<Vec<u8>, _>("id"),
                row.get::<Option<Vec<u8>>, _>("person_id"),
            )
        })
        .collect();

    let user_rows = sqlx::query("SELECT name, person_id FROM user ORDER BY name")
        .fetch_all(pool)
        .await
        .expect("read user")
        .into_iter()
        .map(|row| {
            (
                row.get::<String, _>("name"),
                row.get::<Option<Vec<u8>>, _>("person_id"),
            )
        })
        .collect();

    Snapshot {
        person_count,
        sales_person_rows,
        user_rows,
    }
}

async fn person_id_of_sales_person(pool: &sqlx::SqlitePool, id: Uuid) -> Option<Vec<u8>> {
    sqlx::query("SELECT person_id FROM sales_person WHERE id = ?1")
        .bind(id.as_bytes().to_vec())
        .fetch_one(pool)
        .await
        .expect("read sales_person.person_id")
        .get::<Option<Vec<u8>>, _>("person_id")
}

async fn person_id_of_user(pool: &sqlx::SqlitePool, name: &str) -> Option<Vec<u8>> {
    sqlx::query("SELECT person_id FROM user WHERE name = ?1")
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("read user.person_id")
        .get::<Option<Vec<u8>>, _>("person_id")
}

// ─── Tests ──────────────────────────────────────────────────────────────────

/// I-1 + I-2 (PERS-01 / SC1).
#[tokio::test]
async fn full_identity_coverage() {
    let pool = setup_pool().await;
    let fixtures = seed_fixtures(pool.as_ref()).await;

    run_backfill_section(pool.as_ref())
        .await
        .expect("Backfill-Abschnitt muss durchlaufen");

    let sales_person_gaps = count(
        pool.as_ref(),
        "SELECT COUNT(*) FROM sales_person WHERE person_id IS NULL",
    )
    .await;
    assert_eq!(
        sales_person_gaps, 0,
        "I-1: jede sales_person-Zeile (auch die soft-geloeschte) muss nach dem Backfill eine person_id tragen"
    );

    let user_gaps = count(
        pool.as_ref(),
        "SELECT COUNT(*) FROM user WHERE person_id IS NULL",
    )
    .await;
    assert_eq!(
        user_gaps, 0,
        "I-1: jede user-Zeile muss nach dem Backfill eine person_id tragen"
    );

    let linked_sales_person_id =
        person_id_of_sales_person(pool.as_ref(), fixtures.linked_sales_person).await;
    let linked_user_id = person_id_of_user(pool.as_ref(), LINKED_USER).await;
    assert!(
        linked_sales_person_id.is_some(),
        "I-2: die Sales-Person-Seite des verknuepften Paars braucht eine person_id"
    );
    assert_eq!(
        linked_sales_person_id, linked_user_id,
        "I-2: beide Seiten eines sales_person_user-Paars muessen auf dieselbe person.id zeigen"
    );

    let mismatched_pairs = count(
        pool.as_ref(),
        "SELECT COUNT(*) FROM sales_person_user spu \
         JOIN sales_person sp ON sp.id = spu.sales_person_id \
         JOIN user u ON u.name = spu.user_id \
         WHERE sp.person_id IS NOT u.person_id",
    )
    .await;
    assert_eq!(
        mismatched_pairs, 0,
        "I-2: kein sales_person_user-Paar darf nach dem Backfill auf zwei verschiedene Personen zeigen"
    );
}

/// I-3 (PERS-01 / SC1).
#[tokio::test]
async fn count_identity_holds() {
    let pool = setup_pool().await;
    seed_fixtures(pool.as_ref()).await;

    run_backfill_section(pool.as_ref())
        .await
        .expect("Backfill-Abschnitt muss durchlaufen");

    let persons = count(pool.as_ref(), "SELECT COUNT(*) FROM person").await;
    let sales_persons = count(pool.as_ref(), "SELECT COUNT(*) FROM sales_person").await;
    let users = count(pool.as_ref(), "SELECT COUNT(*) FROM user").await;
    let links = count(pool.as_ref(), "SELECT COUNT(*) FROM sales_person_user").await;

    assert_eq!(
        persons,
        sales_persons + users - links,
        "I-3: COUNT(person) == COUNT(sales_person) + COUNT(user) - COUNT(sales_person_user)"
    );
}

/// I-3-negativ (D-60-07): eine verletzte Zaehlprobe bricht den Lauf ab.
#[tokio::test]
async fn migration_aborts_on_count_mismatch() {
    let pool = setup_pool().await;
    seed_fixtures(pool.as_ref()).await;

    // Eine zusaetzliche person-Zeile, die von niemandem referenziert wird —
    // damit geht die Zaehlprobe nach dem Backfill garantiert nicht auf.
    sqlx::query(
        "INSERT INTO person (id, created, update_process, update_version) \
         VALUES (?1, '2025-01-01T00:00:00', 'test-verwaiste-person', ?2)",
    )
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(pool.as_ref())
    .await
    .expect("seed verwaiste person");

    let result = run_backfill_section(pool.as_ref()).await;

    assert!(
        result.is_err(),
        "D-60-07: die Selbst-Assertion muss den Lauf mit einem Datenbankfehler abbrechen, \
         statt einen halb ausgefuehrten Backfill zu committen — Ergebnis war: {result:?}"
    );
}

/// I-5 (PERS-03 / SC3).
#[tokio::test]
async fn soft_deleted_sales_person_gets_person_id() {
    let pool = setup_pool().await;
    let fixtures = seed_fixtures(pool.as_ref()).await;

    run_backfill_section(pool.as_ref())
        .await
        .expect("Backfill-Abschnitt muss durchlaufen");

    let person_id =
        person_id_of_sales_person(pool.as_ref(), fixtures.soft_deleted_sales_person).await;
    assert!(
        person_id.is_some(),
        "I-5: eine soft-geloeschte Sales Person mit Abrechnungs-Snapshot muss eine person_id bekommen \
         (der Backfill darf NICHT auf den Soft-Delete-Reflex filtern)"
    );

    let snapshot_rows = count(
        pool.as_ref(),
        "SELECT COUNT(*) FROM billing_period_sales_person",
    )
    .await;
    assert_eq!(
        snapshot_rows, 1,
        "PERS-06: der write-once-Snapshot bleibt unveraendert bestehen"
    );
}

/// I-8 (D-60-03), Seite Benutzer.
#[tokio::test]
async fn hr_user_without_sales_person() {
    let pool = setup_pool().await;
    seed_fixtures(pool.as_ref()).await;

    let sales_persons_before = count(pool.as_ref(), "SELECT COUNT(*) FROM sales_person").await;

    run_backfill_section(pool.as_ref())
        .await
        .expect("Backfill-Abschnitt muss durchlaufen");

    let person_id = person_id_of_user(pool.as_ref(), HR_USER).await;
    let person_id = person_id.expect("I-8: ein HR-/Admin-Benutzer ohne Sales Person braucht eine eigene Person");

    let carrier_side = sqlx::query("SELECT COUNT(*) FROM sales_person WHERE person_id = ?1")
        .bind(person_id)
        .fetch_one(pool.as_ref())
        .await
        .expect("count sales_person for hr person")
        .get::<i64, _>(0);
    assert_eq!(
        carrier_side, 0,
        "I-8/D-60-03: zu einem HR-Benutzer darf KEINE sales_person automatisch mit angelegt werden"
    );

    let sales_persons_after = count(pool.as_ref(), "SELECT COUNT(*) FROM sales_person").await;
    assert_eq!(
        sales_persons_before, sales_persons_after,
        "I-8/D-60-03: der Backfill legt keine zusaetzlichen sales_person-Zeilen an"
    );
}

/// I-8 (D-60-03), Seite Sales Person.
#[tokio::test]
async fn volunteer_without_user() {
    let pool = setup_pool().await;
    let fixtures = seed_fixtures(pool.as_ref()).await;

    let users_before = count(pool.as_ref(), "SELECT COUNT(*) FROM user").await;

    run_backfill_section(pool.as_ref())
        .await
        .expect("Backfill-Abschnitt muss durchlaufen");

    let person_id =
        person_id_of_sales_person(pool.as_ref(), fixtures.volunteer_sales_person).await;
    let person_id =
        person_id.expect("I-8: eine freiwillige Sales Person ohne Login braucht eine eigene Person");

    let carrier_side = sqlx::query("SELECT COUNT(*) FROM user WHERE person_id = ?1")
        .bind(person_id)
        .fetch_one(pool.as_ref())
        .await
        .expect("count user for volunteer person")
        .get::<i64, _>(0);
    assert_eq!(
        carrier_side, 0,
        "I-8/D-60-03: zu einer Freiwilligen darf KEIN user automatisch mit angelegt werden"
    );

    let users_after = count(pool.as_ref(), "SELECT COUNT(*) FROM user").await;
    assert_eq!(
        users_before, users_after,
        "I-8/D-60-03: der Backfill legt keine zusaetzlichen user-Zeilen an"
    );
}

/// I-4 (PERS-02 / SC2, D-60-08).
#[tokio::test]
async fn apply_twice_is_noop() {
    let pool = setup_pool().await;
    seed_fixtures(pool.as_ref()).await;

    run_backfill_section(pool.as_ref())
        .await
        .expect("erster Lauf muss durchlaufen");

    let before = snapshot(pool.as_ref()).await;

    // Vorbedingung: der erste Lauf hat tatsaechlich Identitaeten vergeben.
    // Ohne diese Pruefung waere der Vergleich zweier leerer Zustaende trivial
    // gruen und der Nachweis wertlos (D-60-08: "laeuft durch" reicht nicht).
    assert!(
        before.person_count > 0,
        "Vorbedingung I-4: der erste Lauf muss person-Zeilen erzeugt haben"
    );
    assert!(
        before
            .sales_person_rows
            .iter()
            .all(|(_, person_id)| person_id.is_some()),
        "Vorbedingung I-4: der erste Lauf muss jede sales_person-Zeile zugeordnet haben"
    );
    assert!(
        before
            .user_rows
            .iter()
            .all(|(_, person_id)| person_id.is_some()),
        "Vorbedingung I-4: der erste Lauf muss jede user-Zeile zugeordnet haben"
    );

    run_backfill_section(pool.as_ref())
        .await
        .expect("zweiter Lauf darf nicht fehlschlagen (Idempotenz + Selbst-Assertion bleiben gruen)");

    let after = snapshot(pool.as_ref()).await;

    assert_eq!(
        before, after,
        "I-4: ein zweiter Lauf des Backfill-Abschnitts darf weder COUNT(person) noch eine einzige Zuordnung veraendern"
    );
}

/// RESEARCH Frage 4 — Vorab-Absicherung fuer den `PersonDao` in Phase 61.
#[tokio::test]
async fn person_id_parses_as_uuid() {
    let pool = setup_pool().await;
    seed_fixtures(pool.as_ref()).await;

    run_backfill_section(pool.as_ref())
        .await
        .expect("Backfill-Abschnitt muss durchlaufen");

    let rows = sqlx::query("SELECT id FROM person")
        .fetch_all(pool.as_ref())
        .await
        .expect("read person ids");

    assert!(
        !rows.is_empty(),
        "Vorbedingung: der Backfill muss person-Zeilen erzeugt haben"
    );

    for row in rows {
        let bytes = row.get::<Vec<u8>, _>("id");
        assert_eq!(
            bytes.len(),
            16,
            "person.id muss 16 rohe Bytes sein (kein Hex-String) — sonst scheitert Uuid::from_slice in Phase 61"
        );
        Uuid::from_slice(&bytes).expect("person.id muss mit Uuid::from_slice parsebar sein");
    }
}

// ─── Plan 60-02, Task 1: DB-Level-Ablehnung der Doppel-Identitaet ───────────

/// I-6 (PERS-04 / SC4), Seite Sales Person.
///
/// Der Angriff wird als direkter `UPDATE` gefahren — nicht ueber eine
/// Applikationspruefung, denn in Phase 60 gibt es dorthin gar keinen
/// Applikationspfad (D-60-10). Bewusst NICHT ueber eine
/// Fremdschluessel-Verletzung: die Fremdschluessel-Durchsetzung ist in diesem
/// Repo nirgends eingeschaltet, ein FK-basierter Test wuerde lautlos bestehen
/// und nichts beweisen (D-60-06).
#[tokio::test]
async fn duplicate_person_id_rejected_by_db_sales_person() {
    let pool = setup_pool().await;

    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    seed_sales_person(pool.as_ref(), first, "Erste Verkaeuferin", None).await;
    seed_sales_person(pool.as_ref(), second, "Zweite Verkaeuferin", None).await;

    run_backfill_section(pool.as_ref())
        .await
        .expect("Backfill-Abschnitt muss durchlaufen");

    let first_person_id = person_id_of_sales_person(pool.as_ref(), first)
        .await
        .expect("Vorbedingung I-6: die erste Sales Person muss eine person_id tragen");
    let second_person_id = person_id_of_sales_person(pool.as_ref(), second)
        .await
        .expect("Vorbedingung I-6: die zweite Sales Person muss eine person_id tragen");
    assert_ne!(
        first_person_id, second_person_id,
        "Vorbedingung I-6: zwei unverknuepfte Sales Persons muessen auf zwei verschiedene \
         Personen zeigen — sonst waere der Doppel-Link-Versuch unten gar kein Versuch"
    );

    let result = sqlx::query("UPDATE sales_person SET person_id = ?1 WHERE id = ?2")
        .bind(first_person_id.clone())
        .bind(second.as_bytes().to_vec())
        .execute(pool.as_ref())
        .await;

    assert!(
        result.is_err(),
        "I-6/PERS-04: die Datenbank muss zwei sales_person-Zeilen auf derselben person.id \
         ablehnen — Ergebnis war: {result:?}"
    );
    let err = result.unwrap_err();
    let is_database_error = matches!(err, sqlx::Error::Database(_));
    assert!(
        is_database_error,
        "I-6: die Ablehnung muss von der Datenbank kommen (sqlx::Error::Database), \
         nicht von einer Applikationsschicht — Fehler war: {err:?}"
    );
    assert!(
        err.to_string().to_lowercase().contains("unique"),
        "I-6/D-60-05: die Ablehnung muss aus dem UNIQUE-Index stammen — Fehlertext war: {err}"
    );

    let unchanged = person_id_of_sales_person(pool.as_ref(), second).await;
    assert_eq!(
        unchanged,
        Some(second_person_id),
        "I-6: die abgelehnte Schreibung darf nichts halb geschrieben haben — die zweite \
         Zeile muss ihre urspruengliche person_id behalten"
    );

    // Gegenprobe (D-60-03 / T-60-12): der vollstaendige UNIQUE-Index verbietet
    // NICHT mehrere Zeilen ohne Person-Zuordnung. SQLite behandelt leere Werte
    // in einem UNIQUE-Index als verschieden. Genau darauf beruht die dauerhafte
    // Optionalitaet beider Spalten — und es deckt D-60-10 ab: Zeilen, die nach
    // der Migration entstehen, bekommen in Phase 60 bewusst noch keine Person.
    let third = Uuid::new_v4();
    let fourth = Uuid::new_v4();
    seed_sales_person(pool.as_ref(), third, "Dritte ohne Person", None).await;
    seed_sales_person(pool.as_ref(), fourth, "Vierte ohne Person", None).await;

    assert_eq!(
        person_id_of_sales_person(pool.as_ref(), third).await,
        None,
        "D-60-10: eine nach der Migration angelegte sales_person bekommt keine person_id"
    );
    let unassigned = count(
        pool.as_ref(),
        "SELECT COUNT(*) FROM sales_person WHERE person_id IS NULL",
    )
    .await;
    assert_eq!(
        unassigned, 2,
        "D-60-03: zwei sales_person-Zeilen ohne Person-Zuordnung muessen gleichzeitig \
         existieren duerfen — der vollstaendige UNIQUE-Index darf die Optionalitaet nicht brechen"
    );
}

/// I-6b (PERS-04 / SC4, D-60-02), Seite Benutzer.
#[tokio::test]
async fn duplicate_person_id_rejected_by_db_user() {
    let pool = setup_pool().await;

    let first = "erster-benutzer";
    let second = "zweiter-benutzer";
    seed_user(pool.as_ref(), first).await;
    seed_user(pool.as_ref(), second).await;

    run_backfill_section(pool.as_ref())
        .await
        .expect("Backfill-Abschnitt muss durchlaufen");

    let first_person_id = person_id_of_user(pool.as_ref(), first)
        .await
        .expect("Vorbedingung I-6b: der erste Benutzer muss eine person_id tragen");
    let second_person_id = person_id_of_user(pool.as_ref(), second)
        .await
        .expect("Vorbedingung I-6b: der zweite Benutzer muss eine person_id tragen");
    assert_ne!(
        first_person_id, second_person_id,
        "Vorbedingung I-6b: zwei unverknuepfte Benutzer muessen auf zwei verschiedene \
         Personen zeigen"
    );

    let result = sqlx::query("UPDATE user SET person_id = ?1 WHERE name = ?2")
        .bind(first_person_id.clone())
        .bind(second)
        .execute(pool.as_ref())
        .await;

    assert!(
        result.is_err(),
        "I-6b/PERS-04/D-60-02: die Datenbank muss zwei user-Zeilen auf derselben person.id \
         ablehnen — Ergebnis war: {result:?}"
    );
    let err = result.unwrap_err();
    let is_database_error = matches!(err, sqlx::Error::Database(_));
    assert!(
        is_database_error,
        "I-6b: die Ablehnung muss von der Datenbank kommen (sqlx::Error::Database) — \
         Fehler war: {err:?}"
    );
    assert!(
        err.to_string().to_lowercase().contains("unique"),
        "I-6b/D-60-05: die Ablehnung muss aus dem UNIQUE-Index stammen — Fehlertext war: {err}"
    );

    let unchanged = person_id_of_user(pool.as_ref(), second).await;
    assert_eq!(
        unchanged,
        Some(second_person_id),
        "I-6b: die abgelehnte Schreibung darf nichts halb geschrieben haben"
    );

    // Gegenprobe (D-60-03 / T-60-12): reine HR-/Admin-Accounts ohne Sales
    // Person bleiben dauerhaft anlegbar, auch mehrere gleichzeitig ohne
    // Person-Zuordnung.
    seed_user(pool.as_ref(), "dritter-benutzer-ohne-person").await;
    seed_user(pool.as_ref(), "vierter-benutzer-ohne-person").await;

    assert_eq!(
        person_id_of_user(pool.as_ref(), "dritter-benutzer-ohne-person").await,
        None,
        "D-60-10: ein nach der Migration angelegter user bekommt keine person_id"
    );
    let unassigned = count(
        pool.as_ref(),
        "SELECT COUNT(*) FROM user WHERE person_id IS NULL",
    )
    .await;
    assert_eq!(
        unassigned, 2,
        "D-60-03: zwei user-Zeilen ohne Person-Zuordnung muessen gleichzeitig existieren \
         duerfen — Freiwillige ohne Login und HR-Accounts ohne Sales Person bleiben anlegbar"
    );
}
