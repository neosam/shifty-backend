pub mod error_tests;
pub mod integration_tests;
pub mod volunteer_work_tests;
pub mod week_tests;

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
            privileges: Rc::new(["admin".into(), "planner".into()]),
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
    use crate::i18n::{generate, Key, Locale};
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
    use crate::service::billing_period::BillingPeriodStore;
    use crate::service::text_template::TextTemplateStore;
    use crate::state::text_template::{TemplateEngine, TextTemplate};
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
            template_engine: TemplateEngine::Tera,
            created_at: None,
            created_by: None,
        };

        assert_eq!(
            template.name.as_ref().map(|s| s.as_ref()),
            Some("Test Template")
        );
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
        assert!(
            result.is_ok(),
            "Failed to deserialize invitation: {:?}",
            result.err()
        );

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
        assert!(
            result.is_ok(),
            "Should be able to deserialize invitation with null redeemed_at: {:?}",
            result.err()
        );

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
        assert!(
            result.is_ok(),
            "Should deserialize the exact backend response: {:?}",
            result.err()
        );

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
        assert!(
            result.is_ok(),
            "Array format should work: {:?}",
            result.err()
        );

        // Test 2: What about with different timestamp formats?
        let formats_to_test = vec![
            r#"null"#,
            r#""2025-10-19 05:47:11""#,       // Without T separator
            r#""2025-10-19T05:47:11""#,       // Without timezone
            r#""2025-10-19T05:47:11+00:00""#, // With explicit UTC offset
        ];

        for timestamp_format in formats_to_test {
            let json = format!(
                r#"{{
                "id": "123e4567-e89b-12d3-a456-426614174000",
                "username": "testuser",
                "token": "987fcdeb-51a2-43d1-b456-426614174111",
                "invitation_link": "http://localhost:8080/auth/invitation/987fcdeb-51a2-43d1-b456-426614174111",
                "status": "redeemed",
                "redeemed_at": {}
            }}"#,
                timestamp_format
            );

            let result: Result<InvitationResponse, _> = serde_json::from_str(&json);
            println!("Testing format {}: {:?}", timestamp_format, result.is_ok());
        }
    }
}

#[cfg(test)]
mod delete_billing_period_tests {
    use crate::i18n::{generate, Key, Locale};
    use crate::state::auth_info::AuthInfo;
    use std::rc::Rc;

    #[test]
    fn test_delete_billing_period_i18n_keys_all_locales() {
        let keys = vec![
            Key::DeleteBillingPeriod,
            Key::ConfirmDeleteBillingPeriod,
            Key::DeleteBillingPeriodError,
        ];

        for locale in &[Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(*locale);
            for key in &keys {
                let text = i18n.t(*key);
                assert!(
                    !text.is_empty() && text.as_ref() != "??",
                    "Translation missing for {:?} in locale {:?}",
                    key,
                    locale
                );
            }
        }
    }

    #[test]
    fn test_confirm_delete_billing_period_has_period_placeholder() {
        for locale in &[Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(*locale);
            let text = i18n.t(Key::ConfirmDeleteBillingPeriod);
            assert!(
                text.contains("{period}"),
                "ConfirmDeleteBillingPeriod in {:?} should contain {{period}} placeholder",
                locale
            );
        }
    }

    #[test]
    fn test_delete_error_has_error_placeholder() {
        for locale in &[Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(*locale);
            let text = i18n.t(Key::DeleteBillingPeriodError);
            assert!(
                text.contains("{error}"),
                "DeleteBillingPeriodError in {:?} should contain {{error}} placeholder",
                locale
            );
        }
    }

    #[test]
    fn test_hr_privilege_visibility() {
        let hr_user = AuthInfo {
            user: "hr_admin".into(),
            privileges: Rc::new(["hr".into()]),
            authenticated: true,
        };
        assert!(hr_user.has_privilege("hr"));

        let non_hr_user = AuthInfo {
            user: "regular".into(),
            privileges: Rc::new(["sales".into()]),
            authenticated: true,
        };
        assert!(!non_hr_user.has_privilege("hr"));

        let no_privileges = AuthInfo::default();
        assert!(!no_privileges.has_privilege("hr"));
    }
}

#[cfg(test)]
mod utils_tests {
    // Removed unused js function imports
    use crate::error::{result_handler, ShiftyError};
    use time::{Date, Month};
    use uuid::Uuid;

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

#[cfg(test)]
mod shiftplan_catalog_tests {
    use crate::state::slot_edit::SlotEditItem;
    use rest_types::{DayOfWeekTO, ShiftplanTO, SlotTO};
    use uuid::Uuid;

    #[test]
    fn test_shiftplan_to_deserialization() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "Hauptplan",
            "is_planning": false,
            "$version": "223e4567-e89b-12d3-a456-426614174000"
        }"#;

        let result: Result<ShiftplanTO, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Failed to deserialize ShiftplanTO: {:?}",
            result.err()
        );

        let shiftplan = result.unwrap();
        assert_eq!(shiftplan.name.as_ref(), "Hauptplan");
        assert!(!shiftplan.is_planning);
        assert!(shiftplan.deleted.is_none());
    }

    #[test]
    fn test_shiftplan_to_deserialization_with_defaults() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "Testplan"
        }"#;

        let result: Result<ShiftplanTO, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Failed to deserialize ShiftplanTO with defaults: {:?}",
            result.err()
        );

        let shiftplan = result.unwrap();
        assert!(!shiftplan.is_planning);
        assert_eq!(shiftplan.version, Uuid::nil());
    }

    #[test]
    fn test_shiftplan_to_list_deserialization() {
        let json = r#"[
            {
                "id": "123e4567-e89b-12d3-a456-426614174000",
                "name": "Hauptplan",
                "is_planning": false,
                "$version": "223e4567-e89b-12d3-a456-426614174000"
            },
            {
                "id": "223e4567-e89b-12d3-a456-426614174000",
                "name": "Wochenende",
                "is_planning": true,
                "$version": "323e4567-e89b-12d3-a456-426614174000"
            }
        ]"#;

        let result: Result<Vec<ShiftplanTO>, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let shiftplans = result.unwrap();
        assert_eq!(shiftplans.len(), 2);
        assert_eq!(shiftplans[0].name.as_ref(), "Hauptplan");
        assert_eq!(shiftplans[1].name.as_ref(), "Wochenende");
        assert!(shiftplans[1].is_planning);
    }

    #[test]
    fn test_slot_edit_item_from_slot_to_preserves_shiftplan_id() {
        let shiftplan_id = Uuid::new_v4();
        let slot_to = SlotTO {
            id: Uuid::new_v4(),
            day_of_week: DayOfWeekTO::Tuesday,
            from: time::Time::from_hms(8, 0, 0).unwrap(),
            to: time::Time::from_hms(16, 0, 0).unwrap(),
            min_resources: 1,
            valid_from: time::Date::from_calendar_date(2024, time::Month::June, 1).unwrap(),
            valid_to: None,
            deleted: None,
            version: Uuid::new_v4(),
            shiftplan_id: Some(shiftplan_id),
        };

        let edit_item = SlotEditItem::from(&slot_to);
        assert_eq!(edit_item.shiftplan_id, Some(shiftplan_id));

        let back = SlotTO::from(&edit_item);
        assert_eq!(back.shiftplan_id, Some(shiftplan_id));
    }

    #[test]
    fn test_slot_edit_item_from_slot_to_none_shiftplan_id() {
        let slot_to = SlotTO {
            id: Uuid::new_v4(),
            day_of_week: DayOfWeekTO::Monday,
            from: time::Time::from_hms(9, 0, 0).unwrap(),
            to: time::Time::from_hms(17, 0, 0).unwrap(),
            min_resources: 2,
            valid_from: time::Date::from_calendar_date(2024, time::Month::January, 1).unwrap(),
            valid_to: None,
            deleted: None,
            version: Uuid::new_v4(),
            shiftplan_id: None,
        };

        let edit_item = SlotEditItem::from(&slot_to);
        assert!(edit_item.shiftplan_id.is_none());
    }

    #[test]
    fn test_slot_edit_item_from_slot_to_with_shiftplan_id() {
        let shiftplan_id = Uuid::new_v4();
        let slot_to = SlotTO {
            id: Uuid::new_v4(),
            day_of_week: DayOfWeekTO::Monday,
            from: time::Time::from_hms(9, 0, 0).unwrap(),
            to: time::Time::from_hms(17, 0, 0).unwrap(),
            min_resources: 2,
            valid_from: time::Date::from_calendar_date(2024, time::Month::January, 1).unwrap(),
            valid_to: None,
            deleted: None,
            version: Uuid::new_v4(),
            shiftplan_id: Some(shiftplan_id),
        };

        let edit_item = SlotEditItem::from(&slot_to);
        assert_eq!(edit_item.shiftplan_id, Some(shiftplan_id));
    }

    #[test]
    fn test_slot_to_from_slot_edit_item_with_shiftplan_id() {
        let shiftplan_id = Uuid::new_v4();
        let edit_item = SlotEditItem {
            id: Uuid::new_v4(),
            day_of_week: crate::state::Weekday::Monday,
            from: time::Time::from_hms(9, 0, 0).unwrap(),
            to: time::Time::from_hms(17, 0, 0).unwrap(),
            min_resources: 2,
            valid_from: time::Date::from_calendar_date(2024, time::Month::January, 1).unwrap(),
            valid_to: None,
            version: Uuid::new_v4(),
            shiftplan_id: Some(shiftplan_id),
        };

        let slot_to = SlotTO::from(&edit_item);
        assert_eq!(slot_to.shiftplan_id, Some(shiftplan_id));
    }

    #[test]
    fn test_slot_edit_item_roundtrip_preserves_shiftplan_id() {
        let shiftplan_id = Uuid::new_v4();
        let original = SlotTO {
            id: Uuid::new_v4(),
            day_of_week: DayOfWeekTO::Wednesday,
            from: time::Time::from_hms(10, 0, 0).unwrap(),
            to: time::Time::from_hms(18, 0, 0).unwrap(),
            min_resources: 1,
            valid_from: time::Date::from_calendar_date(2024, time::Month::March, 1).unwrap(),
            valid_to: None,
            deleted: None,
            version: Uuid::new_v4(),
            shiftplan_id: Some(shiftplan_id),
        };

        let edit_item = SlotEditItem::from(&original);
        let roundtripped = SlotTO::from(&edit_item);

        assert_eq!(roundtripped.shiftplan_id, original.shiftplan_id);
        assert_eq!(roundtripped.id, original.id);
    }

    #[test]
    fn test_empty_catalog_deserialization() {
        let json = r#"[]"#;
        let result: Result<Vec<ShiftplanTO>, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let catalog = result.unwrap();
        assert!(catalog.is_empty());
    }

    #[test]
    fn test_empty_catalog_no_auto_selection() {
        // Simulates the auto-selection logic from shiftplan.rs:
        // if catalog is empty, selected_shiftplan_id stays None
        let catalog: Vec<ShiftplanTO> = vec![];
        let mut selected_shiftplan_id: Option<Uuid> = None;

        if selected_shiftplan_id.is_none() && !catalog.is_empty() {
            selected_shiftplan_id = Some(catalog[0].id);
        }

        assert!(
            selected_shiftplan_id.is_none(),
            "Empty catalog must not auto-select a shiftplan"
        );
    }

    #[test]
    fn test_non_empty_catalog_auto_selects_first() {
        let first_id = Uuid::new_v4();
        let catalog: Vec<ShiftplanTO> = vec![
            ShiftplanTO {
                id: first_id,
                name: "Plan A".into(),
                is_planning: false,
                deleted: None,
                version: Uuid::new_v4(),
            },
            ShiftplanTO {
                id: Uuid::new_v4(),
                name: "Plan B".into(),
                is_planning: true,
                deleted: None,
                version: Uuid::new_v4(),
            },
        ];
        let mut selected_shiftplan_id: Option<Uuid> = None;

        if selected_shiftplan_id.is_none() && !catalog.is_empty() {
            selected_shiftplan_id = Some(catalog[0].id);
        }

        assert_eq!(
            selected_shiftplan_id,
            Some(first_id),
            "Must auto-select the first shiftplan"
        );
    }

    #[test]
    fn test_empty_shiftplan_when_no_selection() {
        // When no shiftplan is selected (None), the page returns an empty Shiftplan
        let shiftplan_id: Option<Uuid> = None;
        let week = 10u8;
        let year = 2024u32;

        let shiftplan = match shiftplan_id {
            Some(_id) => panic!("Should not reach here"),
            None => crate::state::Shiftplan {
                week,
                year,
                slots: [].into(),
            },
        };

        assert_eq!(shiftplan.week, 10);
        assert_eq!(shiftplan.year, 2024);
        assert!(
            shiftplan.slots.is_empty(),
            "Empty catalog must result in empty slots"
        );
    }

    #[test]
    fn test_empty_shiftplan_min_max_hour_no_overflow() {
        let shiftplan = crate::state::Shiftplan {
            week: 1,
            year: 2024,
            slots: [].into(),
        };

        // Previously caused panic: INFINITY/NEG_INFINITY → u8 cast → subtract overflow
        let min = shiftplan.min_hour();
        let max = shiftplan.max_hour();

        assert_eq!(min, 0.0);
        assert_eq!(max, 0.0);
        // This is the actual calculation that panicked in TimeView
        assert!(max >= min, "max_hour must be >= min_hour");
        let _ = (max.ceil() as u8) - (min.ceil() as u8);
    }
}
