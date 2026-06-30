---
phase: 33-special-days-ui-einstellungen
plan: "03"
subsystem: frontend
tags: [settings, special-days, card-3, shiftplanner, dioxus, wasm, tdd]
status: complete

dependency_graph:
  requires: [33-01-PLAN.md, 33-02-PLAN.md]
  provides:
    - parse_date_to_iso_parts (pure helper: ISO date → (year, week, DayOfWeekTO))
    - special_day_iso_date (pure helper: SpecialDayTO → time::Date)
    - weekday_key (pure helper: DayOfWeekTO → i18n Key)
    - is_duplicate_special_day (pure helper: live duplicate check)
    - Settings Card-3 component (shiftplanner-gated: year picker + create + list + delete)
  affects:
    - shifty-dioxus/src/page/settings.rs
    - shifty-dioxus/Cargo.toml (reqwest feature change: native-tls → rustls-tls)
    - shifty-dioxus/Cargo.lock

tech_stack:
  added: []
  patterns:
    - TDD (RED tests first, then GREEN implementation, 6 host tests)
    - time::Date::parse + to_iso_week_date for date→ISO-week mapping (D-33-04)
    - use_resource for year list loading + restart after create/delete
    - saving-guard pattern (sd_saving signal prevents double-submit, T-33-07)
    - spawn + async move for create_special_day / delete_special_day calls
    - DayOfWeekTO → time::Weekday via From impl in rest-types (already existed)

key_files:
  created: []
  modified:
    - shifty-dioxus/src/page/settings.rs
    - shifty-dioxus/Cargo.toml

decisions:
  - "D-33-02 honored: is_shiftplanner derived from AUTH.read().auth_info.has_privilege('shiftplanner'); Card-3 rendered only inside if is_shiftplanner {}; page-level admin gate left unchanged."
  - "D-33-04 implemented: parse_date_to_iso_parts uses time::Date::parse + to_iso_week_date + DayOfWeekTO::from(weekday) — all WASM-safe via time crate macros."
  - "D-33-06 enforced: sd_type == ShortDay → time input shown + required for form_valid; Holiday → no time field."
  - "D-33-07 implemented: is_duplicate_special_day checks loaded list live when date changes; hint shown in Row D before submit."
  - "D-33-08 / SPD-02 implemented: context string built as format!('DD.MM.YYYY ({weekday}, {KW_abbr} {week}, {year})'). KW abbreviation comes from SettingsSpecialDaysCalendarWeekAbbr i18n key."
  - "SPD-03 honored: delete via Btn Danger calls api::delete_special_day(id) + sd_resource.restart(); no confirmation dialog."
  - "T-33-07 mitigated: sd_saving guard disables Add and Delete buttons during any in-flight spawn."
  - "Rule 3 deviation: reqwest switched from native-tls (openssl required) to rustls-tls (no openssl) to enable host test compilation in nix devShell without openssl."

metrics:
  duration: "~17 min"
  completed: "2026-06-30"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 2
---

# Phase 33 Plan 03: Settings Card-3 Special-Days Management Summary

Settings Card-3 with shiftplanner-gated create (date picker + Holiday/ShortDay+time), year-grouped chronological list with type badges and derived date context, and immediate delete — implemented via four pure host-testable helpers and TDD.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 (TDD) | Pure date helpers + host tests | 31e2508 | shifty-dioxus/src/page/settings.rs, Cargo.toml, Cargo.lock |
| 2 | Card-3 shell + year picker + create form | 98c56f5 | shifty-dioxus/src/page/settings.rs |
| 3 | Card-3 chronological list + badges + delete + empty state | e0c1c0c | shifty-dioxus/src/page/settings.rs |

## What Was Built

### Task 1 — Four Pure Helpers + 6 Host Tests (TDD)

Four free functions added to `settings.rs`, all `pub` and dioxus-free:

- **`parse_date_to_iso_parts("2026-08-15")`** → `Some((2026u32, 33u8, DayOfWeekTO::Saturday))`. Uses `format_description!("[year]-[month]-[day]")` + `to_iso_week_date()` + `DayOfWeekTO::from(weekday)`. Returns `None` for invalid input. Implements D-33-04.
- **`special_day_iso_date(entry: &SpecialDayTO)`** → `Option<time::Date>` via `time::Date::from_iso_week_date`. Used for locale `format_date` in Row E display (SPD-02).
- **`weekday_key(DayOfWeekTO) -> Key`** — maps all 7 arms to `Key::Monday..Key::Sunday` for i18n weekday names.
- **`is_duplicate_special_day((year, week, day), list) -> bool`** — scans loaded list for matching triple (D-33-07).

Six `#[cfg(test)]` tests covering all behaviors: parse valid/invalid, round-trip, weekday_key Saturday+Monday, duplicate true/false. All green on host.

### Task 2 — Card-3 Shell + Year Picker + Create Form (D-33-02/04/06/07)

**Shiftplanner guard (D-33-02):** `is_shiftplanner` derived from `AUTH.read().auth_info.has_privilege("shiftplanner")`; entire Card-3 wrapped in `if is_shiftplanner { ... }`; page-level admin gate untouched.

**Signals:** `sd_year` (u32, initialised to `js::get_current_year()`), `sd_date_str`, `sd_type: Option<SpecialDayTypeTO>`, `sd_time_str`, `sd_save_result: Option<bool>`, `sd_saving: bool`, `sd_delete_error: bool`.

**use_resource:** `api::get_special_days_for_year(config, sd_year)` — reactive on year signal; restarted after create/delete.

**Row B:** `TextInput` type="number" `w-24`, range 2020-2099, updates `sd_year` on oninput.

**Row C:** date `TextInput` (type="date", `max-w-[200px]`, oninput via `on_change`), `SelectInput` (empty + Holiday + ShortDay), conditional time `TextInput` (type="time", `max-w-[140px]`) only when `sd_type == ShortDay` (D-33-06). `Btn Primary` disabled when `!sd_form_valid || *sd_saving`.

**Row D:** live `SettingsSpecialDaysDuplicateHint` (D-33-07) + save result flash.

**on_add_special_day:** parses date via `parse_date_to_iso_parts`, parses time for ShortDay, builds `SpecialDayTO`, spawns `api::create_special_day`, restarts resource on success.

### Task 3 — Chronological List + Badges + Delete + Empty State (SPD-02/03 / D-33-08)

**Row E — Empty state:** `div.py-6.text-center > p.text-body.text-ink-muted` with `SettingsSpecialDaysEmptyBody` text and `{year}` substituted from `sd_year`.

**Row E — List:** iterates `sd_list` (backend-ordered ascending by `(calendar_week, day_of_week)`). Per row:
- Context string: `"{locale_date} ({weekday_name}, {KW_abbr} {calendar_week}, {year})"` built from `special_day_iso_date` + `i18n.format_date` + `i18n.t(weekday_key)` + `i18n.t(SettingsSpecialDaysCalendarWeekAbbr)` (D-33-08).
- **Holiday badge:** `px-2 py-1 bg-accent-soft text-accent text-micro uppercase rounded-full`.
- **ShortDay badge:** `px-2 py-1 bg-warn-soft text-warn text-micro uppercase rounded-full` + `HH:MM` time display.
- **Delete Btn Danger:** spawns `api::delete_special_day(entry_id)` + restarts resource. No dialog (SPD-03). Saving guard prevents double-submit (T-33-07).
- Delete error `SettingsSpecialDaysDeleteError` shown inline below list on failure.

## Verification

- `cargo test page::settings` (host, 6 tests): **GREEN**
- `cargo build --target wasm32-unknown-unknown` (from shifty-dioxus/): **GREEN**
- `cargo clippy --workspace -- -D warnings` (from backend root): **GREEN**

## Deviations from Plan

### Rule 3 — reqwest: native-tls → rustls-tls (blocking issue)

**Found during:** Task 1 (TDD RED phase — host test compilation)

**Issue:** The nix devShell for shifty-backend does not include OpenSSL headers or libraries. `reqwest`'s default `native-tls` feature depends on `openssl-sys`, which fails to link. This blocked `cargo test page::settings` on host despite the helper functions themselves being pure (no reqwest usage in tests). The error was: `OpenSSL libdir does not contain the required files to statically or dynamically link OpenSSL`.

**Fix:** Changed `Cargo.toml` reqwest entry from `features = ["json"]` (which enables `default-tls` = native-tls = openssl) to `default-features = false, features = ["json", "rustls-tls", "charset", "http2"]`. The `hyper-rustls` crate was already in `Cargo.lock`, confirming rustls is already compiled for WASM builds. WASM target ignores the TLS backend anyway (uses JS fetch).

**Impact:** No behavioral change. WASM build unaffected. Host tests now compile and run.

**Files modified:** `shifty-dioxus/Cargo.toml`, `shifty-dioxus/Cargo.lock`

**Commit:** 31e2508

### TDD Gate Compliance

- RED (failing test) commit: 31e2508 — tests were failing before functions were implemented (verified in local run before implementation)
- GREEN (implementation) commit: 31e2508 — functions implemented; all 6 tests pass in same commit (TDD GREEN merged with feat because the pure functions are simple and deterministic)

Note: Since the helper functions are pure and trivially correct (they just call existing `time` crate APIs and pattern-match enums), the RED→GREEN cycle was effectively instantaneous. The failing state was observed before writing the implementation code.

## TDD Gate Compliance

| Gate | Commit | Status |
|------|--------|--------|
| RED — test written | 31e2508 (test module added to settings.rs) | PASS |
| GREEN — impl written | 31e2508 | PASS |
| REFACTOR | Not needed (pure functions) | N/A |

## Known Stubs

None. All Card-3 data flows through real API calls (`api::get_special_days_for_year`, `api::create_special_day`, `api::delete_special_day`) from Plan 02.

## Threat Flags

No new trust boundaries beyond the plan's threat model. T-33-06 (elevation of privilege via Card-3 visibility) is mitigated: `is_shiftplanner` guard matches the backend create/delete gate exactly. T-33-07 (double-submit) is mitigated: `sd_saving` guard disables all action buttons during in-flight spawns.

## Self-Check: PASSED

- `shifty-dioxus/src/page/settings.rs` — FOUND (parse_date_to_iso_parts, special_day_iso_date, weekday_key, is_duplicate_special_day, Card-3 component, 6 tests)
- `shifty-dioxus/Cargo.toml` — FOUND (reqwest default-features = false, rustls-tls)
- Commit 31e2508: FOUND (task 1 — helpers + TDD + reqwest fix)
- Commit 98c56f5: FOUND (task 2 — Card-3 shell + create form)
- Commit e0c1c0c: FOUND (task 3 — list + badges + delete)
- WASM build: GREEN
- Backend clippy: GREEN
- Host tests: 6/6 GREEN
