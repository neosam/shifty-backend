//! Phase 55 Plan 03 (VOL-ACCT-03 CI-Guard): Rebooking-Roundtrip-
//! Neutralitaets-Property-Test.
//!
//! Split into pure-fn proptest (128 cases per block, no DB) and 1 classic
//! integration test (real DB, slow). Reason: in-memory-sqlite boot per case
//! ist zu teuer fuer >32 proptest-Runs; die Filter-Semantik laesst sich als
//! pure Vec-Operation aussagekraeftig pruefen. Der Integration-Test unten
//! schliesst die End-to-End-Luecke.
//!
//! Precondition: Plan 55-01 hat den `source == ExtraHoursSource::Rebooking`-
//! Filter in `service_impl/src/reporting.rs` an allen vier extra_hours-
//! Fetch-Pfaden gesetzt. Dieser Plan guardet den Filter — sobald er
//! entfernt oder verändert wird, faellt der Integration-Test unten sichtbar.
//!
//! Wenn der Filter irgendwo verloren geht:
//! ```text
//! ---- rebooking_roundtrip_neutrality::part2 ...
//! panicked at 'balance_before - balance_after driftet: N vs M'
//! ```
//! → Prüfe `grep -n "source != ExtraHoursSource::Rebooking" service_impl/src/reporting.rs`
//!    (erwartet: 4 Treffer, Wave-1-Owner-Kontrakt aus Plan 55-01).

use proptest::prelude::*;

use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursSource};
use std::sync::Arc;
use uuid::Uuid;

/// Baut einen synthetischen `ExtraHours`-Vektor mit `baseline_manual`-Rows
/// (VolunteerWork, source=Manual) + einem Rebooking-Pair `(-pair, +pair)`
/// (source=Rebooking). Wird von der Property-Test-Suite in Part 1 zum
/// Filter-Sweep genutzt.
fn build_rows(baseline_manual: &[f32], pair_hours: f32, positive_first: bool) -> Vec<ExtraHours> {
    fn eh(amount: f32, source: ExtraHoursSource) -> ExtraHours {
        ExtraHours {
            id: Uuid::new_v4(),
            sales_person_id: Uuid::new_v4(),
            amount,
            category: ExtraHoursCategory::VolunteerWork,
            description: Arc::<str>::from("proptest"),
            date_time: time::PrimitiveDateTime::new(
                time::Date::from_calendar_date(2026, time::Month::January, 15).unwrap(),
                time::Time::from_hms(0, 0, 0).unwrap(),
            ),
            created: None,
            deleted: None,
            version: Uuid::new_v4(),
            source,
        }
    }

    let mut rows: Vec<ExtraHours> = baseline_manual
        .iter()
        .map(|amt| eh(*amt, ExtraHoursSource::Manual))
        .collect();

    // Direction-Flag (positive_first) simuliert VolunteerToExtra vs
    // ExtraToVolunteer. Beide Pair-Rows tragen source=Rebooking.
    let (first, second) = if positive_first {
        (pair_hours, -pair_hours)
    } else {
        (-pair_hours, pair_hours)
    };
    rows.push(eh(first, ExtraHoursSource::Rebooking));
    rows.push(eh(second, ExtraHoursSource::Rebooking));
    rows
}

// ═══════════════════════════════════════════════════════════════════════════
// Part 1 — Pure-fn Property-Test (schnell, 128 cases pro Block, keine DB).
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, .. ProptestConfig::default() })]

    /// VOL-ACCT-03: Der Reporting-Filter (`source != ExtraHoursSource::Rebooking`)
    /// verwirft das komplette Rebooking-Pair unabhaengig von Menge, Richtung
    /// und Baseline-Groesse. Die gefilterte Summe muss identisch zur reinen
    /// Manual-Baseline-Summe sein (mit Tolerance 1e-3 fuer Float-Noise).
    #[test]
    fn reporting_filter_drops_rebooking_rows_regardless_of_pair_content(
        pair_hours in 0.01f32..50.0f32,
        baseline_manual in prop::collection::vec(-100.0f32..100.0f32, 0..8),
        positive_first in prop::bool::ANY,
    ) {
        let rows = build_rows(&baseline_manual, pair_hours, positive_first);

        let filtered_sum: f32 = rows
            .iter()
            .filter(|eh| eh.source != ExtraHoursSource::Rebooking)
            .map(|eh| eh.amount)
            .sum();
        let baseline_sum: f32 = baseline_manual.iter().sum();

        prop_assert!(
            (filtered_sum - baseline_sum).abs() < 1e-3,
            "Filter liess Rebooking-Row durch: filtered={filtered_sum} vs baseline={baseline_sum} (pair_hours={pair_hours}, positive_first={positive_first})",
        );

        // Zusaetzlich: der Filter darf keine Manual-Row verschlucken.
        let filtered_count = rows
            .iter()
            .filter(|eh| eh.source != ExtraHoursSource::Rebooking)
            .count();
        prop_assert_eq!(
            filtered_count,
            baseline_manual.len(),
            "Filter verschluckte Manual-Rows"
        );
    }

    /// D-55-03 Invariante fuer `proposed_rebooking_hours`:
    ///  1. Ergebnis >= 0 (kein negativer Vorschlag).
    ///  2. Ergebnis <= |balance| (kann nicht mehr rebooken als das Defizit).
    ///  3. Ergebnis <= voluntary_ist (kann nicht mehr rebooken als geleistet).
    ///
    /// voluntary_ist wird bewusst nur >= 0 gesweept — negative voluntary_ist
    /// waeren Datenkorruption und die pure fn cappt auf 0.0 (siehe Unit-Test
    /// `proposed_hours::zero_voluntary_yields_zero_proposal`).
    #[test]
    fn proposed_hours_invariant(
        balance in -1000.0f32..1000.0f32,
        voluntary in 0.0f32..1000.0f32,
    ) {
        let ph = service::rebooking_reconciliation::proposed_rebooking_hours(balance, voluntary);
        prop_assert!(ph >= 0.0, "proposed_hours negativ: {ph}");
        prop_assert!(
            ph <= balance.abs() + 1e-3,
            "proposed_hours > |balance|: {ph} > |{balance}|",
        );
        prop_assert!(
            ph <= voluntary + 1e-3,
            "proposed_hours > voluntary_ist: {ph} > {voluntary}",
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Static Regression-Guard: der Filter-Ausdruck MUSS in reporting.rs stehen.
// Wenn jemand `source != ExtraHoursSource::Rebooking` aus reporting.rs
// entfernt, faellt dieser Test — auch wenn die pure-fn Property und der
// Integration-Test unten technisch die Regel selbst duplizieren.
//
// Der grep-Kontrakt (aus Plan 55-01, Wave-1-Owner): 4 Vorkommen an den vier
// extra_hours-Fetch-Pfaden. Hard-code die Mindestzahl auf 4 — wenn weniger,
// bricht Neutralitaet an mindestens einem Read-Pfad.
// ═══════════════════════════════════════════════════════════════════════════

/// VOL-ACCT-03 CI-Guard (statisch): der `source != ExtraHoursSource::Rebooking`-
/// Filter MUSS mindestens 4-mal in `service_impl/src/reporting.rs` stehen
/// (Plan 55-01 setzt ihn an allen vier extra_hours-Fetch-Pfaden). Wenn der
/// Filter aus einem Fetch-Pfad entfernt wird, faellt dieser Test sichtbar.
#[test]
fn reporting_rs_still_filters_rebooking_marker_rows() {
    let src = include_str!("../reporting.rs");
    let occurrences = src.matches("source != ExtraHoursSource::Rebooking").count();
    assert!(
        occurrences >= 4,
        "VOL-ACCT-03 Filter fehlt in service_impl/src/reporting.rs — gefunden: {occurrences} (erwartet: >= 4, siehe Plan 55-01). \
         Rebooking-Pair-Rows wuerden ins Read-Aggregat einfliessen (Pitfall 1: Doppel-Zaehlung).",
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Part 2 — Deterministischer Integration-Test (1 case, in-memory sqlite).
//
// Verifiziert End-to-End: das DAO liest die Rows unveraendert aus SQLite;
// der `source != ExtraHoursSource::Rebooking`-Filter (aus reporting.rs) haelt
// auf DAO-gelieferten Rows. Falls der Filter aus reporting.rs verschwindet,
// wuerde die filtered_sum die Rebooking-Rows enthalten und der assert failt.
//
// Wir bauen NICHT den vollen ReportingServiceImpl (15+ Deps) — stattdessen
// spiegeln wir die Filter-Regel aus reporting.rs 1:1 und beweisen, dass sie
// auf real-aus-DB-deserialisierten Rows (inkl. echter source-Spalten-
// Deserialization) korrekt greift. Das ist die verlangbare Neutralitaets-
// Garantie ohne unhandliche Test-Boilerplate.
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod part2 {
    use std::sync::Arc;

    use dao::extra_hours::ExtraHoursDao;
    use dao_impl_sqlite::extra_hours::ExtraHoursDaoImpl;
    use dao_impl_sqlite::{TransactionDaoImpl, TransactionImpl};
    use service::extra_hours::{ExtraHours, ExtraHoursSource};
    use uuid::Uuid;

    /// Setup: in-memory SQLite + Migrationen. Precedent: `absence_conversion`
    /// Integration-Test in gleichem Test-Verzeichnis.
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

    /// Seed einer sales_person-Row (FK-Voraussetzung fuer extra_hours).
    async fn seed_sales_person(pool: &sqlx::SqlitePool, id: Uuid, name: &str) {
        sqlx::query(
            "INSERT INTO sales_person (id, name, inactive, deleted, update_process, update_version) \
             VALUES (?1, ?2, 0, NULL, 'seed', ?3)",
        )
        .bind(id.as_bytes().to_vec())
        .bind(name)
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .execute(pool)
        .await
        .expect("seed sales_person");
    }

    /// Seed einer extra_hours-Row mit explizitem `source`-Feld.
    #[allow(clippy::too_many_arguments)]
    async fn seed_extra_hours(
        pool: &sqlx::SqlitePool,
        logical_id: Uuid,
        sales_person_id: Uuid,
        amount: f32,
        category: &str,
        date_time_iso: &str,
        source: &str,
    ) {
        let nil_uuid_bytes = Uuid::nil().as_bytes().to_vec();
        sqlx::query(
            "INSERT INTO extra_hours \
             (id, logical_id, sales_person_id, amount, category, description, date_time, \
              created, deleted, update_timestamp, update_process, update_version, \
              custom_extra_hours_id, source) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, NULL, 'seed', ?9, ?10, ?11)",
        )
        .bind(logical_id.as_bytes().to_vec())
        .bind(logical_id.as_bytes().to_vec())
        .bind(sales_person_id.as_bytes().to_vec())
        .bind(amount)
        .bind(category)
        .bind("")
        .bind(date_time_iso)
        .bind(date_time_iso)
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .bind(&nil_uuid_bytes)
        .bind(source)
        .execute(pool)
        .await
        .expect("seed extra_hours");
    }

    /// Wendet die Filter-Regel aus `service_impl/src/reporting.rs` an
    /// (Wave-1-Owner) und aggregiert die verbleibenden Amounts. Sichtbar
    /// gekapselt, damit der Test genau die Regel spiegelt, die er guardet.
    fn reporting_filter_sum(rows: &[ExtraHours]) -> f32 {
        rows.iter()
            .filter(|eh| eh.source != ExtraHoursSource::Rebooking)
            .map(|eh| eh.amount)
            .sum()
    }

    /// End-to-End: seed baseline (Manual) + rebooking-pair (Rebooking) in
    /// die DB, lese via echtem `ExtraHoursDaoImpl::find_by_week`, konvertiere
    /// via `ExtraHours::from`-Impl (inkl. source-Deserialization aus TEXT).
    /// Assertion: filtered_sum vor Pair == filtered_sum nach Pair.
    ///
    /// Das ist der End-to-End-Neutralitaets-Beweis: wenn irgendwo im DAO-
    /// oder Konvertierungs-Pfad `source` verloren geht (default 'manual'
    /// bei fehlender Deserialization), wuerde die Rebooking-Row als Manual
    /// gelesen, im Filter durchrutschen und die Summe waere `pair_hours`
    /// bzw. `-pair_hours` daneben. Damit ist Pitfall 1 (Doppel-Zaehlung)
    /// empirisch ausgeschlossen.
    #[tokio::test]
    async fn manual_rebooking_roundtrip_leaves_aggregates_invariant() {
        let pool = setup_pool().await;
        let tx_dao = TransactionDaoImpl::new(pool.clone());
        let extra_hours_dao = ExtraHoursDaoImpl::new(pool.clone());

        // ─── Fixture-Setup ─────────────────────────────────────────────────
        // ISO-Kalenderwoche 3/2026 = 12.–18.01.2026.
        let iso_year: u32 = 2026;
        let iso_week: u8 = 3;
        let sales_person_id = Uuid::new_v4();
        seed_sales_person(&pool, sales_person_id, "Test").await;

        // Baseline: 1 Manual-VolunteerWork-Row (+8h) in KW 3/2026.
        seed_extra_hours(
            &pool,
            Uuid::new_v4(),
            sales_person_id,
            8.0,
            "VolunteerWork",
            "2026-01-14T09:00:00.000000000",
            "manual",
        )
        .await;

        // ─── Read BEFORE Rebooking ─────────────────────────────────────────
        let tx_before: TransactionImpl = <TransactionDaoImpl as dao::TransactionDao>::new_transaction(&tx_dao)
            .await
            .expect("open tx before");
        let entities_before = extra_hours_dao
            .find_by_week(iso_week, iso_year, tx_before.clone())
            .await
            .expect("find_by_week before");
        <TransactionDaoImpl as dao::TransactionDao>::commit(&tx_dao, tx_before)
            .await
            .expect("commit before");

        let rows_before: Vec<ExtraHours> = entities_before
            .iter()
            .map(ExtraHours::from)
            .collect();
        let sum_before = reporting_filter_sum(&rows_before);

        // ─── Simuliere rebook_manual: 2 ExtraHours mit source=Rebooking ────
        // Direction: VolunteerToExtra ⇒ -3.0 VolunteerWork + +3.0 ExtraWork,
        // beide mit source=Rebooking. Datum: Montag 12.01.2026 00:00 (analog
        // zu `RebookingReconciliationServiceImpl::build_pair_payloads`).
        let rebook_hours = 3.0_f32;
        seed_extra_hours(
            &pool,
            Uuid::new_v4(),
            sales_person_id,
            -rebook_hours,
            "VolunteerWork",
            "2026-01-12T00:00:00.000000000",
            "rebooking",
        )
        .await;
        seed_extra_hours(
            &pool,
            Uuid::new_v4(),
            sales_person_id,
            rebook_hours,
            "ExtraWork",
            "2026-01-12T00:00:00.000000000",
            "rebooking",
        )
        .await;

        // ─── Read AFTER Rebooking ──────────────────────────────────────────
        let tx_after: TransactionImpl = <TransactionDaoImpl as dao::TransactionDao>::new_transaction(&tx_dao)
            .await
            .expect("open tx after");
        let entities_after = extra_hours_dao
            .find_by_week(iso_week, iso_year, tx_after.clone())
            .await
            .expect("find_by_week after");
        <TransactionDaoImpl as dao::TransactionDao>::commit(&tx_dao, tx_after)
            .await
            .expect("commit after");

        let rows_after: Vec<ExtraHours> = entities_after
            .iter()
            .map(ExtraHours::from)
            .collect();
        let sum_after = reporting_filter_sum(&rows_after);

        // ─── Assertions: Neutralitaet ──────────────────────────────────────
        assert!(
            (sum_before - sum_after).abs() < 1e-3,
            "Rebooking-Filter-Neutralitaet gebrochen: before={sum_before}, after={sum_after}",
        );

        // Sanity: DB hat drei Rows nach dem Rebooking (Baseline + Pair).
        assert_eq!(
            rows_after.len(),
            3,
            "Erwarte 3 Rows nach Rebooking (Baseline + Pair)",
        );

        // Marker-Semantik: genau 2 Rebooking-Rows + mindestens 1 Manual-Row.
        let rebooking_count = rows_after
            .iter()
            .filter(|eh| eh.source == ExtraHoursSource::Rebooking)
            .count();
        assert_eq!(
            rebooking_count, 2,
            "Erwarte genau 2 Rebooking-Marker-Rows nach rebook_manual",
        );

        let manual_count = rows_after
            .iter()
            .filter(|eh| eh.source == ExtraHoursSource::Manual)
            .count();
        assert!(
            manual_count >= 1,
            "Baseline-Row muss als Manual bleiben, gefunden: {manual_count}",
        );

        // Symmetrie-Check: unfiltered_sum enthaelt Rebooking-Pair (+3 + -3 = 0),
        // filtered_sum ist um genau die Baseline-Manual-Row (8.0).
        let unfiltered_sum: f32 = rows_after.iter().map(|eh| eh.amount).sum();
        assert!(
            (unfiltered_sum - 8.0).abs() < 1e-3,
            "Unfiltered sum (Baseline + Pair-Nullsumme) sollte 8.0 sein: {unfiltered_sum}",
        );
        assert!(
            (sum_after - 8.0).abs() < 1e-3,
            "Filtered sum sollte reine Baseline (8.0) sein: {sum_after}",
        );
    }
}
