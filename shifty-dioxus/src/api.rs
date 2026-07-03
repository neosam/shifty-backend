use std::rc::Rc;

use rest_types::{
    AbsenceListWithProjectionTO, AbsencePeriodCreateResultTO, AbsencePeriodTO,
    BillingPeriodTO, BlockTO, BookingConflictTO, BookingCreateResultTO, BookingLogTO, BookingTO,
    ConvertExtraHoursRequestTO, CreateBillingPeriodRequestTO, CreateTextTemplateRequestTO,
    CustomExtraHoursTO,
    DayOfWeekTO, EmployeeAttendanceStatisticsTO, EmployeeReportTO, EmployeeWeeklyStatisticsTO,
    EmployeeWorkDetailsTO,
    ExtraHoursCategoryTO,
    ExtraHoursTO, FeatureFlagTO, GenerateInvitationRequest, ImpersonateTO, InvitationResponse,
    PdfExportConfigTO, RoleTO, SalesPersonTO, SalesPersonUnavailableTO, ShiftplanTO,
    ShortEmployeeReportTO, SlotTO, SpecialDayTO, TextTemplateTO, UpdateTextTemplateRequestTO,
    UserRole, UserTO, VacationBalanceTO, VacationEntitlementOffsetTO, VacationPayloadTO,
    WeekMessageTO, WeekStatusTO, WeeklySummaryTO,
};
use tracing::info;
use uuid::Uuid;

use crate::{
    base_types::ImStr,
    error::ShiftyError,
    js,
    state::{week_status::WeekStatus, AuthInfo, Config, ShiftplanAssignment},
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
    let url = format!("{protocol}//{host}/assets/config.json");
    info!("URL: {url}");
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let mut res: Config = response.json().await?;
    // Normalize the backend base URL: a trailing slash in config.json
    // (e.g. "https://host/api/") combined with the `format!("{}/<path>", backend)`
    // join used throughout this module produces a double-slash path
    // ("/api//<path>"). axum 0.8 treats the empty segment as unmatched and returns
    // 404 for every such request. Strip a single trailing slash here, at the one
    // place the deployed value enters the app, so all callers build clean paths.
    res.backend = normalize_backend(&res.backend);
    info!("Loaded");
    Ok(res)
}

/// Trim exactly one trailing `/` from the backend base URL so that
/// `format!("{}/<path>", backend)` never yields a double-slash. Idempotent for
/// values without a trailing slash.
fn normalize_backend(backend: &str) -> Rc<str> {
    Rc::from(backend.strip_suffix('/').unwrap_or(backend))
}

pub async fn get_all_shiftplans(config: Config) -> Result<Rc<[ShiftplanTO]>, reqwest::Error> {
    info!("Fetching shiftplan catalog");
    let url = format!("{}/shiftplan-catalog", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched shiftplan catalog");
    Ok(res)
}

pub async fn create_shiftplan(
    config: Config,
    name: &str,
    is_planning: bool,
) -> Result<ShiftplanTO, reqwest::Error> {
    info!("Creating shiftplan");
    let url = format!("{}/shiftplan-catalog", config.backend);
    let shiftplan = ShiftplanTO {
        id: Uuid::nil(),
        name: name.into(),
        is_planning,
        deleted: None,
        version: Uuid::nil(),
    };
    let client = reqwest::Client::new();
    let response = client.post(url).json(&shiftplan).send().await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Created shiftplan");
    Ok(res)
}

pub async fn update_shiftplan(
    config: Config,
    shiftplan: ShiftplanTO,
) -> Result<ShiftplanTO, reqwest::Error> {
    info!("Updating shiftplan {}", shiftplan.id);
    let url = format!("{}/shiftplan-catalog/{}", config.backend, shiftplan.id);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&shiftplan).send().await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Updated shiftplan");
    Ok(res)
}

pub async fn delete_shiftplan(config: Config, id: Uuid) -> Result<(), reqwest::Error> {
    info!("Deleting shiftplan {id}");
    let url = format!("{}/shiftplan-catalog/{}", config.backend, id);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Deleted shiftplan");
    Ok(())
}

pub async fn get_slot(config: Config, slot_id: Uuid) -> Result<SlotTO, reqwest::Error> {
    info!("Fetching slot {slot_id}");
    let url = format!("{}/slot/{}", config.backend, slot_id);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn update_slot(
    config: Config,
    slot: SlotTO,
    year: u32,
    week: u8,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/shiftplan-edit/slot/{}/{}", config.backend, year, week);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&slot).send().await?;
    response.error_for_status_ref()?;
    info!("Updated slot");
    Ok(())
}

pub async fn update_slot_single_week(
    config: Config,
    slot: SlotTO,
    year: u32,
    week: u8,
) -> Result<(), reqwest::Error> {
    let url = format!(
        "{}/shiftplan-edit/slot/{}/{}/single-week",
        config.backend, year, week
    );
    let client = reqwest::Client::new();
    let response = client.put(url).json(&slot).send().await?;
    response.error_for_status_ref()?;
    info!("Updated slot (single week)");
    Ok(())
}

pub async fn post_slot(config: Config, slot: SlotTO) -> Result<bool, reqwest::Error> {
    info!("Adding slot");
    let url = format!("{}/slot", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&slot).send().await?;
    if response.status() == 409 {
        return Ok(false);
    }
    response.error_for_status_ref()?;
    info!("Added slot");
    Ok(true)
}

pub async fn delete_slot_from(
    config: Config,
    slot_id: Uuid,
    year: u32,
    week: u8,
) -> Result<(), reqwest::Error> {
    info!("Deleting slot {slot_id} from week {week} in year {year}");
    let url = format!(
        "{}/shiftplan-edit/slot/{}/{}/{}",
        config.backend, slot_id, year, week
    );
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Deleted");
    Ok(())
}

/// Book a slot via the conflict-aware endpoint `POST /shiftplan-edit/booking`.
/// Returns `BookingCreateResultTO { booking, warnings }` — the booking is
/// persisted immediately (optimistic create). Callers inspect `warnings` and
/// may call `remove_booking` as a rollback if the user cancels.
pub async fn book_slot_with_conflict_check(
    config: Config,
    sales_person_id: Uuid,
    slot_id: Uuid,
    week: u8,
    year: u32,
) -> Result<BookingCreateResultTO, reqwest::Error> {
    info!(
        "Booking slot (conflict-check) for user {sales_person_id}, slot {slot_id}, week {week}/{year}"
    );
    let url: String = format!("{}/shiftplan-edit/booking", config.backend);
    let booking_to = BookingTO {
        id: Uuid::nil(),
        sales_person_id,
        slot_id,
        calendar_week: week as i32,
        year,
        created: None,
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: Uuid::nil(),
    };
    let client = reqwest::Client::new();
    let response = client.post(url).json(&booking_to).send().await?;
    response.error_for_status_ref()?;
    let result: BookingCreateResultTO = response.json().await?;
    info!("Booked");
    Ok(result)
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
        sales_person.id
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
) -> Result<SalesPersonTO, reqwest::Error> {
    info!("Posting sales person");
    let url = format!("{}/sales-person", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&sales_person).send().await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Posted");
    Ok(res)
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
    info!("Delete user for sales person {sales_person_id}");
    let url = format!("{}/sales-person/{}/user", config.backend, sales_person_id);
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

pub async fn get_employee_weekly_statistics(
    config: Config,
    sales_person_id: Uuid,
) -> Result<Rc<EmployeeWeeklyStatisticsTO>, reqwest::Error> {
    info!("Fetching employee weekly statistics");
    let url = format!(
        "{}/report/{}/weekly-statistics",
        config.backend, sales_person_id
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched weekly statistics");
    Ok(Rc::new(res))
}

pub async fn get_employee_attendance_statistics(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
    until_week: u8,
) -> Result<Option<Rc<EmployeeAttendanceStatisticsTO>>, reqwest::Error> {
    info!("Fetching employee attendance statistics");
    let url = format!(
        "{}/report/{}/attendance-statistics?year={}&until_week={}",
        config.backend, sales_person_id, year, until_week
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    // Server serializes `None` (non-flexible employee / <2 attendance days at
    // the struct level) as JSON `null`, distinct from an HTTP error above.
    let opt = response
        .json::<Option<EmployeeAttendanceStatisticsTO>>()
        .await?;
    info!("Fetched attendance statistics");
    Ok(opt.map(Rc::new))
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

pub async fn update_extra_hour(
    config: Config,
    extra_hours: ExtraHoursTO,
) -> Result<ExtraHoursTO, ShiftyError> {
    info!("Updating extra hour {}", extra_hours.id);
    let url = format!("{}/extra-hours/{}", config.backend, extra_hours.id);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&extra_hours).send().await?;
    if response.status() == reqwest::StatusCode::CONFLICT {
        info!("Update returned 409 Conflict");
        return Err(ShiftyError::Conflict(String::new()));
    }
    response.error_for_status_ref()?;
    let updated: ExtraHoursTO = response.json().await?;
    info!("Updated");
    Ok(updated)
}

// ─────────────────────────────────────────────────────────────────────────
// AbsencePeriod CRUD + VacationBalance read (Phase 8 Wave 4)
//
// Backend endpoints:
//   GET    /absence-period                              → list all (HR-scope)
//   GET    /absence-period/by-sales-person/{sp_id}      → list per person
//   GET    /absence-period/{id}                         → single
//   POST   /absence-period                              → create (returns
//                                                         AbsencePeriodCreateResultTO with
//                                                         non-blocking warnings[])
//   PUT    /absence-period/{id}                         → update (409 / 422)
//   DELETE /absence-period/{id}                         → soft-delete
//   GET    /vacation-balance/{sp_id}/{year}             → self
//   GET    /vacation-balance/team/{year}                → all paid employees
// ─────────────────────────────────────────────────────────────────────────

pub async fn list_absence_periods(
    config: Config,
) -> Result<AbsenceListWithProjectionTO, reqwest::Error> {
    info!("Fetching absence periods (all)");
    let url = format!("{}/absence-period", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json::<AbsenceListWithProjectionTO>().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn list_absence_periods_by_sales_person(
    config: Config,
    sales_person_id: Uuid,
) -> Result<AbsenceListWithProjectionTO, reqwest::Error> {
    info!("Fetching absence periods for sales person {sales_person_id}");
    let url = format!(
        "{}/absence-period/by-sales-person/{}",
        config.backend, sales_person_id
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json::<AbsenceListWithProjectionTO>().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn convert_extra_hours_to_absence(
    config: Config,
    extra_hours_id: Uuid,
    body: ConvertExtraHoursRequestTO,
) -> Result<AbsencePeriodTO, ShiftyError> {
    info!("Converting extra hours {extra_hours_id} to absence");
    let url = format!(
        "{}/extra-hours/{}/convert-to-absence",
        config.backend, extra_hours_id
    );
    let client = reqwest::Client::new();
    let response = client.post(url).json(&body).send().await?;
    if response.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
        let text = response.text().await.unwrap_or_default();
        info!("Convert returned 422 Validation: {}", text);
        return Err(ShiftyError::Validation(text));
    }
    response.error_for_status_ref()?;
    let result: AbsencePeriodTO = response.json().await?;
    info!("Converted");
    Ok(result)
}

/// POST `/absence-period`. The backend rejects non-nil ids and versions on
/// create with HTTP 422 (`IdSetOnCreate`). To prevent accidental ID/version
/// passthrough from the caller (Plan 04 Task 1, W-7 / Pitfall 9 in
/// `08-RESEARCH.md`), this function defensively zeroes both fields before
/// sending.
pub async fn create_absence_period(
    config: Config,
    mut body: AbsencePeriodTO,
) -> Result<AbsencePeriodCreateResultTO, ShiftyError> {
    // W-7 defensive Uuid::nil — backend assigns id and version on create.
    body.id = Uuid::nil();
    body.version = Uuid::nil();
    info!(
        "Creating absence period for sales person {}",
        body.sales_person_id
    );
    let url = format!("{}/absence-period", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&body).send().await?;
    if response.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
        let text = response.text().await.unwrap_or_default();
        info!("Create returned 422 Validation: {}", text);
        return Err(ShiftyError::Validation(text));
    }
    response.error_for_status_ref()?;
    let result: AbsencePeriodCreateResultTO = response.json().await?;
    info!("Created");
    Ok(result)
}

/// PUT `/absence-period/{id}`. Version-conflicts surface as 409, self-overlap
/// validation surfaces as 422 (D-08 / D-11 in `08-CONTEXT.md`).
pub async fn update_absence_period(
    config: Config,
    id: Uuid,
    body: AbsencePeriodTO,
) -> Result<AbsencePeriodCreateResultTO, ShiftyError> {
    info!("Updating absence period {id}");
    let url = format!("{}/absence-period/{}", config.backend, id);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&body).send().await?;
    if response.status() == reqwest::StatusCode::CONFLICT {
        info!("Update returned 409 Conflict");
        return Err(ShiftyError::Conflict(String::new()));
    }
    if response.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
        let text = response.text().await.unwrap_or_default();
        info!("Update returned 422 Validation: {}", text);
        return Err(ShiftyError::Validation(text));
    }
    response.error_for_status_ref()?;
    let result: AbsencePeriodCreateResultTO = response.json().await?;
    info!("Updated");
    Ok(result)
}

pub async fn delete_absence_period(config: Config, id: Uuid) -> Result<(), reqwest::Error> {
    info!("Deleting absence period {id}");
    let url = format!("{}/absence-period/{}", config.backend, id);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Deleted");
    Ok(())
}

pub async fn get_vacation_balance(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
) -> Result<VacationBalanceTO, reqwest::Error> {
    info!(
        "Fetching vacation balance for sales person {sales_person_id} year {year}"
    );
    let url = format!(
        "{}/vacation-balance/{}/{}",
        config.backend, sales_person_id, year
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

/// POST `/vacation-entitlement-offset` (Phase 28, VAC-OFFSET-01, D-28-06b).
/// Upserts the signed per-(person, year) vacation-entitlement offset. HR
/// enforcement is server-side (`HR_PRIVILEGE` gate in the Basic offset
/// service — T-28-09); the FE editor visibility is convenience only. Mirrors
/// the `post(url).json(&body).send()` + `error_for_status_ref()` pattern from
/// `create_absence_period`.
pub async fn save_vacation_entitlement_offset(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
    offset_days: i32,
) -> Result<(), ShiftyError> {
    info!(
        "Saving vacation-entitlement offset for sales person {sales_person_id} year {year}: {offset_days}"
    );
    let body = VacationEntitlementOffsetTO {
        sales_person_id,
        year,
        offset_days,
    };
    let url = format!("{}/vacation-entitlement-offset", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&body).send().await?;
    response.error_for_status_ref()?;
    info!("Saved vacation-entitlement offset");
    Ok(())
}

pub async fn get_team_vacation_balance(
    config: Config,
    year: u32,
) -> Result<Rc<[VacationBalanceTO]>, reqwest::Error> {
    info!("Fetching team vacation balance for year {year}");
    let url = format!("{}/vacation-balance/team/{}", config.backend, year);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

/// Phase 8 Plan 08-07 Gap-Closure (Task 3) — `GET /feature-flag/{key}`.
/// Backend liefert für unbekannte Keys `200 OK` mit `enabled: false`
/// (fail-safe). Das ruft der `feature_flag_service`-Coroutine genau einmal
/// pro App-Start für `absence_range_source_active`.
///
/// Phase 8.6 D-02: konservierte Fetch-Funktion des generischen Flag-
/// Mechanismus; nach dem Cutover-Abriss aktuell ohne Aufrufer.
#[allow(dead_code)]
pub async fn get_feature_flag(
    config: Config,
    key: &str,
) -> Result<FeatureFlagTO, reqwest::Error> {
    info!("Fetching feature flag {key}");
    let url = format!("{}/feature-flag/{}", config.backend, key);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
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

pub async fn get_working_hours_for_week(
    config: Config,
    year: u32,
    week: u8,
) -> Result<Rc<[ShortEmployeeReportTO]>, reqwest::Error> {
    info!("Fetching working hours for week {week} of year {year}");
    let url = format!("{}/report/week/{}/{}", config.backend, year, week);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn get_balance_until_week(
    config: Config,
    year: u32,
    week: u8,
) -> Result<Rc<[ShortEmployeeReportTO]>, reqwest::Error> {
    info!("Fetching balance until week {week} of year {year}");
    let url = format!(
        "{}/report?year={}&until_week={}",
        config.backend, year, week
    );
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

pub async fn delete_user(config: Config, user_id: ImStr) -> Result<(), reqwest::Error> {
    info!("Deleting user {user_id}");
    let url = format!("{}/permission/user/", config.backend);
    let client = reqwest::Client::new();
    let response = client.delete(url).json(&user_id.to_string()).send().await?;
    response.error_for_status_ref()?;
    info!("Deleted user");
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

pub async fn get_booking_log(
    config: Config,
    year: u32,
    week: u8,
) -> Result<Rc<[BookingLogTO]>, reqwest::Error> {
    info!("Fetching booking log for week {week} in year {year}");
    let url = format!("{}/booking-log/{}/{}", config.backend, year, week);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched booking log");
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

/// GET `/special-days/for-year/{year}`. Returns all special days for the given year,
/// sorted ascending by calendar_week and day_of_week (SPD-01 / D-33-05).
pub async fn get_special_days_for_year(
    config: Config,
    year: u32,
) -> Result<Rc<[SpecialDayTO]>, reqwest::Error> {
    let url = format!("{}/special-days/for-year/{}", config.backend, year);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    Ok(res)
}

/// POST `/special-days`. Forces `id` and `version` to `Uuid::nil()` before sending
/// to prevent backend rejections (IdSetOnCreate / VersionSetOnCreate — T-33-04).
/// NOTE: no trailing slash — the Axum 0.8 nested route is registered at `/special-days`
/// (POST `/special-days/` with a trailing slash returns 404 on the real backend).
pub async fn create_special_day(
    config: Config,
    mut body: SpecialDayTO,
) -> Result<SpecialDayTO, reqwest::Error> {
    body.id = Uuid::nil();
    body.version = Uuid::nil();
    let url = format!("{}/special-days", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&body).send().await?;
    response.error_for_status_ref()?;
    let result: SpecialDayTO = response.json().await?;
    Ok(result)
}

/// DELETE `/special-days/{id}`. Errors on non-2xx (SPD-03).
pub async fn delete_special_day(config: Config, id: Uuid) -> Result<(), reqwest::Error> {
    let url = format!("{}/special-days/{}", config.backend, id);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    Ok(())
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

pub async fn put_employee_work_details(
    config: Config,
    work_details: EmployeeWorkDetailsTO,
) -> Result<(), reqwest::Error> {
    let url = format!(
        "{}/working-hours/{}",
        config.backend,
        work_details.id
    );
    let client = reqwest::Client::new();
    let response = client.put(url).json(&work_details).send().await?;
    response.error_for_status_ref()?;
    info!("Updated");
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

pub async fn add_vacation(
    config: Config,
    sales_person_id: Uuid,
    from: time::Date,
    to: time::Date,
    description: ImStr,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/shiftplan-edit/vacation", config.backend,);
    let vacation_to = VacationPayloadTO {
        sales_person_id,
        from,
        to,
        description: description.as_str().into(),
    };
    let client = reqwest::Client::new();
    let response = client.put(url).json(&vacation_to).send().await?;
    response.error_for_status_ref()?;
    info!("Added");
    Ok(())
}

pub async fn get_shiftplan_week(
    config: Config,
    shiftplan_id: Uuid,
    year: u32,
    week: u8,
) -> Result<rest_types::ShiftplanWeekTO, reqwest::Error> {
    info!("Fetching shiftplan for week {week} in year {year}");
    let url = format!(
        "{}/shiftplan-info/{shiftplan_id}/{year}/{week}",
        config.backend
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn get_shiftplan_day(
    config: Config,
    year: u32,
    week: u8,
    day_of_week: rest_types::DayOfWeekTO,
) -> Result<rest_types::ShiftplanDayAggregateTO, reqwest::Error> {
    info!("Fetching shiftplan day aggregate for week {week} in year {year}");
    let day_str = match day_of_week {
        rest_types::DayOfWeekTO::Monday => "Monday",
        rest_types::DayOfWeekTO::Tuesday => "Tuesday",
        rest_types::DayOfWeekTO::Wednesday => "Wednesday",
        rest_types::DayOfWeekTO::Thursday => "Thursday",
        rest_types::DayOfWeekTO::Friday => "Friday",
        rest_types::DayOfWeekTO::Saturday => "Saturday",
        rest_types::DayOfWeekTO::Sunday => "Sunday",
    };
    let url = format!(
        "{}/shiftplan-info/day/{year}/{week}/{day_str}",
        config.backend
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched day aggregate");
    Ok(res)
}

pub async fn get_custom_extra_hours_by_sales_person(
    config: Config,
    sales_person_id: Uuid,
) -> Result<Rc<[CustomExtraHoursTO]>, reqwest::Error> {
    info!("Fetching custom extra hours for sales person {sales_person_id}");
    let url = format!(
        "{}/custom-extra-hours/by-sales-person/{sales_person_id}",
        config.backend
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn post_custom_extra_hours(
    config: Config,
    custom_extra_hours: CustomExtraHoursTO,
) -> Result<(), reqwest::Error> {
    info!("Creating custom extra hours: {}", custom_extra_hours.name);
    let url = format!("{}/custom-extra-hours", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&custom_extra_hours).send().await?;
    response.error_for_status_ref()?;
    info!("Created");
    Ok(())
}

pub async fn put_custom_extra_hours(
    config: Config,
    custom_extra_hours: CustomExtraHoursTO,
) -> Result<(), reqwest::Error> {
    info!("Updating custom extra hours: {}", custom_extra_hours.name);
    let url = format!(
        "{}/custom-extra-hours/{}",
        config.backend, custom_extra_hours.id
    );
    let client = reqwest::Client::new();
    let response = client.put(url).json(&custom_extra_hours).send().await?;
    response.error_for_status_ref()?;
    info!("Updated");
    Ok(())
}

pub async fn delete_custom_extra_hours(
    config: Config,
    custom_extra_hours_id: Uuid,
) -> Result<(), reqwest::Error> {
    info!("Deleting custom extra hours {custom_extra_hours_id}");
    let url = format!(
        "{}/custom-extra-hours/{}",
        config.backend, custom_extra_hours_id
    );
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Deleted custom extra hours");
    Ok(())
}

pub async fn get_week_message(
    config: Config,
    year: u32,
    week: u8,
) -> Result<Option<WeekMessageTO>, reqwest::Error> {
    info!("Fetching week message for {year}/{week}");
    let url = format!(
        "{}/week-message/by-year-and-week/{}/{}",
        config.backend, year, week
    );
    let response = reqwest::get(url).await?;
    if response.status() == 404 {
        return Ok(None);
    }
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched week message");
    Ok(Some(res))
}

pub async fn post_week_message(
    config: Config,
    week_message: WeekMessageTO,
) -> Result<(), reqwest::Error> {
    info!(
        "Posting week message for {}/{}",
        week_message.year, week_message.calendar_week
    );
    let url = format!("{}/week-message", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&week_message).send().await?;
    response.error_for_status_ref()?;
    info!("Posted week message");
    Ok(())
}

pub async fn put_week_message(
    config: Config,
    week_message: WeekMessageTO,
) -> Result<(), reqwest::Error> {
    info!(
        "Updating week message for {}/{}",
        week_message.year, week_message.calendar_week
    );
    let url = format!("{}/week-message/{}", config.backend, week_message.id);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&week_message).send().await?;
    response.error_for_status_ref()?;
    info!("Updated week message");
    Ok(())
}

pub async fn get_week_status(
    config: Config,
    year: u32,
    week: u8,
) -> Result<WeekStatusTO, reqwest::Error> {
    // Backend always returns HTTP 200 with status="unset" when no row exists —
    // it never returns 404 for this endpoint (D-39-06).
    info!("Fetching week status for {year}/{week}");
    let url = format!(
        "{}/week-status/by-year-and-week/{}/{}",
        config.backend, year, week
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched week status");
    Ok(res)
}

pub async fn set_week_status(
    config: Config,
    year: u32,
    week: u8,
    status: WeekStatus,
) -> Result<(), reqwest::Error> {
    info!("Setting week status for {year}/{week}");
    let url = format!(
        "{}/week-status/by-year-and-week/{}/{}",
        config.backend, year, week
    );
    let body = WeekStatusTO {
        year,
        calendar_week: week,
        status: (&status).into(),
    };
    let client = reqwest::Client::new();
    let response = client.put(url).json(&body).send().await?;
    response.error_for_status_ref()?;
    info!("Set week status");
    Ok(())
}

pub async fn get_sales_person_by_user(
    config: Config,
    username: ImStr,
) -> Result<Option<SalesPersonTO>, reqwest::Error> {
    info!("Fetching sales person for user {username}");
    let url = format!("{}/sales-person/by-user/{}", config.backend, username);
    let response = reqwest::get(url).await?;
    if response.status() == 404 {
        return Ok(None);
    }
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched sales person for user");
    Ok(Some(res))
}

pub async fn delete_billing_period(config: Config, id: Uuid) -> Result<(), reqwest::Error> {
    info!("Deleting billing period {id}");
    let url = format!("{}/billing-period/{}", config.backend, id);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Deleted billing period");
    Ok(())
}

pub async fn get_billing_periods(config: Config) -> Result<Rc<[BillingPeriodTO]>, reqwest::Error> {
    info!("Fetching billing periods");
    let url = format!("{}/billing-period", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn get_billing_period(
    config: Config,
    billing_period_id: Uuid,
) -> Result<BillingPeriodTO, reqwest::Error> {
    info!("Fetching billing period {billing_period_id}");
    let url = format!("{}/billing-period/{}", config.backend, billing_period_id);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}

pub async fn post_billing_period(
    config: Config,
    end_date: time::Date,
) -> Result<(), reqwest::Error> {
    info!("Creating billing period with end date {end_date}");
    let url = format!("{}/billing-period", config.backend);
    let request_payload = CreateBillingPeriodRequestTO { end_date };
    let client = reqwest::Client::new();
    let response = client.post(url).json(&request_payload).send().await?;
    response.error_for_status_ref()?;
    info!("Created billing period");
    Ok(())
}

// Text Template APIs
pub async fn get_text_templates(config: Config) -> Result<Rc<[TextTemplateTO]>, reqwest::Error> {
    info!("Fetching all text templates");
    let url = format!("{}/text-templates", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched text templates");
    Ok(res)
}

pub async fn get_text_templates_by_type(
    config: Config,
    template_type: &str,
) -> Result<Rc<[TextTemplateTO]>, reqwest::Error> {
    info!("Fetching text templates by type: {template_type}");
    let url = format!(
        "{}/text-templates/by-type/{}",
        config.backend, template_type
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched text templates by type");
    Ok(res)
}

pub async fn create_text_template(
    config: Config,
    template: CreateTextTemplateRequestTO,
) -> Result<TextTemplateTO, reqwest::Error> {
    info!("Creating text template");
    let url = format!("{}/text-templates", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&template).send().await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Created text template");
    Ok(res)
}

pub async fn update_text_template(
    config: Config,
    template_id: Uuid,
    template: UpdateTextTemplateRequestTO,
) -> Result<TextTemplateTO, reqwest::Error> {
    info!("Updating text template {template_id}");
    let url = format!("{}/text-templates/{}", config.backend, template_id);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&template).send().await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Updated text template");
    Ok(res)
}

pub async fn delete_text_template(config: Config, template_id: Uuid) -> Result<(), reqwest::Error> {
    info!("Deleting text template {template_id}");
    let url = format!("{}/text-templates/{}", config.backend, template_id);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Deleted text template");
    Ok(())
}

pub async fn generate_custom_report(
    config: Config,
    billing_period_id: Uuid,
    template_id: Uuid,
) -> Result<String, reqwest::Error> {
    info!("Generating custom report for billing period {billing_period_id} with template {template_id}");
    let url = format!(
        "{}/billing-period/{}/custom-report/{}",
        config.backend, billing_period_id, template_id
    );
    let client = reqwest::Client::new();
    let response = client.post(url).send().await?;
    response.error_for_status_ref()?;
    let res = response.text().await?;
    info!("Generated custom report");
    Ok(res)
}

pub async fn generate_block_report(
    config: Config,
    template_id: Uuid,
) -> Result<String, reqwest::Error> {
    info!("Generating block report with template {template_id}");
    let url = format!("{}/block-report/{}", config.backend, template_id);
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    response.error_for_status_ref()?;
    let res = response.text().await?;
    info!("Generated block report");
    Ok(res)
}

pub async fn generate_invitation(
    config: Config,
    request: GenerateInvitationRequest,
) -> Result<InvitationResponse, reqwest::Error> {
    info!("Generating invitation for user {}", request.username);
    let url = format!("{}/user-invitation/invitation", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&request).send().await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Generated invitation");
    Ok(res)
}

/// Parses the raw JSON body returned by
/// `GET /user-invitation/invitation/user/{name}` into
/// `Rc<[InvitationResponse]>`.
///
/// Extracted as a pure function so BUG-02's regression path is unit-testable
/// without touching `reqwest`. On parse failure we return
/// [`crate::error::ShiftyError::InvitationParse`] carrying the serde error
/// message concatenated with the first 200 characters of the body — enough
/// to diagnose schema drift without leaking a full response into
/// long-lived state. See `error.rs` for the security note on why the
/// snippet is safe.
pub(crate) fn parse_invitations_response(
    body: &str,
) -> Result<Rc<[InvitationResponse]>, crate::error::ShiftyError> {
    match serde_json::from_str::<Rc<[InvitationResponse]>>(body) {
        Ok(invitations) => {
            info!("Successfully parsed {} invitations", invitations.len());
            for (i, invitation) in invitations.iter().enumerate() {
                info!(
                    "Invitation {}: id={}, username={}, status={:?}, redeemed_at={:?}",
                    i,
                    invitation.id,
                    invitation.username,
                    invitation.status,
                    invitation.redeemed_at
                );
            }
            Ok(invitations)
        }
        Err(e) => {
            tracing::error!("Failed to deserialize invitations: {}", e);
            tracing::error!("Response text was: {}", body);
            let head: String = body.chars().take(200).collect();
            Err(crate::error::ShiftyError::InvitationParse(format!(
                "{e} — body head: {head}"
            )))
        }
    }
}

pub async fn list_user_invitations(
    config: Config,
    username: ImStr,
) -> Result<Rc<[InvitationResponse]>, crate::error::ShiftyError> {
    info!("Fetching invitations for user {username}");
    let url = format!(
        "{}/user-invitation/invitation/user/{}",
        config.backend, username
    );
    info!("Invitation API URL: {}", url);

    let response = reqwest::get(url).await?;
    info!("Response status: {}", response.status());

    response.error_for_status_ref()?;

    // Get the raw response text first to see what we're working with
    let response_text = response.text().await?;
    info!("Raw response body: {}", response_text);

    parse_invitations_response(&response_text)
}

pub async fn revoke_invitation(config: Config, invitation_id: Uuid) -> Result<(), reqwest::Error> {
    info!("Revoking invitation {invitation_id}");
    let url = format!(
        "{}/user-invitation/invitation/{}",
        config.backend, invitation_id
    );
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    info!("Revoked invitation");
    Ok(())
}

pub async fn revoke_session_for_invitation(
    config: Config,
    invitation_id: Uuid,
) -> Result<(), reqwest::Error> {
    info!("Revoking session for invitation {invitation_id}");
    let url = format!(
        "{}/user-invitation/invitation/{}/revoke-session",
        config.backend, invitation_id
    );
    let client = reqwest::Client::new();
    let response = client.post(url).send().await?;
    response.error_for_status_ref()?;
    info!("Revoked session for invitation");
    Ok(())
}

// Sales person shiftplan assignment
pub async fn get_shiftplan_assignments(
    config: Config,
    sales_person_id: Uuid,
) -> Result<Vec<ShiftplanAssignment>, reqwest::Error> {
    info!("Fetching shiftplan assignments for sales person {sales_person_id}");
    let url = format!(
        "{}/sales-person-shiftplan/{}/shiftplans",
        config.backend, sales_person_id
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched shiftplan assignments");
    Ok(res)
}

pub async fn set_shiftplan_assignments(
    config: Config,
    sales_person_id: Uuid,
    assignments: &[ShiftplanAssignment],
) -> Result<(), reqwest::Error> {
    info!("Setting shiftplan assignments for sales person {sales_person_id}");
    let url = format!(
        "{}/sales-person-shiftplan/{}/shiftplans",
        config.backend, sales_person_id
    );
    let client = reqwest::Client::new();
    let response = client.put(url).json(assignments).send().await?;
    response.error_for_status_ref()?;
    info!("Set shiftplan assignments");
    Ok(())
}

pub async fn get_bookable_sales_persons(
    config: Config,
    shiftplan_id: Uuid,
) -> Result<Rc<[SalesPersonTO]>, reqwest::Error> {
    info!("Fetching bookable sales persons for shiftplan {shiftplan_id}");
    let url = format!(
        "{}/sales-person-shiftplan/by-shiftplan/{}",
        config.backend, shiftplan_id
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched bookable sales persons");
    Ok(res)
}

pub async fn get_blocks(
    config: Config,
    from_year: u32,
    from_week: u8,
    to_year: u32,
    to_week: u8,
) -> Result<Rc<[BlockTO]>, reqwest::Error> {
    info!("Fetching blocks from {from_year}/{from_week} to {to_year}/{to_week}");
    let url = format!(
        "{}/blocks/{}/{}/{}/{}",
        config.backend, from_year, from_week, to_year, to_week
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched blocks");
    Ok(res)
}

#[cfg(test)]
mod normalize_backend_tests {
    use super::normalize_backend;

    /// Regression for convert-to-absence-404: deployed config.json carried a
    /// trailing slash ("https://host/api/"), and `format!("{}/<path>", backend)`
    /// produced "/api//<path>", which axum 0.8 rejects with 404 (empty path
    /// segment). The normalized base must never re-introduce a double-slash.
    #[test]
    fn strips_single_trailing_slash() {
        assert_eq!(
            normalize_backend("https://host/api/").as_ref(),
            "https://host/api"
        );
    }

    #[test]
    fn leaves_url_without_trailing_slash_untouched() {
        assert_eq!(
            normalize_backend("https://host/api").as_ref(),
            "https://host/api"
        );
    }

    #[test]
    fn joined_path_has_no_double_slash() {
        let backend = normalize_backend("https://shifty-int.example.de/api/");
        let url = format!(
            "{}/extra-hours/{}/convert-to-absence",
            backend, "cfaba0dd-42a2-4e19-89d7-e8c126340a36"
        );
        // No "//" except the one in the scheme ("https://").
        let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(&url);
        assert!(
            !after_scheme.contains("//"),
            "url unexpectedly contains a double-slash: {url}"
        );
        assert_eq!(
            url,
            "https://shifty-int.example.de/api/extra-hours/cfaba0dd-42a2-4e19-89d7-e8c126340a36/convert-to-absence"
        );
    }

    #[test]
    fn empty_backend_stays_empty() {
        // An empty backend is the "still loading" sentinel (app.rs gates on
        // `!config.backend.is_empty()`); normalization must not change it.
        assert_eq!(normalize_backend("").as_ref(), "");
    }
}

#[cfg(test)]
mod parse_invitations_tests {
    //! BUG-02 (v2.2): the `list_user_invitations` API used to silently
    //! swallow serde-parse failures by returning an empty-Ok array. These
    //! tests pin the new behavior: a Parse-Error must surface as
    //! `ShiftyError::InvitationParse(_)` — never as an empty-Ok — while
    //! valid JSON continues to parse successfully.
    use super::parse_invitations_response;
    use crate::error::ShiftyError;

    /// Test 1 — the new `ShiftyError::InvitationParse` display renders the
    /// diagnostic prefix `"invitation parse error"` and preserves the
    /// underlying cause message.
    #[test]
    fn invitation_parse_display_contains_prefix_and_cause() {
        let err = ShiftyError::InvitationParse("expected `,` at line 3".into());
        let rendered = err.to_string();
        assert!(
            rendered.contains("invitation parse error"),
            "missing prefix in {rendered:?}",
        );
        assert!(
            rendered.contains("expected `,` at line 3"),
            "cause message not surfaced in {rendered:?}",
        );
    }

    /// Test 2 — well-formed JSON (empty array + a populated array) must
    /// continue to parse as `Ok`. Guards against a regression where the
    /// pure-fn extraction accidentally rejected valid bodies.
    #[test]
    fn valid_empty_array_parses_ok() {
        let parsed = parse_invitations_response("[]")
            .expect("valid empty JSON array must parse");
        assert_eq!(parsed.len(), 0);
    }

    #[test]
    fn valid_populated_array_parses_ok() {
        // Minimal InvitationResponse fixture — pins rest-types field names
        // (id/username/token/invitation_link/redeemed_at/status). A schema
        // change on any of these fields fails this test, which is exactly
        // the drift-detection BUG-02 needs to catch before it hits the UI.
        let body = r#"[
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "username": "alice",
                "token": "22222222-2222-2222-2222-222222222222",
                "invitation_link": "http://example.com/invite/abc",
                "redeemed_at": null,
                "status": "valid"
            }
        ]"#;
        let parsed = parse_invitations_response(body)
            .expect("valid invitation JSON must parse");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].username, "alice");
    }

    /// Test 3 — malformed JSON must NOT silently degrade to an empty-Ok
    /// array; it must produce `Err(ShiftyError::InvitationParse(_))` with a
    /// non-empty message. This is the core BUG-02 regression.
    #[test]
    fn invalid_json_returns_invitation_parse_error() {
        let err = parse_invitations_response("nope")
            .expect_err("invalid JSON must surface as an error, not silent-empty");
        match &err {
            ShiftyError::InvitationParse(msg) => {
                assert!(
                    !msg.is_empty(),
                    "InvitationParse message must not be empty",
                );
                assert!(
                    msg.contains("body head"),
                    "expected body-head snippet in message: {msg:?}",
                );
            }
            other => panic!(
                "expected ShiftyError::InvitationParse, got: {other:?}",
            ),
        }
    }

    /// Backend-shape-drift regression: a `{"error": "..."}` object (instead
    /// of the expected top-level array) is exactly the kind of body that
    /// v1.x silently rendered as "no invitations". It must surface an
    /// error now.
    #[test]
    fn wrong_shape_object_returns_invitation_parse_error() {
        let err = parse_invitations_response(r#"{"error":"nope"}"#)
            .expect_err("wrong-shape body must surface as an error");
        assert!(
            matches!(err, ShiftyError::InvitationParse(_)),
            "expected InvitationParse variant, got: {err:?}",
        );
    }
}

// ─── Toggle REST client (Phase 24 D-24-06) ───────────────────────────────────

/// PUT /toggle/{name}/enable or /disable
pub async fn set_toggle(
    config: Config,
    name: &str,
    enabled: bool,
) -> Result<(), reqwest::Error> {
    let verb = if enabled { "enable" } else { "disable" };
    let url = format!("{}/toggle/{}/{}", config.backend, name, verb);
    let client = reqwest::Client::new();
    client.put(url).send().await?.error_for_status()?;
    Ok(())
}

/// GET /toggle/{name}/enabled → bool
pub async fn get_toggle_enabled(config: Config, name: &str) -> Result<bool, reqwest::Error> {
    let url = format!("{}/toggle/{}/enabled", config.backend, name);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    response.json().await
}

// ─── Toggle value REST clients (Phase 25 D-25-06) ────────────────────────────

/// GET /toggle/{name}/value → Option<String>
/// Returns Ok(None) when the server responds 204 (value not set).
pub async fn get_toggle_value(
    config: Config,
    name: &str,
) -> Result<Option<String>, reqwest::Error> {
    let url = format!("{}/toggle/{}/value", config.backend, name);
    let response = reqwest::get(url).await?;
    if response.status() == 204 {
        return Ok(None);
    }
    response.error_for_status_ref()?;
    response.json::<Option<String>>().await
}

/// PUT /toggle/{name}/value — sets the value to the given ISO date string.
pub async fn set_toggle_value(
    config: Config,
    name: &str,
    value: &str,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/toggle/{}/value", config.backend, name);
    let client = reqwest::Client::new();
    client
        .put(url)
        .json(value)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

/// DELETE /toggle/{name}/value — clears the value (disables automation).
pub async fn clear_toggle_value(config: Config, name: &str) -> Result<(), reqwest::Error> {
    let url = format!("{}/toggle/{}/value", config.backend, name);
    let client = reqwest::Client::new();
    client.delete(url).send().await?.error_for_status()?;
    Ok(())
}

// ─── Impersonation REST clients (Phase 32) ───────────────────────────────────

/// GET `/admin/impersonate` — returns the current impersonation status for
/// the authenticated admin (D-32-05).  A 403 from the server indicates the
/// caller is not an admin; the service layer maps that to "not impersonating"
/// rather than surfacing an error to non-admins.
pub async fn get_impersonate_status(config: Config) -> Result<ImpersonateTO, reqwest::Error> {
    info!("Fetching impersonation status");
    let url = format!("{}/admin/impersonate", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched impersonation status");
    Ok(res)
}

/// POST `/admin/impersonate/{user_id}` — start impersonating the given user.
/// `user_id` is the auth username / identity and goes in the URL path (D-32-03:
/// no body payload; `ImpersonateTO` is not changed).  Returns the new
/// impersonation state from the server.
pub async fn start_impersonate(
    config: Config,
    user_id: ImStr,
) -> Result<ImpersonateTO, reqwest::Error> {
    info!("Starting impersonation as {user_id}");
    let url = format!("{}/admin/impersonate/{}", config.backend, user_id.as_str());
    let client = reqwest::Client::new();
    let response = client.post(url).send().await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Started impersonation");
    Ok(res)
}

/// DELETE `/admin/impersonate` — stop the current impersonation session.
/// Returns the cleared impersonation state from the server (D-32-06 / IMP-04).
pub async fn stop_impersonate(config: Config) -> Result<ImpersonateTO, reqwest::Error> {
    info!("Stopping impersonation");
    let url = format!("{}/admin/impersonate", config.backend);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Stopped impersonation");
    Ok(res)
}

// ─── PDF-Export-Config REST client (Phase 48-05 EXP-02 / EXP-03) ────────────

/// GET `/pdf-export-config` — admin-gated. Response has `webdav_app_token = None`
/// per T-48-02 (server masks the token in every response).
pub async fn get_pdf_export_config(
    config: &Config,
) -> Result<PdfExportConfigTO, ShiftyError> {
    info!("Fetching PDF export config");
    let url = format!("{}/pdf-export-config", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res: PdfExportConfigTO = response.json().await?;
    info!("Fetched PDF export config");
    Ok(res)
}

/// PUT `/pdf-export-config` — admin-gated. When `body.webdav_app_token` is
/// `None` the backend keeps the existing token (D-48-UI-TOKEN-KEEP); a
/// `Some(v)` replaces it.
pub async fn put_pdf_export_config(
    config: &Config,
    body: PdfExportConfigTO,
) -> Result<PdfExportConfigTO, ShiftyError> {
    info!("Updating PDF export config");
    let url = format!("{}/pdf-export-config", config.backend);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&body).send().await?;
    response.error_for_status_ref()?;
    let res: PdfExportConfigTO = response.json().await?;
    info!("Updated PDF export config");
    Ok(res)
}

/// POST `/pdf-export-config/trigger` — admin-gated, spawns `run_once_now` in
/// the background. Backend responds with **204 No Content** (per Plan 48-04
/// decision — HYG-05 content-type-surface gate) once accepted; the actual run
/// happens asynchronously and its outcome lands in `last_success_at` /
/// `last_error_at` (visible via GET).
pub async fn trigger_pdf_export(config: &Config) -> Result<(), ShiftyError> {
    info!("Triggering PDF export run");
    let url = format!("{}/pdf-export-config/trigger", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).send().await?;
    response.error_for_status_ref()?;
    info!("PDF export trigger accepted");
    Ok(())
}


