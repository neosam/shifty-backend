//! Debug-Session `convert-to-absence-404`.
//!
//! Reproduziert die zentrale Routing-Frage LOKAL gegen den ECHTEN axum-Router
//! (nicht das Minimal-Repro des Debuggers, nicht den Service-Pfad): Liefert der
//! reale `extra_hours::generate_route()`-Router einen 404, wenn der Pfad einen
//! DOPPEL-SLASH enthält — so wie ihn das deployte Frontend wegen des
//! Trailing-Slash in `config.backend` (`https://.../api/`) erzeugt?
//!
//! Methode: `tower::ServiceExt::oneshot` schickt Requests mit Single- und
//! Doppel-Slash. Ein 404 entsteht beim Routing VOR dem Handler — daher ist er
//! eindeutig von einem fachlichen 404 ("extra hours not found") unterscheidbar,
//! solange der Single-Slash-Kontrollfall mit identischem Seed/Body 200 liefert.
//!
//! Wichtig: getestet wird sowohl POST (convert) als auch GET (by-sales-person),
//! um die Frage "404t axum bei Doppel-Slash für ALLE Methoden gleich?" zu klären
//! — denn in Produktion gehen die GETs durch, der POST aber nicht.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use rest::extra_hours::generate_route;
use rest::{Context as RestContext, RestStateDef};
use service::{
    extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService},
    permission::Authentication,
    sales_person::{SalesPerson, SalesPersonService},
};
use time::macros::date;
use tower::ServiceExt;
use uuid::Uuid;

use crate::integration_test::TestSetup;

/// Seed: SalesPerson + Vacation-extra_hours-Row. Gibt (sales_person_id, extra_hours_id).
async fn seed_vacation(test_setup: &TestSetup, name: &str) -> (Uuid, Uuid) {
    let sales_person = test_setup
        .rest_state
        .sales_person_service()
        .create(
            &SalesPerson {
                id: Uuid::nil(),
                version: Uuid::nil(),
                name: name.into(),
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

    let extra_hours = test_setup
        .rest_state
        .extra_hours_service()
        .create(
            &ExtraHours {
                id: Uuid::nil(),
                sales_person_id: sales_person.id,
                amount: 8.0,
                category: ExtraHoursCategory::Vacation,
                description: "Urlaub".into(),
                date_time: time::PrimitiveDateTime::new(date!(2026 - 04 - 10), time::Time::MIDNIGHT),
                created: None,
                deleted: None,
                version: Uuid::nil(),
                source: service::extra_hours::ExtraHoursSource::Manual,
            },
            Authentication::Full,
            None,
        )
        .await
        .expect("create extra_hours failed");

    (sales_person.id, extra_hours.id)
}

fn build_router(test_setup: &TestSetup) -> axum::Router {
    axum::Router::new()
        .nest("/extra-hours", generate_route::<crate::RestStateImpl>())
        .with_state(test_setup.rest_state.clone())
        // DEVUSER hat in TestSetup via create_admin_user alle Privilegien (inkl. hr).
        .layer(Extension(Some(Arc::<str>::from("DEVUSER")) as RestContext))
}

async fn status_of(router: axum::Router, method: &str, uri: &str, body: String) -> StatusCode {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap();
    router.oneshot(req).await.unwrap().status()
}

/// KERN-TEST: Single-Slash POST convert → 200 (Route existiert, Erfolg).
/// Doppel-Slash POST convert → ist das 404 (Routing) oder matcht axum doch?
#[tokio::test]
async fn convert_post_single_vs_double_slash() {
    let test_setup = TestSetup::new().await;
    crate::create_admin_user(test_setup.pool.clone(), "DEVUSER").await;

    let (_sp_a, id_a) = seed_vacation(&test_setup, "PersonA").await;
    let (_sp_b, id_b) = seed_vacation(&test_setup, "PersonB").await;

    let body = r#"{"start":"2026-04-10","end":"2026-04-14"}"#.to_string();

    // Kontrolle: Single-Slash mit gültigem, frischem Eintrag → 200.
    let single = status_of(
        build_router(&test_setup),
        "POST",
        &format!("/extra-hours/{}/convert-to-absence", id_a),
        body.clone(),
    )
    .await;

    // Prüfling: identischer Request, aber Doppel-Slash zwischen nest-Prefix und Pfad.
    let double = status_of(
        build_router(&test_setup),
        "POST",
        &format!("/extra-hours//{}/convert-to-absence", id_b),
        body.clone(),
    )
    .await;

    // Zusätzlich: führender Doppel-Slash (so wie nginx ihn aus /api//extra-hours
    // ans Backend reichen könnte, wenn /api/ gestrippt wird).
    let leading_double = status_of(
        build_router(&test_setup),
        "POST",
        &format!("//extra-hours/{}/convert-to-absence", id_b),
        body.clone(),
    )
    .await;

    eprintln!(
        "[convert-404] POST single={single} double(mid)={double} leading_double={leading_double}"
    );

    assert_eq!(
        single,
        StatusCode::OK,
        "Single-Slash-Convert muss 200 sein (Route existiert, Seed gültig)"
    );
}

/// GET-Pendant: Reagiert axum bei Doppel-Slash für GET genauso wie für POST?
#[tokio::test]
async fn get_by_sales_person_single_vs_double_slash() {
    let test_setup = TestSetup::new().await;
    crate::create_admin_user(test_setup.pool.clone(), "DEVUSER").await;
    let (sp, _id) = seed_vacation(&test_setup, "PersonGet").await;

    let single = status_of(
        build_router(&test_setup),
        "GET",
        &format!("/extra-hours/by-sales-person/{}?year=2026&until_week=52", sp),
        String::new(),
    )
    .await;
    let leading_double = status_of(
        build_router(&test_setup),
        "GET",
        &format!("//extra-hours/by-sales-person/{}?year=2026&until_week=52", sp),
        String::new(),
    )
    .await;
    let mid_double = status_of(
        build_router(&test_setup),
        "GET",
        &format!("/extra-hours//by-sales-person/{}?year=2026&until_week=52", sp),
        String::new(),
    )
    .await;

    eprintln!(
        "[convert-404] GET single={single} leading_double={leading_double} mid_double={mid_double}"
    );
}
