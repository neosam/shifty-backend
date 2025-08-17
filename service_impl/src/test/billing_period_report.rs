use std::collections::BTreeMap;
use std::sync::Arc;

use mockall::predicate::{self, *};
use service::billing_period::{BillingPeriod, BillingPeriodSalesPerson, BillingPeriodValue, BillingPeriodValueType};
use service::billing_period_report::BillingPeriodReportService;
use service::permission::Authentication;
use service::text_template::TextTemplate;
use service::ServiceError;
use time::macros::datetime;
use uuid::Uuid;

use crate::billing_period_report::{BillingPeriodReportServiceImpl, BillingPeriodReportServiceDeps};

struct MockDeps {
    billing_period_service: service::billing_period::MockBillingPeriodService,
    reporting_service: service::reporting::MockReportingService,
    sales_person_service: service::sales_person::MockSalesPersonService,
    text_template_service: service::text_template::MockTextTemplateService,
    permission_service: service::MockPermissionService,
    uuid_service: service::uuid_service::MockUuidService,
    clock_service: service::clock::MockClockService,
    transaction_dao: dao::MockTransactionDao,
}

impl BillingPeriodReportServiceDeps for MockDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type BillingPeriodService = service::billing_period::MockBillingPeriodService;
    type ReportingService = service::reporting::MockReportingService;
    type SalesPersonService = service::sales_person::MockSalesPersonService;
    type TextTemplateService = service::text_template::MockTextTemplateService;
    type PermissionService = service::MockPermissionService;
    type UuidService = service::uuid_service::MockUuidService;
    type ClockService = service::clock::MockClockService;
    type TransactionDao = dao::MockTransactionDao;
}

impl MockDeps {
    fn build_service(self) -> BillingPeriodReportServiceImpl<MockDeps> {
        BillingPeriodReportServiceImpl {
            billing_period_service: self.billing_period_service.into(),
            reporting_service: self.reporting_service.into(),
            sales_person_service: self.sales_person_service.into(),
            text_template_service: self.text_template_service.into(),
            permission_service: self.permission_service.into(),
            uuid_service: self.uuid_service.into(),
            clock_service: self.clock_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

fn create_test_text_template(template_text: &str) -> TextTemplate {
    TextTemplate {
        id: Uuid::new_v4(),
        version: Uuid::new_v4(),
        name: Some("Test Template".into()),
        template_type: "test_template".into(),
        template_text: template_text.into(),
        created_at: Some(datetime!(2024-01-01 10:00:00)),
        created_by: Some("test_user".into()),
        deleted: None,
        deleted_by: None,
    }
}

fn create_test_billing_period() -> BillingPeriod {
    let mut values1 = BTreeMap::new();
    values1.insert(
        BillingPeriodValueType::Overall,
        BillingPeriodValue {
            value_delta: 160.0,
            value_ytd_from: 320.0,
            value_ytd_to: 480.0,
            value_full_year: 1920.0,
        },
    );
    values1.insert(
        BillingPeriodValueType::Balance,
        BillingPeriodValue {
            value_delta: 10.0,
            value_ytd_from: 20.0,
            value_ytd_to: 30.0,
            value_full_year: 120.0,
        },
    );
    values1.insert(
        BillingPeriodValueType::CustomExtraHours("overtime".into()),
        BillingPeriodValue {
            value_delta: 8.0,
            value_ytd_from: 16.0,
            value_ytd_to: 24.0,
            value_full_year: 96.0,
        },
    );

    let mut values2 = BTreeMap::new();
    values2.insert(
        BillingPeriodValueType::Overall,
        BillingPeriodValue {
            value_delta: 140.0,
            value_ytd_from: 280.0,
            value_ytd_to: 420.0,
            value_full_year: 1680.0,
        },
    );
    values2.insert(
        BillingPeriodValueType::Balance,
        BillingPeriodValue {
            value_delta: -10.0,
            value_ytd_from: -20.0,
            value_ytd_to: -30.0,
            value_full_year: -120.0,
        },
    );

    let sales_person1 = BillingPeriodSalesPerson {
        id: Uuid::new_v4(),
        sales_person_id: Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap(),
        values: values1,
        created_at: datetime!(2024-01-01 10:00:00),
        created_by: "test_user".into(),
        deleted_at: None,
        deleted_by: None,
    };

    let sales_person2 = BillingPeriodSalesPerson {
        id: Uuid::new_v4(),
        sales_person_id: Uuid::parse_str("87654321-4321-4321-4321-210987654321").unwrap(),
        values: values2,
        created_at: datetime!(2024-01-01 10:00:00),
        created_by: "test_user".into(),
        deleted_at: None,
        deleted_by: None,
    };

    BillingPeriod {
        id: Uuid::new_v4(),
        start_date: shifty_utils::ShiftyDate::from_ymd(2024, 7, 15).unwrap(),
        end_date: shifty_utils::ShiftyDate::from_ymd(2024, 8, 14).unwrap(),
        sales_persons: Arc::new([sales_person1, sales_person2]),
        created_at: datetime!(2024-01-01 10:00:00),
        created_by: "test_user".into(),
        deleted_at: None,
        deleted_by: None,
    }
}

#[tokio::test]
async fn test_generate_custom_report_success() {
    let template_id = Uuid::new_v4();
    let billing_period_id = Uuid::new_v4();
    let context = Authentication::Full;
    
    // Create a simple template that extracts specific employee data
    let template = create_test_text_template(
        "Employee Report:\\n{% for person in billing_period.sales_persons %}{% if person.sales_person_id == \"12345678-1234-1234-1234-123456789012\" %}{% for value in person.values %}{% if value.type == \"overall\" %}Employee 1: {{ value.value_delta }}h{% endif %}{% endfor %}{% endif %}{% endfor %}"
    );
    
    let billing_period = create_test_billing_period();

    // Setup mocks
    let mut deps = MockDeps {
        billing_period_service: service::billing_period::MockBillingPeriodService::new(),
        reporting_service: service::reporting::MockReportingService::new(),
        sales_person_service: service::sales_person::MockSalesPersonService::new(),
        text_template_service: service::text_template::MockTextTemplateService::new(),
        permission_service: service::MockPermissionService::new(),
        uuid_service: service::uuid_service::MockUuidService::new(),
        clock_service: service::clock::MockClockService::new(),
        transaction_dao: dao::MockTransactionDao::new(),
    };

    deps.transaction_dao
        .expect_use_transaction()
        .with(predicate::always())
        .times(1)
        .returning(|_| Ok(dao::MockTransaction));

    deps.permission_service
        .expect_check_permission()
        .with(eq(service::permission::HR_PRIVILEGE), eq(context.clone()))
        .times(1)
        .returning(|_, _| Ok(()));

    deps.text_template_service
        .expect_get_by_id()
        .with(eq(template_id), eq(context.clone()), always())
        .times(1)
        .returning(move |_, _, _| Ok(template.clone()));

    deps.billing_period_service
        .expect_get_billing_period_by_id()
        .with(eq(billing_period_id), eq(context.clone()), always())
        .times(1)
        .returning(move |_, _, _| Ok(billing_period.clone()));

    deps.transaction_dao
        .expect_commit()
        .with(always())
        .times(1)
        .returning(|_| Ok(()));

    let service = deps.build_service();

    // Execute
    let result = service
        .generate_custom_report(template_id, billing_period_id, context, None)
        .await;

    // Assert
    if let Err(e) = &result {
        eprintln!("Error in test_generate_custom_report_success: {:?}", e);
    }
    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(report.contains("Employee 1: 160h"));
}

#[tokio::test]
async fn test_generate_custom_report_german_template() {
    let template_id = Uuid::new_v4();
    let billing_period_id = Uuid::new_v4();
    let context = Authentication::Full;
    
    // German template like the user requested
    let template = create_test_text_template(
        "Hallo Frau Saur,\\n\\nhiermit sende ich Ihnen die Stunden für den Abrechnungszeitraum vom {{ billing_period.start_date }} bis {{ billing_period.end_date }}.\\n{% for person in billing_period.sales_persons %}{% if person.sales_person_id == \"12345678-1234-1234-1234-123456789012\" %}{% for value in person.values %}{% if value.type == \"overall\" %}Natalie: {{ value.value_delta | round(precision=0) }} Stunden\\n{% endif %}{% endfor %}{% elif person.sales_person_id == \"87654321-4321-4321-4321-210987654321\" %}{% for value in person.values %}{% if value.type == \"overall\" %}Dany: {{ value.value_delta | round(precision=0) }} Stunden\\n{% endif %}{% endfor %}{% endif %}{% endfor %}\\nViele Grüße,"
    );
    
    let billing_period = create_test_billing_period();

    // Setup mocks
    let mut deps = MockDeps {
        billing_period_service: service::billing_period::MockBillingPeriodService::new(),
        reporting_service: service::reporting::MockReportingService::new(),
        sales_person_service: service::sales_person::MockSalesPersonService::new(),
        text_template_service: service::text_template::MockTextTemplateService::new(),
        permission_service: service::MockPermissionService::new(),
        uuid_service: service::uuid_service::MockUuidService::new(),
        clock_service: service::clock::MockClockService::new(),
        transaction_dao: dao::MockTransactionDao::new(),
    };

    deps.transaction_dao
        .expect_use_transaction()
        .with(predicate::always())
        .times(1)
        .returning(|_| Ok(dao::MockTransaction));

    deps.permission_service
        .expect_check_permission()
        .with(eq(service::permission::HR_PRIVILEGE), eq(context.clone()))
        .times(1)
        .returning(|_, _| Ok(()));

    deps.text_template_service
        .expect_get_by_id()
        .with(eq(template_id), eq(context.clone()), always())
        .times(1)
        .returning(move |_, _, _| Ok(template.clone()));

    deps.billing_period_service
        .expect_get_billing_period_by_id()
        .with(eq(billing_period_id), eq(context.clone()), always())
        .times(1)
        .returning(move |_, _, _| Ok(billing_period.clone()));

    deps.transaction_dao
        .expect_commit()
        .with(always())
        .times(1)
        .returning(|_| Ok(()));

    let service = deps.build_service();

    // Execute
    let result = service
        .generate_custom_report(template_id, billing_period_id, context, None)
        .await;

    // Assert
    if let Err(e) = &result {
        eprintln!("Error in test_generate_custom_report_german_template: {:?}", e);
    }
    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(report.contains("Hallo Frau Saur"));
    assert!(report.contains("2024-07-15"));
    assert!(report.contains("2024-08-14"));
    assert!(report.contains("Natalie: 160 Stunden"));
    assert!(report.contains("Dany: 140 Stunden"));
    assert!(report.contains("Viele Grüße"));
}

#[tokio::test]
async fn test_generate_custom_report_no_hr_permission() {
    let template_id = Uuid::new_v4();
    let billing_period_id = Uuid::new_v4();
    let context = Authentication::Full;

    // Setup mocks
    let mut deps = MockDeps {
        billing_period_service: service::billing_period::MockBillingPeriodService::new(),
        reporting_service: service::reporting::MockReportingService::new(),
        sales_person_service: service::sales_person::MockSalesPersonService::new(),
        text_template_service: service::text_template::MockTextTemplateService::new(),
        permission_service: service::MockPermissionService::new(),
        uuid_service: service::uuid_service::MockUuidService::new(),
        clock_service: service::clock::MockClockService::new(),
        transaction_dao: dao::MockTransactionDao::new(),
    };

    deps.transaction_dao
        .expect_use_transaction()
        .with(predicate::always())
        .times(1)
        .returning(|_| Ok(dao::MockTransaction));

    deps.permission_service
        .expect_check_permission()
        .with(eq(service::permission::HR_PRIVILEGE), eq(context.clone()))
        .times(1)
        .returning(|_, _| Err(ServiceError::Forbidden));

    let service = deps.build_service();

    // Execute
    let result = service
        .generate_custom_report(template_id, billing_period_id, context, None)
        .await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ServiceError::Forbidden));
}

#[tokio::test]
async fn test_generate_custom_report_template_not_found() {
    let template_id = Uuid::new_v4();
    let billing_period_id = Uuid::new_v4();
    let context = Authentication::Full;

    // Setup mocks
    let mut deps = MockDeps {
        billing_period_service: service::billing_period::MockBillingPeriodService::new(),
        reporting_service: service::reporting::MockReportingService::new(),
        sales_person_service: service::sales_person::MockSalesPersonService::new(),
        text_template_service: service::text_template::MockTextTemplateService::new(),
        permission_service: service::MockPermissionService::new(),
        uuid_service: service::uuid_service::MockUuidService::new(),
        clock_service: service::clock::MockClockService::new(),
        transaction_dao: dao::MockTransactionDao::new(),
    };

    deps.transaction_dao
        .expect_use_transaction()
        .with(predicate::always())
        .times(1)
        .returning(|_| Ok(dao::MockTransaction));

    deps.permission_service
        .expect_check_permission()
        .with(eq(service::permission::HR_PRIVILEGE), eq(context.clone()))
        .times(1)
        .returning(|_, _| Ok(()));

    deps.text_template_service
        .expect_get_by_id()
        .with(eq(template_id), eq(context.clone()), always())
        .times(1)
        .returning(|_, _, _| Err(ServiceError::EntityNotFoundGeneric("TextTemplate not found".into())));

    let service = deps.build_service();

    // Execute
    let result = service
        .generate_custom_report(template_id, billing_period_id, context, None)
        .await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ServiceError::EntityNotFoundGeneric(_)));
}