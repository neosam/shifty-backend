use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use dao::shiftplan_report::ShiftplanReportEntity;
use service::{
    extra_hours::{ExtraHours, ExtraHoursCategory, ReportType},
    permission::{Authentication, HR_PRIVILEGE},
    reporting::{
        EmployeeReport, ExtraHoursReportCategory, GroupedReportHours, ShortEmployeeReport,
        WorkingHoursDay,
    },
    working_hours::WorkingHours,
    ServiceError,
};
use tokio::join;
use uuid::Uuid;

pub trait IteratorExt {
    fn collect_to_hash_map_by<K, F>(self, f: F) -> HashMap<K, Arc<[Self::Item]>>
    where
        Self: Iterator + Sized,
        K: Clone + Eq + std::hash::Hash,
        F: Fn(&Self::Item) -> K,
    {
        let vec_map = self.fold(HashMap::new(), |mut map, item| {
            let key = f(&item);
            map.entry(key.clone()).or_insert_with(Vec::new).push(item);
            map
        });
        let vec_map: HashMap<K, Arc<[Self::Item]>> = vec_map
            .into_iter()
            .map(|(key, vec)| (key, vec.into()))
            .collect();
        vec_map
    }
}
impl<T> IteratorExt for T where T: Iterator {}

pub struct ReportingServiceImpl<
    ExtraHoursService,
    ShiftplanReportDao,
    WorkingHoursService,
    SalesPersonService,
    PermissionService,
    ClockService,
    UuidService,
> where
    ExtraHoursService: service::extra_hours::ExtraHoursService + Send + Sync,
    ShiftplanReportDao: dao::shiftplan_report::ShiftplanReportDao + Send + Sync,
    WorkingHoursService: service::working_hours::WorkingHoursService + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    pub extra_hours_service: Arc<ExtraHoursService>,
    pub shiftplan_report_dao: Arc<ShiftplanReportDao>,
    pub working_hours_service: Arc<WorkingHoursService>,
    pub sales_person_service: Arc<SalesPersonService>,
    pub permission_service: Arc<PermissionService>,
    pub clock_service: Arc<ClockService>,
    pub uuid_service: Arc<UuidService>,
}

impl<
        ExtraHoursService,
        ShiftplanReportDao,
        WorkingHoursService,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
    ReportingServiceImpl<
        ExtraHoursService,
        ShiftplanReportDao,
        WorkingHoursService,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
where
    ExtraHoursService: service::extra_hours::ExtraHoursService + Send + Sync,
    ShiftplanReportDao: dao::shiftplan_report::ShiftplanReportDao + Send + Sync,
    WorkingHoursService: service::working_hours::WorkingHoursService + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    pub fn new(
        extra_hours_service: Arc<ExtraHoursService>,
        shiftplan_report_dao: Arc<ShiftplanReportDao>,
        working_hours_service: Arc<WorkingHoursService>,
        sales_person_service: Arc<SalesPersonService>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            extra_hours_service,
            shiftplan_report_dao,
            working_hours_service,
            sales_person_service,
            permission_service,
            clock_service,
            uuid_service,
        }
    }
}

pub fn find_working_hours_for_calendar_week(
    working_hours: &[WorkingHours],
    year: u32,
    week: u8,
) -> Option<&WorkingHours> {
    working_hours.iter().find(|wh| {
        (year, week) >= (wh.from_year, wh.from_calendar_week)
            && (year, week) <= (wh.to_year, wh.to_calendar_week)
    })
}

#[async_trait]
impl<
        ExtraHoursService,
        ShiftplanReportDao,
        WorkingHoursService,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    > service::reporting::ReportingService
    for ReportingServiceImpl<
        ExtraHoursService,
        ShiftplanReportDao,
        WorkingHoursService,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
where
    ExtraHoursService: service::extra_hours::ExtraHoursService + Send + Sync,
    ShiftplanReportDao: dao::shiftplan_report::ShiftplanReportDao + Send + Sync,
    WorkingHoursService: service::working_hours::WorkingHoursService<Context = PermissionService::Context>
        + Send
        + Sync,
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

        let working_hours = self.working_hours_service.all(Authentication::Full).await?;

        let employees = self
            .sales_person_service
            .get_all(Authentication::Full)
            .await?;
        let mut short_employee_report: Vec<ShortEmployeeReport> = Vec::new();
        for paid_employee in employees
            .iter()
            .filter(|employee| employee.is_paid.unwrap_or(false))
        {
            let detailed_shiftplan_report = self
                .shiftplan_report_dao
                .extract_shiftplan_report(paid_employee.id, year, until_week)
                .await?;

            let shiftplan_hours = shiftplan_report
                .iter()
                .filter(|r| r.sales_person_id == paid_employee.id)
                .map(|r| r.hours)
                .sum::<f32>();
            let working_hours: Arc<[WorkingHours]> = working_hours
                .iter()
                .filter(|wh| wh.sales_person_id == paid_employee.id)
                .cloned()
                .collect();
            let extra_hours_array = self
                .extra_hours_service
                .find_by_sales_person_id_and_year(
                    paid_employee.id,
                    year,
                    until_week,
                    Authentication::Full,
                )
                .await?;
            let extra_hours = extra_hours_array.iter().map(|eh| eh.amount).sum::<f32>();
            let planned_hours: f32 = (1..=until_week)
                .map(|week| {
                    find_working_hours_for_calendar_week(&working_hours, year, week)
                        .map(|wh| {
                            if wh.expected_hours == 0.0 {
                                let extra_work: f32 = extra_hours_array
                                    .iter()
                                    .filter(|extra_hours| {
                                        extra_hours.category == ExtraHoursCategory::ExtraWork
                                            && extra_hours.date_time.iso_week() == week
                                            && extra_hours.date_time.year() as u32 == year
                                    })
                                    .map(|extra_hours| extra_hours.amount)
                                    .sum();
                                let shiftplan_hours: f32 = detailed_shiftplan_report
                                    .iter()
                                    .filter(|shift_plan_item| {
                                        shift_plan_item.year == year
                                            && shift_plan_item.calendar_week == week
                                    })
                                    .map(|shift_plan_item| shift_plan_item.hours)
                                    .sum();
                                extra_work + shiftplan_hours
                            } else {
                                wh.expected_hours
                            }
                        })
                        .unwrap_or(0.0)
                })
                .sum();
            let balance_hours = shiftplan_hours + extra_hours - planned_hours;
            short_employee_report.push(ShortEmployeeReport {
                sales_person: Arc::new(paid_employee.clone()),
                balance_hours,
                expected_hours: planned_hours,
                overall_hours: shiftplan_hours + extra_hours,
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
            .working_hours_service
            .find_by_sales_person_id(*sales_person_id, Authentication::Full)
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

        let shiftplan_hours = shiftplan_report.iter().map(|r| r.hours).sum::<f32>() as f32;
        let overall_extra_work_hours = extra_hours
            .iter()
            .filter(|eh| {
                eh.date_time.iso_week() <= until_week
                    && eh.category.as_report_type() == ReportType::WorkingHours
            })
            .map(|eh| eh.amount)
            .sum::<f32>();
        let by_week = hours_per_week(
            &shiftplan_report,
            &extra_hours,
            &working_hours,
            year,
            until_week,
        )?;
        let (vacation_days, sick_leave_days, holiday_days, absence_days) = by_week.iter().fold(
            (0.0, 0.0, 0.0, 0.0),
            |(vacation_days, sick_leave_days, holiday_days, absence_days), week| {
                (
                    vacation_days + week.vacation_days(),
                    sick_leave_days + week.sick_leave_days(),
                    holiday_days + week.holiday_days(),
                    absence_days + week.absence_days(),
                )
            },
        );

        let planned_hours: f32 = by_week.iter().map(|week| week.expected_hours).sum();

        let employee_report = EmployeeReport {
            sales_person: Arc::new(sales_person),
            balance_hours: shiftplan_hours + overall_extra_work_hours - planned_hours,
            overall_hours: shiftplan_hours + overall_extra_work_hours,
            expected_hours: planned_hours,
            shiftplan_hours,
            holiday_days,
            vacation_days,
            sick_leave_days,
            absence_days,
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
            by_week,
            by_month: Arc::new([]),
        };

        Ok(employee_report)
    }

    async fn get_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError> {
        // Auth check is done by working_hours_service
        let working_hours = self
            .working_hours_service
            .all_for_week(week, year, context)
            .await?
            .iter()
            .cloned()
            .collect_to_hash_map_by(|wh| wh.sales_person_id);
        let shiftplan_report = self
            .shiftplan_report_dao
            .extract_shiftplan_report_for_week(year, week)
            .await?;
        let shiftplan_report = shiftplan_report
            .iter()
            .collect_to_hash_map_by(|r| r.sales_person_id);
        let extra_hours = self
            .extra_hours_service
            .find_by_week(year, week, Authentication::Full)
            .await?;
        let extra_hours = extra_hours
            .iter()
            .collect_to_hash_map_by(|eh| eh.sales_person_id);

        let mut result = Vec::new();

        for (sales_person_id, working_hours) in working_hours {
            let shiftplan_hours = shiftplan_report
                .get(&sales_person_id)
                .map(|r| r.iter().map(|r| r.hours).sum::<f32>())
                .unwrap_or(0.0);
            let extra_working_hours = extra_hours
                .get(&sales_person_id)
                .map(|eh| {
                    eh.iter()
                        .filter(|eh| eh.category.as_report_type() == ReportType::WorkingHours)
                        .map(|eh| eh.amount)
                        .sum::<f32>()
                })
                .unwrap_or(0.0);
            let abense_hours = extra_hours
                .get(&sales_person_id)
                .map(|eh| {
                    eh.iter()
                        .filter(|eh| eh.category.as_report_type() == ReportType::AbsenceHours)
                        .map(|eh| eh.amount)
                        .sum::<f32>()
                })
                .unwrap_or(0.0);
            let planned_hours = find_working_hours_for_calendar_week(&working_hours, year, week)
                .map(|wh| wh.expected_hours)
                .unwrap_or(0.0);
            let balance_hours = shiftplan_hours + extra_working_hours - planned_hours;
            result.push(ShortEmployeeReport {
                sales_person: Arc::new(
                    self.sales_person_service
                        .get(sales_person_id, Authentication::Full)
                        .await?,
                ),
                balance_hours,
                expected_hours: planned_hours - abense_hours,
                overall_hours: shiftplan_hours + extra_working_hours,
            });
        }

        Ok(result.into())
    }
}

fn hours_per_week(
    shiftplan_hours_list: &Arc<[ShiftplanReportEntity]>,
    extra_hours_list: &Arc<[ExtraHours]>,
    working_hours: &[WorkingHours],
    year: u32,
    week_until: u8,
) -> Result<Arc<[GroupedReportHours]>, ServiceError> {
    let mut weeks: Vec<GroupedReportHours> = Vec::new();
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
        let (working_hours, days_per_week, workdays_per_week) = working_hours
            .iter()
            .find(|wh| wh.from_calendar_week <= week && wh.to_calendar_week >= week)
            .map(|wh| (wh.expected_hours, wh.days_per_week, wh.workdays_per_week))
            .unwrap_or((0.0, 1, 1));
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
        let expected_hours = if working_hours == 0.0 {
            shiftplan_hours + extra_work_hours
        } else {
            working_hours
        };

        weeks.push(GroupedReportHours {
            from: time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday).unwrap(),
            to: time::Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday).unwrap(),
            contract_weekly_hours: expected_hours,
            expected_hours: expected_hours - absence_hours,
            overall_hours: shiftplan_hours + extra_work_hours,
            balance: shiftplan_hours + extra_work_hours - expected_hours + absence_hours,
            shiftplan_hours,
            days_per_week,
            workdays_per_week,
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
