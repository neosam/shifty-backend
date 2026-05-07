use rest_types::SalesPersonUnavailableTO;
use uuid::Uuid;

use super::Weekday;

pub struct SalesPersonUnavailable {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub year: u32,
    pub week: u8,
    pub day_of_week: Weekday,
    pub version: Uuid,
}
impl From<&SalesPersonUnavailableTO> for SalesPersonUnavailable {
    fn from(sales_person_available: &SalesPersonUnavailableTO) -> Self {
        Self {
            id: sales_person_available.id,
            sales_person_id: sales_person_available.sales_person_id,
            year: sales_person_available.year,
            week: sales_person_available.calendar_week as u8,
            day_of_week: sales_person_available.day_of_week.into(),
            version: sales_person_available.version,
        }
    }
}
impl From<&SalesPersonUnavailable> for SalesPersonUnavailableTO {
    fn from(sales_person_available: &SalesPersonUnavailable) -> Self {
        Self {
            id: sales_person_available.id,
            sales_person_id: sales_person_available.sales_person_id,
            year: sales_person_available.year,
            calendar_week: sales_person_available.week,
            day_of_week: (&sales_person_available.day_of_week).into(),
            created: None,
            deleted: None,
            version: sales_person_available.version,
        }
    }
}
