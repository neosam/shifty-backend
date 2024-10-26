use rest_types::EmployeeWorkDetailsTO;
use time::error::ComponentRange;
use uuid::Uuid;

use crate::{base_types::ImStr, js};

use super::shiftplan::SalesPerson;

#[derive(PartialEq, Clone)]
pub struct WorkingHoursMini {
    pub sales_person_id: Uuid,
    pub sales_person_name: ImStr,
    pub expected_hours: f32,
    pub actual_hours: f32,
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
                time::Weekday::Monday,
            )?,
            to: time::Date::from_iso_week_date(
                details.to_year as i32,
                details.to_calendar_week,
                time::Weekday::Sunday,
            )?,
            workdays_per_week: details.workdays_per_week,

            monday: details.monday,
            tuesday: details.tuesday,
            wednesday: details.wednesday,
            thursday: details.thursday,
            friday: details.friday,
            saturday: details.saturday,
            sunday: details.sunday,

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
            from_year: from_year as u32,
            from_calendar_week: from_week,
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
