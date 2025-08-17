use js_sys::{wasm_bindgen::JsValue, Date};
use wasm_bindgen::prelude::*;

pub fn get_current_year() -> u32 {
    current_datetime().to_iso_week_date().0 as u32
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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = navigator)]
    type Clipboard;

    #[wasm_bindgen(js_namespace = navigator, js_name = clipboard)]
    fn get_clipboard() -> Clipboard;

    #[wasm_bindgen(method, js_name = writeText)]
    fn write_text(this: &Clipboard, text: &str) -> js_sys::Promise;
}

pub async fn copy_to_clipboard(text: &str) -> Result<(), JsValue> {
    let clipboard = get_clipboard();
    let promise = clipboard.write_text(text);
    wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(())
}
