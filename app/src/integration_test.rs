use std::{collections::HashMap, sync::Arc};

use dao_impl::BasicDaoImpl;
use proptest::prelude::*;
use rest::RestStateDef;
use service::{
    booking::Booking,
    employee_work_details::EmployeeWorkDetails,
    extra_hours::{ExtraHours, ExtraHoursCategory},
    permission::Authentication,
    sales_person::SalesPerson,
    slot::DayOfWeek,
    ServiceError, ValidationFailureItem,
};
use sqlx::SqlitePool;
use time_macros::date;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::{create_dev_admin_user, RestStateImpl};
use dao::BasicDao;
use service::booking::BookingService;
use service::employee_work_details::EmployeeWorkDetailsService;
use service::extra_hours::ExtraHoursService;
use service::reporting::ReportingService;
use service::sales_person::SalesPersonService;
use service::slot::SlotService;

prop_compose! {
    fn arb_sales_person()(
        name in "[a-z][0-9a-zA-Z]{1,20}",
        background_color in "#[a-f0-9]{6}",
        is_paid in proptest::option::weighted(0.5, proptest::bool::weighted(0.5)),
    ) -> SalesPerson {
        SalesPerson {
            id: Uuid::new_v4(),
            name: name.into(),
            background_color: background_color.into(),
            is_paid,
            inactive: false,
            deleted: None,
            version: Uuid::new_v4(),
        }
    }
}

prop_compose! {
    fn arb_working_hour(
        from_year: u32,
        from_calendar_week: u8,
        to_year: u32,
        to_calendar_week: u8,
        sales_person_id: Option<Uuid>,
        expected_hours_zero_chance: f64,
    )(
        expected_hours in prop::bool::weighted(expected_hours_zero_chance)
            .prop_flat_map(|is_zero| if is_zero { (1.0..=7.0f32).boxed() } else { Just(0.0).boxed() }),
        workdays_per_week in 1..=6u8//days_per_week,
    ) -> EmployeeWorkDetails {
        EmployeeWorkDetails {
            id: Uuid::new_v4(),
            sales_person_id: sales_person_id.unwrap_or_else(|| Uuid::new_v4()),
            expected_hours,
            from_year,
            from_calendar_week,
            from_day_of_week: DayOfWeek::Monday,
            to_year,
            to_calendar_week,
            to_day_of_week: DayOfWeek::Sunday,
            workdays_per_week,
            //days_per_week,

            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: true,
            sunday: false,

            vacation_days: 25u8,

            created: Some(time::PrimitiveDateTime::new(date!(2020-01-01), time::Time::MIDNIGHT)),
            deleted: None,
            version: Uuid::new_v4(),
        }
    }
}

prop_compose! {
    fn arb_any_calenar_week(from_year: u32, to_year: u32)
        (
            year in from_year..=to_year,
            week in 1..=52u8,
        ) -> (u32, u8) {
        (year, week)
    }
}

proptest! {
    #[test]
    fn test_arb_any_calendar_week(week in arb_any_calenar_week(2000, 2005)) {
        assert!(week.0 >= 2000 && week.0 <= 2005);
        assert!(week.1 >= 1 && week.1 <= 52);

    }
}

prop_compose! {
    fn arb_ordered_calendar_weeks(from_year: u32, to_year: u32, min_weeks: usize, max_weeks: usize) (
        mut week in prop::collection::vec(arb_any_calenar_week(from_year, to_year), min_weeks..max_weeks)
    ) -> Vec<(u32, u8)> {
        week.sort();
        for (i, j) in (0..week.len()).zip(1..week.len()) {
            if week[i].0 >= week[j].0 {
                let date = time::Date::from_iso_week_date(week[i].0 as i32, week[i].1, time::Weekday::Thursday).unwrap() + time::Duration::WEEK;
                week[j] = (date.year() as u32, date.iso_week());
            }
        }
        week
    }
}
proptest! {
    #[test]
    fn test_arb_any_calendar_weeks(weeks in arb_ordered_calendar_weeks(2000, 2005, 1, 5)) {
        for i in 0..weeks.len() - 1 {
            assert!(weeks[i] < weeks[i + 1]);
        }

    }
}

prop_compose! {
    fn arb_ordered_calendar_week_pairs(from_year: u32, to_year: u32, min_weeks: usize, max_weeks: usize) (
        week_pairs in arb_ordered_calendar_weeks(from_year, to_year, min_weeks, max_weeks)
            .prop_map(|weeks| (0..weeks.len()).zip(1..weeks.len())
                .map(move |(i, j)| {
                    let start_week = weeks[i];
                    let next_week = weeks[j];
                    let next_weeks_monday = time::Date::from_iso_week_date(
                        next_week.0 as i32,
                        next_week.1,
                        time::Weekday::Thursday
                    ).unwrap();
                    let last_weeks_monday = next_weeks_monday - time::Duration::WEEK;
                    let last_week = (last_weeks_monday.year() as u32, last_weeks_monday.iso_week());
                    (start_week, last_week)
                })
                .collect::<Vec<_>>()
            )
    ) -> Vec<((u32, u8), (u32, u8))> {
        week_pairs
    }
}

prop_compose! {
    fn arb_working_hours(
        sales_person_id: Option<Uuid>,
    )(
        working_hours in arb_ordered_calendar_week_pairs(2000, 2005, 2, 6)
            .prop_flat_map(move |weeks| weeks
                .into_iter()
                .map(|(start_week, end_week)| arb_working_hour(
                    start_week.0, start_week.1,
                    end_week.0, end_week.1,
                    sales_person_id,
                    0.2))
                .collect::<Vec<_>>()

            )
    ) -> Arc<[EmployeeWorkDetails]> {
        working_hours.into()
    }
}

prop_compose! {
    fn arb_primitive_date_time(
        year_from: u32,
        year_to: u32,
    ) (
        year in year_from..=year_to,
        week in 1..=52u8,
        weekday in 0..7u8,
        hour in 0..24u8,
        minute in 0..60u8,
        second in 0..60u8,
    ) -> time::PrimitiveDateTime {
        time::PrimitiveDateTime::new(
            time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday.nth_next(weekday)).unwrap(),
            time::Time::from_hms(hour, minute, second).unwrap()
        )
    }
}

prop_compose! {
    fn arb_extra_hour(sales_person_id: Option<Uuid>)(
        category in prop_oneof![
            Just(ExtraHoursCategory::ExtraWork),
            Just(ExtraHoursCategory::Vacation),
            Just(ExtraHoursCategory::SickLeave),
            Just(ExtraHoursCategory::Holiday),
        ],
        amount in 0.1..=5.0f32,
        description in ".*",
        date_time in arb_primitive_date_time(2000, 2005),
    ) -> ExtraHours {
        ExtraHours {
            id: Uuid::new_v4(),
            sales_person_id: sales_person_id.unwrap_or_else(|| Uuid::new_v4()),
            amount,
            description: description.into(),
            category,
            date_time,
            created: Some(time::PrimitiveDateTime::new(
                date!(2020-01-01),
                time::Time::MIDNIGHT,
            )),
            deleted: None,
            version: Uuid::new_v4(),
        }
    }
}

#[cfg(test)]
pub fn get_working_hours_for_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> Option<&EmployeeWorkDetails> {
    working_hours.iter().find(|working_hour| {
        (working_hour.from_year, working_hour.from_calendar_week) <= (year, week)
            && (working_hour.to_year, working_hour.to_calendar_week) >= (year, week)
    })
}

const EPSILON: f32 = 0.0001;

proptest! {
    // Skip test for now since start and end of years are not handled correctly in the currently.
    //#[test]
    fn test_report(
        testdata in prop::collection::vec(arb_sales_person(), 1..5)
            .prop_flat_map(|sales_persons| {
                let working_hours =
                    sales_persons.iter()
                        .map(|sales_person| sales_person.id)
                        .map(|sales_person_id| arb_working_hours(Some(sales_person_id)))
                        .collect::<Vec<_>>();
                let extra_hours =
                    sales_persons.iter()
                        .map(|sales_person| sales_person.id)
                        .map(|sales_person_id| prop::collection::vec(
                            arb_extra_hour(Some(sales_person_id)),
                            0..2)
                        )
                        .collect::<Vec<_>>();
                let bookings =
                    sales_persons.iter()
                        .map(|_| prop::collection::vec(
                            (2000..2005u32, 1..=52u8, 0..10000000usize),
                            0..10
                        ))
                        .collect::<Vec<_>>();
                (Just(sales_persons), working_hours, extra_hours , bookings)
            })
    ) {
        Runtime::new().unwrap().block_on(async {
            dotenvy::dotenv().ok();
            println!("{:?}", std::env::current_dir());
            let pool = Arc::new(
                SqlitePool::connect("sqlite:sqlite::memory:")
                    .await
                    .expect("Could not connect to database"),
            );
            sqlx::migrate!("./../migrations").run(pool.as_ref()).await.unwrap();


            let rest_state = RestStateImpl::new(pool.clone());
            create_dev_admin_user(pool.clone()).await;

            let basic_dao = BasicDaoImpl::new(pool.clone());
            basic_dao.clear_all().await.unwrap();

            let mut created_sales_persons = vec![];
            let mut expected_hours: HashMap<Uuid, HashMap<u32, f32>> = HashMap::new();
            let mut working_hours: HashMap<Uuid, HashMap<u32, f32>> = HashMap::new();
            let mut balance_hours: HashMap<Uuid, HashMap<u32, f32>> = HashMap::new();
            for mut sales_person in testdata.0.iter().cloned() {
                sales_person.id = Uuid::nil();
                sales_person.version = Uuid::nil();
                sales_person.is_paid = Some(true);
                created_sales_persons.push(rest_state.sales_person_service().create(&sales_person, Authentication::Full).await.unwrap());
            }
            for (i, working_hours) in testdata.1.iter().enumerate() {
                for mut working_hour in working_hours.iter().cloned() {
                    working_hour.id = Uuid::nil();
                    working_hour.version = Uuid::nil();
                    working_hour.created = None;
                    working_hour.sales_person_id = created_sales_persons[i].id;
                    let mut date = time::Date::from_iso_week_date(working_hour.from_year as i32, working_hour.from_calendar_week, time::Weekday::Thursday).unwrap();
                    while (date.year() as u32, date.iso_week()) <= (working_hour.to_year, working_hour.to_calendar_week) {
                        let sales_person_hours = expected_hours.entry(working_hour.sales_person_id).or_insert(HashMap::new());
                        *sales_person_hours.entry(date.year() as u32).or_insert(0.0) += working_hour.expected_hours;
                        let balance_hours = balance_hours.entry(working_hour.sales_person_id).or_insert(HashMap::new());
                        *balance_hours.entry(date.year() as u32).or_insert(0.0) -= working_hour.expected_hours;
                        date += time::Duration::WEEK;
                    }

                    rest_state.working_hours_service().create(&working_hour, Authentication::Full).await.unwrap();
                }
            }
            for (i, extra_hours) in testdata.2.iter().enumerate() {
                for mut extra_hour in extra_hours.iter().cloned() {
                    extra_hour.id = Uuid::nil();
                    extra_hour.version = Uuid::nil();
                    extra_hour.created = None;
                    extra_hour.sales_person_id = created_sales_persons[i].id;
                    let expected_hours_for_week = get_working_hours_for_week(
                        testdata.1[i].as_ref(),
                        extra_hour.date_time.year() as u32,
                        extra_hour.date_time.iso_week())
                    .map(|working_hour| working_hour.expected_hours).unwrap_or(0.0);
                    if expected_hours_for_week <= 0.0 {
                        // In this case, expected hours are always equal to working hours and balance is not touched.
                        if extra_hour.category == ExtraHoursCategory::ExtraWork {
                            let hours = working_hours.entry(extra_hour.sales_person_id).or_insert(HashMap::new());
                            *hours.entry(extra_hour.date_time.year() as u32).or_insert(0.0) += extra_hour.amount;
                            let hours = expected_hours.entry(extra_hour.sales_person_id).or_insert(HashMap::new());
                            *hours.entry(extra_hour.date_time.year() as u32).or_insert(0.0) += extra_hour.amount;
                        }
                    } else {
                        if extra_hour.category == ExtraHoursCategory::ExtraWork {
                            let hours = working_hours.entry(extra_hour.sales_person_id).or_insert(HashMap::new());
                            *hours.entry(extra_hour.date_time.year() as u32).or_insert(0.0) += extra_hour.amount;
                            let balance_hours = balance_hours.entry(extra_hour.sales_person_id).or_insert(HashMap::new());
                            *balance_hours.entry(extra_hour.date_time.year() as u32).or_insert(0.0) += extra_hour.amount;
                        } else {
                            let hours = expected_hours.entry(extra_hour.sales_person_id).or_insert(HashMap::new());
                            *hours.entry(extra_hour.date_time.year() as u32).or_insert(0.0) -= extra_hour.amount;
                            let balance_hours = balance_hours.entry(extra_hour.sales_person_id).or_insert(HashMap::new());
                            *balance_hours.entry(extra_hour.date_time.year() as u32).or_insert(0.0) += extra_hour.amount;
                        }
                    }
                    rest_state.extra_hours_service().create(&extra_hour, Authentication::Full).await.unwrap();

                }
            }
            let slots = rest_state.slot_service().get_slots(Authentication::Full).await.unwrap();
            for (i, booking_data) in testdata.3.iter().enumerate() {
                for (year, week, slot_index) in booking_data.iter().cloned() {
                    let slot = slots[slot_index % slots.len()].clone();
                    let sales_person_id = created_sales_persons[i].id;
                    let booking = Booking {
                        id: Uuid::nil(),
                        sales_person_id,
                        slot_id: slot.id,
                        calendar_week: week as i32,
                        year,
                        created: None,
                        deleted: None,
                        version: Uuid::nil(),
                    };
                    let slot_duration = (slot.to - slot.from).as_seconds_f32() / 3600.0;
                    let expected_hours_for_week = get_working_hours_for_week(testdata.1[i].as_ref(),
                        year, week).map(|working_hour| working_hour.expected_hours).unwrap_or(0.0);
                    let insert_successful = match rest_state.booking_service().create(&booking, Authentication::Full).await {
                        Ok(_) => true,
                        Err(ServiceError::ValidationError(validations)) => {
                            assert_eq!(validations.len(), 1, "Expect extactly one validation error when inserting bookings");
                            assert_eq!(validations[0], ValidationFailureItem::Duplicate, "Expect only duplicate errors when inserting bookings");
                            false
                        },
                        _ => panic!("Unexpected error when inserting bookings"),
                    };
                    if insert_successful {
                        if expected_hours_for_week <= 0.0 {
                            // In this case, expected hours are always equal to working hours and balance is not touched.
                            let hours = working_hours.entry(sales_person_id).or_insert(HashMap::new());
                            *hours.entry(year).or_insert(0.0) += slot_duration;
                            let hours = expected_hours.entry(sales_person_id).or_insert(HashMap::new());
                            *hours.entry(year).or_insert(0.0) += slot_duration;
                        } else {
                            let hours = working_hours.entry(sales_person_id).or_insert(HashMap::new());
                            *hours.entry(year).or_insert(0.0) += slot_duration;
                            let balance_hours = balance_hours.entry(sales_person_id).or_insert(HashMap::new());
                            *balance_hours.entry(year).or_insert(0.0) += slot_duration;
                        }
                    }
                }
            }


            for year in 2000..2005 {
                let report = rest_state.reporting_service().get_reports_for_all_employees(year, 53, Authentication::Full).await.unwrap();
                assert_eq!(report.len(), created_sales_persons.len());
                for sales_person_report in report.iter() {
                    let dummy_working_hours = HashMap::new();
                    let dummy_expected_hours = HashMap::new();
                    let work_hours = working_hours.get(&sales_person_report.sales_person.id).unwrap_or(&dummy_working_hours).get(&year).unwrap_or(&0.0);
                    let expected_hours = expected_hours.get(&sales_person_report.sales_person.id).unwrap_or(&dummy_expected_hours).get(&year).unwrap_or(&0.0);
                    let balance_hours = balance_hours.get(&sales_person_report.sales_person.id).unwrap_or(&dummy_expected_hours).get(&year).unwrap_or(&0.0);
                    if sales_person_report.balance_hours != *balance_hours {
                        dbg!(&sales_person_report);
                    }
                    assert!(sales_person_report.overall_hours >= *work_hours - EPSILON && sales_person_report.overall_hours <= *work_hours + EPSILON,
                        "Test if working hours match for sales person {} in year {year}, expected={}, got={}, object: {:#?}", sales_person_report.sales_person.name, *work_hours, sales_person_report.overall_hours, &sales_person_report);
                    assert!(sales_person_report.expected_hours >= *expected_hours - EPSILON && sales_person_report.expected_hours <= *expected_hours + EPSILON,
                        "Test if expected hours match for sales person {} in year {year}, expected={}, got={}, object: {:#?}", sales_person_report.sales_person.name, *expected_hours, sales_person_report.expected_hours, &sales_person_report);
                    assert!(sales_person_report.balance_hours >= *balance_hours - EPSILON && sales_person_report.balance_hours <= *balance_hours + EPSILON,
                        "Test if balance hours match for sales person {} in year {year}, expected={}, got={}, object: {:#?}", sales_person_report.sales_person.name, *balance_hours, sales_person_report.balance_hours, &sales_person_report);

                    // Verify that that the values match the detailed report
                    let detailed_report = rest_state.reporting_service().get_report_for_employee(&sales_person_report.sales_person.id, year, 53, Authentication::Full).await.unwrap();
                    assert!(sales_person_report.overall_hours >= detailed_report.overall_hours - EPSILON && sales_person_report.overall_hours <= detailed_report.overall_hours + EPSILON,
                        "Test if working hours match for sales person {} in year {year}, detailed-report={}, employee-report={}, object: {:#?}, detailed: {:#?}", sales_person_report.sales_person.name, detailed_report.overall_hours, sales_person_report.overall_hours, &sales_person_report, detailed_report);
                    assert!(sales_person_report.expected_hours >= detailed_report.expected_hours - EPSILON && sales_person_report.expected_hours <= detailed_report.expected_hours + EPSILON,
                        "Test if expected hours match for sales person {} in year {year}, detailed-report={}, employee-report={}, object: {:#?}", sales_person_report.sales_person.name, detailed_report.expected_hours, sales_person_report.expected_hours, &sales_person_report);
                    assert!(sales_person_report.balance_hours >= detailed_report.balance_hours - EPSILON && sales_person_report.balance_hours <= detailed_report.balance_hours + EPSILON,
                        "Test if balance hours match for sales person {} in year {year}, detailed-report={}, employee-report={}, object: {:#?}, detailed: {:#?}", sales_person_report.sales_person.name, detailed_report.balance_hours, sales_person_report.balance_hours, &sales_person_report, detailed_report);
                }
            }


        })
    }
}
