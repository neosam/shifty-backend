---
phase: 33-special-days-ui-einstellungen
plan: "04"
subsystem: frontend
tags: [dioxus, shiftplan, special-days, dropdown, wasm]
status: complete

dependency_graph:
  requires: ["33-01", "33-02"]
  provides: ["special-day-dropdown-in-shiftplan-grid"]
  affects: ["shifty-dioxus/src/page/shiftplan.rs", "shifty-dioxus/src/component/week_view.rs"]

tech_stack:
  added: []
  patterns:
    - "weekday_sub_headers Vec<(Weekday, Element)> prop on WeekView for injecting interactive elements inside the CSS grid"
    - "use_resource for per-page special-day list; restart() on both resource and shift_plan_context after mutation"
    - "spawn(async move { ... }) wrapper around Signal::set inside DropdownEntry Fn closures (set takes &mut self)"
    - "let mut config_week = config.clone() before use_coroutine to preserve config after coroutine moves it"

key_files:
  modified:
    - shifty-dioxus/src/component/week_view.rs
    - shifty-dioxus/src/page/shiftplan.rs

decisions:
  - "weekday_sub_headers Vec<(Weekday, Element)> added to WeekViewProps; sub-header row rendered only when non-empty (Rule 3 deviation — plan only listed shiftplan.rs but week_view.rs modification is required to inject elements inside the scrollable CSS grid)"
  - "Signal::set is &mut self in Dioxus 0.6.1; DropdownEntry requires Fn; all set() calls inside DropdownEntry closures are wrapped in spawn(async move { ... })"
  - "config is moved into the coroutine via to_owned![..., config, ...]; a config_week clone is created before the coroutine for use in weekday_sub_headers"

metrics:
  duration_minutes: 45
  completed_date: "2026-06-30T12:26:38Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
---

# Phase 33 Plan 04: Per-Weekday Special-Day Dropdown in Shiftplan Grid Summary

**One-liner:** Per-weekday Feiertag/Kurzer-Tag/Nichts DropdownTrigger with inline ShortDay time prompt in the shiftplan WeekView, reloading both special-days and shift-plan resources on mutation.

## What Was Built

### Task 1: Special-Day Resource + Per-Day DropdownTrigger

- Added `weekday_sub_headers: Vec<(Weekday, Element)>` prop to `WeekViewProps` with `#[props(default = vec![])]`
- Added sub-header row rendering in `WeekView`: sticky corner cell + one cell per `visible_day` containing the matching element from the caller
- In `shiftplan.rs`: added `use_resource(get_special_days_for_week)` reading `year`/`week` signals reactively
- Added signals: `shortday_prompt_day: Signal<Option<Weekday>>`, `shortday_time: Signal<String>`, `special_day_error: Signal<Option<(Weekday, ImStr)>>`
- Added `config_week = config.clone()` before the coroutine (which moves `config` via `to_owned!`)
- Week-change `use_effect` clears prompt and error signals on navigation
- Builds 7-day `weekday_sub_headers` vec (empty when `!is_shiftplanner`): each day gets a `DropdownTrigger` with Holiday / ShortDay / Nichts entries
- Nichts entry has `disabled: true` (= hidden by DropdownBase line 52) when no entry exists for the day
- Holiday entry and Nichts (delete) entry each call `spawn(async move { api::... .await })` then `restart()` both `special_days_for_week` and `shift_plan_context`
- Inline colored dot indicator: `bg-accent` for Holiday, `bg-warn` for ShortDay

### Task 2: ShortDay Inline Time Prompt

- When `shortday_prompt_day == Some(day)`, the day's element is the inline form instead of the dropdown trigger
- Form contains `<input type="time">` + Save button (disabled when time empty) + Cancel button
- Save parses the time string with `time::macros::format_description!("[hour]:[minute]")`, builds a `SpecialDayTO` with `day_type: ShortDay` and `time_of_day: Some(parsed_time)`, calls `create_special_day`, then restarts both resources
- Cancel resets `shortday_prompt_day` to `None` and clears `shortday_time`
- Per-day inline error span under both the dropdown and the form on API failure

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Modified week_view.rs in addition to shiftplan.rs**
- **Found during:** Task 1 implementation
- **Issue:** The plan listed only `shiftplan.rs` as modified, but interactive elements can only be placed inside the WeekView grid (which owns the overflow-auto scroll container) — injecting from outside would break horizontal scroll sync
- **Fix:** Added `weekday_sub_headers: Vec<(Weekday, Element)>` prop and sub-header row rendering to `week_view.rs`
- **Files modified:** `shifty-dioxus/src/component/week_view.rs`
- **Commit:** 2580296

**2. [Rule 1 - Bug] Element type is Result<VNode, RenderError>, not Option<VNode>**
- **Found during:** WASM build
- **Issue:** Used `.and_then(|(_, el)| el.clone())` which returns `Option<Result<...>>` — wrong type for `Element`
- **Fix:** Changed to `.map(|(_, el)| el.clone()).unwrap_or_else(|| rsx! {})`
- **Files modified:** `week_view.rs`
- **Commit:** 2580296

**3. [Rule 1 - Bug] i18n.t() returns Rc<str>, not ImStr**
- **Found during:** WASM build
- **Issue:** Type annotation `let x: ImStr = i18n.t(...)` failed because `i18n.t()` returns `Rc<str>`
- **Fix:** Added `.into()` to all 6 i18n string conversions (`From<Rc<str>> for ImStr` exists)
- **Files modified:** `shiftplan.rs`
- **Commit:** 2580296

**4. [Rule 1 - Bug] Signal::set takes &mut self, making DropdownEntry closures FnMut**
- **Found during:** WASM build
- **Issue:** DropdownEntry requires `Fn`, but closures calling `signal.set()` are `FnMut`
- **Fix:** Wrapped all `set()` calls in `DropdownEntry` closures in `spawn(async move { ... })`; also added `mut` to `shortday_time` and `special_day_error` declarations (needed for `use_effect` and onclick handlers)
- **Files modified:** `shiftplan.rs`
- **Commit:** 2580296

**5. [Rule 1 - Bug] config moved into coroutine, unavailable for weekday_sub_headers**
- **Found during:** WASM build
- **Issue:** `config` is moved into the coroutine via `to_owned![..., config, ...]`; the weekday_sub_headers building code (after the coroutine) cannot use `config`
- **Fix:** Added `let config_week = config.clone()` before the coroutine definition; used `config_week.clone()` in the three per-entry `cfg` bindings
- **Files modified:** `shiftplan.rs`
- **Commit:** 2580296

**6. [Rule 1 - Bug] E0317: if-let without else in inline ShortDay error display**
- **Found during:** WASM build
- **Issue:** `{ if let Some(x) = opt { rsx! {...} } }` as a block expression evaluates to `()` in the non-matching case, but `Element = Result<VNode, RenderError>`
- **Fix:** Moved the error span inside the outer `rsx!` div using Dioxus RSX conditional syntax (`if let Some(x) = opt { span { ... } }`) — valid inside rsx! without else clause
- **Files modified:** `shiftplan.rs`
- **Commit:** 2580296

## Verification

- WASM build: `cargo build --target wasm32-unknown-unknown` — passed (warnings only, pre-existing)
- Backend clippy: `cargo clippy --workspace -- -D warnings` — passed (clean)

## Self-Check: PASSED

- `shifty-dioxus/src/component/week_view.rs` — modified ✓
- `shifty-dioxus/src/page/shiftplan.rs` — modified ✓
- Commit 2580296 exists ✓
