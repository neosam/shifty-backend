use rest_types::WeeklySummaryTO;

#[derive(Debug, Clone, PartialEq)]
pub struct WeeklySummary {
    pub week: u8,
    pub year: u32,
    pub available_hours: f32,
    pub required_hours: f32,
    pub monday_available_hours: f32,
    pub tuesday_available_hours: f32,
    pub wednesday_available_hours: f32,
    pub thursday_available_hours: f32,
    pub friday_available_hours: f32,
    pub saturday_available_hours: f32,
    pub sunday_available_hours: f32,
}

impl From<&WeeklySummaryTO> for WeeklySummary {
    fn from(summary: &WeeklySummaryTO) -> Self {
        Self {
            week: summary.week,
            year: summary.year,
            available_hours: summary.overall_available_hours,
            required_hours: summary.required_hours,
            monday_available_hours: summary.monday_available_hours,
            tuesday_available_hours: summary.tuesday_available_hours,
            wednesday_available_hours: summary.wednesday_available_hours,
            thursday_available_hours: summary.thursday_available_hours,
            friday_available_hours: summary.friday_available_hours,
            saturday_available_hours: summary.saturday_available_hours,
            sunday_available_hours: summary.sunday_available_hours,
        }
    }
}

impl WeeklySummary {
    pub fn monday_date(&self) -> time::Date {
        time::Date::from_iso_week_date(self.year as i32, self.week, time::Weekday::Monday).unwrap()
    }
    pub fn sunday_date(&self) -> time::Date {
        time::Date::from_iso_week_date(self.year as i32, self.week, time::Weekday::Sunday).unwrap()
    }
}
