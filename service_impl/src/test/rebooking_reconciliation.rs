//! Phase 55 Plan 01 — Unit-Tests fuer `RebookingReconciliationService`.
//!
//! - `predicate_truth_table`: D-55-01 Grenzfall-Coverage fuer
//!   `alert_predicate(balance, voluntary_ist, cap_active) -> bool`.
//! - `proposed_hours`: D-55-03 Test-Matrix fuer
//!   `proposed_rebooking_hours(balance, voluntary_ist) -> f32`.
//! - `service`: Service-Impl-Tests (Task 3) — HR-Gate, atomarer Doppel-
//!   Eintrag, state-conditional UPDATE, Direction-Symmetrie.
//! - `reporting_filter`: VOL-ACCT-03 Rebooking-Neutralitaets-Filter im
//!   Reporting-Aggregat (Task 3).

pub mod predicate_truth_table {
    use service::rebooking_reconciliation::alert_predicate;

    #[test]
    fn cap_inactive_never_alerts() {
        // cap_active=false ist der harte Kill-Switch.
        assert!(!alert_predicate(-10.0, 5.0, false));
    }

    #[test]
    fn balance_just_above_threshold_no_alert() {
        // Grenzfall D-55-01: -0.49 > -0.5 → kein Alert.
        assert!(!alert_predicate(-0.49, 5.0, true));
    }

    #[test]
    fn balance_at_threshold_triggers_alert() {
        // Grenzfall D-55-01: -0.5 <= -0.5 → Alert.
        assert!(alert_predicate(-0.5, 5.0, true));
    }

    #[test]
    fn balance_below_threshold_triggers_alert() {
        // -0.51 < -0.5 → Alert.
        assert!(alert_predicate(-0.51, 5.0, true));
    }

    #[test]
    fn zero_voluntary_never_alerts() {
        // Kein Ehrenamt → nichts zu rebooken → kein Alert.
        assert!(!alert_predicate(-5.0, 0.0, true));
    }

    #[test]
    fn zero_balance_never_alerts() {
        // Kein Defizit → kein Alert.
        assert!(!alert_predicate(0.0, 5.0, true));
    }

    #[test]
    fn negative_voluntary_guarded() {
        // Negativ-Guard (Datenkorruption): `voluntary_ist > 0.0`.
        assert!(!alert_predicate(-5.0, -0.1, true));
    }
}

pub mod proposed_hours {
    use service::rebooking_reconciliation::proposed_rebooking_hours;

    #[test]
    fn balance_deficit_greater_than_voluntary_is_capped_by_voluntary() {
        // |−10| = 10 > 5 → min = 5.
        assert_eq!(proposed_rebooking_hours(-10.0, 5.0), 5.0);
    }

    #[test]
    fn voluntary_greater_than_balance_deficit_is_capped_by_balance() {
        // |−3| = 3 < 10 → min = 3.
        assert_eq!(proposed_rebooking_hours(-3.0, 10.0), 3.0);
    }

    #[test]
    fn positive_balance_uses_abs() {
        // |0.4| = 0.4 < 5 → 0.4 (Alert-Predicate greift hier nicht, separat).
        assert_eq!(proposed_rebooking_hours(0.4, 5.0), 0.4);
    }

    #[test]
    fn zero_balance_yields_zero_proposal() {
        assert_eq!(proposed_rebooking_hours(0.0, 5.0), 0.0);
    }

    #[test]
    fn zero_voluntary_yields_zero_proposal() {
        assert_eq!(proposed_rebooking_hours(-5.0, 0.0), 0.0);
    }
}
