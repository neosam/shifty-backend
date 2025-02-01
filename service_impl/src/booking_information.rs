use crate::gen_service_impl;
use std::sync::Arc;

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    booking::BookingService,
    booking_information::{
        build_booking_information, BookingInformation, BookingInformationService, WeeklySummary,
        WorkingHoursPerSalesPerson,
    },
    clock::ClockService,
    employee_work_details::EmployeeWorkDetailsService,
    permission::{Authentication, SALES_PRIVILEGE, SHIFTPLANNER_PRIVILEGE},
    reporting::ReportingService,
    sales_person::SalesPersonService,
    sales_person_unavailable::SalesPersonUnavailableService,
    shiftplan_report::ShiftplanReportService,
    slot::{Slot, SlotService},
    special_days::{SpecialDayService, SpecialDayType},
    uuid_service::UuidService,
    PermissionService, ServiceError,
};
use tokio::join;
use uuid::Uuid;

gen_service_impl! {
    struct BookingInformationServiceImpl: BookingInformationService = BookingInformationServiceDeps {
        ShiftplanReportService: ShiftplanReportService<Transaction = Self::Transaction> = shiftplan_report_service,
        SlotService: SlotService<Transaction = Self::Transaction> = slot_service,
        BookingService: BookingService<Transaction = Self::Transaction> = booking_service,
        SalesPersonService: SalesPersonService<Transaction = Self::Transaction> = sales_person_service,
        SalesPersonUnavailableService: SalesPersonUnavailableService<Transaction = Self::Transaction> = sales_person_unavailable_service,
        ReportingService: ReportingService<Transaction = Self::Transaction> = reporting_service,
        SpecialDayService: SpecialDayService = special_day_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Transaction = Self::Transaction> = employee_work_details_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: BookingInformationServiceDeps> BookingInformationService
    for BookingInformationServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

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
            .get_for_week(week, year, Authentication::Full, tx.clone().into())
            .await?;
        let sales_persons = self
            .sales_person_service
            .get_all(Authentication::Full, tx.clone().into())
            .await?;
        let slots = self
            .slot_service
            .get_slots(Authentication::Full, tx.clone().into())
            .await?;
        let unavailable_entries = self
            .sales_person_unavailable_service
            .get_by_week(year, week, Authentication::Full, tx.clone().into())
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
            .get_all(Authentication::Full, tx.clone().into())
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
                .get_week(year, week, Authentication::Full, tx.clone().into())
                .await?;
            let special_days = self
                .special_day_service
                .get_by_week(year, week, Authentication::Full)
                .await?;
            let volunteer_hours = self
                .shiftplan_report_service
                .extract_shiftplan_report_for_week(
                    year,
                    week,
                    Authentication::Full,
                    tx.clone().into(),
                )
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
                monday_available_hours: 0.0,
                tuesday_available_hours: 0.0,
                wednesday_available_hours: 0.0,
                thursday_available_hours: 0.0,
                friday_available_hours: 0.0,
                saturday_available_hours: 0.0,
                sunday_available_hours: 0.0,
            });
        }

        self.transaction_dao.commit(tx).await?;
        Ok(weekly_report.into())
    }

    async fn get_summery_for_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeeklySummary, ServiceError> {
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

        let mut working_hours_per_sales_person = vec![];
        let volunteer_ids: Arc<[Uuid]> = self
            .sales_person_service
            .get_all(Authentication::Full, tx.clone().into())
            .await?
            .iter()
            .filter(|sales_person| !sales_person.is_paid.unwrap_or(false))
            .map(|sales_person| sales_person.id)
            .collect();

        let week_report = self
            .reporting_service
            .get_week(year, week, Authentication::Full, tx.clone().into())
            .await?;
        let special_days = self
            .special_day_service
            .get_by_week(year, week, Authentication::Full)
            .await?;
        let volunteer_hours = self
            .shiftplan_report_service
            .extract_shiftplan_report_for_week(year, week, Authentication::Full, tx.clone().into())
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
            .map(|slot| (slot.to - slot.from).as_seconds_f32() / 3600.0 * slot.min_resources as f32)
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

        // Calculate available hours per day
        let mut monday_hours = 0.0;
        let mut tuesday_hours = 0.0;
        let mut wednesday_hours = 0.0;
        let mut thursday_hours = 0.0;
        let mut friday_hours = 0.0;
        let mut saturday_hours = 0.0;
        let mut sunday_hours = 0.0;

        // Get paid employees and their work details
        let paid_employees = self
            .sales_person_service
            .get_all(Authentication::Full, tx.clone().into())
            .await?
            .iter()
            .filter(|sales_person| sales_person.is_paid.unwrap_or(false))
            .map(|sp| sp.id)
            .collect::<Vec<_>>();

        let work_details = self
            .employee_work_details_service
            .all(Authentication::Full, tx.clone().into())
            .await?;

        let unavailable_days = self
            .sales_person_unavailable_service
            .get_by_week(year, week, Authentication::Full, tx.clone().into())
            .await?;

        // Calculate per-day hours for each paid employee
        for employee_id in paid_employees {
            if let Some(details) = work_details.iter().find(|d| {
                d.sales_person_id == employee_id
                    && (d.from_year < year || (d.from_year == year && d.from_calendar_week <= week))
                    && (d.to_year > year || (d.to_year == year && d.to_calendar_week >= week))
            }) {
                let working_days = details.potential_weekday_list().len() as f32;
                if working_days > 0.0 {
                    let hours_per_day = details.expected_hours / working_days;

                    // Check each day if employee is available (not in unavailable_days)
                    let is_unavailable = |day: service::slot::DayOfWeek| {
                        unavailable_days
                            .iter()
                            .any(|ud| ud.sales_person_id == employee_id && ud.day_of_week == day)
                    };

                    // Add hours to each working day if employee is available
                    for day in details.potential_weekday_list().iter() {
                        let service_day = match day {
                            time::Weekday::Monday => service::slot::DayOfWeek::Monday,
                            time::Weekday::Tuesday => service::slot::DayOfWeek::Tuesday,
                            time::Weekday::Wednesday => service::slot::DayOfWeek::Wednesday,
                            time::Weekday::Thursday => service::slot::DayOfWeek::Thursday,
                            time::Weekday::Friday => service::slot::DayOfWeek::Friday,
                            time::Weekday::Saturday => service::slot::DayOfWeek::Saturday,
                            time::Weekday::Sunday => service::slot::DayOfWeek::Sunday,
                        };

                        if !is_unavailable(service_day) {
                            match service_day {
                                service::slot::DayOfWeek::Monday => monday_hours += hours_per_day,
                                service::slot::DayOfWeek::Tuesday => tuesday_hours += hours_per_day,
                                service::slot::DayOfWeek::Wednesday => {
                                    wednesday_hours += hours_per_day
                                }
                                service::slot::DayOfWeek::Thursday => {
                                    thursday_hours += hours_per_day
                                }
                                service::slot::DayOfWeek::Friday => friday_hours += hours_per_day,
                                service::slot::DayOfWeek::Saturday => {
                                    saturday_hours += hours_per_day
                                }
                                service::slot::DayOfWeek::Sunday => sunday_hours += hours_per_day,
                            }
                        }
                    }
                }
            }
        }

        // Get volunteer hours per day from shiftplan report
        let volunteer_hours_by_day = self
            .shiftplan_report_service
            .extract_shiftplan_report_for_week(year, week, Authentication::Full, tx.clone().into())
            .await?
            .iter()
            .filter(|report| volunteer_ids.iter().any(|id| *id == report.sales_person_id))
            .fold((0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0), |mut acc, report| {
                match report.day_of_week {
                    service::slot::DayOfWeek::Monday => acc.0 += report.hours,
                    service::slot::DayOfWeek::Tuesday => acc.1 += report.hours,
                    service::slot::DayOfWeek::Wednesday => acc.2 += report.hours,
                    service::slot::DayOfWeek::Thursday => acc.3 += report.hours,
                    service::slot::DayOfWeek::Friday => acc.4 += report.hours,
                    service::slot::DayOfWeek::Saturday => acc.5 += report.hours,
                    service::slot::DayOfWeek::Sunday => acc.6 += report.hours,
                }
                acc
            });

        // Add volunteer hours from each day's available hours
        monday_hours += volunteer_hours_by_day.0;
        tuesday_hours += volunteer_hours_by_day.1;
        wednesday_hours += volunteer_hours_by_day.2;
        thursday_hours += volunteer_hours_by_day.3;
        friday_hours += volunteer_hours_by_day.4;
        saturday_hours += volunteer_hours_by_day.5;
        sunday_hours += volunteer_hours_by_day.6;

        // Calculate required hours per day from slots
        let required_hours_by_day =
            slots
                .iter()
                .fold((0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0), |mut acc, slot| {
                    let hours =
                        (slot.to - slot.from).as_seconds_f32() / 3600.0 * slot.min_resources as f32;
                    match slot.day_of_week {
                        service::slot::DayOfWeek::Monday => acc.0 += hours,
                        service::slot::DayOfWeek::Tuesday => acc.1 += hours,
                        service::slot::DayOfWeek::Wednesday => acc.2 += hours,
                        service::slot::DayOfWeek::Thursday => acc.3 += hours,
                        service::slot::DayOfWeek::Friday => acc.4 += hours,
                        service::slot::DayOfWeek::Saturday => acc.5 += hours,
                        service::slot::DayOfWeek::Sunday => acc.6 += hours,
                    }
                    acc
                });

        let summary = WeeklySummary {
            year,
            week,
            overall_available_hours,
            working_hours_per_sales_person: working_hours_per_sales_person.into(),
            required_hours: slot_hours,
            monday_available_hours: monday_hours - required_hours_by_day.0,
            tuesday_available_hours: tuesday_hours - required_hours_by_day.1,
            wednesday_available_hours: wednesday_hours - required_hours_by_day.2,
            thursday_available_hours: thursday_hours - required_hours_by_day.3,
            friday_available_hours: friday_hours - required_hours_by_day.4,
            saturday_available_hours: saturday_hours - required_hours_by_day.5,
            sunday_available_hours: sunday_hours - required_hours_by_day.6,
        };

        self.transaction_dao.commit(tx).await?;
        Ok(summary)
    }
}
