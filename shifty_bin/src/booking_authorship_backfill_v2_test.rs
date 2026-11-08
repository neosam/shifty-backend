//! Tests für den korrigierten Nachzug der Buchungs-Urheberschaft
//! (`migrations/sqlite/20261108120000_backfill-booking-authorship-v2.sql`).
//!
//! **Warum eine zweite Migration und eine zweite Testdatei:** v1
//! (`20261108000000`) ist ausgeliefert und mit Prüfsumme in `_sqlx_migrations`
//! eingetragen — eine Änderung an der Datei lässt den Dienst beim Start
//! abbrechen. v1 bleibt deshalb unverändert, samt ihren Tests, die weiterhin
//! dokumentieren, was sie tut.
//!
//! v1 gruppierte nach `(sales_person_id, year, calendar_week, slot.day_of_week)`
//! und nahm als Vorgängerin die Zeile mit dem größten `created` darunter. Dieser
//! Schlüssel wirft **alle Slots eines Wochentags** in einen Topf. In der Folge
//! blieben Zeilen auf `'system'` stehen (die vermeintliche Vorgängerin lebt noch,
//! die Kette bricht sofort ab) und einzelne Zeilen bekamen den Urheber einer
//! **anderen** Schicht desselben Wochentags.
//!
//! v2 verbindet stattdessen über die zeitliche Nachbarschaft:
//! `modify_slot` ruft pro Buchung `delete()` und direkt danach `create()` auf,
//! die Stempel liegen Millisekunden auseinander.
//!
//! Die Reihenfolge im Harness entspricht dem Feld: Zeilen seeden, **dann v1**,
//! **dann v2**. Genau so trifft v2 auf einer bereits deployten Datenbank auf.
//! Ein Test fährt v2 zusätzlich allein, um zu belegen, dass sie nicht auf v1
//! angewiesen ist.

use std::sync::Arc;

use sqlx::Row;
use uuid::Uuid;

const MIGRATION_SQL_V1: &str =
    include_str!("../../migrations/sqlite/20261108000000_backfill-booking-authorship.sql");
const MIGRATION_SQL_V2: &str =
    include_str!("../../migrations/sqlite/20261108120000_backfill-booking-authorship-v2.sql");

const BACKFILL_START: &str = "-- BACKFILL START";
const BACKFILL_END: &str = "-- BACKFILL END";

/// Version der **ersten** Nachzugs-Migration. Die Kette läuft bis ausschließlich
/// hierhin, damit die Zeilen vor beiden Nachzügen geseedet werden können.
const BACKFILL_V1_MIGRATION_VERSION: i64 = 20261108000000;

const BACKFILL_PROCESS: &str = "backfill-booking-authorship";

fn backfill_block(sql: &'static str) -> &'static str {
    let start = sql
        .find(BACKFILL_START)
        .expect("BACKFILL START Marker fehlt in der Migrationsdatei");
    let end = sql
        .find(BACKFILL_END)
        .expect("BACKFILL END Marker fehlt in der Migrationsdatei");
    &sql[start..end]
}

/// Migrationskette **vor** beiden Nachzügen.
async fn setup_pool_before_backfills() -> Arc<sqlx::SqlitePool> {
    let pool = Arc::new(
        sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Could not connect to in-memory SQLite"),
    );
    let all = sqlx::migrate!("./../migrations/sqlite");
    let upto: Vec<sqlx::migrate::Migration> = all
        .iter()
        .filter(|migration| migration.version < BACKFILL_V1_MIGRATION_VERSION)
        .cloned()
        .collect();
    let migrator = sqlx::migrate::Migrator {
        migrations: std::borrow::Cow::Owned(upto),
        ..sqlx::migrate::Migrator::DEFAULT
    };
    migrator
        .run(pool.as_ref())
        .await
        .expect("Could not run migration chain before the backfills");
    pool
}

async fn run_v1(pool: &sqlx::SqlitePool) {
    sqlx::raw_sql(backfill_block(MIGRATION_SQL_V1))
        .execute(pool)
        .await
        .expect("v1-Backfill-Abschnitt muss ausfuehrbar sein");
}

async fn run_v2(pool: &sqlx::SqlitePool) {
    sqlx::raw_sql(backfill_block(MIGRATION_SQL_V2))
        .execute(pool)
        .await
        .expect("v2-Backfill-Abschnitt muss ausfuehrbar sein");
}

/// Die Reihenfolge, in der eine deployte Datenbank sie sieht.
async fn run_both(pool: &sqlx::SqlitePool) {
    run_v1(pool).await;
    run_v2(pool).await;
}

/// Legt einen Slot an. `day_of_week` ist beim Umbau stabil, die Zeiten nicht —
/// deshalb trägt der Slot hier eine Zeit, die der Nachzug bewusst ignoriert.
async fn seed_slot(pool: &sqlx::SqlitePool, id: Uuid, day_of_week: i64, time_from: &str) {
    sqlx::query(
        "INSERT INTO slot (id, day_of_week, time_from, time_to, valid_from, valid_to, \
                           deleted, min_resources, update_process, update_version) \
         VALUES (?1, ?2, ?3, '23:59:00', '2026-01-01', NULL, NULL, 1, 'seed', ?4)",
    )
    .bind(id.as_bytes().to_vec())
    .bind(day_of_week)
    .bind(time_from)
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

async fn update_version_of(pool: &sqlx::SqlitePool, id: Uuid) -> Vec<u8> {
    sqlx::query("SELECT update_version FROM booking WHERE id = ?1")
        .bind(id.as_bytes().to_vec())
        .fetch_one(pool)
        .await
        .expect("read booking")
        .try_get::<Vec<u8>, _>("update_version")
        .expect("decode update_version")
}

fn sp() -> Uuid {
    Uuid::from_u128(0x00AA)
}

// ---------------------------------------------------------------------------
// Der gemeldete Fall
// ---------------------------------------------------------------------------

/// Der auf der Integrationsumgebung beobachtete Freitag, Zeile für Zeile mit den
/// echten Zeitstempeln. Eine Person hat sich an einem Freitag vier Schichten
/// selbst eingetragen; drei davon wurden umgebaut.
///
/// Unter v1 blieben die 18:00- und die 19:00-Zeile auf `'system'` stehen: ihre
/// vermeintliche Vorgängerin nach Gruppenschlüssel war die LEBENDE 17:00-Zeile
/// von 07:17:18, deren `deleted_by` NULL ist — die Kette brach sofort ab.
#[tokio::test]
async fn v2_repariert_den_gemeldeten_freitag_vollstaendig() {
    let pool = setup_pool_before_backfills().await;

    // Vier Slots, alle am selben Wochentag, verschiedene Zeiten.
    let slot_16 = Uuid::from_u128(0x5116);
    let slot_17_alt = Uuid::from_u128(0x5117);
    let slot_18_alt = Uuid::from_u128(0x5118);
    let slot_19_alt = Uuid::from_u128(0x5119);
    let slot_17_neu = Uuid::from_u128(0x5217);
    let slot_18_neu = Uuid::from_u128(0x5218);
    let slot_19_neu = Uuid::from_u128(0x5219);
    for (id, time_from) in [
        (slot_16, "16:00:00"),
        (slot_17_alt, "17:00:00"),
        (slot_18_alt, "18:00:00"),
        (slot_19_alt, "19:00:00"),
        (slot_17_neu, "17:00:00"),
        (slot_18_neu, "18:00:00"),
        (slot_19_neu, "19:00:00"),
    ] {
        seed_slot(&pool, id, 5, time_from).await;
    }

    let b16 = Uuid::from_u128(0xB016);
    let b17_alt = Uuid::from_u128(0xB117);
    let b18_alt = Uuid::from_u128(0xB118);
    let b19_alt = Uuid::from_u128(0xB119);
    let b17_neu = Uuid::from_u128(0xB217);
    let b18_neu = Uuid::from_u128(0xB218);
    let b19_neu = Uuid::from_u128(0xB219);

    // 16:00 wurde nie umgebaut.
    seed_booking(
        &pool, b16, sp(), slot_16,
        "2026-10-02T12:36:16.201727600", None, Some("jakob.l"), None,
    )
    .await;
    // Die drei umgebauten Paare, Abstand delete -> create jeweils wenige ms.
    seed_booking(
        &pool, b17_alt, sp(), slot_17_alt,
        "2026-10-02T12:36:16.902174899", Some("2026-11-08T07:17:18.212075459"),
        Some("jakob.l"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b17_neu, sp(), slot_17_neu,
        "2026-11-08T07:17:18.220225594", None, Some("system"), None,
    )
    .await;
    seed_booking(
        &pool, b18_alt, sp(), slot_18_alt,
        "2026-10-02T12:36:17.652195120", Some("2026-11-08T07:17:24.097474460"),
        Some("jakob.l"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b18_neu, sp(), slot_18_neu,
        "2026-11-08T07:17:24.104249595", None, Some("system"), None,
    )
    .await;
    seed_booking(
        &pool, b19_alt, sp(), slot_19_alt,
        "2026-10-02T12:36:18.245657244", Some("2026-11-08T07:17:27.752902705"),
        Some("jakob.l"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b19_neu, sp(), slot_19_neu,
        "2026-11-08T07:17:27.757124631", None, Some("system"), None,
    )
    .await;

    run_both(&pool).await;

    for (id, zeit) in [(b17_neu, "17:00"), (b18_neu, "18:00"), (b19_neu, "19:00")] {
        assert_eq!(
            created_by_of(&pool, id).await.as_deref(),
            Some("jakob.l"),
            "die umgebaute {zeit}-Buchung muss ihren Urheber zurueckbekommen"
        );
    }
    assert_eq!(
        created_by_of(&pool, b16).await.as_deref(),
        Some("jakob.l"),
        "die nie umgebaute Buchung bleibt wie sie ist"
    );
    assert_eq!(
        update_process_of(&pool, b16).await,
        "seed",
        "und sie darf nicht angefasst worden sein"
    );
}

/// Der schlimmere Fall: v1 schreibt einen **falschen** Namen. Eine Person hat an
/// einem Wochentag zwei Schichten — eine hat die planende Person eingetragen, die
/// andere die Person selbst. v1 ordnet der 17:00-Zeile den Urheber der
/// 18:00-Zeile zu, weil beide im selben Gruppen-Topf liegen.
#[tokio::test]
async fn v2_korrigiert_eine_falschzuordnung_von_v1() {
    let pool = setup_pool_before_backfills().await;
    let slot_17_alt = Uuid::from_u128(0x6117);
    let slot_18_alt = Uuid::from_u128(0x6118);
    let slot_17_neu = Uuid::from_u128(0x6217);
    let slot_18_neu = Uuid::from_u128(0x6218);
    for (id, time_from) in [
        (slot_17_alt, "17:00:00"),
        (slot_18_alt, "18:00:00"),
        (slot_17_neu, "17:00:00"),
        (slot_18_neu, "18:00:00"),
    ] {
        seed_slot(&pool, id, 4, time_from).await;
    }

    let b17_alt = Uuid::from_u128(0xC117);
    let b18_alt = Uuid::from_u128(0xC118);
    let b17_neu = Uuid::from_u128(0xC217);
    let b18_neu = Uuid::from_u128(0xC218);

    // 17:00 hat die Planerin eingetragen, 18:00 die Person selbst.
    seed_booking(
        &pool, b17_alt, sp(), slot_17_alt,
        "2026-10-02T12:00:00.000000000", Some("2026-11-08T07:00:00.100000000"),
        Some("dina.b"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b18_alt, sp(), slot_18_alt,
        "2026-10-02T12:00:01.000000000", Some("2026-11-08T07:00:00.200000000"),
        Some("jakob.l"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b17_neu, sp(), slot_17_neu,
        "2026-11-08T07:00:00.108000000", None, Some("system"), None,
    )
    .await;
    seed_booking(
        &pool, b18_neu, sp(), slot_18_neu,
        "2026-11-08T07:00:00.208000000", None, Some("system"), None,
    )
    .await;

    // Erst v1 allein -- das belegt die Falschzuordnung, die v2 aufraeumen muss.
    run_v1(&pool).await;
    assert_eq!(
        created_by_of(&pool, b17_neu).await.as_deref(),
        Some("jakob.l"),
        "Vorbedingung: v1 ordnet hier nachweislich falsch zu. Stimmt das nicht \
         mehr, ist dieser Test gegenstandslos und muss neu gedacht werden"
    );

    run_v2(&pool).await;

    assert_eq!(
        created_by_of(&pool, b17_neu).await.as_deref(),
        Some("dina.b"),
        "v2 muss die Falschzuordnung von v1 korrigieren"
    );
    assert_eq!(
        created_by_of(&pool, b18_neu).await.as_deref(),
        Some("jakob.l"),
        "und die von v1 uebersprungene Zeile nachziehen"
    );
}

/// Der Kern des Defekts, in kleinster Form: zwei Slots desselben Wochentags,
/// beide umgebaut. Unter v1 bleibt der zweite auf `'system'`, weil seine
/// vermeintliche Vorgängerin die lebende erste Zeile ist.
#[tokio::test]
async fn v2_haelt_slots_desselben_wochentags_auseinander() {
    let pool = setup_pool_before_backfills().await;
    let a_alt = Uuid::from_u128(0x7101);
    let b_alt = Uuid::from_u128(0x7102);
    let a_neu = Uuid::from_u128(0x7201);
    let b_neu = Uuid::from_u128(0x7202);
    for (id, time_from) in [
        (a_alt, "09:00:00"),
        (b_alt, "13:00:00"),
        (a_neu, "09:00:00"),
        (b_neu, "13:00:00"),
    ] {
        seed_slot(&pool, id, 2, time_from).await;
    }

    let ba_alt = Uuid::from_u128(0xDA01);
    let bb_alt = Uuid::from_u128(0xDB01);
    let ba_neu = Uuid::from_u128(0xDA02);
    let bb_neu = Uuid::from_u128(0xDB02);
    seed_booking(
        &pool, ba_alt, sp(), a_alt,
        "2026-05-01T08:00:00.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("alice"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, ba_neu, sp(), a_neu,
        "2026-06-01T10:00:00.004000000", None, Some("system"), None,
    )
    .await;
    seed_booking(
        &pool, bb_alt, sp(), b_alt,
        "2026-05-01T08:00:01.000000000", Some("2026-06-01T10:00:00.010000000"),
        Some("alice"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, bb_neu, sp(), b_neu,
        "2026-06-01T10:00:00.014000000", None, Some("system"), None,
    )
    .await;

    run_both(&pool).await;

    assert_eq!(
        created_by_of(&pool, ba_neu).await.as_deref(),
        Some("alice"),
        "erster umgebauter Slot"
    );
    assert_eq!(
        created_by_of(&pool, bb_neu).await.as_deref(),
        Some("alice"),
        "zweiter umgebauter Slot desselben Wochentags -- genau hier versagte v1"
    );
}

// ---------------------------------------------------------------------------
// Faelle, in denen NICHT geschrieben werden darf
// ---------------------------------------------------------------------------

/// Das Zeitfenster ist die neue Abbruchbedingung. Liegt zwischen Löschung und
/// Neuanlage mehr als eine Sekunde, sind es zwei getrennte Vorgänge und kein
/// Umbau — es wird nicht gepaart.
#[tokio::test]
async fn v2_paart_nicht_ueber_das_zeitfenster_hinaus() {
    let pool = setup_pool_before_backfills().await;
    let alt = Uuid::from_u128(0x8101);
    let neu = Uuid::from_u128(0x8102);
    seed_slot(&pool, alt, 3, "09:00:00").await;
    seed_slot(&pool, neu, 3, "09:00:00").await;

    let b_alt = Uuid::from_u128(0xE001);
    let b_neu = Uuid::from_u128(0xE002);
    seed_booking(
        &pool, b_alt, sp(), alt,
        "2026-05-01T08:00:00.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("alice"), Some("system"),
    )
    .await;
    // Fuenf Sekunden spaeter -- kein Umbau.
    seed_booking(
        &pool, b_neu, sp(), neu,
        "2026-06-01T10:00:05.000000000", None, Some("system"), None,
    )
    .await;

    run_v2(&pool).await;

    assert_eq!(
        created_by_of(&pool, b_neu).await.as_deref(),
        Some("system"),
        "ausserhalb des Zeitfensters darf nicht gepaart werden"
    );
}

/// Eine eigenständige System-Buchung — der Auto-Rebooking-Cron
/// (`voluntary_rebooking_scheduler`) bucht legitim als `'system'`. Ohne
/// system-gelöschte Vorgängerin im Zeitfenster bleibt sie unberührt.
#[tokio::test]
async fn v2_laesst_echte_system_buchung_unberuehrt() {
    let pool = setup_pool_before_backfills().await;
    let slot = Uuid::from_u128(0x8201);
    seed_slot(&pool, slot, 3, "09:00:00").await;

    let live = Uuid::from_u128(0xE101);
    seed_booking(
        &pool, live, sp(), slot,
        "2026-06-01T03:00:00.000000000", None, Some("system"), None,
    )
    .await;

    run_both(&pool).await;

    assert_eq!(
        created_by_of(&pool, live).await.as_deref(),
        Some("system"),
        "eine eigenstaendige System-Buchung hat keine Urheberschaft zum Wiederherstellen"
    );
    assert_eq!(
        update_process_of(&pool, live).await,
        "seed",
        "und darf nicht angefasst worden sein"
    );
}

/// Hat ein Mensch gelöscht, sind es zwei getrennte Episoden. Die alte
/// Urheberschaft gehört nicht an die neue Zeile.
#[tokio::test]
async fn v2_laeuft_nicht_ueber_eine_menschen_loeschung() {
    let pool = setup_pool_before_backfills().await;
    let alt = Uuid::from_u128(0x8301);
    let neu = Uuid::from_u128(0x8302);
    seed_slot(&pool, alt, 3, "09:00:00").await;
    seed_slot(&pool, neu, 3, "09:00:00").await;

    let b_alt = Uuid::from_u128(0xE201);
    let b_neu = Uuid::from_u128(0xE202);
    seed_booking(
        &pool, b_alt, sp(), alt,
        "2026-05-01T08:00:00.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("alice"), Some("bob"),
    )
    .await;
    seed_booking(
        &pool, b_neu, sp(), neu,
        "2026-06-01T10:00:00.005000000", None, Some("system"), None,
    )
    .await;

    run_both(&pool).await;

    assert_eq!(
        created_by_of(&pool, b_neu).await.as_deref(),
        Some("system"),
        "nach einer Menschen-Loeschung darf nicht zurueckgelaufen werden"
    );
}

/// Altbestand: die Vorgängerin trägt kein `deleted_by` (Zeilen von vor
/// `20250115000000_add-user-tracking-to-booking.sql`). Ob Umbau oder Mensch ist
/// nicht entscheidbar — also wird nicht geraten.
#[tokio::test]
async fn v2_stoppt_wenn_vorgaengerin_kein_deleted_by_hat() {
    let pool = setup_pool_before_backfills().await;
    let alt = Uuid::from_u128(0x8401);
    let neu = Uuid::from_u128(0x8402);
    seed_slot(&pool, alt, 3, "09:00:00").await;
    seed_slot(&pool, neu, 3, "09:00:00").await;

    let b_alt = Uuid::from_u128(0xE301);
    let b_neu = Uuid::from_u128(0xE302);
    seed_booking(
        &pool, b_alt, sp(), alt,
        "2026-05-01T08:00:00.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("alice"), None,
    )
    .await;
    seed_booking(
        &pool, b_neu, sp(), neu,
        "2026-06-01T10:00:00.005000000", None, Some("system"), None,
    )
    .await;

    run_both(&pool).await;

    assert_eq!(
        created_by_of(&pool, b_neu).await.as_deref(),
        Some("system"),
        "ohne deleted_by ist die Kette nicht entscheidbar"
    );
}

/// Zwei Vorgängerinnen mit demselben `deleted`, aber verschiedenen Urhebern:
/// es wird nicht geraten.
#[tokio::test]
async fn v2_raet_nicht_bei_mehrdeutigkeit() {
    let pool = setup_pool_before_backfills().await;
    let alt_a = Uuid::from_u128(0x8501);
    let alt_b = Uuid::from_u128(0x8502);
    let neu = Uuid::from_u128(0x8503);
    for (id, time_from) in [(alt_a, "09:00:00"), (alt_b, "09:00:00"), (neu, "09:00:00")] {
        seed_slot(&pool, id, 3, time_from).await;
    }

    let b_alt_a = Uuid::from_u128(0xE401);
    let b_alt_b = Uuid::from_u128(0xE402);
    let b_neu = Uuid::from_u128(0xE403);
    seed_booking(
        &pool, b_alt_a, sp(), alt_a,
        "2026-05-01T08:00:00.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("alice"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b_alt_b, sp(), alt_b,
        "2026-05-01T08:00:01.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("bob"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b_neu, sp(), neu,
        "2026-06-01T10:00:00.005000000", None, Some("system"), None,
    )
    .await;

    run_both(&pool).await;

    assert_eq!(
        created_by_of(&pool, b_neu).await.as_deref(),
        Some("system"),
        "bei zwei verschiedenen Urhebern auf derselben Stufe wird nicht geraten"
    );
}

/// Wochentage bleiben getrennt, auch wenn die Zeitstempel benachbart sind.
#[tokio::test]
async fn v2_haelt_wochentage_auseinander() {
    let pool = setup_pool_before_backfills().await;
    let mo_alt = Uuid::from_u128(0x8601);
    let di_neu = Uuid::from_u128(0x8602);
    seed_slot(&pool, mo_alt, 1, "09:00:00").await;
    seed_slot(&pool, di_neu, 2, "09:00:00").await;

    let b_mo = Uuid::from_u128(0xE501);
    let b_di = Uuid::from_u128(0xE502);
    seed_booking(
        &pool, b_mo, sp(), mo_alt,
        "2026-05-01T08:00:00.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("alice"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b_di, sp(), di_neu,
        "2026-06-01T10:00:00.005000000", None, Some("system"), None,
    )
    .await;

    run_both(&pool).await;

    assert_eq!(
        created_by_of(&pool, b_di).await.as_deref(),
        Some("system"),
        "ein Montags-Umbau darf keinen Dienstag erklaeren"
    );
}

/// Eine Buchung mit echtem Urheber bleibt unangetastet.
#[tokio::test]
async fn v2_laesst_echte_urheber_unberuehrt() {
    let pool = setup_pool_before_backfills().await;
    let alt = Uuid::from_u128(0x8701);
    let neu = Uuid::from_u128(0x8702);
    seed_slot(&pool, alt, 3, "09:00:00").await;
    seed_slot(&pool, neu, 3, "09:00:00").await;

    let b_alt = Uuid::from_u128(0xE601);
    let b_neu = Uuid::from_u128(0xE602);
    seed_booking(
        &pool, b_alt, sp(), alt,
        "2026-05-01T08:00:00.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("alice"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b_neu, sp(), neu,
        "2026-06-01T10:00:00.005000000", None, Some("bob"), None,
    )
    .await;

    run_both(&pool).await;

    assert_eq!(
        created_by_of(&pool, b_neu).await.as_deref(),
        Some("bob"),
        "ein echter Urheber wird nicht ueberschrieben"
    );
    assert_eq!(
        update_process_of(&pool, b_neu).await,
        "seed",
        "und die Zeile darf nicht angefasst worden sein"
    );
}

// ---------------------------------------------------------------------------
// Mehrfach-Umbau, Idempotenz, Unabhaengigkeit von v1
// ---------------------------------------------------------------------------

/// Zwei Umbauten hintereinander: der Lauf muss über beide Glieder zurück.
#[tokio::test]
async fn v2_laeuft_ueber_mehrere_umbauten_zurueck() {
    let pool = setup_pool_before_backfills().await;
    let s1 = Uuid::from_u128(0x8801);
    let s2 = Uuid::from_u128(0x8802);
    let s3 = Uuid::from_u128(0x8803);
    for id in [s1, s2, s3] {
        seed_slot(&pool, id, 3, "09:00:00").await;
    }

    let gen1 = Uuid::from_u128(0xE701);
    let gen2 = Uuid::from_u128(0xE702);
    let gen3 = Uuid::from_u128(0xE703);
    seed_booking(
        &pool, gen1, sp(), s1,
        "2026-05-01T08:00:00.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("alice"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, gen2, sp(), s2,
        "2026-06-01T10:00:00.005000000", Some("2026-07-01T11:00:00.000000000"),
        Some("system"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, gen3, sp(), s3,
        "2026-07-01T11:00:00.006000000", None, Some("system"), None,
    )
    .await;

    run_both(&pool).await;

    assert_eq!(
        created_by_of(&pool, gen3).await.as_deref(),
        Some("alice"),
        "der Lauf muss ueber zwei Umbauten zurueckreichen"
    );
}

/// Zweiter Lauf schreibt nichts mehr — auch `update_version` bleibt gleich.
#[tokio::test]
async fn v2_ist_beim_zweiten_lauf_ein_no_op() {
    let pool = setup_pool_before_backfills().await;
    let alt = Uuid::from_u128(0x8901);
    let neu = Uuid::from_u128(0x8902);
    seed_slot(&pool, alt, 3, "09:00:00").await;
    seed_slot(&pool, neu, 3, "09:00:00").await;

    let b_alt = Uuid::from_u128(0xE801);
    let b_neu = Uuid::from_u128(0xE802);
    seed_booking(
        &pool, b_alt, sp(), alt,
        "2026-05-01T08:00:00.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("alice"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b_neu, sp(), neu,
        "2026-06-01T10:00:00.005000000", None, Some("system"), None,
    )
    .await;

    run_both(&pool).await;
    assert_eq!(created_by_of(&pool, b_neu).await.as_deref(), Some("alice"));
    assert_eq!(update_process_of(&pool, b_neu).await, BACKFILL_PROCESS);
    let version_nach_erstem = update_version_of(&pool, b_neu).await;

    run_v2(&pool).await;

    assert_eq!(
        created_by_of(&pool, b_neu).await.as_deref(),
        Some("alice"),
        "der zweite Lauf darf nichts aendern"
    );
    assert_eq!(
        update_version_of(&pool, b_neu).await,
        version_nach_erstem,
        "der zweite Lauf darf nicht einmal update_version neu setzen"
    );
}

/// v2 steht für sich: sie braucht v1 nicht, um zu wirken.
#[tokio::test]
async fn v2_wirkt_auch_ohne_v1() {
    let pool = setup_pool_before_backfills().await;
    let alt = Uuid::from_u128(0x8A01);
    let neu = Uuid::from_u128(0x8A02);
    seed_slot(&pool, alt, 3, "09:00:00").await;
    seed_slot(&pool, neu, 3, "09:00:00").await;

    let b_alt = Uuid::from_u128(0xE901);
    let b_neu = Uuid::from_u128(0xE902);
    seed_booking(
        &pool, b_alt, sp(), alt,
        "2026-05-01T08:00:00.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("alice"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b_neu, sp(), neu,
        "2026-06-01T10:00:00.005000000", None, Some("system"), None,
    )
    .await;

    run_v2(&pool).await;

    assert_eq!(
        created_by_of(&pool, b_neu).await.as_deref(),
        Some("alice"),
        "v2 muss allein funktionieren"
    );
}

/// Die Zurücknahme. Hat v1 eine Zeile gestempelt, die v2 nicht bestätigen kann,
/// fällt sie auf `'system'` zurück. Hier liegen Löschung und Neuanlage fünf
/// Sekunden auseinander: v1 paart trotzdem (sie schaut nur auf die Ordnung nach
/// `created`), v2 nicht (Zeitfenster).
///
/// Das ist Absicht — eine falsche Urheberschaft im Audit-Trail ist schlimmer als
/// eine fehlende `*`-Markierung. Der Preis ist eine fehlende `*` in den Fällen,
/// in denen v1 zufällig richtig lag.
#[tokio::test]
async fn v2_nimmt_zurueck_was_sie_nicht_bestaetigen_kann() {
    let pool = setup_pool_before_backfills().await;
    let alt = Uuid::from_u128(0x8B01);
    let neu = Uuid::from_u128(0x8B02);
    seed_slot(&pool, alt, 3, "09:00:00").await;
    seed_slot(&pool, neu, 3, "09:00:00").await;

    let b_alt = Uuid::from_u128(0xEA01);
    let b_neu = Uuid::from_u128(0xEA02);
    seed_booking(
        &pool, b_alt, sp(), alt,
        "2026-05-01T08:00:00.000000000", Some("2026-06-01T10:00:00.000000000"),
        Some("alice"), Some("system"),
    )
    .await;
    seed_booking(
        &pool, b_neu, sp(), neu,
        "2026-06-01T10:00:05.000000000", None, Some("system"), None,
    )
    .await;

    run_v1(&pool).await;
    assert_eq!(
        created_by_of(&pool, b_neu).await.as_deref(),
        Some("alice"),
        "Vorbedingung: v1 paart hier nachweislich ueber fuenf Sekunden hinweg. \
         Stimmt das nicht mehr, ist dieser Test gegenstandslos"
    );

    run_v2(&pool).await;

    assert_eq!(
        created_by_of(&pool, b_neu).await.as_deref(),
        Some("system"),
        "was v2 nicht bestaetigen kann, darf nicht stehenbleiben"
    );
}

/// Ohne betroffene Zeilen ist der Lauf ein No-op und bricht nicht ab.
#[tokio::test]
async fn v2_ist_no_op_ohne_betroffene_zeilen() {
    let pool = setup_pool_before_backfills().await;
    run_both(&pool).await;

    let count: i64 = sqlx::query("SELECT COUNT(*) AS c FROM booking")
        .fetch_one(pool.as_ref())
        .await
        .expect("count")
        .try_get("c")
        .expect("decode");
    assert_eq!(count, 0, "leere Tabelle bleibt leer");
}
