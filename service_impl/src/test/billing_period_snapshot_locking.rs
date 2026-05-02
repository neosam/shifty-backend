//! Locking-Test-Suite fuer Snapshot-Schema-Versionierung (SNAP-02).
//!
//! Diese Datei verhindert stille Drift zwischen der Snapshot-Berechnung und
//! `CURRENT_SNAPSHOT_SCHEMA_VERSION`. Siehe CLAUDE.md § "Billing Period Snapshot
//! Schema Versioning" fuer die Bump-Trigger-Regeln.
//!
//! - `test_snapshot_schema_version_pinned`: erwartet 3 (Wave-2-Forcing —
//!   pre-Wave-2 ROT, post-Wave-2 GRUEN).
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
        CURRENT_SNAPSHOT_SCHEMA_VERSION, 3,
        "CURRENT_SNAPSHOT_SCHEMA_VERSION muss 3 sein nach Phase-2-Bump \
         (Trigger: neuer value_type UnpaidLeave + AbsencePeriod-derived \
         Vacation/Sick/UnpaidLeave). \
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
            BillingPeriodValueType::Holiday => {}
            BillingPeriodValueType::Volunteer => {}
            BillingPeriodValueType::VacationDays => {}
            BillingPeriodValueType::VacationEntitlement => {}
            BillingPeriodValueType::CustomExtraHours(_) => {}
            // POST-WAVE-2: BillingPeriodValueType::UnpaidLeave => {}
        }
    }
    // Compiler-only Check: ensure_locked wird nie aufgerufen.
    let _ = ensure_locked;
}
