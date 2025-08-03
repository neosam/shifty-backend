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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShiftyDate {
    year: u32,
    week: u8,
    day_of_week: DayOfWeek,
}

impl ShiftyDate {
    pub fn new(year: u32, week: u8, day_of_week: DayOfWeek) -> Result<Self, ShiftyDateUtilsError> {
        time::Date::from_iso_week_date(year as i32, week, day_of_week.into())?;
        Ok(Self {
            year,
            week,
            day_of_week,
        })
    }

    pub fn from_ymd(year: u32, month: u8, day: u8) -> Result<Self, ShiftyDateUtilsError> {
        let date = time::Date::from_calendar_date(year as i32, month.try_into()?, day)
            .map_err(ShiftyDateUtilsError::DateError)?;
        let (week_year, iso_week, weekday) = date.to_iso_week_date();
        Ok(Self {
            year: week_year as u32,
            week: iso_week,
            day_of_week: DayOfWeek::from(weekday),
        })
    }

    pub fn first_day_in_year(year: u32) -> Self {
        ShiftyDate::from_ymd(year, 1, 1)
            .expect("Every year has a first of January, right?  Right?? :-O")
    }

    pub fn last_day_in_year(year: u32) -> Self {
        ShiftyDate::from_ymd(year, 12, 31)
            .expect("Every year has a 31st of December, right?  Right?? :-O")
    }

    pub fn to_date(&self) -> time::Date {
        time::Date::from_iso_week_date(self.year as i32, self.week, self.day_of_week.into())
            .expect("Date values should be checked during creation")
    }

    pub fn from_date(date: time::Date) -> Self {
        let (year, week, day_of_week) = date.to_iso_week_date();
        Self {
            year: year as u32,
            week,
            day_of_week: day_of_week.into(),
        }
    }

    pub fn as_shifty_week(&self) -> ShiftyWeek {
        ShiftyWeek::new(self.year, self.week)
    }

    pub fn min(&self, o: ShiftyDate) -> ShiftyDate {
        if self < &o { *self } else { o }
    }

    pub fn max(&self, o: ShiftyDate) -> ShiftyDate {
        if self > &o { *self } else { o }
    }

    pub fn year(&self) -> u32 {
        self.year
    }

    pub fn week(&self) -> u8 {
        self.week
    }

    pub fn day_of_week(&self) -> DayOfWeek {
        self.day_of_week
    }
}

impl TryFrom<(u32, u8, DayOfWeek)> for ShiftyDate {
    type Error = ShiftyDateUtilsError;

    fn try_from(tuple: (u32, u8, DayOfWeek)) -> Result<ShiftyDate, ShiftyDateUtilsError> {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}

impl From<ShiftyDate> for (u32, u8, DayOfWeek) {
    fn from(date: ShiftyDate) -> Self {
        (date.year, date.week, date.day_of_week)
    }
}

impl From<time::Date> for ShiftyDate {
    fn from(date: time::Date) -> Self {
        ShiftyDate::from_date(date)
    }
}

impl From<time::PrimitiveDateTime> for ShiftyDate {
    fn from(date_time: time::PrimitiveDateTime) -> Self {
        ShiftyDate::from_date(date_time.date())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShiftyWeek {
    pub year: u32,
    pub week: u8,
}

impl ShiftyWeek {
    pub fn new(year: u32, week: u8) -> Self {
        if week > time::util::weeks_in_year(year as i32) {
            Self {
                year: year + 1,
                week: week - time::util::weeks_in_year(year as i32),
            }
        } else {
            Self { year, week }
        }
    }

    pub fn next(&self) -> Self {
        if time::util::weeks_in_year(self.year as i32) == self.week {
            Self::new(self.year + 1, 1)
        } else {
            Self::new(self.year, self.week + 1)
        }
    }

    pub fn iter_until(&self, end: &Self) -> ShiftyWeekIterator {
        ShiftyWeekIterator::new(*self, *end)
    }

    pub fn as_date(&self, weekday: DayOfWeek) -> ShiftyDate {
        ShiftyDate::new(self.year, self.week, weekday).expect("Valid week and weekday")
    }
}

impl From<(u32, u8)> for ShiftyWeek {
    fn from(tuple: (u32, u8)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}
impl From<ShiftyWeek> for (u32, u8) {
    fn from(week: ShiftyWeek) -> Self {
        (week.year, week.week)
    }
}

pub struct ShiftyWeekIterator {
    current: ShiftyWeek,
    end: ShiftyWeek,
}

impl ShiftyWeekIterator {
    pub fn new(start: ShiftyWeek, end: ShiftyWeek) -> Self {
        Self {
            current: start,
            end,
        }
    }
}

impl Iterator for ShiftyWeekIterator {
    type Item = ShiftyWeek;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.end {
            None
        } else {
            let next = self.current;
            self.current = self.current.next();
            Some(next)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shifty_date_creation() {
        let date = ShiftyDate::new(2023, 1, DayOfWeek::Monday).unwrap();
        assert_eq!(date.year, 2023);
        assert_eq!(date.week, 1);
        assert_eq!(date.day_of_week, DayOfWeek::Monday);
    }

    #[test]
    fn test_shifty_week_next() {
        let week = ShiftyWeek::new(2023, 52);
        let next_week = week.next();
        assert_eq!(next_week.year, 2024);
        assert_eq!(next_week.week, 1);
    }

    #[test]
    fn test_shifty_date_min() {
        let date1 = ShiftyDate::new(2023, 1, DayOfWeek::Monday).unwrap();
        let date2 = ShiftyDate::new(2023, 1, DayOfWeek::Tuesday).unwrap();
        let date3 = ShiftyDate::new(2023, 30, DayOfWeek::Monday).unwrap();
        let date4 = ShiftyDate::new(2024, 1, DayOfWeek::Monday).unwrap();
        let min_date1 = date1.min(date2);
        let min_date2 = date1.min(date3);
        let min_date3 = date1.min(date4);
        assert_eq!(min_date1, date1);
        assert_eq!(min_date2, date1);
        assert_eq!(min_date3, date1);
    }

    #[test]
    fn test_shifty_date_max() {
        let date1 = ShiftyDate::new(2023, 1, DayOfWeek::Monday).unwrap();
        let date2 = ShiftyDate::new(2023, 1, DayOfWeek::Tuesday).unwrap();
        let date3 = ShiftyDate::new(2023, 30, DayOfWeek::Monday).unwrap();
        let date4 = ShiftyDate::new(2024, 1, DayOfWeek::Monday).unwrap();
        let max_date1 = date1.max(date2);
        let max_date2 = date1.max(date3);
        let max_date3 = date1.max(date4);
        assert_eq!(max_date1, date2);
        assert_eq!(max_date2, date3);
        assert_eq!(max_date3, date4);
    }
}
