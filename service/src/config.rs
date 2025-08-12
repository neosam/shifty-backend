use std::sync::Arc;

use crate::ServiceError;
use async_trait::async_trait;
use mockall::automock;

pub struct Config {
    pub timezone: Arc<str>,
    pub ical_label: Arc<str>,
}

#[automock]
#[async_trait]
pub trait ConfigService {
    async fn get_config(&self) -> Result<Config, ServiceError>;
}
