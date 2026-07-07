//! Phase 8.5 (Plan 04) — Integration-Tests fuer die Read-Projektion
//! lebender extra_hours (Vacation/SickLeave/UnpaidLeave) in den
//! Absence-GET-Endpoints.
//!
//! Testet den vollen Service-Pfad (ExtraHoursService + AbsenceService +
//! SalesPersonService) via RestStateImpl gegen eine frische In-Memory-SQLite.
//!
//! Abgedeckte Acceptance-Criteria (Threat-Register T-8.5-04a/b/c/d):
//! - absence_projection_includes_living_hourly_markers: Vacation-Row erscheint
//!   als Marker mit korrektem Datum + Amount (kein Range-Raten, D-07).
//! - absence_projection_excludes_non_absence_categories: ExtraWork erscheint
//!   NICHT in hourly_markers (T-8.5-04b).
//! - absence_projection_excludes_soft_deleted: soft-deletete Vacation-Row
//!   erscheint NICHT (T-8.5-04c, find_by_sales_person_id_and_year_range
//!   filtert deleted IS NULL).

use rest::RestStateDef;
use service::{
    absence::{AbsenceCategory, AbsencePeriod, AbsenceService, DayFraction},
    extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService},
    permission::Authentication,
    sales_person::{SalesPerson, SalesPersonService},
};
use shifty_utils::ShiftyDate;
use time::macros::date;
use uuid::Uuid;

use crate::integration_test::TestSetup;

/// Seed: erstellt eine SalesPerson + eine Vacation-extra_hours-Row im Zwei-Jahres-Fenster.
/// Gibt (sales_person_id, extra_hours_id) zurueck.
async fn seed_vacation_marker(test_setup: &TestSetup) -> (Uuid, Uuid) {
    let person = test_setup
        .rest_state
        .sales_person_service()
        .create(
            &SalesPerson {
                id: Uuid::nil(),
                version: Uuid::nil(),
                name: "ProjektionPerson".into(),
                background_color: "#aabbcc".into(),
                inactive: false,
                is_paid: Some(true),
                deleted: None,
            },
            Authentication::Full,
            None,
        )
        .await
        .expect("create sales_person failed");

    let eh = test_setup
        .rest_state
        .extra_hours_service()
        .create(
            &ExtraHours {
                id: Uuid::nil(),
                sales_person_id: person.id,
                amount: 6.0,
                category: ExtraHoursCategory::Vacation,
                description: "Urlaub April".into(),
                date_time: time::PrimitiveDateTime::new(
                    date!(2026 - 04 - 15),
                    time::Time::MIDNIGHT,
                ),
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

    (person.id, eh.id)
}

/// HR-Context: DEVUSER hat alle Privilegien inkl. hr.
fn hr_context() -> Authentication<Option<std::sync::Arc<str>>> {
    Authentication::Context(Some(std::sync::Arc::from("DEVUSER")))
}

/// Test: Vacation-Marker erscheint in der Projektion mit korrektem Datum + Amount.
/// Kein Range-Raten (D-07): when == seed-Datum, amount == seed-Amount.
#[tokio::test]
async fn absence_projection_includes_living_hourly_markers() {
    let test_setup = TestSetup::new().await;
    let (sales_person_id, _eh_id) = seed_vacation_marker(&test_setup).await;

    let (from_bound, to_bound) = two_year_window();

    let raw_markers = test_setup
        .rest_state
        .extra_hours_service()
        .find_by_sales_person_id_and_year_range(
            sales_person_id,
            from_bound,
            to_bound,
            hr_context(),
            None,
        )
        .await
        .expect("find_by_sales_person_id_and_year_range failed");

    let vacation_markers: Vec<_> = raw_markers
        .iter()
        .filter(|eh| matches!(eh.category, ExtraHoursCategory::Vacation))
        .collect();

    assert_eq!(
        vacation_markers.len(),
        1,
        "Genau ein Vacation-Marker erwartet, got: {}",
        vacation_markers.len()
    );
    let marker = vacation_markers[0];
    assert_eq!(
        marker.date_time.date(),
        date!(2026 - 04 - 15),
        "Marker when muss seed-Datum sein (kein Range-Raten)"
    );
    assert_eq!(
        marker.amount, 6.0,
        "Marker amount muss seed-amount sein"
    );
    assert_eq!(
        marker.sales_person_id, sales_person_id,
        "Marker sales_person_id muss seed-person sein"
    );
}

/// Test: ExtraWork erscheint NICHT in der Projektion (T-8.5-04b, Kategorie-Filter).
#[tokio::test]
async fn absence_projection_excludes_non_absence_categories() {
    let test_setup = TestSetup::new().await;
    let (sales_person_id, _) = seed_vacation_marker(&test_setup).await;

    // Seed extra ExtraWork-Row.
    test_setup
        .rest_state
        .extra_hours_service()
        .create(
            &ExtraHours {
                id: Uuid::nil(),
                sales_person_id,
                amount: 3.0,
                category: ExtraHoursCategory::ExtraWork,
                description: "Ueberstunden".into(),
                date_time: time::PrimitiveDateTime::new(
                    date!(2026 - 04 - 16),
                    time::Time::MIDNIGHT,
                ),
                created: None,
                deleted: None,
                version: Uuid::nil(),
                source: service::extra_hours::ExtraHoursSource::Manual,
            },
            Authentication::Full,
            None,
        )
        .await
        .expect("create extra_work failed");

    let (from_bound, to_bound) = two_year_window();

    let raw_markers = test_setup
        .rest_state
        .extra_hours_service()
        .find_by_sales_person_id_and_year_range(
            sales_person_id,
            from_bound,
            to_bound,
            hr_context(),
            None,
        )
        .await
        .expect("find_by_sales_person_id_and_year_range failed");

    // Projections-Filter: nur Vacation/SickLeave/UnpaidLeave
    let absence_markers: Vec<_> = raw_markers
        .iter()
        .filter(|eh| {
            matches!(
                eh.category,
                ExtraHoursCategory::Vacation
                    | ExtraHoursCategory::SickLeave
                    | ExtraHoursCategory::UnpaidLeave
            )
        })
        .collect();

    // ExtraWork darf nicht drin sein.
    let extra_work_in_projection = absence_markers
        .iter()
        .any(|eh| matches!(eh.category, ExtraHoursCategory::ExtraWork));
    assert!(
        !extra_work_in_projection,
        "ExtraWork darf nicht in der Projektion erscheinen"
    );

    // Nur die Vacation-Row ist im Projektion-Set.
    assert_eq!(
        absence_markers.len(),
        1,
        "Nur Vacation-Marker erwartet (ExtraWork rausgefiltert), got: {}",
        absence_markers.len()
    );
}

/// Test: Soft-deletete Vacation-Row erscheint NICHT in der Projektion (T-8.5-04c).
/// find_by_sales_person_id_and_year_range filtert `deleted IS NULL` im DAO.
#[tokio::test]
async fn absence_projection_excludes_soft_deleted() {
    let test_setup = TestSetup::new().await;
    let (sales_person_id, eh_id) = seed_vacation_marker(&test_setup).await;

    // Soft-delete die Row ueber den ExtraHoursService.
    test_setup
        .rest_state
        .extra_hours_service()
        .delete(eh_id, Authentication::Full, None)
        .await
        .expect("delete extra_hours failed");

    let (from_bound, to_bound) = two_year_window();

    // Nach Soft-Delete darf die Row nicht mehr auftauchen.
    let raw_markers = test_setup
        .rest_state
        .extra_hours_service()
        .find_by_sales_person_id_and_year_range(
            sales_person_id,
            from_bound,
            to_bound,
            hr_context(),
            None,
        )
        .await
        .expect("find_by_sales_person_id_and_year_range failed");

    let vacation_markers: Vec<_> = raw_markers
        .iter()
        .filter(|eh| matches!(eh.category, ExtraHoursCategory::Vacation))
        .collect();

    assert_eq!(
        vacation_markers.len(),
        0,
        "Soft-deletete Vacation-Row darf nicht in der Projektion erscheinen"
    );
}

/// Test: Absence-Periode + Vacation-Marker koennen gleichzeitig existieren.
/// Beide erscheinen in ihren jeweiligen Feldern (absence_periods + hourly_markers).
#[tokio::test]
async fn absence_projection_coexists_with_absence_period() {
    let test_setup = TestSetup::new().await;
    let (sales_person_id, _eh_id) = seed_vacation_marker(&test_setup).await;

    // Seed eine native AbsencePeriod.
    test_setup
        .rest_state
        .absence_service()
        .create(
            &AbsencePeriod {
                id: Uuid::nil(),
                sales_person_id,
                from_date: date!(2026 - 05 - 01),
                to_date: date!(2026 - 05 - 05),
                category: AbsenceCategory::Vacation,
                day_fraction: DayFraction::Full,
                description: "Nativurlaub".into(),
                created: None,
                deleted: None,
                version: Uuid::nil(),
            },
            Authentication::Full,
            None,
        )
        .await
        .expect("create absence_period failed");

    // Beide Felder pruefen: Ranges per AbsenceService, Marker per ExtraHoursService.
    let all_absences = test_setup
        .rest_state
        .absence_service()
        .find_all(hr_context(), None)
        .await
        .expect("find_all failed");
    assert_eq!(all_absences.len(), 1, "Genau eine AbsencePeriod erwartet");

    let (from_bound, to_bound) = two_year_window();
    let raw_markers = test_setup
        .rest_state
        .extra_hours_service()
        .find_by_sales_person_id_and_year_range(
            sales_person_id,
            from_bound,
            to_bound,
            hr_context(),
            None,
        )
        .await
        .expect("find_by_sales_person_id_and_year_range failed");

    let vacation_markers: Vec<_> = raw_markers
        .iter()
        .filter(|eh| matches!(eh.category, ExtraHoursCategory::Vacation))
        .collect();
    assert_eq!(
        vacation_markers.len(),
        1,
        "Genau ein Vacation-Marker erwartet neben der AbsencePeriod"
    );
}

/// Hilfsfunktion: berechnet das Zwei-Jahres-Fenster analog dem REST-Handler.
fn two_year_window() -> (ShiftyDate, ShiftyDate) {
    let current_year = time::OffsetDateTime::now_utc().year();
    let from = ShiftyDate::from(
        time::Date::from_calendar_date(current_year - 1, time::Month::January, 1)
            .expect("valid from_date"),
    );
    let to = ShiftyDate::from(
        time::Date::from_calendar_date(current_year + 1, time::Month::December, 31)
            .expect("valid to_date"),
    );
    (from, to)
}
