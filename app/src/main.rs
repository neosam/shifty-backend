use std::sync::Arc;

use sqlx::SqlitePool;

type PermissionService =
    service_impl::PermissionServiceImpl<dao_impl::PermissionDaoImpl, service_impl::UserServiceDev>;
type HelloService = service_impl::HelloServiceImpl<dao_impl::HelloDaoImpl, PermissionService>;

#[derive(Clone)]
pub struct RestStateImpl {
    hello_service: Arc<HelloService>,
    permission_service: Arc<PermissionService>,
}
impl rest::RestStateDef for RestStateImpl {
    type HelloService = HelloService;
    type PermissionService = PermissionService;

    fn hello_service(&self) -> Arc<Self::HelloService> {
        self.hello_service.clone()
    }
    fn permission_service(&self) -> Arc<Self::PermissionService> {
        self.permission_service.clone()
    }
}
impl RestStateImpl {
    pub fn new(pool: Arc<sqlx::Pool<sqlx::Sqlite>>) -> Self {
        let hello_dao = dao_impl::HelloDaoImpl::new(pool.clone());
        let permission_dao = dao_impl::PermissionDaoImpl::new(pool);

        // Always authenticate with DEVUSER during development.
        // This is used to test the permission service locally without a login service.
        //
        // TODO: Implement a proper authentication service when used in produciton. Maybe
        // use differnet implementations on debug then on release.  Or control it via a
        // feature.
        let user_service = service_impl::UserServiceDev;
        let permission_service = Arc::new(service_impl::PermissionServiceImpl::new(
            permission_dao.into(),
            user_service.into(),
        ));
        let hello_service = Arc::new(service_impl::HelloServiceImpl::new(
            hello_dao.into(),
            permission_service.clone(),
        ));
        Self {
            hello_service,
            permission_service,
        }
    }
}

async fn create_dev_admin_user(pool: Arc<SqlitePool>) {
    use dao::PermissionDao;
    // On development create the DEVUSER and give it admin permissions.
    let permission_dao = dao_impl::PermissionDaoImpl::new(pool);

    let users = permission_dao.all_users().await.expect("Expected users");
    let contains_admin_user = users.iter().any(|user| user.name.as_ref() == "DEVUSER");
    if !contains_admin_user {
        permission_dao
            .create_user(
                &dao::UserEntity {
                    name: "DEVUSER".into(),
                },
                "dev-first-start",
            )
            .await
            .expect("Expected being able to create the DEVUSER");
        permission_dao
            .add_user_role("DEVUSER", "admin", "dev-first-start")
            .await
            .expect("Expected being able to make DEVUSER an admin");
    }
}

#[tokio::main]
async fn main() {
    let pool = Arc::new(
        SqlitePool::connect("sqlite:./localdb.sqlite3")
            .await
            .expect("Could not connect to database"),
    );

    let rest_state = RestStateImpl::new(pool.clone());
    create_dev_admin_user(pool.clone()).await;
    rest::start_server(rest_state).await
}
