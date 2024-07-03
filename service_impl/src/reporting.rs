use std::sync::Arc;

use async_trait::async_trait;
use dao::{shiftplan_report::ShiftplanReportEntity, working_hours::WorkingHoursEntity};
use service::{
    extra_hours::{ExtraHours, ExtraHoursCategory, ReportType},
    permission::{Authentication, HR_PRIVILEGE},
    reporting::{
        EmployeeReport, ExtraHoursReportCategory, ShortEmployeeReport, WorkingHours,
        WorkingHoursDay,
    },
    ServiceError,
};
use tokio::join;
use uuid::Uuid;

pub struct ReportingServiceImpl<
    ExtraHoursService,
    ShiftplanReportDao,
    WorkingHoursDao,
    SalesPersonService,
    PermissionService,
    ClockService,
    UuidService,
> where
    ExtraHoursService: service::extra_hours::ExtraHoursService + Send + Sync,
    ShiftplanReportDao: dao::shiftplan_report::ShiftplanReportDao + Send + Sync,
    WorkingHoursDao: dao::working_hours::WorkingHoursDao + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    pub extra_hours_service: Arc<ExtraHoursService>,
    pub shiftplan_report_dao: Arc<ShiftplanReportDao>,
    pub working_hours_dao: Arc<WorkingHoursDao>,
    pub sales_person_service: Arc<SalesPersonService>,
    pub permission_service: Arc<PermissionService>,
    pub clock_service: Arc<ClockService>,
    pub uuid_service: Arc<UuidService>,
}

impl<
        ExtraHoursService,
        ShiftplanReportDao,
        WorkingHoursDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
    ReportingServiceImpl<
        ExtraHoursService,
        ShiftplanReportDao,
        WorkingHoursDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
where
    ExtraHoursService: service::extra_hours::ExtraHoursService + Send + Sync,
    ShiftplanReportDao: dao::shiftplan_report::ShiftplanReportDao + Send + Sync,
    WorkingHoursDao: dao::working_hours::WorkingHoursDao + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    pub fn new(
        extra_hours_service: Arc<ExtraHoursService>,
        shiftplan_report_dao: Arc<ShiftplanReportDao>,
        working_hours_dao: Arc<WorkingHoursDao>,
        sales_person_service: Arc<SalesPersonService>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            extra_hours_service,
            shiftplan_report_dao,
            working_hours_dao,
            sales_person_service,
            permission_service,
            clock_service,
            uuid_service,
        }
    }
}

pub fn find_working_hours_for_calendar_week(
    working_hours: &[WorkingHoursEntity],
    year: u32,
    week: u8,
) -> Option<&WorkingHoursEntity> {
    working_hours.iter().find(|wh| {
        (year, week) >= (wh.from_year, wh.from_calendar_week)
            && (year, week) <= (wh.to_year, wh.to_calendar_week)
    })
}

#[async_trait]
impl<
        ExtraHoursService,
        ShiftplanReportDao,
        WorkingHoursDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    > service::reporting::ReportingService
    for ReportingServiceImpl<
        ExtraHoursService,
        ShiftplanReportDao,
        WorkingHoursDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
where
    ExtraHoursService: service::extra_hours::ExtraHoursService + Send + Sync,
    ShiftplanReportDao: dao::shiftplan_report::ShiftplanReportDao + Send + Sync,
    WorkingHoursDao: dao::working_hours::WorkingHoursDao + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService<Context = PermissionService::Context>
        + Send
        + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    type Context = PermissionService::Context;

    async fn get_reports_for_all_employees(
        &self,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let shiftplan_report = self
            .shiftplan_report_dao
            .extract_quick_shiftplan_report(year, until_week)
            .await?;

        let working_hours = self.working_hours_dao.all().await?;

        let employees = self
            .sales_person_service
            .get_all(Authentication::Full)
            .await?;
        let mut short_employee_report: Vec<ShortEmployeeReport> = Vec::new();
        for paid_employee in employees
            .iter()
            .filter(|employee| employee.is_paid.unwrap_or(false))
        {
            let shiftplan_hours = shiftplan_report
                .iter()
                .filter(|r| r.sales_person_id == paid_employee.id)
                .map(|r| r.hours)
                .sum::<f32>();
            let working_hours: Arc<[WorkingHoursEntity]> = working_hours
                .iter()
                .filter(|wh| wh.sales_person_id == paid_employee.id)
                .cloned()
                .collect();
            let planned_hours: f32 = (1..=until_week)
                .map(|week| {
                    find_working_hours_for_calendar_week(&working_hours, year, week)
                        .map(|wh| wh.expected_hours)
                        .unwrap_or(0.0)
                })
                .sum();
            let extra_hours = self
                .extra_hours_service
                .find_by_sales_person_id_and_year(
                    paid_employee.id,
                    year,
                    until_week,
                    Authentication::Full,
                )
                .await?
                .iter()
                .map(|eh| eh.amount)
                .sum::<f32>();
            let balance_hours = shiftplan_hours + extra_hours - planned_hours;
            short_employee_report.push(ShortEmployeeReport {
                sales_person: Arc::new(paid_employee.clone()),
                balance_hours,
            });
        }
        Ok(short_employee_report.into())
    }

    async fn get_report_for_employee(
        &self,
        sales_person_id: &Uuid,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
    ) -> Result<EmployeeReport, ServiceError> {
        let (hr_permission, user_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service
                .verify_user_is_sales_person(*sales_person_id, context.clone())
        );
        hr_permission.or(user_permission)?;

        let sales_person = self
            .sales_person_service
            .get(*sales_person_id, context)
            .await?;
        let working_hours = self
            .working_hours_dao
            .find_by_sales_person_id(*sales_person_id)
            .await?;
        let shiftplan_report = self
            .shiftplan_report_dao
            .extract_shiftplan_report(*sales_person_id, year, until_week)
            .await?;
        let extra_hours = self
            .extra_hours_service
            .find_by_sales_person_id_and_year(
                *sales_person_id,
                year,
                until_week,
                Authentication::Full,
            )
            .await?;

        let planned_hours: f32 = (1..=until_week)
            .map(|week| {
                find_working_hours_for_calendar_week(&working_hours, year, week)
                    .map(|wh| wh.expected_hours)
                    .unwrap_or(0.0)
            })
            .sum();
        let shiftplan_hours = shiftplan_report.iter().map(|r| r.hours).sum::<f32>() as f32;
        let overall_extra_work_hours = extra_hours
            .iter()
            .filter(|eh| {
                eh.date_time.iso_week() <= until_week
                    && eh.category.as_report_type() == ReportType::WorkingHours
            })
            .map(|eh| eh.amount)
            .sum::<f32>();
        let overall_absense_hours = extra_hours
            .iter()
            .filter(|eh| {
                eh.date_time.iso_week() <= until_week
                    && eh.category.as_report_type() == ReportType::AbsenceHours
            })
            .map(|eh| eh.amount)
            .sum::<f32>();

        let employee_report = EmployeeReport {
            sales_person: Arc::new(sales_person),
            balance_hours: shiftplan_hours + overall_extra_work_hours - planned_hours
                + overall_absense_hours,
            overall_hours: shiftplan_hours + overall_extra_work_hours,
            expected_hours: planned_hours - overall_absense_hours,
            shiftplan_hours,
            extra_work_hours: extra_hours
                .iter()
                .filter(|extra_hours| extra_hours.category == ExtraHoursCategory::ExtraWork)
                .map(|extra_hours| extra_hours.amount)
                .sum(),
            vacation_hours: extra_hours
                .iter()
                .filter(|extra_hours| extra_hours.category == ExtraHoursCategory::Vacation)
                .map(|extra_hours| extra_hours.amount)
                .sum(),
            sick_leave_hours: extra_hours
                .iter()
                .filter(|extra_hours| extra_hours.category == ExtraHoursCategory::SickLeave)
                .map(|extra_hours| extra_hours.amount)
                .sum(),
            holiday_hours: extra_hours
                .iter()
                .filter(|extra_hours| extra_hours.category == ExtraHoursCategory::Holiday)
                .map(|extra_hours| extra_hours.amount)
                .sum(),
            by_week: hours_per_week(
                &shiftplan_report,
                &extra_hours,
                &working_hours,
                year,
                until_week,
            )?,
            by_month: Arc::new([]),
        };

        Ok(employee_report)
    }
}

fn hours_per_week(
    shiftplan_hours_list: &Arc<[ShiftplanReportEntity]>,
    extra_hours_list: &Arc<[ExtraHours]>,
    working_hours: &[WorkingHoursEntity],
    year: u32,
    week_until: u8,
) -> Result<Arc<[WorkingHours]>, ServiceError> {
    let mut weeks: Vec<WorkingHours> = Vec::new();
    for week in 1..=week_until {
        let filtered_extra_hours_list = extra_hours_list
            .iter()
            .filter(|eh| eh.date_time.iso_week() == week && eh.date_time.year() == year as i32)
            .collect::<Vec<_>>();
        let shiftplan_hours = shiftplan_hours_list
            .iter()
            .filter(|r| r.calendar_week == week)
            .map(|r| r.hours)
            .sum::<f32>();
        let working_hours = working_hours
            .iter()
            .filter(|wh| wh.from_calendar_week <= week && wh.to_calendar_week >= week)
            .map(|wh| wh.expected_hours)
            .sum::<f32>();
        let extra_work_hours = filtered_extra_hours_list
            .iter()
            .filter(|eh| eh.category.as_report_type() == ReportType::WorkingHours)
            .map(|eh| eh.amount)
            .sum::<f32>();
        let absence_hours = filtered_extra_hours_list
            .iter()
            .filter(|eh| eh.category.as_report_type() == ReportType::AbsenceHours)
            .map(|eh| eh.amount)
            .sum::<f32>();

        let mut day_list = extra_hours_list
            .iter()
            .map(|eh| {
                Ok(WorkingHoursDay {
                    date: eh.date_time.date(),
                    hours: eh.amount,
                    category: (&eh.category).into(),
                })
            })
            .chain(shiftplan_hours_list.iter().map(|working_hours_day| {
                Ok::<WorkingHoursDay, ServiceError>(WorkingHoursDay {
                    date: time::Date::from_iso_week_date(
                        year as i32,
                        working_hours_day.calendar_week,
                        time::Weekday::Sunday.nth_next(working_hours_day.day_of_week.to_number()),
                    )?,
                    hours: working_hours_day.hours,
                    category: ExtraHoursReportCategory::Shiftplan,
                })
            }))
            .collect::<Result<Vec<WorkingHoursDay>, ServiceError>>()?;
        day_list.sort_by_key(|day| day.date);

        weeks.push(WorkingHours {
            from: time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday).unwrap(),
            to: time::Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday).unwrap(),
            expected_hours: working_hours - absence_hours,
            overall_hours: shiftplan_hours + extra_work_hours,
            balance: shiftplan_hours + extra_work_hours - working_hours + absence_hours,
            shiftplan_hours,
            extra_work_hours: filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category == ExtraHoursCategory::ExtraWork)
                .map(|eh| eh.amount)
                .sum(),
            vacation_hours: filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category == ExtraHoursCategory::Vacation)
                .map(|eh| eh.amount)
                .sum(),
            sick_leave_hours: filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category == ExtraHoursCategory::SickLeave)
                .map(|eh| eh.amount)
                .sum(),
            holiday_hours: filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
                .map(|eh| eh.amount)
                .sum(),
            days: day_list
                .iter()
                .filter(|day| day.date.iso_week() == week && day.date.year() == year as i32)
                .cloned()
                .collect(),
        });
    }
    Ok(weeks.into())
}
