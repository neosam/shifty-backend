use rest_types::WeeklySummaryTO;

#[derive(Debug, Clone, PartialEq)]
pub struct WeeklySummary {
    pub week: u8,
    pub year: u32,
    pub available_hours: f32,
    pub required_hours: f32,
}

impl From<&WeeklySummaryTO> for WeeklySummary {
    fn from(summary: &WeeklySummaryTO) -> Self {
        Self {
            week: summary.week,
            year: summary.year,
            available_hours: summary.overall_available_hours,
            required_hours: summary.required_hours,
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
