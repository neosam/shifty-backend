#![allow(non_snake_case)]

use std::rc::Rc;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::Level;

mod api;
mod app;
mod auth;
mod component;
mod error;
mod i18n;
mod js;
mod loader;
mod page;
mod router;
mod service;
mod state;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(app::App);
}
