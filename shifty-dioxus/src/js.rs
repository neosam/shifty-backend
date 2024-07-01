use js_sys::{wasm_bindgen::JsValue, Date};

pub fn get_current_year() -> u32 {
    let date = Date::new_0();
    date.get_full_year() as u32
}

// Function to get the current week number based on ISO 8601
pub fn get_current_week() -> u8 {
    current_datetime().iso_week()
}

pub fn js_date_to_primitive_date_time(date: &Date) -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(
        time::Date::from_calendar_date(
            date.get_full_year() as i32,
            time::Month::January.nth_next(date.get_month() as u8),
            date.get_date() as u8,
        )
        .unwrap(),
        time::Time::from_hms(
            date.get_hours() as u8,
            date.get_minutes() as u8,
            date.get_seconds() as u8,
        )
        .unwrap(),
    )
}

pub fn current_datetime() -> time::PrimitiveDateTime {
    let date = Date::new_0();
    js_date_to_primitive_date_time(&date)
}

pub fn date_time_str_to_primitive_date_time(date_time_str: &str) -> time::PrimitiveDateTime {
    let date = Date::new(&JsValue::from_str(date_time_str));
    js_date_to_primitive_date_time(&date)
}
