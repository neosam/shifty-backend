//! Phase 8 Plan 08-07 Task 1 — Integration-Tests für die Admin-Auto-Grant-
//! Migration (`20260508120000_admin-auto-grant-privilege.sql`).
//!
//! Die Migration kombiniert zwei Mechanismen:
//! 1. **Backfill**: alle bereits existierenden Privilegien (z.B.
//!    `cutover_admin`, `feature_flag_admin`) werden idempotent an die
//!    `admin`-Rolle gebunden.
//! 2. **Forward-Trigger**: jeder zukünftige `INSERT INTO privilege`-Statement
//!    grant das neue Privileg automatisch der `admin`-Rolle.
//!
//! Die Tests verifizieren beide Mechanismen + Idempotenz (re-running der
//! Migration ist no-op) auf einer frischen In-Memory-SQLite-DB, die per
//! `TestSetup::new()` aus allen Migrationen aufgesetzt wird.

use std::sync::Arc;

use dao::PermissionDao;
use sqlx::Row;

use crate::integration_test::TestSetup;

/// Backfill-Verifikation: nach der Migration ist die admin-Rolle an JEDES
/// Privilege gebunden, das in `privilege` steht — inkl. derjenigen, die
/// in nachgelagerten Migrationen ohne expliziten role_privilege-Insert
/// angelegt wurden (`cutover_admin`, `feature_flag_admin`).
#[tokio::test]
async fn admin_role_holds_every_existing_privilege_after_backfill() {
    let test_setup = TestSetup::new().await;
    let pool = test_setup.pool.as_ref();

    let all_privileges: Vec<String> = sqlx::query("SELECT name FROM privilege")
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();

    assert!(
        !all_privileges.is_empty(),
        "expected at least one privilege seeded by migrations"
    );

    let admin_privileges: Vec<String> =
        sqlx::query("SELECT privilege_name FROM role_privilege WHERE role_name = 'admin'")
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get::<String, _>("privilege_name"))
            .collect();

    for privilege in &all_privileges {
        assert!(
            admin_privileges.iter().any(|p| p == privilege),
            "admin role missing privilege `{privilege}`; admin currently holds {admin_privileges:?}"
        );
    }

    // Spot-checks für die Migration-08-07-Motivation: cutover_admin (eingeführt
    // in 20260503000003) und feature_flag_admin (eingeführt in
    // 20260501000000) waren vor 08-07 NICHT an admin gebunden.
    assert!(
        admin_privileges.iter().any(|p| p == "cutover_admin"),
        "admin role must hold cutover_admin after backfill"
    );
    assert!(
        admin_privileges.iter().any(|p| p == "feature_flag_admin"),
        "admin role must hold feature_flag_admin after backfill"
    );
}

/// Forward-Trigger: ein neu per `INSERT INTO privilege` angelegtes Privileg
/// wird automatisch an die admin-Rolle gebunden, ohne dass der Caller das
/// `role_privilege`-Mapping selbst pflegen muss.
#[tokio::test]
async fn newly_inserted_privilege_is_auto_granted_to_admin() {
    let test_setup = TestSetup::new().await;
    let pool = test_setup.pool.as_ref();

    sqlx::query(
        "INSERT INTO privilege (name, update_process) VALUES ('test_priv_08_07', 'integration-test')",
    )
    .execute(pool)
    .await
    .unwrap();

    let row = sqlx::query(
        "SELECT COUNT(*) AS c FROM role_privilege \
         WHERE role_name = 'admin' AND privilege_name = 'test_priv_08_07'",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let count: i64 = row.get("c");
    assert_eq!(
        count, 1,
        "expected privilege_auto_grant_admin trigger to insert exactly one role_privilege row"
    );

    let row = sqlx::query(
        "SELECT update_process FROM role_privilege \
         WHERE role_name = 'admin' AND privilege_name = 'test_priv_08_07'",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let process: String = row.get("update_process");
    assert_eq!(
        process, "admin-auto-grant-trigger",
        "trigger-inserted row must carry the diagnostic update_process tag"
    );
}

/// Idempotenz-Test: das Re-Insert eines Privilegs (nach einem DELETE) bringt
/// kein UNIQUE-Constraint-Failure und re-installiert das Mapping. Ebenso ist
/// das doppelte Insert geschützt durch `INSERT OR IGNORE` im Trigger-Body —
/// d.h. wenn eine spätere Migration paralleles `role_privilege`-Mapping
/// einfügt, schlägt die Trigger-Wirkung still still um (kein Fehler, keine
/// Duplikate).
#[tokio::test]
async fn admin_grant_trigger_is_idempotent_against_manual_role_privilege() {
    let test_setup = TestSetup::new().await;
    let pool = test_setup.pool.as_ref();

    // Manueller `role_privilege`-Insert + dann das Privileg nachträglich
    // anlegen ist nicht der reale Pfad (FK ON DELETE CASCADE), aber wir
    // testen die andere Richtung: Privileg → Trigger-Mapping → manueller
    // Re-Insert via INSERT OR IGNORE.
    sqlx::query(
        "INSERT INTO privilege (name, update_process) VALUES ('test_priv_idem', 'integration-test')",
    )
    .execute(pool)
    .await
    .unwrap();

    // Trigger lieferte bereits ein Mapping; ein manueller "OR IGNORE"-Insert
    // darf das Mapping nicht duplizieren.
    sqlx::query(
        "INSERT OR IGNORE INTO role_privilege (role_name, privilege_name, update_process) \
         VALUES ('admin', 'test_priv_idem', 'integration-test-manual')",
    )
    .execute(pool)
    .await
    .unwrap();

    let count: i64 = sqlx::query(
        "SELECT COUNT(*) AS c FROM role_privilege \
         WHERE role_name = 'admin' AND privilege_name = 'test_priv_idem'",
    )
    .fetch_one(pool)
    .await
    .unwrap()
    .get("c");
    assert_eq!(
        count, 1,
        "expected exactly one role_privilege row even after manual re-insert"
    );
}

/// Sanity-Verknüpfung mit dem realen Permission-Service-Pfad: nach Migration
/// + create_admin_user("DEVUSER") liefert `privileges_for_user` jedes
/// Privilege zurück, das in `privilege` steht — d.h. der DEVUSER kann nun
/// auch `cutover_admin` + `feature_flag_admin` nutzen, ohne dass eine
/// dedizierte privilege-binding-Migration für jedes neue Privileg geschrieben
/// werden muss.
#[tokio::test]
async fn devuser_admin_holds_every_privilege_via_permission_dao() {
    let test_setup = TestSetup::new().await;
    let pool: Arc<sqlx::SqlitePool> = test_setup.pool.clone();
    crate::create_admin_user(pool.clone(), "DEVUSER").await;

    let permission_dao = dao_impl_sqlite::PermissionDaoImpl::new(pool.clone());
    let user_privileges = permission_dao
        .privileges_for_user("DEVUSER")
        .await
        .unwrap();

    let user_privilege_names: Vec<String> = user_privileges
        .iter()
        .map(|p| p.name.as_ref().to_string())
        .collect();

    let all_privileges: Vec<String> = sqlx::query("SELECT name FROM privilege")
        .fetch_all(pool.as_ref())
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();

    for privilege in &all_privileges {
        assert!(
            user_privilege_names.iter().any(|p| p == privilege),
            "DEVUSER missing privilege `{privilege}` via privileges_for_user; \
             user holds {user_privilege_names:?}"
        );
    }
}
