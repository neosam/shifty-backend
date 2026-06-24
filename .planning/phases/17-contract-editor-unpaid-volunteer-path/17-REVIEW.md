---
phase: 17-contract-editor-unpaid-volunteer-path
reviewed: 2026-06-24T00:00:00Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - service_impl/src/billing_period_report.rs
  - service_impl/src/booking_information.rs
  - service_impl/src/reporting.rs
  - service_impl/src/test/booking_information.rs
  - service_impl/src/test/reporting_additive_merge.rs
  - shifty-dioxus/src/component/contract_modal.rs
  - shifty-dioxus/src/component/employees_list.rs
  - shifty-dioxus/src/i18n/en.rs
  - shifty-dioxus/src/i18n/de.rs
  - shifty-dioxus/src/i18n/cs.rs
  - shifty-dioxus/src/i18n/mod.rs
  - shifty-dioxus/src/loader.rs
  - shifty-dioxus/src/state/employee.rs
  - shifty-dioxus/src/state/employee_work_details.rs
findings:
  critical: 2
  warning: 4
  info: 3
  total: 9
status: issues_found
---

# Phase 17: Code Review Report

**Reviewed:** 2026-06-24
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Phase 17 makes `committed_voluntary` editable in the contract editor and adds a
show-all toggle to reveal unpaid volunteers in the employee list. The four phase
invariants (D-01 through D-06) are largely implemented correctly. The `is_paid`
gate is present and consistent across `billing_period_report.rs`, `reporting.rs`,
and `booking_information.rs`. The D-05 gate (`cap || expected == 0.0`) is symmetric
between service and frontend. The D-03 show_all toggle separates `inactive`
from `!is_paid` correctly. The `committed_voluntary` field round-trips faithfully
through both TryFrom directions in `employee_work_details.rs`.

Two blockers require attention before shipping:

1. **Division by zero in `weight_for_week`** (`reporting.rs`): if an
   `EmployeeWorkDetails` record has no workday booleans set to `true`, the
   denominator `all_potential_workdays` is 0, producing `NaN`/`inf` that
   propagates silently into every hours and balance field.

2. **Division by zero in `holiday_hours()` / `vacation_day_in_hours()`**
   (`employee_work_details.rs`, frontend): `days_per_week()` and
   `workdays_per_week` can both be 0 for the same reason — all weekday booleans
   false — causing `hours_per_holiday` and `hours_per_day` to be NaN in the
   `EmployeeWorkDetailsTO` emitted on save.

---

## Critical Issues

### CR-01: Division by zero in `weight_for_week` when no workdays are set

**File:** `service_impl/src/reporting.rs:959`

**Issue:** `potential_weekday_list()` collects only those weekday booleans that
are `true` on the `EmployeeWorkDetails` record. Its `.len()` is stored in
`all_potential_workdays` as `u8` and then used as the denominator:

```rust
let all_potential_workdays = workdays.len() as u8;          // line 912ish
// ...
let relation = num_potential_workdays_in_week as f32 / all_potential_workdays as f32;  // line 959
```

When all seven weekday flags are `false` (a record that is technically valid in
the schema), `all_potential_workdays == 0` and `relation` becomes `f32::NAN` (or
`INFINITY` if the numerator is nonzero). All downstream fields that multiply by
`relation` — `expected_hours`, `dynamic_working_hours_for_week`,
`workdays_per_week` — silently become NaN, which then contaminates billing
snapshots, reports, and carryover calculations.

The same denominator is also used later at line 966 for `workdays_per_week`.

**Fix:** Guard the denominator before the division:

```rust
if all_potential_workdays == 0 {
    // Record covers this week but has no workdays — skip (0 contribution).
    return (0.0, 0.0, 0, 0.0);
}
let relation = num_potential_workdays_in_week as f32 / all_potential_workdays as f32;
```

Alternatively, validate at contract-creation time that at least one weekday flag
is set, and return a service error if not.

---

### CR-02: Division by zero in `holiday_hours()` and `vacation_day_in_hours()` (frontend)

**File:** `shifty-dioxus/src/state/employee_work_details.rs:141-146`

**Issue:** Both helper methods divide by a count that can be zero:

```rust
pub fn vacation_day_in_hours(&self) -> f32 {
    self.expected_hours / self.workdays_per_week as f32   // panics/NaN if == 0
}
pub fn holiday_hours(&self) -> f32 {
    self.expected_hours / self.days_per_week() as f32     // panics/NaN if == 0
}
```

`days_per_week()` counts the `true` weekday flags; `workdays_per_week` is a
user-supplied field defaulting to 6 but can be set to 0. Both values are passed
directly into `EmployeeWorkDetailsTO.hours_per_day` and
`EmployeeWorkDetailsTO.hours_per_holiday` on every save:

```rust
// TryFrom<&EmployeeWorkDetails> for EmployeeWorkDetailsTO (line 224-226)
days_per_week: details.days_per_week(),
hours_per_day: details.vacation_day_in_hours(),    // NaN propagated to backend
hours_per_holiday: details.holiday_hours(),        // NaN propagated to backend
```

NaN values sent to the backend will be stored in the DB and corrupt subsequent
vacation and holiday calculations for that employee.

In Rust `f32` arithmetic NaN does not panic but it silently poisons every
downstream comparison and multiplication. On WASM targets the division produces
`NaN`; the HTTP payload will serialize as `null` or `null`-like depending on
serde config, which may cause a deserialization error on the backend or silently
store null.

**Fix:**

```rust
pub fn vacation_day_in_hours(&self) -> f32 {
    if self.workdays_per_week == 0 {
        return 0.0;
    }
    self.expected_hours / self.workdays_per_week as f32
}
pub fn holiday_hours(&self) -> f32 {
    let dpw = self.days_per_week();
    if dpw == 0 {
        return 0.0;
    }
    self.expected_hours / dpw as f32
}
```

---

## Warnings

### WR-01: Duplicate `extract_shiftplan_report_for_week` call in `get_summery_for_week`

**File:** `service_impl/src/booking_information.rs:339` and `490`

**Issue:** `extract_shiftplan_report_for_week` is invoked twice for the same
`(year, week)` pair within a single call to `get_summery_for_week`. The first
call at ~line 339 collects shiftplan hours to compute the raw `volunteer_hours`
sum; the second call at ~line 490 re-fetches the same shiftplan data for
`paid_hours` aggregation. Each call is a DAO round-trip (database query). Under
load this doubles the number of shiftplan queries for this endpoint.

More critically, because the results are fetched independently there is a narrow
TOCTOU window: if a booking is inserted or deleted between the two calls, the two
result sets may disagree, producing an internally inconsistent response
(e.g., `paid_hours` not matching the volunteer band data).

**Fix:** Fetch once, store the result in a local binding, and reuse it for both
computations.

---

### WR-02: `is_paid.unwrap_or(false)` silently excludes employees with `None` from billing and reports

**File:** `service_impl/src/billing_period_report.rs:328`, `service_impl/src/reporting.rs:163`, `service_impl/src/reporting.rs:888`

**Issue:** The guard `if !sales_person.is_paid.unwrap_or(false) { continue; }`
treats `Option::None` as "not paid", silently excluding from billing snapshots
and reports any employee whose `is_paid` column is NULL in the database (i.e.,
pre-Phase-17 rows created before the column was added, if the migration did not
set a NOT NULL DEFAULT TRUE).

If the migration added the column as nullable without a default, existing rows
have `is_paid = NULL` and they vanish from billing silently — no error, no log
entry, just missing data.

**Fix:** Verify the SQLite migration sets `is_paid` as `NOT NULL DEFAULT TRUE`
for existing rows. If it does, the `unwrap_or` is safe (defensive redundancy).
If it does not, either backfill `NULL` → `true` in a follow-up migration, or
change `unwrap_or(false)` to `unwrap_or(true)` so that ambiguous rows are
treated as paid (conservative fail-open for billing).

Add a comment at each `unwrap_or` call site documenting which invariant it relies
on.

---

### WR-03: Sparse `Volunteer` key in billing snapshots — inconsistent schema across rows

**File:** `service_impl/src/billing_period_report.rs:241`

**Issue:** The `Volunteer` value-type key is only inserted when
`report_delta.volunteer_hours != 0.0`:

```rust
if report_delta.volunteer_hours != 0.0 {
    // insert BillingPeriodValueType::Volunteer row
}
```

This means snapshots produced when an employee has no volunteer hours lack a
`Volunteer` row, while snapshots produced after they log even 1 hour of volunteer
work do have one. Any consumer that expects a complete and consistent schema
(e.g., a billing validator that diffs live vs. stored values) must handle both
sparse and dense representations, or it will misidentify "missing volunteer row"
as a diff.

The schema is version-pinned at 7 with no validator code change, so this is a
latent correctness risk for the next consumer that assumes the row must exist.

**Fix:** Either always insert the `Volunteer` row (with value 0.0 when hours are
zero), or document explicitly in the schema versioning comment that the key is
optional-absent when zero, so future validators know to treat absence as zero.

---

### WR-04: Missing Czech translations for `DynamicHourLabel` and text-template management keys (pre-existing, not introduced by Phase 17)

**File:** `shifty-dioxus/src/i18n/cs.rs`

**Issue:** Several keys present in `en.rs` and `de.rs` are absent from `cs.rs`:

- `DynamicHourLabel` (present in `en.rs` line 270, `de.rs` line 313; not found in `cs.rs`)
- `TextTemplateManagement`, `AddNewTemplate`, `EditTemplate`, `CustomReports`,
  `GenerateReport`, `SelectTemplate`, `GeneratingReport`, `GeneratedReport`,
  `CreateNewTemplate`, `Saving`, `TemplateName`

These are pre-existing omissions, not introduced by Phase 17, but Phase 17 adds
new keys to all three locales correctly (`CommittedVoluntaryLabel`,
`EmployeesShowAll`). The missing keys will cause the Czech locale to fall back to
whatever the i18n library returns for missing keys (typically the key identifier
or an empty string), degrading the Czech UI.

**Fix:** Add the missing translations to `cs.rs` for all absent keys. This is
a separate cleanup task from Phase 17 but should be tracked.

---

## Info

### IN-01: Dead branch in `cancel_label` in `ContractModal`

**File:** `shifty-dioxus/src/component/contract_modal.rs:77-81`

**Issue:** Both branches of the `cancel_label` expression produce the same string:

```rust
let cancel_label = if read_only {
    i18n.t(Key::Cancel)    // "Cancel"
} else {
    i18n.t(Key::Cancel)    // "Cancel" — identical
};
```

The original intent was likely to use a "Close" label for read-only mode (where
the button just dismisses the modal without discarding changes). The dead branch
is harmless at runtime but indicates incomplete implementation.

**Fix:** Add a `Close` i18n key and use it in the `read_only` branch, or remove
the branch and use `i18n.t(Key::Cancel)` directly.

---

### IN-02: `load_unpaid_volunteer_employees` always fetches even when `show_all=false`

**File:** `shifty-dioxus/src/component/employees_list.rs:78-80`, `shifty-dioxus/src/loader.rs:348-359`

**Issue:** The `unpaid_employees` resource is created unconditionally regardless
of the current `show_all` signal value. When `show_all=false` (the default), the
result is fetched and then ignored at the merge step. This means an extra
`GET /sales-person` request is fired on every component mount even when the user
never toggles the switch.

**Fix:** Gate the resource on `show_all`:

```rust
let unpaid_employees = use_resource(move || async move {
    if *show_all.read() {
        Some(loader::load_unpaid_volunteer_employees(config2.to_owned()).await)
    } else {
        None
    }
});
```

Or use `use_memo` / reactive resource with a dependency on `show_all`.

---

### IN-03: `tracing::info!` in hot path inside `hours_per_week` loop

**File:** `service_impl/src/reporting.rs:983`

**Issue:** There is a `tracing::info!` call inside the per-week iteration loop
of `hours_per_week`:

```rust
for week in from_week.iter_until(&to_week) {
    tracing::info!("Week: {}, Year: {}", week.week, week.year);
    // ...
    .inspect(|r: &&ShiftplanReportDay| {
        tracing::info!("{:?} - {:?}", r.to_date(), r);
    })
```

For a full-year report this fires 52+ times per employee. At `info` level in
production this generates substantial log volume and measurable overhead from
string formatting.

**Fix:** Downgrade to `tracing::debug!` or `tracing::trace!` so production
logging at `info` level is not polluted.

---

_Reviewed: 2026-06-24_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
