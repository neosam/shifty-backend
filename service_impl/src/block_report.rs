use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use tera::{Context, Tera};
use uuid::Uuid;

use dao::TransactionDao;
use service::block::{Block, BlockService};
use service::clock::ClockService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::text_template::TextTemplateService;
use service::PermissionService;
use service::{block_report::BlockReportService, ServiceError};

use crate::gen_service_impl;

#[derive(Serialize)]
struct SimpleBlock {
    year: u32,
    week: u8,
    sales_person_name: Option<String>,
    day_of_week: String,
    from: String,
    to: String,
    date: String,
}

impl From<&Block> for SimpleBlock {
    fn from(block: &Block) -> Self {
        Self {
            year: block.year,
            week: block.week,
            sales_person_name: block.sales_person.as_ref().map(|sp| sp.name.to_string()),
            day_of_week: format!("{:?}", block.day_of_week),
            from: block.from.to_string(),
            to: block.to.to_string(),
            date: block.date().map(|d| d.to_string()).unwrap_or_default(),
        }
    }
}

/// Checks if a block is in the future (has not ended yet)
fn is_block_in_future(block: &Block, current_datetime: time::PrimitiveDateTime) -> bool {
    match block.datetime_to() {
        Ok(block_end_datetime) => block_end_datetime > current_datetime,
        Err(_) => true, // If we can't determine the datetime, include the block by default
    }
}

gen_service_impl! {
    struct BlockReportServiceImpl: BlockReportService = BlockReportServiceDeps {
        BlockService: BlockService<Context = Self::Context, Transaction = Self::Transaction> = block_service,
        TextTemplateService: TextTemplateService<Context = Self::Context, Transaction = Self::Transaction> = text_template_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: BlockReportServiceDeps> BlockReportService for BlockReportServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn generate_block_report(
        &self,
        template_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<str>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Check HR permission
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        // Get the template
        let template = self
            .text_template_service
            .get_by_id(template_id, context.clone(), Some(tx.clone()))
            .await?;

        // Get current date and time for filtering
        let current_date = self.clock_service.date_now();
        let current_datetime = self.clock_service.date_time_now();
        let (current_year, current_week, _) = current_date.to_iso_week_date();
        let current_year = current_year as u32;
        let current_week = current_week;

        // Calculate next weeks
        let (next_year, next_week) = if current_week == 53 {
            (current_year + 1, 1)
        } else {
            (current_year, current_week + 1)
        };

        let (week_after_next_year, week_after_next_week) = if next_week == 53 {
            (next_year + 1, 1)
        } else {
            (next_year, next_week + 1)
        };

        // Collect all blocks for the three weeks
        let mut unsufficiently_booked_blocks = Vec::new();

        // Get unsufficiently booked blocks for all three weeks
        let current_week_unbooked = self
            .block_service
            .get_unsufficiently_booked_blocks(
                current_year,
                current_week,
                context.clone(),
                Some(tx.clone()),
            )
            .await?;
        let next_week_unbooked = self
            .block_service
            .get_unsufficiently_booked_blocks(
                next_year,
                next_week,
                context.clone(),
                Some(tx.clone()),
            )
            .await?;
        let week_after_next_unbooked = self
            .block_service
            .get_unsufficiently_booked_blocks(
                week_after_next_year,
                week_after_next_week,
                context.clone(),
                Some(tx.clone()),
            )
            .await?;

        // Filter to only include future blocks (not yet ended)
        unsufficiently_booked_blocks.extend(
            current_week_unbooked
                .iter()
                .filter(|block| is_block_in_future(block, current_datetime))
                .cloned(),
        );
        unsufficiently_booked_blocks.extend(
            next_week_unbooked
                .iter()
                .filter(|block| is_block_in_future(block, current_datetime))
                .cloned(),
        );
        unsufficiently_booked_blocks.extend(
            week_after_next_unbooked
                .iter()
                .filter(|block| is_block_in_future(block, current_datetime))
                .cloned(),
        );

        // Filter blocks by week and convert to SimpleBlock for template serialization
        let current_week_blocks: Vec<SimpleBlock> = unsufficiently_booked_blocks
            .iter()
            .filter(|b| b.year == current_year && b.week == current_week)
            .map(SimpleBlock::from)
            .collect();
        let next_week_blocks: Vec<SimpleBlock> = unsufficiently_booked_blocks
            .iter()
            .filter(|b| b.year == next_year && b.week == next_week)
            .map(SimpleBlock::from)
            .collect();
        let week_after_next_blocks: Vec<SimpleBlock> = unsufficiently_booked_blocks
            .iter()
            .filter(|b| b.year == week_after_next_year && b.week == week_after_next_week)
            .map(SimpleBlock::from)
            .collect();
        let all_simple_blocks: Vec<SimpleBlock> = unsufficiently_booked_blocks
            .iter()
            .map(SimpleBlock::from)
            .collect();

        // Create template context
        let mut tera_context = Context::new();
        tera_context.insert("current_week_blocks", &current_week_blocks);
        tera_context.insert("next_week_blocks", &next_week_blocks);
        tera_context.insert("week_after_next_blocks", &week_after_next_blocks);
        tera_context.insert("unsufficiently_booked_blocks", &all_simple_blocks);
        tera_context.insert("current_week", &current_week);
        tera_context.insert("current_year", &current_year);
        tera_context.insert("next_week", &next_week);
        tera_context.insert("next_year", &next_year);
        tera_context.insert("week_after_next_week", &week_after_next_week);
        tera_context.insert("week_after_next_year", &week_after_next_year);

        // Render the template
        let mut tera = Tera::default();
        let rendered = tera
            .render_str(&template.template_text, &tera_context)
            .map_err(|e| {
                ServiceError::ValidationError(Arc::new([
                    service::ValidationFailureItem::InvalidValue(Arc::from(format!(
                        "Template rendering error: {}",
                        e
                    ))),
                ]))
            })?;

        self.transaction_dao.commit(tx).await?;
        Ok(Arc::from(rendered))
    }
}
