use std::rc::Rc;

use rest_types::BookingLogTO;
use time::PrimitiveDateTime;

use super::shiftplan::Weekday;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookingLog {
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: Weekday,
    pub sales_person_name: Rc<str>,
    pub time_from: time::Time,
    pub time_to: time::Time,
    pub created: PrimitiveDateTime,
    pub deleted: Option<PrimitiveDateTime>,
    pub created_by: Rc<str>,
    pub deleted_by: Option<Rc<str>>,
}

impl From<&BookingLogTO> for BookingLog {
    fn from(log: &BookingLogTO) -> Self {
        Self {
            year: log.year,
            calendar_week: log.calendar_week,
            day_of_week: log.day_of_week.into(),
            sales_person_name: log.name.to_string().into(),
            time_from: log.time_from,
            time_to: log.time_to,
            created: log.created,
            deleted: log.deleted,
            created_by: log.created_by.to_string().into(),
            deleted_by: log.deleted_by.as_ref().map(|s| s.to_string().into()),
        }
    }
}
