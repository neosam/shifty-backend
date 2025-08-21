use crate::state::{AuthInfo, Config};
use std::rc::Rc;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_week_calculation() {
    use crate::js::{get_current_week, get_current_year};
    
    let week = get_current_week();
    let year = get_current_year();
    
    assert!(week >= 1 && week <= 53);
    assert!(year >= 2024 && year <= 2100);
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_auth_info_structure() {
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
    fn test_auth_info_default() {
        let auth_info = AuthInfo::default();
        
        assert_eq!(auth_info.user.as_ref(), "");
        assert_eq!(auth_info.privileges.len(), 0);
        assert!(!auth_info.authenticated);
    }

    #[test]
    fn test_config_structure() {
        let config = Config {
            backend: "http://localhost:3000".into(),
            application_title: "Shifty Test".into(),
            is_prod: false,
            env_short_description: "TEST".into(),
            show_vacation: true,
        };
        
        assert_eq!(config.backend.as_ref(), "http://localhost:3000");
        assert_eq!(config.application_title.as_ref(), "Shifty Test");
        assert!(!config.is_prod);
        assert!(config.show_vacation);
    }

    #[test]
    fn test_config_production_settings() {
        let prod_config = Config {
            backend: "https://api.shifty.com".into(),
            application_title: "Shifty".into(),
            is_prod: true,
            env_short_description: "PROD".into(),
            show_vacation: false,
        };
        
        assert!(prod_config.is_prod);
        assert!(prod_config.backend.starts_with("https://"));
        assert!(!prod_config.show_vacation);
    }

    #[test]
    fn test_auth_info_privilege_variations() {
        let auth_info = AuthInfo {
            user: "john.doe@company.com".into(),
            privileges: Rc::new([
                "shift_planner".into(),
                "employee_manager".into(),
                "report_viewer".into(),
                "admin".into(),
            ]),
            authenticated: true,
        };
        
        // Test email-like usernames
        assert!(auth_info.user.contains("@"));
        assert_eq!(auth_info.privileges.len(), 4);
        
        // Test all privileges
        assert!(auth_info.has_privilege("shift_planner"));
        assert!(auth_info.has_privilege("employee_manager"));
        assert!(auth_info.has_privilege("report_viewer"));
        assert!(auth_info.has_privilege("admin"));
        assert!(!auth_info.has_privilege("super_admin"));
    }

    #[test]
    fn test_auth_info_edge_cases() {
        let auth_info = AuthInfo {
            user: "user_with_special-chars.123".into(),
            privileges: Rc::new([
                "privilege-with-hyphen".into(),
                "privilege_with_underscore".into(),
                "PrivilegeWithCaps".into(),
                "".into(), // Empty privilege
            ]),
            authenticated: true,
        };
        
        // Test special characters in username
        assert!(auth_info.user.contains("-"));
        assert!(auth_info.user.contains("."));
        assert!(auth_info.user.contains("_"));
        
        // Test privilege variations
        assert!(auth_info.has_privilege("privilege-with-hyphen"));
        assert!(auth_info.has_privilege("privilege_with_underscore"));
        assert!(auth_info.has_privilege("PrivilegeWithCaps"));
        assert!(auth_info.has_privilege(""));
        
        // Case sensitivity test
        assert!(!auth_info.has_privilege("privilegewithcaps"));
    }

    #[test]
    fn test_uuid_generation() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        
        assert_ne!(uuid1, uuid2);
        assert_ne!(uuid1, Uuid::nil());
        
        // Test UUID format
        let uuid_str = uuid1.to_string();
        assert_eq!(uuid_str.len(), 36); // UUID string length
        assert_eq!(uuid_str.chars().filter(|c| *c == '-').count(), 4); // 4 hyphens in UUID
    }

    #[test]
    fn test_config_environment_variations() {
        let environments = vec![
            ("DEV", false, "http://localhost:3000"),
            ("STAGING", false, "https://staging.api.shifty.com"),
            ("PROD", true, "https://api.shifty.com"),
            ("TEST", false, "http://test.shifty.local"),
        ];
        
        for (env, is_prod, backend) in environments {
            let config = Config {
                backend: backend.into(),
                application_title: "Shifty".into(),
                is_prod,
                env_short_description: env.into(),
                show_vacation: !is_prod,
            };
            
            assert_eq!(config.env_short_description.as_ref(), env);
            assert_eq!(config.is_prod, is_prod);
            assert_eq!(config.backend.as_ref(), backend);
            
            if is_prod {
                assert!(config.backend.starts_with("https://"));
                assert!(!config.show_vacation);
            } else {
                assert!(config.show_vacation);
            }
        }
    }
}