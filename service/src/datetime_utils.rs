use std::{collections::BTreeMap, sync::Arc};

use time::{error::ComponentRange, Date};

use crate::slot::DayOfWeek;

pub fn calenar_week_to_date(
    year: i32,
    week: u8,
    day_of_week: DayOfWeek,
) -> Result<Date, ComponentRange> {
    Date::from_iso_week_date(year, week, day_of_week.into())
}

pub fn date_to_calendar_week(date: Date) -> (i32, u8, DayOfWeek) {
    let year = date.year();
    let week = date.iso_week();
    let day_of_week = DayOfWeek::from(date.weekday());

    (year, week, day_of_week)
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct CalendarWeek {
    pub year: i32,
    pub week: u8,
}

pub trait AsCalendarWeek {
    fn as_date(&self) -> CalendarWeek;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Month {
    pub year: i32,
    pub month: u8,
}
pub trait AsMonth {
    fn as_date(&self) -> Month;
}

pub fn group_by_calendar_week<T: AsCalendarWeek + Clone>(
    items: &[T],
) -> BTreeMap<CalendarWeek, Arc<[T]>> {
    let mut map = BTreeMap::new();
    for item in items {
        let calendar_week = item.as_date();
        map.entry(calendar_week)
            .or_insert_with(Vec::new)
            .push(item.to_owned());
    }
    map.into_iter()
        .map(|(calendar_week, items)| (calendar_week, items.into()))
        .collect()
}

pub fn group_by_month<T: AsMonth + Clone>(items: &[T]) -> BTreeMap<Month, Arc<[T]>> {
    let mut map = BTreeMap::new();
    for item in items {
        let month = item.as_date();
        map.entry(month)
            .or_insert_with(Vec::new)
            .push(item.to_owned());
    }
    map.into_iter()
        .map(|(month, items)| (month, items.into()))
        .collect()
}
