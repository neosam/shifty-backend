//! Tests für den Nachzug der Buchungs-Urheberschaft
//! (`migrations/sqlite/20261108000000_backfill-booking-authorship.sql`).
//!
//! Die Tests fahren gegen eine echte SQLite-in-memory-DB. Aufbau wie in
//! `person_link_migration_test.rs`: die Migrationskette läuft **bis
//! ausschließlich** zur Nachzugs-Version, dann werden Zeilen geseedet, dann wird
//! genau der markierte BACKFILL-Abschnitt ausgeführt. So lässt sich der
//! Vorzustand herstellen, den der Nachzug im Feld antrifft.
//!
//! Der inhaltliche Schwerpunkt liegt auf den Fällen, in denen **nicht**
//! geschrieben werden darf. Ein Nachzug, der zu viel ersetzt, schreibt falsche
//! Urheberschaft in den Audit-Trail — das ist schlimmer als eine fehlende
//! `*`-Markierung.

use std::sync::Arc;

use sqlx::Row;
use uuid::Uuid;

const MIGRATION_SQL: &str =
    include_str!("../../migrations/sqlite/20261108000000_backfill-booking-authorship.sql");

const BACKFILL_START: &str = "-- BACKFILL START";
const BACKFILL_END: &str = "-- BACKFILL END";

/// Version der Nachzugs-Migration. Wer die Datei umbenennt, muss das hier
/// mitziehen.
const BACKFILL_MIGRATION_VERSION: i64 = 20261108000000;

const BACKFILL_PROCESS: &str = "backfill-booking-authorship";

/// Schneidet den markierten Backfill-Abschnitt aus der Migrationsdatei. Damit
/// lässt er sich gezielt ein zweites Mal fahren (Idempotenz-Nachweis).
fn backfill_block() -> &'static str {
    let start = MIGRATION_SQL
        .find(BACKFILL_START)
        .expect("BACKFILL START Marker fehlt in der Migrationsdatei");
    let end = MIGRATION_SQL
        .find(BACKFILL_END)
        .expect("BACKFILL END Marker fehlt in der Migrationsdatei");
    &MIGRATION_SQL[start..end]
}

/// Migrationskette **vor** dem Nachzug.
async fn setup_pool_before_backfill() -> Arc<sqlx::SqlitePool> {
    let pool = Arc::new(
        sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Could not connect to in-memory SQLite"),
    );
    let all = sqlx::migrate!("./../migrations/sqlite");
    let upto: Vec<sqlx::migrate::Migration> = all
        .iter()
        .filter(|migration| migration.version < BACKFILL_MIGRATION_VERSION)
        .cloned()
        .collect();
    let migrator = sqlx::migrate::Migrator {
        migrations: std::borrow::Cow::Owned(upto),
        ..sqlx::migrate::Migrator::DEFAULT
    };
    migrator
        .run(pool.as_ref())
        .await
        .expect("Could not run migration chain before the backfill");
    pool
}

async fn run_backfill(pool: &sqlx::SqlitePool) {
    sqlx::raw_sql(backfill_block())
        .execute(pool)
        .await
        .expect("Backfill-Abschnitt muss ausfuehrbar sein");
}

/// Legt einen Slot an. `day_of_week` bestimmt die Gruppierung des Nachzugs.
async fn seed_slot(pool: &sqlx::SqlitePool, id: Uuid, day_of_week: i64) {
    sqlx::query(
        "INSERT INTO slot (id, day_of_week, time_from, time_to, valid_from, valid_to, \
                           deleted, min_resources, update_process, update_version) \
         VALUES (?1, ?2, '08:00:00', '12:00:00', '2026-01-01', NULL, NULL, 1, 'seed', ?3)",
    )
    .bind(id.as_bytes().to_vec())
    .bind(day_of_week)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(pool)
    .await
    .expect("seed slot");
}

#[allow(clippy::too_many_arguments)]
async fn seed_booking(
    pool: &sqlx::SqlitePool,
    id: Uuid,
    sales_person_id: Uuid,
    slot_id: Uuid,
    created: &str,
    deleted: Option<&str>,
    created_by: Option<&str>,
    deleted_by: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO booking (id, sales_person_id, slot_id, calendar_week, year, \
                              created, deleted, created_by, deleted_by, \
                              update_process, update_version) \
         VALUES (?1, ?2, ?3, 20, 2026, ?4, ?5, ?6, ?7, 'seed', ?8)",
    )
    .bind(id.as_bytes().to_vec())
    .bind(sales_person_id.as_bytes().to_vec())
    .bind(slot_id.as_bytes().to_vec())
    .bind(created)
    .bind(deleted)
    .bind(created_by)
    .bind(deleted_by)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(pool)
    .await
    .expect("seed booking");
}

async fn created_by_of(pool: &sqlx::SqlitePool, id: Uuid) -> Option<String> {
    sqlx::query("SELECT created_by FROM booking WHERE id = ?1")
        .bind(id.as_bytes().to_vec())
        .fetch_one(pool)
        .await
        .expect("read booking")
        .try_get::<Option<String>, _>("created_by")
        .expect("decode created_by")
}

async fn update_process_of(pool: &sqlx::SqlitePool, id: Uuid) -> String {
    sqlx::query("SELECT update_process FROM booking WHERE id = ?1")
        .bind(id.as_bytes().to_vec())
        .fetch_one(pool)
        .await
        .expect("read booking")
        .try_get::<String, _>("update_process")
        .expect("decode update_process")
}

fn sp() -> Uuid {
    Uuid::from_u128(0x00AA)
}

/// Kernfall: ein Umbau. Die lebende Zeile trägt 'system', ihre system-gelöschte
/// Vorgängerin den echten Namen. Genau dieser Fall ist die verschwundene
/// `*`-Markierung.
#[tokio::test]
async fn nachzug_stellt_urheber_nach_einem_umbau_wieder_her() {
    let pool = setup_pool_before_backfill().await;
    let old_slot = Uuid::from_u128(0x5101);
    let new_slot = Uuid::from_u128(0x5102);
    seed_slot(&pool, old_slot, 1).await;
    seed_slot(&pool, new_slot, 1).await;

    let predecessor = Uuid::from_u128(0xB001);
    let live = Uuid::from_u128(0xB002);
    seed_booking(
        &pool, predecessor, sp(), old_slot,
        "2026-05-01T10:00:00.000000001", Some("2026-05-02T09:00:00.000000002"),
        Some("alice"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, live, sp(), new_slot,
        "2026-05-02T09:00:00.000000003", None, Some("system"), None,
    )
    .await;

    run_backfill(&pool).await;

    assert_eq!(
        created_by_of(&pool, live).await.as_deref(),
        Some("alice"),
        "die lebende Buchung muss den Urheber ihrer Vorgaengerin erhalten"
    );
    assert_eq!(
        update_process_of(&pool, live).await,
        BACKFILL_PROCESS,
        "die Aenderung muss im Audit-Trail als Nachzug erkennbar sein"
    );
    assert_eq!(
        created_by_of(&pool, predecessor).await.as_deref(),
        Some("alice"),
        "die soft-geloeschte Vorgaengerin bleibt unberuehrt"
    );
}

/// Zwei Umbauten hintereinander: die Kette muss über das mittlere
/// 'system'-Glied hinweg bis zum echten Namen laufen.
#[tokio::test]
async fn nachzug_laeuft_ueber_mehrere_umbauten_zurueck() {
    let pool = setup_pool_before_backfill().await;
    let slot = Uuid::from_u128(0x5201);
    seed_slot(&pool, slot, 2).await;

    let first = Uuid::from_u128(0xB101);
    let middle = Uuid::from_u128(0xB102);
    let live = Uuid::from_u128(0xB103);
    seed_booking(&pool, first, sp(), slot, "2026-05-01T10:00:00.000000001",
        Some("2026-05-02T09:00:00.000000002"), Some("alice"), Some("system")).await;
    seed_booking(&pool, middle, sp(), slot, "2026-05-02T09:00:00.000000003",
        Some("2026-05-03T08:00:00.000000004"), Some("system"), Some("system")).await;
    seed_booking(&pool, live, sp(), slot, "2026-05-03T08:00:00.000000005",
        None, Some("system"), None).await;

    run_backfill(&pool).await;

    assert_eq!(
        created_by_of(&pool, live).await.as_deref(),
        Some("alice"),
        "der Nachzug muss ueber das mittlere 'system'-Glied hinweg laufen"
    );
}

/// Hat ein Mensch gelöscht, sind es zwei getrennte Episoden. Der frühere Name
/// darf NICHT übernommen werden — sonst schreibt der Nachzug eine Urheberschaft
/// an eine Buchung, die gar nicht aus jener Kette stammt.
#[tokio::test]
async fn nachzug_laeuft_nicht_ueber_eine_menschen_loeschung() {
    let pool = setup_pool_before_backfill().await;
    let slot = Uuid::from_u128(0x5301);
    seed_slot(&pool, slot, 3).await;

    let deleted_by_human = Uuid::from_u128(0xB201);
    let live = Uuid::from_u128(0xB202);
    seed_booking(&pool, deleted_by_human, sp(), slot, "2026-05-01T10:00:00.000000001",
        Some("2026-05-02T09:00:00.000000002"), Some("alice"), Some("bob")).await;
    seed_booking(&pool, live, sp(), slot, "2026-05-02T09:00:00.000000003",
        None, Some("system"), None).await;

    run_backfill(&pool).await;

    assert_eq!(
        created_by_of(&pool, live).await.as_deref(),
        Some("system"),
        "ueber eine Menschen-Loeschung darf nicht attribuiert werden"
    );
}

/// Eine lebende 'system'-Buchung ohne jede Vorgängerin ist keine Umbau-Folge,
/// sondern eine echte System-Buchung — der Auto-Rebooking-Cron bucht so. Sie
/// muss unangetastet bleiben.
#[tokio::test]
async fn nachzug_laesst_echte_system_buchung_unberuehrt() {
    let pool = setup_pool_before_backfill().await;
    let slot = Uuid::from_u128(0x5401);
    seed_slot(&pool, slot, 4).await;

    let live = Uuid::from_u128(0xB301);
    seed_booking(&pool, live, sp(), slot, "2026-05-01T10:00:00.000000001",
        None, Some("system"), None).await;

    run_backfill(&pool).await;

    assert_eq!(
        created_by_of(&pool, live).await.as_deref(),
        Some("system"),
        "eine System-Buchung ohne Vorgaengerkette ist kein Umbau-Schaden"
    );
    assert_eq!(
        update_process_of(&pool, live).await,
        "seed",
        "unberuehrte Zeilen behalten ihren update_process"
    );
}

/// Altbestand vor der Audit-Migration: die Vorgängerin trägt kein `deleted_by`.
/// Ob Umbau oder Mensch ist nicht entscheidbar — nicht laufen.
#[tokio::test]
async fn nachzug_stoppt_wenn_vorgaengerin_kein_deleted_by_hat() {
    let pool = setup_pool_before_backfill().await;
    let slot = Uuid::from_u128(0x5501);
    seed_slot(&pool, slot, 5).await;

    let predecessor = Uuid::from_u128(0xB401);
    let live = Uuid::from_u128(0xB402);
    seed_booking(&pool, predecessor, sp(), slot, "2026-05-01T10:00:00.000000001",
        Some("2026-05-02T09:00:00.000000002"), Some("alice"), None).await;
    seed_booking(&pool, live, sp(), slot, "2026-05-02T09:00:00.000000003",
        None, Some("system"), None).await;

    run_backfill(&pool).await;

    assert_eq!(
        created_by_of(&pool, live).await.as_deref(),
        Some("system"),
        "ohne deleted_by ist die Kette nicht belastbar"
    );
}

/// Zwei Vorgängerinnen mit identischem `created` und verschiedenen Urhebern:
/// die Paarung ist nicht entscheidbar und es wird nichts geschrieben.
#[tokio::test]
async fn nachzug_raet_nicht_bei_mehrdeutigkeit() {
    let pool = setup_pool_before_backfill().await;
    let slot = Uuid::from_u128(0x5601);
    seed_slot(&pool, slot, 6).await;

    let a = Uuid::from_u128(0xB501);
    let b = Uuid::from_u128(0xB502);
    let live = Uuid::from_u128(0xB503);
    seed_booking(&pool, a, sp(), slot, "2026-05-01T10:00:00.000000001",
        Some("2026-05-02T09:00:00.000000002"), Some("alice"), Some("system")).await;
    seed_booking(&pool, b, sp(), slot, "2026-05-01T10:00:00.000000001",
        Some("2026-05-02T09:00:00.000000002"), Some("carol"), Some("system")).await;
    seed_booking(&pool, live, sp(), slot, "2026-05-02T09:00:00.000000003",
        None, Some("system"), None).await;

    run_backfill(&pool).await;

    assert_eq!(
        created_by_of(&pool, live).await.as_deref(),
        Some("system"),
        "bei zwei verschiedenen Kandidaten darf nicht geraten werden"
    );
}

/// Zwei Buchungen derselben Person in derselben Woche an VERSCHIEDENEN
/// Wochentagen dürfen sich nicht vermischen — dafür ist der `day_of_week` in der
/// Gruppierung. Ohne ihn bekäme die Montags-Buchung womöglich den Urheber der
/// Dienstags-Kette.
#[tokio::test]
async fn nachzug_haelt_wochentage_auseinander() {
    let pool = setup_pool_before_backfill().await;
    let monday_old = Uuid::from_u128(0x5701);
    let monday_new = Uuid::from_u128(0x5702);
    let tuesday_old = Uuid::from_u128(0x5703);
    let tuesday_new = Uuid::from_u128(0x5704);
    seed_slot(&pool, monday_old, 1).await;
    seed_slot(&pool, monday_new, 1).await;
    seed_slot(&pool, tuesday_old, 2).await;
    seed_slot(&pool, tuesday_new, 2).await;

    let mon_pred = Uuid::from_u128(0xB601);
    let mon_live = Uuid::from_u128(0xB602);
    let tue_pred = Uuid::from_u128(0xB603);
    let tue_live = Uuid::from_u128(0xB604);

    seed_booking(&pool, mon_pred, sp(), monday_old, "2026-05-01T10:00:00.000000001",
        Some("2026-05-02T09:00:00.000000002"), Some("alice"), Some("system")).await;
    seed_booking(&pool, mon_live, sp(), monday_new, "2026-05-02T09:00:00.000000003",
        None, Some("system"), None).await;
    // Dienstags-Kette liegt ZEITLICH DAZWISCHEN — ohne Wochentag-Gruppierung
    // waere sie der naechste Vorgaenger der Montags-Zeile.
    seed_booking(&pool, tue_pred, sp(), tuesday_old, "2026-05-02T09:00:00.000000002",
        Some("2026-05-02T09:00:00.000000004"), Some("dave"), Some("system")).await;
    seed_booking(&pool, tue_live, sp(), tuesday_new, "2026-05-02T09:00:00.000000005",
        None, Some("system"), None).await;

    run_backfill(&pool).await;

    assert_eq!(
        created_by_of(&pool, mon_live).await.as_deref(),
        Some("alice"),
        "Montag muss seinen eigenen Urheber bekommen"
    );
    assert_eq!(
        created_by_of(&pool, tue_live).await.as_deref(),
        Some("dave"),
        "Dienstag muss seinen eigenen Urheber bekommen, nicht den von Montag"
    );
}

/// Ein zweiter Durchlauf darf nichts mehr tun. Die Bedingung
/// `created_by = 'system'` ist die zweite Verteidigungslinie neben der
/// Einmal-Ausfuehrung durch sqlx.
#[tokio::test]
async fn nachzug_ist_beim_zweiten_lauf_ein_no_op() {
    let pool = setup_pool_before_backfill().await;
    let slot = Uuid::from_u128(0x5801);
    seed_slot(&pool, slot, 1).await;

    let predecessor = Uuid::from_u128(0xB701);
    let live = Uuid::from_u128(0xB702);
    seed_booking(&pool, predecessor, sp(), slot, "2026-05-01T10:00:00.000000001",
        Some("2026-05-02T09:00:00.000000002"), Some("alice"), Some("system")).await;
    seed_booking(&pool, live, sp(), slot, "2026-05-02T09:00:00.000000003",
        None, Some("system"), None).await;

    run_backfill(&pool).await;
    let after_first = created_by_of(&pool, live).await;
    run_backfill(&pool).await;
    let after_second = created_by_of(&pool, live).await;

    assert_eq!(after_first.as_deref(), Some("alice"));
    assert_eq!(
        after_second, after_first,
        "der zweite Lauf darf den Wert nicht veraendern"
    );
}

/// Eine lebende Buchung mit echtem Urheber wird nicht angefasst — der Nachzug
/// darf bestehende Urheberschaft nicht ueberschreiben.
#[tokio::test]
async fn nachzug_laesst_echte_urheber_unberuehrt() {
    let pool = setup_pool_before_backfill().await;
    let slot = Uuid::from_u128(0x5901);
    seed_slot(&pool, slot, 1).await;

    let predecessor = Uuid::from_u128(0xB801);
    let live = Uuid::from_u128(0xB802);
    seed_booking(&pool, predecessor, sp(), slot, "2026-05-01T10:00:00.000000001",
        Some("2026-05-02T09:00:00.000000002"), Some("alice"), Some("system")).await;
    seed_booking(&pool, live, sp(), slot, "2026-05-02T09:00:00.000000003",
        None, Some("erika"), None).await;

    run_backfill(&pool).await;

    assert_eq!(
        created_by_of(&pool, live).await.as_deref(),
        Some("erika"),
        "ein vorhandener echter Urheber darf nicht ersetzt werden"
    );
}

/// Ohne betroffene Zeilen ist der Nachzug ein No-op und laeuft sauber durch —
/// der Normalfall auf einer frischen Datenbank.
#[tokio::test]
async fn nachzug_ist_no_op_ohne_betroffene_zeilen() {
    let pool = setup_pool_before_backfill().await;
    run_backfill(&pool).await;

    let count: i64 = sqlx::query("SELECT COUNT(*) AS c FROM booking")
        .fetch_one(pool.as_ref())
        .await
        .expect("count bookings")
        .try_get("c")
        .expect("decode count");
    assert_eq!(count, 0);
}
