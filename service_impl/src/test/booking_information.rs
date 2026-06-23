//! D-05 two-band fixture suite for committed_voluntary_hours (Band 1) + volunteer_hours (Band 2).
//!
//! Tests the pure per-person surplus helper `volunteer_surplus_above_committed` and the
//! two-band FORMULA-B decomposition:
//!   Band 1 = cap-gated Σ_person committed
//!   Band 2 = Σ_person max(actual_p − committed_p, 0)
//!   No-double-count invariant: committed + surplus(actual, committed) = max(committed, actual)
//!
//! Also contains a regression test asserting CURRENT_SNAPSHOT_SCHEMA_VERSION stays 7
//! (D-01 / CVC-05: Phase 15 touches no persisted BillingPeriodValueType).

use crate::booking_information::{volunteer_surplus_above_committed, volunteer_surplus_band2};

/// Epsilon helper — never use == for f32 comparisons.
fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}

// ─── Per-person Band-2 surplus fixtures ──────────────────────────────────────

#[test]
fn cvc04_band2_surplus() {
    // Person: cap=true, c=5, a=7 → surplus = max(7−5, 0) = 2; Band 1 = 5, Band 2 = 2
    let committed: f32 = 5.0;
    let actual: f32 = 7.0;

    let band2 = volunteer_surplus_above_committed(actual, committed);
    let band1 = committed; // cap=true, so full pledge counts

    assert!(approx(band2, 2.0), "Band 2 surplus expected 2.0, got {band2}");
    assert!(approx(band1, 5.0), "Band 1 expected 5.0, got {band1}");

    // No-double-count invariant: committed + surplus = max(committed, actual)
    assert!(
        approx(band1 + band2, committed.max(actual)),
        "invariant failed: {} + {} != max({}, {})",
        band1,
        band2,
        committed,
        actual
    );
}

#[test]
fn cvc04_band2_pledge_covers() {
    // Person: cap=true, c=5, a=3 → surplus = max(3−5, 0) = 0 (floor at 0)
    let committed: f32 = 5.0;
    let actual: f32 = 3.0;

    let band2 = volunteer_surplus_above_committed(actual, committed);
    let band1 = committed;

    assert!(approx(band2, 0.0), "Band 2 expected 0.0 (pledge covers), got {band2}");
    assert!(approx(band1, 5.0), "Band 1 expected 5.0, got {band1}");

    // No-double-count invariant: band sum = max(committed, actual) = 5
    assert!(
        approx(band1 + band2, committed.max(actual)),
        "invariant failed: {} + {} != max({}, {})",
        band1,
        band2,
        committed,
        actual
    );
}

#[test]
fn cvc04_boundary_equal() {
    // Person: c=5, a=5 → boundary committed==actual → surplus = 0
    let committed: f32 = 5.0;
    let actual: f32 = 5.0;

    let band2 = volunteer_surplus_above_committed(actual, committed);
    let band1 = committed;

    assert!(approx(band2, 0.0), "Band 2 expected 0.0 at boundary, got {band2}");
    assert!(approx(band1, 5.0), "Band 1 expected 5.0, got {band1}");

    // No-double-count invariant
    assert!(
        approx(band1 + band2, committed.max(actual)),
        "invariant failed: {} + {} != {}",
        band1,
        band2,
        committed.max(actual)
    );
}

#[test]
fn cvc04_zero_actual() {
    // Person: c=5, a=0 (forward-looking pledge with no actuals yet)
    // Band 1 = 5 (pledge still counts), Band 2 = 0 (no surplus)
    let committed: f32 = 5.0;
    let actual: f32 = 0.0;

    let band2 = volunteer_surplus_above_committed(actual, committed);
    let band1 = committed;

    assert!(approx(band2, 0.0), "Band 2 expected 0.0 (no actuals), got {band2}");
    assert!(approx(band1, 5.0), "Band 1 expected 5.0, got {band1}");

    // No-double-count invariant: max(5, 0) = 5
    assert!(
        approx(band1 + band2, committed.max(actual)),
        "invariant failed: {} + {} != {}",
        band1,
        band2,
        committed.max(actual)
    );
}

#[test]
fn cvc06_cap_false_zero() {
    // Person: is_paid=false, cap=FALSE, c=5 (pre-gated to 0.0), a=7
    //
    // The cap-gating to committed=0.0 is enforced UPSTREAM by the per-row
    // `.filter(cap_planned_hours_to_expected)` in get_weekly_summary (CVC-06).
    // At the helper level a cap=false person presents committed=0.0.
    //
    // Band 1 = 0.0 (cap=false → committed contributes 0 to Band 1)
    // Band 2 = volunteer_surplus_above_committed(7.0, 0.0) = 7.0 (full actual)
    let committed_after_cap_gate: f32 = 0.0; // cap=false → gated to 0 upstream
    let actual: f32 = 7.0;

    let band2 = volunteer_surplus_above_committed(actual, committed_after_cap_gate);
    let band1 = committed_after_cap_gate; // 0.0

    assert!(approx(band2, 7.0), "Band 2 expected 7.0 (full actual, cap=false), got {band2}");
    assert!(approx(band1, 0.0), "Band 1 expected 0.0 (cap=false), got {band1}");
}

// ─── Multi-week aggregation fixture ──────────────────────────────────────────

#[test]
fn cvc04_multi_week_sum() {
    // Same person across two weeks, surplus per week BEFORE summing (per-week-before-sum rule).
    //   W1: c=5, a=7 → Band1_W1=5, Band2_W1=max(7-5,0)=2
    //   W2: c=5, a=3 → Band1_W2=5, Band2_W2=max(3-5,0)=0
    //   Total Band 1 = 5+5 = 10, Total Band 2 = 2+0 = 2
    //
    // NOTE: per-week-before-sum is mandatory — max is nonlinear. Taking max AFTER
    // summing would give max(Σ,Σ) = max(10,10) = 10 which is WRONG for Band 1 sum context.
    let committed_w1: f32 = 5.0;
    let actual_w1: f32 = 7.0;
    let committed_w2: f32 = 5.0;
    let actual_w2: f32 = 3.0;

    let band1_w1 = committed_w1;
    let band2_w1 = volunteer_surplus_above_committed(actual_w1, committed_w1);
    let band1_w2 = committed_w2;
    let band2_w2 = volunteer_surplus_above_committed(actual_w2, committed_w2);

    let total_band1 = band1_w1 + band1_w2;
    let total_band2 = band2_w1 + band2_w2;

    assert!(approx(total_band1, 10.0), "Total Band 1 expected 10.0, got {total_band1}");
    assert!(approx(total_band2, 2.0), "Total Band 2 expected 2.0, got {total_band2}");
}

// ─── Multi-person aggregation fixture ────────────────────────────────────────

#[test]
fn cvc04_multi_person() {
    // FORMULA B (per-person two-band decomposition):
    //   Person A (cap=true, c=5, a=0) → B1+=5, B2+=surplus(0,5)=0
    //   Person B (cap=false → committed gated to 0 upstream, a=3) → B1+=0, B2+=surplus(3,0)=3
    //   Total: committed_voluntary_hours=5, volunteer_hours=3, grand_total=8
    //
    // This is FORMULA B (per-person): total = Σ_person max(committed_p, actual_p) = max(5,0)+max(0,3) = 5+3 = 8
    // FORMULA A (wrong): max(Σcommitted, Σactual) = max(5,3) = 5 — superseded by User clarification D-05.
    let committed_a: f32 = 5.0; // cap=true
    let actual_a: f32 = 0.0;
    let committed_b: f32 = 0.0; // cap=false → gated to 0 upstream
    let actual_b: f32 = 3.0;

    let band1_a = committed_a; // cap=true: pledge counts fully
    let band2_a = volunteer_surplus_above_committed(actual_a, committed_a);
    let band1_b = committed_b; // cap=false: 0
    let band2_b = volunteer_surplus_above_committed(actual_b, committed_b);

    let total_band1 = band1_a + band1_b;
    let total_band2 = band2_a + band2_b;
    let grand_total = total_band1 + total_band2;

    assert!(
        approx(total_band1, 5.0),
        "committed_voluntary_hours (Band 1) expected 5.0, got {total_band1}"
    );
    assert!(
        approx(total_band2, 3.0),
        "volunteer_hours (Band 2) expected 3.0, got {total_band2}"
    );
    assert!(
        approx(grand_total, 8.0),
        "grand total (FORMULA B) expected 8.0 (not 5.0/Formula A), got {grand_total}"
    );
}

// ─── Paid + capped: Band 2 stays zero ────────────────────────────────────────

#[test]
fn cvc04_paid_capped_band2_zero() {
    // is_paid=true, cap=true, c=5 — in Achse B actual_volunteer=0 for paid persons
    // (Research Option b: paid persons' actual_vol = 0 in the year-view path).
    // Band 1 = 5 (pledge counted), Band 2 = surplus(0, 5) = 0.
    let committed: f32 = 5.0;
    let actual_volunteer: f32 = 0.0; // paid person: no volunteer hours in Achse B

    let band2 = volunteer_surplus_above_committed(actual_volunteer, committed);
    let band1 = committed;

    assert!(approx(band2, 0.0), "Band 2 expected 0.0 for paid+capped person, got {band2}");
    assert!(approx(band1, 5.0), "Band 1 expected 5.0, got {band1}");
}

// ─── Backward-compatibility: committed=0 ─────────────────────────────────────

#[test]
fn cvc06_committed_zero_backward_compat() {
    // When committed=0 for every person, Band 2 = Σ actual = the pre-v1.4 volunteer_hours.
    // Band 1 = 0.0 (no pledge).
    // This proves the two-band decomposition is a strict superset of pre-v1.4 behavior.
    let actuals = [2.0_f32, 4.0, 1.0];

    let band2: f32 = actuals
        .iter()
        .map(|&a| volunteer_surplus_above_committed(a, 0.0))
        .sum();
    let band1: f32 = 0.0; // committed=0 for all persons

    let plain_sum: f32 = actuals.iter().sum();

    assert!(
        approx(band2, plain_sum),
        "committed=0 ⇒ volunteer_hours should be bit-identical to pre-v1.4 Σactual={plain_sum}, got {band2}"
    );
    assert!(
        approx(band2, 7.0),
        "Band 2 expected 7.0 (2+4+1), got {band2}"
    );
    assert!(
        approx(band1, 0.0),
        "Band 1 expected 0.0 (no pledges), got {band1}"
    );
}

// ─── CR-01 regression: multi-day per-person aggregation ──────────────────────
//
// These tests exercise `volunteer_surplus_band2`, which MUST aggregate per-day
// shiftplan-report rows into per-person weekly totals BEFORE applying max(actual−committed,0).
// The buggy per-day form (mapping volunteer_surplus_above_committed over each row) would
// yield 0.0 for the single-person case below where neither single day exceeds committed.

#[test]
fn cvc04_multi_day_single_person() {
    // CR-01 regression: one volunteer, committed=5, Mon 3.0h + Tue 4.0h (weekly=7.0).
    //
    // Correct (per-person weekly aggregation first):
    //   weekly_actual = 3.0 + 4.0 = 7.0
    //   surplus = max(7.0 − 5.0, 0) = 2.0
    //
    // Buggy per-day form (CR-01):
    //   max(3.0 − 5.0, 0) + max(4.0 − 5.0, 0) = 0.0 + 0.0 = 0.0  ← WRONG
    let person = uuid::Uuid::new_v4();
    let per_day = vec![(person, 3.0_f32), (person, 4.0_f32)];
    let result = volunteer_surplus_band2(per_day, |_| 5.0_f32);
    assert!(
        (result - 2.0_f32).abs() < 0.001,
        "CR-01: expected per-person weekly surplus 2.0, got {result} \
         (buggy per-day form would yield 0.0)"
    );
}

#[test]
fn cvc04_multi_day_multi_person() {
    // CR-01 regression, multi-person variant.
    //   Person A: committed=5, Mon 3.0h + Tue 4.0h (weekly=7.0) → surplus = max(7−5,0) = 2.0
    //   Person B: committed=0 (cap=false), Mon 1.5h + Wed 1.5h (weekly=3.0) → surplus = 3.0
    //   Total Band 2 = 2.0 + 3.0 = 5.0
    //
    // Buggy per-day form (CR-01) would compute:
    //   Person A: max(3−5,0) + max(4−5,0) = 0
    //   Person B: max(1.5−0,0) + max(1.5−0,0) = 3.0
    //   Total (buggy) = 3.0  ← WRONG (under-counts A's surplus)
    let person_a = uuid::Uuid::new_v4();
    let person_b = uuid::Uuid::new_v4();
    let per_day = vec![
        (person_a, 3.0_f32), // Mon
        (person_a, 4.0_f32), // Tue
        (person_b, 1.5_f32), // Mon
        (person_b, 1.5_f32), // Wed
    ];
    let result = volunteer_surplus_band2(per_day, |id| {
        if id == person_a {
            5.0_f32
        } else {
            0.0_f32 // cap=false → 0
        }
    });
    assert!(
        (result - 5.0_f32).abs() < 0.001,
        "CR-01 multi-person: expected total Band 2 = 5.0, got {result} \
         (buggy per-day form would yield 3.0, missing person A's 2.0 surplus)"
    );
}

// ─── No-bump regression test (D-01 / CVC-05) ─────────────────────────────────

#[test]
fn snapshot_schema_version_unchanged_at_7() {
    // D-01 / CVC-05: Phase 15 touches NO persisted value_type — committed_voluntary_hours
    // (Band 1) and the reduced volunteer_hours (Band 2) are Achse-B (year-view) only and
    // are never read by billing_period_report.rs. Therefore the CLAUDE.md bump rule is
    // NOT triggered; version stays 7.
    assert_eq!(
        crate::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION,
        7
    );
}
