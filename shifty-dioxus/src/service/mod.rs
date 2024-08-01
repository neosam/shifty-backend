use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{
    api,
    error::ShiftyError,
    i18n::{self, I18n, Locale},
    state::Config,
};

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

pub static I18N: GlobalSignal<I18n<i18n::Key, i18n::Locale>> =
    Signal::global(|| i18n::generate(i18n::Locale::En));

pub async fn i18n_service(mut rx: UnboundedReceiver<()>) {
    let set_browser_language = || async {
        let language = web_sys::window()
            .map(|w| w.navigator())
            .and_then(|n| n.language())
            .map(|locale| locale[..2].to_string())
            .unwrap_or_else(|| "en".to_string());
        let i18n = i18n::generate(i18n::Locale::from_str(&language));
        *I18N.write() = i18n;
    };

    set_browser_language().await;
}
