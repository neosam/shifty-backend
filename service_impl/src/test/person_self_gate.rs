//! Phase 61 Plan 03 (PERS-05 / D-61-08, D-61-09) — Integrationstests fuer die
//! **Selbst-Zugriffs-Pruefung** auf Basis von `person_id`-Joins.
//!
//! `service_impl/src/sales_person.rs::verify_user_is_sales_person` entscheidet
//! anhand von `get_assigned_user`, ob ein eingeloggter Mitarbeiter „seine
//! eigene" Sales Person anfassen darf. `find_sales_person_by_user_id` traegt
//! denselben Gate auf den Lese-Pfad. Ein fail-open an dieser Stelle ist ein
//! Fremdzugriff auf Buchungs- und Lohndaten — deshalb faellt die Entscheidung
//! ausschliesslich im SQL-`JOIN` (`u.person_id = sp.person_id`), wo SQLs
//! Drei-Werte-Logik `NULL = NULL` nie wahr werden laesst (D-61-08).
//!
//! Die Tests fahren gegen eine echte SQLite-in-memory-DB (`sqlx::migrate!`) und
//! gegen die echte DAO-Implementierung — bewusst **ohne** DAO-Mock: ein
//! gemocktes DAO fuehrt kein SQL aus und koennte die eigentliche
//! Sicherheitseigenschaft (Entscheidung im Join statt in Rust) strukturell
//! nicht pruefen (Muster: `person_creation.rs`, `person_candidates.rs`).
//!
//! Abgedeckte Invarianten:
//!
//! - **G-1** (SC1, „self darf"): `find_sales_person_by_user_id(U_self)` liefert
//!   genau `S_self` (`self_may_access_own_sales_person`).
//! - **G-2** (SC1, „fremd darf nicht", T-61-02): `U_fremd` bekommt `S_fremd`
//!   und **nicht** `S_self` — der Gleichheitsvergleich in
//!   `verify_user_is_sales_person` schlaegt damit fuer `S_self` fehl
//!   (`foreign_sales_person_rejected`).
//! - **G-3** (SC1, „unverknuepft darf nicht"): ein Benutzer mit eigener Person,
//!   aber ohne Sales Person, bekommt `None` (`unlinked_user_rejected`).
//! - **G-4** (D-61-08 / T-61-01, der Kernnachweis): Sales Person **und**
//!   Benutzer mit `person_id = NULL` matchen sich **nicht**. Ein Rust-seitiger
//!   `Option<Uuid>`-Vergleich wuerde hier `None == None` als `true` werten und
//!   fail-open gehen (`null_person_id_does_not_match_null_person_id`).
//! - **G-5**: Eine ausschliesslich ueber `person_id` bestehende Verknuepfung
//!   loest auf (`person_id_link_without_legacy_row_resolves`).
//! - **G-6** (D-61-09): Alt-Query und Neu-Query liefern auf **einer**
//!   dual-befuellten Fixture-DB dieselbe Accept/Reject-Entscheidung und dieselbe
//!   Sales-Person-ID (`byte_identity_old_vs_new_query`).
//! - **G-7**: `find_by_user` und `find_sales_person_by_user_id` sind
//!   deckungsgleich — die vorbestehende Duplikation bleibt erhalten, wird aber
//!   nachgewiesen (`find_by_user_und_find_sales_person_by_user_id_sind_deckungsgleich`).
//!
//! **Befristetes Artefakt:** Die mit `LEGACY-VERGLEICH START` / `LEGACY-VERGLEICH
//! ENDE` markierte Region ist die „Vorher"-Haelfte von D-61-09. Sie zitiert
//! zwangslaeufig das abgeloeste Datenmodell und kann deshalb nur existieren,
//! solange dieses noch da ist. Plan 61-05 entfernt den Block **vollstaendig**
//! zusammen mit der Migration `20260917000000_drop-sales-person-user.sql`, weil
//! Success Criterion 2 einen sauberen Endzustand verlangt. Die Marker sind Teil
//! dieses Vertrags — wer sie umbenennt, macht den Abbau manuell.

use std::sync::Arc;

use dao::sales_person::{SalesPersonDao, SalesPersonEntity};
use dao::TransactionDao;
use dao_impl_sqlite::sales_person::SalesPersonDaoImpl;
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

fn dao(pool: &Arc<sqlx::SqlitePool>) -> SalesPersonDaoImpl {
    SalesPersonDaoImpl::new(pool.clone())
}

// ─── Fixture ────────────────────────────────────────────────────────────────

const U_SELF: &str = "u-self";
const U_FREMD: &str = "u-fremd";
const U_ALLEIN: &str = "u-allein";
const U_NUR_PERSON_ID: &str = "u-nur-person-id";
const U_OHNE_PERSON: &str = "u-ohne-person";

/// Die Sales-Person-IDs der Fixture. Die Benutzernamen sind Konstanten (oben),
/// weil `user.name` der Primaerschluessel ist.
struct Fixture {
    /// Verknuepft mit `U_SELF` ueber eine gemeinsame Person.
    s_self: Uuid,
    /// Verknuepft mit `U_FREMD` ueber eine **andere** gemeinsame Person.
    s_fremd: Uuid,
    /// Verknuepft mit `U_NUR_PERSON_ID` ueber eine gemeinsame Person.
    s_nur_person_id: Uuid,
    /// Eigene Person, aber kein Benutzer (Freiwillige ohne Login, D-60-03).
    s_ohne_user: Uuid,
    /// `person_id IS NULL` — wie `U_OHNE_PERSON`. Beide duerfen sich **nicht**
    /// finden (D-61-08).
    s_ohne_person: Uuid,
}

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

async fn seed_sales_person(
    pool: &sqlx::SqlitePool,
    id: Uuid,
    name: &str,
    person_id: Option<Uuid>,
) {
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

/// Baut die gemeinsame Fixture **ausschliesslich** ueber `person_id`. Die
/// „Vorher"-Haelfte (Zeilen im abgeloesten Datenmodell) legt nur der
/// D-61-09-Vergleichstest zusaetzlich an — siehe LEGACY-VERGLEICH-Region unten.
async fn seed_fixture(pool: &sqlx::SqlitePool) -> Fixture {
    let p_self = Uuid::new_v4();
    let p_fremd = Uuid::new_v4();
    let p_allein = Uuid::new_v4();
    let p_nur_person_id = Uuid::new_v4();
    let p_ohne_user = Uuid::new_v4();

    for person in [p_self, p_fremd, p_allein, p_nur_person_id, p_ohne_user] {
        seed_person(pool, person).await;
    }

    let fixture = Fixture {
        s_self: Uuid::new_v4(),
        s_fremd: Uuid::new_v4(),
        s_nur_person_id: Uuid::new_v4(),
        s_ohne_user: Uuid::new_v4(),
        s_ohne_person: Uuid::new_v4(),
    };

    seed_sales_person(pool, fixture.s_self, "Selbst", Some(p_self)).await;
    seed_sales_person(pool, fixture.s_fremd, "Fremd", Some(p_fremd)).await;
    seed_sales_person(
        pool,
        fixture.s_nur_person_id,
        "Nur ueber Person verknuepft",
        Some(p_nur_person_id),
    )
    .await;
    seed_sales_person(pool, fixture.s_ohne_user, "Ohne Login", Some(p_ohne_user)).await;
    seed_sales_person(pool, fixture.s_ohne_person, "Ohne Person", None).await;

    seed_user(pool, U_SELF, Some(p_self)).await;
    seed_user(pool, U_FREMD, Some(p_fremd)).await;
    seed_user(pool, U_ALLEIN, Some(p_allein)).await;
    seed_user(pool, U_NUR_PERSON_ID, Some(p_nur_person_id)).await;
    seed_user(pool, U_OHNE_PERSON, None).await;

    fixture
}

async fn find_sales_person_by_user_id(
    pool: &Arc<sqlx::SqlitePool>,
    user_id: &str,
) -> Option<SalesPersonEntity> {
    let tx = tx_of(pool).await;
    let result = dao(pool)
        .find_sales_person_by_user_id(user_id, tx.clone())
        .await
        .expect("find_sales_person_by_user_id");
    commit(pool, tx).await;
    result
}

async fn find_by_user(pool: &Arc<sqlx::SqlitePool>, user_id: &str) -> Option<SalesPersonEntity> {
    let tx = tx_of(pool).await;
    let result = dao(pool)
        .find_by_user(user_id, tx.clone())
        .await
        .expect("find_by_user");
    commit(pool, tx).await;
    result
}

// ─── SC1-Matrix ─────────────────────────────────────────────────────────────

/// G-1 (SC1, „self darf"): Der eingeloggte Mitarbeiter findet genau seine
/// eigene Sales Person.
#[tokio::test]
async fn self_may_access_own_sales_person() {
    let pool = setup_pool().await;
    let fixture = seed_fixture(pool.as_ref()).await;

    let found = find_sales_person_by_user_id(&pool, U_SELF)
        .await
        .expect("SC1 'self darf': U_SELF muss seine eigene Sales Person finden");

    assert_eq!(
        found.id, fixture.s_self,
        "SC1 'self darf': U_SELF muss genau S_self bekommen"
    );
}

/// G-2 (SC1, „fremd darf nicht" / T-61-02): Ein fremder Datensatz bleibt
/// verwehrt. Geprueft wird die **Zuordnung**, nicht nur `is_some()` — ein
/// falsch formulierter Join (etwa `ON u.person_id = sp.id`) wuerde sonst
/// unbemerkt durchgehen.
#[tokio::test]
async fn foreign_sales_person_rejected() {
    let pool = setup_pool().await;
    let fixture = seed_fixture(pool.as_ref()).await;

    let found = find_sales_person_by_user_id(&pool, U_FREMD)
        .await
        .expect("U_FREMD hat eine eigene Sales Person");

    assert_eq!(
        found.id, fixture.s_fremd,
        "U_FREMD muss S_fremd bekommen, nicht irgendeine Sales Person"
    );
    assert_ne!(
        found.id, fixture.s_self,
        "SC1 'fremd darf nicht': U_FREMD darf S_self nie aufloesen — der \
         Gleichheitsvergleich in verify_user_is_sales_person muss fuer S_self fehlschlagen"
    );
}

/// G-3 (SC1, „unverknuepft darf nicht"): Ein Benutzer mit eigener Person, aber
/// ohne Sales Person, bekommt nichts.
#[tokio::test]
async fn unlinked_user_rejected() {
    let pool = setup_pool().await;
    seed_fixture(pool.as_ref()).await;

    assert!(
        find_sales_person_by_user_id(&pool, U_ALLEIN)
            .await
            .is_none(),
        "SC1 'unverknuepft darf nicht': U_ALLEIN hat keine Sales Person"
    );
}

/// G-4 (D-61-08 / T-61-01) — der Kernnachweis dieses Plans.
///
/// Sales Person und Benutzer tragen **beide** `person_id = NULL`. Eine
/// Rust-seitige `Option<Uuid>`-Gleichheit wuerde hier `None == None` als `true`
/// werten und dem Benutzer eine fremde Sales Person aushaendigen (fail-open).
/// Der SQL-Join liefert `None`, weil `NULL = NULL` in SQLs Drei-Werte-Logik nie
/// wahr wird (fail-closed).
#[tokio::test]
async fn null_person_id_does_not_match_null_person_id() {
    let pool = setup_pool().await;
    let fixture = seed_fixture(pool.as_ref()).await;

    let found = find_sales_person_by_user_id(&pool, U_OHNE_PERSON).await;

    assert!(
        found.is_none(),
        "D-61-08: ein Benutzer ohne Person darf keine Sales Person ohne Person \
         aufloesen (NULL = NULL matcht nie) — bekommen wurde {:?}",
        found.map(|entity| entity.id)
    );
    assert_ne!(
        Uuid::nil(),
        fixture.s_ohne_person,
        "Die Fixture muss eine Sales Person ohne Person enthalten, sonst ist der Test blind"
    );
}

/// G-5: Eine Verknuepfung, die ausschliesslich ueber `person_id` besteht, loest
/// auf. Vor dem Rewrite ist dieser Test rot (die Fixture traegt keine Zeile im
/// abgeloesten Datenmodell), danach gruen.
#[tokio::test]
async fn person_id_link_without_legacy_row_resolves() {
    let pool = setup_pool().await;
    let fixture = seed_fixture(pool.as_ref()).await;

    let found = find_sales_person_by_user_id(&pool, U_NUR_PERSON_ID)
        .await
        .expect("Eine rein ueber person_id bestehende Verknuepfung muss aufloesen");

    assert_eq!(found.id, fixture.s_nur_person_id);
}

/// G-7: Beide Lesepfade sind deckungsgleich. Die Duplikation ist Bestand; der
/// Test friert sie ein, damit ein spaeterer Rewrite nicht nur eine Haelfte
/// erwischt.
#[tokio::test]
async fn find_by_user_und_find_sales_person_by_user_id_sind_deckungsgleich() {
    let pool = setup_pool().await;
    seed_fixture(pool.as_ref()).await;

    for user in [U_SELF, U_FREMD, U_ALLEIN, U_NUR_PERSON_ID, U_OHNE_PERSON] {
        let via_find_by_user = find_by_user(&pool, user).await.map(|entity| entity.id);
        let via_self_gate = find_sales_person_by_user_id(&pool, user)
            .await
            .map(|entity| entity.id);

        assert_eq!(
            via_find_by_user, via_self_gate,
            "find_by_user und find_sales_person_by_user_id muessen fuer '{user}' dasselbe liefern"
        );
    }
}

// ── LEGACY-VERGLEICH START (Plan 61-05 entfernt diesen Block) ──
//
// D-61-09: Die Vorher/Nachher-Permission-Matrix. Diese Region zitiert
// zwangslaeufig das abgeloeste Datenmodell, um die „Vorher"-Haelfte ueberhaupt
// herstellen zu koennen — sie kann deshalb nur existieren, solange dieses noch
// da ist. Plan 61-05 loescht den gesamten Block zusammen mit der Migration
// `20260917000000_drop-sales-person-user.sql`; ausserhalb dieser Marker kommt
// der alte Tabellenname in dieser Datei nirgends vor.
//
// Bewusst rohes `sqlx::query` statt der `query!`-Makros: die Alt-Query darf
// nicht in den `.sqlx`-Offline-Cache wandern und nicht zur Compile-Zeit gegen
// das Schema gebunden werden, sonst bricht sie den Build in dem Moment, in dem
// 61-05 die Tabelle entfernt — statt sauber mit diesem Block zu verschwinden.

/// Ergaenzt die Fixture um die „Vorher"-Haelfte: dieselben drei Paarungen, nun
/// zusaetzlich im abgeloesten Datenmodell. Damit ist **eine** Fixture-DB
/// dual-befuellt und Alt- wie Neu-Query koennen auf ihr gegeneinander laufen.
async fn seed_legacy_links(pool: &sqlx::SqlitePool, fixture: &Fixture) {
    for (sales_person_id, user_id) in [
        (fixture.s_self, U_SELF),
        (fixture.s_fremd, U_FREMD),
        (fixture.s_nur_person_id, U_NUR_PERSON_ID),
    ] {
        sqlx::query(
            "INSERT INTO sales_person_user (sales_person_id, user_id, update_process) \
             VALUES (?1, ?2, 'seed-legacy')",
        )
        .bind(sales_person_id.as_bytes().to_vec())
        .bind(user_id)
        .execute(pool)
        .await
        .expect("seed legacy link");
    }
}

/// Die Alt-Fassung von `find_sales_person_by_user_id` im Wortlaut vor dem
/// Rewrite (nur auf `sp.id` reduziert — verglichen wird die Entscheidung und
/// die Sales-Person-ID).
async fn alt_find_sales_person_id_by_user_id(pool: &sqlx::SqlitePool, user_id: &str) -> Option<Uuid> {
    sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT sp.id FROM sales_person sp \
         JOIN sales_person_user spu ON sp.id = spu.sales_person_id \
         WHERE spu.user_id = ?1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .expect("alt-query find_sales_person_by_user_id")
    .map(|id| Uuid::from_slice(&id).expect("sales_person.id ist ein UUID"))
}

/// G-6 (D-61-09): Vorher/Nachher-Permission-Matrix als **ein** Test gegen
/// **eine** dual-befuellte Fixture-DB. Ohne diesen Vergleich liesse sich eine
/// Verhaltensaenderung nachtraeglich nicht mehr von einem Datenfehler
/// unterscheiden (T-61-04).
#[tokio::test]
async fn byte_identity_old_vs_new_query() {
    let pool = setup_pool().await;
    let fixture = seed_fixture(pool.as_ref()).await;
    seed_legacy_links(pool.as_ref(), &fixture).await;

    for user in [U_SELF, U_FREMD, U_ALLEIN] {
        let alt = alt_find_sales_person_id_by_user_id(pool.as_ref(), user).await;
        let neu = find_sales_person_by_user_id(&pool, user)
            .await
            .map(|entity| entity.id);

        assert_eq!(
            alt.is_some(),
            neu.is_some(),
            "D-61-09: Alt und Neu muessen fuer '{user}' dieselbe Accept/Reject-Entscheidung treffen \
             (alt={alt:?}, neu={neu:?})"
        );
        assert_eq!(
            alt, neu,
            "D-61-09: Alt und Neu muessen fuer '{user}' dieselbe Sales-Person-ID liefern"
        );
    }

}
// ── LEGACY-VERGLEICH ENDE ──
