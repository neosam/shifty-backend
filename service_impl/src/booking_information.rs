use std::sync::Arc;

use async_trait::async_trait;
use service::{
    booking_information::{
        build_booking_information, BookingInformation, WeeklySummary, WorkingHoursPerSalesPerson,
    },
    permission::{Authentication, SALES_PRIVILEGE, SHIFTPLANNER_PRIVILEGE},
    slot::Slot,
    special_days::SpecialDayType,
    ServiceError,
};
use tokio::join;
use uuid::Uuid;

pub struct BookingInformationServiceImpl<
    ShiftplanReportService,
    SlotService,
    BookingService,
    SalesPersonService,
    SalesPersonUnavailableService,
    ReportingService,
    SpecialDayService,
    PermissionService,
    ClockService,
    UuidService,
    TransactionDao,
> where
    ShiftplanReportService: service::shiftplan_report::ShiftplanReportService + Send + Sync,
    SlotService: service::slot::SlotService + Send + Sync,
    BookingService: service::booking::BookingService + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    SalesPersonUnavailableService:
        service::sales_person_unavailable::SalesPersonUnavailableService + Send + Sync,
    ReportingService: service::reporting::ReportingService + Send + Sync,
    SpecialDayService: service::special_days::SpecialDayService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    TransactionDao: dao::TransactionDao + Send + Sync,
{
    pub shiftplan_report_service: Arc<ShiftplanReportService>,
    pub slot_service: Arc<SlotService>,
    pub booking_service: Arc<BookingService>,
    pub sales_person_service: Arc<SalesPersonService>,
    pub sales_person_unavailable_service: Arc<SalesPersonUnavailableService>,
    pub reporting_service: Arc<ReportingService>,
    pub special_day_service: Arc<SpecialDayService>,
    pub permission_service: Arc<PermissionService>,
    pub clock_service: Arc<ClockService>,
    pub uuid_service: Arc<UuidService>,
    pub transaction_dao: Arc<TransactionDao>,
}

impl<
        ShiftplanReportService,
        SlotService,
        BookingService,
        SalesPersonService,
        SalesPersonUnavailableService,
        ReportingService,
        SpecialDayService,
        PermissionService,
        ClockService,
        UuidService,
        TransactionDao,
    >
    BookingInformationServiceImpl<
        ShiftplanReportService,
        SlotService,
        BookingService,
        SalesPersonService,
        SalesPersonUnavailableService,
        ReportingService,
        SpecialDayService,
        PermissionService,
        ClockService,
        UuidService,
        TransactionDao,
    >
where
    ShiftplanReportService: service::shiftplan_report::ShiftplanReportService + Send + Sync,
    SlotService: service::slot::SlotService + Send + Sync,
    BookingService: service::booking::BookingService + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    SalesPersonUnavailableService:
        service::sales_person_unavailable::SalesPersonUnavailableService + Send + Sync,
    ReportingService: service::reporting::ReportingService + Send + Sync,
    SpecialDayService: service::special_days::SpecialDayService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    TransactionDao: dao::TransactionDao + Send + Sync,
{
    pub fn new(
        shiftplan_report_service: Arc<ShiftplanReportService>,
        slot_service: Arc<SlotService>,
        booking_service: Arc<BookingService>,
        sales_person_service: Arc<SalesPersonService>,
        sales_person_unavailable_service: Arc<SalesPersonUnavailableService>,
        reporting_service: Arc<ReportingService>,
        special_day_service: Arc<SpecialDayService>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
        transaction_dao: Arc<TransactionDao>,
    ) -> Self {
        Self {
            shiftplan_report_service,
            slot_service,
            booking_service,
            sales_person_service,
            sales_person_unavailable_service,
            reporting_service,
            special_day_service,
            permission_service,
            clock_service,
            uuid_service,
            transaction_dao,
        }
    }
}

#[async_trait]
impl<
        ShiftplanReportService,
        SlotService,
        BookingService,
        SalesPersonService,
        SalesPersonUnavailableService,
        ReportingService,
        SpecialDayService,
        PermissionService,
        ClockService,
        UuidService,
        TransactionDao,
    > service::booking_information::BookingInformationService
    for BookingInformationServiceImpl<
        ShiftplanReportService,
        SlotService,
        BookingService,
        SalesPersonService,
        SalesPersonUnavailableService,
        ReportingService,
        SpecialDayService,
        PermissionService,
        ClockService,
        UuidService,
        TransactionDao,
    >
where
    ShiftplanReportService: service::shiftplan_report::ShiftplanReportService + Send + Sync,
    SlotService: service::slot::SlotService<
            Context = PermissionService::Context,
            Transaction = TransactionDao::Transaction,
        > + Send
        + Sync,
    BookingService: service::booking::BookingService<
            Context = PermissionService::Context,
            Transaction = TransactionDao::Transaction,
        > + Send
        + Sync,
    SalesPersonService: service::sales_person::SalesPersonService<Context = PermissionService::Context>
        + Send
        + Sync,
    SalesPersonUnavailableService:
        service::sales_person_unavailable::SalesPersonUnavailableService + Send + Sync,
    ReportingService: service::reporting::ReportingService + Send + Sync,
    SpecialDayService: service::special_days::SpecialDayService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    TransactionDao: dao::TransactionDao + Send + Sync,
{
    type Context = PermissionService::Context;
    type Transaction = TransactionDao::Transaction;

    async fn get_booking_conflicts_for_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[BookingInformation]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;
        let bookings = self
            .booking_service
            .get_for_week(week, year, Authentication::Full, Some(tx.clone()))
            .await?;
        let sales_persons = self
            .sales_person_service
            .get_all(Authentication::Full)
            .await?;
        let slots = self
            .slot_service
            .get_slots(Authentication::Full, tx.clone().into())
            .await?;
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

        self.transaction_dao.commit(tx).await?;
        Ok(conflicts)
    }

    async fn get_weekly_summary(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[WeeklySummary]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (shiftplanner, sales) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context.clone())
        );
        shiftplanner.or(sales)?;

        let is_shiftplanner = self
            .permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
            .await
            .is_ok();

        let mut weekly_report = vec![];
        let weeks_in_year = time::util::weeks_in_year(year as i32);
        let volunteer_ids: Arc<[Uuid]> = self
            .sales_person_service
            .get_all(Authentication::Full)
            .await?
            .iter()
            .filter(|sales_person| !sales_person.is_paid.unwrap_or(false))
            .map(|sales_person| sales_person.id)
            .collect();
        for week in 1..=(weeks_in_year + 3) {
            let (year, week) = if week > weeks_in_year as u8 {
                (year + 1, week - weeks_in_year as u8)
            } else {
                (year, week)
            };
            let mut working_hours_per_sales_person = vec![];
            let week_report = self
                .reporting_service
                .get_week(year, week, Authentication::Full)
                .await?;
            let special_days = self
                .special_day_service
                .get_by_week(year, week, Authentication::Full)
                .await?;
            let volunteer_hours = self
                .shiftplan_report_service
                .extract_shiftplan_report_for_week(year, week, Authentication::Full)
                .await?
                .iter()
                .filter(|report| volunteer_ids.iter().any(|id| *id == report.sales_person_id))
                .map(|report| report.hours)
                .sum::<f32>();
            let slots: Arc<[Slot]> = self
                .slot_service
                .get_slots_for_week(year, week, Authentication::Full, tx.clone().into())
                .await?
                .iter()
                .filter(|slot| {
                    !special_days.iter().any(|day| {
                        day.day_of_week == slot.day_of_week
                            && (day.day_type == SpecialDayType::Holiday
                                || day.day_type == SpecialDayType::ShortDay
                                    && day.time_of_day.is_some()
                                    && slot.to > day.time_of_day.unwrap())
                    })
                })
                .cloned()
                .collect();
            let slot_hours = slots
                .iter()
                .map(|slot| {
                    (slot.to - slot.from).as_seconds_f32() / 3600.0 * slot.min_resources as f32
                })
                .sum::<f32>();
            let mut overall_available_hours = volunteer_hours;
            for report in week_report.iter() {
                overall_available_hours += report.expected_hours;
                if is_shiftplanner {
                    working_hours_per_sales_person.push(WorkingHoursPerSalesPerson {
                        sales_person_id: report.sales_person.id,
                        sales_person_name: report.sales_person.name.clone(),
                        available_hours: report.expected_hours,
                    });
                }
            }
            weekly_report.push(WeeklySummary {
                year,
                week,
                overall_available_hours,
                working_hours_per_sales_person: working_hours_per_sales_person.into(),
                required_hours: slot_hours,
            });
        }

        self.transaction_dao.commit(tx).await?;
        Ok(weekly_report.into())
    }
}
