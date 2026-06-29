//! Locking-Test-Suite fuer Snapshot-Schema-Versionierung (SNAP-02).
//!
//! Diese Datei verhindert stille Drift zwischen der Snapshot-Berechnung und
//! `CURRENT_SNAPSHOT_SCHEMA_VERSION`. Siehe CLAUDE.md § "Billing Period Snapshot
//! Schema Versioning" fuer die Bump-Trigger-Regeln.
//!
//! - `test_snapshot_schema_version_pinned`: erwartet 12 (Phase 28 VAC-OFFSET-01 /
//!   D-28-05 — off-by-one fix in `vacation_days_for_year` changes the persisted
//!   `BillingPeriodValueType::VacationEntitlement`; `VacationDays` is unaffected).
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
        CURRENT_SNAPSHOT_SCHEMA_VERSION, 12,
        "CURRENT_SNAPSHOT_SCHEMA_VERSION muss 12 sein nach Phase 28 (VAC-OFFSET-01 / D-28-05): \
         off-by-one fix in EmployeeWorkDetails::vacation_days_for_year — die Jahres-START- \
         Proration zieht jetzt nur die Tage STRIKT vor dem Vertragsstart ab (ein 1.1.-Start \
         zieht 0 ab statt ~1/365 des Jahresanspruchs). Das aendert den persistierten \
         BillingPeriodValueType::VacationEntitlement (reporting.rs:853 <- :803). \
         Laut CLAUDE.md (Snapshot Schema Versioning: 'Change the computation that produces \
         an existing value_type') ist ein Bump Pflicht. \
         HINWEIS: VacationDays (genommener Urlaub) ist NICHT betroffen — nur VacationEntitlement. \
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
