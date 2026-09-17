//! Phase 61 Plan 01 (PERS-05 / D-61-02) — Integrationstests fuer die beiden
//! „unverknuepfte Kandidaten"-Lesepfade des Basic-Tier `PersonDao`.
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

use std::sync::Arc;

use dao::person::PersonDao;
use dao_impl_sqlite::person::PersonDaoImpl;
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
}

const LINKED_USER: &str = "u2-verknuepft";
const UNLINKED_USER: &str = "u4-ohne-sales-person";

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

    for person in [p1, p2, p3, p4] {
        seed_person(pool, person).await;
    }

    let unlinked_sales_person = Uuid::new_v4();
    let linked_sales_person = Uuid::new_v4();
    let deleted_sales_person = Uuid::new_v4();

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

    seed_user(pool, LINKED_USER, p2).await;
    seed_user(pool, UNLINKED_USER, p4).await;

    Fixtures {
        unlinked_sales_person,
        linked_sales_person,
        deleted_sales_person,
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
