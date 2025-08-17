use std::sync::Arc;

use mockall::predicate::{self, *};
use service::block::Block;
use service::block_report::BlockReportService;
use service::permission::Authentication;
use service::text_template::TextTemplate;
use service::ServiceError;
use time::macros::datetime;
use uuid::Uuid;
use shifty_utils::DayOfWeek;

use crate::block_report::{BlockReportServiceImpl, BlockReportServiceDeps};

struct MockDeps {
    block_service: service::block::MockBlockService,
    text_template_service: service::text_template::MockTextTemplateService,
    permission_service: service::MockPermissionService,
    clock_service: service::clock::MockClockService,
    transaction_dao: dao::MockTransactionDao,
}

impl BlockReportServiceDeps for MockDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type BlockService = service::block::MockBlockService;
    type TextTemplateService = service::text_template::MockTextTemplateService;
    type PermissionService = service::MockPermissionService;
    type ClockService = service::clock::MockClockService;
    type TransactionDao = dao::MockTransactionDao;
}

impl MockDeps {
    fn build_service(self) -> BlockReportServiceImpl<MockDeps> {
        BlockReportServiceImpl {
            block_service: self.block_service.into(),
            text_template_service: self.text_template_service.into(),
            permission_service: self.permission_service.into(),
            clock_service: self.clock_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

fn create_test_text_template(template_text: &str) -> TextTemplate {
    TextTemplate {
        id: Uuid::new_v4(),
        name: Some("Test Block Template".into()),
        template_type: "block_report".into(),
        template_text: template_text.into(),
        created_at: Some(datetime!(2024-01-01 10:00:00)),
        created_by: Some("test_user".into()),
        deleted: None,
        deleted_by: None,
        version: Uuid::new_v4(),
    }
}

fn create_test_blocks() -> Vec<Block> {
    vec![
        Block {
            year: 2024,
            week: 33,
            sales_person: None,
            day_of_week: DayOfWeek::Monday,
            from: time::Time::from_hms(14, 0, 0).unwrap(),  // Future time (after mocked 12:00)
            to: time::Time::from_hms(17, 0, 0).unwrap(),
            bookings: Arc::new([]),
            slots: Arc::new([]),
        },
        Block {
            year: 2024,
            week: 34,
            sales_person: None,
            day_of_week: DayOfWeek::Tuesday,
            from: time::Time::from_hms(14, 0, 0).unwrap(),
            to: time::Time::from_hms(17, 0, 0).unwrap(),
            bookings: Arc::new([]),
            slots: Arc::new([]),
        },
    ]
}

#[tokio::test]
async fn test_generate_block_report_success() {
    let template_id = Uuid::new_v4();
    let context = Authentication::Full;
    
    let template = create_test_text_template(
        "Block Report:\\n{% for block in current_week_blocks %}Week {{ block.week }}: {{ block.day_of_week }} {{ block.from }}-{{ block.to }}\\n{% endfor %}"
    );
    
    let test_blocks = create_test_blocks();

    // Setup mocks
    let mut deps = MockDeps {
        block_service: service::block::MockBlockService::new(),
        text_template_service: service::text_template::MockTextTemplateService::new(),
        permission_service: service::MockPermissionService::new(),
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

    // Set the mock date to early in week 33 so the test blocks are in the future
    deps.clock_service
        .expect_date_now()
        .times(1)
        .returning(|| time::Date::from_calendar_date(2024, time::Month::August, 12).unwrap()); // Monday of week 33

    deps.clock_service
        .expect_date_time_now()
        .times(1)
        .returning(|| time::Date::from_calendar_date(2024, time::Month::August, 12).unwrap().with_time(time::Time::from_hms(9, 0, 0).unwrap())); // Early morning

    // Mock get_unsufficiently_booked_blocks for three weeks
    deps.block_service
        .expect_get_unsufficiently_booked_blocks()
        .with(eq(2024u32), eq(33u8), eq(context.clone()), always())
        .times(1)
        .returning({
            let blocks = test_blocks.clone();
            move |_, _, _, _| Ok(blocks.iter().filter(|b| b.week == 33).cloned().collect())
        });

    deps.block_service
        .expect_get_unsufficiently_booked_blocks()
        .with(eq(2024u32), eq(34u8), eq(context.clone()), always())
        .times(1)
        .returning({
            let blocks = test_blocks.clone();
            move |_, _, _, _| Ok(blocks.iter().filter(|b| b.week == 34).cloned().collect())
        });

    deps.block_service
        .expect_get_unsufficiently_booked_blocks()
        .with(eq(2024u32), eq(35u8), eq(context.clone()), always())
        .times(1)
        .returning(|_, _, _, _| Ok(Arc::new([])));

    deps.transaction_dao
        .expect_commit()
        .with(always())
        .times(1)
        .returning(|_| Ok(()));

    let service = deps.build_service();

    // Execute
    let result = service
        .generate_block_report(template_id, context, None)
        .await;

    // Assert
    assert!(result.is_ok());
    let report = result.unwrap();
    println!("Generated report: {}", report);
    assert!(report.contains("Block Report:"));
    assert!(report.contains("Week 33: Monday 14:00:00.0-17:00:00.0"));
}

#[tokio::test]
async fn test_generate_block_report_no_hr_permission() {
    let template_id = Uuid::new_v4();
    let context = Authentication::Full;

    // Setup mocks
    let mut deps = MockDeps {
        block_service: service::block::MockBlockService::new(),
        text_template_service: service::text_template::MockTextTemplateService::new(),
        permission_service: service::MockPermissionService::new(),
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
        .generate_block_report(template_id, context, None)
        .await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ServiceError::Forbidden));
}

#[tokio::test]
async fn test_generate_block_report_template_not_found() {
    let template_id = Uuid::new_v4();
    let context = Authentication::Full;

    // Setup mocks
    let mut deps = MockDeps {
        block_service: service::block::MockBlockService::new(),
        text_template_service: service::text_template::MockTextTemplateService::new(),
        permission_service: service::MockPermissionService::new(),
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
        .generate_block_report(template_id, context, None)
        .await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ServiceError::EntityNotFoundGeneric(_)));
}