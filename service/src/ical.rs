//! Service which provides iCalendar data.

use crate::ServiceError;
use mockall::automock;
use std::sync::Arc;

use crate::block::Block;

#[automock]
pub trait IcalService {
    fn convert_blocks_to_ical_string(
        &self,
        blocks: Arc<[Block]>,
        title: Arc<str>,
        timezone: Arc<str>,
    ) -> Result<Arc<str>, ServiceError>;
}
