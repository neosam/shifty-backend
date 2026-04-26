use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::routing::post;
use axum::Router;
use dao::BasicDao;
use service::booking::BookingService;
use service::employee_work_details::EmployeeWorkDetailsService;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService};
use service::permission::Authentication;
use service::sales_person::{SalesPerson, SalesPersonService};
use service::slot::SlotService;
use service::special_days::{SpecialDay, SpecialDayService, SpecialDayType};
use service::ServiceError;
use shifty_utils::DayOfWeek;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::error_handler;
use crate::RestStateDef;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Dev", description = "Development-only endpoints for seeding and clearing test data"),
    ),
    paths(
        seed_dev_data,
        clear_dev_data,
    ),
)]
pub struct DevApiDoc;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/seed", post(seed_dev_data::<RestState>))
        .route("/clear", post(clear_dev_data::<RestState>))
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/seed",
    tags = ["Dev"],
    description = "Seed the database with test data for local development. Only available in dev builds.",
    responses(
        (status = 200, description = "Test data seeded successfully"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn seed_dev_data<RestState: RestStateDef>(
    rest_state: State<RestState>,
) -> Response {
    error_handler(
        (async {
            seed_data(&*rest_state).await?;
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(
                    serde_json::to_string(
                        &serde_json::json!({"message": "Test data seeded successfully"}),
                    )
                    .unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/clear",
    tags = ["Dev"],
    description = "Clear all data from the database. Only available in dev builds.",
    responses(
        (status = 200, description = "All data cleared successfully"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn clear_dev_data<RestState: RestStateDef>(
    rest_state: State<RestState>,
) -> Response {
    error_handler(
        (async {
            clear_data(&*rest_state).await?;
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(
                    serde_json::to_string(
                        &serde_json::json!({"message": "All data cleared successfully"}),
                    )
                    .unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

async fn clear_data<RestState: RestStateDef>(
    rest_state: &RestState,
) -> Result<(), crate::RestError> {
    rest_state
        .basic_dao()
        .clear_all()
        .await
        .map_err(|_| ServiceError::InternalError)?;
    Ok(())
}

#[allow(dead_code)]
pub async fn seed_dev_data_impl<RestState: RestStateDef>(
    rest_state: &RestState,
) -> Result<(), crate::RestError> {
    seed_data(rest_state).await
}

async fn seed_data<RestState: RestStateDef>(
    rest_state: &RestState,
) -> Result<(), crate::RestError> {
    let auth = Authentication::Full;

    let sales_persons = seed_sales_persons(rest_state, auth.clone()).await?;
    seed_work_details(rest_state, &sales_persons, auth.clone()).await?;
    seed_extra_hours(rest_state, &sales_persons, auth.clone()).await?;
    seed_bookings(rest_state, &sales_persons, auth.clone()).await?;
    seed_special_days(rest_state, auth).await?;

    Ok(())
}

struct SeedSalesPersons {
    anna: SalesPerson,
    max: SalesPerson,
    lisa: SalesPerson,
    tom: SalesPerson,
    sarah: SalesPerson,
}

async fn seed_sales_persons<RestState: RestStateDef>(
    rest_state: &RestState,
    auth: Authentication<crate::Context>,
) -> Result<SeedSalesPersons, crate::RestError> {
    let service = rest_state.sales_person_service();

    let create = |name: &str, color: &str, is_paid: bool, inactive: bool| SalesPerson {
        id: Uuid::nil(),
        name: Arc::from(name),
        background_color: Arc::from(color),
        is_paid: Some(is_paid),
        inactive,
        deleted: None,
        version: Uuid::nil(),
    };

    let anna = service
        .create(&create("Anna Müller", "#FF6B6B", true, false), auth.clone(), None)
        .await?;
    let max = service
        .create(&create("Max Schmidt", "#4ECDC4", true, false), auth.clone(), None)
        .await?;
    let lisa = service
        .create(&create("Lisa Weber", "#45B7D1", true, true), auth.clone(), None)
        .await?;
    let tom = service
        .create(&create("Tom Bauer", "#96CEB4", false, false), auth.clone(), None)
        .await?;
    let sarah = service
        .create(&create("Sarah Fischer", "#FFEAA7", true, false), auth, None)
        .await?;

    Ok(SeedSalesPersons {
        anna,
        max,
        lisa,
        tom,
        sarah,
    })
}

async fn seed_work_details<RestState: RestStateDef>(
    rest_state: &RestState,
    sp: &SeedSalesPersons,
    auth: Authentication<crate::Context>,
) -> Result<(), crate::RestError> {
    use service::employee_work_details::EmployeeWorkDetails;

    let service = rest_state.working_hours_service();

    let create =
        |sales_person_id: Uuid,
         hours: f32,
         workdays: u8,
         vacation: u8,
         mon: bool,
         tue: bool,
         wed: bool,
         thu: bool,
         fri: bool,
         sat: bool,
         sun: bool| {
            EmployeeWorkDetails {
                id: Uuid::nil(),
                sales_person_id,
                expected_hours: hours,
                from_day_of_week: DayOfWeek::Monday,
                from_calendar_week: 1,
                from_year: 2020,
                to_day_of_week: DayOfWeek::Sunday,
                to_calendar_week: 52,
                to_year: 2030,
                workdays_per_week: workdays,
                is_dynamic: false,
                cap_planned_hours_to_expected: false,
                monday: mon,
                tuesday: tue,
                wednesday: wed,
                thursday: thu,
                friday: fri,
                saturday: sat,
                sunday: sun,
                vacation_days: vacation,
                created: None,
                deleted: None,
                version: Uuid::nil(),
            }
        };

    // Anna: 40h, Mo-Fr, 30 vacation
    service
        .create(
            &create(sp.anna.id, 40.0, 5, 30, true, true, true, true, true, false, false),
            auth.clone(),
            None,
        )
        .await?;
    // Max: 20h, Mo-Mi, 15 vacation
    service
        .create(
            &create(sp.max.id, 20.0, 3, 15, true, true, true, false, false, false, false),
            auth.clone(),
            None,
        )
        .await?;
    // Lisa: 30h, Mo-Do, 24 vacation
    service
        .create(
            &create(sp.lisa.id, 30.0, 4, 24, true, true, true, true, false, false, false),
            auth.clone(),
            None,
        )
        .await?;
    // Tom: 10h, Sa-So, 0 vacation
    service
        .create(
            &create(sp.tom.id, 10.0, 2, 0, false, false, false, false, false, true, true),
            auth.clone(),
            None,
        )
        .await?;
    // Sarah: 35h, Mo-Fr, 28 vacation
    service
        .create(
            &create(sp.sarah.id, 35.0, 5, 28, true, true, true, true, true, false, false),
            auth,
            None,
        )
        .await?;

    Ok(())
}

async fn seed_extra_hours<RestState: RestStateDef>(
    rest_state: &RestState,
    sp: &SeedSalesPersons,
    auth: Authentication<crate::Context>,
) -> Result<(), crate::RestError> {
    let service = rest_state.extra_hours_service();
    let now = time::OffsetDateTime::now_utc();
    let today = now.date();
    let (year, week, _weekday) = today.to_iso_week_date();
    let year = year as u32;

    let monday = time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday)
        .expect("valid date");
    let tuesday = monday + time::Duration::days(1);
    let wednesday = monday + time::Duration::days(2);

    let create = |sales_person_id: Uuid, amount: f32, category: ExtraHoursCategory, description: &str, date: time::Date| {
        ExtraHours {
            id: Uuid::nil(),
            sales_person_id,
            amount,
            category,
            description: Arc::from(description),
            date_time: time::PrimitiveDateTime::new(date, time::Time::from_hms(8, 0, 0).unwrap()),
            created: None,
            deleted: None,
            version: Uuid::nil(),
        }
    };

    // Anna: 8h Vacation on Monday
    service
        .create(
            &create(sp.anna.id, 8.0, ExtraHoursCategory::Vacation, "Urlaub", monday),
            auth.clone(),
            None,
        )
        .await?;
    // Max: 8h SickLeave on Tuesday
    service
        .create(
            &create(sp.max.id, 8.0, ExtraHoursCategory::SickLeave, "Krankheit", tuesday),
            auth.clone(),
            None,
        )
        .await?;
    // Sarah: 2h ExtraWork on Wednesday
    service
        .create(
            &create(sp.sarah.id, 2.0, ExtraHoursCategory::ExtraWork, "Überstunden", wednesday),
            auth,
            None,
        )
        .await?;

    Ok(())
}

async fn seed_bookings<RestState: RestStateDef>(
    rest_state: &RestState,
    sp: &SeedSalesPersons,
    auth: Authentication<crate::Context>,
) -> Result<(), crate::RestError> {
    let now = time::OffsetDateTime::now_utc();
    let today = now.date();
    let (year, week, _weekday) = today.to_iso_week_date();
    let year = year as u32;

    let slots = rest_state
        .slot_service()
        .get_slots(auth.clone(), None)
        .await?;

    let book = |sales_person_id: Uuid, slot_id: Uuid| service::booking::Booking {
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

    // Anna: Tue, Wed, Thu, Fri (Mon = vacation)
    for slot in slots.iter().filter(|s| {
        matches!(
            s.day_of_week,
            DayOfWeek::Tuesday | DayOfWeek::Wednesday | DayOfWeek::Thursday | DayOfWeek::Friday
        )
    }) {
        rest_state
            .booking_service()
            .create(&book(sp.anna.id, slot.id), auth.clone(), None)
            .await?;
    }

    // Max: Mon only (Tue = sick)
    for slot in slots.iter().filter(|s| s.day_of_week == DayOfWeek::Monday) {
        rest_state
            .booking_service()
            .create(&book(sp.max.id, slot.id), auth.clone(), None)
            .await?;
    }

    // Tom: Sat
    for slot in slots.iter().filter(|s| s.day_of_week == DayOfWeek::Saturday) {
        rest_state
            .booking_service()
            .create(&book(sp.tom.id, slot.id), auth.clone(), None)
            .await?;
    }

    // Sarah: Mon-Fri
    for slot in slots.iter().filter(|s| {
        matches!(
            s.day_of_week,
            DayOfWeek::Monday
                | DayOfWeek::Tuesday
                | DayOfWeek::Wednesday
                | DayOfWeek::Thursday
                | DayOfWeek::Friday
        )
    }) {
        rest_state
            .booking_service()
            .create(&book(sp.sarah.id, slot.id), auth.clone(), None)
            .await?;
    }

    Ok(())
}

async fn seed_special_days<RestState: RestStateDef>(
    rest_state: &RestState,
    auth: Authentication<crate::Context>,
) -> Result<(), crate::RestError> {
    let service = rest_state.special_day_service();
    let now = time::OffsetDateTime::now_utc();
    let year = now.year() as u32;

    // Karfreitag: KW 14, Friday, Holiday
    service
        .create(
            &SpecialDay {
                id: Uuid::nil(),
                year,
                calendar_week: 14,
                day_of_week: DayOfWeek::Friday,
                day_type: SpecialDayType::Holiday,
                time_of_day: None,
                created: None,
                deleted: None,
                version: Uuid::nil(),
            },
            auth.clone(),
        )
        .await?;

    // Ostermontag: KW 14, Monday, Holiday
    service
        .create(
            &SpecialDay {
                id: Uuid::nil(),
                year,
                calendar_week: 14,
                day_of_week: DayOfWeek::Monday,
                day_type: SpecialDayType::Holiday,
                time_of_day: None,
                created: None,
                deleted: None,
                version: Uuid::nil(),
            },
            auth.clone(),
        )
        .await?;

    // Heiligabend: KW 52, Wednesday, ShortDay at 12:00
    service
        .create(
            &SpecialDay {
                id: Uuid::nil(),
                year,
                calendar_week: 52,
                day_of_week: DayOfWeek::Wednesday,
                day_type: SpecialDayType::ShortDay,
                time_of_day: Some(time::Time::from_hms(12, 0, 0).unwrap()),
                created: None,
                deleted: None,
                version: Uuid::nil(),
            },
            auth,
        )
        .await?;

    Ok(())
}
