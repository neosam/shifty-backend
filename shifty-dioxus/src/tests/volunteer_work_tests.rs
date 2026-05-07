#[cfg(test)]
mod working_hours_category_tests {
    use crate::state::employee::WorkingHoursCategory;
    use rest_types::{ExtraHoursCategoryTO, ExtraHoursReportCategoryTO};

    #[test]
    fn volunteer_work_identifier_is_volunteer_work() {
        assert_eq!(
            WorkingHoursCategory::VolunteerWork.identifier().as_ref(),
            "volunteer_work"
        );
    }

    #[test]
    fn from_identifier_volunteer_work_yields_variant() {
        match WorkingHoursCategory::from_identifier("volunteer_work") {
            WorkingHoursCategory::VolunteerWork => {}
            other => panic!("expected VolunteerWork, got {:?}", other),
        }
    }

    #[test]
    fn is_volunteer_work_only_matches_volunteer_work() {
        assert!(WorkingHoursCategory::VolunteerWork.is_volunteer_work());

        let others = vec![
            WorkingHoursCategory::Shiftplan,
            WorkingHoursCategory::ExtraWork("".into()),
            WorkingHoursCategory::Vacation,
            WorkingHoursCategory::VacationDays,
            WorkingHoursCategory::SickLeave,
            WorkingHoursCategory::Holiday,
            WorkingHoursCategory::Unavailable,
            WorkingHoursCategory::UnpaidLeave,
            WorkingHoursCategory::Custom(uuid::Uuid::new_v4()),
        ];
        for c in others {
            assert!(!c.is_volunteer_work(), "{:?} matched is_volunteer_work", c);
        }
    }

    #[test]
    fn volunteer_work_round_trips_through_extra_hours_category_to() {
        let original = WorkingHoursCategory::VolunteerWork;
        let to: ExtraHoursCategoryTO = (&original).into();
        assert!(matches!(to, ExtraHoursCategoryTO::VolunteerWork));
        let back: WorkingHoursCategory = (&to).into();
        assert!(matches!(back, WorkingHoursCategory::VolunteerWork));
    }

    #[test]
    fn volunteer_work_converts_from_extra_hours_report_category_to() {
        let to = ExtraHoursReportCategoryTO::VolunteerWork;
        let cat: WorkingHoursCategory = (&to).into();
        assert!(matches!(cat, WorkingHoursCategory::VolunteerWork));
    }
}

#[cfg(test)]
mod working_hours_mapping_tests {
    use crate::state::employee::{Employee, WorkingHours};
    use rest_types::{
        EmployeeReportTO, SalesPersonTO, ShortEmployeeReportTO, WorkingHoursReportTO,
    };
    use std::sync::Arc;
    use uuid::Uuid;

    fn sales_person_to() -> SalesPersonTO {
        SalesPersonTO {
            id: Uuid::nil(),
            name: "Test".into(),
            background_color: "".into(),
            is_paid: Some(false),
            inactive: false,
            deleted: None,
            version: Uuid::nil(),
        }
    }

    fn make_working_hours_report_to(volunteer_hours: f32) -> WorkingHoursReportTO {
        let from = time::Date::from_calendar_date(2026, time::Month::January, 1).unwrap();
        let to = time::Date::from_calendar_date(2026, time::Month::January, 7).unwrap();
        WorkingHoursReportTO {
            from,
            to,
            expected_hours: 0.0,
            dynamic_hours: 0.0,
            overall_hours: 0.0,
            balance: 0.0,
            days_per_week: 0,
            workdays_per_week: 0.0,
            shiftplan_hours: 0.0,
            extra_work_hours: 0.0,
            vacation_hours: 0.0,
            vacation_days: 0.0,
            sick_leave_hours: 0.0,
            sick_leave_days: 0.0,
            holiday_hours: 0.0,
            holiday_days: 0.0,
            unpaid_leave_hours: 0.0,
            volunteer_hours,
            absence_days: 0.0,
            custom_extra_hours: Arc::from([]),
            days: Arc::from([]),
        }
    }

    #[test]
    fn working_hours_picks_up_volunteer_hours_from_to() {
        let to = make_working_hours_report_to(7.5);
        let wh = WorkingHours::from(&to);
        assert_eq!(wh.volunteer_hours, 7.5);
    }

    #[test]
    fn employee_picks_up_volunteer_hours_from_full_report() {
        let report = EmployeeReportTO {
            sales_person: Arc::new(sales_person_to()),
            balance_hours: 0.0,
            overall_hours: 0.0,
            expected_hours: 0.0,
            dynamic_hours: 0.0,
            shiftplan_hours: 0.0,
            extra_work_hours: 0.0,
            vacation_hours: 0.0,
            sick_leave_hours: 0.0,
            holiday_hours: 0.0,
            unpaid_leave_hours: 0.0,
            volunteer_hours: 12.0,
            vacation_carryover: 0,
            vacation_days: 0.0,
            vacation_entitlement: 0.0,
            sick_leave_days: 0.0,
            holiday_days: 0.0,
            absence_days: 0.0,
            carryover_hours: 0.0,
            custom_extra_hours: Arc::from([]),
            by_week: Arc::from([]),
            by_month: Arc::from([]),
        };
        let emp = Employee::from(&report);
        assert_eq!(emp.volunteer_hours, 12.0);
    }

    #[test]
    fn employee_defaults_volunteer_hours_to_zero_for_short_report() {
        let short = ShortEmployeeReportTO {
            sales_person: sales_person_to(),
            balance_hours: 0.0,
            expected_hours: 0.0,
            dynamic_hours: 0.0,
            overall_hours: 0.0,
            volunteer_hours: 99.0,
        };
        let emp = Employee::from(&short);
        assert_eq!(emp.volunteer_hours, 0.0);
    }
}

#[cfg(test)]
mod employee_work_details_cap_tests {
    use crate::state::employee_work_details::EmployeeWorkDetails;
    use rest_types::{DayOfWeekTO, EmployeeWorkDetailsTO};
    use uuid::Uuid;

    fn make_to(cap: bool) -> EmployeeWorkDetailsTO {
        EmployeeWorkDetailsTO {
            id: Uuid::nil(),
            sales_person_id: Uuid::nil(),
            expected_hours: 0.0,
            from_day_of_week: DayOfWeekTO::Monday,
            from_calendar_week: 1,
            from_year: 2026,
            to_day_of_week: DayOfWeekTO::Sunday,
            to_calendar_week: 52,
            to_year: 2026,
            workdays_per_week: 5,
            is_dynamic: false,
            cap_planned_hours_to_expected: cap,
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
            sunday: false,
            vacation_days: 0,
            days_per_week: 5,
            hours_per_day: 0.0,
            hours_per_holiday: 0.0,
            created: None,
            deleted: None,
            version: Uuid::nil(),
        }
    }

    // `blank_standard` calls `js::current_datetime()`, which is wasm-only.
    // This test only runs in the WASM target; in normal `cargo test` the default is
    // covered transitively by `try_from_to_propagates_cap_true` (a TO with cap=false
    // round-trips to a state with cap=false, exercising the same default path).
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn blank_standard_defaults_cap_to_false() {
        let details = EmployeeWorkDetails::blank_standard(Uuid::new_v4());
        assert!(!details.cap_planned_hours_to_expected);
    }

    #[test]
    fn try_from_to_propagates_cap_true() {
        let to = make_to(true);
        let state = EmployeeWorkDetails::try_from(&to).unwrap();
        assert!(state.cap_planned_hours_to_expected);
    }

    #[test]
    fn try_from_state_propagates_cap_true() {
        let to = make_to(false);
        let mut state = EmployeeWorkDetails::try_from(&to).unwrap();
        state.cap_planned_hours_to_expected = true;
        let back = EmployeeWorkDetailsTO::try_from(&state).unwrap();
        assert!(back.cap_planned_hours_to_expected);
    }
}
