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
    use crate::state::text_template::{TextTemplate, TemplateEngine};
    use uuid::Uuid;

    #[test]
    fn test_text_template_creation() {
        let template = TextTemplate {
            id: Uuid::new_v4(),
            name: Some("Weekly Report".into()),
            template_type: "report".into(),
            template_text: "Weekly report for {{week}} of {{year}}".into(),
            template_engine: TemplateEngine::Tera,
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
            template_engine: TemplateEngine::Tera,
            created_at: None,
            created_by: None,
        };
        
        assert!(template.name.is_none());
        assert_eq!(template.template_type.as_ref(), "billing-period");
    }

    #[test]
    fn test_template_engine_to_serialize_tera() {
        use rest_types::TemplateEngineTO;
        let json = serde_json::to_string(&TemplateEngineTO::Tera).unwrap();
        assert_eq!(json, "\"tera\"");
    }

    #[test]
    fn test_template_engine_to_serialize_minijinja() {
        use rest_types::TemplateEngineTO;
        let json = serde_json::to_string(&TemplateEngineTO::MiniJinja).unwrap();
        assert_eq!(json, "\"minijinja\"");
    }

    #[test]
    fn test_template_engine_to_deserialize_tera() {
        use rest_types::TemplateEngineTO;
        let engine: TemplateEngineTO = serde_json::from_str("\"tera\"").unwrap();
        assert_eq!(engine, TemplateEngineTO::Tera);
    }

    #[test]
    fn test_template_engine_to_deserialize_minijinja() {
        use rest_types::TemplateEngineTO;
        let engine: TemplateEngineTO = serde_json::from_str("\"minijinja\"").unwrap();
        assert_eq!(engine, TemplateEngineTO::MiniJinja);
    }

    #[test]
    fn test_template_engine_to_roundtrip() {
        use rest_types::TemplateEngineTO;
        for engine in [TemplateEngineTO::Tera, TemplateEngineTO::MiniJinja] {
            let json = serde_json::to_string(&engine).unwrap();
            let deserialized: TemplateEngineTO = serde_json::from_str(&json).unwrap();
            assert_eq!(engine, deserialized);
        }
    }

    #[test]
    fn test_text_template_to_with_engine_roundtrip() {
        use rest_types::{TextTemplateTO, TemplateEngineTO};
        let json = serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "template_type": "billing-period",
            "template_text": "hello",
            "template_engine": "minijinja"
        });
        let template: TextTemplateTO = serde_json::from_value(json).unwrap();
        assert_eq!(template.template_engine, TemplateEngineTO::MiniJinja);

        let serialized = serde_json::to_value(&template).unwrap();
        assert_eq!(serialized["template_engine"], "minijinja");
    }

    #[test]
    fn test_create_text_template_request_to_with_engine() {
        use rest_types::{CreateTextTemplateRequestTO, TemplateEngineTO};
        let req = CreateTextTemplateRequestTO {
            name: None,
            template_type: "billing-period".into(),
            template_text: "test".into(),
            template_engine: TemplateEngineTO::Tera,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["template_engine"], "tera");
    }

    #[test]
    fn test_update_text_template_request_to_with_engine() {
        use rest_types::{UpdateTextTemplateRequestTO, TemplateEngineTO};
        let req = UpdateTextTemplateRequestTO {
            name: Some("name".into()),
            template_type: "billing-period".into(),
            template_text: "test".into(),
            template_engine: TemplateEngineTO::MiniJinja,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["template_engine"], "minijinja");
    }
}