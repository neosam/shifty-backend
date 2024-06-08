#![allow(non_snake_case)]

use std::{rc::Rc, sync::Arc};

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{info, Level};

mod api;
mod app;
mod auth;
mod blog;
mod home;
mod i18n;
mod not_authenticated;
mod router;
mod shiftplan;
mod state;
mod top_bar;
mod week_view;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(app::App);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    pub user: Arc<str>,
    pub privileges: Arc<[Arc<str>]>,
    #[serde(default)]
    pub authenticated: bool,
}

impl Default for AuthInfo {
    fn default() -> Self {
        Self {
            user: "".into(),
            privileges: Arc::new([]),
            authenticated: false,
        }
    }
}

impl AuthInfo {
    pub fn has_privilege(&self, privilege: &str) -> bool {
        self.privileges.iter().any(|p| p.as_ref() == privilege)
    }
}
