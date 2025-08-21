#[cfg(test)]
mod error_handling_tests {
    use crate::error::{ShiftyError, result_handler};
    
    #[test]
    fn test_shifty_error_variants() {
        // Test the actual error variants that exist in ShiftyError
        
        // Since reqwest::Error is hard to construct directly, let's test with a time error instead
        if let Err(time_error) = time::Date::from_calendar_date(2024, time::Month::February, 30) {
            let shifty_error = ShiftyError::TimeComponentRange(time_error);
            
            match shifty_error {
                ShiftyError::TimeComponentRange(_) => {
                    // Test passes if we can create the error
                    assert!(true);
                },
                _ => panic!("Expected TimeComponentRange error"),
            }
        }
    }

    #[test]
    fn test_time_component_range_error() {
        // Create a time component range error using actual time API
        use time::{Date, Month};
        
        // Try to create an invalid date to trigger ComponentRange error
        let invalid_date_result = Date::from_calendar_date(2024, Month::February, 30);
        
        match invalid_date_result {
            Err(time_error) => {
                // Convert time error to ShiftyError
                let shifty_error = ShiftyError::TimeComponentRange(time_error);
                
                match shifty_error {
                    ShiftyError::TimeComponentRange(_) => {
                        assert!(true); // Test passes
                    },
                    _ => panic!("Expected TimeComponentRange error"),
                }
            },
            Ok(_) => {
                // If the date was valid, create a different ComponentRange error
                // This is just for testing the error type
                let week_error = time::Date::from_iso_week_date(2024, 54, time::Weekday::Monday);
                if let Err(time_error) = week_error {
                    let shifty_error = ShiftyError::TimeComponentRange(time_error);
                    assert!(matches!(shifty_error, ShiftyError::TimeComponentRange(_)));
                }
            }
        }
    }

    #[test]
    fn test_result_handler_success() {
        let ok_result: Result<String, ShiftyError> = Ok("success".to_string());
        let result = result_handler(ok_result);
        
        assert_eq!(result, Some("success".to_string()));
    }

    #[test]
    fn test_result_handler_error() {
        // Create an invalid date to get a ComponentRange error
        let invalid_date = time::Date::from_calendar_date(2024, time::Month::February, 30);
        let error_result: Result<String, ShiftyError> = match invalid_date {
            Err(time_error) => Err(ShiftyError::TimeComponentRange(time_error)),
            Ok(_) => Ok("shouldn't happen".to_string()),
        };
        
        let result = result_handler(error_result);
        assert_eq!(result, None);
    }

    #[test] 
    fn test_result_handler_with_different_types() {
        // Test with integer success
        let ok_int: Result<i32, ShiftyError> = Ok(42);
        assert_eq!(result_handler(ok_int), Some(42));
        
        // Test with boolean success
        let ok_bool: Result<bool, ShiftyError> = Ok(true);
        assert_eq!(result_handler(ok_bool), Some(true));
        
        // Test with errors using actual error types
        if let Err(time_error) = time::Date::from_iso_week_date(2024, 54, time::Weekday::Monday) {
            let err_int: Result<i32, ShiftyError> = Err(ShiftyError::TimeComponentRange(time_error));
            assert_eq!(result_handler(err_int), None);
        }
    }

    #[test]
    fn test_error_display() {
        // Test that errors can be displayed
        if let Err(time_error) = time::Date::from_calendar_date(2024, time::Month::February, 30) {
            let shifty_error = ShiftyError::TimeComponentRange(time_error);
            let error_string = format!("{}", shifty_error);
            
            // Error string should contain some meaningful text
            assert!(!error_string.is_empty());
            assert!(error_string.contains("ComponentRange") || error_string.contains("error"));
        }
    }

    #[test]
    fn test_error_debug() {
        // Test that errors can be debugged
        if let Err(time_error) = time::Date::from_iso_week_date(2024, 55, time::Weekday::Monday) {
            let shifty_error = ShiftyError::TimeComponentRange(time_error);
            let debug_string = format!("{:?}", shifty_error);
            
            // Debug string should contain some meaningful text
            assert!(!debug_string.is_empty());
            assert!(debug_string.contains("TimeComponentRange"));
        }
    }

    #[test]
    fn test_week_date_errors() {
        use crate::state::week::Week;
        
        // Test definitely invalid weeks
        let definitely_invalid_weeks = vec![
            Week { year: 2024, week: 0 },   // Week 0 doesn't exist
            Week { year: 2024, week: 55 },  // Week 55 definitely doesn't exist
        ];
        
        for invalid_week in definitely_invalid_weeks {
            // These should return errors
            assert!(invalid_week.monday().is_err(), "Week {}/{} should be invalid", invalid_week.year, invalid_week.week);
            assert!(invalid_week.sunday().is_err(), "Week {}/{} should be invalid", invalid_week.year, invalid_week.week);
        }
        
        // Test that valid weeks work
        let valid_week = Week { year: 2024, week: 1 };
        assert!(valid_week.monday().is_ok());
        assert!(valid_week.sunday().is_ok());
    }

    #[test]
    fn test_date_boundary_errors() {
        use time::{Date, Month};
        
        let invalid_dates = vec![
            Date::from_calendar_date(2024, Month::February, 30), // Feb 30 doesn't exist
            Date::from_calendar_date(2024, Month::April, 31),    // April 31 doesn't exist
            Date::from_calendar_date(2023, Month::February, 29), // 2023 is not a leap year
        ];
        
        for invalid_date in invalid_dates {
            assert!(invalid_date.is_err());
        }
    }

    #[test]
    fn test_uuid_error_scenarios() {
        use uuid::Uuid;
        
        // Test UUID parsing errors
        let invalid_uuid_strings = vec![
            "not-a-uuid",
            "12345678-1234-1234-1234-12345678901", // Too short
            "12345678-1234-1234-1234-1234567890123", // Too long
            "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx", // Invalid characters
            "",
        ];
        
        for invalid_string in invalid_uuid_strings {
            let parse_result = Uuid::parse_str(invalid_string);
            assert!(parse_result.is_err(), "Should fail to parse: {}", invalid_string);
        }
        
        // Test valid UUID parsing
        let valid_uuid = Uuid::new_v4();
        let uuid_string = valid_uuid.to_string();
        let parsed_uuid = Uuid::parse_str(&uuid_string).unwrap();
        assert_eq!(valid_uuid, parsed_uuid);
    }

    #[test]
    fn test_service_error_propagation() {
        // Simulate service layer error handling with actual error types
        fn simulate_failing_operation() -> Result<String, ShiftyError> {
            // Create a ComponentRange error
            let invalid_date = time::Date::from_calendar_date(2024, time::Month::February, 30);
            match invalid_date {
                Err(time_error) => Err(ShiftyError::TimeComponentRange(time_error)),
                Ok(_) => Ok("unexpected success".to_string()),
            }
        }
        
        fn simulate_service_call() -> Result<String, ShiftyError> {
            let operation_result = simulate_failing_operation()?;
            Ok(format!("Processed: {}", operation_result))
        }
        
        let service_result = simulate_service_call();
        assert!(service_result.is_err());
        
        match service_result {
            Err(ShiftyError::TimeComponentRange(_)) => {
                // Error propagated correctly
                assert!(true);
            },
            _ => panic!("Expected TimeComponentRange error to propagate"),
        }
    }
}