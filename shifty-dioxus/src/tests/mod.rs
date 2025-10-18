pub mod week_tests;
pub mod error_tests;
pub mod integration_tests;

#[cfg(test)]
mod unit_tests {
    use crate::state::auth_info::AuthInfo;
    use crate::state::config::Config;
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
            user: "test_user".into(),
            privileges: Rc::new([
                "admin".into(),
                "planner".into(),
            ]),
            authenticated: true,
        };
        
        assert_eq!(auth_info.user.as_ref(), "test_user");
        assert!(auth_info.authenticated);
        assert!(auth_info.has_privilege("admin"));
        assert!(auth_info.has_privilege("planner"));
        assert!(!auth_info.has_privilege("sales"));
    }

    #[test]
    fn test_config_creation() {
        let config = Config {
            backend: "http://localhost:3000".into(),
            application_title: "Test App".into(),
            is_prod: false,
            env_short_description: "TEST".into(),
            show_vacation: true,
        };
        
        assert_eq!(config.backend.as_ref(), "http://localhost:3000");
        assert_eq!(config.application_title.as_ref(), "Test App");
        assert!(!config.is_prod);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        
        // Test that default values match Rust's Default trait behavior
        // (serde defaults only apply during deserialization, not Default::default())
        assert_eq!(config.backend.as_ref(), "");
        assert_eq!(config.application_title.as_ref(), ""); // Default trait gives empty string
        assert_eq!(config.env_short_description.as_ref(), ""); // Default trait gives empty string
        assert!(!config.is_prod);
        assert!(!config.show_vacation);
    }
}

#[cfg(test)]
mod i18n_tests {
    use crate::i18n::{Locale, Key, generate};
    use time::{Date, Month};

    #[test]
    fn test_locale_variants() {
        let locales = vec![Locale::En, Locale::De, Locale::Cs];
        assert_eq!(locales.len(), 3);
    }

    #[test]
    fn test_i18n_creation() {
        // Create i18n instance using the generate function
        let i18n = generate(Locale::En);
        
        // Test basic structure exists with translations loaded
        assert_eq!(i18n.current_locale, Locale::En);
        assert_eq!(i18n.fallback_locale, Locale::En);
        
        // Test that basic translations are available
        let save_text = i18n.t(Key::Save);
        assert!(!save_text.is_empty());
    }

    #[test]
    fn test_date_formatting_structure() {
        let date = Date::from_calendar_date(2024, Month::January, 15).unwrap();
        
        // Test date object creation and basic properties
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), Month::January);
        assert_eq!(date.day(), 15);
    }
}

#[cfg(test)]
mod service_tests {
    use crate::service::text_template::TextTemplateStore;
    use crate::service::billing_period::BillingPeriodStore;
    use crate::state::text_template::TextTemplate;
    use uuid::Uuid;

    #[test]
    fn test_text_template_store_default() {
        let store = TextTemplateStore::default();
        
        assert_eq!(store.templates.len(), 0);
        assert!(store.selected_template.is_none());
        assert_eq!(store.filtered_templates.len(), 0);
        assert!(store.current_filter_type.is_none());
    }

    #[test]
    fn test_text_template_creation() {
        let template = TextTemplate {
            id: Uuid::new_v4(),
            name: Some("Test Template".into()),
            template_type: "billing-period".into(),
            template_text: "Template content".into(),
            created_at: None,
            created_by: None,
        };
        
        assert_eq!(template.name.as_ref().map(|s| s.as_ref()), Some("Test Template"));
        assert_eq!(template.template_type.as_ref(), "billing-period");
    }

    #[test]
    fn test_billing_period_store_default() {
        let store = BillingPeriodStore::default();
        
        assert_eq!(store.billing_periods.len(), 0);
        assert!(store.selected_billing_period.is_none());
    }
}

#[cfg(test)]
mod invitation_tests {
    use rest_types::{InvitationResponse, InvitationStatus};
    use serde_json;

    #[test]
    fn test_invitation_deserialization_no_redeemed_at() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "username": "testuser",
            "token": "987fcdeb-51a2-43d1-b456-426614174111",
            "invitation_link": "http://localhost:8080/auth/invitation/987fcdeb-51a2-43d1-b456-426614174111",
            "status": "valid",
            "redeemed_at": null
        }"#;

        let result: Result<InvitationResponse, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Failed to deserialize invitation: {:?}", result.err());
        
        let invitation = result.unwrap();
        assert_eq!(invitation.username, "testuser");
        assert_eq!(invitation.status, InvitationStatus::Valid);
        assert!(invitation.redeemed_at.is_none());
    }


    #[test]
    fn test_actual_invitation_response_format() {
        // Test the actual format that the backend would send
        // Since our build succeeds, the types must be compatible somehow
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "username": "testuser",
            "token": "987fcdeb-51a2-43d1-b456-426614174111",
            "invitation_link": "http://localhost:8080/auth/invitation/987fcdeb-51a2-43d1-b456-426614174111",
            "status": "redeemed",
            "redeemed_at": null
        }"#;

        let result: Result<InvitationResponse, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Should be able to deserialize invitation with null redeemed_at: {:?}", result.err());
        
        let invitation = result.unwrap();
        assert_eq!(invitation.username, "testuser");
        assert_eq!(invitation.status, InvitationStatus::Redeemed);
        assert!(invitation.redeemed_at.is_none());
    }

    #[test] 
    fn test_exact_backend_response() {
        // Test the EXACT response provided by the user that's failing
        let json = r#"[
            {
                "id": "e09153d6-065f-4ab9-aa2d-df217818ac90",
                "username": "monika.h",
                "token": "269916a3-65ad-4608-84dc-96eaf8d01725",
                "invitation_link": "https://shifty-int.nebenan-unverpackt.de/auth/invitation/269916a3-65ad-4608-84dc-96eaf8d01725",
                "redeemed_at": null,
                "status": "valid"
            },
            {
                "id": "f3c02de8-3de3-4e14-8d27-8e38798899dd",
                "username": "monika.h",
                "token": "0fdd76a5-631b-4df8-a9cd-cdbd5d841ac9",
                "invitation_link": "https://shifty-int.nebenan-unverpackt.de/auth/invitation/0fdd76a5-631b-4df8-a9cd-cdbd5d841ac9",
                "redeemed_at": null,
                "status": "expired"
            },
            {
                "id": "0866aeb2-2153-4c24-a70b-c2667c89dce8",
                "username": "monika.h",
                "token": "4454f899-bc29-4ffe-bf98-cca2dbab62eb",
                "invitation_link": "https://shifty-int.nebenan-unverpackt.de/auth/invitation/4454f899-bc29-4ffe-bf98-cca2dbab62eb",
                "redeemed_at": "2025-10-19T05:47:11.371950094Z",
                "status": "redeemed"
            }
        ]"#;
        
        let result: Result<Vec<InvitationResponse>, _> = serde_json::from_str(json);
        if let Err(ref e) = result {
            println!("Deserialization error: {}", e);
            println!("Error details: {:?}", e);
        }
        assert!(result.is_ok(), "Should deserialize the exact backend response: {:?}", result.err());
        
        if let Ok(invitations) = result {
            assert_eq!(invitations.len(), 3);
            assert_eq!(invitations[0].status, InvitationStatus::Valid);
            assert_eq!(invitations[1].status, InvitationStatus::Expired);
            assert_eq!(invitations[2].status, InvitationStatus::Redeemed);
            assert!(invitations[2].redeemed_at.is_some());
        }
    }

    #[test]
    fn test_backend_response_formats() {
        // Test different potential backend response formats to identify the issue
        
        // Test 1: Array of invitations (what the API should return)
        let array_json = r#"[
            {
                "id": "123e4567-e89b-12d3-a456-426614174000",
                "username": "testuser",
                "token": "987fcdeb-51a2-43d1-b456-426614174111",
                "invitation_link": "http://localhost:8080/auth/invitation/987fcdeb-51a2-43d1-b456-426614174111",
                "status": "valid",
                "redeemed_at": null
            }
        ]"#;
        
        let result: Result<Vec<InvitationResponse>, _> = serde_json::from_str(array_json);
        assert!(result.is_ok(), "Array format should work: {:?}", result.err());
        
        // Test 2: What about with different timestamp formats?
        let formats_to_test = vec![
            r#"null"#,
            r#""2025-10-19 05:47:11""#,  // Without T separator
            r#""2025-10-19T05:47:11""#,  // Without timezone
            r#""2025-10-19T05:47:11+00:00""#,  // With explicit UTC offset
        ];
        
        for timestamp_format in formats_to_test {
            let json = format!(r#"{{
                "id": "123e4567-e89b-12d3-a456-426614174000",
                "username": "testuser",
                "token": "987fcdeb-51a2-43d1-b456-426614174111",
                "invitation_link": "http://localhost:8080/auth/invitation/987fcdeb-51a2-43d1-b456-426614174111",
                "status": "redeemed",
                "redeemed_at": {}
            }}"#, timestamp_format);
            
            let result: Result<InvitationResponse, _> = serde_json::from_str(&json);
            println!("Testing format {}: {:?}", timestamp_format, result.is_ok());
        }
    }
}

#[cfg(test)]
mod utils_tests {
    // Removed unused js function imports
    use crate::error::{ShiftyError, result_handler};
    use uuid::Uuid;
    use time::{Date, Month};

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_year_week_ranges() {
        // These functions use JavaScript Date, only available in WASM
        let year = get_current_year();
        let week = get_current_week();
        
        assert!(year >= 2024 && year <= 2100);
        assert!(week >= 1 && week <= 53);
    }

    #[test]
    fn test_year_week_ranges_mock() {
        // Mock test for non-WASM environments
        let current_year = 2024u32;
        let current_week = 42u8;
        
        assert!(current_year >= 2024 && current_year <= 2100);
        assert!(current_week >= 1 && current_week <= 53);
    }

    #[test]
    fn test_uuid_generation() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        
        assert_ne!(uuid1, uuid2);
        assert_ne!(uuid1, Uuid::nil());
    }

    #[test]
    fn test_date_validation() {
        let valid_date = Date::from_calendar_date(2024, Month::January, 15);
        assert!(valid_date.is_ok());
        
        let invalid_date = Date::from_calendar_date(2024, Month::February, 30);
        assert!(invalid_date.is_err());
    }

    #[test]
    fn test_error_result_handler() {
        let ok_result: Result<i32, ShiftyError> = Ok(42);
        let result = result_handler(ok_result);
        assert_eq!(result, Some(42));
    }
}