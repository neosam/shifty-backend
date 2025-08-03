use std::fmt::{Display, Formatter};
use thiserror::*;

use time::Weekday;

#[derive(Debug, Error)]
pub enum ShiftyDateUtilsError {
    #[error("Invalid date: {0}")]
    DateError(#[from] time::error::ComponentRange),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<Weekday> for DayOfWeek {
    fn from(weekday: Weekday) -> Self {
        match weekday {
            Weekday::Monday => Self::Monday,
            Weekday::Tuesday => Self::Tuesday,
            Weekday::Wednesday => Self::Wednesday,
            Weekday::Thursday => Self::Thursday,
            Weekday::Friday => Self::Friday,
            Weekday::Saturday => Self::Saturday,
            Weekday::Sunday => Self::Sunday,
        }
    }
}
impl From<DayOfWeek> for Weekday {
    fn from(day_of_week: DayOfWeek) -> Self {
        match day_of_week {
            DayOfWeek::Monday => Self::Monday,
            DayOfWeek::Tuesday => Self::Tuesday,
            DayOfWeek::Wednesday => Self::Wednesday,
            DayOfWeek::Thursday => Self::Thursday,
            DayOfWeek::Friday => Self::Friday,
            DayOfWeek::Saturday => Self::Saturday,
            DayOfWeek::Sunday => Self::Sunday,
        }
    }
}

impl Display for DayOfWeek {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DayOfWeek::Monday => "Monday",
                DayOfWeek::Tuesday => "Tuesday",
                DayOfWeek::Wednesday => "Wednesday",
                DayOfWeek::Thursday => "Thursday",
                DayOfWeek::Friday => "Friday",
                DayOfWeek::Saturday => "Saturday",
                DayOfWeek::Sunday => "Sunday",
            }
        )
    }
}

impl DayOfWeek {
    pub fn to_number(&self) -> u8 {
        match self {
            DayOfWeek::Monday => 1,
            DayOfWeek::Tuesday => 2,
            DayOfWeek::Wednesday => 3,
            DayOfWeek::Thursday => 4,
            DayOfWeek::Friday => 5,
            DayOfWeek::Saturday => 6,
            DayOfWeek::Sunday => 7,
        }
    }

    pub fn from_number(number: u8) -> Option<Self> {
        match number {
            1 => Some(DayOfWeek::Monday),
            2 => Some(DayOfWeek::Tuesday),
            3 => Some(DayOfWeek::Wednesday),
            4 => Some(DayOfWeek::Thursday),
            5 => Some(DayOfWeek::Friday),
            6 => Some(DayOfWeek::Saturday),
            7 => Some(DayOfWeek::Sunday),
            _ => None,
        }
    }
}

pub struct ShiftyDate {
    pub year: u32,
    pub week: u8,
    pub day_of_week: DayOfWeek,
}

impl ShiftyDate {
    pub fn new(year: u32, week: u8, day_of_week: DayOfWeek) -> Self {
        Self {
            year,
            week,
            day_of_week,
        }
    }

    pub fn to_date(&self) -> Result<time::Date, ShiftyDateUtilsError> {
        Ok(time::Date::from_iso_week_date(
            self.year as i32,
            self.week,
            self.day_of_week.into(),
        )?)
    }

    pub fn from_date(date: time::Date) -> Self {
        let (year, week, day_of_week) = date.to_iso_week_date();
        Self {
            year: year as u32,
            week,
            day_of_week: day_of_week.into(),
        }
    }
}
