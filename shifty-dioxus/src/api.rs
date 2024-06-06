use std::sync::Arc;

use dioxus::prelude::*;
use tracing::info;

use crate::{state::Config, AuthInfo};

pub async fn fetch_auth_info(backend_url: Arc<str>) -> Result<AuthInfo, reqwest::Error> {
    info!("Fetching username");
    let res = reqwest::get(format!("{}/auth-info", backend_url))
        .await?
        .json()
        .await?;
    info!("Fetched");
    Ok(res)
}

pub async fn load_config() -> Result<Config, reqwest::Error> {
    info!("Loading config.json");
    let url = web_sys::window()
        .expect("no window")
        .location()
        .href()
        .expect("no href");
    info!("URL: {url}");
    let res = reqwest::get(format!("{}/config.json", url))
        .await?
        .json()
        .await?;
    info!("Loaded");
    Ok(res)
}
