#[cfg(test)]
mod integration_test;

use std::sync::Arc;

use dao_impl::{
    carryover, employee_work_details::EmployeeWorkDetailsDaoImpl, extra_hours::ExtraHoursDaoImpl,
    shiftplan_report::ShiftplanReportDaoImpl,
};
#[cfg(feature = "mock_auth")]
use service::permission::MockContext;
use service::{scheduler::SchedulerService, shiftplan::ShiftplanService};
use service_impl::{carryover::CarryoverServiceDeps, permission::PermissionServiceDeps, shiftplan::ShiftplanServiceDeps};
use sqlx::SqlitePool;
use tracing_subscriber::fmt::format::FmtSpan;

#[cfg(feature = "mock_auth")]
type UserService = service_impl::UserServiceDev;
#[cfg(feature = "mock_auth")]
type Context = MockContext;
#[cfg(feature = "oidc")]
type UserService = service_impl::UserServiceImpl;
#[cfg(feature = "oidc")]
type Context = Option<Arc<str>>;
type Transaction = dao_impl::TransactionImpl;
type TransactionDao = dao_impl::TransactionDaoImpl;
pub struct PermissionServiceDependencies;
impl PermissionServiceDeps for PermissionServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type PermissionDao = dao_impl::PermissionDaoImpl;
    type UserService = UserService;
}
type PermissionService = service_impl::PermissionServiceImpl<PermissionServiceDependencies>;

type SessionDao = dao_impl::session::SessionDaoImpl;
type ShiftplanReportDao = dao_impl::shiftplan_report::ShiftplanReportDaoImpl;
pub struct SessionServiceDependencies;
impl service_impl::session::SessionServiceDeps for SessionServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type SessionDao = SessionDao;
    type ClockService = service_impl::clock::ClockServiceImpl;
    type UuidService = service_impl::uuid_service::UuidServiceImpl;
}
type SessionService = service_impl::session::SessionServiceImpl<SessionServiceDependencies>;

type ClockService = service_impl::clock::ClockServiceImpl;
type UuidService = service_impl::uuid_service::UuidServiceImpl;
type SlotService = service_impl::slot::SlotServiceImpl<
    dao_impl::slot::SlotDaoImpl,
    PermissionService,
    ClockService,
    UuidService,
    TransactionDao,
>;

pub struct SalesPersonServiceDependencies;
impl service_impl::sales_person::SalesPersonServiceDeps for SalesPersonServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type SalesPersonDao = dao_impl::sales_person::SalesPersonDaoImpl;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type SalesPersonService =
    service_impl::sales_person::SalesPersonServiceImpl<SalesPersonServiceDependencies>;
type SpecialDayService = service_impl::special_days::SpecialDayServiceImpl<
    dao_impl::special_day::SpecialDayDaoImpl,
    PermissionService,
    ClockService,
    UuidService,
>;

pub struct SalesPersonUnavailableServiceDependencies;
impl service_impl::sales_person_unavailable::SalesPersonUnavailableServiceDeps
    for SalesPersonUnavailableServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type SalesPersonUnavailableDao =
        dao_impl::sales_person_unavailable::SalesPersonUnavailableDaoImpl;
    type SalesPersonService = SalesPersonService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type SalesPersonUnavailableService =
    service_impl::sales_person_unavailable::SalesPersonUnavailableServiceImpl<
        SalesPersonUnavailableServiceDependencies,
    >;
pub struct BookingServiceDependencies;
impl service_impl::booking::BookingServiceDeps for BookingServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type BookingDao = dao_impl::booking::BookingDaoImpl;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type SalesPersonService = SalesPersonService;
    type SlotService = SlotService;
    type TransactionDao = TransactionDao;
}
type BookingService = service_impl::booking::BookingServiceImpl<BookingServiceDependencies>;

pub struct ShiftplanReportServiceDependencies;
impl service_impl::shiftplan_report::ShiftplanReportServiceDeps
    for ShiftplanReportServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type ShiftplanReportDao = ShiftplanReportDao;
    type TransactionDao = TransactionDao;
}
type ShiftplanReportService =
    service_impl::shiftplan_report::ShiftplanReportServiceImpl<ShiftplanReportServiceDependencies>;

pub struct BookingInformationServiceDependencies;
impl service_impl::booking_information::BookingInformationServiceDeps
    for BookingInformationServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type ShiftplanReportService = ShiftplanReportService;
    type SlotService = SlotService;
    type BookingService = BookingService;
    type SalesPersonService = SalesPersonService;
    type SalesPersonUnavailableService = SalesPersonUnavailableService;
    type ReportingService = ReportingService;
    type SpecialDayService = SpecialDayService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type BookingInformationService = service_impl::booking_information::BookingInformationServiceImpl<
    BookingInformationServiceDependencies,
>;

pub struct ExtraHoursServiceDependencies;
impl service_impl::extra_hours::ExtraHoursServiceDeps for ExtraHoursServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ExtraHoursDao = dao_impl::extra_hours::ExtraHoursDaoImpl;
    type PermissionService = PermissionService;
    type SalesPersonService = SalesPersonService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type ExtraHoursService =
    service_impl::extra_hours::ExtraHoursServiceImpl<ExtraHoursServiceDependencies>;

type CarryoverDao = dao_impl::carryover::CarryoverDaoImpl;

pub struct CarryoverServiceDependencies;
impl CarryoverServiceDeps for CarryoverServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type CarryoverDao = CarryoverDao;
    type TransactionDao = TransactionDao;
}

type CarryoverService = service_impl::carryover::CarryoverServiceImpl<CarryoverServiceDependencies>;

type IcalService = service_impl::ical::IcalServiceImpl;

pub struct ShiftplanServiceDependencies;
impl ShiftplanServiceDeps for ShiftplanServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type SlotService = SlotService;
    type BookingService = BookingService;
    type SalesPersonService = SalesPersonService;
    type TransactionDao = TransactionDao;
}

pub struct BlockServiceDependencies;
impl service_impl::block::BlockServiceDeps for BlockServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type SlotService = SlotService;
    type BookingService = BookingService;
    type SalesPersonService = SalesPersonService;
    type ClockService = ClockService;
    type IcalService = IcalService;
    type TransactionDao = TransactionDao;
}
type BlockService = service_impl::block::BlockServiceImpl<BlockServiceDependencies>;

pub struct ReportingServiceDependencies;
impl service_impl::reporting::ReportingServiceDeps for ReportingServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ExtraHoursService = ExtraHoursService;
    type ShiftplanReportService = ShiftplanReportService;
    type EmployeeWorkDetailsService = WorkingHoursService;
    type SalesPersonService = SalesPersonService;
    type CarryoverService = CarryoverService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type ReportingService = service_impl::reporting::ReportingServiceImpl<ReportingServiceDependencies>;

pub struct WorkingHoursServiceDependencies;
impl service_impl::employee_work_details::EmployeeWorkDetailsServiceDeps
    for WorkingHoursServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type EmployeeWorkDetailsDao = EmployeeWorkDetailsDaoImpl;
    type SalesPersonService = SalesPersonService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type WorkingHoursService = service_impl::employee_work_details::EmployeeWorkDetailsServiceImpl<
    WorkingHoursServiceDependencies,
>;
pub struct ShiftplanEditServiceDependencies;
impl service_impl::shiftplan_edit::ShiftplanEditServiceDeps for ShiftplanEditServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type PermissionService = PermissionService;
    type SlotService = SlotService;
    type BookingService = BookingService;
    type CarryoverService = CarryoverService;
    type ReportingService = ReportingService;
    type SalesPersonService = SalesPersonService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
    type EmployeeWorkDetailsService = WorkingHoursService;
    type ExtraHoursService = ExtraHoursService;
    type SalesPersonUnavailableService = SalesPersonUnavailableService;
}
type ShiftplanEditService =
    service_impl::shiftplan_edit::ShiftplanEditServiceImpl<ShiftplanEditServiceDependencies>;

pub struct SchedulerServiceDependencies;
impl service_impl::scheduler::SchedulerServiceDeps for SchedulerServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ShiftplanEditService = ShiftplanEditService;
}
type SchedulerServiceImpl =
    service_impl::scheduler::SchedulerServiceImpl<SchedulerServiceDependencies>;

#[derive(Clone)]
pub struct RestStateImpl {
    user_service: Arc<UserService>,
    session_service: Arc<SessionService>,
    permission_service: Arc<PermissionService>,
    slot_service: Arc<SlotService>,
    sales_person_service: Arc<SalesPersonService>,
    special_day_service: Arc<SpecialDayService>,
    sales_person_unavailable_service: Arc<SalesPersonUnavailableService>,
    booking_service: Arc<BookingService>,
    booking_information_service: Arc<BookingInformationService>,
    reporting_service: Arc<ReportingService>,
    working_hours_service: Arc<WorkingHoursService>,
    extra_hours_service: Arc<ExtraHoursService>,
    shiftplan_edit_service: Arc<ShiftplanEditService>,
    block_service: Arc<BlockService>,
    shiftplan_service: Arc<ShiftplanServiceImpl<ShiftplanServiceDependencies>>,
}
impl rest::RestStateDef for RestStateImpl {
    type UserService = UserService;
    type SessionService = SessionService;
    type PermissionService = PermissionService;
    type SlotService = SlotService;
    type SalesPersonService = SalesPersonService;
    type SpecialDayService = SpecialDayService;
    type SalesPersonUnavailableService = SalesPersonUnavailableService;
    type BookingService = BookingService;
    type BookingInformationService = BookingInformationService;
    type ReportingService = ReportingService;
    type WorkingHoursService = WorkingHoursService;
    type ExtraHoursService = ExtraHoursService;
    type ShiftplanEditService = ShiftplanEditService;
    type BlockService = BlockService;
    type ShiftplanService = ShiftplanServiceImpl<ShiftplanServiceDependencies>;

    fn backend_version(&self) -> Arc<str> {
        Arc::from(env!("CARGO_PKG_VERSION"))
    }

    fn user_service(&self) -> Arc<Self::UserService> {
        self.user_service.clone()
    }
    fn session_service(&self) -> Arc<Self::SessionService> {
        self.session_service.clone()
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
    fn special_day_service(&self) -> Arc<Self::SpecialDayService> {
        self.special_day_service.clone()
    }
    fn sales_person_unavailable_service(&self) -> Arc<Self::SalesPersonUnavailableService> {
        self.sales_person_unavailable_service.clone()
    }
    fn booking_service(&self) -> Arc<Self::BookingService> {
        self.booking_service.clone()
    }
    fn booking_information_service(&self) -> Arc<Self::BookingInformationService> {
        self.booking_information_service.clone()
    }
    fn reporting_service(&self) -> Arc<Self::ReportingService> {
        self.reporting_service.clone()
    }
    fn working_hours_service(&self) -> Arc<Self::WorkingHoursService> {
        self.working_hours_service.clone()
    }
    fn extra_hours_service(&self) -> Arc<Self::ExtraHoursService> {
        self.extra_hours_service.clone()
    }
    fn shiftplan_edit_service(&self) -> Arc<Self::ShiftplanEditService> {
        self.shiftplan_edit_service.clone()
    }
    fn block_service(&self) -> Arc<Self::BlockService> {
        self.block_service.clone()
    }

    fn shiftplan_service(&self) -> Arc<Self::ShiftplanService> {
        self.shiftplan_service.clone()
    }
}
impl RestStateImpl {
    pub fn new(pool: Arc<sqlx::Pool<sqlx::Sqlite>>) -> Self {
        let transaction_dao = Arc::new(TransactionDao::new(pool.clone()));
        let permission_dao = dao_impl::PermissionDaoImpl::new(pool.clone());
        let slot_dao = dao_impl::slot::SlotDaoImpl::new(pool.clone());
        let carryover_dao = Arc::new(carryover::CarryoverDaoImpl::new(pool.clone()));
        let sales_person_dao = dao_impl::sales_person::SalesPersonDaoImpl::new(pool.clone());
        let booking_dao = dao_impl::booking::BookingDaoImpl::new(pool.clone());
        let extra_hours_dao = Arc::new(ExtraHoursDaoImpl::new(pool.clone()));
        let shiftplan_report_dao = Arc::new(ShiftplanReportDaoImpl::new(pool.clone()));
        let working_hours_dao = Arc::new(EmployeeWorkDetailsDaoImpl::new(pool.clone()));
        let special_day_dao = dao_impl::special_day::SpecialDayDaoImpl::new(pool.clone());
        let session_dao = SessionDao::new(pool.clone());

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
        let permission_service = Arc::new(service_impl::PermissionServiceImpl {
            permission_dao: permission_dao.into(),
            user_service: user_service.clone(),
        });
        let clock_service = Arc::new(service_impl::clock::ClockServiceImpl);
        let uuid_service = Arc::new(service_impl::uuid_service::UuidServiceImpl);
        let session_service = Arc::new(service_impl::session::SessionServiceImpl {
            session_dao: Arc::new(session_dao),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
        });
        let slot_service = Arc::new(service_impl::slot::SlotServiceImpl::new(
            slot_dao.into(),
            permission_service.clone(),
            clock_service.clone(),
            uuid_service.clone(),
            transaction_dao.clone(),
        ));
        let sales_person_service = Arc::new(service_impl::sales_person::SalesPersonServiceImpl {
            sales_person_dao: sales_person_dao.into(),
            permission_service: permission_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });
        let special_day_service = Arc::new(service_impl::special_days::SpecialDayServiceImpl::new(
            special_day_dao.into(),
            permission_service.clone(),
            clock_service.clone(),
            uuid_service.clone(),
        ));
        let sales_person_unavailable_service = Arc::new(
            service_impl::sales_person_unavailable::SalesPersonUnavailableServiceImpl {
                sales_person_unavailable_dao: Arc::new(
                    dao_impl::sales_person_unavailable::SalesPersonUnavailableDaoImpl::new(pool),
                ),
                sales_person_service: sales_person_service.clone(),
                permission_service: permission_service.clone(),
                clock_service: clock_service.clone(),
                uuid_service: uuid_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );
        let booking_service = Arc::new(service_impl::booking::BookingServiceImpl {
            transaction_dao: transaction_dao.clone(),
            booking_dao: booking_dao.into(),
            permission_service: permission_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            sales_person_service: sales_person_service.clone(),
            slot_service: slot_service.clone(),
        });
        let extra_hours_service = Arc::new(service_impl::extra_hours::ExtraHoursServiceImpl {
            extra_hours_dao: extra_hours_dao,
            permission_service: permission_service.clone(),
            sales_person_service: sales_person_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });
        let working_hours_service = Arc::new(
            service_impl::employee_work_details::EmployeeWorkDetailsServiceImpl {
                employee_work_details_dao: working_hours_dao,
                sales_person_service: sales_person_service.clone(),
                permission_service: permission_service.clone(),
                clock_service: clock_service.clone(),
                uuid_service: uuid_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );
        let shiftplan_report_service = Arc::new(ShiftplanReportService {
            shiftplan_report_dao: shiftplan_report_dao.clone(),
            transaction_dao: transaction_dao.clone(),
        });
        let carryover_service = Arc::new(service_impl::carryover::CarryoverServiceImpl {
            carryover_dao,
            transaction_dao: transaction_dao.clone(),
        });
        let reporting_service = Arc::new(service_impl::reporting::ReportingServiceImpl {
            extra_hours_service: extra_hours_service.clone(),
            shiftplan_report_service: shiftplan_report_service.clone(),
            employee_work_details_service: working_hours_service.clone(),
            sales_person_service: sales_person_service.clone(),
            carryover_service: carryover_service.clone(),
            permission_service: permission_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });

        let booking_information_service = Arc::new(
            service_impl::booking_information::BookingInformationServiceImpl {
                shiftplan_report_service: shiftplan_report_service.clone(),
                slot_service: slot_service.clone(),
                booking_service: booking_service.clone(),
                sales_person_service: sales_person_service.clone(),
                sales_person_unavailable_service: sales_person_unavailable_service.clone(),
                reporting_service: reporting_service.clone(),
                special_day_service: special_day_service.clone(),
                permission_service: permission_service.clone(),
                clock_service: clock_service.clone(),
                uuid_service: uuid_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );
        let shiftplan_edit_service =
            Arc::new(service_impl::shiftplan_edit::ShiftplanEditServiceImpl {
                permission_service: permission_service.clone(),
                slot_service: slot_service.clone(),
                booking_service: booking_service.clone(),
                sales_person_service: sales_person_service.clone(),
                employee_work_details_service: working_hours_service.clone(),
                carryover_service: carryover_service.clone(),
                reporting_service: reporting_service.clone(),
                uuid_service: uuid_service.clone(),
                transaction_dao: transaction_dao.clone(),
                extra_hours_service: extra_hours_service.clone(),
                sales_person_unavailable_service: sales_person_unavailable_service.clone(),
            });
        let shiftplan_service = Arc::new(service_impl::shiftplan::ShiftplanServiceImpl {
            slot_service: slot_service.clone(),
            booking_service: booking_service.clone(),
            sales_person_service: sales_person_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });

        let block_service = Arc::new(service_impl::block::BlockServiceImpl {
            slot_service: slot_service.clone(),
            booking_service: booking_service.clone(),
            sales_person_service: sales_person_service.clone(),
            clock_service: clock_service.clone(),
            ical_service: Arc::new(service_impl::ical::IcalServiceImpl),
            transaction_dao: transaction_dao.clone(),
        });
        Self {
            user_service,
            session_service,
            permission_service,
            slot_service,
            sales_person_service,
            special_day_service,
            sales_person_unavailable_service,
            booking_service,
            booking_information_service,
            reporting_service,
            working_hours_service,
            extra_hours_service,
            shiftplan_edit_service,
            block_service,
            shiftplan_service,
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
    let version = env!("CARGO_PKG_VERSION");

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .json()
        .with_span_events(FmtSpan::CLOSE)
        .with_span_list(true)
        .with_file(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::info!("Shifty backend version: {}", version);
    dotenvy::dotenv().ok();
    let pool = Arc::new(
        SqlitePool::connect("sqlite:./localdb.sqlite3")
            .await
            .expect("Could not connect to database"),
    );

    let rest_state = RestStateImpl::new(pool.clone());
    create_dev_admin_user(pool.clone()).await;

    let scheduler_service = SchedulerServiceImpl::new(rest_state.shiftplan_edit_service.clone());
    scheduler_service
        .start()
        .await
        .expect("Expected the scheduler to start");

    rest::start_server(rest_state).await
}
