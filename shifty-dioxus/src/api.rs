use std::rc::Rc;

use rest_types::{BookingTO, SalesPersonTO, SlotTO};
use tracing::info;
use uuid::Uuid;

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

pub async fn get_slots(config: Config) -> Result<Rc<[SlotTO]>, reqwest::Error> {
    info!("Fetching slots");
    let url = format!("{}/slot", config.backend);
    let response = reqwest::get(url).await?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn get_bookings_for_week(
    config: Config,
    week: u8,
    year: u32,
) -> Result<Rc<[BookingTO]>, reqwest::Error> {
    info!("Fetching bookings for week {week} in year {year}");
    let url = format!("{}/booking/week/{year}/{week}", config.backend);
    let response = reqwest::get(url).await?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn add_booking(
    config: Config,
    sales_person_id: Uuid,
    slot_id: Uuid,
    week: u8,
    year: u32,
) -> Result<(), reqwest::Error> {
    info!(
        "Adding booking for user {sales_person_id} to slot {slot_id} in week {week} of year {year}"
    );
    let url: String = format!("{}/booking", config.backend,);
    let booking_to = BookingTO {
        id: Uuid::nil(),
        sales_person_id,
        slot_id,
        calendar_week: week as i32,
        year,
        created: None,
        deleted: None,
        version: Uuid::nil(),
    };
    let client = reqwest::Client::new();
    let response = client.post(url).json(&booking_to).send().await?;
    response.error_for_status_ref()?;
    info!("Added");
    Ok(())
}

pub async fn remove_booking(config: Config, booking_id: Uuid) -> Result<(), reqwest::Error> {
    info!("Removing booking {booking_id}");
    let url = format!("{}/booking/{booking_id}", config.backend,);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Removed");
    Ok(())
}

pub async fn copy_week(
    config: Config,
    from_week: u8,
    from_year: u32,
    to_week: u8,
    to_year: u32,
) -> Result<(), reqwest::Error> {
    info!("Copying week {from_week} of year {from_year} to week {to_week} of year {to_year}");
    let url = format!("{}/booking/copy?from_year={from_year}&from_week={from_week}&to_year={to_year}&to_week={to_week}", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).send().await?;
    response.error_for_status_ref()?;
    info!("Copied");
    Ok(())
}

pub async fn get_sales_persons(config: Config) -> Result<Rc<[SalesPersonTO]>, reqwest::Error> {
    info!("Fetching sales persons");
    let url = format!("{}/sales-person", config.backend);
    let response = reqwest::get(url).await?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn get_current_sales_person(
    config: Config,
) -> Result<Option<SalesPersonTO>, reqwest::Error> {
    info!("Fetching current sales person");
    let url = format!("{}/sales-person/current", config.backend);
    let response = reqwest::get(url).await?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}
