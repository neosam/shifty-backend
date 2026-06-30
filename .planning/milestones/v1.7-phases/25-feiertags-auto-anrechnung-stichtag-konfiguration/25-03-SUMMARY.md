---
phase: 25-feiertags-auto-anrechnung-stichtag-konfiguration
plan: 03
subsystem: frontend
tags: [dioxus, wasm, i18n, settings, toggle, rest-client, date-input]

requires:
  - phase: 25-feiertags-auto-anrechnung-stichtag-konfiguration
    plan: 01
    provides: "GET/PUT/DELETE /toggle/{name}/value REST endpoints"

provides:
  - "get_toggle_value / set_toggle_value / clear_toggle_value REST clients in api.rs"
  - "get_holiday_cutoff_date / set_holiday_cutoff_date loaders keyed to holiday_auto_credit"
  - "Settings page Card 2 — date input with Save/Clear/inline-feedback/unset-hint, admin-gated"
  - "5 new i18n keys (SettingsHolidayAutoCreditLabel/Description/Save/Clear/UnsetHint) in en/de/cs"
  - "i18n_phase25_keys_present_in_all_locales completeness test"

affects:
  - "Admin /settings page — second card visible only to admins"

tech-stack:
  added: []
  patterns:
    - "Toggle value REST client: GET 204 → None; GET 200 → Some(String)"
    - "Date input via TextInput { input_type: date } with ImStr value signal"
    - "Save enabled on non-empty date_str (WASM signal caveat D-25-06)"
    - "Client-side ISO date validation via time::macros::format_description before PUT"
    - "Inline feedback using SettingsSaved / SettingsSaveError (reused from Phase 24)"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/page/settings.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/cs.rs

key-decisions:
  - "TextInput { input_type: date } used for date field per UI-SPEC Component Inventory"
  - "Save button disabled only on is_cutoff_saving (not on empty) per UI-SPEC Row C; handler returns early on empty"
  - "Client-side date validation via format_description![year]-[month]-[day] macro — available via time 'macros' feature in shifty-dioxus/Cargo.toml"
  - "date_str Signal<String> converted to ImStr on each render for TextInput value prop"
  - "cutoff_save_result / cutoff_saving named distinctly from Card 1 save_result / saving"

metrics:
  duration: 25min
  completed: 2026-06-28
  tasks_completed: 2
  tasks_total: 3
  files_modified: 7

status: human-verify-pending
---

# Phase 25 Plan 03: Settings Holiday-Cutoff Date Field Summary

**REST clients + loaders + 5 i18n keys (en/de/cs) + Settings Card 2 date input wired to toggle value endpoints; WASM build green; Task 3 (human browser verify) pending**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-06-28
- **Completed (Tasks 1-2):** 2026-06-28
- **Tasks completed:** 2 of 3 (Task 3 = human-verify checkpoint, pending)
- **Files modified:** 7 frontend files

## Accomplishments

### Task 1: API clients, loaders, i18n keys

- `api.rs`: three new functions after Phase-24 toggle clients (~line 1595):
  - `get_toggle_value(config, name) → Result<Option<String>, reqwest::Error>` — GET `/toggle/{name}/value`, 204 → `Ok(None)`
  - `set_toggle_value(config, name, value) → Result<(), reqwest::Error>` — PUT with JSON body
  - `clear_toggle_value(config, name) → Result<(), reqwest::Error>` — DELETE

- `loader.rs`: two new functions after Phase-24 toggle loaders (~line 1024):
  - `get_holiday_cutoff_date(config) → Result<Option<String>, ShiftyError>` — delegates to `get_toggle_value(..., "holiday_auto_credit")`
  - `set_holiday_cutoff_date(config, Option<&str>) → Result<(), ShiftyError>` — set_toggle_value for Some, clear_toggle_value for None

- `i18n/mod.rs`: 5 new Key variants added after SettingsSaveError:
  `SettingsHolidayAutoCreditLabel`, `...Description`, `...Save`, `...Clear`, `...UnsetHint`

- `i18n/en.rs`, `de.rs`, `cs.rs`: verbatim strings from 25-UI-SPEC.md Copywriting Contract

- `i18n/mod.rs`: `i18n_phase25_keys_present_in_all_locales` completeness test — verifies all 5 keys resolve to non-empty, non-"??" values in all 3 locales

**Verification:** `cargo test i18n` — 42 tests, all passed including the new Phase 25 test.

### Task 2: Settings Card 2 — holiday auto-credit date input

- `settings.rs` rewritten to add Card 2 below the existing Phase-24 toggle card:
  - Signals: `date_str: Signal<String>`, `date_str_loaded_empty: Signal<bool>`, `cutoff_save_result: Signal<Option<bool>>`, `cutoff_saving: Signal<bool>`
  - `use_resource(get_holiday_cutoff_date)` + `use_effect` sync: `Some(Ok(Some(date)))` → set date_str + loaded_empty=false; `Some(Ok(None))` → date_str="" + loaded_empty=true
  - `on_save_cutoff`: validates non-empty + ISO date format (defense in depth via `time::format_description!`), calls `loader::set_holiday_cutoff_date(cfg, Some(&val))`
  - `on_clear_cutoff`: calls `loader::set_holiday_cutoff_date(cfg, None)`, sets date_str="" + loaded_empty=true on Ok
  - RSX: Card 2 `div.bg-surface.border.border-border.rounded-md.p-4.flex.flex-col.gap-3.mt-4`:
    - Row A: label span + description span
    - Row B: `TextInput { input_type: "date", value: date_value }` in `div.max-w-[200px]`
    - Row C: Save button (disabled only on saving), Clear button (disabled on saving||empty), inline feedback `Some(true)/Some(false)/None`
    - Row D: unset hint span shown when `loaded_empty` is true
  - Admin guard: same `if !is_admin { return not-authorized rsx! }` as Card 1; both cards share the single guard

**Verification:** WASM build via `nix-shell -p openssl pkg-config lld --run 'cd shifty-dioxus && cargo build --target wasm32-unknown-unknown'` — succeeded with 46 pre-existing warnings, no errors.

## Task Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1+2  | API clients + loaders + i18n + Settings Card 2 | 59dc35e | 7 files |

## Deviations from Plan

None — plan executed exactly as written. Minor implementation details:

- Used `format_description!("[year]-[month]-[day]")` macro for ISO date validation (available via `time` crate `macros` feature already in Cargo.toml), matching the pattern in `base_components.rs:182`.
- Save button `disabled: is_cutoff_saving` (not `is_cutoff_saving || date_empty`) per UI-SPEC Row C exactly; handler returns early on empty string (defense in depth at two levels).

## Pending: Task 3 — Human Browser Verify

Task 3 is a `type="checkpoint:human-verify"` that cannot be automated due to the WASM programmatic date input signal caveat.

**How to verify:**
1. Start backend (port 3000) and frontend (`dx serve`, port 8080)
2. As admin, open http://localhost:8080/settings — Card 2 "Feiertags-Automatik aktiv ab" visible
3. Pick a date, click "Datum speichern" → inline "Gespeichert." appears
4. Reload page → saved date still shown (persisted)
5. Click "Löschen (deaktivieren)" → field clears, unset hint "Nicht gesetzt — Automatik inaktiv." shows
6. Switch locale (en/cs) → labels translate
7. As non-admin, open /settings → "Not authorized." shown (card not accessible)

**Resume:** Type "approved" or describe what rendered/persisted incorrectly.

## Known Stubs

None — all data wiring is live (REST clients call real backend endpoints established in 25-01).

## Threat Surface Scan

No new network endpoints. Three new REST client functions in api.rs call the endpoints already established and gated in 25-01. Admin guard in settings.rs (T-25-07) reuses the existing `is_admin` check from Card 1. Client-side ISO date validation (T-25-08) added before PUT.

## Self-Check

- [x] `shifty-dioxus/src/api.rs` — contains `get_toggle_value`, `set_toggle_value`, `clear_toggle_value` with `/value`
- [x] `shifty-dioxus/src/loader.rs` — contains `get_holiday_cutoff_date`, `set_holiday_cutoff_date` with `"holiday_auto_credit"`
- [x] `shifty-dioxus/src/page/settings.rs` — contains `SettingsHolidayAutoCredit` keys, Card 2 RSX
- [x] `shifty-dioxus/src/i18n/mod.rs` — contains `SettingsHolidayAutoCreditLabel` and 4 sibling variants + completeness test
- [x] `shifty-dioxus/src/i18n/en.rs` — 5 English translations present
- [x] `shifty-dioxus/src/i18n/de.rs` — 5 German translations present
- [x] `shifty-dioxus/src/i18n/cs.rs` — 5 Czech translations present
- [x] Commit `59dc35e` — EXISTS (`feat(25): settings holiday-cutoff date field (25-03)`)
- [x] `cargo test i18n` — 42 tests passed, including `i18n_phase25_keys_present_in_all_locales`
- [x] `cargo build --target wasm32-unknown-unknown` — Finished with no errors (46 pre-existing warnings)

## Self-Check: PASSED

---
*Phase: 25-feiertags-auto-anrechnung-stichtag-konfiguration*
*Completed (Tasks 1-2): 2026-06-28*
*Task 3 (human-verify): PENDING*
