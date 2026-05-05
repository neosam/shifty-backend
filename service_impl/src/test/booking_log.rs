//! Service-Tier-Tests für `BookingLogServiceImpl`.
//!
//! Regression: bookings_view kann `created_by = NULL` enthalten (z. B. für
//! Bookings, die vor der Audit-Tracking-Migration `20250115000000` angelegt
//! wurden, oder über System-Pfade, die historisch `Authentication::Full`
//! ohne created_by-Befüllung verwendet haben). Der Service muss diese
//! Einträge sauber durchreichen — KEIN 500.

use std::sync::Arc;

use dao::{
    booking_log::{BookingLogEntity, MockBookingLogDao},
    MockTransaction, MockTransactionDao,
};
use service::{
    booking_log::{BookingLog, BookingLogService},
    permission::Authentication,
    MockPermissionService,
};
use shifty_utils::DayOfWeek;
use time::macros::{datetime, time};

use crate::booking_log::BookingLogServiceImpl;
use crate::test::error_test::NoneTypeExt;

/// `BookingLog.created_by: None` muss verlustfrei vom DAO durch den Service
/// propagiert werden. Vor Fix wurde NULL im DAO als
/// `DaoError::EnumValueNotFound` abgewiesen → 500 im REST-Layer.
#[tokio::test]
async fn test_get_booking_logs_for_week_passes_through_null_created_by() {
    let entity = BookingLogEntity {
        year: 2026,
        calendar_week: 18,
        day_of_week: DayOfWeek::Monday,
        name: "Alice".into(),
        time_from: time!(08:00:00),
        time_to: time!(16:00:00),
        created: datetime!(2026 - 04 - 15 10:30:00),
        deleted: None,
        created_by: None,
        deleted_by: None,
    };

    let mut booking_log_dao = MockBookingLogDao::new();
    booking_log_dao
        .expect_get_booking_logs_for_week()
        .returning(move |_, _, _| Ok(Arc::from(vec![entity.clone()])));

    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    struct Deps;
    impl crate::booking_log::BookingLogServiceDeps for Deps {
        type Context = ();
        type Transaction = MockTransaction;
        type BookingLogDao = MockBookingLogDao;
        type PermissionService = MockPermissionService;
        type TransactionDao = MockTransactionDao;
    }

    let service: BookingLogServiceImpl<Deps> = BookingLogServiceImpl {
        booking_log_dao: Arc::new(booking_log_dao),
        permission_service: Arc::new(permission_service),
        transaction_dao: Arc::new(transaction_dao),
    };

    let result: Arc<[BookingLog]> = service
        .get_booking_logs_for_week(2026, 18, Authentication::Full, None)
        .await
        .expect("get_booking_logs_for_week must succeed for NULL created_by");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].created_by, None);
    assert_eq!(result[0].name.as_ref(), "Alice");
    assert_eq!(result[0].calendar_week, 18);

    // Suppress unused warning for NoneTypeExt — kept for future expansion.
    let _ = ().auth();
}
