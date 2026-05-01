use std::sync::Arc;

use rest::RestStateDef;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService};
use service::permission::Authentication;
use service::sales_person::{SalesPerson, SalesPersonService};
use sqlx::Row;
use time::macros::datetime;
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

async fn create_extra_hours(test_setup: &TestSetup, sales_person_id: Uuid) -> ExtraHours {
    test_setup
        .rest_state
        .extra_hours_service()
        .create(
            &ExtraHours {
                id: Uuid::nil(),
                sales_person_id,
                amount: 4.0,
                category: ExtraHoursCategory::ExtraWork,
                description: "initial".into(),
                date_time: datetime!(2026-04-12 8:00:00),
                created: None,
                deleted: None,
                version: Uuid::nil(),
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap()
}

/// Spec: every freshly-created row has id == logical_id.
#[tokio::test]
async fn test_create_assigns_id_equal_to_logical_id() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Alice").await;
    let created = create_extra_hours(&test_setup, sp.id).await;

    let pool = test_setup.pool.as_ref();
    let id_bytes = created.id.as_bytes().to_vec();
    let row = sqlx::query("SELECT id, logical_id FROM extra_hours WHERE logical_id = ?")
        .bind(&id_bytes)
        .fetch_one(pool)
        .await
        .unwrap();
    let physical_id: Vec<u8> = row.get("id");
    let logical_id: Vec<u8> = row.get("logical_id");
    assert_eq!(physical_id, logical_id);
    assert_eq!(physical_id, created.id.as_bytes().to_vec());
}

/// Spec: update soft-deletes the active row and inserts a new active row sharing
/// the same logical_id, with new physical id and new version.
#[tokio::test]
async fn test_update_creates_tombstone_and_new_active_row() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Bob").await;
    let initial = create_extra_hours(&test_setup, sp.id).await;

    let updated = test_setup
        .rest_state
        .extra_hours_service()
        .update(
            &ExtraHours {
                amount: 5.5,
                description: "corrected".into(),
                ..initial.clone()
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        updated.id, initial.id,
        "logical id stays the same across update"
    );
    assert_ne!(
        updated.version, initial.version,
        "version is rotated on update"
    );

    let pool = test_setup.pool.as_ref();
    let logical_id_bytes = initial.id.as_bytes().to_vec();
    let rows = sqlx::query(
        "SELECT id, deleted, amount FROM extra_hours WHERE logical_id = ? ORDER BY created ASC",
    )
    .bind(&logical_id_bytes)
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2, "tombstone + new active row");
    let tombstone_deleted: Option<String> = rows[0].get("deleted");
    let new_deleted: Option<String> = rows[1].get("deleted");
    let new_amount: f64 = rows[1].get("amount");
    assert!(tombstone_deleted.is_some(), "first row should be a tombstone");
    assert!(new_deleted.is_none(), "second row should be active");
    assert!(
        (new_amount - 5.5).abs() < 1e-6,
        "new row carries the updated amount"
    );
}

/// Spec: find_by_logical_id (via service.update lookup) returns None when only
/// tombstones exist for that logical_id. Update of a deleted entry → not found.
#[tokio::test]
async fn test_update_of_deleted_entry_returns_not_found() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Carol").await;
    let initial = create_extra_hours(&test_setup, sp.id).await;

    test_setup
        .rest_state
        .extra_hours_service()
        .delete(initial.id, Authentication::Full, None)
        .await
        .unwrap();

    let result = test_setup
        .rest_state
        .extra_hours_service()
        .update(
            &ExtraHours {
                amount: 6.0,
                ..initial.clone()
            },
            Authentication::Full,
            None,
        )
        .await;

    assert!(matches!(
        result,
        Err(service::ServiceError::EntityNotFound(_))
    ));
}

/// Spec: stale version → conflict.
#[tokio::test]
async fn test_update_with_stale_version_returns_conflict() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Dave").await;
    let initial = create_extra_hours(&test_setup, sp.id).await;

    let stale_version = Uuid::new_v4();
    let result = test_setup
        .rest_state
        .extra_hours_service()
        .update(
            &ExtraHours {
                version: stale_version,
                amount: 6.0,
                ..initial.clone()
            },
            Authentication::Full,
            None,
        )
        .await;

    assert!(matches!(
        result,
        Err(service::ServiceError::EntityConflicts(_, _, _))
    ));
}

/// Spec: partial unique index — at most one active row per logical_id.
#[tokio::test]
async fn test_partial_unique_index_rejects_two_active_rows_with_same_logical_id() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Eve").await;
    let initial = create_extra_hours(&test_setup, sp.id).await;

    let pool = test_setup.pool.as_ref();
    let new_id = Uuid::new_v4();
    let new_id_bytes = new_id.as_bytes().to_vec();
    let logical_id_bytes = initial.id.as_bytes().to_vec();
    let sales_person_bytes = sp.id.as_bytes().to_vec();
    let version_bytes = Uuid::new_v4().as_bytes().to_vec();
    let custom_bytes = Uuid::nil().as_bytes().to_vec();

    let result = sqlx::query(
        "INSERT INTO extra_hours \
         (id, logical_id, sales_person_id, amount, category, description, custom_extra_hours_id, \
          date_time, created, deleted, update_process, update_version) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?)",
    )
    .bind(&new_id_bytes)
    .bind(&logical_id_bytes)
    .bind(&sales_person_bytes)
    .bind(4.0_f64)
    .bind("ExtraWork")
    .bind::<Option<&str>>(None)
    .bind(&custom_bytes)
    .bind("2026-04-12T08:00:00")
    .bind("2026-04-28T12:00:00")
    .bind("test_unique")
    .bind(&version_bytes)
    .execute(pool)
    .await;

    assert!(
        result.is_err(),
        "second active row with same logical_id should violate the partial unique index"
    );
}

/// Spec: REST PUT /extra-hours/{id} with valid body returns 200 and the updated TO.
/// We exercise the service directly here (REST adds a thin wrapper) since the
/// HTTP error mapping for EntityConflicts is well-tested elsewhere.
#[tokio::test]
async fn test_update_propagates_through_to_persisted_state() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Frank").await;
    let initial = create_extra_hours(&test_setup, sp.id).await;

    let _ = test_setup
        .rest_state
        .extra_hours_service()
        .update(
            &ExtraHours {
                amount: 9.0,
                ..initial.clone()
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap();

    let active: Arc<[ExtraHours]> = test_setup
        .rest_state
        .extra_hours_service()
        .find_by_sales_person_id_and_year_range(
            sp.id,
            shifty_utils::ShiftyDate::from_ymd(2026, 4, 1).unwrap(),
            shifty_utils::ShiftyDate::from_ymd(2026, 4, 30).unwrap(),
            Authentication::Full,
            None,
        )
        .await
        .unwrap();
    assert_eq!(active.len(), 1, "only one active row visible after update");
    assert_eq!(active[0].id, initial.id);
    assert!((active[0].amount - 9.0).abs() < 1e-6);
}
