use std::{env, sync::Arc};

use async_trait::async_trait;
use service::{
    config::{Config, ConfigService},
    ServiceError,
};

pub struct ConfigServiceImpl;

#[async_trait]
impl ConfigService for ConfigServiceImpl {
    async fn get_config(&self) -> Result<Config, ServiceError> {
        let timezone = env::var("TIMEZONE").unwrap_or("UTC".to_string());
        let ical_label = env::var("ICAL_LABEL").unwrap_or("Schicht".to_string());

        Ok(Config {
            timezone: Arc::from(timezone),
            ical_label: Arc::from(ical_label),
        })
    }
}
