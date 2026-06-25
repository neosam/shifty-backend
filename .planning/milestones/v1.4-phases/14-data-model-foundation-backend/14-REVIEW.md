---
phase: 14-data-model-foundation-backend
reviewed: 2026-06-23T11:55:40Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - dao/src/employee_work_details.rs
  - dao_impl_sqlite/src/employee_work_details.rs
  - service/src/employee_work_details.rs
  - service_impl/src/employee_work_details.rs
  - service_impl/src/reporting.rs
  - rest-types/src/lib.rs
  - migrations/sqlite/20260623120000_add-committed-voluntary-to-employee-work-details.sql
  - service_impl/src/test/employee_work_details.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: issues_found
---

# Phase 14: Code Review Report

**Reviewed:** 2026-06-23T11:55:40Z
**Depth:** standard
**Files Reviewed:** 8
**Status:** issues_found (info-level only — no blockers, no warnings)

## Summary

Phase 14 threads a new time-versioned, numeric field `committed_voluntary: f32`
through every backend layer of the `EmployeeWorkDetails` aggregate (SQLite
migration → `EmployeeWorkDetailsDb` row + `EmployeeWorkDetailsEntity` →
`EmployeeWorkDetails` service struct → `EmployeeWorkDetailsTO`), plus the
update carry-forward line and a `committed_voluntary_for_calendar_week` SUM
helper in `reporting.rs`. The field is intentionally inert this phase.

I reviewed the four high-risk areas called out in the brief and found the
implementation correct on all of them:

- **Type/cast correctness.** The DB row carries `committed_voluntary: f64`
  (matching SQLite `REAL`), and is converted with `as f32` in
  `TryFrom<&EmployeeWorkDetailsDb>` (dao_impl_sqlite line 65) and back with
  `as f64` in both `create` (line 334) and `update` (line 430) before binding.
  No bool-coercion (`!= 0`) was applied to the numeric column — that pattern is
  correctly reserved for the genuine boolean columns. The SQLx offline cache was
  regenerated; the new SELECT entries describe `committed_voluntary` as
  `type_info: Float`, `nullable: false`, which matches the non-`Option<f64>`
  struct field. `NOT NULL DEFAULT 0` in the migration is what makes the
  non-nullable mapping sound for pre-existing rows.

- **Carry-forward / no silent default-reset.** The service `update()`
  (service_impl/src/employee_work_details.rs line 249) copies
  `entity.committed_voluntary = employee_work_details.committed_voluntary` onto
  the freshly-loaded entity, mirroring the `cap_planned_hours_to_expected` line
  exactly. The `with_from_date`/`with_to_date` spreads use `..self.clone()`,
  which carries the field forward automatically. Regression test
  `update_propagates_committed_voluntary_to_dao` (CVC-02) pins this.

- **SQLx query/binding correctness.** `committed_voluntary` appears in the
  correct ordinal position in all four SELECTs, the INSERT column list +
  placeholder count (24 columns / 24 `?`) match, and the UPDATE SET clause binds
  it in argument order. `cargo build --workspace` under `SQLX_OFFLINE=true`
  compiles clean, which validates the macro bindings against the cache.

- **SUM vs `.any()`.** `committed_voluntary_for_calendar_week` uses
  `.map(|wh| wh.committed_voluntary).sum()` — not the boolean `.any()` pattern
  used by the cap flag. Four tests pin SUM semantics (two-row sum, single, no
  active row, empty slice). All 5 phase tests pass.

Remaining findings are info-level quality observations only.

## Info

### IN-01: Pre-existing `unwrap()` on `sales_person_id` parse in row conversion

**File:** `dao_impl_sqlite/src/employee_work_details.rs:50`
**Issue:** `Uuid::from_slice(working_hours.sales_person_id.as_ref()).unwrap()`
will panic if a row ever contains a malformed `sales_person_id`, whereas the
adjacent `id` parse (line 49) uses `?` and propagates a `DaoError`. This is
pre-existing (not introduced by Phase 14) and not reachable through the new
field, so it is informational. Flagging because the new column conversion
sits two lines below it.
**Fix:** Replace `.unwrap()` with `?` for consistency with the surrounding
fallible conversions:
```rust
sales_person_id: Uuid::from_slice(working_hours.sales_person_id.as_ref())?,
```

### IN-02: CVC-02 test's stale value coincides with the type default

**File:** `service_impl/src/test/employee_work_details.rs:104,144`
**Issue:** The "stale persisted" value in `update_propagates_committed_voluntary_to_dao`
is `0.0`, which is also the `f32` default. The test still proves propagation
(it asserts the DAO receives `2.5`, the input), but a regression that wrote a
hardcoded `0.0` instead of the loaded value would be invisible to this test
because both happen to be `0.0`. The companion cap-flag test has the same
shape. Low value-add to change, but a non-zero stale value (e.g. `1.0`) would
make the test strictly stronger.
**Fix:** Use a non-default stale value so "wrote the stale loaded value" and
"wrote the type default" are distinguishable:
```rust
.returning(move |_, _| Ok(entity_with_cap_and_committed(id, version, false, 1.0)));
// ...and construct the input from the same stale 1.0 baseline, then set 2.5.
```

### IN-03: Whitespace artifact in `find_by_sales_person_id` SELECT

**File:** `dao_impl_sqlite/src/employee_work_details.rs:237-239`
**Issue:** The `find_by_sales_person_id` query has three blank lines between
`vacation_days,` and `created,`, whereas the sibling queries (`all`,
`find_by_id`, `find_for_week`) use a single blank line. Cosmetic only; no
behavioral effect.
**Fix:** Collapse the extra blank lines to match the other three SELECTs.

---

_Reviewed: 2026-06-23T11:55:40Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
