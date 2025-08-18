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

    #[wasm_bindgen(js_namespace = navigator, js_name = clipboard, getter, catch)]
    fn get_clipboard() -> Result<Clipboard, JsValue>;

    #[wasm_bindgen(method, js_name = writeText, catch)]
    fn write_text(this: &Clipboard, text: &str) -> Result<js_sys::Promise, JsValue>;
}

pub async fn copy_to_clipboard(text: &str) -> Result<(), JsValue> {
    // Try modern clipboard API first
    match get_clipboard() {
        Ok(clipboard) => {
            match clipboard.write_text(text) {
                Ok(promise) => {
                    wasm_bindgen_futures::JsFuture::from(promise).await?;
                    Ok(())
                }
                Err(_) => {
                    // Fallback to execCommand
                    copy_with_exec_command(text)
                }
            }
        }
        Err(_) => {
            // Clipboard API not available, use fallback
            copy_with_exec_command(text)
        }
    }
}

fn copy_with_exec_command(text: &str) -> Result<(), JsValue> {
    use wasm_bindgen::JsCast;
    use js_sys::Reflect;
    
    let window = web_sys::window().ok_or(JsValue::from_str("No window object"))?;
    let document = window.document().ok_or(JsValue::from_str("No document object"))?;
    
    // Create a temporary textarea element
    let textarea = document
        .create_element("textarea")
        .map_err(|e| JsValue::from(e))?
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .map_err(|_| JsValue::from_str("Failed to create textarea"))?;
    
    // Set the text and styling
    textarea.set_value(text);
    textarea.style().set_property("position", "fixed").ok();
    textarea.style().set_property("left", "-9999px").ok();
    textarea.style().set_property("top", "-9999px").ok();
    
    // Append to body
    document
        .body()
        .ok_or(JsValue::from_str("No body element"))?
        .append_child(&textarea)
        .map_err(|e| JsValue::from(e))?;
    
    // Select and copy
    textarea.select();
    
    // Call execCommand using Reflect
    let exec_command = Reflect::get(&document, &JsValue::from_str("execCommand"))
        .map_err(|_| JsValue::from_str("execCommand not available"))?;
    
    let exec_command_fn = exec_command
        .dyn_ref::<js_sys::Function>()
        .ok_or(JsValue::from_str("execCommand is not a function"))?;
    
    let success = exec_command_fn
        .call1(&document, &JsValue::from_str("copy"))
        .map_err(|_| JsValue::from_str("execCommand call failed"))?
        .as_bool()
        .unwrap_or(false);
    
    // Remove the temporary element
    textarea.remove();
    
    if success {
        Ok(())
    } else {
        Err(JsValue::from_str("execCommand('copy') failed"))
    }
}
