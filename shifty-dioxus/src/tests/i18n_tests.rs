#[cfg(test)]
mod i18n_tests {
    use crate::i18n::{Key, Locale, I18n};
    use time::{Date, Month, Weekday};

    #[test]
    fn test_locale_variants() {
        let locales = vec![Locale::En, Locale::De, Locale::Cs];
        
        assert_eq!(locales.len(), 3);
    }

    #[test]
    fn test_i18n_key_translation() {
        let i18n = I18n::new(Locale::En);
        
        // Test that common keys return non-empty translations
        let save_text = i18n.t(Key::Save);
        let cancel_text = i18n.t(Key::Cancel);
        let edit_text = i18n.t(Key::Edit);
        let delete_text = i18n.t(Key::Delete);
        
        assert!(!save_text.is_empty());
        assert!(!cancel_text.is_empty());
        assert!(!edit_text.is_empty());
        assert!(!delete_text.is_empty());
    }

    #[test]
    fn test_i18n_locale_switching() {
        let i18n_en = I18n::new(Locale::En);
        let i18n_de = I18n::new(Locale::De);
        
        // The actual text will differ between locales
        // We just ensure they return something
        let en_text = i18n_en.t(Key::Save);
        let de_text = i18n_de.t(Key::Save);
        
        assert!(!en_text.is_empty());
        assert!(!de_text.is_empty());
    }

    #[test]
    fn test_i18n_date_formatting() {
        let i18n = I18n::new(Locale::En);
        let date = Date::from_calendar_date(2024, Month::January, 15).unwrap();
        
        let formatted = i18n.format_date(&date);
        
        // Date should be formatted as something non-empty
        assert!(!formatted.is_empty());
        // Should contain the day
        assert!(formatted.contains("15"));
    }

    #[test]
    fn test_i18n_weekday_formatting() {
        let i18n = I18n::new(Locale::En);
        
        // Test all weekdays
        let weekdays = vec![
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
            Weekday::Sunday,
        ];
        
        for weekday in weekdays {
            let formatted = i18n.format_weekday(&weekday);
            assert!(!formatted.is_empty());
        }
    }

    #[test]
    fn test_basic_keys_have_translations() {
        let i18n_en = I18n::new(Locale::En);
        let i18n_de = I18n::new(Locale::De);
        let i18n_cs = I18n::new(Locale::Cs);
        
        // Test a sample of basic keys that should exist
        let keys = vec![
            Key::Save,
            Key::Cancel,
            Key::Edit,
            Key::Delete,
            Key::Monday,
            Key::Tuesday,
            Key::Wednesday,
            Key::Thursday,
            Key::Friday,
            Key::Saturday,
            Key::Sunday,
        ];
        
        for key in keys {
            let en_text = i18n_en.t(key);
            let de_text = i18n_de.t(key);
            let cs_text = i18n_cs.t(key);
            
            assert!(!en_text.is_empty(), "English translation missing for key: {:?}", key);
            assert!(!de_text.is_empty(), "German translation missing for key: {:?}", key);
            assert!(!cs_text.is_empty(), "Czech translation missing for key: {:?}", key);
        }
    }

    #[test]
    fn test_month_names() {
        let i18n = I18n::new(Locale::En);
        
        let months = vec![
            Month::January,
            Month::February,
            Month::March,
            Month::April,
            Month::May,
            Month::June,
            Month::July,
            Month::August,
            Month::September,
            Month::October,
            Month::November,
            Month::December,
        ];
        
        for month in months {
            let formatted = i18n.format_month(&month);
            assert!(!formatted.is_empty());
        }
    }
}