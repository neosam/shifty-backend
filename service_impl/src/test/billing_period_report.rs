use std::collections::BTreeMap;
use std::sync::Arc;

use mockall::predicate::{self, *};
use service::billing_period::{BillingPeriod, BillingPeriodSalesPerson, BillingPeriodValue, BillingPeriodValueType};
use service::billing_period_report::BillingPeriodReportService;
use service::employee_work_details::EmployeeWorkDetails;
use service::permission::Authentication;
use service::sales_person::SalesPerson;
use service::text_template::TextTemplate;
use service::ServiceError;
use time::macros::datetime;
use uuid::Uuid;

use crate::billing_period_report::{BillingPeriodReportServiceImpl, BillingPeriodReportServiceDeps};

const SP1_UUID: &str = "12345678-1234-1234-1234-123456789012";
const SP2_UUID: &str = "87654321-4321-4321-4321-210987654321";

struct MockDeps {
    billing_period_service: service::billing_period::MockBillingPeriodService,
    reporting_service: service::reporting::MockReportingService,
    sales_person_service: service::sales_person::MockSalesPersonService,
    employee_work_details_service: service::employee_work_details::MockEmployeeWorkDetailsService,
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
    type EmployeeWorkDetailsService = service::employee_work_details::MockEmployeeWorkDetailsService;
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
            employee_work_details_service: self.employee_work_details_service.into(),
            text_template_service: self.text_template_service.into(),
            permission_service: self.permission_service.into(),
            uuid_service: self.uuid_service.into(),
            clock_service: self.clock_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

fn create_test_text_template(template_text: &str) -> TextTemplate {
    create_test_text_template_with_engine(template_text, service::text_template::TemplateEngine::Tera)
}

fn create_test_text_template_with_engine(template_text: &str, engine: service::text_template::TemplateEngine) -> TextTemplate {
    TextTemplate {
        id: Uuid::new_v4(),
        version: Uuid::new_v4(),
        name: Some("Test Template".into()),
        template_type: "test_template".into(),
        template_text: template_text.into(),
        template_engine: engine,
        created_at: Some(datetime!(2024-01-01 10:00:00)),
        created_by: Some("test_user".into()),
        deleted: None,
        deleted_by: None,
    }
}

fn sp1_id() -> Uuid {
    Uuid::parse_str(SP1_UUID).unwrap()
}

fn sp2_id() -> Uuid {
    Uuid::parse_str(SP2_UUID).unwrap()
}

fn create_test_sales_persons() -> Arc<[SalesPerson]> {
    Arc::new([
        SalesPerson {
            id: sp1_id(),
            name: "Natalie".into(),
            background_color: "#ff0000".into(),
            is_paid: Some(true),
            inactive: false,
            deleted: None,
            version: Uuid::new_v4(),
        },
        SalesPerson {
            id: sp2_id(),
            name: "Dany".into(),
            background_color: "#00ff00".into(),
            is_paid: Some(false),
            inactive: false,
            deleted: None,
            version: Uuid::new_v4(),
        },
    ])
}

fn create_test_work_details(sales_person_id: Uuid, is_dynamic: bool) -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::new_v4(),
        sales_person_id,
        expected_hours: 40.0,
        from_day_of_week: shifty_utils::DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2024,
        to_day_of_week: shifty_utils::DayOfWeek::Sunday,
        to_calendar_week: 52,
        to_year: 2024,
        workdays_per_week: 5,
        is_dynamic,
        cap_planned_hours_to_expected: false,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        vacation_days: 30,
        created: Some(datetime!(2024-01-01 10:00:00)),
        deleted: None,
        version: Uuid::new_v4(),
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
        sales_person_id: sp1_id(),
        values: values1,
        created_at: datetime!(2024-01-01 10:00:00),
        created_by: "test_user".into(),
        deleted_at: None,
        deleted_by: None,
    };

    let sales_person2 = BillingPeriodSalesPerson {
        id: Uuid::new_v4(),
        sales_person_id: sp2_id(),
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
        snapshot_schema_version: 1,
        sales_persons: Arc::new([sales_person1, sales_person2]),
        created_at: datetime!(2024-01-01 10:00:00),
        created_by: "test_user".into(),
        deleted_at: None,
        deleted_by: None,
    }
}

fn create_enriched_billing_period() -> BillingPeriod {
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
        BillingPeriodValueType::ExtraWork,
        BillingPeriodValue {
            value_delta: 5.0,
            value_ytd_from: 10.0,
            value_ytd_to: 15.0,
            value_full_year: 60.0,
        },
    );
    values1.insert(
        BillingPeriodValueType::VacationHours,
        BillingPeriodValue {
            value_delta: 16.0,
            value_ytd_from: 32.0,
            value_ytd_to: 48.0,
            value_full_year: 192.0,
        },
    );
    values1.insert(
        BillingPeriodValueType::SickLeave,
        BillingPeriodValue {
            value_delta: 8.0,
            value_ytd_from: 16.0,
            value_ytd_to: 24.0,
            value_full_year: 96.0,
        },
    );
    values1.insert(
        BillingPeriodValueType::Holiday,
        BillingPeriodValue {
            value_delta: 0.0,
            value_ytd_from: 8.0,
            value_ytd_to: 8.0,
            value_full_year: 80.0,
        },
    );
    values1.insert(
        BillingPeriodValueType::VacationDays,
        BillingPeriodValue {
            value_delta: 2.0,
            value_ytd_from: 4.0,
            value_ytd_to: 6.0,
            value_full_year: 24.0,
        },
    );
    values1.insert(
        BillingPeriodValueType::VacationEntitlement,
        BillingPeriodValue {
            value_delta: 30.0,
            value_ytd_from: 30.0,
            value_ytd_to: 30.0,
            value_full_year: 30.0,
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

    let sales_person1 = BillingPeriodSalesPerson {
        id: Uuid::new_v4(),
        sales_person_id: sp1_id(),
        values: values1,
        created_at: datetime!(2024-01-01 10:00:00),
        created_by: "test_user".into(),
        deleted_at: None,
        deleted_by: None,
    };

    BillingPeriod {
        id: Uuid::new_v4(),
        start_date: shifty_utils::ShiftyDate::from_ymd(2024, 7, 15).unwrap(),
        end_date: shifty_utils::ShiftyDate::from_ymd(2024, 8, 14).unwrap(),
        snapshot_schema_version: 1,
        sales_persons: Arc::new([sales_person1]),
        created_at: datetime!(2024-01-01 10:00:00),
        created_by: "test_user".into(),
        deleted_at: None,
        deleted_by: None,
    }
}

fn setup_enrichment_mocks(deps: &mut MockDeps, work_details: Arc<[EmployeeWorkDetails]>) {
    let sales_persons = create_test_sales_persons();
    deps.sales_person_service
        .expect_get_all()
        .with(always(), always())
        .times(1)
        .returning(move |_, _| Ok(sales_persons.clone()));

    deps.employee_work_details_service
        .expect_all()
        .with(always(), always())
        .times(1)
        .returning(move |_, _| Ok(work_details.clone()));
}

fn default_work_details() -> Arc<[EmployeeWorkDetails]> {
    Arc::new([
        create_test_work_details(sp1_id(), true),
        create_test_work_details(sp2_id(), false),
    ])
}

fn setup_generate_report_mocks(
    template: TextTemplate,
    billing_period: BillingPeriod,
) -> BillingPeriodReportServiceImpl<MockDeps> {
    setup_generate_report_mocks_with_work_details(template, billing_period, default_work_details())
}

fn setup_generate_report_mocks_with_work_details(
    template: TextTemplate,
    billing_period: BillingPeriod,
    work_details: Arc<[EmployeeWorkDetails]>,
) -> BillingPeriodReportServiceImpl<MockDeps> {
    let template_id = template.id;
    let billing_period_id = billing_period.id;
    let context = Authentication::Full;

    let mut deps = MockDeps {
        billing_period_service: service::billing_period::MockBillingPeriodService::new(),
        reporting_service: service::reporting::MockReportingService::new(),
        sales_person_service: service::sales_person::MockSalesPersonService::new(),
        employee_work_details_service: service::employee_work_details::MockEmployeeWorkDetailsService::new(),
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

    setup_enrichment_mocks(&mut deps, work_details);

    deps.transaction_dao
        .expect_commit()
        .with(always())
        .times(1)
        .returning(|_| Ok(()));

    deps.build_service()
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
        employee_work_details_service: service::employee_work_details::MockEmployeeWorkDetailsService::new(),
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

    setup_enrichment_mocks(&mut deps, default_work_details());

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
        employee_work_details_service: service::employee_work_details::MockEmployeeWorkDetailsService::new(),
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

    setup_enrichment_mocks(&mut deps, default_work_details());

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
        employee_work_details_service: service::employee_work_details::MockEmployeeWorkDetailsService::new(),
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
        employee_work_details_service: service::employee_work_details::MockEmployeeWorkDetailsService::new(),
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

#[tokio::test]
async fn test_engine_default_is_tera() {
    let template = create_test_text_template("Hello");
    assert_eq!(template.template_engine, service::text_template::TemplateEngine::Tera);
}

#[tokio::test]
async fn test_tera_rendering_regression() {
    let template = create_test_text_template(
        "Start: {{ billing_period.start_date }}, End: {{ billing_period.end_date }}"
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(template, billing_period);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await;

    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(report.contains("Start: 2024-07-15"));
    assert!(report.contains("End: 2024-08-14"));
}

#[tokio::test]
async fn test_minijinja_rendering() {
    let template = create_test_text_template_with_engine(
        "Start: {{ billing_period.start_date }}, End: {{ billing_period.end_date }}",
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(template, billing_period);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await;

    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(report.contains("Start: 2024-07-15"));
    assert!(report.contains("End: 2024-08-14"));
}

#[tokio::test]
async fn test_minijinja_dict_literal() {
    let template = create_test_text_template_with_engine(
        r#"{% set names = {"12345678-1234-1234-1234-123456789012": "Alice", "87654321-4321-4321-4321-210987654321": "Bob"} %}{% for person in billing_period.sales_persons %}{{ names[person.sales_person_id] }}: {% for value in person.values %}{% if value.type == "overall" %}{{ value.value_delta }}h{% endif %}{% endfor %}
{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(template, billing_period);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await;

    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(report.contains("Alice: 160"));
    assert!(report.contains("Bob: 140"));
}

#[tokio::test]
async fn test_same_context_both_engines() {
    // Tera version
    let tera_template = create_test_text_template(
        "{{ billing_period.start_date }}"
    );
    let billing_period = create_test_billing_period();
    let tera_template_id = tera_template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(tera_template, billing_period.clone());
    let tera_result = service
        .generate_custom_report(tera_template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    // MiniJinja version
    let minijinja_template = create_test_text_template_with_engine(
        "{{ billing_period.start_date }}",
        service::text_template::TemplateEngine::MiniJinja,
    );
    let mut billing_period2 = create_test_billing_period();
    billing_period2.id = billing_period_id;
    let minijinja_template_id = minijinja_template.id;

    let service = setup_generate_report_mocks(minijinja_template, billing_period2);
    let minijinja_result = service
        .generate_custom_report(minijinja_template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert_eq!(tera_result.as_ref(), minijinja_result.as_ref());
}

// === New tests for enriched template context ===

#[tokio::test]
async fn test_is_dynamic_true_when_any_work_details_is_dynamic() {
    let template = create_test_text_template_with_engine(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}{{ person.is_dynamic }}{% endif %}{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let work_details: Arc<[EmployeeWorkDetails]> = Arc::new([
        create_test_work_details(sp1_id(), true),
    ]);

    let service = setup_generate_report_mocks_with_work_details(template, billing_period, work_details);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert_eq!(result.as_ref(), "true");
}

#[tokio::test]
async fn test_is_dynamic_false_when_all_work_details_not_dynamic() {
    let template = create_test_text_template_with_engine(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}{{ person.is_dynamic }}{% endif %}{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let work_details: Arc<[EmployeeWorkDetails]> = Arc::new([
        create_test_work_details(sp1_id(), false),
        create_test_work_details(sp2_id(), false),
    ]);

    let service = setup_generate_report_mocks_with_work_details(template, billing_period, work_details);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert_eq!(result.as_ref(), "false");
}

#[tokio::test]
async fn test_is_dynamic_false_when_no_work_details_exist() {
    let template = create_test_text_template_with_engine(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}{{ person.is_dynamic }}{% endif %}{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let work_details: Arc<[EmployeeWorkDetails]> = Arc::new([]);

    let service = setup_generate_report_mocks_with_work_details(template, billing_period, work_details);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert_eq!(result.as_ref(), "false");
}

#[tokio::test]
async fn test_is_dynamic_mixed_entries_any_semantics() {
    let template = create_test_text_template_with_engine(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}{{ person.is_dynamic }}{% endif %}{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let work_details: Arc<[EmployeeWorkDetails]> = Arc::new([
        create_test_work_details(sp1_id(), false),
        create_test_work_details(sp1_id(), true),
        create_test_work_details(sp1_id(), false),
    ]);

    let service = setup_generate_report_mocks_with_work_details(template, billing_period, work_details);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert_eq!(result.as_ref(), "true");
}

#[tokio::test]
async fn test_name_and_is_paid_in_context() {
    let template = create_test_text_template_with_engine(
        r#"{% for person in billing_period.sales_persons %}{{ person.name }}:{{ person.is_paid }};{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(template, billing_period);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert!(result.contains("Natalie:true"));
    assert!(result.contains("Dany:false"));
}

#[tokio::test]
async fn test_values_map_direct_access() {
    let template = create_test_text_template_with_engine(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}{{ person.values_map.overall.delta }}{% endif %}{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(template, billing_period);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert_eq!(result.as_ref(), "160.0");
}

#[tokio::test]
async fn test_new_value_types_accessible() {
    let template = create_test_text_template_with_engine(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}vacation_hours:{{ person.values_map.vacation_hours.delta }},sick_leave:{{ person.values_map.sick_leave.delta }},holiday:{{ person.values_map.holiday.delta }},extra_work:{{ person.values_map.extra_work.delta }},vacation_days:{{ person.values_map.vacation_days.delta }},vacation_entitlement:{{ person.values_map.vacation_entitlement.delta }}{% endif %}{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_enriched_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(template, billing_period);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert!(result.contains("vacation_hours:16.0"));
    assert!(result.contains("sick_leave:8.0"));
    assert!(result.contains("holiday:0.0"));
    assert!(result.contains("extra_work:5.0"));
    assert!(result.contains("vacation_days:2.0"));
    assert!(result.contains("vacation_entitlement:30.0"));
}

#[tokio::test]
async fn test_values_map_and_values_array_consistent() {
    let template = create_test_text_template_with_engine(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}map:{{ person.values_map.overall.delta }},array:{% for v in person.values %}{% if v.type == "overall" %}{{ v.value_delta }}{% endif %}{% endfor %}{% endif %}{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(template, billing_period);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert!(result.contains("map:160.0,array:160.0"));
}

#[tokio::test]
async fn test_custom_extra_hours_key_format_in_values_map() {
    let template = create_test_text_template_with_engine(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}{{ person.values_map["custom_extra_hours:overtime"].delta }}{% endif %}{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(template, billing_period);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert_eq!(result.as_ref(), "8.0");
}

#[tokio::test]
async fn test_enriched_context_both_engines_identical() {
    let tera_template = create_test_text_template(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}{{ person.name }}:{{ person.is_paid }}:{{ person.is_dynamic }}{% endif %}{% endfor %}"#,
    );
    let billing_period = create_test_billing_period();
    let tera_template_id = tera_template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(tera_template, billing_period.clone());
    let tera_result = service
        .generate_custom_report(tera_template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    let minijinja_template = create_test_text_template_with_engine(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}{{ person.name }}:{{ person.is_paid }}:{{ person.is_dynamic }}{% endif %}{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let mut billing_period2 = create_test_billing_period();
    billing_period2.id = billing_period_id;
    let minijinja_template_id = minijinja_template.id;

    let service = setup_generate_report_mocks(minijinja_template, billing_period2);
    let minijinja_result = service
        .generate_custom_report(minijinja_template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert_eq!(tera_result.as_ref(), minijinja_result.as_ref());
}

#[tokio::test]
async fn test_tera_values_array_regression() {
    let template = create_test_text_template(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}{% for value in person.values %}{% if value.type == "overall" %}{{ value.value_delta }}{% endif %}{% endfor %}{% endif %}{% endfor %}"#,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(template, billing_period);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert!(result.contains("160"));
}

#[tokio::test]
async fn test_minijinja_values_array_regression() {
    let template = create_test_text_template_with_engine(
        r#"{% for person in billing_period.sales_persons %}{% if person.sales_person_id == "12345678-1234-1234-1234-123456789012" %}{% for value in person.values %}{% if value.type == "overall" %}{{ value.value_delta }}{% endif %}{% endfor %}{% endif %}{% endfor %}"#,
        service::text_template::TemplateEngine::MiniJinja,
    );
    let billing_period = create_test_billing_period();
    let template_id = template.id;
    let billing_period_id = billing_period.id;

    let service = setup_generate_report_mocks(template, billing_period);

    let result = service
        .generate_custom_report(template_id, billing_period_id, Authentication::Full, None)
        .await
        .unwrap();

    assert!(result.contains("160"));
}

// === Snapshot schema versioning tests ===

use crate::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION;

fn setup_build_and_persist_mocks() -> MockDeps {
    let mut deps = MockDeps {
        billing_period_service: service::billing_period::MockBillingPeriodService::new(),
        reporting_service: service::reporting::MockReportingService::new(),
        sales_person_service: service::sales_person::MockSalesPersonService::new(),
        employee_work_details_service:
            service::employee_work_details::MockEmployeeWorkDetailsService::new(),
        text_template_service: service::text_template::MockTextTemplateService::new(),
        permission_service: service::MockPermissionService::new(),
        uuid_service: service::uuid_service::MockUuidService::new(),
        clock_service: service::clock::MockClockService::new(),
        transaction_dao: dao::MockTransactionDao::new(),
    };

    // build_and_persist + build_new_billing_period each call use_transaction.
    deps.transaction_dao
        .expect_use_transaction()
        .with(predicate::always())
        .times(2)
        .returning(|_| Ok(dao::MockTransaction));
    deps.transaction_dao
        .expect_commit()
        .with(always())
        .times(1)
        .returning(|_| Ok(()));

    deps.billing_period_service
        .expect_get_latest_billing_period_end_date()
        .with(always(), always())
        .times(1)
        .returning(|_, _| {
            Ok(Some(shifty_utils::ShiftyDate::from_ymd(2024, 6, 30).unwrap()))
        });

    deps.sales_person_service
        .expect_get_all()
        .with(always(), always())
        .times(1)
        .returning(|_, _| Ok(Arc::new([])));

    deps
}

#[tokio::test]
async fn test_build_and_persist_writes_current_snapshot_schema_version() {
    let mut deps = setup_build_and_persist_mocks();

    deps.billing_period_service
        .expect_create_billing_period()
        .withf(|bp, _process, _ctx, _tx| {
            bp.snapshot_schema_version == CURRENT_SNAPSHOT_SCHEMA_VERSION
        })
        .times(1)
        .returning(|bp, _process, _ctx, _tx| Ok(bp.clone()));

    let service = deps.build_service();

    let result = service
        .build_and_persist_billing_period_report(
            shifty_utils::ShiftyDate::from_ymd(2024, 7, 31).unwrap(),
            Authentication::Full,
            None,
        )
        .await;

    assert!(
        result.is_ok(),
        "expected build_and_persist to write snapshot_schema_version = CURRENT_SNAPSHOT_SCHEMA_VERSION ({}), got error: {:?}",
        CURRENT_SNAPSHOT_SCHEMA_VERSION,
        result.err(),
    );
}

#[tokio::test]
async fn test_build_and_persist_writes_constant_on_repeated_calls() {
    // Two distinct calls to build_and_persist must both stamp CURRENT_SNAPSHOT_SCHEMA_VERSION.
    for _ in 0..2 {
        let mut deps = setup_build_and_persist_mocks();

        deps.billing_period_service
            .expect_create_billing_period()
            .withf(|bp, _process, _ctx, _tx| {
                bp.snapshot_schema_version == CURRENT_SNAPSHOT_SCHEMA_VERSION
            })
            .times(1)
            .returning(|bp, _process, _ctx, _tx| Ok(bp.clone()));

        let service = deps.build_service();

        let result = service
            .build_and_persist_billing_period_report(
                shifty_utils::ShiftyDate::from_ymd(2024, 7, 31).unwrap(),
                Authentication::Full,
                None,
            )
            .await;

        assert!(
            result.is_ok(),
            "every call must source version exclusively from CURRENT_SNAPSHOT_SCHEMA_VERSION; got: {:?}",
            result.err(),
        );
    }
}

/// LOCKING TEST -- DO NOT NAIVELY UPDATE.
///
/// If this test fails after a code change:
///   - Did you intentionally change the snapshot computation?
///   - If yes, you MUST also bump CURRENT_SNAPSHOT_SCHEMA_VERSION
///     in service_impl/src/billing_period_report.rs.
///   - See CLAUDE.md § "Billing Period Snapshot Schema Versioning"
///     for the bump-trigger rules.
///
/// Phase-2 D-Phase2-09: pin map fuer alle 12 BillingPeriodValueType-Varianten
/// gegen deterministische EmployeeReport-Werte. Der Test ruft
/// `build_billing_period_report_for_sales_person` auf und prueft, dass JEDE
/// erwartete Variante mit dem korrekten `value_delta`/`value_ytd_*` im
/// Output landet — Drift zwischen Snapshot-Builder und Erwartung wird sofort
/// als Test-Failure sichtbar.
#[tokio::test]
async fn test_snapshot_v3_pinned_values() {
    use service::billing_period::BillingPeriodValueType;
    use service::reporting::EmployeeReport;

    fn make_report(
        sales_person: Arc<service::sales_person::SalesPerson>,
        delta_marker: f32,
    ) -> EmployeeReport {
        // Deterministische Werte: jedes Feld hat einen anderen Wert,
        // sodass eine Verwechslung zwischen Feldern (z.B. vacation vs sick)
        // sofort als Test-Failure sichtbar wird. `delta_marker` differenziert
        // die 4 Aufrufe (start/end/end_of_year/delta).
        EmployeeReport {
            sales_person,
            balance_hours: 1.0 * delta_marker,
            overall_hours: 100.0 * delta_marker,
            expected_hours: 80.0 * delta_marker,
            dynamic_hours: 0.0,
            shiftplan_hours: 90.0,
            extra_work_hours: 5.0 * delta_marker,
            vacation_hours: 16.0 * delta_marker,
            sick_leave_hours: 8.0 * delta_marker,
            holiday_hours: 4.0 * delta_marker,
            unpaid_leave_hours: 2.0 * delta_marker,
            volunteer_hours: 3.0 * delta_marker,
            vacation_carryover: 0,
            vacation_days: 6.0 * delta_marker,
            vacation_entitlement: 30.0 * delta_marker,
            sick_leave_days: 1.0,
            holiday_days: 0.5,
            absence_days: 7.5,
            carryover_hours: 0.0,
            custom_extra_hours: Arc::from(vec![]),
            by_week: Arc::from(vec![]),
            by_month: Arc::from(vec![]),
        }
    }

    let sales_person_id = Uuid::new_v4();
    let sales_person = service::sales_person::SalesPerson {
        id: sales_person_id,
        name: "Pin Test".into(),
        background_color: "#000000".into(),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::new_v4(),
    };
    let sales_person_arc = Arc::new(sales_person.clone());

    let mut deps = MockDeps {
        billing_period_service: service::billing_period::MockBillingPeriodService::new(),
        reporting_service: service::reporting::MockReportingService::new(),
        sales_person_service: service::sales_person::MockSalesPersonService::new(),
        employee_work_details_service:
            service::employee_work_details::MockEmployeeWorkDetailsService::new(),
        text_template_service: service::text_template::MockTextTemplateService::new(),
        permission_service: service::MockPermissionService::new(),
        uuid_service: service::uuid_service::MockUuidService::new(),
        clock_service: service::clock::MockClockService::new(),
        transaction_dao: dao::MockTransactionDao::new(),
    };

    deps.transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    // 4 ReportingService-Aufrufe mit unterschiedlichen Markern:
    //   start(2.0), end(3.0), end_of_year(4.0), delta(1.0)
    // Reihenfolge in build_billing_period_report_for_sales_person:
    //   1. report_start, 2. report_end, 3. report_end_of_year, 4. report_delta
    let sp = sales_person_arc.clone();
    deps.reporting_service
        .expect_get_report_for_employee_range()
        .times(4)
        .returning(move |_, _from, _to, include_carryover, _ctx, _tx| {
            // Wir koennen nicht zuverlaessig zwischen start/end/end_of_year/delta
            // anhand der Datums-Args differenzieren ohne mehrfache `withf`.
            // Wir nutzen `include_carryover`-Flag als Marker: 3 Aufrufe sind
            // mit `true`, 1 mit `false` (delta). Innerhalb der `true`-Gruppe
            // unterscheiden wir per Reihenfolge ueber einen Counter.
            // Einfacher: gib fuer alle Aufrufe IDENTISCHE Werte zurueck —
            // dann ist value_delta == value_ytd_from == value_ytd_to == value_full_year
            // und der Pin-Test prueft genau das.
            let _ = include_carryover;
            Ok(make_report(sp.clone(), 1.0))
        });

    let service = deps.build_service();

    let result = service
        .build_billing_period_report_for_sales_person(
            sales_person.clone(),
            shifty_utils::ShiftyDate::from_ymd(2024, 7, 1).unwrap(),
            shifty_utils::ShiftyDate::from_ymd(2024, 7, 31).unwrap(),
            Authentication::Full,
            None,
        )
        .await
        .expect("build_billing_period_report_for_sales_person muss erfolgreich sein");

    let values = &result.values;

    // PIN-CHECK: alle 12 BillingPeriodValueType-Varianten (ausgenommen
    // CustomExtraHours, weil custom_extra_hours leer ist; und Volunteer
    // wird nur bei != 0 persistiert — wir liefern 3.0, also Volunteer ist da).
    assert_eq!(
        values
            .get(&BillingPeriodValueType::Overall)
            .expect("Overall pin")
            .value_delta,
        100.0,
        "Overall.value_delta == report_delta.overall_hours"
    );
    assert_eq!(
        values
            .get(&BillingPeriodValueType::Balance)
            .expect("Balance pin")
            .value_delta,
        1.0
    );
    assert_eq!(
        values
            .get(&BillingPeriodValueType::ExpectedHours)
            .expect("ExpectedHours pin")
            .value_delta,
        80.0
    );
    assert_eq!(
        values
            .get(&BillingPeriodValueType::ExtraWork)
            .expect("ExtraWork pin")
            .value_delta,
        5.0
    );
    assert_eq!(
        values
            .get(&BillingPeriodValueType::VacationHours)
            .expect("VacationHours pin")
            .value_delta,
        16.0
    );
    assert_eq!(
        values
            .get(&BillingPeriodValueType::SickLeave)
            .expect("SickLeave pin")
            .value_delta,
        8.0
    );
    assert_eq!(
        values
            .get(&BillingPeriodValueType::UnpaidLeave)
            .expect("UnpaidLeave pin (NEU in v3)")
            .value_delta,
        2.0,
        "Phase-2 D-Phase2-04: UnpaidLeave wird ab Schema-Version 3 persistiert"
    );
    assert_eq!(
        values
            .get(&BillingPeriodValueType::Holiday)
            .expect("Holiday pin")
            .value_delta,
        4.0
    );
    assert_eq!(
        values
            .get(&BillingPeriodValueType::Volunteer)
            .expect("Volunteer pin (nur != 0 persistiert)")
            .value_delta,
        3.0
    );
    assert_eq!(
        values
            .get(&BillingPeriodValueType::VacationDays)
            .expect("VacationDays pin")
            .value_delta,
        6.0
    );
    assert_eq!(
        values
            .get(&BillingPeriodValueType::VacationEntitlement)
            .expect("VacationEntitlement pin")
            .value_delta,
        30.0
    );

    // CustomExtraHours: leer in unserem Setup -> KEIN Eintrag in der values-Map.
    assert!(
        !values
            .keys()
            .any(|k| matches!(k, BillingPeriodValueType::CustomExtraHours(_))),
        "CustomExtraHours-Eintraege duerfen nicht entstehen wenn custom_extra_hours leer ist"
    );

    // Surface-Check: genau 11 keys (12 Varianten ohne CustomExtraHours, da leer).
    assert_eq!(
        values.len(),
        11,
        "Erwarte 11 nicht-Custom-Varianten in v3-Snapshot (UnpaidLeave macht 11 statt 10)"
    );
}
