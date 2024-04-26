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

    let permission_service = service_impl::PermissionServiceImpl::new(permission_dao.into());
    let hello_service =
        service_impl::HelloServiceImpl::new(hello_dao.into(), permission_service.into());
    rest::start_server(hello_service).await
}
