use std::sync::Arc;

use async_trait::async_trait;
use dao::working_hours;
use service::{
    booking_information::{
        build_booking_information, BookingInformation, WeeklySummary, WorkingHoursPerSalesPerson,
    },
    permission::{Authentication, SHIFTPLANNER_PRIVILEGE},
    reporting::{self, ReportingService},
    ServiceError,
};

pub struct BookingInformationServiceImpl<
    SlotService,
    BookingService,
    SalesPersonService,
    SalesPersonUnavailableService,
    ReportingService,
    PermissionService,
    ClockService,
    UuidService,
> where
    SlotService: service::slot::SlotService + Send + Sync,
    BookingService: service::booking::BookingService + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    SalesPersonUnavailableService:
        service::sales_person_unavailable::SalesPersonUnavailableService + Send + Sync,
    ReportingService: service::reporting::ReportingService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    pub slot_service: Arc<SlotService>,
    pub booking_service: Arc<BookingService>,
    pub sales_person_service: Arc<SalesPersonService>,
    pub sales_person_unavailable_service: Arc<SalesPersonUnavailableService>,
    pub reporting_service: Arc<ReportingService>,
    pub permission_service: Arc<PermissionService>,
    pub clock_service: Arc<ClockService>,
    pub uuid_service: Arc<UuidService>,
}

impl<
        SlotService,
        BookingService,
        SalesPersonService,
        SalesPersonUnavailableService,
        ReportingService,
        PermissionService,
        ClockService,
        UuidService,
    >
    BookingInformationServiceImpl<
        SlotService,
        BookingService,
        SalesPersonService,
        SalesPersonUnavailableService,
        ReportingService,
        PermissionService,
        ClockService,
        UuidService,
    >
where
    SlotService: service::slot::SlotService + Send + Sync,
    BookingService: service::booking::BookingService + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    SalesPersonUnavailableService:
        service::sales_person_unavailable::SalesPersonUnavailableService + Send + Sync,
    ReportingService: service::reporting::ReportingService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    pub fn new(
        slot_service: Arc<SlotService>,
        booking_service: Arc<BookingService>,
        sales_person_service: Arc<SalesPersonService>,
        sales_person_unavailable_service: Arc<SalesPersonUnavailableService>,
        reporting_service: Arc<ReportingService>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            slot_service,
            booking_service,
            sales_person_service,
            sales_person_unavailable_service,
            reporting_service,
            permission_service,
            clock_service,
            uuid_service,
        }
    }
}

#[async_trait]
impl<
        SlotService,
        BookingService,
        SalesPersonService,
        SalesPersonUnavailableService,
        ReportingService,
        PermissionService,
        ClockService,
        UuidService,
    > service::booking_information::BookingInformationService
    for BookingInformationServiceImpl<
        SlotService,
        BookingService,
        SalesPersonService,
        SalesPersonUnavailableService,
        ReportingService,
        PermissionService,
        ClockService,
        UuidService,
    >
where
    SlotService: service::slot::SlotService<Context = PermissionService::Context> + Send + Sync,
    BookingService:
        service::booking::BookingService<Context = PermissionService::Context> + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService<Context = PermissionService::Context>
        + Send
        + Sync,
    SalesPersonUnavailableService:
        service::sales_person_unavailable::SalesPersonUnavailableService + Send + Sync,
    ReportingService: service::reporting::ReportingService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    type Context = PermissionService::Context;

    async fn get_booking_conflicts_for_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[BookingInformation]>, ServiceError> {
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;
        let bookings = self
            .booking_service
            .get_for_week(week, year, Authentication::Full)
            .await?;
        let sales_persons = self
            .sales_person_service
            .get_all(Authentication::Full)
            .await?;
        let slots = self.slot_service.get_slots(Authentication::Full).await?;
        let unavailable_entries = self
            .sales_person_unavailable_service
            .get_by_week(year, week, Authentication::Full)
            .await?;
        let booking_informations = build_booking_information(slots, bookings, sales_persons);
        let conflicts = booking_informations
            .iter()
            .filter(|booking_information| {
                unavailable_entries.iter().any(|unavailable| {
                    unavailable.sales_person_id == booking_information.sales_person.id
                        && unavailable.day_of_week == booking_information.slot.day_of_week
                })
            })
            .cloned()
            .collect();

        Ok(conflicts)
    }

    async fn get_weekly_summary(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[WeeklySummary]>, ServiceError> {
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;
        let mut weekly_report = vec![];
        for week in 1..=time::util::weeks_in_year(year as i32) {
            let mut overall_available_hours = 0.0;
            let mut working_hours_per_sales_person = vec![];
            let week_report = self
                .reporting_service
                .get_week(year, week, Authentication::Full)
                .await?;
            let slots = self.slot_service.get_slots(Authentication::Full).await?;
            let slot_hours = slots
                .iter()
                .map(|slot| {
                    (slot.to - slot.from).as_seconds_f32() / 3600.0 * slot.min_resources as f32
                })
                .sum::<f32>();
            for report in week_report.iter() {
                overall_available_hours += report.expected_hours;
                working_hours_per_sales_person.push(WorkingHoursPerSalesPerson {
                    sales_person_id: report.sales_person.id,
                    sales_person_name: report.sales_person.name.clone(),
                    available_hours: report.expected_hours,
                });
            }
            weekly_report.push(WeeklySummary {
                year,
                week,
                overall_available_hours,
                working_hours_per_sales_person: working_hours_per_sales_person.into(),
                required_hours: slot_hours,
            });
        }

        Ok(weekly_report.into())
    }
}
