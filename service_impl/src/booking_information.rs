use crate::gen_service_impl;
use crate::reporting::find_working_hours_for_calendar_week;
use std::sync::Arc;

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    absence::AbsenceService,
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
use shifty_utils::DayOfWeek;
use tokio::join;
use uuid::Uuid;

/// D-05 / CVC-04: Band 2 per-person surplus = max(actual − committed, 0).
/// `committed` is the person's OWN cap-gated committed_voluntary for the week
/// (0.0 for cap=false rows — gated at the call site, CVC-06). `committed = 0`
/// ⇒ returns `actual` unchanged ⇒ Band 2 bit-identical to pre-v1.4.
/// NEVER subtract aggregate-from-aggregate — the max is nonlinear, so this MUST
/// be applied per person before summing (person-set overlap is real, D-05).
pub(crate) fn volunteer_surplus_above_committed(actual: f32, committed: f32) -> f32 {
    (actual - committed).max(0.0)
}

/// Band 2 aggregate (D-05 / CVC-04): sum max(weekly_actual_p − committed_p, 0) PER PERSON.
///
/// The per-person weekly actual MUST be summed across the per-day shiftplan-report rows BEFORE
/// the nonlinear max — `extract_shiftplan_report_for_week` returns one row per (person, day)
/// because the DAO query groups by `sales_person_id, year, day_of_week`. Subtracting committed
/// per-day instead of per-week under-counts the surplus (CR-01 BLOCKER):
///
/// Example: committed=5, Mon 3h + Tue 4h (weekly actual=7).
/// - Correct (per-week): max(7 − 5, 0) = 2.0
/// - Buggy (per-day):    max(3 − 5, 0) + max(4 − 5, 0) = 0 + 0 = 0.0  ← CR-01
///
/// `per_day_actuals`: iterator of `(sales_person_id, hours)` for each per-day report row.
/// `committed_for_person`: closure returning the cap-gated weekly committed for a given person.
pub(crate) fn volunteer_surplus_band2(
    per_day_actuals: impl IntoIterator<Item = (uuid::Uuid, f32)>,
    committed_for_person: impl Fn(uuid::Uuid) -> f32,
) -> f32 {
    use std::collections::HashMap;
    let mut weekly: HashMap<uuid::Uuid, f32> = HashMap::new();
    for (person, hours) in per_day_actuals {
        *weekly.entry(person).or_insert(0.0) += hours;
    }
    weekly
        .into_iter()
        .map(|(person, actual)| {
            volunteer_surplus_above_committed(actual, committed_for_person(person))
        })
        .sum()
}

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
        // VFA-01 (D-26-01/D-26-03): AbsenceService provides volunteer absences for the year-view.
        // BookingInformationService (business-logic tier) → AbsenceService (business-logic tier):
        // no DI cycle because AbsenceService does NOT consume BookingInformationService
        // (Service-Tier rule; D-Phase3-18 regression-lock preserved).
        AbsenceService: AbsenceService<Context = Self::Context, Transaction = Self::Transaction> = absence_service,
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
        // Pitfall 4: load work-details ONCE before the per-week loop (not N times)
        let all_work_details = self
            .employee_work_details_service
            .all(Authentication::Full, tx.clone().into())
            .await?;
        for week in 1..=(weeks_in_year + 3) {
            let (year, week) = if week > weeks_in_year {
                (year + 1, week - weeks_in_year)
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
            // Band 2 (D-05 / CVC-04 / CR-01 fix): per-person surplus = Σ max(actual_p − committed_p, 0).
            // CRITICAL: `extract_shiftplan_report_for_week` returns ONE ROW PER (person, day) because
            // the DAO query groups by `sales_person_id, year, day_of_week`. We MUST aggregate
            // per-person weekly actuals BEFORE applying the nonlinear max (CR-01 blocker):
            //   Buggy per-day form: max(3−5,0) + max(4−5,0) = 0 when actual=7, committed=5.
            //   Correct per-week:   max(7−5, 0) = 2.
            // volunteer_surplus_band2 accumulates per-day rows into per-person weekly totals first.
            let shiftplan_reports = self
                .shiftplan_report_service
                .extract_shiftplan_report_for_week(
                    year,
                    week,
                    Authentication::Full,
                    tx.clone().into(),
                )
                .await?;
            let per_day_actuals = shiftplan_reports
                .iter()
                .filter(|report| volunteer_ids.contains(&report.sales_person_id))
                .map(|report| (report.sales_person_id, report.hours));
            let volunteer_hours = volunteer_surplus_band2(per_day_actuals, |sp_id| {
                // Per-person cap-gated weekly committed (CVC-06): sum over the person's active rows.
                find_working_hours_for_calendar_week(&all_work_details, year, week)
                    .filter(|wh| {
                        wh.sales_person_id == sp_id
                            && (wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0) // D-05: cap || rein-freiwillig (expected_hours=0)
                    })
                    .map(|wh| wh.committed_voluntary) // D-03 flat, no weight
                    .sum()
            });
            // Band 1 (D-04 / CVC-04): cap-gated Σ_person committed per week (flat, no weight D-03).
            // Explicit per-row cap filter (Pitfall 5 / CVC-06): non-capped rows contribute 0.
            let committed_voluntary_hours: f32 = find_working_hours_for_calendar_week(
                &all_work_details,
                year,
                week,
            )
            .filter(|wh| wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0) // D-05: cap || rein-freiwillig (expected_hours=0), symmetrisch zu D-01 Editor-Sichtbarkeit
            .map(|wh| wh.committed_voluntary) // D-03 flat, no weight
            .sum();
            let slots: Arc<[Slot]> = self
                .slot_service
                .get_slots_for_week_all_plans(year, week, Authentication::Full, tx.clone().into())
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
            let mut paid_hours = 0.0;
            for report in week_report.iter() {
                paid_hours += report.dynamic_hours;
                if is_shiftplanner {
                    let absence_hours = report.vacation_hours
                        + report.sick_leave_hours
                        + report.holiday_hours
                        + report.custom_absence_hours.iter().map(|c| c.hours).sum::<f32>();
                    working_hours_per_sales_person.push(WorkingHoursPerSalesPerson {
                        sales_person_id: report.sales_person.id,
                        sales_person_name: report.sales_person.name.clone(),
                        available_hours: report.expected_hours,
                        absence_hours,
                        vacation_hours: report.vacation_hours,
                        sick_leave_hours: report.sick_leave_hours,
                        holiday_hours: report.holiday_hours,
                        unavailable_hours: report.unavailable_hours,
                        custom_absence_hours: report.custom_absence_hours.clone(),
                    });
                }
            }
            // D-01 (Phase 16): available = paid + committed (Band 1, pledge) + volunteer (Band 2, surplus).
            // No double-count: Band 2 already subtracted committed per-person (Σ max(actual−committed,0)).
            let overall_available_hours =
                committed_voluntary_hours + volunteer_hours + paid_hours;
            weekly_report.push(WeeklySummary {
                year,
                week,
                overall_available_hours,
                paid_hours,
                volunteer_hours,
                committed_voluntary_hours,
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
            .filter(|report| volunteer_ids.contains(&report.sales_person_id))
            .map(|report| report.hours)
            .sum::<f32>();
        let slots: Arc<[Slot]> = self
            .slot_service
                .get_slots_for_week_all_plans(year, week, Authentication::Full, tx.clone().into())
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
        let mut paid_hours = 0.0;
        for report in week_report.iter() {
            paid_hours += report.dynamic_hours;
            if is_shiftplanner {
                let absence_hours = report.vacation_hours
                    + report.sick_leave_hours
                    + report.holiday_hours
                    + report.custom_absence_hours.iter().map(|c| c.hours).sum::<f32>();
                working_hours_per_sales_person.push(WorkingHoursPerSalesPerson {
                    sales_person_id: report.sales_person.id,
                    sales_person_name: report.sales_person.name.clone(),
                    available_hours: report.expected_hours,
                    absence_hours,
                    vacation_hours: report.vacation_hours,
                    sick_leave_hours: report.sick_leave_hours,
                    holiday_hours: report.holiday_hours,
                    unavailable_hours: report.unavailable_hours,
                    custom_absence_hours: report.custom_absence_hours.clone(),
                });
            }
        }
        let overall_available_hours = volunteer_hours + paid_hours;

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
                // Check each day if employee is available (not in unavailable_days)
                let is_unavailable = |day: DayOfWeek| {
                    unavailable_days
                        .iter()
                        .any(|ud| ud.sales_person_id == employee_id && ud.day_of_week == day)
                };

                // Count working days excluding unavailable days
                let working_days = details
                    .potential_weekday_list()
                    .iter()
                    .filter(|&day| {
                        let service_day = match day {
                            time::Weekday::Monday => DayOfWeek::Monday,
                            time::Weekday::Tuesday => DayOfWeek::Tuesday,
                            time::Weekday::Wednesday => DayOfWeek::Wednesday,
                            time::Weekday::Thursday => DayOfWeek::Thursday,
                            time::Weekday::Friday => DayOfWeek::Friday,
                            time::Weekday::Saturday => DayOfWeek::Saturday,
                            time::Weekday::Sunday => DayOfWeek::Sunday,
                        };
                        !is_unavailable(service_day)
                    })
                    .count() as f32;

                if working_days > 0.0 {
                    let hours_per_day = details.expected_hours / working_days;

                    // Check each day if employee is available (not in unavailable_days)
                    let is_unavailable = |day: DayOfWeek| {
                        unavailable_days
                            .iter()
                            .any(|ud| ud.sales_person_id == employee_id && ud.day_of_week == day)
                    };

                    // Add hours to each working day if employee is available
                    for day in details.potential_weekday_list().iter() {
                        let service_day = match day {
                            time::Weekday::Monday => DayOfWeek::Monday,
                            time::Weekday::Tuesday => DayOfWeek::Tuesday,
                            time::Weekday::Wednesday => DayOfWeek::Wednesday,
                            time::Weekday::Thursday => DayOfWeek::Thursday,
                            time::Weekday::Friday => DayOfWeek::Friday,
                            time::Weekday::Saturday => DayOfWeek::Saturday,
                            time::Weekday::Sunday => DayOfWeek::Sunday,
                        };

                        if !is_unavailable(service_day) {
                            match service_day {
                                DayOfWeek::Monday => monday_hours += hours_per_day,
                                DayOfWeek::Tuesday => tuesday_hours += hours_per_day,
                                DayOfWeek::Wednesday => wednesday_hours += hours_per_day,
                                DayOfWeek::Thursday => thursday_hours += hours_per_day,
                                DayOfWeek::Friday => friday_hours += hours_per_day,
                                DayOfWeek::Saturday => saturday_hours += hours_per_day,
                                DayOfWeek::Sunday => sunday_hours += hours_per_day,
                            }
                        }
                    }
                }
            }
        }

        // Get volunteer hours per day from shiftplan report
        let volunteer_reports = self
            .shiftplan_report_service
            .extract_shiftplan_report_for_week(year, week, Authentication::Full, tx.clone().into())
            .await?;

        // Accumulate hours by day for volunteers
        let volunteer_hours_by_day = volunteer_reports
            .iter()
            .filter(|report| volunteer_ids.contains(&report.sales_person_id))
            .fold((0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0), |mut acc, report| {
                match report.day_of_week {
                    DayOfWeek::Monday => acc.0 += report.hours,
                    DayOfWeek::Tuesday => acc.1 += report.hours,
                    DayOfWeek::Wednesday => acc.2 += report.hours,
                    DayOfWeek::Thursday => acc.3 += report.hours,
                    DayOfWeek::Friday => acc.4 += report.hours,
                    DayOfWeek::Saturday => acc.5 += report.hours,
                    DayOfWeek::Sunday => acc.6 += report.hours,
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
                        DayOfWeek::Monday => acc.0 += hours,
                        DayOfWeek::Tuesday => acc.1 += hours,
                        DayOfWeek::Wednesday => acc.2 += hours,
                        DayOfWeek::Thursday => acc.3 += hours,
                        DayOfWeek::Friday => acc.4 += hours,
                        DayOfWeek::Saturday => acc.5 += hours,
                        DayOfWeek::Sunday => acc.6 += hours,
                    }
                    acc
                });

        let summary = WeeklySummary {
            year,
            week,
            overall_available_hours,
            paid_hours,
            volunteer_hours,
            // Phase 15: Band 1 is year-view-only (D-04); single-week variant keeps inert 0.0
            // placeholder (see 15-01-SUMMARY.md). volunteer_hours is left at full actual (no
            // per-person surplus reduction) because this variant feeds a per-day consumer.
            committed_voluntary_hours: 0.0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use service::booking_information::WeeklySummary;

    // --- volunteer_surplus_above_committed helper tests (Task 1, t1-t3) ---

    #[test]
    fn surplus_over_fulfilled() {
        // t1: committed=5, actual=7 → surplus = max(7-5, 0) = 2
        let result = volunteer_surplus_above_committed(7.0, 5.0);
        assert!((result - 2.0).abs() < 0.001, "expected 2.0, got {result}");
    }

    #[test]
    fn surplus_pledge_covers() {
        // t2: committed=5, actual=3 → surplus = max(3-5, 0) = 0 (floor)
        let result = volunteer_surplus_above_committed(3.0, 5.0);
        assert!((result - 0.0).abs() < 0.001, "expected 0.0, got {result}");
    }

    #[test]
    fn surplus_committed_zero_backward_compat() {
        // t3: committed=0, actual=7 → surplus = max(7-0, 0) = 7 (no-op, identical to pre-v1.4)
        let result = volunteer_surplus_above_committed(7.0, 0.0);
        assert!((result - 7.0).abs() < 0.001, "expected 7.0, got {result}");
    }

    #[test]
    fn weekly_summary_constructs_with_committed_field() {
        // t4: WeeklySummary with committed_voluntary_hours: 0.0 constructs, Clone/Debug/PartialEq work
        let summary = WeeklySummary {
            year: 2026,
            week: 1,
            overall_available_hours: 40.0,
            required_hours: 35.0,
            paid_hours: 40.0,
            volunteer_hours: 5.0,
            committed_voluntary_hours: 0.0,
            monday_available_hours: 8.0,
            tuesday_available_hours: 8.0,
            wednesday_available_hours: 8.0,
            thursday_available_hours: 8.0,
            friday_available_hours: 8.0,
            saturday_available_hours: 0.0,
            sunday_available_hours: 0.0,
            working_hours_per_sales_person: Arc::from(vec![]),
        };
        let cloned = summary.clone();
        assert_eq!(summary, cloned);
        // Debug formatting must not panic
        let _debug = format!("{:?}", summary);
        assert!((summary.committed_voluntary_hours - 0.0).abs() < 0.001);
    }
}
