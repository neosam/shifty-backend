---
phase: 24-paid-capacity-enforcement-config
plan: "04"
subsystem: frontend-settings-ui
tags: [settings, admin-gate, toggle-api, nav, dioxus, phase-24]
dependency_graph:
  requires: [24-03]
  provides: [Route::Settings, SettingsPage, NavTarget::Settings, api::set_toggle, api::get_toggle_enabled, loader::set_toggle, loader::get_toggle_enabled]
  affects: [24-05-PLAN.md]
tech_stack:
  added: []
  patterns: [use_resource for async load, spawn for async mutation, inline feedback signal, is_admin_target nav gating]
key_files:
  created:
    - shifty-dioxus/src/page/settings.rs
  modified:
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/page/mod.rs
    - shifty-dioxus/src/router.rs
    - shifty-dioxus/src/component/top_bar.rs
decisions:
  - "Used spawn() for the toggle PUT (not use_future/coroutine) — matches the lightest mutation pattern; no separate service/coroutine needed for a one-function settings page"
  - "Saved/Error feedback via Signal<Option<bool>> — None=idle, Some(true)=saved, Some(false)=error — avoids a separate enum and matches the minimal inline feedback contract from UI-SPEC"
  - "NavTarget::Settings added as 6th site in nav_entry() test helper match — ensures exhaustive match stays sound"
  - "Updated hr_admin_user_partitions_into_top_level_and_full_admin_group test to include Einstellungen in admin labels"
metrics:
  duration: ~30min
  completed: 2026-06-27
  tasks_completed: 3
  tasks_total: 3
---

# Phase 24 Plan 04: Admin-gated Settings Page — Summary

One-liner: Admin-only `/settings/` route + nav entry + SettingsPage with single paid-limit hard/soft toggle wired to the existing Toggle REST API via `api::set_toggle` / `api::get_toggle_enabled`.

## Tasks Completed

| # | Task | Status | Key Artifacts |
|---|------|--------|---------------|
| 1 | Toggle REST client (api.rs + loader.rs) | Done | api.rs: `set_toggle`, `get_toggle_enabled`; loader.rs: `ShiftyError`-returning wrappers |
| 2 | SettingsPage + route + barrel export | Done | page/settings.rs, page/mod.rs, router.rs |
| 3 | Admin-gated Settings nav wiring + WASM gate | Done | top_bar.rs (6 sites), nav tests, WASM build |

## What Was Built

### API Layer (`shifty-dioxus/src/api.rs`)
Two new functions at the bottom of api.rs:
- `set_toggle(config, name, enabled)` — PUT `/toggle/{name}/enable` or `/disable`
- `get_toggle_enabled(config, name)` — GET `/toggle/{name}/enabled` returning bool

### Loader Layer (`shifty-dioxus/src/loader.rs`)
Two thin `ShiftyError`-returning wrappers:
- `set_toggle(config, name, enabled) -> Result<(), ShiftyError>`
- `get_toggle_enabled(config, name) -> Result<bool, ShiftyError>`

### Settings Page (`shifty-dioxus/src/page/settings.rs`)
- `SettingsPage` component with `TopBar` at top
- Initial toggle state loaded via `use_resource(|| loader::get_toggle_enabled(config, "paid_limit_hard_enforcement"))`
- State sync via `use_effect` on resource resolution
- Toggle click dispatches via `spawn(async { loader::set_toggle(...) })` with in-flight `saving` signal disabling the button
- On success: flips `hard_enforcement` signal + sets `save_result = Some(true)` (shows "Saved.")
- On error: reverts state (signal unchanged) + sets `save_result = Some(false)` (shows error)
- Exact UI-SPEC class strings used (Off: `border-border bg-surface`, On: `border-bad text-bad bg-bad-soft font-semibold`)
- `aria-pressed="true"/"false"` attribute on toggle button

### Admin Nav Gate (`shifty-dioxus/src/component/top_bar.rs`)
6 sites wired in lockstep with `NavTarget::UserManagement`:
1. `NavVisibility` struct: `pub settings: bool`
2. `nav_visibility()`: `settings: has("admin")`
3. `NavTarget` enum: new `Settings` variant
4. `is_active_for()`: `NavTarget::Settings => matches!(route, Route::Settings {})`
5. `is_admin_target()`: `NavTarget::Settings` added to matches! set
6. Items builder: `if visibility.settings { items.push((NavTarget::Settings, Route::Settings {}, i18n.t(Key::Settings))) }`

Test helper `nav_entry()` extended with `NavTarget::Settings => Route::Settings {}`.

### Router & Barrel
- `router.rs`: `pub use crate::page::SettingsPage as Settings;` + `#[route("/settings/")] Settings {}`
- `page/mod.rs`: `pub mod settings;` + `pub use settings::SettingsPage;`

## Admin Gate Wiring

The client-side gate is `settings: has("admin")` in `nav_visibility()`, matching the `user_management` and `templates` pattern exactly. The nav entry is classified as `is_admin_target(NavTarget::Settings) = true`, placing it in the admin dropdown (not top-level nav). A non-admin who manually navigates to `/settings/` sees the page but any PUT to the Toggle REST API is rejected server-side (backend enforces `toggle_admin` privilege via the existing Toggle REST endpoint).

## Toggle API Wiring

1. Page loads: `use_resource` calls `loader::get_toggle_enabled(config, "paid_limit_hard_enforcement")` → GET `/toggle/paid_limit_hard_enforcement/enabled` → bool
2. `use_effect` syncs the result into `hard_enforcement` signal
3. Button click: `spawn(async { loader::set_toggle(config, "paid_limit_hard_enforcement", !current) })` → PUT `/toggle/paid_limit_hard_enforcement/enable` or `/disable`
4. Result: success flips signal + shows `SettingsSaved`; error leaves signal unchanged + shows `SettingsSaveError`

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| cargo build (native) | `nix develop . --command cargo build` from `shifty-dioxus/` | PASS (48 warnings, all pre-existing) |
| cargo test | `nix develop . --command cargo test` | PASS — 669 passed, 0 failed |
| cargo test top_bar | `cargo test top_bar` | PASS — 43 passed, 0 failed |
| WASM build gate | `nix develop . --command cargo build --target wasm32-unknown-unknown` | PASS (43 warnings, all pre-existing) |

(Clippy not run — dioxus workspace excluded from CI clippy gate per memory note; ~198 pre-existing lints.)

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written.

### Test Updates Required (Rule 2: completeness)

**Updated `hr_admin_user_partitions_into_top_level_and_full_admin_group` test**: This test asserted an exact list of admin panel labels. Adding `NavTarget::Settings` caused "Einstellungen" to appear at the end of the admin group for `sales+hr+admin` users. Updated the expected `admin_labels` vec to include "Einstellungen" — the test correctly reflects the new behavior.

**Added `nav_visibility_non_admin_hides_settings` test**: New test asserting that `sales+hr` users have `v.settings = false`, mirroring the plan's requirement to assert Settings is hidden for non-admins.

## Known Stubs

None — the toggle wiring is complete (load + flip + feedback). Toggle name is hardcoded as `"paid_limit_hard_enforcement"` per plan scope (D-24-06 explicitly scopes to this one switch; generic toggle UI is deferred).

## Threat Flags

None — no new trust boundaries beyond those documented in the plan's threat model (T-24-09 to T-24-11).

## Self-Check

- `shifty-dioxus/src/page/settings.rs` contains `paid_limit_hard_enforcement`: FOUND
- `shifty-dioxus/src/api.rs` contains `/toggle/`: FOUND
- `shifty-dioxus/src/loader.rs` contains `set_toggle`: FOUND
- `shifty-dioxus/src/page/mod.rs` exports `SettingsPage`: FOUND
- `shifty-dioxus/src/router.rs` contains `/settings/`: FOUND
- `shifty-dioxus/src/router.rs` contains `Settings` alias: FOUND
- `shifty-dioxus/src/component/top_bar.rs` contains `NavTarget::Settings` (5 sites): FOUND
- `shifty-dioxus/src/component/top_bar.rs` contains `settings: has("admin")`: FOUND
- `shifty-dioxus/src/page/settings.rs` contains `aria-pressed`: FOUND
- cargo test: 669 passed, 0 failed
- cargo build --target wasm32-unknown-unknown: Finished dev profile exit 0

## Self-Check: PASSED
