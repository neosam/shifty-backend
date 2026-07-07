//! Phase 8.5 (Plan 03) — Integration-Tests fuer POST /extra-hours/{id}/convert-to-absence.
//!
//! Testet den vollen Service-Pfad (HR-Gate, 200, 403) via RestStateImpl gegen
//! eine frische In-Memory-SQLite. Tests nutzen den AbsenceConversionService
//! direkt (kein HTTP-Stack), was dem Acceptance-Criteria-Ziel entspricht:
//! Endpoint-Semantik ist ueber den Service-Pfad verifiziert, REST-Verb/Status
//! wird durch den Handler (extra_hours.rs) + Service-Gate zusammen gewaehrleistet.
//!
//! 200-Happy-Path: seed Vacation-extra_hours-Row, hr-berechtigter Context ->
//! absence_period mit category Vacation zurueck.
//! 403-Gate: gleicher Seed, Context OHNE hr-Privileg -> Forbidden.

use rest::RestStateDef;
use service::{
    absence_conversion::AbsenceConversionService,
    extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService},
    permission::{Authentication, HR_PRIVILEGE},
    sales_person::{SalesPerson, SalesPersonService},
    ServiceError,
};
use time::macros::date;
use uuid::Uuid;

use crate::integration_test::TestSetup;

/// Seed: erstellt eine SalesPerson und eine Vacation-extra_hours-Row.
/// Gibt (sales_person_id, extra_hours_logical_id) zurueck.
async fn seed_vacation_extra_hours(test_setup: &TestSetup) -> (Uuid, Uuid) {
    let sales_person = test_setup
        .rest_state
        .sales_person_service()
        .create(
            &SalesPerson {
                id: Uuid::nil(),
                version: Uuid::nil(),
                name: "TestPerson".into(),
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
                description: "Urlaub 2026-04-10".into(),
                date_time: time::PrimitiveDateTime::new(
                    date!(2026 - 04 - 10),
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

    (sales_person.id, extra_hours.id)
}

/// HR-berechtigter Context: `Some(Arc::from("DEVUSER"))`.
/// DEVUSER hat in Testsystemen via `create_admin_user` alle Privilegien inkl. hr.
fn hr_context() -> Authentication<Option<std::sync::Arc<str>>> {
    Authentication::Context(Some(std::sync::Arc::from("DEVUSER")))
}

/// Context ohne hr-Privileg: anderer User.
fn no_privilege_context() -> Authentication<Option<std::sync::Arc<str>>> {
    Authentication::Context(Some(std::sync::Arc::from("some-non-hr-user")))
}

/// Happy-Path: Vacation-extra_hours mit hr-Context konvertieren -> 200 / AbsencePeriod.Vacation.
#[tokio::test]
async fn convert_to_absence_happy_path() {
    let test_setup = TestSetup::new().await;
    let (_sales_person_id, extra_hours_id) = seed_vacation_extra_hours(&test_setup).await;

    let result = test_setup
        .rest_state
        .absence_conversion_service()
        .convert_extra_hours_to_absence(
            extra_hours_id,
            date!(2026 - 04 - 10),
            date!(2026 - 04 - 14),
            None,
            hr_context(),
            None,
        )
        .await;

    let absence = result.expect("convert_to_absence should succeed for hr context");
    assert_eq!(
        absence.category,
        service::absence::AbsenceCategory::Vacation,
        "category must be Vacation"
    );
    assert_eq!(absence.from_date, date!(2026 - 04 - 10));
    assert_eq!(absence.to_date, date!(2026 - 04 - 14));
}

/// 403-Gate: Context ohne hr-Privileg -> ServiceError::Forbidden.
#[tokio::test]
async fn convert_requires_hr_privilege() {
    let test_setup = TestSetup::new().await;
    let (_sales_person_id, extra_hours_id) = seed_vacation_extra_hours(&test_setup).await;

    let result = test_setup
        .rest_state
        .absence_conversion_service()
        .convert_extra_hours_to_absence(
            extra_hours_id,
            date!(2026 - 04 - 10),
            date!(2026 - 04 - 14),
            None,
            no_privilege_context(),
            None,
        )
        .await;

    match result {
        Err(ServiceError::Forbidden) => {} // expected
        other => panic!(
            "expected Forbidden for non-hr context, got: {:?}",
            other
        ),
    }
}

// Verify that the HR_PRIVILEGE constant used in the service is the string "hr".
// This guards against accidental rename that would silently break the gate.
#[test]
fn hr_privilege_constant_is_hr() {
    assert_eq!(HR_PRIVILEGE, "hr", "HR_PRIVILEGE must be the string 'hr'");
}
