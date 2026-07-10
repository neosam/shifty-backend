//! Phase 54 Plan 04 + Gap-Closure G1 (Plan 54-07) — HTTP-Integrationstest fuer
//! `GET /report/{id}/voluntary-stats?from_date=YYYY-MM-DD&to_date=YYYY-MM-DD`.
//!
//! Zielsetzung (Plan 54-04 Task 3):
//!   1. HR-User (`DEVUSER` via `create_admin_user`) → 200 mit Some-Feldern.
//!   2. Non-HR-User (`some-non-hr-user`, kein Role-Binding) → 200 mit allen
//!      Feldern null (API-Level-Redaktion, Praezedenz VAC-OFFSET-01 v1.8).
//!
//! Der Plan-Text-Alternativvorschlag "rest/tests/voluntary_stats.rs mit
//! Handrolled-Fixture" ist im Repo unpraktisch, weil `RestStateDef` inzwischen
//! ~35 Services fuehrt und eine Ad-hoc-Fixture > 50 Zeilen unimplemented!()
//! braeuchte. Die Praezedenz im Repo (convert_to_absence.rs, feature_flag.rs
//! aus Phase 8.5 / 8.7) fuehrt HTTP-Roundtrip-Tests im shifty_bin-integration_test
//! Modul mit dem echten `RestStateImpl` und `tower::ServiceExt::oneshot`. Diese
//! Datei folgt dem etablierten Muster — sie ist der explizit im Plan-Text
//! zugelassene Fallback (siehe Task 3 "Alternativ: falls Vollimpl zu invasiv,
//! HTTP-Test im shifty_bin/tests/-Modul").

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use http_body_util::BodyExt;
use rest::report::generate_route;
use rest::{Context as RestContext, RestStateDef};
use rest_types::VoluntaryStatsTO;
use service::employee_work_details::{EmployeeWorkDetails, EmployeeWorkDetailsService};
use service::extra_hours::{
    ExtraHours, ExtraHoursCategory, ExtraHoursService, ExtraHoursSource,
};
use service::permission::Authentication;
use service::sales_person::{SalesPerson, SalesPersonService};
use shifty_utils::DayOfWeek;
use time::macros::date;
use time::PrimitiveDateTime;
use tower::ServiceExt;
use uuid::Uuid;

use crate::integration_test::TestSetup;

/// Seed: eine SalesPerson + ein EmployeeWorkDetails-Vertrag ueber 4 ISO-Wochen
/// (2026 KW 10..=13, alle 5 Wochentage Mo..Fr) mit `committed_voluntary = 2.0`,
/// und ein Manual-VolunteerWork ExtraHours-Eintrag von 8.0h in KW 10.
///
/// Erwartete VoluntaryStats fuer HR (year=2026):
/// - contract_weeks = 4 (KW 10, 11, 12, 13)
/// - ist_total     = 8.0 (Manual VolunteerWork)
/// - soll_total    = 4 * 2.0 = 8.0
/// - delta         = ist_total - soll_total = 0.0
/// - ist_per_contract_week = 8.0 / 4 = 2.0
async fn seed_voluntary_scenario(test_setup: &TestSetup) -> Uuid {
    let sales_person = test_setup
        .rest_state
        .sales_person_service()
        .create(
            &SalesPerson {
                id: Uuid::nil(),
                version: Uuid::nil(),
                name: "VOL-TESTPERSON".into(),
                background_color: "#112233".into(),
                inactive: false,
                is_paid: Some(true),
                deleted: None,
            },
            Authentication::Full,
            None,
        )
        .await
        .expect("create sales_person failed");

    test_setup
        .rest_state
        .working_hours_service()
        .create(
            &EmployeeWorkDetails {
                id: Uuid::nil(),
                sales_person_id: sales_person.id,
                expected_hours: 40.0,
                from_year: 2026,
                from_calendar_week: 10,
                from_day_of_week: DayOfWeek::Monday,
                to_year: 2026,
                to_calendar_week: 13,
                to_day_of_week: DayOfWeek::Sunday,
                workdays_per_week: 5,
                monday: true,
                tuesday: true,
                wednesday: true,
                thursday: true,
                friday: true,
                saturday: false,
                sunday: false,
                is_dynamic: false,
                cap_planned_hours_to_expected: false,
                committed_voluntary: 2.0,
                vacation_days: 25,
                created: None,
                deleted: None,
                version: Uuid::nil(),
            },
            Authentication::Full,
            None,
        )
        .await
        .expect("create working_hours failed");

    // Manual VolunteerWork-Row am Montag der KW 10/2026 (2026-03-02).
    test_setup
        .rest_state
        .extra_hours_service()
        .create(
            &ExtraHours {
                id: Uuid::nil(),
                sales_person_id: sales_person.id,
                amount: 8.0,
                category: ExtraHoursCategory::VolunteerWork,
                description: "Freiwillig 8h".into(),
                date_time: PrimitiveDateTime::new(date!(2026 - 03 - 02), time::Time::MIDNIGHT),
                created: None,
                deleted: None,
                version: Uuid::nil(),
                source: ExtraHoursSource::Manual,
            },
            Authentication::Full,
            None,
        )
        .await
        .expect("create extra_hours failed");

    sales_person.id
}

/// HR-Pfad: DEVUSER hat via `create_admin_user` alle Privilegien inkl. `hr`.
/// Erwartet 200 mit gesetzten Some-Feldern und den geseedeten Werten.
#[tokio::test]
async fn rest_voluntary_stats_hr_returns_populated_fields() {
    let test_setup = TestSetup::new().await;
    let sales_person_id = seed_voluntary_scenario(&test_setup).await;

    let router = axum::Router::new()
        .nest("/report", generate_route::<crate::RestStateImpl>())
        .with_state(test_setup.rest_state.clone())
        .layer(Extension(Some(Arc::<str>::from("DEVUSER")) as RestContext));

    let uri = format!(
        "/report/{}/voluntary-stats?from_date=2026-01-01&to_date=2026-12-31",
        sales_person_id
    );
    let req = Request::builder()
        .method("GET")
        .uri(&uri)
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "HR must receive 200");

    let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let to: VoluntaryStatsTO = serde_json::from_slice(&body_bytes).expect("parse response DTO");

    assert_eq!(
        to.contract_weeks,
        Some(4),
        "HR expects contract_weeks = 4 (KW 10..=13/2026)"
    );
    // Range = 2026-01-01..=2026-12-31 (Full-Year).
    // Vertrag KW 10..=13 = 2026-03-02..=2026-03-29 = 28 aktive Tage.
    // soll_total = 28 * 2.0 / 7.0 = 8.0 (tages-basiert, Float-Toleranz).
    let ist_total = to.ist_total.expect("HR expects ist_total set");
    let soll_total = to.soll_total.expect("HR expects soll_total set");
    let delta = to.delta.expect("HR expects delta set");
    let ist_per = to
        .ist_per_contract_week
        .expect("HR expects ist_per_contract_week set");
    assert!(
        (ist_total - 8.0).abs() < 1e-3,
        "HR expects ist_total = 8.0 (Manual VolunteerWork); got {ist_total}"
    );
    assert!(
        (soll_total - 8.0).abs() < 1e-3,
        "HR expects soll_total = 8.0 (28 days * 2.0 / 7 tages-basiert); got {soll_total}"
    );
    assert!(
        delta.abs() < 1e-3,
        "HR expects delta = 0.0; got {delta}"
    );
    assert!(
        (ist_per - 2.0).abs() < 1e-3,
        "HR expects F1 = 8.0 / 4 = 2.0; got {ist_per}"
    );
    // Quick-Task 260710: Erfuellungsgrad = ist_total / soll_total * 100.
    // Fixture liefert ist=8.0, soll=8.0 => 100 %.
    let pct = to
        .ist_per_soll_pct
        .expect("HR expects ist_per_soll_pct set when soll_total > 0");
    assert!(
        (pct - 100.0).abs() < 1e-3,
        "HR expects Erfuellungsgrad = 8.0/8.0*100 = 100.0; got {pct}"
    );
}

/// Non-HR-Pfad: unbekannter User, kein Role-Binding auf `hr`.
/// Erwartet 200 mit allen Feldern `null` (API-Level-Redaktion, Praezedenz
/// VAC-OFFSET-01 v1.8). KEIN 403.
#[tokio::test]
async fn rest_voluntary_stats_non_hr_returns_all_null() {
    let test_setup = TestSetup::new().await;
    let sales_person_id = seed_voluntary_scenario(&test_setup).await;

    let router = axum::Router::new()
        .nest("/report", generate_route::<crate::RestStateImpl>())
        .with_state(test_setup.rest_state.clone())
        .layer(Extension(
            Some(Arc::<str>::from("some-non-hr-user")) as RestContext,
        ));

    let uri = format!(
        "/report/{}/voluntary-stats?from_date=2026-01-01&to_date=2026-12-31",
        sales_person_id
    );
    let req = Request::builder()
        .method("GET")
        .uri(&uri)
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Non-HR must receive 200 (redaction, not 403)"
    );

    let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    let to: VoluntaryStatsTO = serde_json::from_slice(&body_bytes)
        .unwrap_or_else(|e| panic!("parse response DTO failed: {e} body={body_str}"));

    assert!(
        to.ist_per_contract_week.is_none(),
        "Non-HR expects ist_per_contract_week = null, got {:?}",
        to.ist_per_contract_week
    );
    assert!(
        to.ist_total.is_none(),
        "Non-HR expects ist_total = null, got {:?}",
        to.ist_total
    );
    assert!(
        to.soll_total.is_none(),
        "Non-HR expects soll_total = null, got {:?}",
        to.soll_total
    );
    assert!(to.delta.is_none(), "Non-HR expects delta = null, got {:?}", to.delta);
    assert!(
        to.contract_weeks.is_none(),
        "Non-HR expects contract_weeks = null, got {:?}",
        to.contract_weeks
    );
    assert!(
        to.ist_per_soll_pct.is_none(),
        "Non-HR expects ist_per_soll_pct = null, got {:?}",
        to.ist_per_soll_pct
    );

    // Verifiziere zusaetzlich die JSON-Wire-Repraesentation: alle Felder muessen
    // physisch als `null` (bzw. weggelassen im serde-Default-Weg) im JSON auftauchen.
    assert!(
        body_str.contains("\"ist_per_contract_week\":null")
            && body_str.contains("\"contract_weeks\":null")
            && body_str.contains("\"ist_per_soll_pct\":null"),
        "Non-HR response body must serialize null-fields explicitly; got: {body_str}"
    );
}
