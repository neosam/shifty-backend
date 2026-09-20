//! Phase 61 Plan 02 (PERS-05 / D-61-01) — Integrationstests fuer die
//! `person_id`-Vergabe bei **Neuanlage**.
//!
//! Der Phase-60-Backfill hat jeder Bestandszeile eine Person gegeben. Ohne
//! diesen Plan faellt aber jede *nach* dem Deploy neu angelegte Sales Person
//! bzw. jeder neue Login sofort wieder aus der person-basierten
//! Selbst-Zugriffs-Pruefung heraus (Plan 61-03). Die Tests hier sind das Gate
//! dafuer.
//!
//! Die Tests fahren gegen eine echte SQLite-in-memory-DB (`sqlx::migrate!`) und
//! gegen die echten DAO-Implementierungen — bewusst **ohne** DAO-Mock: die
//! Vergabe ist reines SQL auf einer Transaktion, ein gemockter DAO wuerde weder
//! die Atomaritaet noch den UNIQUE-Index ausfuehren und damit nichts beweisen
//! (Muster: `person_candidates.rs`, `person_backfill.rs`).
//!
//! Abgedeckte Invarianten:
//!
//! - **N-1** (D-61-01): `SalesPersonDaoImpl::create` vergibt ein nicht-NULL
//!   `person_id` mit genau einer passenden `person`-Zeile
//!   (`create_sales_person_vergibt_person_id`).
//! - **N-2** (T-61-09): Zwei `create`-Aufrufe erzeugen zwei verschiedene
//!   Personen — der vollstaendige `sales_person_person_id_unique_idx` haelt
//!   (`create_sales_person_vergibt_je_zeile_eine_eigene_person`).
//! - **N-3** (D-60-03 / T-61-10): `create` legt **keine** `user`-Zeile an
//!   (`create_sales_person_legt_keinen_user_an`).
//! - **N-4**: Die Person-Anlage haengt an der uebergebenen Transaktion — ein
//!   Rollback laesst weder `sales_person`- noch `person`-Zeile zurueck
//!   (`create_sales_person_rollback_laesst_keine_person_zurueck`).
//!
//! Ergaenzt in Task 2 (Login-Auto-Anlage, `PermissionDaoImpl::create_user`):
//!
//! - **N-5** (D-61-01): `create_user` vergibt ein nicht-NULL `person_id` mit
//!   genau einer passenden `person`-Zeile (`create_user_vergibt_person_id`).
//! - **N-6** (T-61-06): Zwei Benutzer bekommen zwei verschiedene Personen — der
//!   vollstaendige `user_person_id_unique_idx` haelt
//!   (`create_user_vergibt_je_benutzer_eine_eigene_person`).
//! - **N-7** (D-60-03 / T-61-10): `create_user` legt **keine** `sales_person`-
//!   Zeile an (`create_user_legt_keine_sales_person_an`).
//! - **N-8** (Research Pitfall 4, der Atomaritaets-Nachweis): Ein zweiter
//!   `create_user` mit demselben Namen scheitert am Primaerschluessel; die
//!   `person`-Anzahl bleibt danach **unveraendert** — die interne Transaktion
//!   hat die bereits geschriebene `person`-Zeile mitgenommen
//!   (`create_user_ist_atomar`). Ohne interne Transaktion bliebe hier eine
//!   traegerlose Person zurueck oder — schlimmer — ein `user` ohne Identitaet.
//!
//! Ergaenzt in Plan 61-08 (Gap-Closure Review-Befund **CR-02**,
//! `BasicDaoImpl::clear_all`):
//!
//! - **N-9** (CR-02 / T-61-29): Nach `clear_all` ist `sales_person` leer, aber
//!   jede ueberlebende `user`-Zeile traegt weiterhin eine aufloesbare
//!   `person_id` — die Logins ueberleben das Leeren und behalten ihre
//!   Identitaet (D-61-01)
//!   (`clear_all_laesst_keinen_benutzer_ohne_person_zurueck`).
//! - **N-10** (CR-02, Gegenprobe): Nach `clear_all` bleibt keine `person`-Zeile
//!   ohne Traeger zurueck — die Person der geleerten Sales Person verschwindet
//!   mit. Der Test schuetzt davor, dass der Fix in die andere Richtung kippt
//!   und das Loeschen ersatzlos entfaellt
//!   (`clear_all_raeumt_traegerlose_personen_auf`).

use std::sync::Arc;

use dao::sales_person::{SalesPersonDao, SalesPersonEntity};
use dao::{BasicDao, PermissionDao, TransactionDao, UserEntity};
use dao_impl_sqlite::sales_person::SalesPersonDaoImpl;
use dao_impl_sqlite::{BasicDaoImpl, PermissionDaoImpl, TransactionDaoImpl, TransactionImpl};
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

fn sales_person_entity(name: &str) -> SalesPersonEntity {
    SalesPersonEntity {
        id: Uuid::new_v4(),
        name: name.into(),
        background_color: "#ffffff".into(),
        is_paid: true,
        deleted: None,
        inactive: false,
        version: Uuid::new_v4(),
    }
}

// ─── Lese-Helfer ────────────────────────────────────────────────────────────

async fn count(pool: &sqlx::SqlitePool, sql: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(sql)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|err| panic!("count query failed ({sql}): {err}"))
}

async fn person_id_of_sales_person(pool: &sqlx::SqlitePool, id: Uuid) -> Option<Vec<u8>> {
    sqlx::query_scalar::<_, Option<Vec<u8>>>("SELECT person_id FROM sales_person WHERE id = ?1")
        .bind(id.as_bytes().to_vec())
        .fetch_optional(pool)
        .await
        .expect("select person_id")
        .flatten()
}

async fn person_row_count(pool: &sqlx::SqlitePool, person_id: &[u8]) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM person WHERE id = ?1")
        .bind(person_id.to_vec())
        .fetch_one(pool)
        .await
        .expect("count person")
}

// ─── N-1 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_sales_person_vergibt_person_id() {
    let pool = setup_pool().await;
    let dao = SalesPersonDaoImpl::new(pool.clone());
    let entity = sales_person_entity("S1 neu angelegt");

    let tx = tx_of(&pool).await;
    dao.create(&entity, "test-process", tx.clone())
        .await
        .expect("create sales person");
    commit(&pool, tx).await;

    let person_id = person_id_of_sales_person(pool.as_ref(), entity.id)
        .await
        .expect("D-61-01: neu angelegte Sales Person muss ein person_id tragen");

    assert_eq!(
        1,
        person_row_count(pool.as_ref(), &person_id).await,
        "zu dem vergebenen person_id muss genau eine person-Zeile existieren"
    );
}

// ─── N-2 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_sales_person_vergibt_je_zeile_eine_eigene_person() {
    let pool = setup_pool().await;
    let dao = SalesPersonDaoImpl::new(pool.clone());
    let first = sales_person_entity("S1 erste");
    let second = sales_person_entity("S2 zweite");

    let tx = tx_of(&pool).await;
    dao.create(&first, "test-process", tx.clone())
        .await
        .expect("create first");
    dao.create(&second, "test-process", tx.clone())
        .await
        .expect("create second — der UNIQUE-Index darf nicht greifen (T-61-09)");
    commit(&pool, tx).await;

    let first_person = person_id_of_sales_person(pool.as_ref(), first.id)
        .await
        .expect("first person_id");
    let second_person = person_id_of_sales_person(pool.as_ref(), second.id)
        .await
        .expect("second person_id");

    assert_ne!(
        first_person, second_person,
        "zwei Sales Persons duerfen niemals dieselbe Person teilen (PERS-04)"
    );
    assert_eq!(
        2,
        count(pool.as_ref(), "SELECT COUNT(*) FROM person").await,
        "zwei create-Aufrufe muessen zwei person-Zeilen erzeugen"
    );
}

// ─── N-3 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_sales_person_legt_keinen_user_an() {
    let pool = setup_pool().await;
    let dao = SalesPersonDaoImpl::new(pool.clone());
    let before = count(pool.as_ref(), "SELECT COUNT(*) FROM user").await;

    let tx = tx_of(&pool).await;
    dao.create(
        &sales_person_entity("S1 ohne Login"),
        "test-process",
        tx.clone(),
    )
    .await
    .expect("create sales person");
    commit(&pool, tx).await;

    assert_eq!(
        before,
        count(pool.as_ref(), "SELECT COUNT(*) FROM user").await,
        "D-60-03/T-61-10: die Gegenseite wird NICHT automatisch mit angelegt"
    );
}

// ─── N-4 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_sales_person_rollback_laesst_keine_person_zurueck() {
    let pool = setup_pool().await;
    let dao = SalesPersonDaoImpl::new(pool.clone());
    let entity = sales_person_entity("S1 verworfen");

    let tx = tx_of(&pool).await;
    dao.create(&entity, "test-process", tx.clone())
        .await
        .expect("create sales person");
    rollback(&pool, tx).await;

    assert_eq!(
        None,
        person_id_of_sales_person(pool.as_ref(), entity.id).await,
        "die verworfene Sales Person darf nicht sichtbar sein"
    );
    assert_eq!(
        0,
        count(pool.as_ref(), "SELECT COUNT(*) FROM person").await,
        "die person-Zeile haengt an derselben Transaktion — ein Rollback nimmt sie mit"
    );
}

// ─── Lese-Helfer (Benutzer-Seite) ───────────────────────────────────────────

async fn person_id_of_user(pool: &sqlx::SqlitePool, name: &str) -> Option<Vec<u8>> {
    sqlx::query_scalar::<_, Option<Vec<u8>>>("SELECT person_id FROM user WHERE name = ?1")
        .bind(name)
        .fetch_optional(pool)
        .await
        .expect("select person_id")
        .flatten()
}

fn user_entity(name: &str) -> UserEntity {
    UserEntity { name: name.into() }
}

// ─── N-5 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_user_vergibt_person_id() {
    let pool = setup_pool().await;
    let dao = PermissionDaoImpl::new(pool.clone());

    let tx = tx_of(&pool).await;
    dao.create_user(&user_entity("alice"), None, "test-process", tx.clone())
        .await
        .expect("create user");
    commit(&pool, tx).await;

    let person_id = person_id_of_user(pool.as_ref(), "alice")
        .await
        .expect("D-61-01: ein neu angelegter Benutzer muss ein person_id tragen");

    assert_eq!(
        1,
        person_row_count(pool.as_ref(), &person_id).await,
        "zu dem vergebenen person_id muss genau eine person-Zeile existieren"
    );
}

// ─── N-6 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_user_vergibt_je_benutzer_eine_eigene_person() {
    let pool = setup_pool().await;
    let dao = PermissionDaoImpl::new(pool.clone());

    let tx = tx_of(&pool).await;
    dao.create_user(&user_entity("alice"), None, "test-process", tx.clone())
        .await
        .expect("create alice");
    dao.create_user(&user_entity("bob"), None, "test-process", tx.clone())
        .await
        .expect("create bob — der UNIQUE-Index darf nicht greifen (T-61-06)");
    commit(&pool, tx).await;

    let alice = person_id_of_user(pool.as_ref(), "alice")
        .await
        .expect("alice person_id");
    let bob = person_id_of_user(pool.as_ref(), "bob")
        .await
        .expect("bob person_id");

    assert_ne!(
        alice, bob,
        "zwei Benutzer duerfen niemals dieselbe Person teilen (PERS-04 / T-61-06)"
    );
    assert_eq!(
        2,
        count(pool.as_ref(), "SELECT COUNT(*) FROM person").await,
        "zwei create_user-Aufrufe muessen zwei person-Zeilen erzeugen"
    );
}

// ─── N-7 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_user_legt_keine_sales_person_an() {
    let pool = setup_pool().await;
    let dao = PermissionDaoImpl::new(pool.clone());
    let before = count(pool.as_ref(), "SELECT COUNT(*) FROM sales_person").await;

    let tx = tx_of(&pool).await;
    dao.create_user(&user_entity("alice"), None, "test-process", tx.clone())
        .await
        .expect("create user");
    commit(&pool, tx).await;

    assert_eq!(
        before,
        count(pool.as_ref(), "SELECT COUNT(*) FROM sales_person").await,
        "D-60-03/T-61-10: ein reiner HR-/Admin-Account bekommt KEINE Schichtplan-Identitaet"
    );
}

// ─── N-8 — Atomaritaets-Nachweis (Research Pitfall 4) ───────────────────────

#[tokio::test]
async fn create_user_ist_atomar() {
    let pool = setup_pool().await;
    let dao = PermissionDaoImpl::new(pool.clone());

    let tx = tx_of(&pool).await;
    dao.create_user(&user_entity("alice"), None, "test-process", tx.clone())
        .await
        .expect("create alice");
    commit(&pool, tx).await;

    let persons_before = count(pool.as_ref(), "SELECT COUNT(*) FROM person").await;
    assert_eq!(1, persons_before);

    // Phase 63 Plan 01: Die Atomaritaet liegt jetzt bei der **uebergebenen**
    // Transaktion statt bei einer methodeninternen. Der Nachweis bleibt
    // derselbe — er wird nur eine Ebene hoeher gefuehrt: der fehlgeschlagene
    // Aufruf wird vom Aufrufer verworfen und darf nichts zuruecklassen.
    let tx = tx_of(&pool).await;
    let second = dao
        .create_user(&user_entity("alice"), None, "test-process", tx.clone())
        .await;
    assert!(
        second.is_err(),
        "ein zweiter Benutzer mit demselben Namen muss am Primaerschluessel scheitern"
    );
    rollback(&pool, tx).await;

    assert_eq!(
        persons_before,
        count(pool.as_ref(), "SELECT COUNT(*) FROM person").await,
        "Pitfall 4: der fehlgeschlagene Aufruf darf KEINE traegerlose person-Zeile hinterlassen"
    );
    assert_eq!(
        1,
        count(pool.as_ref(), "SELECT COUNT(*) FROM user").await,
        "der fehlgeschlagene Aufruf darf den Bestand nicht veraendern"
    );
}

// ─── N-9 — clear_all laesst keinen Benutzer ohne Person zurueck ─────────────

#[tokio::test]
async fn clear_all_laesst_keinen_benutzer_ohne_person_zurueck() {
    let pool = setup_pool().await;
    // clear_all nimmt KEINE Transaktion und arbeitet direkt auf dem Pool — die
    // Fixture muss deshalb vorher committet sein, sonst sieht clear_all sie nicht.
    let tx = tx_of(&pool).await;
    PermissionDaoImpl::new(pool.clone())
        .create_user(&user_entity("alice"), None, "test-process", tx.clone())
        .await
        .expect("create alice");
    SalesPersonDaoImpl::new(pool.clone())
        .create(&sales_person_entity("S1"), "test-process", tx.clone())
        .await
        .expect("create sales person");
    commit(&pool, tx).await;

    BasicDaoImpl::new(pool.clone())
        .clear_all()
        .await
        .expect("clear_all");

    assert_eq!(
        0,
        count(pool.as_ref(), "SELECT COUNT(*) FROM sales_person").await,
        "clear_all muss sales_person weiterhin vollstaendig leeren"
    );

    let person_id = person_id_of_user(pool.as_ref(), "alice").await.expect(
        "CR-02: jede user-Zeile traegt eine aufloesbare person_id — auch nach clear_all; \
         die Datenbank faengt das nicht ab, user.person_id hat keine FOREIGN KEY-Deklaration",
    );
    assert_eq!(
        1,
        person_row_count(pool.as_ref(), &person_id).await,
        "CR-02: jede person-Zeile hat mindestens einen Traeger, und umgekehrt muss die \
         person_id einer ueberlebenden user-Zeile aufloesbar bleiben; ein pauschales \
         DELETE FROM person hinterlaesst hier einen identitaetslosen Benutzer, und die \
         Datenbank meldet das nicht, weil user.person_id keine FOREIGN KEY-Deklaration traegt"
    );
}

// ─── N-10 — clear_all raeumt traegerlose Personen auf (Gegenprobe zu N-9) ───

#[tokio::test]
async fn clear_all_raeumt_traegerlose_personen_auf() {
    let pool = setup_pool().await;
    // clear_all nimmt KEINE Transaktion und arbeitet direkt auf dem Pool — die
    // Fixture muss deshalb vorher committet sein, sonst sieht clear_all sie nicht.
    let tx = tx_of(&pool).await;
    PermissionDaoImpl::new(pool.clone())
        .create_user(&user_entity("alice"), None, "test-process", tx.clone())
        .await
        .expect("create alice");
    SalesPersonDaoImpl::new(pool.clone())
        .create(&sales_person_entity("S1"), "test-process", tx.clone())
        .await
        .expect("create sales person");
    commit(&pool, tx).await;

    BasicDaoImpl::new(pool.clone())
        .clear_all()
        .await
        .expect("clear_all");

    assert_eq!(
        0,
        count(
            pool.as_ref(),
            "SELECT COUNT(*) FROM person p \
             WHERE NOT EXISTS (SELECT 1 FROM sales_person sp WHERE sp.person_id = p.id) \
               AND NOT EXISTS (SELECT 1 FROM user u WHERE u.person_id = p.id)"
        )
        .await,
        "CR-02: jede person-Zeile hat mindestens einen Traeger — die Person der geleerten \
         Sales Person muss mit verschwinden; die Datenbank faengt das nicht ab"
    );
    assert_eq!(
        count(pool.as_ref(), "SELECT COUNT(*) FROM user").await,
        count(pool.as_ref(), "SELECT COUNT(*) FROM person").await,
        "nach clear_all sind die Benutzer die einzigen verbliebenen Traeger: die Person der \
         Sales Person ist weg, die des Benutzers steht noch"
    );
}
