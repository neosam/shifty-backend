---
phase: 29-urlaubs-balken-konsistenz-fe
reviewed: 2026-06-29T00:00:00Z
depth: standard
files_reviewed: 1
files_reviewed_list:
  - shifty-dioxus/src/page/absences.rs
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: issues_found
---

# Phase 29: Code Review Report

**Reviewed:** 2026-06-29
**Depth:** standard
**Files Reviewed:** 1 (`shifty-dioxus/src/page/absences.rs`)
**Status:** issues_found (2 × INFO)

## Summary

The change extracts a pure `compute_vacation_bar` helper, switches the bar numerator from `used_days` to `used_days + planned_days`, wires it into `PersonVacationCard`, and adds 5 unit tests. The logic is correct, the formula matches D-29-01, the clamp and zero-guard are sound, and Pitfall-5 (Tailwind static class literals) is properly preserved — the caller holds the `if low { … } else { … }` match with literal strings while the helper returns only a `bool`. No bugs, no security issues, no clippy-visible dead code.

Two minor INFO-level findings follow: one naming inconsistency carried forward from the old code, and one gap in the test suite's explicit documentation of the fix.

## Info

### IN-01: Binding `used_pct` is a misnomer — it now includes `planned_days`

**File:** `shifty-dioxus/src/page/absences.rs:882`
**Issue:** The destructuring in `PersonVacationCard`:
```rust
let (used_pct, low) = compute_vacation_bar(&props.balance);
```
inherits the name `used_pct` from the old `used_days`-only formula. The value now represents `(used_days + planned_days) / total`, which the function's own doc-comment correctly calls `fill_pct`. A reader maintaining this component in the future will see `bar_style = format!("width:{}%", used_pct)` and may wrongly assume the bar reflects only consumed days, missing the planned-days contribution.
**Fix:** Rename the binding to `fill_pct` (matching the doc-comment) or `consumed_pct`:
```rust
let (fill_pct, low) = compute_vacation_bar(&props.balance);
// ...
let bar_style = format!("width:{}%", fill_pct);
```

---

### IN-02: No test covers `planned_days > 0` in the sub-100% (non-overdraw) case

**File:** `shifty-dioxus/src/page/absences.rs:4051–4092`
**Issue:** Of the 5 tests, tests 2–5 all set `planned_days = 0.0`. Test 1 does exercise `planned_days = 13` but only at the overdraw boundary (fill clamps to 100). There is no test that verifies `planned_days` shifts the bar in the normal case, e.g. `used=6, planned=6, total=18 → fill=66%`. This gap means the test suite documents the overdraw clamp and the zero-guard well, but does not explicitly show a reader that a non-zero `planned_days` below the overdraw threshold increases the fill above what `used_days` alone would give.

Note: test 1 already proves the fix is present (if `planned_days` were ignored, fill would be 33%, not 100%), so this is a documentation-quality nit rather than a reliability gap.

**Fix:** Add one test (can reuse `vacation_balance_fixture`):
```rust
/// D-29-01: planned_days contributes to fill in the sub-100% case.
#[test]
fn compute_vacation_bar_planned_adds_to_fill() {
    // entitled 18, carryover 0 → total 18; used 6, planned 6 → 12/18*100 = 66.
    let b = vacation_balance_fixture(18.0, 0, 6.0, 6.0, 6.0);
    let (fill, low) = compute_vacation_bar(&b);
    assert_eq!(fill, 66, "(6+6)/18*100 = 66.66 truncates to 66");
    assert!(!low, "remaining=6 is above threshold");
}
```

---

_Reviewed: 2026-06-29_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
