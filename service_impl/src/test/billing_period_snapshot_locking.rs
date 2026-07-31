//! Locking-Test-Suite fuer Snapshot-Schema-Versionierung (SNAP-02).
//!
//! Diese Datei verhindert stille Drift zwischen der Snapshot-Berechnung und
//! `CURRENT_SNAPSHOT_SCHEMA_VERSION`. Siehe CLAUDE.md § "Billing Period Snapshot
//! Schema Versioning" fuer die Bump-Trigger-Regeln.
//!
//! - `test_snapshot_schema_version_pinned`: erwartet 13 (Phase 57 VOL-ACCT-04 /
//!   D-57-10 — der Marker-Filter ist aus den Read-Aggregaten entfernt, Rebooking-
//!   ExtraHours fliessen jetzt in die persistierten value_types Overall/Balance/
//!   ExtraWork/Volunteer ein; Bump-Trigger "Change the input set the computation
//!   reads from").
//! - `test_billing_period_value_type_surface_locked`: Compile-Error wenn
//!   Enum-Variante hinzu/weg ohne Test-Update.

use service::billing_period::BillingPeriodValueType;

use crate::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION;

/// LOCKING TEST -- DO NOT NAIVELY UPDATE.
///
/// If this test fails after a code change:
///   - Did you intentionally change the snapshot computation?
///   - If yes, you MUST also bump CURRENT_SNAPSHOT_SCHEMA_VERSION
///     in service_impl/src/billing_period_report.rs.
///   - See CLAUDE.md § "Billing Period Snapshot Schema Versioning"
///     for the bump-trigger rules.
#[test]
fn test_snapshot_schema_version_pinned() {
    assert_eq!(
        CURRENT_SNAPSHOT_SCHEMA_VERSION, 13,
        "CURRENT_SNAPSHOT_SCHEMA_VERSION muss 13 sein nach Phase 57 (VOL-ACCT-04 / D-57-10): \
         der pauschale source==Manual-Vorfilter ist aus den Read-Aggregaten von \
         ReportingService::get_report_for_employee_range entfernt (D-57-06). Rebooking-Paare \
         fliessen dadurch in die persistierten value_types ein — das +h ExtraWork-Bein hebt \
         Overall, Balance und ExtraWork, das -h VolunteerWork-Bein senkt Volunteer. \
         Laut CLAUDE.md (Snapshot Schema Versioning: 'Change the input set the computation \
         reads from') ist ein Bump Pflicht: eine frische Neuberechnung derselben Periode \
         weicht von einem v12-Snapshot ab, und ohne Bump waere diese Drift von einem echten \
         Datenfehler nicht unterscheidbar. \
         HINWEIS: ExpectedHours, VacationEntitlement und VacationDays sind NICHT betroffen — \
         rebook() schreibt nur die Kategorien VolunteerWork und ExtraWork. \
         Beleg: test/billing_period_straddling.rs::test_straddling_snapshot_rebooking_shifts_values. \
         Siehe service_impl/src/billing_period_report.rs § CURRENT_SNAPSHOT_SCHEMA_VERSION."
    );
}

/// LOCKING TEST -- DO NOT NAIVELY UPDATE.
///
/// Wenn der Compiler hier eine fehlende Variante meldet:
/// bist du sicher, dass du nicht `CURRENT_SNAPSHOT_SCHEMA_VERSION` bumpen wolltest?
/// Siehe CLAUDE.md § "Billing Period Snapshot Schema Versioning".
#[test]
fn test_billing_period_value_type_surface_locked() {
    fn ensure_locked(value_type: &BillingPeriodValueType) {
        // Wenn der Compiler hier `non-exhaustive patterns` meldet: Wave 2 hat
        // die `UnpaidLeave`-Variante hinzugefuegt — fuege einen neuen Arm
        // `BillingPeriodValueType::UnpaidLeave => {}` ein und behalte den
        // Snapshot-Bump 2 -> 3 in `service_impl/src/billing_period_report.rs:37`.
        match value_type {
            BillingPeriodValueType::Overall => {}
            BillingPeriodValueType::Balance => {}
            BillingPeriodValueType::ExpectedHours => {}
            BillingPeriodValueType::ExtraWork => {}
            BillingPeriodValueType::VacationHours => {}
            BillingPeriodValueType::SickLeave => {}
            BillingPeriodValueType::UnpaidLeave => {}
            BillingPeriodValueType::Holiday => {}
            BillingPeriodValueType::Volunteer => {}
            BillingPeriodValueType::VacationDays => {}
            BillingPeriodValueType::VacationEntitlement => {}
            BillingPeriodValueType::CustomExtraHours(_) => {}
        }
    }
    // Compiler-only Check: ensure_locked wird nie aufgerufen.
    let _ = ensure_locked;
}
