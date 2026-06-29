---
phase: 31-abwesenheit-nicht-verf-gbar-markierung-im-schichtplan-fe
reviewed: 2026-06-29T00:00:00Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - shifty-dioxus/src/service/absence_marker.rs
  - shifty-dioxus/src/service/mod.rs
  - shifty-dioxus/src/page/shiftplan.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: resolved
resolution_note: "No blockers/warnings. IN-02 (missing symmetric end-partial-overlap test) FIXED: added vacation_full_partial_overlap_end_yields_fri_sat_sun → absence_marker tests now 8/8 green. IN-01 (guard semantically redundant — loader returns the week-independent full absence list) addressed with a clarifying comment at the guard site (guard kept for Phase-30 pattern consistency; per-week filtering happens in the pure helper at render time). IN-03 (extend may yield duplicate Weekdays) ACCEPTED as no-effect: WeekView uses .contains(), so duplicates do not affect rendering; dedup deferred as speculative future-consumer hardening."
---

# Phase 31: Code Review Report

**Reviewed:** 2026-06-29
**Depth:** standard
**Files Reviewed:** 3
**Status:** issues_found (INFO only — no blockers or warnings)

## Summary

Phase 31 adds the absence-to-discourage-marker bridge: a pure helper
(`absence_marker.rs`) maps a person's `AbsencePeriod` list to `Vec<Weekday>` for
the displayed week, and `shiftplan.rs` wires it up with a `person_absences`
signal, a guarded loader closure, four trigger call-sites, and a union-merge into
`discourage_weekdays`.

**Category fidelity (SC2):** `category_triggers_marker` uses an exhaustive match
returning `true` for all three current variants (Vacation, SickLeave,
UnpaidLeave).  The only additional filter is `day_fraction == Full`, which mirrors
`shiftplan_edit.rs:538` exactly.  Zero drift.

**Date-range overlap:** `from_date <= day && day <= to_date` with `day` derived as
`week_monday + Duration::days(offset)` for offset in `0u8..=6` is correct — both
endpoints are inclusive, the Monday anchor is pinned to the rendered week via
`time::Date::from_iso_week_date(..., Monday)`, and the offset covers Mon(0) through
Sun(6) without gap or overrun.

**Trigger completeness:** `reload_absence_days` is called at exactly the four
correct sites (initial load, NextWeek, PreviousWeek, UpdateSalesPerson).
`ToggleAvailability` is correctly absent — absence data is unaffected by
availability toggles.

**Helper purity and tests:** The helper has no Dioxus/browser dependencies and is
exercised by 7 `cargo test` cases covering the main scenarios.

Three INFO-level observations follow; none indicate incorrect behavior.

---

## Info

### IN-01: Guard in `reload_absence_days` is semantically misleading (harmless)

**File:** `shifty-dioxus/src/page/shiftplan.rs:396-405`

**Issue:** The `is_current_selection((req_year, req_week), *SELECTED_WEEK.read())`
guard was designed for week-specific data (e.g., `reload_unavailable_days` fetches
data for a single year/week).  `load_absence_periods_by_sales_person` has no
year/week parameter — it returns all absences for the person regardless of week.
So any successful response is valid for any displayed week; the guard can only
suppress a result that is already correct.

In practice the guard is harmless: (a) the same-week request is always fired
immediately after the week switch, so a suppressed old-request result is promptly
replaced; (b) `absence_periods_to_discourage_days` filters to the current week
inside the render, so no stale dates would ever appear even if the guard were
absent.  The comment "silently drop the result if the user has already switched to
a different week while this request was in flight" reads as if the data is
week-scoped when it is not.

**Fix:** Either remove the guard from `reload_absence_days` (correct and simpler),
or update the comment to note that the guard here prevents an unnecessary overwrite
rather than preventing stale data:

```rust
// The absence list is person-scoped (not week-scoped), so any in-flight
// result is still valid.  We skip the write only to avoid a redundant
// signal update if the user navigated away before this response arrived;
// the new week's own request will set the value immediately after.
if is_current_selection((req_year, req_week), *SELECTED_WEEK.read()) {
    *person_absences.write() = result;
}
```

---

### IN-02: Test suite covers start-partial-overlap but not end-partial-overlap

**File:** `shifty-dioxus/src/service/absence_marker.rs:180-194`

**Issue:** `vacation_full_partial_overlap_start_yields_mon_tue_wed` verifies an
absence that begins before Monday and ends mid-week.  There is no symmetric test
for an absence that begins mid-week and ends after Sunday.  The code handles this
correctly (`day <= ap.to_date` is satisfied for all days from `from_date` through
Sunday when `to_date > Sunday`), but the coverage gap means a future refactor that
accidentally flips the `to_date` comparator would not be caught.

**Fix:** Add a complementary test:

```rust
/// Vacation, Full, starting Wednesday and ending next Friday →
/// Wednesday, Thursday, Friday, Saturday, Sunday (partial overlap at week end).
#[test]
fn vacation_full_partial_overlap_end_yields_wed_to_sun() {
    let absences = [make_absence(
        AbsenceCategory::Vacation,
        date!(2026 - 07 - 01), // Wednesday of test week
        date!(2026 - 07 - 10), // Next Friday (past Sunday)
        DayFraction::Full,
    )];
    let result = absence_periods_to_discourage_days(&absences, MONDAY);
    assert_eq!(
        result,
        vec![
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
            Weekday::Sunday,
        ],
        "Partial overlap at week end should yield Wed–Sun"
    );
}
```

---

### IN-03: Union-merge produces duplicate `Weekday` entries when unavailable and absent on the same day

**File:** `shifty-dioxus/src/page/shiftplan.rs:1172-1185`

**Issue:** The `extend` merge appends absence-derived weekdays to the
`unavailable_days`-derived Vec without deduplication.  If an employee has both an
`SalesPersonUnavailable` entry and an `AbsencePeriod` covering the same weekday,
that `Weekday` appears twice in the resulting `Rc<[Weekday]>`.  `WeekView` consumes
the list via `.contains()` (line 1322), so the duplicate has no visible effect on
rendering.  The Vec is slightly larger than necessary.

**Fix:** If this ever becomes observable (e.g., a future consumer iterates the list
to emit one badge per entry), deduplicate before converting:

```rust
discourage.sort_unstable_by_key(|w| w.num_from_monday());
discourage.dedup();
discourage.into()
```

For now the current code is functionally correct; the fix can wait until there is
an actual consumer that would be affected.

---

_Reviewed: 2026-06-29_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
