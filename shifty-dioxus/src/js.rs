use js_sys::Date;

pub fn get_current_year() -> u32 {
    let date = Date::new_0();
    date.get_full_year() as u32
}

// Function to get the current week number based on ISO 8601
pub fn get_current_week() -> u8 {
    let date = Date::new_0();

    // Get the first day of the year
    let start_of_year = Date::new_with_year_month_day(date.get_full_year(), 0, 1);
    let day_of_year = ((date.get_time() - start_of_year.get_time()) / (1000.0 * 60.0 * 60.0 * 24.0))
        .floor() as u32
        + 1;

    // Calculate ISO week number
    let week_number = ((day_of_year + 6 - (start_of_year.get_day() as u32)) / 7);

    week_number as u8
}

// Function to get the date of the first day of the week
pub fn get_first_day_of_week(week: u8, year: u32) -> time::Date {
    let date = time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday).unwrap();
    date
}
