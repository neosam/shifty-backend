//! Phase 8.3 (Plan 02) — No-Drift-Garantie + Halbtag-Round-Trip auf der
//! DAO/Service-Ebene gegen In-Memory-SQLite (`TestSetup`-Stack, analog zu
//! `absence_period.rs`).
//!
//! Zwei Tests:
//! 1. `absence_period_default_day_fraction_is_full_on_legacy_rows` — legacy
//!    INSERTs OHNE `day_fraction`-Spalte werden durch die Migration zu
//!    `'full'` aufgefüllt (no-drift, siehe CONTEXT.md). Der DAO-Read liefert
//!    `DayFractionEntity::Full` zurück.
//! 2. `absence_period_create_and_select_with_half_persists_correctly` — eine
//!    via Service erzeugte Periode mit `Half` ist nach Reload weiterhin
//!    `Half` (Round-Trip).
//!
//! Beide Tests pinnen die Plan-02-Garantie, dass `day_fraction` voll durch
//! die DAO-Schicht persistiert.
//!
//! Note: dieser File liegt zwar im `shifty_bin`-Crate (analog zu
//! `absence_period.rs`), gehört aber zum Plan-02-Liefergegenstand — das
//! Mock-only `service_impl`-Crate hat keine sqlx-Abhängigkeit und kann
//! keine In-Memory-SQLite-Tests beherbergen.

use rest::RestStateDef;
use service::absence::{AbsenceCategory, AbsencePeriod, AbsenceService, DayFraction};
use service::permission::Authentication;
use service::sales_person::{SalesPerson, SalesPersonService};
use time::macros::date;
use uuid::Uuid;

use crate::integration_test::TestSetup;

async fn create_sales_person(test_setup: &TestSetup, name: &str) -> SalesPerson {
    test_setup
        .rest_state
        .sales_person_service()
        .create(
            &SalesPerson {
                id: Uuid::nil(),
                version: Uuid::nil(),
                name: name.into(),
                background_color: "#000000".into(),
                inactive: false,
                is_paid: Some(true),
                deleted: None,
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap()
}

/// Spec: legacy rows that pre-date Plan 01's migration (or any client that
/// omits the column from its INSERT) must default to `DayFraction::Full`
/// when read back. The migration declares
/// `day_fraction TEXT NOT NULL DEFAULT 'full'` — this test verifies that
/// the DAO read path picks up the DB default and maps it to
/// `DayFractionEntity::Full`, preserving the no-drift guarantee for
/// existing data (CONTEXT.md).
#[tokio::test]
async fn absence_period_default_day_fraction_is_full_on_legacy_rows() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Legacy").await;

    let pool = test_setup.pool.as_ref();
    let id_bytes = Uuid::new_v4().as_bytes().to_vec();
    let logical_id_bytes = id_bytes.clone();
    let sales_person_bytes = sp.id.as_bytes().to_vec();
    let version_bytes = Uuid::new_v4().as_bytes().to_vec();

    // Direct-SQL INSERT that OMITS the `day_fraction` column entirely —
    // simulates rows persisted by clients/code paths that pre-date Plan 02.
    // The migration's `DEFAULT 'full'` must kick in.
    sqlx::query(
        "INSERT INTO absence_period \
         (id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_process, update_version) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?)",
    )
    .bind(&id_bytes)
    .bind(&logical_id_bytes)
    .bind(&sales_person_bytes)
    .bind("Vacation")
    .bind("2026-06-01")
    .bind("2026-06-05")
    .bind::<Option<&str>>(None)
    .bind("2026-06-01T00:00:00")
    .bind("legacy_insert")
    .bind(&version_bytes)
    .execute(pool)
    .await
    .expect("legacy INSERT (without day_fraction) should succeed and default to 'full'");

    // Read back via the public service path. The Plan-02 TryFrom must
    // resolve the DB default 'full' to DayFraction::Full.
    let logical_id = Uuid::from_slice(&logical_id_bytes).unwrap();
    let absence = test_setup
        .rest_state
        .absence_service()
        .find_by_id(logical_id, Authentication::Full, None)
        .await
        .expect("find_by_id must succeed for the legacy row");
    assert_eq!(
        absence.day_fraction,
        DayFraction::Full,
        "Legacy row inserted without day_fraction must read back as Full (no-drift)"
    );
}

/// Spec: a freshly created absence period with `DayFraction::Half` must
/// round-trip through the DB and come back as `Half` — this is the
/// foundation Plan 04 (reporting halving) builds on. Without this
/// guarantee, persisting `Half` would silently drop back to `Full`.
#[tokio::test]
async fn absence_period_create_and_select_with_half_persists_correctly() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Half").await;

    let created = test_setup
        .rest_state
        .absence_service()
        .create(
            &AbsencePeriod {
                id: Uuid::nil(),
                sales_person_id: sp.id,
                category: AbsenceCategory::Vacation,
                from_date: date!(2026 - 07 - 06),
                to_date: date!(2026 - 07 - 06),
                description: "Halbtag".into(),
                created: None,
                deleted: None,
                version: Uuid::nil(),
                day_fraction: DayFraction::Half,
            },
            Authentication::Full,
            None,
        )
        .await
        .expect("create with Half must succeed")
        .absence;

    assert_eq!(
        created.day_fraction,
        DayFraction::Half,
        "create() must echo back Half"
    );

    // Reload via find_by_id to confirm the round-trip went through the
    // DB column, not just the in-memory return value of create().
    let reloaded = test_setup
        .rest_state
        .absence_service()
        .find_by_id(created.id, Authentication::Full, None)
        .await
        .expect("find_by_id must succeed");
    assert_eq!(
        reloaded.day_fraction,
        DayFraction::Half,
        "reload after create with Half must read Half from the DB"
    );

    // Defense-in-depth: a raw SELECT must see the lowercase 'half' string
    // in the column — verifies the DB-storage convention from Plan 02.
    let pool = test_setup.pool.as_ref();
    let logical_id_bytes = created.id.as_bytes().to_vec();
    let row: (String,) = sqlx::query_as(
        "SELECT day_fraction FROM absence_period WHERE logical_id = ? AND deleted IS NULL",
    )
    .bind(&logical_id_bytes)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(
        row.0, "half",
        "DB column must store the lowercase 'half' representation"
    );
}
