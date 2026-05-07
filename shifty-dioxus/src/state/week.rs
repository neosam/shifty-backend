use crate::error::ShiftyError;

pub struct Week {
    pub year: u32,
    pub week: u8,
}

impl Week {
    pub fn monday(&self) -> Result<time::Date, ShiftyError> {
        let date =
            time::Date::from_iso_week_date(self.year as i32, self.week, time::Weekday::Monday)?;
        Ok(date)
    }
    pub fn sunday(&self) -> Result<time::Date, ShiftyError> {
        let date =
            time::Date::from_iso_week_date(self.year as i32, self.week, time::Weekday::Sunday)?;
        Ok(date)
    }
}
