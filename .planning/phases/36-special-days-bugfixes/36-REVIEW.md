---
phase: 36-special-days-bugfixes
reviewed: 2026-07-01T00:00:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - dao_impl_sqlite/src/special_day.rs
  - service_impl/src/special_days.rs
  - service_impl/src/test/special_days.rs
  - shifty-dioxus/src/component/form/inputs.rs
  - shifty-dioxus/src/page/settings.rs
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
---

# Phase 36: Code Review Report

**Reviewed:** 2026-07-01
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Reviewed the SDF-01 (same-date replacement) and SDF-02 (controlled `SelectInput`)
bugfix diff against base `96254f2`. The core fix is correct and well-tested:

- The service replacement branch (`service_impl/src/special_days.rs:143-163`)
  correctly looks up the active same-date row, preserves `id`/`created`, mutates
  only `day_type`/`time_of_day`/`version`, and performs a single UPDATE; the
  "no existing row" path still calls `create`; both directional switches
  (Holiday↔ShortDay) are unit-tested with a real non-nil existing id.
- The widened DAO `update` SQL (`dao_impl_sqlite/src/special_day.rs:184-197`) is
  correct, has no accidental widening (only `day_type`/`time_of_day` added, WHERE
  still `id = ?`), and is only called by `delete` (no-op writeback of loaded
  values) and the new replace path. The `.sqlx` offline cache was regenerated
  (`query-6cc953f8….json` holds the new statement, no stale entry) — CI
  `SQLX_OFFLINE=true` will not break.
- The `SelectInput` `value` prop is `#[props(!optional, default = None)]`, so all
  other callers (absences, slot_edit, billing_period_details, extra_hours_modal,
  text_template_management) stay uncontrolled — backward-compat is verified by
  `select_input_uncontrolled_when_no_value_prop`.
- No Signal write-borrow is held across an `.await` in `settings.rs`
  (`on_add_special_day` reads all signals into owned values before `spawn`, and
  only calls `.set()` inside the async block) — no already-borrowed panic risk.
- No new user-visible i18n `Key` was introduced by the diff.

Remaining findings are robustness / UX-consistency / test-quality issues; none
are blockers.

## Warnings

### WR-01: Read-then-write replacement is not atomic and no UNIQUE constraint backs same-date uniqueness

**File:** `service_impl/src/special_days.rs:143-163`, `migrations/sqlite/20241020064536_add-special-day-table.sql`
**Issue:** The code comment claims a "single atomic UPDATE", which is true only for
the write statement. The *decision* between create and replace is a non-transactional
read-modify-write: `find_by_week()` runs, then `update()`/`create()` runs, with no
transaction wrapping them (this service has no `Transaction` support at all). The
`special_day` table has **no UNIQUE constraint** on `(year, calendar_week, day_of_week)`.
Consequences:
- Two concurrent `create` calls for the same new date both observe "no existing row"
  and both `create` → two active rows for one date.
- If two active rows already exist for a date (from such a race, or historical data),
  the replacement `.find(...)` picks only the first and updates it, leaving the second
  active duplicate untouched — the "replace" is silently partial.
The single-UPDATE property does not protect the create-vs-replace branch selection.
**Fix:** Preferred: add a partial UNIQUE index enforcing one active row per date, e.g.
`CREATE UNIQUE INDEX ux_special_day_active ON special_day(year, calendar_week, day_of_week) WHERE deleted IS NULL;`
and handle the constraint violation as a replace/retry. At minimum, document that this
service is not concurrency-safe and that duplicate active rows are possible. (Largely
pre-existing, but the phase's "atomic replacement" wording overstates the guarantee.)

### WR-02: Duplicate hint is now semantically stale — create silently overwrites an existing entry

**File:** `shifty-dioxus/src/page/settings.rs:691-695` (hint), `service_impl/src/special_days.rs:138-163` (behavior)
**Issue:** Before this phase a same-date create was rejected (`ValidationError(Duplicate)`),
so the red inline hint "A special day already exists for this date." /
"An diesem Tag ist bereits ein Sondertag eingetragen." matched a real blocking
condition. After SDF-01 the backend *replaces* the existing row instead. The Add
button is gated only by `sd_form_valid` (settings.rs:684) and is **not** blocked by
`sd_is_duplicate`, so a user who sees the red "already exists" hint can still click
Add and will silently overwrite the prior entry (e.g. turn an existing Holiday into a
ShortDay) with no indication that an overwrite occurred. The messaging frames a normal
overwrite as an error/conflict and gives no confirmation of destructive replacement.
**Fix:** Reword the hint to reflect replace semantics (e.g. "A special day already
exists for this date and will be replaced.") in all three locales, or surface a
distinct "replaced existing entry" success state after the create resolves. Keep it a
non-blocking inline banner (consistent with the project's no-dialog preference), but
make the copy match the new behavior.

## Info

### IN-01: `test_create_replaces_same_date_entry` does not actually verify id preservation

**File:** `service_impl/src/test/special_days.rs:300-345`
**Issue:** The "existing" fixture is `make_entity()`, whose `id` is `Uuid::nil()`
(special_days.rs test:66). The replace path clones it, so the `withf` assertion checks
`entity.id == Uuid::nil()` (test:325) — i.e. it asserts a *nil* id on a supposedly
persisted active row, which is a semantically invalid state and the opposite of the
SDF-01 guarantee ("preserve the existing row's id"). The test passes only because the
fixture is degenerate; it would still pass if the service wrongly dropped the id. The
real guarantee is covered by `test_create_switches_holiday_to_shortday` /
`_shortday_to_holiday` (which use `existing_id()`), making this test redundant and
misleading.
**Fix:** Use `make_existing_entity(existing_id(), Holiday, None)` and assert
`entity.id == existing_id()` so the test locks the id-preservation invariant.

### IN-02: Switch tests cannot catch a `created`-timestamp regression

**File:** `service_impl/src/test/special_days.rs:414-478`, `483-552`
**Issue:** In both switch tests the existing entity's `created` is `fixed_created()`
and the clock's `date_time_now()` also returns `fixed_created()`. Since both values are
identical, the `expected_updated.created = fixed_created()` assertion cannot distinguish
"created preserved from the existing row" (the intended behavior) from "created
overwritten with the clock's now". A regression that overwrote `created` on replace
would go undetected here.
**Fix:** Give the existing entity a distinct `created` (e.g. 2025-06-01) different from
the clock's now, and assert the updated entity keeps the existing value.

### IN-03: Redundant active-row filter and discarded clock call on the replace path

**File:** `service_impl/src/special_days.rs:147-149`, `128`
**Issue:** (a) `.find(|e| e.day_of_week == entity.day_of_week && e.deleted.is_none())`
re-checks `deleted.is_none()`, but `find_by_week` already filters `WHERE deleted IS NULL`
(dao special_day.rs:96), so the second predicate is always true — harmless defense-in-depth
but effectively dead. (b) `special_day.created = Some(self.clock_service.date_time_now())`
(line 128) is computed unconditionally, then discarded on the replace path because
`updated` keeps the existing row's `created`. Minor wasted work / slightly misleading
control flow.
**Fix:** Optional cleanup — drop the redundant `deleted.is_none()` (or add a comment
that it is intentional defense-in-depth), and note that `created` is only meaningful on
the create branch. No behavior change required.

---

_Reviewed: 2026-07-01_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
