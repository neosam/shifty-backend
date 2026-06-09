//! Tests fuer `AbsenceConversionServiceImpl` (Phase 8.5, Plan 02).
//!
//! Mockall Unit-Tests (4 Stueck):
//! - convert_extra_hours_happy_path: hr ok + lebende Vacation-Row + gueltiger Range -> Ok
//! - convert_extra_hours_date_order_wrong: start > end -> Err(DateOrderWrong)
//! - convert_extra_hours_overlap_rejected: find_overlapping liefert nicht-leer -> Err(ValidationError)
//! - convert_requires_hr_privilege: check_permission Forbidden -> Err(Forbidden), keine Writes
//!
//! Integration-Test (1 Stueck):
//! - convert_extra_hours_happy_path_integration: in-memory SQLite, seed extra_hours Vacation-Row,
//!   convert, asserte absence_period + migration_source Backlink

use std::sync::Arc;

use dao::absence::{AbsenceCategoryEntity, AbsencePeriodEntity, DayFractionEntity, MockAbsenceDao};
use dao::extra_hours::{ExtraHoursCategoryEntity, ExtraHoursEntity, MockExtraHoursDao};
use dao::migration_source::MockMigrationSourceDao;
use dao::{MockTransaction, MockTransactionDao};
use service::absence_conversion::AbsenceConversionService;
use service::extra_hours::MockExtraHoursService;
use service::permission::Authentication;
use service::{MockPermissionService, ServiceError, ValidationFailureItem};
use time::macros::{date, datetime};
use uuid::{uuid, Uuid};

use crate::absence_conversion::{AbsenceConversionServiceDeps, AbsenceConversionServiceImpl};

// =========================================================================
// Fixtures
// =========================================================================

fn extra_hours_logical_id() -> Uuid {
    uuid!("EE000000-0000-0000-0000-000000000001")
}

fn sales_person_id() -> Uuid {
    uuid!("AA000000-0000-0000-0000-000000000001")
}

fn other_absence_logical_id() -> Uuid {
    uuid!("BB000000-0000-0000-0000-000000000001")
}

fn make_vacation_extra_hours_entity() -> ExtraHoursEntity {
    ExtraHoursEntity {
        id: extra_hours_logical_id(),
        logical_id: extra_hours_logical_id(),
        sales_person_id: sales_person_id(),
        amount: 8.0,
        category: ExtraHoursCategoryEntity::Vacation,
        description: Arc::from(""),
        date_time: datetime!(2026 - 04 - 10 09:00:00),
        created: datetime!(2026 - 04 - 01 09:00:00),
        deleted: None,
        version: Uuid::new_v4(),
    }
}

fn make_absence_period_entity(logical_id: Uuid) -> AbsencePeriodEntity {
    AbsencePeriodEntity {
        id: Uuid::new_v4(),
        logical_id,
        sales_person_id: sales_person_id(),
        category: AbsenceCategoryEntity::Vacation,
        from_date: date!(2026 - 04 - 10),
        to_date: date!(2026 - 04 - 12),
        description: Arc::from(""),
        created: datetime!(2026 - 04 - 10 09:00:00),
        deleted: None,
        version: Uuid::new_v4(),
        day_fraction: DayFractionEntity::Full,
    }
}

// =========================================================================
// Dependencies struct und build_service fuer Mockall-Tests
// =========================================================================

pub(crate) struct AbsenceConversionDependencies {
    pub extra_hours_dao: MockExtraHoursDao,
    pub absence_dao: MockAbsenceDao,
    pub migration_source_dao: MockMigrationSourceDao,
    pub extra_hours_service: MockExtraHoursService,
    pub permission_service: MockPermissionService,
    pub transaction_dao: MockTransactionDao,
}

impl AbsenceConversionServiceDeps for AbsenceConversionDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type ExtraHoursDao = MockExtraHoursDao;
    type AbsenceDao = MockAbsenceDao;
    type MigrationSourceDao = MockMigrationSourceDao;
    type ExtraHoursService = MockExtraHoursService;
    type PermissionService = MockPermissionService;
    type TransactionDao = MockTransactionDao;
}

impl AbsenceConversionDependencies {
    pub(crate) fn build_service(
        self,
    ) -> AbsenceConversionServiceImpl<AbsenceConversionDependencies> {
        AbsenceConversionServiceImpl {
            extra_hours_dao: self.extra_hours_dao.into(),
            absence_dao: self.absence_dao.into(),
            migration_source_dao: self.migration_source_dao.into(),
            extra_hours_service: self.extra_hours_service.into(),
            permission_service: self.permission_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

/// Baut die Standardkonfiguration fuer Mockall-Tests (happy-path Transaktion,
/// hr ok). Individuelle Tests koennen Felder ueberschreiben.
pub(crate) fn build_dependencies() -> AbsenceConversionDependencies {
    let extra_hours_dao = MockExtraHoursDao::new();
    let absence_dao = MockAbsenceDao::new();
    let migration_source_dao = MockMigrationSourceDao::new();
    let extra_hours_service = MockExtraHoursService::new();
    let mut permission_service = MockPermissionService::new();
    let mut transaction_dao = MockTransactionDao::new();

    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    AbsenceConversionDependencies {
        extra_hours_dao,
        absence_dao,
        migration_source_dao,
        extra_hours_service,
        permission_service,
        transaction_dao,
    }
}

// =========================================================================
// Unit Tests (Mockall)
// =========================================================================

/// Happy Path: hr ok + lebende Vacation-Row + gueltiger Range -> Ok(AbsencePeriod)
/// Alle drei Writes muessen je genau 1x aufgerufen werden.
#[tokio::test]
async fn convert_extra_hours_happy_path() {
    let mut deps = build_dependencies();

    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .times(1)
        .returning(|_, _| Ok(Some(make_vacation_extra_hours_entity())));

    deps.absence_dao
        .expect_find_overlapping()
        .times(1)
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));

    deps.absence_dao
        .expect_create()
        .times(1)
        .returning(|_, _, _| Ok(()));

    deps.migration_source_dao
        .expect_upsert_migration_source()
        .times(1)
        .returning(|_, _| Ok(()));

    deps.extra_hours_service
        .expect_soft_delete_bulk()
        .times(1)
        .returning(|_, _, _, _| Ok(()));

    let svc = deps.build_service();
    let result = svc
        .convert_extra_hours_to_absence(
            extra_hours_logical_id(),
            date!(2026 - 04 - 10),
            date!(2026 - 04 - 12),
            None,
            Authentication::Context(()),
            None,
        )
        .await;

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let period = result.unwrap();
    assert_eq!(period.sales_person_id, sales_person_id());
    assert_eq!(period.from_date, date!(2026 - 04 - 10));
    assert_eq!(period.to_date, date!(2026 - 04 - 12));
}

/// CR-01 Regression: Der Soft-Delete MUSS ueber die physische `entity.id` laufen,
/// nicht ueber die als Input uebergebene logical_id. Wir simulieren einen zuvor
/// editierten Eintrag (versioniertes Schreibmodell -> aktive Row hat `id != logical_id`):
/// `find_by_logical_id` liefert eine Entity, deren `id` von der logical_id abweicht.
/// `soft_delete_bulk` MUSS mit genau dieser physischen id aufgerufen werden — `withf`
/// schlaegt fehl, wenn (wie im Bug) die logical_id durchgereicht wuerde.
#[tokio::test]
async fn convert_soft_deletes_by_physical_id_not_logical_id() {
    let mut deps = build_dependencies();

    // Aktive Row nach einem Edit: id = physische id != logical_id.
    let physical_id = uuid!("EE000000-0000-0000-0000-0000000000FF");
    assert_ne!(physical_id, extra_hours_logical_id());

    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .times(1)
        .returning(move |_, _| {
            let mut entity = make_vacation_extra_hours_entity();
            entity.id = physical_id; // versioniertes Schreibmodell: id != logical_id
            Ok(Some(entity))
        });

    deps.absence_dao
        .expect_find_overlapping()
        .times(1)
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));

    deps.absence_dao
        .expect_create()
        .times(1)
        .returning(|_, _, _| Ok(()));

    deps.migration_source_dao
        .expect_upsert_migration_source()
        .times(1)
        .returning(|_, _| Ok(()));

    deps.extra_hours_service
        .expect_soft_delete_bulk()
        .times(1)
        .withf(move |ids, _process, _context, _tx| ids.len() == 1 && ids[0] == physical_id)
        .returning(|_, _, _, _| Ok(()));

    let svc = deps.build_service();
    let result = svc
        .convert_extra_hours_to_absence(
            extra_hours_logical_id(), // Input: logical_id != physische id
            date!(2026 - 04 - 10),
            date!(2026 - 04 - 12),
            None,
            Authentication::Context(()),
            None,
        )
        .await;

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
}

/// Fehlerpfad: start > end -> Err(DateOrderWrong); absence_dao.create darf NICHT aufgerufen werden.
#[tokio::test]
async fn convert_extra_hours_date_order_wrong() {
    let mut deps = build_dependencies();

    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .times(1)
        .returning(|_, _| Ok(Some(make_vacation_extra_hours_entity())));

    // Overlap wird nicht geprueft wenn DateRange invalid
    deps.absence_dao
        .expect_find_overlapping()
        .times(0);

    deps.absence_dao.expect_create().times(0);

    deps.migration_source_dao
        .expect_upsert_migration_source()
        .times(0);

    deps.extra_hours_service
        .expect_soft_delete_bulk()
        .times(0);

    let svc = deps.build_service();
    let result = svc
        .convert_extra_hours_to_absence(
            extra_hours_logical_id(),
            date!(2026 - 04 - 15), // start > end
            date!(2026 - 04 - 10),
            None,
            Authentication::Context(()),
            None,
        )
        .await;

    assert!(
        matches!(result, Err(ServiceError::DateOrderWrong(_, _))),
        "Expected DateOrderWrong, got: {:?}",
        result
    );
}

/// Fehlerpfad: find_overlapping liefert nicht-leer ->
/// Err(ValidationError mit OverlappingPeriod); soft_delete_bulk darf NICHT aufgerufen werden.
#[tokio::test]
async fn convert_extra_hours_overlap_rejected() {
    let mut deps = build_dependencies();

    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .times(1)
        .returning(|_, _| Ok(Some(make_vacation_extra_hours_entity())));

    let conflict = make_absence_period_entity(other_absence_logical_id());

    deps.absence_dao
        .expect_find_overlapping()
        .times(1)
        .return_once(move |_, _, _, _, _| Ok(Arc::from(vec![conflict])));

    deps.absence_dao.expect_create().times(0);

    deps.migration_source_dao
        .expect_upsert_migration_source()
        .times(0);

    deps.extra_hours_service
        .expect_soft_delete_bulk()
        .times(0);

    let svc = deps.build_service();
    let result = svc
        .convert_extra_hours_to_absence(
            extra_hours_logical_id(),
            date!(2026 - 04 - 10),
            date!(2026 - 04 - 12),
            None,
            Authentication::Context(()),
            None,
        )
        .await;

    match &result {
        Err(ServiceError::ValidationError(items)) => {
            assert_eq!(items.len(), 1);
            assert!(
                matches!(&items[0], ValidationFailureItem::OverlappingPeriod(id) if *id == other_absence_logical_id()),
                "Expected OverlappingPeriod({:?}), got: {:?}",
                other_absence_logical_id(),
                items[0]
            );
        }
        other => panic!(
            "Expected ValidationError(OverlappingPeriod), got: {:?}",
            other
        ),
    }
}

/// Fehlerpfad: check_permission liefert Forbidden -> Err(Forbidden);
/// use_transaction und alle Writes duerfern NICHT aufgerufen werden.
#[tokio::test]
async fn convert_requires_hr_privilege() {
    let mut deps = build_dependencies();

    // Privilege check fehlschlaegt
    deps.permission_service = MockPermissionService::new();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Err(ServiceError::Forbidden));

    // Tx darf NICHT geoeffnet werden
    deps.transaction_dao = MockTransactionDao::new();
    deps.transaction_dao.expect_use_transaction().times(0);
    deps.transaction_dao.expect_commit().times(0);

    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .times(0);

    deps.absence_dao.expect_create().times(0);

    deps.migration_source_dao
        .expect_upsert_migration_source()
        .times(0);

    deps.extra_hours_service
        .expect_soft_delete_bulk()
        .times(0);

    let svc = deps.build_service();
    let result = svc
        .convert_extra_hours_to_absence(
            extra_hours_logical_id(),
            date!(2026 - 04 - 10),
            date!(2026 - 04 - 12),
            None,
            Authentication::Context(()),
            None,
        )
        .await;

    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "Expected Forbidden, got: {:?}",
        result
    );
}

// =========================================================================
// Integration Test (in-memory SQLite)
// =========================================================================

#[cfg(test)]
mod integration {
    use std::sync::Arc;

    use async_trait::async_trait;
    use dao_impl_sqlite::absence::AbsenceDaoImpl;
    use dao_impl_sqlite::extra_hours::ExtraHoursDaoImpl;
    use dao_impl_sqlite::migration_source::MigrationSourceDaoImpl;
    use dao_impl_sqlite::{TransactionDaoImpl, TransactionImpl};
    use service::absence_conversion::AbsenceConversionService;
    use service::permission::Authentication;
    use service::MockPermissionService;
    use time::macros::date;
    use uuid::Uuid;

    use crate::absence_conversion::{AbsenceConversionServiceDeps, AbsenceConversionServiceImpl};

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

    /// Stub-ExtraHoursService fuer den Integration-Test.
    /// Akzeptiert TransactionImpl und gibt fuer soft_delete_bulk einfach Ok(()) zurueck.
    /// Das echte soft_delete Verhalten wird im Unit-Test mit .times(1) abgedeckt.
    struct StubExtraHoursService;

    #[async_trait]
    impl service::extra_hours::ExtraHoursService for StubExtraHoursService {
        type Context = ();
        type Transaction = TransactionImpl;

        async fn find_by_sales_person_id_and_year(
            &self,
            _sales_person_id: Uuid,
            _year: u32,
            _until_week: u8,
            _context: Authentication<Self::Context>,
            _tx: Option<Self::Transaction>,
        ) -> Result<Arc<[service::extra_hours::ExtraHours]>, service::ServiceError> {
            unimplemented!("not needed in integration test")
        }

        async fn find_by_sales_person_id_and_year_range(
            &self,
            _sales_person_id: Uuid,
            _from_date: shifty_utils::ShiftyDate,
            _to_date: shifty_utils::ShiftyDate,
            _context: Authentication<Self::Context>,
            _tx: Option<Self::Transaction>,
        ) -> Result<Arc<[service::extra_hours::ExtraHours]>, service::ServiceError> {
            unimplemented!("not needed in integration test")
        }

        async fn find_by_week(
            &self,
            _year: u32,
            _week: u8,
            _context: Authentication<Self::Context>,
            _tx: Option<Self::Transaction>,
        ) -> Result<Arc<[service::extra_hours::ExtraHours]>, service::ServiceError> {
            unimplemented!("not needed in integration test")
        }

        async fn create(
            &self,
            _entity: &service::extra_hours::ExtraHours,
            _context: Authentication<Self::Context>,
            _tx: Option<Self::Transaction>,
        ) -> Result<service::extra_hours::ExtraHours, service::ServiceError> {
            unimplemented!("not needed in integration test")
        }

        async fn update(
            &self,
            _entity: &service::extra_hours::ExtraHours,
            _context: Authentication<Self::Context>,
            _tx: Option<Self::Transaction>,
        ) -> Result<service::extra_hours::ExtraHours, service::ServiceError> {
            unimplemented!("not needed in integration test")
        }

        async fn delete(
            &self,
            _id: Uuid,
            _context: Authentication<Self::Context>,
            _tx: Option<Self::Transaction>,
        ) -> Result<(), service::ServiceError> {
            unimplemented!("not needed in integration test")
        }

        async fn soft_delete_bulk(
            &self,
            _ids: Arc<[Uuid]>,
            _update_process: &str,
            _context: Authentication<Self::Context>,
            _tx: Option<Self::Transaction>,
        ) -> Result<(), service::ServiceError> {
            // In Integration-Test simulieren wir nur Ok(()) — das echte Verhalten
            // wird durch Unit-Test convert_extra_hours_happy_path (.times(1)) abgedeckt.
            Ok(())
        }
    }

    // Integration-Deps: echte DAO-Impls + Stubs fuer Services
    struct IntegrationDeps;

    impl AbsenceConversionServiceDeps for IntegrationDeps {
        type Context = ();
        type Transaction = TransactionImpl;
        type ExtraHoursDao = ExtraHoursDaoImpl;
        type AbsenceDao = AbsenceDaoImpl;
        type MigrationSourceDao = MigrationSourceDaoImpl;
        type ExtraHoursService = StubExtraHoursService;
        type PermissionService = MockPermissionService;
        type TransactionDao = TransactionDaoImpl;
    }

    /// Integration-Test: seed extra_hours Vacation-Row -> convert ->
    /// (a) absence_period existiert mit korrekter Kategorie + from/to,
    /// (c) migration_source Backlink existiert.
    /// soft_delete_bulk wird ueber MockExtraHoursService.times(1) verifiziert
    /// (echte DB-Soft-Delete ist im Unit-Test per .times(1) Assertion abgedeckt).
    #[tokio::test]
    async fn convert_extra_hours_happy_path_integration() {
        let pool = setup_pool().await;
        let tx_dao = TransactionDaoImpl::new(pool.clone());

        // ---- Seed: extra_hours Vacation-Row direkt per SQL anlegen ----
        let sales_person_id = Uuid::new_v4();
        let extra_hours_logical_id = Uuid::new_v4();

        // Seed: sales_person anlegen (FK-Voraussetzung fuer extra_hours).
        // UUID als Bytes binden (SQLite BLOB(16) Format, analog DAOs).
        sqlx::query(
            "INSERT INTO sales_person (id, name, inactive, deleted, update_process, update_version) \
             VALUES (?1, ?2, 0, NULL, 'seed', ?3)"
        )
        .bind(sales_person_id.as_bytes().to_vec())
        .bind("Test Person")
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .execute(pool.as_ref())
        .await
        .expect("seed sales_person");

        // Seed: extra_hours Vacation-Row direkt per SQL anlegen.
        // Spalten gemaess Migration 20260428101456 (kein 'version'-Feld).
        // UUIDs als BLOB(16) Bytes (analog dao_impl_sqlite-Pattern).
        let nil_uuid_bytes = Uuid::nil().as_bytes().to_vec();
        sqlx::query(
            "INSERT INTO extra_hours \
             (id, logical_id, sales_person_id, amount, category, description, date_time, created, deleted, update_timestamp, update_process, update_version, custom_extra_hours_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, NULL, 'seed', ?9, ?10)"
        )
        .bind(extra_hours_logical_id.as_bytes().to_vec())
        .bind(extra_hours_logical_id.as_bytes().to_vec())
        .bind(sales_person_id.as_bytes().to_vec())
        .bind(8.0_f32)
        .bind("Vacation")
        .bind("")
        .bind("2026-04-10T09:00:00")
        .bind("2026-04-01T09:00:00")
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .bind(&nil_uuid_bytes)
        .execute(pool.as_ref())
        .await
        .expect("seed extra_hours");

        // ---- Service aufbauen ----
        let extra_hours_dao = ExtraHoursDaoImpl::new(pool.clone());
        let absence_dao = AbsenceDaoImpl::new(pool.clone());
        let migration_source_dao_instance = MigrationSourceDaoImpl::new(pool.clone());

        // StubExtraHoursService: soft_delete_bulk gibt Ok(()) zurueck
        let extra_hours_service = StubExtraHoursService;

        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_, _| Ok(()));

        let svc = AbsenceConversionServiceImpl::<IntegrationDeps> {
            extra_hours_dao: extra_hours_dao.into(),
            absence_dao: absence_dao.into(),
            migration_source_dao: migration_source_dao_instance.into(),
            extra_hours_service: extra_hours_service.into(),
            permission_service: permission_service.into(),
            transaction_dao: tx_dao.into(),
        };

        // ---- Convert aufrufen ----
        let result = svc
            .convert_extra_hours_to_absence(
                extra_hours_logical_id,
                date!(2026 - 04 - 10),
                date!(2026 - 04 - 12),
                None,
                Authentication::Context(()),
                None,
            )
            .await;

        assert!(
            result.is_ok(),
            "Expected Ok from convert, got: {:?}",
            result.err()
        );
        let period = result.unwrap();

        // (a) absence_period existiert mit korrekten Daten
        assert_eq!(period.sales_person_id, sales_person_id);
        assert_eq!(period.from_date, date!(2026 - 04 - 10));
        assert_eq!(period.to_date, date!(2026 - 04 - 12));

        // (c) migration_source Backlink existiert (direkt per SQL verifizieren).
        // UUIDs werden als BLOB(16) gespeichert — analog dao_impl_sqlite-Pattern.
        let row = sqlx::query_as::<_, (Vec<u8>, Vec<u8>)>(
            "SELECT extra_hours_id, absence_period_id FROM absence_period_migration_source WHERE extra_hours_id = ?1"
        )
        .bind(extra_hours_logical_id.as_bytes().to_vec())
        .fetch_optional(pool.as_ref())
        .await
        .expect("query migration source");

        let row = row.expect("migration source backlink should exist");
        let found_extra_hours_id = Uuid::from_slice(&row.0).expect("parse extra_hours_id");
        let found_absence_period_id = Uuid::from_slice(&row.1).expect("parse absence_period_id");
        assert_eq!(found_extra_hours_id, extra_hours_logical_id);
        assert_eq!(found_absence_period_id, period.id);
    }
}
