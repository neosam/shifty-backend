use std::rc::Rc;

use rest_types::SlotTO;
use tracing::info;

use crate::state::{AuthInfo, Config};

pub async fn fetch_auth_info(backend_url: Rc<str>) -> Result<Option<AuthInfo>, reqwest::Error> {
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

pub async fn get_slots() -> Result<Rc<[SlotTO]>, reqwest::Error> {
    info!("Fetching slots");
    let response = reqwest::get("http://localhost:8080/slot").await?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}
