//! Integration- und Unit-Tests für [`PdfExportConfigService`] (Phase 48,
//! EXP-02/EXP-03, D-48-BASIC / D-48-ADMIN).
//!
//! - Admin-Gate-Tests via Mocks (analog `vacation_entitlement_offset`).
//! - Persistenz-Tests via echte SQLite-in-memory DB + `sqlx::migrate!` (analog
//!   `absence_conversion`).
//! - Ein Grep-Gate auf den Snapshot-Constant, damit dieser Plan die
//!   `CURRENT_SNAPSHOT_SCHEMA_VERSION` niemals versehentlich bumped (D-48-NO-
//!   SNAPSHOT).

use std::sync::Arc;

use dao::{pdf_export_config::MockPdfExportConfigDao, MockTransaction, MockTransactionDao};
use mockall::predicate::{always, eq};
use service::{
    clock::MockClockService,
    pdf_export_config::{PdfExportConfig, PdfExportConfigService, PdfExportConfigUpdate},
    permission::Authentication,
    uuid_service::MockUuidService,
    MockPermissionService,
};
use uuid::{uuid, Uuid};

use crate::pdf_export_config::{PdfExportConfigServiceDeps, PdfExportConfigServiceImpl};
use crate::test::error_test::test_forbidden;

// ─── Mock-Setup (Admin-Gate + Update-Merge-Semantik) ───────────────────────

pub struct PdfExportConfigServiceDependencies {
    pub pdf_export_config_dao: MockPdfExportConfigDao,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
    pub transaction_dao: MockTransactionDao,
}

impl PdfExportConfigServiceDeps for PdfExportConfigServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type PdfExportConfigDao = MockPdfExportConfigDao;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = MockTransactionDao;
}

impl PdfExportConfigServiceDependencies {
    pub fn build_service(self) -> PdfExportConfigServiceImpl<PdfExportConfigServiceDependencies> {
        PdfExportConfigServiceImpl {
            pdf_export_config_dao: Arc::new(self.pdf_export_config_dao),
            permission_service: Arc::new(self.permission_service),
            clock_service: Arc::new(self.clock_service),
            uuid_service: Arc::new(self.uuid_service),
            transaction_dao: Arc::new(self.transaction_dao),
        }
    }
}

pub fn build_dependencies(admin: bool) -> PdfExportConfigServiceDependencies {
    let pdf_export_config_dao = MockPdfExportConfigDao::new();

    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(always(), always())
        .returning(move |privilege, context| {
            if context == Authentication::Full || (admin && privilege == "admin") {
                Ok(())
            } else {
                Err(service::ServiceError::Forbidden)
            }
        });
    permission_service
        .expect_check_only_full_authentication()
        .with(always())
        .returning(|context| {
            if context == Authentication::Full {
                Ok(())
            } else {
                Err(service::ServiceError::Forbidden)
            }
        });

    let mut clock_service = MockClockService::new();
    clock_service.expect_date_time_now().returning(fixed_datetime);

    let uuid_service = MockUuidService::new();

    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    PdfExportConfigServiceDependencies {
        pdf_export_config_dao,
        permission_service,
        clock_service,
        uuid_service,
        transaction_dao,
    }
}

fn seed_id() -> Uuid {
    uuid!("00000000-0000-0000-0000-000000000048")
}
fn v1() -> Uuid {
    uuid!("11111111-1111-1111-1111-111111111111")
}
fn v2() -> Uuid {
    uuid!("22222222-2222-2222-2222-222222222222")
}
fn fixed_datetime() -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(
        time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
        time::Time::from_hms(23, 42, 0).unwrap(),
    )
}
fn seed_entity() -> dao::pdf_export_config::PdfExportConfigEntity {
    dao::pdf_export_config::PdfExportConfigEntity {
        id: seed_id(),
        enabled: false,
        nextcloud_url: None,
        webdav_user: None,
        webdav_app_token: None,
        target_folder: None,
        weeks_horizon: 8,
        cron_schedule: Arc::from("0 6 * * 1"),
        last_success_at: None,
        last_error_at: None,
        last_error_message: None,
        version: v1(),
    }
}

/// D-48-ADMIN: `get` als Non-Admin → Forbidden, KEIN DAO-Read.
#[tokio::test]
async fn get_non_admin_forbidden() {
    let mut dependencies = build_dependencies(false);
    // Beweis: keine DAO-Reads auf dem denied-Pfad.
    dependencies
        .pdf_export_config_dao
        .expect_get()
        .times(0)
        .returning(|_| Ok(seed_entity()));

    let service = dependencies.build_service();
    let result = service.get(().into(), None).await;
    test_forbidden(&result);
}

/// D-48-ADMIN: `update` als Non-Admin → Forbidden, KEIN DAO-Write.
#[tokio::test]
async fn update_non_admin_forbidden() {
    let mut dependencies = build_dependencies(false);
    dependencies
        .pdf_export_config_dao
        .expect_get()
        .times(0)
        .returning(|_| Ok(seed_entity()));
    dependencies
        .pdf_export_config_dao
        .expect_update()
        .times(0)
        .returning(|_, _, _| Ok(()));

    let service = dependencies.build_service();
    let update = PdfExportConfigUpdate {
        enabled: true,
        nextcloud_url: Some(Arc::from("https://cloud.example.com")),
        webdav_user: Some(Arc::from("user")),
        webdav_app_token: Some(Arc::from("secret")),
        target_folder: Some(Arc::from("Schichtplaene/")),
        weeks_horizon: 4,
        cron_schedule: Arc::from("0 7 * * 1"),
    };
    let result = service.update(update, ().into(), None).await;
    test_forbidden(&result);
}

/// Update mit leerem Token: bestehender Token bleibt erhalten.
#[tokio::test]
async fn update_with_empty_token_keeps_existing() {
    let mut dependencies = build_dependencies(true);

    // Ausgangs-Row: Token bereits gesetzt.
    let existing_with_token = dao::pdf_export_config::PdfExportConfigEntity {
        webdav_app_token: Some(Arc::from("alt-token")),
        nextcloud_url: Some(Arc::from("https://old.example.com")),
        ..seed_entity()
    };

    // get() wird zweimal aufgerufen: einmal zum current-lesen, einmal
    // read-after-write. Wir geben zweimal die "gleiche" Row zurück (die
    // zweite Rückgabe simuliert die persistierte Row nach update).
    let expected_persisted = dao::pdf_export_config::PdfExportConfigEntity {
        enabled: true,
        nextcloud_url: Some(Arc::from("https://new.example.com")),
        webdav_user: Some(Arc::from("user")),
        // Token BLEIBT der alte, weil None im Update gesendet wurde.
        webdav_app_token: Some(Arc::from("alt-token")),
        target_folder: Some(Arc::from("Schichtplaene/")),
        weeks_horizon: 4,
        cron_schedule: Arc::from("0 7 * * 1"),
        version: v2(),
        ..existing_with_token.clone()
    };

    let existing_clone = existing_with_token.clone();
    let expected_clone = expected_persisted.clone();
    dependencies
        .pdf_export_config_dao
        .expect_get()
        .times(2)
        .returning({
            let mut call = 0;
            move |_| {
                call += 1;
                if call == 1 {
                    Ok(existing_clone.clone())
                } else {
                    Ok(expected_clone.clone())
                }
            }
        });
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("pdf-export-config-service::update version"))
        .returning(|_| v2());
    dependencies
        .pdf_export_config_dao
        .expect_update()
        .with(eq(expected_persisted.clone()), always(), always())
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = dependencies.build_service();
    let update = PdfExportConfigUpdate {
        enabled: true,
        nextcloud_url: Some(Arc::from("https://new.example.com")),
        webdav_user: Some(Arc::from("user")),
        // Leerer Token = None ⇒ „behalten".
        webdav_app_token: None,
        target_folder: Some(Arc::from("Schichtplaene/")),
        weeks_horizon: 4,
        cron_schedule: Arc::from("0 7 * * 1"),
    };
    let result: PdfExportConfig = service.update(update, ().into(), None).await.unwrap();

    // Domain-Struct enthält den (persistierten) alten Token — die Maskierung
    // passiert erst im REST-Layer via `From<&PdfExportConfig> for PdfExportConfigTO`.
    assert_eq!(
        result.webdav_app_token.as_deref(),
        Some("alt-token"),
        "Empty token in update must keep the existing value"
    );
    assert_eq!(
        result.nextcloud_url.as_deref(),
        Some("https://new.example.com")
    );
    assert_eq!(result.version, v2());
}

/// Update mit gesetztem Token: neuer Token überschreibt den alten.
#[tokio::test]
async fn update_with_set_token_replaces_existing() {
    let mut dependencies = build_dependencies(true);

    let existing_with_token = dao::pdf_export_config::PdfExportConfigEntity {
        webdav_app_token: Some(Arc::from("alt-token")),
        ..seed_entity()
    };
    let after_update = dao::pdf_export_config::PdfExportConfigEntity {
        webdav_app_token: Some(Arc::from("neu-token")),
        enabled: true,
        version: v2(),
        ..existing_with_token.clone()
    };

    let existing_clone = existing_with_token.clone();
    let after_clone = after_update.clone();
    dependencies
        .pdf_export_config_dao
        .expect_get()
        .times(2)
        .returning({
            let mut call = 0;
            move |_| {
                call += 1;
                if call == 1 {
                    Ok(existing_clone.clone())
                } else {
                    Ok(after_clone.clone())
                }
            }
        });
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("pdf-export-config-service::update version"))
        .returning(|_| v2());
    dependencies
        .pdf_export_config_dao
        .expect_update()
        .with(eq(after_update.clone()), always(), always())
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = dependencies.build_service();
    let update = PdfExportConfigUpdate {
        enabled: true,
        nextcloud_url: None,
        webdav_user: None,
        webdav_app_token: Some(Arc::from("neu-token")),
        target_folder: None,
        weeks_horizon: 8,
        cron_schedule: Arc::from("0 6 * * 1"),
    };
    let result = service.update(update, ().into(), None).await.unwrap();
    assert_eq!(result.webdav_app_token.as_deref(), Some("neu-token"));
}

// ─── Integration-Tests gegen echte SQLite-in-memory DB ────────────────────
//
// Diese Tests fahren die echte Migration `20260703000000_create-pdf-export-
// config.sql` gegen `sqlite::memory:` und benutzen den echten
// `PdfExportConfigDaoImpl` — kein Mocking der DB. Damit wird nachgewiesen,
// dass Schema + DAO zusammenpassen (D-48-CONFIG) und dass `record_success`
// bzw. `record_error` die Status-Felder in der DB richtig setzen.

#[cfg(test)]
mod integration {
    use std::sync::Arc;

    use dao_impl_sqlite::pdf_export_config::PdfExportConfigDaoImpl;
    use dao_impl_sqlite::{TransactionDaoImpl, TransactionImpl};
    use mockall::predicate::always;
    use service::pdf_export_config::PdfExportConfigService;
    use service::permission::Authentication;
    use service::uuid_service::MockUuidService;
    use service::MockPermissionService;
    use uuid::{uuid, Uuid};

    use crate::pdf_export_config::{PdfExportConfigServiceDeps, PdfExportConfigServiceImpl};
    use service::clock::MockClockService;

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

    fn now_a() -> time::PrimitiveDateTime {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, 7.try_into().unwrap(), 3).unwrap(),
            time::Time::from_hms(6, 0, 0).unwrap(),
        )
    }
    fn now_b() -> time::PrimitiveDateTime {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, 7.try_into().unwrap(), 10).unwrap(),
            time::Time::from_hms(6, 0, 0).unwrap(),
        )
    }

    struct IntegrationDeps;
    impl PdfExportConfigServiceDeps for IntegrationDeps {
        type Context = ();
        type Transaction = TransactionImpl;
        type PdfExportConfigDao = PdfExportConfigDaoImpl;
        type PermissionService = MockPermissionService;
        type ClockService = MockClockService;
        type UuidService = MockUuidService;
        type TransactionDao = TransactionDaoImpl;
    }

    fn build_perm_full() -> MockPermissionService {
        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_privilege, context| {
                if context == Authentication::Full {
                    Ok(())
                } else {
                    Err(service::ServiceError::Forbidden)
                }
            });
        permission_service
            .expect_check_only_full_authentication()
            .returning(|context| {
                if context == Authentication::Full {
                    Ok(())
                } else {
                    Err(service::ServiceError::Forbidden)
                }
            });
        permission_service
    }

    /// Frische DB nach `sqlx::migrate!` liefert die Seed-Row: enabled=false,
    /// weeks_horizon=8, cron_schedule="0 0 6 * * 1" (6-Feld nach v2.3.1
    /// `20260704000000_fix-pdf-export-cron-6-field.sql`), alle Text- und
    /// Status-Felder None. (D-48-CONFIG)
    #[tokio::test]
    async fn fresh_db_returns_seed_row() {
        let pool = setup_pool().await;
        let dao = PdfExportConfigDaoImpl::new(pool.clone());
        let tx_dao = TransactionDaoImpl::new(pool.clone());

        let service = PdfExportConfigServiceImpl::<IntegrationDeps> {
            pdf_export_config_dao: Arc::new(dao),
            permission_service: Arc::new(build_perm_full()),
            clock_service: Arc::new(MockClockService::new()),
            uuid_service: Arc::new(MockUuidService::new()),
            transaction_dao: Arc::new(tx_dao),
        };

        let cfg = service
            .get(Authentication::Full, None)
            .await
            .expect("get() must succeed as Full auth");
        assert!(!cfg.enabled);
        assert_eq!(cfg.weeks_horizon, 8);
        assert_eq!(&*cfg.cron_schedule, "0 0 6 * * 1");
        assert!(cfg.nextcloud_url.is_none());
        assert!(cfg.webdav_user.is_none());
        assert!(cfg.webdav_app_token.is_none());
        assert!(cfg.target_folder.is_none());
        assert!(cfg.last_success_at.is_none());
        assert!(cfg.last_error_at.is_none());
        assert!(cfg.last_error_message.is_none());
    }

    /// Admin-Update setzt URL/User/Token/Enabled → next get() liefert alle Werte
    /// (Persistenz-Beweis auf der Service-Domain-Ebene). Die Maskierung des
    /// Tokens passiert im REST-Layer (Task 2 / Test in `rest-types`).
    #[tokio::test]
    async fn admin_update_persists_full_values() {
        let pool = setup_pool().await;
        let dao = PdfExportConfigDaoImpl::new(pool.clone());
        let tx_dao = TransactionDaoImpl::new(pool.clone());

        let mut uuid_service = MockUuidService::new();
        let new_v: Uuid = uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        uuid_service
            .expect_new_uuid()
            .with(always())
            .returning(move |_| new_v);

        let service = PdfExportConfigServiceImpl::<IntegrationDeps> {
            pdf_export_config_dao: Arc::new(dao),
            permission_service: Arc::new(build_perm_full()),
            clock_service: Arc::new(MockClockService::new()),
            uuid_service: Arc::new(uuid_service),
            transaction_dao: Arc::new(tx_dao),
        };

        let update = service::pdf_export_config::PdfExportConfigUpdate {
            enabled: true,
            nextcloud_url: Some(Arc::from("https://cloud.example.com")),
            webdav_user: Some(Arc::from("admin-user")),
            webdav_app_token: Some(Arc::from("secret-token")),
            target_folder: Some(Arc::from("Schichtplaene/")),
            weeks_horizon: 12,
            cron_schedule: Arc::from("0 7 * * 1"),
        };
        let after = service
            .update(update, Authentication::Full, None)
            .await
            .expect("update() must succeed as Full auth");
        assert!(after.enabled);
        assert_eq!(after.weeks_horizon, 12);
        assert_eq!(&*after.cron_schedule, "0 7 * * 1");
        assert_eq!(
            after.nextcloud_url.as_deref(),
            Some("https://cloud.example.com")
        );
        assert_eq!(after.webdav_user.as_deref(), Some("admin-user"));
        // Persistenz-Beweis: das Domain-Struct enthält den Token — die
        // Maskierung passiert erst bei der DTO-Conversion (T-48-02).
        assert_eq!(after.webdav_app_token.as_deref(), Some("secret-token"));
        assert_eq!(after.target_folder.as_deref(), Some("Schichtplaene/"));
        assert_eq!(after.version, new_v);

        // Zweites get() liefert die persistierten Werte erneut.
        let re_read = service.get(Authentication::Full, None).await.unwrap();
        assert!(re_read.enabled);
        assert_eq!(re_read.webdav_app_token.as_deref(), Some("secret-token"));
    }

    /// record_success setzt last_success_at + clearet last_error_*.
    /// Anschließend record_error setzt last_error_at + last_error_message,
    /// last_success_at bleibt unverändert.
    #[tokio::test]
    async fn record_success_and_record_error_persist() {
        let pool = setup_pool().await;
        let dao = PdfExportConfigDaoImpl::new(pool.clone());
        let tx_dao = TransactionDaoImpl::new(pool.clone());

        let mut uuid_service = MockUuidService::new();
        uuid_service
            .expect_new_uuid()
            .returning(|_| Uuid::new_v4());

        let service = PdfExportConfigServiceImpl::<IntegrationDeps> {
            pdf_export_config_dao: Arc::new(dao),
            permission_service: Arc::new(build_perm_full()),
            clock_service: Arc::new(MockClockService::new()),
            uuid_service: Arc::new(uuid_service),
            transaction_dao: Arc::new(tx_dao),
        };

        // First: success.
        service
            .record_success(now_a(), Authentication::Full, None)
            .await
            .expect("record_success as Full auth");
        let after_success = service.get(Authentication::Full, None).await.unwrap();
        assert_eq!(after_success.last_success_at, Some(now_a()));
        assert!(after_success.last_error_at.is_none());
        assert!(after_success.last_error_message.is_none());

        // Then: error.
        service
            .record_error(
                now_b(),
                Arc::from("boom"),
                Authentication::Full,
                None,
            )
            .await
            .expect("record_error as Full auth");
        let after_error = service.get(Authentication::Full, None).await.unwrap();
        assert_eq!(
            after_error.last_success_at,
            Some(now_a()),
            "record_error must NOT clear last_success_at"
        );
        assert_eq!(after_error.last_error_at, Some(now_b()));
        assert_eq!(after_error.last_error_message.as_deref(), Some("boom"));
    }
}

// ─── Snapshot-Version-Gate ──────────────────────────────────────────────────

/// Grep-Gate: die aktuelle `CURRENT_SNAPSHOT_SCHEMA_VERSION` in
/// `service_impl/src/billing_period_report.rs` MUSS 12 sein. Phase 48 fügt
/// keine `BillingPeriodValueType`-Zeile hinzu und darf den Snapshot-Constant
/// deswegen nicht bumpen (D-48-NO-SNAPSHOT).
#[test]
fn snapshot_version_unchanged_grep_gate() {
    const SRC: &str = include_str!("../billing_period_report.rs");
    let needle = "pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;";
    assert!(
        SRC.contains(needle),
        "Phase 48 must NOT bump snapshot version — EXP is not a persisted \
         BillingPeriodValueType. Expected `{needle}` in billing_period_report.rs"
    );
}
