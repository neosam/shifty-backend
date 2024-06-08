use std::sync::Arc;

use dioxus::prelude::*;
use tracing::info;

use crate::{state::Config, AuthInfo};

pub async fn fetch_auth_info(backend_url: Arc<str>) -> Result<Option<AuthInfo>, reqwest::Error> {
    info!("Fetching username");
    let response = reqwest::get(format!("{}/auth-info", backend_url)).await?;
    if response.status() != 200 {
        return Ok(None);
    }
    let mut res: AuthInfo = response.json().await?;
    res.authenticated = true;
    info!("Fetched");
    Ok(Some(res))
}

pub async fn load_config() -> Result<Config, reqwest::Error> {
    info!("Loading config.json");
    //let url = web_sys::window()
    //    .expect("no window")
    //    .location()
    //    .href()
    //    .expect("no href");
    let protocol = web_sys::window()
        .expect("no window")
        .location()
        .protocol()
        .expect("no protocol");
    let host = web_sys::window()
        .expect("no window")
        .location()
        .host()
        .expect("no host");
    let url = format!("{protocol}//{host}/config.json");
    info!("URL: {url}");
    let res = reqwest::get(url).await?.json().await?;
    info!("Loaded");
    Ok(res)
}
