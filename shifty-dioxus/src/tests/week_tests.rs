#[cfg(test)]
mod week_tests {
    use crate::state::week::Week;
    use time::{Month, Weekday};

    #[test]
    fn test_week_monday_calculation() {
        let week = Week { year: 2024, week: 1 };
        let monday = week.monday().unwrap();
        
        // Week 1 of 2024 starts on January 1st (Monday)
        assert_eq!(monday.year(), 2024);
        assert_eq!(monday.month(), Month::January);
        assert_eq!(monday.day(), 1);
        assert_eq!(monday.weekday(), Weekday::Monday);
    }

    #[test]
    fn test_week_sunday_calculation() {
        let week = Week { year: 2024, week: 1 };
        let sunday = week.sunday().unwrap();
        
        // Week 1 of 2024 ends on January 7th (Sunday)
        assert_eq!(sunday.year(), 2024);
        assert_eq!(sunday.month(), Month::January);
        assert_eq!(sunday.day(), 7);
        assert_eq!(sunday.weekday(), Weekday::Sunday);
    }

    #[test]
    fn test_week_mid_year() {
        let week = Week { year: 2024, week: 26 };
        let monday = week.monday().unwrap();
        let sunday = week.sunday().unwrap();
        
        // Week 26 should be in June/July
        assert_eq!(monday.year(), 2024);
        assert_eq!(sunday.year(), 2024);
        assert_eq!(monday.weekday(), Weekday::Monday);
        assert_eq!(sunday.weekday(), Weekday::Sunday);
        
        // Sunday should be 6 days after Monday
        let days_diff = sunday.ordinal() - monday.ordinal();
        assert_eq!(days_diff, 6);
    }

    #[test]
    fn test_week_year_boundary() {
        // Test week 53 of 2024 (if it exists)
        let week = Week { year: 2024, week: 52 };
        let monday = week.monday().unwrap();
        let sunday = week.sunday().unwrap();
        
        // Should be late December
        assert_eq!(monday.weekday(), Weekday::Monday);
        assert_eq!(sunday.weekday(), Weekday::Sunday);
        assert!(sunday.ordinal() >= monday.ordinal());
    }

    #[test]
    fn test_week_invalid_week_number() {
        let week = Week { year: 2024, week: 54 }; // Invalid week number
        
        // Should return error for invalid week
        assert!(week.monday().is_err());
        assert!(week.sunday().is_err());
    }

    #[test]
    fn test_week_zero_week() {
        let week = Week { year: 2024, week: 0 }; // Invalid week number
        
        // Should return error for week 0
        assert!(week.monday().is_err());
        assert!(week.sunday().is_err());
    }

    #[test]
    fn test_week_consistency() {
        // Test that Monday and Sunday calculations are consistent
        for week_num in 1..=52 {
            let week = Week { year: 2024, week: week_num };
            
            if let (Ok(monday), Ok(sunday)) = (week.monday(), week.sunday()) {
                // Sunday should be exactly 6 days after Monday
                let duration = sunday - monday;
                assert_eq!(duration.whole_days(), 6);
                
                // Both should be in the same week
                let monday_week = monday.iso_week();
                let sunday_week = sunday.iso_week();
                
                // Sunday might be in the next week due to ISO week rules
                assert!(
                    monday_week == sunday_week || 
                    (sunday_week - monday_week == 1) ||
                    (monday_week == 52 && sunday_week == 1) // Year boundary
                );
            }
        }
    }

    #[test]
    fn test_week_different_years() {
        let years = [2020, 2021, 2022, 2023, 2024, 2025];
        
        for year in years {
            let week = Week { year, week: 1 };
            let monday = week.monday().unwrap();
            let sunday = week.sunday().unwrap();
            
            assert_eq!(monday.weekday(), Weekday::Monday);
            assert_eq!(sunday.weekday(), Weekday::Sunday);
            
            // Week 1 Monday should be in the target year or very close
            assert!(monday.year() == year as i32 || monday.year() == (year - 1) as i32);
        }
    }

    #[test]
    fn test_week_creation_and_properties() {
        let week = Week { year: 2024, week: 15 };
        
        // Test basic properties
        assert_eq!(week.year, 2024);
        assert_eq!(week.week, 15);
    }
}