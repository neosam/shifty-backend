use rest::RestStateDef;
use rest_types::{BillingPeriodTO, CreateBillingPeriodRequestTO};
use service::billing_period::BillingPeriodService;
use service::billing_period_report::BillingPeriodReportService;
use service::permission::Authentication;
use service_impl::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION;
use shifty_utils::ShiftyDate;
use sqlx::Row;
use uuid::Uuid;

use crate::integration_test::TestSetup;

/// `build_and_persist_billing_period_report` currently returns the nil UUID
/// instead of the persisted row's id (latent bug, out of scope for this change).
/// Tests look the row up via `get_billing_period_overview` instead.
async fn latest_billing_period_id(test_setup: &TestSetup) -> Uuid {
    let overview = test_setup
        .rest_state
        .billing_period_service()
        .get_billing_period_overview(Authentication::Full, None)
        .await
        .unwrap();
    assert!(!overview.is_empty(), "expected at least one persisted billing period");
    overview[0].id
}

#[tokio::test]
async fn test_pre_existing_billing_period_row_is_backfilled_to_version_one() {
    // Spec Req 1, Scenario 2: rows that existed before the migration receive
    // snapshot_schema_version = 1 via the column DEFAULT. Simulate a "pre-existing"
    // row by inserting directly via raw SQL while omitting the version column.
    let test_setup = TestSetup::new().await;
    let pool = test_setup.pool.as_ref();

    let raw_id = Uuid::new_v4();
    let raw_id_bytes = raw_id.as_bytes().to_vec();
    let version_bytes = Uuid::new_v4().as_bytes().to_vec();

    sqlx::query(
        "INSERT INTO billing_period \
            (id, from_date_time, to_date_time, created, created_by, update_version, update_process) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&raw_id_bytes)
    .bind("2024-01-01T00:00:00Z")
    .bind("2024-01-31T23:59:59Z")
    .bind("2024-01-01T00:00:00Z")
    .bind("legacy")
    .bind(&version_bytes)
    .bind("legacy_process")
    .execute(pool)
    .await
    .unwrap();

    let row = sqlx::query("SELECT snapshot_schema_version FROM billing_period WHERE id = ?")
        .bind(&raw_id_bytes)
        .fetch_one(pool)
        .await
        .unwrap();
    let version: i64 = row.get("snapshot_schema_version");

    assert_eq!(
        version, 1,
        "pre-existing rows must be backfilled to snapshot_schema_version = 1 via the column DEFAULT"
    );
}

#[tokio::test]
async fn test_soft_delete_preserves_snapshot_schema_version() {
    // Spec Req 3: soft-delete must NOT change snapshot_schema_version.
    let test_setup = TestSetup::new().await;
    let rest_state = &test_setup.rest_state;
    let context = Authentication::Full;

    rest_state
        .billing_period_report_service()
        .build_and_persist_billing_period_report(
            ShiftyDate::from_ymd(2024, 1, 31).unwrap(),
            context.clone(),
            None,
        )
        .await
        .unwrap();

    let billing_period_id = latest_billing_period_id(&test_setup).await;
    let id_bytes = billing_period_id.as_bytes().to_vec();

    let row_before =
        sqlx::query("SELECT snapshot_schema_version FROM billing_period WHERE id = ?")
            .bind(&id_bytes)
            .fetch_one(test_setup.pool.as_ref())
            .await
            .unwrap();
    let version_before: i64 = row_before.get("snapshot_schema_version");

    rest_state
        .billing_period_service()
        .delete_billing_period(billing_period_id, context, None)
        .await
        .unwrap();

    let row_after =
        sqlx::query("SELECT snapshot_schema_version, deleted FROM billing_period WHERE id = ?")
            .bind(&id_bytes)
            .fetch_one(test_setup.pool.as_ref())
            .await
            .unwrap();
    let version_after: i64 = row_after.get("snapshot_schema_version");
    let deleted: Option<String> = row_after.get("deleted");

    assert!(deleted.is_some(), "row should have been soft-deleted");
    assert_eq!(
        version_after, version_before,
        "snapshot_schema_version must not change on soft-delete"
    );
}

#[tokio::test]
async fn test_get_billing_period_to_exposes_snapshot_schema_version() {
    // Spec Req 4, Scenario 1: the GET response (BillingPeriodTO) carries
    // snapshot_schema_version equal to the value persisted on the row.
    let test_setup = TestSetup::new().await;
    let rest_state = &test_setup.rest_state;
    let context = Authentication::Full;

    rest_state
        .billing_period_report_service()
        .build_and_persist_billing_period_report(
            ShiftyDate::from_ymd(2024, 1, 31).unwrap(),
            context.clone(),
            None,
        )
        .await
        .unwrap();

    let billing_period_id = latest_billing_period_id(&test_setup).await;
    let billing_period = rest_state
        .billing_period_service()
        .get_billing_period_by_id(billing_period_id, context, None)
        .await
        .unwrap();

    let to = BillingPeriodTO::from(&billing_period);

    assert_eq!(
        to.snapshot_schema_version, CURRENT_SNAPSHOT_SCHEMA_VERSION,
        "GET /billing_period/{{id}} response must expose the persisted snapshot_schema_version"
    );

    // Also verify it appears in the JSON serialization (what the HTTP layer emits).
    let json = serde_json::to_value(&to).unwrap();
    assert_eq!(
        json["snapshot_schema_version"]
            .as_u64()
            .expect("field must be present in JSON"),
        CURRENT_SNAPSHOT_SCHEMA_VERSION as u64,
    );
}

#[tokio::test]
async fn test_create_endpoint_ignores_client_supplied_snapshot_schema_version() {
    // Spec Req 4, Scenario 2: a POST body that attempts to specify
    // snapshot_schema_version: 999 must result in a row whose value is
    // CURRENT_SNAPSHOT_SCHEMA_VERSION, independent of the request body.
    let test_setup = TestSetup::new().await;
    let rest_state = &test_setup.rest_state;
    let context = Authentication::Full;

    let payload_json = serde_json::json!({
        "end_date": "2024-01-31",
        "snapshot_schema_version": 999,
    });
    // CreateBillingPeriodRequestTO has no snapshot_schema_version field; serde
    // silently drops the extra key. This is exactly the structural enforcement
    // the spec requires: the client cannot influence the persisted version.
    let payload: CreateBillingPeriodRequestTO = serde_json::from_value(payload_json).unwrap();

    rest_state
        .billing_period_report_service()
        .build_and_persist_billing_period_report(
            ShiftyDate::from_date(payload.end_date),
            context,
            None,
        )
        .await
        .unwrap();

    let billing_period_id = latest_billing_period_id(&test_setup).await;
    let id_bytes = billing_period_id.as_bytes().to_vec();
    let row = sqlx::query("SELECT snapshot_schema_version FROM billing_period WHERE id = ?")
        .bind(&id_bytes)
        .fetch_one(test_setup.pool.as_ref())
        .await
        .unwrap();
    let persisted: i64 = row.get("snapshot_schema_version");

    assert_eq!(
        persisted as u32, CURRENT_SNAPSHOT_SCHEMA_VERSION,
        "POST must ignore client-supplied snapshot_schema_version and stamp the constant"
    );
}
