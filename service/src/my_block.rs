use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use shifty_utils::ShiftyWeek;

use crate::block::Block;
use crate::permission::Authentication;
use crate::ServiceError;

/// A service trait for retrieving blocks for the currently logged-in user.
///
/// This service extracts the sales person ID from the authentication context
/// and retrieves blocks across a range of weeks, ordering them by date and time.
#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait MyBlockService {
    /// Context type for authentication.
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    /// Transaction type from your DAO layer.
    type Transaction: dao::Transaction;

    /// Returns all blocks for the currently logged-in user within the specified week range.
    ///
    /// The blocks are ordered by date (year, week, day of week) and then by start time.
    /// If the logged-in user is not associated with a sales person, returns an empty list.
    ///
    /// # Arguments
    /// * `from` - The start week (inclusive)
    /// * `until` - The end week (inclusive)
    /// * `context` - Authentication context to identify the current user
    /// * `tx` - Optional transaction for database operations
    async fn get_my_blocks(
        &self,
        from: ShiftyWeek,
        until: ShiftyWeek,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Block]>, ServiceError>;
}
