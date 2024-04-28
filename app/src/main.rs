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

#[tokio::main]
async fn main() {
    let pool = Arc::new(
        SqlitePool::connect("sqlite:./localdb.sqlite3")
            .await
            .expect("Could not connect to database"),
    );
    let rest_state = RestStateImpl::new(pool);
    rest::start_server(rest_state).await
}
