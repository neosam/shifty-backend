//! Regression tests for the EmployeeWorkDetails update path.
//!
//! Background: prior to the fix, `EmployeeWorkDetailsDaoImpl::update` cast
//! `expected_hours` (an `f32`) to `i64` before binding it to the SQL parameter.
//! The DB column is `FLOAT` and the entity field is `f32`, so this cast silently
//! truncated fractional working hours on every update — e.g. `40.25` was stored
//! as `40` and read back as `40.0`.
//!
//! These tests roundtrip a fractional `expected_hours` value through the real
//! service + in-memory SQLite stack to lock the fix in place.
//!
//! Note on `reload_active`: the create() path currently writes the *id* into
//! the `update_version` column instead of the freshly-allocated version (see
//! `dao_impl_sqlite/src/employee_work_details.rs`, line ~338). That is a
//! separate pre-existing bug — for these tests we always re-fetch via
//! `find_by_sales_person_id` so we work with the version SQLite actually
//! persisted, and the update can pass the version-check.

use rest::RestStateDef;
use service::employee_work_details::{EmployeeWorkDetails, EmployeeWorkDetailsService};
use service::permission::Authentication;
use service::sales_person::{SalesPerson, SalesPersonService};
use shifty_utils::DayOfWeek;
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

fn ewd_template(sales_person_id: Uuid, expected_hours: f32) -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::nil(),
        sales_person_id,
        expected_hours,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2026,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 52,
        to_year: 2030,
        workdays_per_week: 5,
        is_dynamic: false,
        cap_planned_hours_to_expected: false,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        vacation_days: 25,
        created: None,
        deleted: None,
        version: Uuid::nil(),
    }
}

async fn create_ewd(
    test_setup: &TestSetup,
    sales_person_id: Uuid,
    expected_hours: f32,
) -> EmployeeWorkDetails {
    test_setup
        .rest_state
        .working_hours_service()
        .create(
            &ewd_template(sales_person_id, expected_hours),
            Authentication::Full,
            None,
        )
        .await
        .unwrap()
}

/// Reload the single active EWD row for the given sales person. Use this
/// (not the create() return value) when you need the version that SQLite
/// actually persisted.
async fn reload_active(test_setup: &TestSetup, sales_person_id: Uuid) -> EmployeeWorkDetails {
    let rows = test_setup
        .rest_state
        .working_hours_service()
        .find_by_sales_person_id(sales_person_id, Authentication::Full, None)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1, "expected exactly one active EWD row");
    rows[0].clone()
}

/// Spec: create() persists fractional `expected_hours` exactly.
/// (Already worked before the fix because create() used `as f64` — kept as a
/// guardrail in case someone later "harmonises" create() to mirror the buggy
/// update().)
#[tokio::test]
async fn test_create_preserves_fractional_expected_hours() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Alice").await;
    let created = create_ewd(&test_setup, sp.id, 40.25).await;
    assert!(
        (created.expected_hours - 40.25).abs() < 1e-4,
        "create() must preserve 40.25, got {}",
        created.expected_hours
    );

    let reloaded = reload_active(&test_setup, sp.id).await;
    assert!(
        (reloaded.expected_hours - 40.25).abs() < 1e-4,
        "reload after create must preserve 40.25, got {}",
        reloaded.expected_hours
    );
}

/// Spec (regression): update() must persist fractional `expected_hours` exactly,
/// not truncate to an integer. Frontend sends `40.25`, server must store and
/// return `40.25`.
///
/// This test pins the fix for the silent `expected_hours as i64` truncation in
/// `EmployeeWorkDetailsDaoImpl::update`.
#[tokio::test]
async fn test_update_preserves_fractional_expected_hours() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Bob").await;
    // Start from an integer value so the *update* is the only step that could
    // introduce truncation.
    let _created = create_ewd(&test_setup, sp.id, 40.0).await;
    let initial = reload_active(&test_setup, sp.id).await;

    let updated = test_setup
        .rest_state
        .working_hours_service()
        .update(
            &EmployeeWorkDetails {
                expected_hours: 40.25,
                ..initial.clone()
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap();

    assert!(
        (updated.expected_hours - 40.25).abs() < 1e-4,
        "update() must return 40.25 directly, got {}",
        updated.expected_hours
    );

    // Now reload from the DB to make sure the value actually round-tripped
    // through the column (catches a future bug where update() returns the input
    // verbatim but persists a truncated value).
    let reloaded = reload_active(&test_setup, sp.id).await;
    assert!(
        (reloaded.expected_hours - 40.25).abs() < 1e-4,
        "reload after update must preserve 40.25, got {}",
        reloaded.expected_hours
    );
}

/// Spec (regression, finer precision): also verify a value that requires more
/// than one decimal — e.g. `38.75` (3/4 of an hour) — to make sure the fix
/// covers `step=0.01` inputs from the frontend, not just multiples of 0.25.
#[tokio::test]
async fn test_update_preserves_two_decimal_expected_hours() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Carol").await;
    let _created = create_ewd(&test_setup, sp.id, 38.0).await;
    let initial = reload_active(&test_setup, sp.id).await;

    let updated = test_setup
        .rest_state
        .working_hours_service()
        .update(
            &EmployeeWorkDetails {
                expected_hours: 38.75,
                ..initial.clone()
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap();

    assert!(
        (updated.expected_hours - 38.75).abs() < 1e-4,
        "update() must return 38.75, got {}",
        updated.expected_hours
    );

    let reloaded = reload_active(&test_setup, sp.id).await;
    assert!(
        (reloaded.expected_hours - 38.75).abs() < 1e-4,
        "reload must preserve 38.75, got {}",
        reloaded.expected_hours
    );
}
