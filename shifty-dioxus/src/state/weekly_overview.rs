use rest_types::WeeklySummaryTO;

#[derive(Debug, Clone, PartialEq)]
pub struct WeeklySummary {
    pub week: u8,
    pub year: u32,
    pub available_hours: f32,
}

impl From<&WeeklySummaryTO> for WeeklySummary {
    fn from(summary: &WeeklySummaryTO) -> Self {
        Self {
            week: summary.week,
            year: summary.year,
            available_hours: summary.overall_available_hours,
        }
    }
}
