//! End-to-End-Integrationstests fuer die Absence-Domain (Phase 1).
//!
//! Alle Tests laufen gegen eine frische In-Memory-SQLite via [`TestSetup::new`]
//! und decken den vollen CRUD-Pfad inkl. der Schema-Constraints
//! (DB-CHECK + Partial-Unique-Index) sowie der Self-Overlap-Detection (D-12,
//! D-15) und Soft-Delete-Logik ab. Die Tests benutzen `Authentication::Full`,
//! d.h. der Service-Pfad ist nicht durch die Permission-Middleware gegated.

use rest::RestStateDef;
use service::absence::{AbsenceCategory, AbsencePeriod, AbsenceService};
use service::permission::Authentication;
use service::sales_person::{SalesPerson, SalesPersonService};
use sqlx::Row;
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

async fn create_absence_period(test_setup: &TestSetup, sales_person_id: Uuid) -> AbsencePeriod {
    test_setup
        .rest_state
        .absence_service()
        .create(
            &AbsencePeriod {
                id: Uuid::nil(),
                sales_person_id,
                category: AbsenceCategory::Vacation,
                from_date: date!(2026 - 04 - 12),
                to_date: date!(2026 - 04 - 15),
                description: "initial".into(),
                created: None,
                deleted: None,
                version: Uuid::nil(),
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap()
        // Phase-3-Plan-03: AbsenceService::create returniert nun
        // AbsencePeriodCreateResult { absence, warnings } — Integration-Tests
        // unwrappen hier .absence; Warnings werden in Plan 06-Tests
        // (Cross-Source-Stubs) explizit verifiziert.
        .absence
}

/// Spec: every freshly-created absence_period has id == logical_id (D-07).
#[tokio::test]
async fn test_create_assigns_id_equal_to_logical_id() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Alice").await;
    let created = create_absence_period(&test_setup, sp.id).await;

    assert_ne!(created.id, Uuid::nil(), "id must be assigned");
    assert_ne!(created.version, Uuid::nil(), "version must be assigned");

    let pool = test_setup.pool.as_ref();
    let logical_id_bytes = created.id.as_bytes().to_vec();
    let row = sqlx::query("SELECT id, logical_id FROM absence_period WHERE logical_id = ?")
        .bind(&logical_id_bytes)
        .fetch_one(pool)
        .await
        .unwrap();
    let physical_id: Vec<u8> = row.get("id");
    let logical_id: Vec<u8> = row.get("logical_id");
    assert_eq!(
        physical_id, logical_id,
        "first version: physical id and logical_id are identical"
    );
    assert_eq!(physical_id, created.id.as_bytes().to_vec());
}

/// Spec: update soft-deletes the active row and inserts a new active row
/// sharing the same logical_id with a fresh physical id and a rotated version.
#[tokio::test]
async fn test_update_creates_tombstone_and_new_active_row() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Bob").await;
    let initial = create_absence_period(&test_setup, sp.id).await;

    let updated = test_setup
        .rest_state
        .absence_service()
        .update(
            &AbsencePeriod {
                to_date: date!(2026 - 04 - 18),
                description: "extended".into(),
                ..initial.clone()
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap()
        // Phase-3-Plan-03: AbsencePeriodCreateResult.absence-Unwrap.
        .absence;

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
        "SELECT id, deleted, to_date FROM absence_period WHERE logical_id = ? ORDER BY created ASC",
    )
    .bind(&logical_id_bytes)
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2, "tombstone + new active row");
    let tombstone_deleted: Option<String> = rows[0].get("deleted");
    let new_deleted: Option<String> = rows[1].get("deleted");
    let new_to: String = rows[1].get("to_date");
    assert!(
        tombstone_deleted.is_some(),
        "first row should be a tombstone"
    );
    assert!(new_deleted.is_none(), "second row should be active");
    assert_eq!(new_to, "2026-04-18", "new row carries the updated to_date");
}

/// Spec: partial unique index — at most one active row per logical_id.
#[tokio::test]
async fn test_partial_unique_index_enforces_one_active_per_logical_id() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Eve").await;
    let initial = create_absence_period(&test_setup, sp.id).await;

    let pool = test_setup.pool.as_ref();
    let new_id_bytes = Uuid::new_v4().as_bytes().to_vec();
    let logical_id_bytes = initial.id.as_bytes().to_vec();
    let sales_person_bytes = sp.id.as_bytes().to_vec();
    let version_bytes = Uuid::new_v4().as_bytes().to_vec();

    let result = sqlx::query(
        "INSERT INTO absence_period \
         (id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_process, update_version) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?)",
    )
    .bind(&new_id_bytes)
    .bind(&logical_id_bytes)
    .bind(&sales_person_bytes)
    .bind("Vacation")
    .bind("2026-05-01")
    .bind("2026-05-05")
    .bind::<Option<&str>>(None)
    .bind("2026-05-01T00:00:00")
    .bind("test_unique")
    .bind(&version_bytes)
    .execute(pool)
    .await;

    assert!(
        result.is_err(),
        "second active row with same logical_id should violate the partial unique index"
    );
}

/// Spec: the DB CHECK constraint rejects rows where to_date < from_date —
/// defense-in-depth alongside the service-layer DateRange validation.
#[tokio::test]
async fn test_check_constraint_rejects_inverted_range() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Mallory").await;

    let pool = test_setup.pool.as_ref();
    let id_bytes = Uuid::new_v4().as_bytes().to_vec();
    let logical_id_bytes = id_bytes.clone();
    let sales_person_bytes = sp.id.as_bytes().to_vec();
    let version_bytes = Uuid::new_v4().as_bytes().to_vec();

    let result = sqlx::query(
        "INSERT INTO absence_period \
         (id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_process, update_version) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?)",
    )
    .bind(&id_bytes)
    .bind(&logical_id_bytes)
    .bind(&sales_person_bytes)
    .bind("Vacation")
    .bind("2026-05-20")
    .bind("2026-05-15") // inverted: to_date < from_date
    .bind::<Option<&str>>(None)
    .bind("2026-05-01T00:00:00")
    .bind("test_check")
    .bind(&version_bytes)
    .execute(pool)
    .await;

    assert!(result.is_err(), "DB CHECK should reject inverted range");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.to_lowercase().contains("check"),
        "expected CHECK violation, got: {}",
        err_msg
    );
}

/// Spec: creating a second absence_period for the same sales person and
/// category whose range overlaps the existing one yields a ValidationError.
#[tokio::test]
async fn test_create_overlapping_same_category_returns_validation_error() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Nina").await;
    let _initial = create_absence_period(&test_setup, sp.id).await; // 2026-04-12..15 Vacation

    let conflict = AbsencePeriod {
        id: Uuid::nil(),
        sales_person_id: sp.id,
        category: AbsenceCategory::Vacation,
        from_date: date!(2026 - 04 - 14), // overlaps initial
        to_date: date!(2026 - 04 - 18),
        description: "conflict".into(),
        created: None,
        deleted: None,
        version: Uuid::nil(),
    };
    let result = test_setup
        .rest_state
        .absence_service()
        .create(&conflict, Authentication::Full, None)
        .await;
    assert!(
        matches!(result, Err(service::ServiceError::ValidationError(_))),
        "expected ValidationError, got: {:?}",
        result
    );
}

/// Spec (D-12): cross-category overlap is allowed — e.g. SickLeave during
/// Vacation; categories are filtered independently in find_overlapping.
#[tokio::test]
async fn test_create_overlapping_different_category_succeeds() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Olive").await;
    let _initial = create_absence_period(&test_setup, sp.id).await; // Vacation 12..15

    let sick = AbsencePeriod {
        id: Uuid::nil(),
        sales_person_id: sp.id,
        category: AbsenceCategory::SickLeave, // different category
        from_date: date!(2026 - 04 - 13),
        to_date: date!(2026 - 04 - 14), // covered by Vacation
        description: "sick".into(),
        created: None,
        deleted: None,
        version: Uuid::nil(),
    };
    let result = test_setup
        .rest_state
        .absence_service()
        .create(&sick, Authentication::Full, None)
        .await;
    assert!(
        result.is_ok(),
        "different categories should be allowed to overlap (D-12), got: {:?}",
        result
    );
}

/// Spec (D-15): updating the own absence_period to extend its range MUST NOT
/// collide with itself in the self-overlap check (exclude_logical_id is set).
#[tokio::test]
async fn test_update_can_extend_range_without_self_collision() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Peggy").await;
    let initial = create_absence_period(&test_setup, sp.id).await; // 12..15

    let extended = AbsencePeriod {
        id: initial.id,
        sales_person_id: sp.id,
        category: AbsenceCategory::Vacation,
        from_date: date!(2026 - 04 - 10), // earlier
        to_date: date!(2026 - 04 - 20),   // later — fully encompasses old range
        description: "extended".into(),
        created: initial.created,
        deleted: None,
        version: initial.version,
    };
    let result = test_setup
        .rest_state
        .absence_service()
        .update(&extended, Authentication::Full, None)
        .await;
    assert!(
        result.is_ok(),
        "extending self range should not collide with self (D-15), got: {:?}",
        result
    );
}

/// Spec: delete is a soft-delete — find_by_id no longer surfaces the row,
/// but a tombstone exists in the DB with a populated `deleted` column.
#[tokio::test]
async fn test_delete_softdeletes_row() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Quinn").await;
    let initial = create_absence_period(&test_setup, sp.id).await;

    test_setup
        .rest_state
        .absence_service()
        .delete(initial.id, Authentication::Full, None)
        .await
        .unwrap();

    let result = test_setup
        .rest_state
        .absence_service()
        .find_by_id(initial.id, Authentication::Full, None)
        .await;
    assert!(
        matches!(result, Err(service::ServiceError::EntityNotFound(_))),
        "find_by_id after delete should yield EntityNotFound, got: {:?}",
        result
    );

    let pool = test_setup.pool.as_ref();
    let logical_id_bytes = initial.id.as_bytes().to_vec();
    let row = sqlx::query(
        "SELECT deleted FROM absence_period WHERE logical_id = ? ORDER BY created DESC LIMIT 1",
    )
    .bind(&logical_id_bytes)
    .fetch_one(pool)
    .await
    .unwrap();
    let deleted: Option<String> = row.get("deleted");
    assert!(
        deleted.is_some(),
        "row should have deleted set after soft delete"
    );
}
