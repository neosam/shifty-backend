// Simple integration test example for custom reports
// This would go in shifty_bin/src/integration_test.rs

use rest::RestStateDef;
use service::billing_period::{BillingPeriod, BillingPeriodSalesPerson, BillingPeriodValue, BillingPeriodValueType};
use service::billing_period::BillingPeriodService;
use service::sales_person::{SalesPerson, SalesPersonService};
use service::text_template::TextTemplateService;
use service::permission::Authentication;
use shifty_utils::ShiftyDate;
use time::macros::datetime;
use uuid::Uuid;
use std::collections::BTreeMap;
use std::sync::Arc;

#[tokio::test]
async fn test_simple_custom_report_generation() {
    // Create test setup (using existing TestSetup pattern)
    let test_setup = TestSetup::new().await;
    let rest_state = &test_setup.rest_state;
    let context = Authentication::Full;

    // 1. Create a simple sales person
    let sales_person = SalesPerson {
        id: Uuid::nil(),
        version: Uuid::nil(),
        name: "Test Employee".into(),
        background_color: "#000000".into(),
        inactive: false,
        is_paid: Some(true),
        deleted: None,
    };

    let created_sp = rest_state
        .sales_person_service()
        .create(&sales_person, context.clone(), None)
        .await
        .unwrap();

    // 2. Create a simple text template
    let template_text = format!(
        "Employee Report:\\n{% for person in billing_period.sales_persons %}{% if person.sales_person_id == \"{}\" %}Found employee!{% endif %}{% endfor %}",
        created_sp.id
    );

    let text_template = service::text_template::TextTemplate {
        id: Uuid::nil(),
        version: Uuid::nil(),
        template_type: "simple_test".into(),
        template_text: template_text.into(),
        created_at: None,
        created_by: None,
        deleted: None,
        deleted_by: None,
    };

    let created_template = rest_state
        .text_template_service()
        .create(&text_template, context.clone(), None)
        .await
        .unwrap();

    // 3. Create a billing period with the sales person
    let mut values = BTreeMap::new();
    values.insert(
        BillingPeriodValueType::Overall,
        BillingPeriodValue {
            value_delta: 40.0,
            value_ytd_from: 0.0,
            value_ytd_to: 40.0,
            value_full_year: 40.0,
        },
    );

    let sales_person_data = BillingPeriodSalesPerson {
        id: Uuid::new_v4(),
        sales_person_id: created_sp.id,
        values,
        created_at: datetime!(2024-01-01 10:00:00),
        created_by: "test".into(),
        deleted_at: None,
        deleted_by: None,
    };

    let billing_period = BillingPeriod {
        id: Uuid::nil(),
        start_date: ShiftyDate::from_ymd(2024, 1, 1).unwrap(),
        end_date: ShiftyDate::from_ymd(2024, 1, 31).unwrap(),
        sales_persons: Arc::new([sales_person_data]),
        created_at: datetime!(2024-01-01 10:00:00),
        created_by: "test_user".into(),
        deleted_at: None,
        deleted_by: None,
    };

    let created_billing_period = rest_state
        .billing_period_service()
        .create_billing_period(&billing_period, "test", context.clone(), None)
        .await
        .unwrap();

    // 4. Generate custom report
    let generated_report = rest_state
        .billing_period_report_service()
        .generate_custom_report(
            created_template.id,
            created_billing_period.id,
            context,
            None,
        )
        .await
        .unwrap();

    // 5. Verify the report
    assert!(generated_report.contains("Employee Report:"));
    assert!(generated_report.contains("Found employee!"));

    println!("✅ Simple custom report test passed");
    println!("Generated report: {}", generated_report);
}

#[tokio::test] 
async fn test_german_hours_report() {
    let test_setup = TestSetup::new().await;
    let rest_state = &test_setup.rest_state;
    let context = Authentication::Full;

    // Create two employees: Natalie and Dany
    let natalie = SalesPerson {
        id: Uuid::nil(),
        version: Uuid::nil(),
        name: "Natalie".into(),
        background_color: "#ff0000".into(),
        inactive: false,
        is_paid: Some(true),
        deleted: None,
    };

    let dany = SalesPerson {
        id: Uuid::nil(),
        version: Uuid::nil(),
        name: "Dany".into(),
        background_color: "#00ff00".into(),
        inactive: false,
        is_paid: Some(true),
        deleted: None,
    };

    let created_natalie = rest_state.sales_person_service()
        .create(&natalie, context.clone(), None).await.unwrap();
    let created_dany = rest_state.sales_person_service()
        .create(&dany, context.clone(), None).await.unwrap();

    // Create German template
    let template_text = format!(
        "Hallo Frau Saur,\\n\\nhiermit sende ich Ihnen die Stunden für den Abrechnungszeitraum vom {{{{ billing_period.start_date }}}} bis {{{{ billing_period.end_date }}}}.\\n{{% for person in billing_period.sales_persons %}}{{% if person.sales_person_id == \\\"{}\\\" %}}{{% for value in person.values %}}{{% if value.type == \\\"overall\\\" %}}Natalie: {{{{ value.value_delta | round(precision=0) }}}} Stunden\\n{{% endif %}}{{% endfor %}}{{% elif person.sales_person_id == \\\"{}\\\" %}}{{% for value in person.values %}}{{% if value.type == \\\"overall\\\" %}}Dany: {{{{ value.value_delta | round(precision=0) }}}} Stunden\\n{{% endif %}}{{% endfor %}}{{% endif %}}{{% endfor %}}\\nViele Grüße,",
        created_natalie.id, created_dany.id
    );

    let text_template = service::text_template::TextTemplate {
        id: Uuid::nil(),
        version: Uuid::nil(),
        template_type: "german_hours_report".into(),
        template_text: template_text.into(),
        created_at: None,
        created_by: None,
        deleted: None,
        deleted_by: None,
    };

    let created_template = rest_state.text_template_service()
        .create(&text_template, context.clone(), None).await.unwrap();

    // Create billing period with both employees
    let mut natalie_values = BTreeMap::new();
    natalie_values.insert(BillingPeriodValueType::Overall, BillingPeriodValue {
        value_delta: 4.0, value_ytd_from: 0.0, value_ytd_to: 4.0, value_full_year: 4.0,
    });

    let mut dany_values = BTreeMap::new(); 
    dany_values.insert(BillingPeriodValueType::Overall, BillingPeriodValue {
        value_delta: 24.0, value_ytd_from: 0.0, value_ytd_to: 24.0, value_full_year: 24.0,
    });

    let sales_persons_data = vec![
        BillingPeriodSalesPerson {
            id: Uuid::new_v4(),
            sales_person_id: created_natalie.id,
            values: natalie_values,
            created_at: datetime!(2025-07-15 10:00:00),
            created_by: "test".into(),
            deleted_at: None,
            deleted_by: None,
        },
        BillingPeriodSalesPerson {
            id: Uuid::new_v4(),
            sales_person_id: created_dany.id,
            values: dany_values,
            created_at: datetime!(2025-07-15 10:00:00),
            created_by: "test".into(),
            deleted_at: None,
            deleted_by: None,
        },
    ];

    let billing_period = BillingPeriod {
        id: Uuid::nil(),
        start_date: ShiftyDate::from_ymd(2025, 7, 15).unwrap(),
        end_date: ShiftyDate::from_ymd(2025, 8, 14).unwrap(),
        sales_persons: sales_persons_data.into(),
        created_at: datetime!(2025-07-15 10:00:00),
        created_by: "test_user".into(),
        deleted_at: None,
        deleted_by: None,
    };

    let created_billing_period = rest_state.billing_period_service()
        .create_billing_period(&billing_period, "test", context.clone(), None).await.unwrap();

    // Generate the German report
    let generated_report = rest_state.billing_period_report_service()
        .generate_custom_report(created_template.id, created_billing_period.id, context, None)
        .await.unwrap();

    // Verify German report content
    assert!(generated_report.contains("Hallo Frau Saur"));
    assert!(generated_report.contains("2025-07-15"));
    assert!(generated_report.contains("2025-08-14"));
    assert!(generated_report.contains("Natalie: 4 Stunden"));
    assert!(generated_report.contains("Dany: 24 Stunden"));
    assert!(generated_report.contains("Viele Grüße"));

    println!("✅ German hours report test passed");
    println!("Generated report: {}", generated_report);
}

// Usage instructions:
// 1. Add these tests to shifty_bin/src/integration_test.rs 
// 2. Import the TestSetup struct that's already defined there
// 3. Run with: cargo test --features mock_auth test_simple_custom_report_generation
// 4. Run with: cargo test --features mock_auth test_german_hours_report