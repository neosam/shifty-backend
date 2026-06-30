---
phase: 33-special-days-ui-einstellungen
reviewed: 2026-06-30T13:30:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - dao_impl_sqlite/src/special_day.rs
  - rest/src/special_day.rs
  - service/src/special_days.rs
  - service_impl/src/special_days.rs
  - service_impl/src/test/special_days.rs
  - shifty-dioxus/src/page/settings.rs
  - shifty-dioxus/src/page/shiftplan.rs
findings:
  critical: 0
  warning: 1
  info: 1
  total: 2
status: issues_found
---

# Phase 33: Code Review Report (Iteration 2 — Re-Review)

**Reviewed:** 2026-06-30T13:30:00Z
**Depth:** standard
**Status:** issues_found (no blockers; the six prior WARNINGs are correctly fixed)

## Summary

This is a re-review after the code-fixer addressed the six WARNINGs from iteration 1.
All six fixes were verified line-by-line and, where possible, by execution:

- **WR-01 (DAO `.unwrap()` → `?`)** — `dao_impl_sqlite/src/special_day.rs:52` now reads
  `version: Uuid::from_slice(&entity.update_version)?`. Consistent with the sibling
  `id` field and the crate's `TryFrom` convention. Fixed.
- **WR-02/03/05 (service create validation + duplicate guard + tests)** — verified:
  - ShortDay-needs-`time_of_day` and `calendar_week` bounds (`1..=weeks_in_year(year)`)
    are checked before any clock/uuid/DAO call and accumulated into a single
    `ServiceError::ValidationError`. Variants are sensible (`InvalidValue`, `Duplicate`).
  - Duplicate guard uses `find_by_week(entity.year, entity.calendar_week)` (which is
    soft-delete-filtered) and rejects on a matching `day_of_week`. No false positive
    (empty list → create proceeds, covered by `test_create_success`) and no false
    negative (matching Monday/W1/2026 → `Duplicate`, covered by
    `test_create_rejects_duplicate`). Holiday `time_of_day` is normalized to `None`
    before persistence — correct.
  - **Basic-tier preserved:** `SpecialDayServiceImpl` still consumes only DAO +
    `PermissionService` + `ClockService` + `UuidService` (no domain service). Tier
    convention intact.
  - Ordering is correct: id/version nil-checks run before the duplicate `find_by_week`,
    so the early-return tests need no DAO expectations and mockall does not panic.
  - 6 new tests added; **all 12 special-day tests pass** and
    `cargo clippy -p service_impl --tests -- -D warnings` is **clean** (the
    `calendar_week < 1 || > max_week` form does not trip `manual_range_contains`).
- **WR-04 (settings year-picker follows ISO-week-year + form reset)** — on create
  success the handler sets `sd_year.set(iso_year)` then `sd_resource.restart()`; the
  resource closure reads `*sd_year.read()`, so the just-created entry is reloaded under
  its ISO year and cannot silently vanish across a year boundary. Form fields are reset
  to prevent an immediate exact-duplicate resubmit. Fixed.
- **WR-06 (shiftplan inline ShortDay dual-format parse + inline error)** —
  `shiftplan.rs:948-955` parses `HH:MM:SS` then falls back to `HH:MM`, and on failure
  sets `special_day_error` (inline span) instead of returning silently. Matches the
  Settings-card parse path (`settings.rs:386-395`) and the backend `time_of_day`
  format. Fixed.

No BLOCKER-class defect was found. Soft-delete (`WHERE deleted IS NULL`), the
`#[utoipa::path]` annotations on all four handlers + their inclusion in `ApiDoc`, and
the shiftplanner gate on `create`/`delete` are all intact. The two findings below are
a pre-existing UX/validation gap and a cosmetic label, neither introduced nor
regressed by the fixes.

## Warnings

### WR-01: Settings "Add" button is enabled with no day-type selected

**File:** `shifty-dioxus/src/page/settings.rs:357-358, 648`
**Issue:** `sd_form_valid = !sd_date_val.is_empty() && (sd_type_val != Some(ShortDay) || !sd_time_val.is_empty())`.
When no type is selected, `sd_type_val == None`, so `None != Some(ShortDay)` is `true`
and the form is considered valid as soon as a date is entered. The "Add" button
(`disabled: !sd_form_valid || ...`) is therefore clickable with no type chosen. The
create handler then hits `let Some(day_type) = ty else { sd_save_result.set(Some(false)); return; }`
(line 379) and surfaces the generic `SettingsSaveError` banner — a confusing failure
for what is really an incomplete form. (Pre-existing from iteration 1; not a regression,
but it is the one remaining real input-validation gap on this surface.)
**Fix:** Include an explicit "type selected" predicate in form validity:
```rust
let sd_form_valid = !sd_date_val.is_empty()
    && sd_type_val.is_some()
    && (sd_type_val != Some(SpecialDayTypeTO::ShortDay) || !sd_time_val.is_empty());
```

## Info

### IN-01: Misleading uuid-service process label in `delete`

**File:** `service_impl/src/special_days.rs:185`
**Issue:** `entity.version = self.uuid_service.new_uuid("booking-version");` uses a
copy-pasted `"booking-version"` seed label inside the special-day delete path. The
create path correctly uses `"special-day-service::create version"`. The label is only a
debug/seed tag and has no functional effect, but it is misleading when tracing uuid
generation for special days.
**Fix:**
```rust
entity.version = self.uuid_service.new_uuid("special-day-service::delete version");
```

---

_Reviewed: 2026-06-30T13:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
