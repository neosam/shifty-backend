use rest_types::EmployeeWorkDetailsTO;
use time::error::ComponentRange;
use uuid::Uuid;

use crate::{base_types::ImStr, js};

#[derive(PartialEq, Clone)]
pub struct WorkingHoursMini {
    pub sales_person_id: Uuid,
    pub sales_person_name: ImStr,
    pub expected_hours: f32,
    pub dynamic_hours: f32,
    pub actual_hours: f32,
    pub balance_hours: f32,
    pub background_color: ImStr,
    pub committed_voluntary_hours: f32,
}

impl Default for WorkingHoursMini {
    fn default() -> Self {
        Self {
            sales_person_id: Uuid::nil(),
            sales_person_name: "".into(),
            expected_hours: 0.0,
            dynamic_hours: 0.0,
            actual_hours: 0.0,
            balance_hours: 0.0,
            background_color: "#cccccc".into(),
            committed_voluntary_hours: 0.0,
        }
    }
}

#[cfg(test)]
mod working_hours_mini_tests {
    use super::*;

    #[test]
    fn working_hours_mini_default_background_color_is_neutral_gray() {
        let mini = WorkingHoursMini::default();
        assert_eq!(mini.background_color.as_str(), "#cccccc");
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct EmployeeWorkDetails {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub expected_hours: f32,
    pub from: time::Date,
    pub to: time::Date,
    pub workdays_per_week: u8,

    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,

    pub dynamic: bool,

    pub cap_planned_hours_to_expected: bool,

    pub committed_voluntary: f32,

    pub vacation_days: u8,

    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

impl EmployeeWorkDetails {
    pub fn blank_standard(sales_person_id: Uuid) -> EmployeeWorkDetails {
        EmployeeWorkDetails {
            id: Uuid::nil(),
            sales_person_id,
            expected_hours: 0.0,
            from: js::current_datetime().date(),
            to: js::current_datetime().date(),
            workdays_per_week: 6,

            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: true,
            sunday: false,

            dynamic: false,

            cap_planned_hours_to_expected: false,

            committed_voluntary: 0.0,

            vacation_days: 0,

            created: None,
            deleted: None,
            version: Uuid::nil(),
        }
    }

    pub fn from_as_calendar_week(&self) -> (u32, u8, time::Weekday) {
        let (year, week, day) = self.from.to_iso_week_date();
        (year as u32, week, day)
    }

    pub fn to_as_calendar_week(&self) -> (u32, u8, time::Weekday) {
        let (year, week, day) = self.to.to_iso_week_date();
        (year as u32, week, day)
    }

    pub fn days_per_week(&self) -> u8 {
        let mut days = 0;
        if self.monday {
            days += 1;
        }
        if self.tuesday {
            days += 1;
        }
        if self.wednesday {
            days += 1;
        }
        if self.thursday {
            days += 1;
        }
        if self.friday {
            days += 1;
        }
        if self.saturday {
            days += 1;
        }
        if self.sunday {
            days += 1;
        }
        days
    }

    pub fn vacation_day_in_hours(&self) -> f32 {
        self.expected_hours / self.workdays_per_week as f32
    }
    pub fn holiday_hours(&self) -> f32 {
        self.expected_hours / self.days_per_week() as f32
    }
}

impl TryFrom<&EmployeeWorkDetailsTO> for EmployeeWorkDetails {
    type Error = ComponentRange;
    fn try_from(details: &EmployeeWorkDetailsTO) -> Result<Self, ComponentRange> {
        Ok(Self {
            id: details.id,
            sales_person_id: details.sales_person_id,
            expected_hours: details.expected_hours,
            from: time::Date::from_iso_week_date(
                details.from_year as i32,
                details.from_calendar_week,
                details.from_day_of_week.into(),
            )?,
            to: time::Date::from_iso_week_date(
                details.to_year as i32,
                details.to_calendar_week,
                details.to_day_of_week.into(),
            )?,
            workdays_per_week: details.workdays_per_week,

            monday: details.monday,
            tuesday: details.tuesday,
            wednesday: details.wednesday,
            thursday: details.thursday,
            friday: details.friday,
            saturday: details.saturday,
            sunday: details.sunday,

            dynamic: details.is_dynamic,

            cap_planned_hours_to_expected: details.cap_planned_hours_to_expected,

            committed_voluntary: details.committed_voluntary,

            vacation_days: details.vacation_days,

            created: details.created,
            deleted: details.deleted,
            version: details.version,
        })
    }
}

impl TryFrom<&EmployeeWorkDetails> for EmployeeWorkDetailsTO {
    type Error = ComponentRange;
    fn try_from(details: &EmployeeWorkDetails) -> Result<Self, ComponentRange> {
        let (from_year, from_week, _) = details.from.to_iso_week_date();
        let (to_year, to_week, _) = details.to.to_iso_week_date();
        Ok(Self {
            id: details.id,
            sales_person_id: details.sales_person_id,
            expected_hours: details.expected_hours,
            from_day_of_week: details.from.weekday().into(),
            from_year: from_year as u32,
            from_calendar_week: from_week,
            to_day_of_week: details.to.weekday().into(),
            to_year: to_year as u32,
            to_calendar_week: to_week,
            workdays_per_week: details.workdays_per_week,

            monday: details.monday,
            tuesday: details.tuesday,
            wednesday: details.wednesday,
            thursday: details.thursday,
            friday: details.friday,
            saturday: details.saturday,
            sunday: details.sunday,

            is_dynamic: details.dynamic,

            cap_planned_hours_to_expected: details.cap_planned_hours_to_expected,
            // D-02: threaded in Phase 17 — committed_voluntary round-trips faithfully.
            committed_voluntary: details.committed_voluntary,

            vacation_days: details.vacation_days,

            days_per_week: details.days_per_week(),
            hours_per_day: details.vacation_day_in_hours(),
            hours_per_holiday: details.holiday_hours(),

            created: details.created,
            deleted: details.deleted,
            version: details.version,
        })
    }
}

#[cfg(test)]
mod employee_work_details_tests {
    use super::*;
    use rest_types::{DayOfWeekTO, EmployeeWorkDetailsTO};

    fn make_to(committed_voluntary: f32) -> EmployeeWorkDetailsTO {
        EmployeeWorkDetailsTO {
            id: Uuid::nil(),
            sales_person_id: Uuid::nil(),
            expected_hours: 40.0,
            from_day_of_week: DayOfWeekTO::Monday,
            from_calendar_week: 1,
            from_year: 2024,
            to_day_of_week: DayOfWeekTO::Sunday,
            to_calendar_week: 52,
            to_year: 2024,
            workdays_per_week: 5,
            is_dynamic: false,
            cap_planned_hours_to_expected: false,
            committed_voluntary,
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
            sunday: false,
            vacation_days: 25,
            days_per_week: 5,
            hours_per_day: 8.0,
            hours_per_holiday: 8.0,
            created: None,
            deleted: None,
            version: Uuid::nil(),
        }
    }

    /// D-02: committed_voluntary round-trips faithfully through
    /// EmployeeWorkDetails → EmployeeWorkDetailsTO → EmployeeWorkDetails.
    #[test]
    fn committed_voluntary_round_trip() {
        let to = make_to(3.5);
        let details: EmployeeWorkDetails = EmployeeWorkDetails::try_from(&to)
            .expect("TO→State conversion should succeed");
        assert!(
            (details.committed_voluntary - 3.5).abs() < 0.001,
            "after TO→State: expected committed_voluntary 3.5, got {}",
            details.committed_voluntary
        );

        let to2 = EmployeeWorkDetailsTO::try_from(&details)
            .expect("State→TO conversion should succeed");
        assert!(
            (to2.committed_voluntary - 3.5).abs() < 0.001,
            "after State→TO: expected committed_voluntary 3.5, got {}",
            to2.committed_voluntary
        );

        let details2: EmployeeWorkDetails = EmployeeWorkDetails::try_from(&to2)
            .expect("round-trip TO→State should succeed");
        assert!(
            (details2.committed_voluntary - 3.5).abs() < 0.001,
            "after round-trip: expected committed_voluntary 3.5, got {}",
            details2.committed_voluntary
        );
    }

    /// D-02: the TO→State direction maps committed_voluntary (previously missing).
    #[test]
    fn committed_voluntary_from_to_maps_field() {
        let to = make_to(7.25);
        let details: EmployeeWorkDetails = EmployeeWorkDetails::try_from(&to)
            .expect("TO→State conversion should succeed");
        assert!(
            (details.committed_voluntary - 7.25).abs() < 0.001,
            "TO→State: expected committed_voluntary 7.25, got {}",
            details.committed_voluntary
        );
    }
}
