use std::sync::Arc;

use dao_impl::{
    extra_hours::ExtraHoursDaoImpl, shiftplan_report::ShiftplanReportDaoImpl,
    working_hours::WorkingHoursDaoImpl,
};
use sqlx::SqlitePool;

#[cfg(feature = "mock_auth")]
type UserService = service_impl::UserServiceDev;
#[cfg(feature = "oidc")]
type UserService = service_impl::UserServiceImpl;
type PermissionService =
    service_impl::PermissionServiceImpl<dao_impl::PermissionDaoImpl, UserService>;
type ClockService = service_impl::clock::ClockServiceImpl;
type UuidService = service_impl::uuid_service::UuidServiceImpl;
type SlotService = service_impl::slot::SlotServiceImpl<
    dao_impl::slot::SlotDaoImpl,
    PermissionService,
    ClockService,
    UuidService,
>;
type SalesPersonService = service_impl::sales_person::SalesPersonServiceImpl<
    dao_impl::sales_person::SalesPersonDaoImpl,
    PermissionService,
    ClockService,
    UuidService,
>;
type BookingService = service_impl::booking::BookingServiceImpl<
    dao_impl::booking::BookingDaoImpl,
    PermissionService,
    ClockService,
    UuidService,
    SalesPersonService,
    SlotService,
>;
type ReportingService = service_impl::reporting::ReportingServiceImpl<
    dao_impl::extra_hours::ExtraHoursDaoImpl,
    dao_impl::shiftplan_report::ShiftplanReportDaoImpl,
    dao_impl::working_hours::WorkingHoursDaoImpl,
    SalesPersonService,
    PermissionService,
    ClockService,
    UuidService,
>;
type WorkingHoursService = service_impl::working_hours::WorkingHoursServiceImpl<
    dao_impl::working_hours::WorkingHoursDaoImpl,
    PermissionService,
    ClockService,
    UuidService,
>;

#[derive(Clone)]
pub struct RestStateImpl {
    user_service: Arc<UserService>,
    permission_service: Arc<PermissionService>,
    slot_service: Arc<SlotService>,
    sales_person_service: Arc<SalesPersonService>,
    booking_service: Arc<BookingService>,
    reporting_service: Arc<ReportingService>,
    working_hours_service: Arc<WorkingHoursService>,
}
impl rest::RestStateDef for RestStateImpl {
    type UserService = UserService;
    type PermissionService = PermissionService;
    type SlotService = SlotService;
    type SalesPersonService = SalesPersonService;
    type BookingService = BookingService;
    type ReportingService = ReportingService;
    type WorkingHoursService = WorkingHoursService;

    fn user_service(&self) -> Arc<Self::UserService> {
        self.user_service.clone()
    }
    fn permission_service(&self) -> Arc<Self::PermissionService> {
        self.permission_service.clone()
    }
    fn slot_service(&self) -> Arc<Self::SlotService> {
        self.slot_service.clone()
    }
    fn sales_person_service(&self) -> Arc<Self::SalesPersonService> {
        self.sales_person_service.clone()
    }
    fn booking_service(&self) -> Arc<Self::BookingService> {
        self.booking_service.clone()
    }
    fn reporting_service(&self) -> Arc<Self::ReportingService> {
        self.reporting_service.clone()
    }
    fn working_hours_service(&self) -> Arc<Self::WorkingHoursService> {
        self.working_hours_service.clone()
    }
}
impl RestStateImpl {
    pub fn new(pool: Arc<sqlx::Pool<sqlx::Sqlite>>) -> Self {
        let permission_dao = dao_impl::PermissionDaoImpl::new(pool.clone());
        let slot_dao = dao_impl::slot::SlotDaoImpl::new(pool.clone());
        let sales_person_dao = dao_impl::sales_person::SalesPersonDaoImpl::new(pool.clone());
        let booking_dao = dao_impl::booking::BookingDaoImpl::new(pool.clone());
        let extra_hours_dao = Arc::new(ExtraHoursDaoImpl::new(pool.clone()));
        let shiftplan_report_dao = Arc::new(ShiftplanReportDaoImpl::new(pool.clone()));
        let working_hours_dao = Arc::new(WorkingHoursDaoImpl::new(pool.clone()));

        // Always authenticate with DEVUSER during development.
        // This is used to test the permission service locally without a login service.
        //
        // TODO: Implement a proper authentication service when used in produciton. Maybe
        // use differnet implementations on debug then on release.  Or control it via a
        // feature.
        #[cfg(feature = "mock_auth")]
        let user_service = service_impl::UserServiceDev;
        #[cfg(feature = "oidc")]
        let user_service = service_impl::UserServiceImpl;
        let user_service = Arc::new(user_service);
        let permission_service = Arc::new(service_impl::PermissionServiceImpl::new(
            permission_dao.into(),
            user_service.clone(),
        ));
        let clock_service = Arc::new(service_impl::clock::ClockServiceImpl);
        let uuid_service = Arc::new(service_impl::uuid_service::UuidServiceImpl);
        let slot_service = Arc::new(service_impl::slot::SlotServiceImpl::new(
            slot_dao.into(),
            permission_service.clone(),
            clock_service.clone(),
            uuid_service.clone(),
        ));
        let sales_person_service =
            Arc::new(service_impl::sales_person::SalesPersonServiceImpl::new(
                sales_person_dao.into(),
                permission_service.clone(),
                clock_service.clone(),
                uuid_service.clone(),
            ));
        let booking_service = Arc::new(service_impl::booking::BookingServiceImpl::new(
            booking_dao.into(),
            permission_service.clone(),
            clock_service.clone(),
            uuid_service.clone(),
            sales_person_service.clone(),
            slot_service.clone(),
        ));
        let reporting_service = Arc::new(service_impl::reporting::ReportingServiceImpl::new(
            extra_hours_dao,
            shiftplan_report_dao,
            working_hours_dao.clone(),
            sales_person_service.clone(),
            permission_service.clone(),
            clock_service.clone(),
            uuid_service.clone(),
        ));
        let working_hours_service =
            Arc::new(service_impl::working_hours::WorkingHoursServiceImpl::new(
                working_hours_dao,
                permission_service.clone(),
                clock_service,
                uuid_service,
            ));
        Self {
            user_service,
            permission_service,
            slot_service,
            sales_person_service,
            booking_service,
            reporting_service,
            working_hours_service,
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
    dotenvy::dotenv().ok();
    let pool = Arc::new(
        SqlitePool::connect("sqlite:./localdb.sqlite3")
            .await
            .expect("Could not connect to database"),
    );

    let rest_state = RestStateImpl::new(pool.clone());
    create_dev_admin_user(pool.clone()).await;
    rest::start_server(rest_state).await
}
