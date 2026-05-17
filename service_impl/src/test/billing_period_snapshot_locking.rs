//! Locking-Test-Suite fuer Snapshot-Schema-Versionierung (SNAP-02).
//!
//! Diese Datei verhindert stille Drift zwischen der Snapshot-Berechnung und
//! `CURRENT_SNAPSHOT_SCHEMA_VERSION`. Siehe CLAUDE.md § "Billing Period Snapshot
//! Schema Versioning" fuer die Bump-Trigger-Regeln.
//!
//! - `test_snapshot_schema_version_pinned`: erwartet 4 (Phase-8.3-Bump —
//!   day_fraction=Half halbiert die effektive Soll-Stundenzahl in
//!   `derive_hours_for_range`, was die Vacation/SickLeave/UnpaidLeave-
//!   value_type-Computation aendert).
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
        CURRENT_SNAPSHOT_SCHEMA_VERSION, 4,
        "CURRENT_SNAPSHOT_SCHEMA_VERSION muss 4 sein nach Phase-8.3-Bump \
         (Trigger: AbsencePeriod.day_fraction = Half halbiert die effektive \
         Soll-Stundenzahl pro Tag in derive_hours_for_range; betrifft die \
         Vacation/SickLeave/UnpaidLeave-derived value_types hours + days). \
         Siehe CLAUDE.md § Snapshot Schema Versioning."
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
