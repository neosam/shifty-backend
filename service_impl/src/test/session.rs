use std::sync::Arc;

use mockall::predicate::eq;
use service::session::SessionService;

use crate::session::{SessionServiceDeps, SessionServiceImpl};

pub struct SessionServiceTestDeps;
impl SessionServiceDeps for SessionServiceTestDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type SessionDao = dao::session::MockSessionDao;
    type UuidService = service::uuid_service::MockUuidService;
    type ClockService = service::clock::MockClockService;
}

fn build_service(
    session_dao: dao::session::MockSessionDao,
    uuid_service: service::uuid_service::MockUuidService,
    clock_service: service::clock::MockClockService,
) -> SessionServiceImpl<SessionServiceTestDeps> {
    SessionServiceImpl {
        session_dao: session_dao.into(),
        uuid_service: uuid_service.into(),
        clock_service: clock_service.into(),
    }
}

#[tokio::test]
async fn test_start_impersonate() {
    let mut session_dao = dao::session::MockSessionDao::new();
    session_dao
        .expect_update_impersonate()
        .with(eq("session-123"), eq(Some(Arc::from("target-user"))))
        .returning(|_, _| Ok(()))
        .times(1);

    let uuid_service = service::uuid_service::MockUuidService::new();
    let clock_service = service::clock::MockClockService::new();
    let service = build_service(session_dao, uuid_service, clock_service);

    let result = service
        .start_impersonate(Arc::from("session-123"), Arc::from("target-user"))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_stop_impersonate() {
    let mut session_dao = dao::session::MockSessionDao::new();
    session_dao
        .expect_update_impersonate()
        .with(eq("session-123"), eq(None::<Arc<str>>))
        .returning(|_, _| Ok(()))
        .times(1);

    let uuid_service = service::uuid_service::MockUuidService::new();
    let clock_service = service::clock::MockClockService::new();
    let service = build_service(session_dao, uuid_service, clock_service);

    let result = service.stop_impersonate(Arc::from("session-123")).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_new_session_has_no_impersonation() {
    let mut session_dao = dao::session::MockSessionDao::new();
    session_dao
        .expect_create()
        .returning(|entity| {
            assert!(entity.impersonate_user_id.is_none());
            Ok(())
        })
        .times(1);

    let mut uuid_service = service::uuid_service::MockUuidService::new();
    uuid_service
        .expect_new_uuid()
        .returning(|_| uuid::Uuid::new_v4());

    let clock_service = service::clock::MockClockService::new();
    let service = build_service(session_dao, uuid_service, clock_service);

    let session = service.new_session_for_user("testuser").await.unwrap();
    assert_eq!(session.user_id.as_ref(), "testuser");
    assert!(session.impersonate_user_id.is_none());
}

#[tokio::test]
async fn test_verify_session_with_impersonate() {
    let mut session_dao = dao::session::MockSessionDao::new();
    session_dao
        .expect_find_by_id()
        .with(eq("session-123"))
        .returning(|_| {
            Ok(Some(dao::session::SessionEntity {
                id: Arc::from("session-123"),
                user_id: Arc::from("admin"),
                expires: 9999999999,
                created: 1000000000,
                impersonate_user_id: Some(Arc::from("target-user")),
            }))
        })
        .times(1);

    let uuid_service = service::uuid_service::MockUuidService::new();
    let clock_service = service::clock::MockClockService::new();
    let service = build_service(session_dao, uuid_service, clock_service);

    let session = service
        .verify_user_session("session-123")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.user_id.as_ref(), "admin");
    assert_eq!(
        session.impersonate_user_id.as_deref(),
        Some("target-user")
    );
}
