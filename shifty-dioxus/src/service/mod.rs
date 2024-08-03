use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use js_sys::WebAssembly::Global;

use crate::{
    api,
    error::ShiftyError,
    i18n::{self, I18n, Locale},
    state::{
        dropdown::{Dropdown, DropdownEntry},
        AuthInfo, Config,
    },
};

pub async fn load_auth_info() {
    if CONFIG.read().backend.is_empty() {
        return;
    }
    let auth_info = api::fetch_auth_info(CONFIG.read().backend.clone()).await;

    match auth_info {
        Ok(Some(auth_info)) => {
            *AUTH.write() = AuthStore {
                auth_info: Some(auth_info),
                loading_done: true,
            };
        }
        Ok(None) => {
            *AUTH.write() = AuthStore {
                auth_info: None,
                loading_done: true,
            };
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
            *AUTH.write() = AuthStore {
                auth_info: None,
                loading_done: true,
            };
        }
        _ => {}
    }
}
pub async fn load_config() {
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
    load_auth_info().await;
}

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

pub static DROPDOWN: GlobalSignal<Option<Dropdown>> = Signal::global(|| None);

pub enum DropdownAction {
    OpenDropdown(f64, f64, Rc<[DropdownEntry]>),
    CloseDropdown,
    ToggleDropdown(f64, f64, Rc<[DropdownEntry]>),
}

pub async fn open_dropdown(x: f64, y: f64, entries: Rc<[DropdownEntry]>) {
    *DROPDOWN.write() = Some(Dropdown { x, y, entries });
}
pub async fn close_dropdown() {
    *DROPDOWN.write() = None;
}
pub async fn toggle_dropdown(x: f64, y: f64, entries: Rc<[DropdownEntry]>) {
    if DROPDOWN.read().is_some() {
        close_dropdown().await;
    } else {
        open_dropdown(x, y, entries).await;
    }
}

pub async fn dropdown_service(mut rx: UnboundedReceiver<DropdownAction>) {
    while let Some(action) = rx.next().await {
        match action {
            DropdownAction::OpenDropdown(x, y, entries) => open_dropdown(x, y, entries).await,
            DropdownAction::CloseDropdown => close_dropdown().await,
            DropdownAction::ToggleDropdown(x, y, entries) => toggle_dropdown(x, y, entries).await,
        }
    }
}

// Config service
pub static CONFIG: GlobalSignal<Config> = Signal::global(|| Config::default());
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

#[derive(Default, Clone, Eq, PartialEq)]
pub struct AuthStore {
    pub auth_info: Option<AuthInfo>,
    pub loading_done: bool,
}

pub static AUTH: GlobalSignal<AuthStore> = Signal::global(|| AuthStore::default());

pub async fn auth_service(mut rx: UnboundedReceiver<()>) {
    load_auth_info().await;
}
