use crate::test::error_test::*;
use crate::*;
use mockall::predicate::eq;
use service::PermissionService;
use tokio;

fn generate_dependencies_mocks_permission(
    grant: bool,
    privilege: &'static str,
) -> (
    dao::MockPermissionDao,
    service::user_service::MockUserService,
) {
    let mut permission_dao = dao::MockPermissionDao::new();
    permission_dao
        .expect_has_privilege()
        .with(eq("DEVUSER"), eq(privilege))
        .returning(move |_, _| Ok(grant));

    let mut user_service = service::user_service::MockUserService::new();
    user_service
        .expect_current_user()
        .returning(|_| Ok("DEVUSER".into()));
    (permission_dao, user_service)
}

#[tokio::test]
async fn test_check_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(true, "hello");

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    let result = permission_service.check_permission("hello", ()).await;
    result.expect("Expected successful authorization");
}

#[tokio::test]
async fn test_check_permission_denied() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "hello");

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    let result = permission_service.check_permission("hello", ()).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_user_service_dev() {
    use service::user_service::UserService;
    let user_service = UserServiceDev;
    assert_eq!(
        "DEVUSER",
        user_service.current_user(()).await.unwrap().as_ref()
    );
}

#[tokio::test]
async fn test_create_user() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao
        .expect_create_user()
        .with(
            eq(dao::UserEntity {
                name: "testuser".into(),
            }),
            eq("permission-service"),
        )
        .times(1)
        .returning(|_, _| Ok(()));

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    permission_service
        .create_user("testuser", ())
        .await
        .expect("Extected successful user creation");
}

#[tokio::test]
async fn test_create_user_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(&permission_service.create_user("testuser", ()).await);
}

#[tokio::test]
async fn test_delete_user() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao
        .expect_delete_user()
        .with(eq("testuser"))
        .times(1)
        .returning(|_| Ok(()));

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

    permission_service
        .delete_user("testuser", ())
        .await
        .expect("Expected successful delete");
}
#[tokio::test]
async fn test_delete_user_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(&permission_service.delete_user("testuser", ()).await);
}

#[tokio::test]
async fn test_create_role() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao
        .expect_create_role()
        .with(
            eq(dao::RoleEntity {
                name: "testrole".into(),
            }),
            eq("permission-service"),
        )
        .times(1)
        .returning(|_, _| Ok(()));

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    permission_service
        .create_role("testrole", ())
        .await
        .expect("Extected successful role creation");
}

#[tokio::test]
async fn test_create_role_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(&permission_service.create_role("testrole", ()).await);
}

#[tokio::test]
async fn test_delete_role() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao
        .expect_delete_role()
        .with(eq("testrole"))
        .times(1)
        .returning(|_| Ok(()));

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

    permission_service
        .delete_role("testrole", ())
        .await
        .expect("Expected successful delete");
}

#[tokio::test]
async fn test_delete_role_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(&permission_service.delete_role("testrole", ()).await);
}

#[tokio::test]
async fn test_create_privilege() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao
        .expect_create_privilege()
        .with(
            eq(dao::PrivilegeEntity {
                name: "testprivilege".into(),
            }),
            eq("permission-service"),
        )
        .times(1)
        .returning(|_, _| Ok(()));

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

    permission_service
        .create_privilege("testprivilege", ())
        .await
        .expect("Extected successful privilege creation");
}
#[tokio::test]
async fn test_create_privilege_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(
        &permission_service
            .create_privilege("testprivilege", ())
            .await,
    );
}

#[tokio::test]
async fn test_delete_privilege() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao
        .expect_delete_privilege()
        .with(eq("testprivilege"))
        .times(1)
        .returning(|_| Ok(()));

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

    permission_service
        .delete_privilege("testprivilege", ())
        .await
        .expect("Expected successful delete");
}

#[tokio::test]
async fn test_delete_privilege_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(
        &permission_service
            .delete_privilege("testprivilege", ())
            .await,
    );
}

#[tokio::test]
async fn test_add_user_role() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao
        .expect_add_user_role()
        .with(eq("testuser"), eq("testrole"), eq("permission-service"))
        .times(1)
        .returning(|_, _, _| Ok(()));

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

    permission_service
        .add_user_role("testuser", "testrole", ())
        .await
        .expect("Extected successful user role creation");
}

#[tokio::test]
async fn test_add_user_role_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(
        &permission_service
            .add_user_role("testuser", "testrole", ())
            .await,
    );
}

#[tokio::test]
async fn test_add_role_privilege() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao
        .expect_add_role_privilege()
        .with(
            eq("testrole"),
            eq("testprivilege"),
            eq("permission-service"),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

    permission_service
        .add_role_privilege("testrole", "testprivilege", ())
        .await
        .expect("Extected successful role privilege creation");
}

#[tokio::test]
async fn test_add_role_privilege_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(
        &permission_service
            .add_role_privilege("testrole", "testprivilege", ())
            .await,
    );
}

#[tokio::test]
async fn test_delete_role_privilege() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao
        .expect_delete_role_privilege()
        .with(eq("testrole"), eq("testprivilege"))
        .times(1)
        .returning(|_, _| Ok(()));

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

    permission_service
        .delete_role_privilege("testrole", "testprivilege", ())
        .await
        .expect("Extected successful role privilege deletion");
}

#[tokio::test]
async fn test_delete_role_privilege_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(
        &permission_service
            .delete_role_privilege("testrole", "testprivilege", ())
            .await,
    );
}

#[tokio::test]
async fn test_delete_user_role() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao
        .expect_delete_user_role()
        .with(eq("testuser"), eq("testrole"))
        .times(1)
        .returning(|_, _| Ok(()));

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

    permission_service
        .delete_user_role("testuser", "testrole", ())
        .await
        .expect("Extected successful user role deletion");
}

#[tokio::test]
async fn test_delete_user_role_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(
        &permission_service
            .delete_user_role("testuser", "testrole", ())
            .await,
    );
}

#[tokio::test]
async fn test_all_roles() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao.expect_all_roles().times(1).returning(|| {
        Ok(Arc::new([
            dao::RoleEntity {
                name: "testrole".into(),
            },
            dao::RoleEntity {
                name: "testrole2".into(),
            },
        ]))
    });

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

    let all_roles = permission_service
        .get_all_roles(())
        .await
        .expect("Expected roles successfully");
    assert_eq!(all_roles.len(), 2);
    assert_eq!(all_roles[0].name.as_ref(), "testrole");
    assert_eq!(all_roles[1].name.as_ref(), "testrole2");
}

#[tokio::test]
async fn test_all_roles_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(&permission_service.get_all_roles(()).await);
}

#[tokio::test]
async fn test_all_users() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao.expect_all_users().times(1).returning(|| {
        Ok(Arc::new([
            dao::UserEntity {
                name: "testuser".into(),
            },
            dao::UserEntity {
                name: "testuser2".into(),
            },
        ]))
    });

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

    let all_users = permission_service
        .get_all_users(())
        .await
        .expect("Expected users successfully");

    assert_eq!(all_users.len(), 2);
    assert_eq!(all_users[0].name.as_ref(), "testuser");
    assert_eq!(all_users[1].name.as_ref(), "testuser2");
}

#[tokio::test]
async fn test_all_users_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(&permission_service.get_all_users(()).await);
}

#[tokio::test]
async fn test_all_privileges() {
    let (mut permission_dao, user_service) = generate_dependencies_mocks_permission(true, "admin");
    permission_dao
        .expect_all_privileges()
        .times(1)
        .returning(|| {
            Ok(Arc::new([
                dao::PrivilegeEntity {
                    name: "testprivilege".into(),
                },
                dao::PrivilegeEntity {
                    name: "testprivilege2".into(),
                },
            ]))
        });

    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));

    let all_privileges = permission_service
        .get_all_privileges(())
        .await
        .expect("Expected privileges successfully");

    assert_eq!(all_privileges.len(), 2);
    assert_eq!(all_privileges[0].name.as_ref(), "testprivilege");
    assert_eq!(all_privileges[1].name.as_ref(), "testprivilege2");
}

#[tokio::test]
async fn test_all_privileges_without_permission() {
    let (permission_dao, user_service) = generate_dependencies_mocks_permission(false, "admin");
    let permission_service =
        PermissionServiceImpl::new(Arc::new(permission_dao), Arc::new(user_service));
    test_forbidden(&permission_service.get_all_privileges(()).await);
}
