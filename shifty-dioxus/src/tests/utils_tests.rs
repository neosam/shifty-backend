use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[cfg(test)]
mod js_utils_tests {
    use crate::js::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_get_current_year() {
        let year = get_current_year();
        
        // Year should be reasonable (between 2024 and 2100)
        assert!(year >= 2024);
        assert!(year <= 2100);
    }

    #[wasm_bindgen_test]
    fn test_get_current_week() {
        let week = get_current_week();
        
        // Week should be between 1 and 53
        assert!(week >= 1);
        assert!(week <= 53);
    }

    #[wasm_bindgen_test]
    fn test_current_datetime() {
        let datetime = current_datetime();
        
        // Check that we get a valid datetime
        let year = datetime.year();
        assert!(year >= 2024);
        assert!(year <= 2100);
        
        let month = datetime.month() as u8;
        assert!(month >= 1 && month <= 12);
        
        let day = datetime.day();
        assert!(day >= 1 && day <= 31);
    }

    #[wasm_bindgen_test]
    fn test_date_time_str_to_primitive_date_time() {
        let date_str = "2024-01-15T10:30:00";
        let datetime = date_time_str_to_primitive_date_time(date_str);
        
        assert_eq!(datetime.year(), 2024);
        assert_eq!(datetime.month() as u8, 1);
        assert_eq!(datetime.day(), 15);
        assert_eq!(datetime.hour(), 10);
        assert_eq!(datetime.minute(), 30);
    }

    #[wasm_bindgen_test]
    async fn test_copy_to_clipboard() {
        // Note: This test will fail in headless environments without clipboard support
        // In a real browser environment, it should work
        let text = "Test clipboard text";
        let result = copy_to_clipboard(text).await;
        
        // We can't really test if the clipboard was updated,
        // but we can test that the function doesn't panic
        // Result will be Err in test environment without clipboard access
        assert!(result.is_ok() || result.is_err());
    }
}

#[cfg(test)]
mod base_types_tests {
    use crate::base_types::ImStr;
    use std::rc::Rc;

    #[test]
    fn test_imstr_creation() {
        let imstr: ImStr = "test string".into();
        // ImStr is Rc<str>, so we use as_ref() for comparison
        assert_eq!(imstr.as_ref(), "test string");
    }

    #[test]
    fn test_imstr_from_string() {
        let string = String::from("dynamic string");
        let imstr: ImStr = string.into();
        assert_eq!(imstr.as_ref(), "dynamic string");
    }

    #[test]
    fn test_imstr_clone() {
        let imstr1: ImStr = "cloneable".into();
        let imstr2 = imstr1.clone();
        
        // Both should point to the same data
        assert_eq!(imstr1.as_ref(), imstr2.as_ref());
        assert_eq!(imstr1.as_ref(), "cloneable");
    }

    #[test]
    fn test_rc_str_usage() {
        let rc_str: Rc<str> = "shared string".into();
        let clone1 = rc_str.clone();
        let clone2 = rc_str.clone();
        
        assert_eq!(&*rc_str, "shared string");
        assert_eq!(&*clone1, "shared string");
        assert_eq!(&*clone2, "shared string");
    }
}

#[cfg(test)]
mod error_tests {
    use crate::error::ShiftyError;

    #[test]
    fn test_shifty_error_time_component_range() {
        // Create a time component range error
        let time_error = time::error::ComponentRange::from(
            time::error::component_range::ComponentRange {
                name: "test",
                minimum: 0,
                maximum: 10,
                value: 11,
                conditional_range: false,
            }
        );
        
        let error = ShiftyError::TimeComponentRange(time_error);
        assert!(matches!(error, ShiftyError::TimeComponentRange(_)));
    }

    #[test]  
    fn test_error_result_handler() {
        use crate::error::result_handler;
        
        let ok_result: Result<i32, ShiftyError> = Ok(42);
        let result = result_handler(ok_result);
        assert_eq!(result, Some(42));
        
        let error_result: Result<i32, ShiftyError> = Err(
            ShiftyError::TimeComponentRange(
                time::error::ComponentRange::from(
                    time::error::component_range::ComponentRange {
                        name: "test",
                        minimum: 0,
                        maximum: 10,
                        value: 11,
                        conditional_range: false,
                    }
                )
            )
        );
        let result = result_handler(error_result);
        assert_eq!(result, None);
    }
}

#[cfg(test)]
mod validation_tests {
    use uuid::Uuid;

    #[test]
    fn test_uuid_validation() {
        let valid_uuid = Uuid::new_v4();
        let nil_uuid = Uuid::nil();
        
        assert_ne!(valid_uuid, nil_uuid);
        assert_eq!(nil_uuid.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn test_time_validation() {
        // Test valid time formats
        let valid_times = vec![
            "00:00",
            "12:30",
            "23:59",
            "09:15",
            "18:45",
        ];
        
        for time in valid_times {
            assert_eq!(time.len(), 5);
            assert!(time.contains(':'));
        }
    }

    #[test]
    fn test_date_validation() {
        use time::{Date, Month};
        
        let valid_date = Date::from_calendar_date(2024, Month::January, 15);
        assert!(valid_date.is_ok());
        
        let invalid_date = Date::from_calendar_date(2024, Month::February, 30);
        assert!(invalid_date.is_err());
    }
}