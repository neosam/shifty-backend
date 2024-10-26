use std::rc::Rc;

use rest_types::{
    BookingConflictTO, BookingTO, DayOfWeekTO, EmployeeReportTO, EmployeeWorkDetailsTO,
    ExtraHoursCategoryTO, ExtraHoursTO, RoleTO, SalesPersonTO, SalesPersonUnavailableTO,
    ShortEmployeeReportTO, SlotTO, SpecialDayTO, UserRole, UserTO, WeeklySummaryTO,
};
use tracing::info;
use uuid::Uuid;

use crate::{
    base_types::ImStr,
    error::ShiftyError,
    js,
    state::{AuthInfo, Config},
};

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
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res: Config = response.json().await?;
    info!("Loaded");
    Ok(res)
}

pub async fn get_slots(
    config: Config,
    year: u32,
    week: u8,
) -> Result<Rc<[SlotTO]>, reqwest::Error> {
    info!("Fetching slots");
    let url = format!("{}/slot/week/{year}/{week}", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
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
    response.error_for_status_ref()?;
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
    response.error_for_status_ref()?;
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
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn get_sales_person(
    config: Config,
    sales_person_id: Uuid,
) -> Result<SalesPersonTO, reqwest::Error> {
    info!("Fetching sales person {sales_person_id}");
    let url = format!("{}/sales-person/{sales_person_id}", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn put_sales_person(
    config: Config,
    sales_person: SalesPersonTO,
) -> Result<(), reqwest::Error> {
    info!("Posting sales person");
    let url = format!(
        "{}/sales-person/{}",
        config.backend,
        sales_person.id.to_string()
    );
    let client = reqwest::Client::new();
    let response = client.put(url).json(&sales_person).send().await?;
    response.error_for_status_ref()?;
    info!("Posted");
    Ok(())
}

pub async fn post_sales_person(
    config: Config,
    sales_person: SalesPersonTO,
) -> Result<(), reqwest::Error> {
    info!("Posting sales person");
    let url = format!("{}/sales-person", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&sales_person).send().await?;
    response.error_for_status_ref()?;
    info!("Posted");
    Ok(())
}

pub async fn get_user_for_sales_person(
    config: Config,
    sales_person_id: Uuid,
) -> Result<Option<Rc<str>>, reqwest::Error> {
    info!("Fetching user for sales person {sales_person_id}");
    let url = format!("{}/sales-person/{sales_person_id}/user", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn post_user_to_sales_person(
    config: Config,
    sales_person_id: Uuid,
    user_id: ImStr,
) -> Result<(), reqwest::Error> {
    info!("Posting user {user_id} to sales person {sales_person_id}");
    let url = format!("{}/sales-person/{sales_person_id}/user", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(user_id.as_str()).send().await?;
    response.error_for_status_ref()?;
    info!("Posted");
    Ok(())
}

pub async fn delete_user_from_sales_person(
    config: Config,
    sales_person_id: Uuid,
) -> Result<(), reqwest::Error> {
    info!("Deleting user from sales person {sales_person_id}");
    let url = format!("{}/sales-person/{sales_person_id}/user", config.backend);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Deleted");
    Ok(())
}

pub async fn get_short_reports(
    config: Config,
    year: u32,
    calendar_week: u8,
) -> Result<Rc<[ShortEmployeeReportTO]>, reqwest::Error> {
    info!("Fetching short reports");
    let url = format!(
        "{}/report?year={}&until_week={}",
        config.backend, year, calendar_week
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn get_employee_reports(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
    calendar_week: u8,
) -> Result<Rc<EmployeeReportTO>, reqwest::Error> {
    info!("Fetching employee reports");
    let url = format!(
        "{}/report/{}?year={}&until_week={}",
        config.backend, sales_person_id, year, calendar_week
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn add_extra_hour(
    config: Config,
    sales_person_id: Uuid,
    amount: f32,
    category: ExtraHoursCategoryTO,
    description: String,
    date_time: String,
) -> Result<(), ShiftyError> {
    let url: String = format!("{}/extra-hours", config.backend,);
    info!("Parsing datetime");
    info!("Datetime: {}", date_time);
    //let date_time = PrimitiveDateTime::parse(&date_time, &format).unwrap();
    let date_time = js::date_time_str_to_primitive_date_time(&date_time);
    info!("Datetime: {}", date_time);
    let booking_to = ExtraHoursTO {
        id: Uuid::nil(),
        sales_person_id,
        amount,
        description: description.into(),
        date_time,
        category,
        created: None,
        deleted: None,
        version: Uuid::nil(),
    };
    let client = reqwest::Client::new();
    let response = client.post(url).json(&booking_to).send().await?;
    response.error_for_status_ref()?;
    response.json().await?;
    info!("Added");
    Ok(())
}

pub async fn get_extra_hours_for_year(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
    until_week: u8,
) -> Result<Rc<[ExtraHoursTO]>, reqwest::Error> {
    info!("Fetching extra hours");
    let url = format!(
        "{}/extra-hours/by-sales-person/{}?year={}&until_week={}",
        config.backend, sales_person_id, year, until_week,
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn delete_extra_hour(config: Config, extra_hour_id: Uuid) -> Result<(), reqwest::Error> {
    info!("Deleting extra hour {extra_hour_id}");
    let url = format!("{}/extra-hours/{}", config.backend, extra_hour_id);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Deleted");
    Ok(())
}

pub async fn get_version(config: Config) -> Result<Rc<str>, reqwest::Error> {
    info!("Fetching version");
    let url = format!("{}/version", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.text().await?;
    info!("Fetched");
    Ok(res.into())
}

pub async fn get_unavailable_sales_person_days_for_week(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
    week: u8,
) -> Result<Rc<[SalesPersonUnavailableTO]>, reqwest::Error> {
    info!("Fetching unavailable sales person days for week {week} in year {year}");
    let url = format!(
        "{}/sales-person/{sales_person_id}/unavailable?year={year}&calendar_week={week}",
        config.backend
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn create_unavailable_sales_person_day(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
    week: u8,
    day_of_week: DayOfWeekTO,
) -> Result<(), reqwest::Error> {
    info!(
        "Creating unavailable sales person day for user {sales_person_id} in week {week} of year {year}"
    );
    let url = format!("{}/sales-person/unavailable", config.backend);
    let unavailable_to = SalesPersonUnavailableTO {
        id: Uuid::nil(),
        sales_person_id,
        year,
        calendar_week: week,
        day_of_week,
        created: None,
        deleted: None,
        version: Uuid::nil(),
    };
    let client = reqwest::Client::new();
    let response = client.post(url).json(&unavailable_to).send().await?;
    response.error_for_status_ref()?;
    info!("Created");
    Ok(())
}

pub async fn delete_unavailable_sales_person_day(
    config: Config,
    unavailable_id: Uuid,
) -> Result<(), reqwest::Error> {
    info!("Deleting unavailable sales person day {unavailable_id}");
    let url = format!(
        "{}/sales-person/unavailable/{}",
        config.backend, unavailable_id
    );
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Deleted");
    Ok(())
}

pub async fn get_working_hours_minified_for_week(
    config: Config,
    year: u32,
    week: u8,
) -> Result<Rc<[ShortEmployeeReportTO]>, reqwest::Error> {
    info!("Fetching working hours minified in week {week} of year {year}");
    let url = format!("{}/report/week/{}/{}", config.backend, year, week);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn get_all_users(config: Config) -> Result<Rc<[UserTO]>, reqwest::Error> {
    info!("Fetching all users");
    let url = format!("{}/permission/user", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn get_all_roles(config: Config) -> Result<Rc<[RoleTO]>, reqwest::Error> {
    info!("Fetching all roles");
    let url = format!("{}/permission/role", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn get_roles_from_user(
    config: Config,
    user_id: ImStr,
) -> Result<Rc<[RoleTO]>, reqwest::Error> {
    info!("Fetching roles from user {user_id}");
    let url = format!(
        "{}/permission/user/{}/roles",
        config.backend,
        user_id.as_str()
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn add_role_to_user(config: Config, user_role: UserRole) -> Result<(), reqwest::Error> {
    let url = format!("{}/permission/user-role", config.backend,);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&user_role).send().await?;
    response.error_for_status_ref()?;
    info!("Added");
    Ok(())
}

pub async fn remove_role_from_user(
    config: Config,
    user_role: UserRole,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/permission/user-role", config.backend,);
    let client = reqwest::Client::new();
    let response = client.delete(url).json(&user_role).send().await?;
    response.error_for_status_ref()?;
    info!("Removed");
    Ok(())
}

pub async fn add_user(config: Config, user: UserTO) -> Result<(), reqwest::Error> {
    info!("Adding user");
    let url = format!("{}/permission/user", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&user).send().await?;
    response.error_for_status_ref()?;
    info!("Added");
    Ok(())
}

pub async fn get_booking_conflicts_for_week(
    config: Config,
    year: u32,
    week: u8,
) -> Result<Rc<[BookingConflictTO]>, reqwest::Error> {
    let url = format!(
        "{}/booking-information/conflicts/for-week/{}/{}",
        config.backend, year, week,
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    Ok(res)
}

pub async fn get_weekly_overview(
    config: Config,
    year: u32,
) -> Result<Rc<[WeeklySummaryTO]>, reqwest::Error> {
    let url = format!(
        "{}/booking-information/weekly-resource-report/{}",
        config.backend, year,
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    Ok(res)
}

pub async fn get_special_days_for_week(
    config: Config,
    year: u32,
    week: u8,
) -> Result<Rc<[SpecialDayTO]>, reqwest::Error> {
    let url = format!("{}/special-days/for-week/{}/{}", config.backend, year, week,);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    Ok(res)
}

pub async fn get_employee_work_details_for_sales_person(
    config: Config,
    sales_person_id: Uuid,
) -> Result<Rc<[EmployeeWorkDetailsTO]>, reqwest::Error> {
    let url = format!(
        "{}/working-hours/for-sales-person/{}",
        config.backend, sales_person_id,
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    Ok(res)
}

pub async fn post_employee_work_details(
    config: Config,
    work_details: EmployeeWorkDetailsTO,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/working-hours", config.backend,);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&work_details).send().await?;
    response.error_for_status_ref()?;
    info!("Posted");
    Ok(())
}

pub async fn delete_employee_work_details(
    config: Config,
    work_details_id: Uuid,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/working-hours/{}", config.backend, work_details_id);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Deleted");
    Ok(())
}
