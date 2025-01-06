use crate::{api, state::AuthInfo};
use dioxus::prelude::*;

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
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
    }
}

#[derive(Default, Clone, Eq, PartialEq)]
pub struct AuthStore {
    pub auth_info: Option<AuthInfo>,
    pub loading_done: bool,
}

pub static AUTH: GlobalSignal<AuthStore> = Signal::global(|| AuthStore::default());

#[allow(dead_code)]
pub async fn auth_service(_rx: UnboundedReceiver<()>) {
    load_auth_info().await;
}
