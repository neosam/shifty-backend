use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{
    api,
    service::error::{ErrorStore, ERROR_STORE},
    state::Config,
};

use super::auth;

pub async fn load_config() {
    match api::load_config().await {
        Ok(config) => {
            *CONFIG.write() = config;
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
        }
    }
    auth::load_auth_info().await;
}

// Config service
pub static CONFIG: GlobalSignal<Config> = Signal::global(|| Config::default());
#[allow(dead_code)]
pub enum ConfigAction {
    LoadConfig,
}
pub async fn config_service(mut rx: UnboundedReceiver<ConfigAction>) {
    load_config().await;

    while let Some(action) = rx.next().await {
        match action {
            ConfigAction::LoadConfig => {
                load_config().await;
            }
        }
    }
}
