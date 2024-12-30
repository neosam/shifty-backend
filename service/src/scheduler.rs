use crate::ServiceError;
use async_trait::async_trait;
use mockall::automock;
use std::fmt::Debug;

#[automock(type Context=();)]
#[async_trait]
pub trait SchedulerService {
    /// The type of the authentication context your scheduler might need to pass
    /// to other services when invoking them.
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

    /// Start the scheduler in a background task.
    /// After calling this, scheduled jobs (added via other methods) will run automatically.
    async fn start(&self) -> Result<(), ServiceError>;

    /// Schedules a periodic job that updates carryover for the previous year.
    /// The `cron` parameter is a cron expression (e.g. `"0 * * * * *"` to run hourly).
    async fn schedule_carryover_updates(&self, cron: &'static str) -> Result<(), ServiceError>;
}
