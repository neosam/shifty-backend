use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

/// A service for generating block-based reports using text templates.
/// This service provides template context with current week blocks, next two weeks blocks,
/// current user blocks, and unsufficiently booked blocks.
#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait BlockReportService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Generates a custom block report using the specified template.
    /// The template context includes:
    /// - current_week_blocks: All blocks for the current week
    /// - next_week_blocks: All blocks for next week  
    /// - week_after_next_blocks: All blocks for the week after next
    /// - current_user_blocks: Blocks for the authenticated user across all three weeks
    /// - unsufficiently_booked_blocks: Blocks that are not sufficiently booked across all three weeks
    /// - current_week: Current week number
    /// - current_year: Current year
    async fn generate_block_report(
        &self,
        template_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<str>, ServiceError>;
}