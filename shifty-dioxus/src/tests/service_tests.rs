#[cfg(test)]
mod text_template_service_tests {
    use crate::service::text_template::*;
    use crate::state::text_template::TextTemplate;
    use std::rc::Rc;
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
            template_text: "This is a test template".into(),
            created_at: None,
            created_by: None,
        };
        
        assert!(template.name.is_some());
        assert_eq!(template.template_type.as_ref(), "billing-period");
        assert_eq!(template.template_text.as_ref(), "This is a test template");
    }

    #[test]
    fn test_text_template_action_enum() {
        // Test that we can create various action types
        let actions = vec![
            TextTemplateAction::LoadTemplates,
            TextTemplateAction::LoadTemplatesByType("billing-period".to_string()),
            TextTemplateAction::SaveTemplate(TextTemplate {
                id: Uuid::new_v4(),
                name: None,
                template_type: "test".into(),
                template_text: "test content".into(),
                created_at: None,
                created_by: None,
            }),
        ];
        
        assert_eq!(actions.len(), 3);
    }
}

#[cfg(test)]
mod billing_period_service_tests {
    use crate::service::billing_period::*;
    use uuid::Uuid;

    #[test]
    fn test_billing_period_store_default() {
        let store = BillingPeriodStore::default();
        
        assert_eq!(store.billing_periods.len(), 0);
        assert!(store.selected_billing_period.is_none());
        assert!(store.extended_employee_reports.is_none());
    }

    #[test]
    fn test_billing_period_action_variants() {
        // Test that all action variants can be created
        let actions = vec![
            BillingPeriodAction::LoadAllBillingPeriods,
            BillingPeriodAction::LoadBillingPeriod(Uuid::new_v4()),
            BillingPeriodAction::CreateBillingPeriod(2024, 1),
        ];
        
        assert_eq!(actions.len(), 3);
    }
}

#[cfg(test)]
mod employee_service_tests {
    use crate::service::employee::*;
    use crate::state::employee::Employee;
    use uuid::Uuid;

    #[test]
    fn test_employee_store_default() {
        let store = EmployeeStore::default();
        
        assert_eq!(store.employees.len(), 0);
        assert!(store.selected_employee.is_none());
    }

    #[test]
    fn test_employee_creation() {
        let employee = Employee {
            id: Uuid::new_v4(),
            name: "John Doe".into(),
            active: true,
            extra_hours_september_2024: 0.0,
            expected_hours: 40.0,
            description: None,
        };
        
        assert_eq!(employee.name.as_ref(), "John Doe");
        assert!(employee.active);
        assert_eq!(employee.expected_hours, 40.0);
    }

    #[test]
    fn test_employee_action_variants() {
        let actions = vec![
            EmployeeAction::LoadEmployees,
            EmployeeAction::LoadEmployee(Uuid::new_v4()),
        ];
        
        assert_eq!(actions.len(), 2);
    }
}

#[cfg(test)]
mod weekly_summary_tests {
    use crate::service::weekly_summary::*;

    #[test]
    fn test_weekly_summary_store_default() {
        let store = WeeklySummaryStore::default();
        
        assert!(store.weekly_summary.is_none());
        assert!(store.is_loading);
    }

    #[test]
    fn test_weekly_summary_action_variants() {
        let action = WeeklySummaryAction::LoadWeeklySummary(2024, 1);
        
        match action {
            WeeklySummaryAction::LoadWeeklySummary(year, week) => {
                assert_eq!(year, 2024);
                assert_eq!(week, 1);
            }
        }
    }
}