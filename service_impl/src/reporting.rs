use crate::gen_service_impl;
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    carryover::CarryoverService,
    clock::ClockService,
    employee_work_details::{EmployeeWorkDetails, EmployeeWorkDetailsService},
    extra_hours::{Availability, ExtraHours, ExtraHoursCategory, ExtraHoursService, ReportType},
    permission::{Authentication, HR_PRIVILEGE},
    reporting::{
        CustomExtraHours, EmployeeReport, ExtraHoursReportCategory, GroupedReportHours,
        ShortEmployeeReport, WorkingHoursDay,
    },
    sales_person::SalesPersonService,
    shiftplan_report::{ShiftplanReportDay, ShiftplanReportService},
    uuid_service::UuidService,
    PermissionService, ServiceError,
};
use shifty_utils::{DayOfWeek, ShiftyDate, ShiftyWeek};
use tokio::join;
use tracing::info;
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

#[test]
pub fn iterator_test() {
    let vec = vec![(1, 1), (2, 5), (1, 6)];
    let map = vec.iter().collect_to_hash_map_by(|e| e.0);
    assert_eq!(map.len(), 2);
    let first_sum = map.get(&1).unwrap().iter().map(|e| e.1).sum::<i32>();
    let second_sum = map.get(&2).unwrap().iter().map(|e| e.1).sum::<i32>();
    assert_eq!(first_sum, 7);
    assert_eq!(second_sum, 5);
}

gen_service_impl! {
    struct ReportingServiceImpl: ReportingService = ReportingServiceDeps {
        ExtraHoursService: ExtraHoursService<Transaction = Self::Transaction> = extra_hours_service,
        ShiftplanReportService: ShiftplanReportService<Transaction = Self::Transaction> = shiftplan_report_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Transaction = Self::Transaction, Context = Self::Context> = employee_work_details_service,
        SalesPersonService: SalesPersonService<Transaction = Self::Transaction, Context = Self::Context> = sales_person_service,
        CarryoverService: CarryoverService<Transaction = Self::Transaction> = carryover_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

pub fn find_working_hours_for_calendar_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> impl Iterator<Item = &EmployeeWorkDetails> {
    working_hours.iter().filter(move |wh| {
        (year, week) >= (wh.from_year, wh.from_calendar_week)
            && (year, week) <= (wh.to_year, wh.to_calendar_week)
    })
}

#[async_trait]
impl<Deps: ReportingServiceDeps> service::reporting::ReportingService
    for ReportingServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_reports_for_all_employees(
        &self,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError> {
        let until_week = until_week.min(time::util::weeks_in_year(year as i32));

        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let working_hours = self
            .employee_work_details_service
            .all(Authentication::Full, tx.clone().into())
            .await?;

        let employees = self
            .sales_person_service
            .get_all(Authentication::Full, tx.clone().into())
            .await?;
        let mut short_employee_report: Vec<ShortEmployeeReport> = Vec::new();
        for paid_employee in employees
            .iter()
            .filter(|employee| employee.is_paid.unwrap_or(false))
        {
            let last_year = year - 1;
            let last_years_last_week = time::util::weeks_in_year(last_year as i32);
            let detailed_shiftplan_report = self
                .shiftplan_report_service
                .extract_shiftplan_report(
                    paid_employee.id,
                    last_year,
                    last_years_last_week,
                    if until_week == time::util::weeks_in_year(year as i32) {
                        year + 1
                    } else {
                        year
                    },
                    if until_week == time::util::weeks_in_year(year as i32) {
                        1
                    } else {
                        until_week
                    },
                    Authentication::Full,
                    tx.clone().into(),
                )
                .await?;

            let working_hours: Arc<[EmployeeWorkDetails]> = working_hours
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
                    tx.clone().into(),
                )
                .await?;
            let previous_year_carryover = self
                .carryover_service
                .get_carryover(
                    paid_employee.id,
                    year - 1,
                    Authentication::Full,
                    tx.clone().into(),
                )
                .await?
                .map(|c| c.carryover_hours)
                .unwrap_or(0.0);

            let additional_weeks = if until_week >= time::util::weeks_in_year(year as i32) {
                1
            } else {
                0
            };
            let (shiftplan_hours, extra_working_hours, absense_hours, planned_hours): (
                f32,
                f32,
                f32,
                f32,
            ) = (0..=until_week + additional_weeks)
                .map(|week| {
                    let target_year = year;
                    let year = if week == 0 {
                        year - 1
                    } else if week > time::util::weeks_in_year(year as i32) {
                        year + 1
                    } else {
                        year
                    };
                    let week = if week == 0 {
                        time::util::weeks_in_year(year as i32)
                    } else if week > time::util::weeks_in_year(year as i32) {
                        week - time::util::weeks_in_year(year as i32)
                    } else {
                        week
                    };

                    let expected_hours =
                        find_working_hours_for_calendar_week(&working_hours, year, week)
                            .map(|wh| weight_for_week(year, week, 
                                &wh.with_to_date(
                                    wh.to_date()
                                        .unwrap_or(ShiftyDate::last_day_in_year(target_year))
                                        .min(ShiftyDate::last_day_in_year(target_year))
                                    ).with_from_date(
                                        wh.from_date()
                                            .unwrap_or(ShiftyDate::first_day_in_year(target_year))
                                            .max(ShiftyDate::first_day_in_year(target_year))
                                    )
                                ))
                            .map(|(expected_hours, _, _)| expected_hours)
                            .sum();
                    // If expected hours is 0 or less, the planned hours and the working hours are the same
                    // because the balance should never be affected in this case.
                    let shiftplan_hours: f32 = detailed_shiftplan_report
                        .iter()
                        .filter(|shift_plan_item| {
                            shift_plan_item.year == year && shift_plan_item.calendar_week == week && shift_plan_item.to_date().map(|d| d.to_date().year() as u32).ok() == Some(target_year)
                        })
                        .map(|shift_plan_item| shift_plan_item.hours)
                        .sum();
                    if expected_hours <= 0.0 {
                        let extra_work: f32 = extra_hours_array
                            .iter()
                            .filter(|extra_hours| {
                                extra_hours.category == ExtraHoursCategory::ExtraWork
                                &&
                                extra_hours.date_time.iso_week() == week
                                    && extra_hours.date_time.year() as u32 == year
                            })
                            .map(|extra_hours| extra_hours.amount)
                            .sum();
                        /*let absense_hours: f32 = extra_hours_array
                            .iter()
                            .filter(|extra_hours| {
                                extra_hours.category != ExtraHoursCategory::ExtraWork
                                &&
                                extra_hours.date_time.iso_week() == week
                                    && extra_hours.date_time.year() as u32 == year
                            })
                            .map(|extra_hours| extra_hours.amount)
                            .sum();*/
                        let overall_hours = extra_work + shiftplan_hours;// - absense_hours;
                        (shiftplan_hours, extra_work, 0.0, overall_hours)
                    } else {
                        let extra_working_hours = extra_hours_array
                            .iter()
                            .filter(|eh| eh.category.as_report_type() == ReportType::WorkingHours
                                && eh.date_time.iso_week() == week
                                && eh.date_time.year() as u32 == year)
                            .map(|eh| eh.amount)
                            .sum::<f32>();
                        let absense_hours = extra_hours_array
                            .iter()
                            .filter(|eh| eh.category.as_report_type() == ReportType::AbsenceHours
                                && eh.date_time.iso_week() == week
                                && eh.date_time.year() as u32 == year)
                            .map(|eh| eh.amount)
                            .sum::<f32>();
                        (
                            shiftplan_hours,
                            extra_working_hours,
                            absense_hours,
                            expected_hours,
                        )
                    }
                })
                .fold(
                    (0.0, 0.0, 0.0, 0.0),
                    |(shiftplan_hours, extra_work, absense, planned),
                     (shiftplan_hours_week, extra_work_week, absense_week, planned_week)| {
                        (
                            shiftplan_hours + shiftplan_hours_week,
                            extra_work + extra_work_week,
                            absense + absense_week,
                            planned + planned_week,
                        )
                    },
                );
            let expected_hours = planned_hours - absense_hours;
            let overall_hours = shiftplan_hours + extra_working_hours;
            let balance_hours = overall_hours - expected_hours + previous_year_carryover;
            short_employee_report.push(ShortEmployeeReport {
                sales_person: Arc::new(paid_employee.clone()),
                balance_hours,
                expected_hours,
                overall_hours,
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
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeReport, ServiceError> {
        let first_day_of_year =
            ShiftyDate::first_day_in_year(year);
        let until_week = until_week.min(time::util::weeks_in_year(year as i32));
        let to_date = if until_week == time::util::weeks_in_year(year as i32) {
            ShiftyDate::last_day_in_year(year)
        } else {
            ShiftyDate::new(year, until_week, DayOfWeek::Sunday)?
        };

        self.get_report_for_employee_range(
            sales_person_id,
            first_day_of_year,
            to_date,
            context,
            tx,
        )
        .await
    }

    async fn get_report_for_employee_range(
        &self,
        sales_person_id: &Uuid,
        from_date: ShiftyDate,
        to_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeReport, ServiceError> {
        let (hr_permission, user_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                *sales_person_id,
                context.clone(),
                tx.clone().into()
            ),
        );
        hr_permission.or(user_permission)?;

        let sales_person = self
            .sales_person_service
            .get(*sales_person_id, context.clone(), tx.clone().into())
            .await?;
        let working_hours = self
            .employee_work_details_service
            .find_by_sales_person_id(*sales_person_id, Authentication::Full, tx.clone().into())
            .await?;
        let shiftplan_report = self
            .shiftplan_report_service
            .extract_shiftplan_report(
                *sales_person_id,
                from_date.year(),
                from_date.week(),
                to_date.year(),
                to_date.week(),
                Authentication::Full,
                tx.clone().into(),
            )
            .await?;
        let extra_hours = self
            .extra_hours_service
            .find_by_sales_person_id_and_year_range(
                *sales_person_id,
                from_date.as_shifty_week(),
                to_date.as_shifty_week(),
                Authentication::Full,
                tx.clone().into(),
            )
            .await?;

        let shiftplan_hours = shiftplan_report
            .iter()
            .filter(|r| {
                if let Ok(date) = r.to_date() {
                    date >= from_date && date <= to_date
                } else {
                    false
                }
            })
            .map(|r| r.hours)
            .sum::<f32>() as f32;
        let overall_extra_work_hours = extra_hours
            .iter()
            .filter(|eh| {
                eh.to_date() >= from_date
                    && eh.to_date() <= to_date
                    && eh.category.as_report_type() == ReportType::WorkingHours
            })
            .map(|eh| eh.amount)
            .sum::<f32>();
        let by_week = hours_per_week(
            &shiftplan_report,
            &extra_hours,
            &working_hours,
            from_date,
            to_date,
        )?;
        let shiftplan_hours_by_week = by_week.iter().map(|week| week.shiftplan_hours).sum::<f32>();
        tracing::info!("Shiftplan hours: {}", shiftplan_hours_by_week);
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
        let vacation_entitlement = working_hours
            .iter()
            .map(|wh| wh.vacation_days_for_year(from_date.year()))
            .sum::<f32>()
            .round();
        let (previous_year_carryover, previous_year_vacation) = self
            .carryover_service
            .get_carryover(
                *sales_person_id,
                from_date.year() - 1,
                Authentication::Full,
                tx.clone().into(),
            )
            .await?
            .map(|c| (c.carryover_hours, c.vacation))
            .unwrap_or((0.0, 0));

        let aggregated_custom_extra_hours: Arc<[CustomExtraHours]> = {
            let mut map: HashMap<(Uuid, String), f32> = HashMap::new();
            for week_report in by_week.iter() {
                for custom_hour_entry in week_report.custom_extra_hours.iter() {
                    *map.entry((custom_hour_entry.id, custom_hour_entry.name.clone()))
                        .or_insert(0.0) += custom_hour_entry.hours;
                }
            }
            map.into_iter()
                .map(|((id, name), hours)| CustomExtraHours { id, name, hours })
                .collect::<Vec<_>>()
                .into()
        };

        let employee_report = EmployeeReport {
            sales_person: Arc::new(sales_person),
            balance_hours: shiftplan_hours + overall_extra_work_hours - planned_hours
                + previous_year_carryover,
            overall_hours: shiftplan_hours + overall_extra_work_hours,
            expected_hours: planned_hours,
            shiftplan_hours,
            holiday_days,
            vacation_carryover: previous_year_vacation,
            vacation_days: vacation_days,
            vacation_entitlement: vacation_entitlement + previous_year_vacation as f32,
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
            carryover_hours: previous_year_carryover,
            by_week,
            by_month: Arc::new([]),
            custom_extra_hours: aggregated_custom_extra_hours,
        };

        Ok(employee_report)
    }

    async fn get_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError> {
        // Auth check is done by working_hours_service
        let working_hours = self
            .employee_work_details_service
            .all_for_week(week, year, context.clone(), tx.clone().into())
            .await?
            .iter()
            .cloned()
            .collect_to_hash_map_by(|wh| wh.sales_person_id);
        let shiftplan_report = self
            .shiftplan_report_service
            .extract_shiftplan_report_for_week(year, week, Authentication::Full, tx.clone().into())
            .await?;
        let shiftplan_report = shiftplan_report
            .iter()
            .collect_to_hash_map_by(|r| r.sales_person_id);
        let extra_hours = self
            .extra_hours_service
            .find_by_week(year, week, Authentication::Full, tx.clone().into())
            .await?;
        info!("Extra hours: {:?}", &extra_hours);
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
                        .filter(|eh| eh.category.availability() == Availability::Available)
                        .map(|eh| eh.amount)
                        .sum::<f32>()
                })
                .unwrap_or(0.0);
            let abense_hours = extra_hours
                .get(&sales_person_id)
                .map(|eh| {
                    eh.iter()
                        .filter(|eh| eh.category.availability() == Availability::Unavailable)
                        .map(|eh| eh.amount)
                        .sum::<f32>()
                })
                .unwrap_or(0.0);
            let planned_hours: f32 =
                find_working_hours_for_calendar_week(&working_hours, year, week)
                    .map(|wh| weight_for_week(year, week, wh).0)
                    .sum();
            let expected_hours = planned_hours - abense_hours;
            let overall_hours = shiftplan_hours + extra_working_hours;
            let balance_hours = overall_hours - expected_hours;
            result.push(ShortEmployeeReport {
                sales_person: Arc::new(
                    self.sales_person_service
                        .get(sales_person_id, Authentication::Full, tx.clone().into())
                        .await?,
                ),
                balance_hours,
                expected_hours,
                overall_hours,
            });
        }

        Ok(result.into())
    }
}

fn weight_for_week(
    year: u32,
    week: u8,
    employee_work_details: &EmployeeWorkDetails,
) -> (f32, u8, f32) {
    let workdays: Arc<[time::Weekday]> = employee_work_details.potential_weekday_list();
    let all_potential_workdays = workdays.len() as u8;

    // Remove the workdays that are outside of the employee's contract.
    let workdays: Arc<[DayOfWeek]> = if year < employee_work_details.from_year
        || year > employee_work_details.to_year
        || (year == employee_work_details.from_year && week < employee_work_details.from_calendar_week)
        || (year == employee_work_details.to_year && week > employee_work_details.to_calendar_week)
    {
        Arc::new([])
    } else if employee_work_details.from_year == employee_work_details.to_year
        && employee_work_details.from_calendar_week == employee_work_details.to_calendar_week 
    {
        workdays
            .iter()
            .map(|workday| DayOfWeek::from(*workday))
            .filter(|workday| *workday >= employee_work_details.from_day_of_week && *workday <= employee_work_details.to_day_of_week)
            .collect()
    } else if year == employee_work_details.from_year
        && week == employee_work_details.from_calendar_week
    {
        workdays
            .iter()
            .map(|workday| DayOfWeek::from(*workday))
            .filter(|workday| *workday >= employee_work_details.from_day_of_week)
            .collect()
    } else if year == employee_work_details.to_year
        && week == employee_work_details.to_calendar_week
    {
        workdays
            .iter()
            .map(|workday| DayOfWeek::from(*workday))
            .filter(|workday| *workday <= employee_work_details.to_day_of_week)
            .collect()
    } else {
        workdays
            .iter()
            .map(|workday| DayOfWeek::from(*workday))
            .collect()
    };

    let num_potential_workdays_in_week = workdays.iter().count();
    let relation = num_potential_workdays_in_week as f32 / all_potential_workdays as f32;
    (
        employee_work_details.expected_hours * relation,
        num_potential_workdays_in_week as u8,
        employee_work_details.workdays_per_week as f32 * relation,
    )
}

fn hours_per_week(
    shiftplan_hours_list: &Arc<[ShiftplanReportDay]>,
    extra_hours_list: &Arc<[ExtraHours]>,
    working_hours: &[EmployeeWorkDetails],
    from_date: ShiftyDate,
    to_date: ShiftyDate,
) -> Result<Arc<[GroupedReportHours]>, ServiceError> {
    let from_week = from_date.as_shifty_week();
    let to_week = to_date.as_shifty_week();

    let mut weeks: Vec<GroupedReportHours> = Vec::new();
    for week in from_week.iter_until(&to_week) {
        tracing::info!("Week: {}, Year: {}", week.week, week.year);
        let filtered_extra_hours_list = extra_hours_list
            .iter()
            .filter(|eh| eh.to_date().as_shifty_week() == week)
            .collect::<Vec<_>>();
        let filtered_shiftplan_hours_list = shiftplan_hours_list
            .iter()
            .filter(|r| {
                if let Ok(date) = r.to_date() {
                    date.as_shifty_week() == week
                } else {
                    false
                }
            })
            .map(|r: &ShiftplanReportDay| {
                tracing::info!("{:?} - {:?}", r.to_date(), r);
                r
            })
            .collect::<Vec<_>>();
        let shiftplan_hours = filtered_shiftplan_hours_list
            .iter()
            .map(|r: &&ShiftplanReportDay| r.hours)
            .sum::<f32>();
        let (working_hours_for_week, days_per_week, workdays_per_week) =
            find_working_hours_for_calendar_week(working_hours, week.year, week.week)
                .map(|wh| weight_for_week(week.year, week.week, 
                    &wh.with_to_date(
                        wh.to_date()
                            .unwrap_or(to_date)
                            .min(to_date)
                        ).with_from_date(
                            wh.from_date()
                                .unwrap_or(from_date)
                                .max(from_date)
                        )
                    ))
                .fold(
                    (0.0f32, 0u8, 0f32),
                    |(working_hours_acc, days_per_week_acc, workdays_per_week_acc),
                     (wh, dpw, wpw)| {
                        (
                            working_hours_acc + wh,
                            days_per_week_acc + dpw,
                            workdays_per_week_acc + wpw,
                        )
                    },
                );
        let extra_work_hours = filtered_extra_hours_list
            .iter()
            .filter(|eh| eh.category.as_report_type() == ReportType::WorkingHours)
            .map(|eh| eh.amount)
            .sum::<f32>();
        let absence_hours = if working_hours_for_week <= 0.0 {
            0.0f32
        } else {
            filtered_extra_hours_list
                .iter()
                .filter(|eh| eh.category.as_report_type() == ReportType::AbsenceHours)
                .map(|eh| eh.amount)
                .sum::<f32>()
        };

        let mut day_list = filtered_extra_hours_list
            .iter()
            .map(|eh| {
                Ok(WorkingHoursDay {
                    date: eh.date_time.date(),
                    hours: eh.amount,
                    category: (&eh.category).into(),
                })
            })
            .chain(
                filtered_shiftplan_hours_list
                    .iter()
                    .map(|working_hours_day| {
                        Ok::<WorkingHoursDay, ServiceError>(WorkingHoursDay {
                            date: time::Date::from_iso_week_date(
                                week.year as i32,
                                working_hours_day.calendar_week,
                                time::Weekday::Sunday
                                    .nth_next(working_hours_day.day_of_week.to_number()),
                            )?,
                            hours: working_hours_day.hours,
                            category: ExtraHoursReportCategory::Shiftplan,
                        })
                    }),
            )
            .collect::<Result<Vec<WorkingHoursDay>, ServiceError>>()?;
        day_list.sort_by_key(|day| day.date);
        let expected_hours = if working_hours_for_week == 0.0 {
            shiftplan_hours + extra_work_hours
        } else {
            working_hours_for_week
        };

        let custom_extra_hours: Arc<[service::reporting::CustomExtraHours]> = {
            let mut map: HashMap<(Uuid, String), f32> = HashMap::new();
            for eh_entry in filtered_extra_hours_list.iter() {
                if let ExtraHoursCategory::CustomExtraHours(lazy_load_custom_def) =
                    &eh_entry.category
                {
                    if let Some(custom_def) = lazy_load_custom_def.get() {
                        let key = (custom_def.id, custom_def.name.to_string());
                        *map.entry(key).or_insert(0.0) += eh_entry.amount;
                    }
                }
            }
            map.into_iter()
                .map(|((id, name), hours)| service::reporting::CustomExtraHours { id, name, hours })
                .collect::<Vec<_>>()
                .into()
        };

        weeks.push(GroupedReportHours {
            from: week.as_date(DayOfWeek::Monday).max(from_date),
            to: week.as_date(DayOfWeek::Sunday).min(to_date),
            year: week.year,
            week: week.week,
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
            custom_extra_hours,
            days: day_list.iter().cloned().collect(),
        });
    }
    Ok(weeks.into())
}
