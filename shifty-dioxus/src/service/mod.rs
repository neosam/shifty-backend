use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{api, error::ShiftyError, state::Config};

#[derive(Default, Debug)]
pub struct ErrorStore {
    pub error: Option<ShiftyError>,
}
pub static ERROR_STORE: GlobalSignal<ErrorStore> = Signal::global(|| ErrorStore::default());

pub enum ErrorAction {
    SetError(ShiftyError),
}

pub async fn error_service(mut rx: UnboundedReceiver<ErrorAction>) {
    while let Some(action) = rx.next().await {
        match action {
            ErrorAction::SetError(error) => {
                *ERROR_STORE.write() = ErrorStore { error: Some(error) };
            }
        }
    }
}

pub struct Dropdown;

pub static DROPDOWN: GlobalSignal<Dropdown> = Signal::global(|| Dropdown);

pub enum DropdownAction {}

pub async fn dropdown_service(rx: UnboundedReceiver<DropdownAction>) {}

// Config service
pub static CONFIG: GlobalSignal<Config> = Signal::global(|| Config::default());
pub enum ConfigAction {
    LoadConfig,
}
pub async fn config_service(mut rx: UnboundedReceiver<ConfigAction>) {
    let load_config = || async {
        let config = api::load_config().await;
        match config {
            Ok(config) => {
                *CONFIG.write() = config;
            }
            Err(err) => {
                *ERROR_STORE.write() = ErrorStore {
                    error: Some(err.into()),
                };
            }
        }
        *CONFIG.write() = api::load_config().await.unwrap();
    };

    load_config().await;

    while let Some(action) = rx.next().await {
        match action {
            ConfigAction::LoadConfig => {
                load_config().await;
            }
        }
    }
}
