---
phase: 33-special-days-ui-einstellungen
plan: "02"
subsystem: frontend
tags: [api, i18n, special-days, wasm, dioxus]
status: complete

dependency_graph:
  requires: [33-01-PLAN.md]
  provides: [api::get_special_days_for_year, api::create_special_day, api::delete_special_day, i18n::Key (18 variants)]
  affects: [shifty-dioxus/src/api.rs, shifty-dioxus/src/i18n/mod.rs, shifty-dioxus/src/i18n/de.rs, shifty-dioxus/src/i18n/en.rs, shifty-dioxus/src/i18n/cs.rs]

tech_stack:
  added: []
  patterns: [reqwest GET/POST/DELETE, Uuid::nil() guard on POST, WASM i18n exhaustive-Key compile gate]

key_files:
  created: []
  modified:
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs

decisions:
  - "D-33-05 honored: get_special_days_for_year targets /special-days/for-year/{year} matching Plan 01's backend route."
  - "T-33-04 mitigation applied: create_special_day forces body.id and body.version to Uuid::nil() before POST."
  - "18 keys added (not 17 as PATTERNS.md stated) to match 33-UI-SPEC § 'New i18n Keys' exactly."

metrics:
  duration: "~15 min"
  completed: "2026-06-30"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 5
---

# Phase 33 Plan 02: FE Foundation — API Functions + i18n Keys Summary

Three Dioxus frontend API functions and 18 i18n Key variants with de/en/cs translations,
providing the shared foundation for Plans 03 (Settings Card-3) and 04 (Shiftplan dropdown).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add three Special-Day FE API functions | e4cd664 | shifty-dioxus/src/api.rs |
| 2 | Add 18 i18n keys + de/en/cs translations | 67a2462 | shifty-dioxus/src/i18n/mod.rs, de.rs, en.rs, cs.rs |

## What Was Built

### Task 1 — Three API Functions (api.rs)

Added after `get_special_days_for_week` (line ~984):

- **`get_special_days_for_year(config, year)`** — GET `/special-days/for-year/{year}`, returns `Rc<[SpecialDayTO]>`. Matches D-33-05 backend route from Plan 01.
- **`create_special_day(config, mut body)`** — POST `/special-days/` with `body.id = Uuid::nil()` and `body.version = Uuid::nil()` forced before send. Returns the created `SpecialDayTO`. Implements T-33-04 mitigation.
- **`delete_special_day(config, id)`** — DELETE `/special-days/{id}`, errors on non-2xx. Matches SPD-03.

No new imports were needed: `SpecialDayTO`, `Uuid`, `Config`, and `Rc` were already in scope.

### Task 2 — 18 i18n Keys (mod.rs + locale files)

Added 18 Key variants to the `Key` enum in mod.rs after `NavToEmployeeReport`:

**Settings Card-3 keys (14):**
`SettingsSpecialDaysSectionLabel`, `SettingsSpecialDaysSectionDescription`, `SettingsSpecialDaysYearLabel`,
`SettingsSpecialDaysDateLabel`, `SettingsSpecialDaysTypeLabel`, `SettingsSpecialDaysTypeHoliday`,
`SettingsSpecialDaysTypeShortDay`, `SettingsSpecialDaysTimeLabel`, `SettingsSpecialDaysAddBtn`,
`SettingsSpecialDaysEmptyBody`, `SettingsSpecialDaysDuplicateHint`, `SettingsSpecialDaysDeleteBtn`,
`SettingsSpecialDaysDeleteError`, `SettingsSpecialDaysCalendarWeekAbbr`

**Shiftplan dropdown keys (4):**
`ShiftplanDayTypeHoliday`, `ShiftplanDayTypeShortDay`, `ShiftplanDayTypeNone`, `ShiftplanDayShortDayConfirm`

54 translation entries added (18 × 3 locales), verbatim from 33-UI-SPEC § "New i18n Keys". No reused
keys were duplicated (SettingsSaved, SettingsSaveError, CancelLabel, weekday keys were intentionally omitted).

## Verification

- WASM build (`cd shifty-dioxus && cargo build --target wasm32-unknown-unknown`): **GREEN** — the compile-time exhaustive Key match confirms all 18 new keys have translations in all three locales.
- `cargo clippy --workspace -- -D warnings` from backend root: **GREEN**.

## Deviations from Plan

### Note: 18 keys vs. PATTERNS.md "17 new Keys"

The PATTERNS.md § "add 17 new Keys" lists 17 variants, but 33-UI-SPEC § "New i18n Keys" table has 18 rows. The PLAN.md itself specifies 18 in the task action text. 18 keys were implemented to match the authoritative spec.

No other deviations. Plan executed as written.

## Known Stubs

None. This plan only adds foundation functions and i18n keys, with no UI rendering.

## Threat Flags

No new trust boundaries introduced beyond what the plan's threat model covers.
T-33-04 (Tampering via create body) is mitigated: `create_special_day` forces `id`/`version` to `Uuid::nil()`.

## Self-Check: PASSED

- `shifty-dioxus/src/api.rs` — FOUND (3 new functions: get_special_days_for_year, create_special_day, delete_special_day)
- `shifty-dioxus/src/i18n/mod.rs` — FOUND (18 new Key variants)
- `shifty-dioxus/src/i18n/de.rs` — FOUND (18 German translations)
- `shifty-dioxus/src/i18n/en.rs` — FOUND (18 English translations)
- `shifty-dioxus/src/i18n/cs.rs` — FOUND (18 Czech translations)
- Commit e4cd664: FOUND
- Commit 67a2462: FOUND
