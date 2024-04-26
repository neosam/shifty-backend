use std::sync::Arc;

use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    let pool = Arc::new(
        SqlitePool::connect("sqlite:./localdb.sqlite3")
            .await
            .expect("Could not connect to database"),
    );
    let hello_dao = dao_impl::HelloDaoImpl::new(pool.clone());
    let permission_dao = dao_impl::PermissionDaoImpl::new(pool);

    // Always authenticate with DEVUSER during development.
    // This is used to test the permission service locally without a login service.
    //
    // TODO: Implement a proper authentication service when used in produciton. Maybe
    // use differnet implementations on debug then on release.  Or control it via a
    // feature.
    let user_service = service_impl::UserServiceDev;
    let permission_service =
        service_impl::PermissionServiceImpl::new(permission_dao.into(), user_service.into());
    let hello_service =
        service_impl::HelloServiceImpl::new(hello_dao.into(), permission_service.into());
    rest::start_server(hello_service).await
}
