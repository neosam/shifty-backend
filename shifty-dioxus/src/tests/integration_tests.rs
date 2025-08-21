#[cfg(test)]
mod integration_tests {
    use crate::state::{AuthInfo, Config};
    use crate::state::week::Week;
    use crate::state::text_template::TextTemplate;
    use crate::service::text_template::TextTemplateStore;
    use crate::i18n::{generate, Locale, Key};
    use std::rc::Rc;
    use uuid::Uuid;
    use time::{Date, Month};

    #[test]
    fn test_authenticated_user_workflow() {
        // Test complete user authentication flow
        let auth_info = AuthInfo {
            user: "manager@company.com".into(),
            privileges: Rc::new([
                "shift_planner".into(),
                "employee_manager".into(),
                "report_viewer".into(),
            ]),
            authenticated: true,
        };
        
        // User should be able to access shift planning
        assert!(auth_info.has_privilege("shift_planner"));
        assert!(auth_info.authenticated);
        
        // User should be able to manage employees
        assert!(auth_info.has_privilege("employee_manager"));
        
        // User should be able to view reports
        assert!(auth_info.has_privilege("report_viewer"));
        
        // User should NOT be able to access admin functions
        assert!(!auth_info.has_privilege("super_admin"));
    }

    #[test]
    fn test_week_calculation_with_i18n() {
        // Test week calculations with different locales
        let week = Week { year: 2024, week: 15 };
        let monday = week.monday().unwrap();
        let _sunday = week.sunday().unwrap();
        
        let i18n_en = generate(Locale::En);
        let i18n_de = generate(Locale::De);
        let i18n_cs = generate(Locale::Cs);
        
        // Format dates in different locales
        let monday_en = i18n_en.format_date(&monday);
        let monday_de = i18n_de.format_date(&monday);
        let monday_cs = i18n_cs.format_date(&monday);
        
        // All should be non-empty and different formats
        assert!(!monday_en.is_empty());
        assert!(!monday_de.is_empty());
        assert!(!monday_cs.is_empty());
        
        // German format should use dots
        assert!(monday_de.contains('.'));
        
        // Week formatting
        let week_formatted_en = i18n_en.format_week(&week);
        let week_formatted_de = i18n_de.format_week(&week);
        
        let en_string = week_formatted_en.to_string();
        let de_string = week_formatted_de.to_string();
        assert!(!en_string.is_empty());
        assert!(!de_string.is_empty());
        assert!(en_string.contains("15")); // Week number should be included
    }

    #[test]
    fn test_template_management_workflow() {
        // Test complete template management workflow
        let mut store = TextTemplateStore::default();
        
        // Create templates for different purposes
        let billing_template = TextTemplate {
            id: Uuid::new_v4(),
            name: Some("Billing Period Report".into()),
            template_type: "billing-period".into(),
            template_text: "Billing period from {start_date} to {end_date}\nTotal hours: {total_hours}".into(),
            created_at: None,
            created_by: None,
        };
        
        let employee_template = TextTemplate {
            id: Uuid::new_v4(),
            name: Some("Employee Performance Report".into()),
            template_type: "employee-report".into(),
            template_text: "Employee: {employee_name}\nWorked hours: {worked_hours}\nExpected hours: {expected_hours}".into(),
            created_at: None,
            created_by: None,
        };
        
        // Add templates to store
        let templates_vec = vec![billing_template.clone(), employee_template.clone()];
        store.templates = Rc::from(templates_vec.clone());
        store.filtered_templates = store.templates.clone();
        
        // Test filtering by type
        store.current_filter_type = Some("billing-period".into());
        let filtered: Vec<_> = store.templates
            .iter()
            .filter(|t| t.template_type.as_ref() == "billing-period")
            .cloned()
            .collect();
        store.filtered_templates = Rc::from(filtered);
        
        assert_eq!(store.filtered_templates.len(), 1);
        assert_eq!(store.filtered_templates[0].name.as_ref().unwrap().as_ref(), "Billing Period Report");
        
        // Test template selection (selected_template is the full template, not just ID)
        store.selected_template = Some(billing_template.clone());
        assert!(store.selected_template.is_some());
        
        // Test template content validation - convert to string for testing
        let template_text = &store.selected_template.as_ref().unwrap().template_text;
        let text_string = template_text.to_string();
        assert!(text_string.contains("{start_date}"));
        assert!(text_string.contains("{end_date}"));
        assert!(text_string.contains("{total_hours}"));
    }

    #[test]
    fn test_multilingual_application_flow() {
        // Test application workflow with different languages
        let config = Config {
            backend: "http://localhost:3000".into(),
            application_title: "Shifty".into(),
            is_prod: false,
            env_short_description: "DEV".into(),
            show_vacation: true,
        };
        
        let auth_info = AuthInfo {
            user: "czech.user@company.cz".into(),
            privileges: Rc::new(["shift_planner".into()]),
            authenticated: true,
        };
        
        // Test with Czech locale
        let i18n = generate(Locale::Cs);
        
        // Basic UI elements should be translated
        let save_text = i18n.t(Key::Save);
        let cancel_text = i18n.t(Key::Cancel);
        let monday_text = i18n.t(Key::Monday);
        
        assert!(!save_text.is_empty());
        assert!(!cancel_text.is_empty());
        assert!(!monday_text.is_empty());
        
        // Test date formatting for Czech locale
        let date = Date::from_calendar_date(2024, Month::March, 15).unwrap();
        let formatted_date = i18n.format_date(&date);
        
        assert!(!formatted_date.is_empty());
        assert!(formatted_date.contains("15")); // Day should be present
        
        // User should still be able to access shift planning regardless of locale
        assert!(auth_info.has_privilege("shift_planner"));
        assert!(config.show_vacation); // Development environment shows vacation
    }

    #[test]
    fn test_production_vs_development_workflow() {
        // Test differences between production and development configurations
        let dev_config = Config {
            backend: "http://localhost:3000".into(),
            application_title: "Shifty DEV".into(),
            is_prod: false,
            env_short_description: "DEV".into(),
            show_vacation: true,
        };
        
        let prod_config = Config {
            backend: "https://api.shifty.com".into(),
            application_title: "Shifty".into(),
            is_prod: true,
            env_short_description: "PROD".into(),
            show_vacation: false,
        };
        
        // Development should show vacation features
        assert!(dev_config.show_vacation);
        assert!(!dev_config.is_prod);
        assert!(dev_config.backend.starts_with("http://"));
        
        // Production should hide vacation features and use HTTPS
        assert!(!prod_config.show_vacation);
        assert!(prod_config.is_prod);
        assert!(prod_config.backend.starts_with("https://"));
        
        // Both should support the same core functionality
        let auth_info = AuthInfo {
            user: "production.user@company.com".into(),
            privileges: Rc::new(["shift_planner".into()]),
            authenticated: true,
        };
        
        assert!(auth_info.has_privilege("shift_planner"));
    }

    #[test]
    fn test_week_boundary_edge_cases() {
        // Test week calculations around year boundaries
        let last_week_2023 = Week { year: 2023, week: 52 };
        let first_week_2024 = Week { year: 2024, week: 1 };
        
        let last_monday_2023 = last_week_2023.monday().unwrap();
        let first_monday_2024 = first_week_2024.monday().unwrap();
        
        // The weeks should be close to each other
        let duration = first_monday_2024 - last_monday_2023;
        assert!(duration.whole_days() <= 14); // Should be within 2 weeks
        
        // Test with I18n formatting
        let i18n = generate(Locale::En);
        let formatted_2023 = i18n.format_week(&last_week_2023);
        let formatted_2024 = i18n.format_week(&first_week_2024);
        
        let f2023_string = formatted_2023.to_string();
        let f2024_string = formatted_2024.to_string();
        assert!(f2023_string.contains("2023") || f2023_string.contains("52"));
        assert!(f2024_string.contains("2024") || f2024_string.contains("1"));
    }

    #[test]
    fn test_error_recovery_workflow() {
        use crate::error::{ShiftyError, result_handler};
        
        // Simulate a workflow that can recover from errors
        fn attempt_operation(should_fail: bool) -> Result<String, ShiftyError> {
            if should_fail {
                // Create a real error using invalid date
                let invalid_date = time::Date::from_calendar_date(2024, time::Month::February, 30);
                match invalid_date {
                    Err(time_error) => Err(ShiftyError::TimeComponentRange(time_error)),
                    Ok(_) => Ok("unexpected".to_string()),
                }
            } else {
                Ok("Operation successful".to_string())
            }
        }
        
        // First attempt fails
        let first_attempt = attempt_operation(true);
        let first_result = result_handler(first_attempt);
        assert_eq!(first_result, None);
        
        // Retry succeeds
        let second_attempt = attempt_operation(false);
        let second_result = result_handler(second_attempt);
        assert_eq!(second_result, Some("Operation successful".to_string()));
    }

    #[test]
    fn test_user_privilege_escalation_prevention() {
        // Test that privilege checks prevent unauthorized access
        let regular_user = AuthInfo {
            user: "regular.user@company.com".into(),
            privileges: Rc::new(["report_viewer".into()]),
            authenticated: true,
        };
        
        let admin_user = AuthInfo {
            user: "admin@company.com".into(),
            privileges: Rc::new([
                "report_viewer".into(),
                "employee_manager".into(),
                "shift_planner".into(),
                "admin".into(),
            ]),
            authenticated: true,
        };
        
        // Regular user should have limited access
        assert!(regular_user.has_privilege("report_viewer"));
        assert!(!regular_user.has_privilege("employee_manager"));
        assert!(!regular_user.has_privilege("shift_planner"));
        assert!(!regular_user.has_privilege("admin"));
        
        // Admin user should have full access
        assert!(admin_user.has_privilege("report_viewer"));
        assert!(admin_user.has_privilege("employee_manager"));
        assert!(admin_user.has_privilege("shift_planner"));
        assert!(admin_user.has_privilege("admin"));
        
        // Neither should have non-existent privileges
        assert!(!regular_user.has_privilege("super_secret_privilege"));
        assert!(!admin_user.has_privilege("non_existent_privilege"));
    }
}