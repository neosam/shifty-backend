use std::collections::BTreeMap;
use std::sync::Arc;

use service::billing_period::{BillingPeriod, BillingPeriodSalesPerson, BillingPeriodValue, BillingPeriodValueType};
use service::billing_period::BillingPeriodService;
use service::billing_period_report::BillingPeriodReportService;
use service::sales_person::{SalesPerson, SalesPersonService};
use service::text_template::TextTemplateService;
use service::permission::Authentication;
use shifty_utils::ShiftyDate;
use time::macros::datetime;
use uuid::Uuid;

use crate::integration_test::TestSetup;
use rest::RestStateDef;

#[tokio::test]
async fn test_custom_report_generation_end_to_end() {
    let test_setup = TestSetup::new().await;
    let rest_state = &test_setup.rest_state;
    let context = Authentication::Full;

    // 1. Create sales persons
    let sales_person1 = SalesPerson {
        id: Uuid::nil(),
        version: Uuid::nil(),
        name: "Natalie".into(),
        background_color: "#000000".into(),
        inactive: false,
        is_paid: Some(true),
        deleted: None,
    };

    let sales_person2 = SalesPerson {
        id: Uuid::nil(),
        version: Uuid::nil(),
        name: "Dany".into(),
        background_color: "#000000".into(),
        inactive: false,
        is_paid: Some(true),
        deleted: None,
    };

    let sp1 = rest_state
        .sales_person_service()
        .create(&sales_person1, context.clone(), None)
        .await
        .unwrap();

    let sp2 = rest_state
        .sales_person_service()
        .create(&sales_person2, context.clone(), None)
        .await
        .unwrap();

    // 2. Create billing period with test data
    let mut values1 = BTreeMap::new();
    values1.insert(
        BillingPeriodValueType::Overall,
        BillingPeriodValue {
            value_delta: 4.0,
            value_ytd_from: 0.0,
            value_ytd_to: 4.0,
            value_full_year: 4.0,
        },
    );

    let mut values2 = BTreeMap::new();
    values2.insert(
        BillingPeriodValueType::Overall,
        BillingPeriodValue {
            value_delta: 24.0,
            value_ytd_from: 0.0,
            value_ytd_to: 24.0,
            value_full_year: 24.0,
        },
    );

    let sales_person_data1 = BillingPeriodSalesPerson {
        id: Uuid::new_v4(),
        sales_person_id: sp1.id,
        values: values1,
        created_at: datetime!(2024-01-01 10:00:00),
        created_by: "test".into(),
        deleted_at: None,
        deleted_by: None,
    };

    let sales_person_data2 = BillingPeriodSalesPerson {
        id: Uuid::new_v4(),
        sales_person_id: sp2.id,
        values: values2,
        created_at: datetime!(2024-01-01 10:00:00),
        created_by: "test".into(),
        deleted_at: None,
        deleted_by: None,
    };

    let billing_period = BillingPeriod {
        id: Uuid::nil(),
        start_date: ShiftyDate::from_ymd(2025, 7, 15).unwrap(),
        end_date: ShiftyDate::from_ymd(2025, 8, 14).unwrap(),
        sales_persons: Arc::new([sales_person_data1, sales_person_data2]),
        created_at: datetime!(2025-07-15 10:00:00),
        created_by: "test_user".into(),
        deleted_at: None,
        deleted_by: None,
    };

    let created_billing_period = rest_state
        .billing_period_service()
        .create_billing_period(&billing_period, "test", context.clone(), None)
        .await
        .unwrap();

    // 3. Create German text template
    let template_text = format!(
        "Hallo Frau Saur,\\n\\nhiermit sende ich Ihnen die Stunden für den Abrechnungszeitraum vom {{{{ billing_period.start_date }}}} bis {{{{ billing_period.end_date }}}}.\\n{{% for person in billing_period.sales_persons %}}{{% if person.sales_person_id == \\\"{}\\\" %}}{{% for value in person.values %}}{{% if value.type == \\\"overall\\\" %}}Natalie: {{{{ value.value_delta | round(precision=0) }}}} Stunden\\n{{% endif %}}{{% endfor %}}{{% elif person.sales_person_id == \\\"{}\\\" %}}{{% for value in person.values %}}{{% if value.type == \\\"overall\\\" %}}Dany: {{{{ value.value_delta | round(precision=0) }}}} Stunden\\n{{% endif %}}{{% endfor %}}{{% endif %}}{{% endfor %}}\\nViele Grüße,",
        sp1.id, sp2.id
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

    let created_template = rest_state
        .text_template_service()
        .create(&text_template, context.clone(), None)
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

    // 5. Verify report content
    assert!(generated_report.contains("Hallo Frau Saur"));
    assert!(generated_report.contains("2025-07-15"));
    assert!(generated_report.contains("2025-08-14"));
    assert!(generated_report.contains("Natalie: 4 Stunden"));
    assert!(generated_report.contains("Dany: 24 Stunden"));
    assert!(generated_report.contains("Viele Grüße"));

    println!("Generated report:\\n{}", generated_report);
}

#[tokio::test]
async fn test_custom_report_with_custom_extra_hours() {
    let test_setup = TestSetup::new().await;
    let rest_state = &test_setup.rest_state;
    let context = Authentication::Full;

    // Create sales person
    let sales_person = SalesPerson {
        id: Uuid::nil(),
        version: Uuid::nil(),
        name: "John".into(),
        background_color: "#000000".into(),
        inactive: false,
        is_paid: Some(true),
        deleted: None,
    };

    let sp = rest_state
        .sales_person_service()
        .create(&sales_person, context.clone(), None)
        .await
        .unwrap();

    // Create billing period with custom extra hours
    let mut values = BTreeMap::new();
    values.insert(
        BillingPeriodValueType::Overall,
        BillingPeriodValue {
            value_delta: 160.0,
            value_ytd_from: 0.0,
            value_ytd_to: 160.0,
            value_full_year: 160.0,
        },
    );
    values.insert(
        BillingPeriodValueType::CustomExtraHours("overtime".into()),
        BillingPeriodValue {
            value_delta: 8.0,
            value_ytd_from: 0.0,
            value_ytd_to: 8.0,
            value_full_year: 8.0,
        },
    );
    values.insert(
        BillingPeriodValueType::CustomExtraHours("bonus".into()),
        BillingPeriodValue {
            value_delta: 4.0,
            value_ytd_from: 0.0,
            value_ytd_to: 4.0,
            value_full_year: 4.0,
        },
    );

    let sales_person_data = BillingPeriodSalesPerson {
        id: Uuid::new_v4(),
        sales_person_id: sp.id,
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

    // Create template that extracts custom hours
    let template_text = format!(
        "Custom Hours Report for {{{{ billing_period.start_date }}}} - {{{{ billing_period.end_date }}}}:\\n{{% for person in billing_period.sales_persons %}}{{% if person.sales_person_id == \\\"{}\\\" %}}{{% for value in person.values %}}{{% if value.type starts_with \\\"custom_extra_hours:\\\" %}}{{{{ value.type | replace(from=\\\"custom_extra_hours:\\\", to=\\\"\\\") | title }}}}: {{{{ value.value_delta }}}}h\\n{{% endif %}}{{% endfor %}}{{% endif %}}{{% endfor %}}",
        sp.id
    );

    let text_template = service::text_template::TextTemplate {
        id: Uuid::nil(),
        version: Uuid::nil(),
        template_type: "custom_hours_report".into(),
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

    // Generate custom report
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

    // Verify custom hours are extracted
    assert!(generated_report.contains("2024-01-01"));
    assert!(generated_report.contains("2024-01-31"));
    assert!(generated_report.contains("Overtime: 8h"));
    assert!(generated_report.contains("Bonus: 4h"));

    println!("Custom hours report:\\n{}", generated_report);
}