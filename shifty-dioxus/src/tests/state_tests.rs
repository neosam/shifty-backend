#[cfg(test)]
mod auth_info_tests {
    use crate::state::auth_info::AuthInfo;
    use std::rc::Rc;

    #[test]
    fn test_auth_info_default() {
        let auth_info = AuthInfo::default();
        
        assert_eq!(auth_info.user.as_ref(), "");
        assert_eq!(auth_info.privileges.len(), 0);
        assert!(!auth_info.authenticated);
    }

    #[test]
    fn test_auth_info_with_privileges() {
        let auth_info = AuthInfo {
            user: "admin".into(),
            privileges: Rc::new([
                "admin".into(),
                "planner".into(),
            ]),
            authenticated: true,
        };
        
        assert_eq!(auth_info.user.as_ref(), "admin");
        assert!(auth_info.authenticated);
        assert!(auth_info.has_privilege("admin"));
        assert!(auth_info.has_privilege("planner"));
        assert!(!auth_info.has_privilege("sales"));
    }
}

#[cfg(test)]
mod config_tests {
    use crate::state::config::Config;

    #[test]
    fn test_config_creation() {
        let config = Config {
            backend_url: "http://localhost:3000".into(),
        };
        
        assert_eq!(config.backend_url.as_ref(), "http://localhost:3000");
    }

    #[test]
    fn test_config_with_different_urls() {
        let configs = vec![
            Config { backend_url: "http://localhost:3000".into() },
            Config { backend_url: "https://api.example.com".into() },
            Config { backend_url: "http://192.168.1.1:8080".into() },
        ];
        
        assert_eq!(configs[0].backend_url.as_ref(), "http://localhost:3000");
        assert_eq!(configs[1].backend_url.as_ref(), "https://api.example.com");
        assert_eq!(configs[2].backend_url.as_ref(), "http://192.168.1.1:8080");
    }
}

#[cfg(test)]
mod employee_tests {
    use crate::state::employee::Employee;
    use uuid::Uuid;

    #[test]
    fn test_employee_creation() {
        let employee = Employee {
            id: Uuid::new_v4(),
            name: "Jane Doe".into(),
            active: true,
            extra_hours_september_2024: 10.5,
            expected_hours: 38.5,
            description: Some("Senior Developer".into()),
        };
        
        assert_eq!(employee.name.as_ref(), "Jane Doe");
        assert!(employee.active);
        assert_eq!(employee.extra_hours_september_2024, 10.5);
        assert_eq!(employee.expected_hours, 38.5);
        assert_eq!(employee.description.as_ref().map(|s| s.as_ref()), Some("Senior Developer"));
    }
}

#[cfg(test)]
mod week_tests {
    use crate::state::week::Week;

    #[test]
    fn test_week_creation() {
        let week = Week {
            year: 2024,
            week: 42,
        };
        
        assert_eq!(week.year, 2024);
        assert_eq!(week.week, 42);
    }

    #[test]
    fn test_week_validation() {
        let valid_weeks = vec![
            Week { year: 2024, week: 1 },
            Week { year: 2024, week: 52 },
            Week { year: 2025, week: 26 },
        ];
        
        for week in valid_weeks {
            assert!(week.week >= 1 && week.week <= 53);
            assert!(week.year >= 1900);
        }
    }
}

#[cfg(test)]
mod shiftplan_tests {
    use crate::state::shiftplan::{Weekday, Shiftplan};

    #[test]
    fn test_weekday_enum() {
        let weekdays = vec![
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
            Weekday::Sunday,
        ];
        
        assert_eq!(weekdays.len(), 7);
        assert_eq!(Weekday::Monday.num_from_monday(), 0);
        assert_eq!(Weekday::Sunday.num_from_monday(), 6);
    }

    #[test]
    fn test_weekday_from_num() {
        assert_eq!(Weekday::from_num_from_monday(0), Weekday::Monday);
        assert_eq!(Weekday::from_num_from_monday(6), Weekday::Sunday);
    }

    #[test]
    fn test_shiftplan_default() {
        let shiftplan = Shiftplan::default();
        
        assert_eq!(shiftplan.bookings.len(), 0);
        assert_eq!(shiftplan.slots.len(), 0);
        assert!(shiftplan.week_message.is_none());
    }
}

#[cfg(test)]
mod text_template_tests {
    use crate::state::text_template::TextTemplate;
    use uuid::Uuid;

    #[test]
    fn test_text_template_creation() {
        let template = TextTemplate {
            id: Uuid::new_v4(),
            name: Some("Weekly Report".into()),
            template_type: "report".into(),
            template_text: "Weekly report for {{week}} of {{year}}".into(),
            created_at: None,
            created_by: None,
        };
        
        assert!(template.name.is_some());
        assert_eq!(template.name.as_ref().map(|s| s.as_ref()), Some("Weekly Report"));
        assert_eq!(template.template_type.as_ref(), "report");
        assert!(template.template_text.contains("{{week}}"));
    }

    #[test]
    fn test_text_template_without_name() {
        let template = TextTemplate {
            id: Uuid::new_v4(),
            name: None,
            template_type: "billing-period".into(),
            template_text: "Billing period template".into(),
            created_at: None,
            created_by: None,
        };
        
        assert!(template.name.is_none());
        assert_eq!(template.template_type.as_ref(), "billing-period");
    }
}