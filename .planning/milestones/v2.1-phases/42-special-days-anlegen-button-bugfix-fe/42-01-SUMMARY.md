---
phase: 42-special-days-anlegen-button-bugfix-fe
plan: 01
subsystem: frontend (shifty-dioxus, Settings Card-3 Special-Days)
tags: [frontend, bugfix, state, tdd, pure-fn, dioxus, wasm]
requires:
  - "settings.rs Card-3 create form (Phase 33/36): create_special_day API + controlled SelectInput (D-06/D-08)"
provides:
  - "is_special_day_form_valid: pure, unit-tested validity predicate"
  - "SpecialDayForm + special_day_form_after_create: pure post-create retention policy (Option 2)"
  - "Anlegen button stays enabled after create → repeated create without dropdown toggle"
affects:
  - "shifty-dioxus/src/page/settings.rs (SettingsPage Card-3 only)"
tech-stack:
  added: []
  patterns:
    - "Extract inline render-body logic into pure pub(crate) fns + #[cfg(test)] unit tests (analog sd_type_to_select_value / is_duplicate_special_day)"
    - "Model post-create signal retention as a pure value fn, wired load-bearing into the success arm (no dead code under the WASM build)"
key-files:
  created: []
  modified:
    - "shifty-dioxus/src/page/settings.rs"
    - ".planning/phases/42-special-days-anlegen-button-bugfix-fe/42-VALIDATION.md"
decisions:
  - "D-42-01: Option 2 — remove the 3 post-create field resets; date/type/time stay filled so the button stays enabled"
  - "D-42-02: sd_year.set(iso_year) + sd_resource.restart() kept (not form-field resets)"
  - "D-42-03: sd_is_duplicate stays informative, NOT coupled to Btn.disabled"
  - "D-42-04: sd_save_result unchanged (Gespeichert stays until next submit)"
  - "D-42-05: validity predicate + retention policy extracted to tested pure fns"
  - "D-42-06: SSR/VirtualDom mount test skipped (Fall B) — component not sensibly mountable without a live harness; pure-fn tests are the sole hard gate"
metrics:
  duration: "~12min"
  tasks_completed: 3
  files_changed: 2
  completed: 2026-07-02
status: complete
---

# Phase 42 Plan 01: Special-Days-„Anlegen"-Button-Bugfix (FE) Summary

FE-only state bugfix: extracted the Card-3 create-form validity predicate and a
post-create retention policy into pure, unit-tested functions, then removed the three
post-create field resets so date/type/time stay filled — keeping the „Anlegen" button
enabled for repeated creates without re-toggling the type dropdown.

## What Was Built

- **`is_special_day_form_valid(date_str, ty, time_str) -> bool`** — the former inline
  `sd_form_valid` predicate (date non-empty AND type is Some AND (type ≠ ShortDay OR
  time non-empty)), extracted verbatim in semantics into a pure `pub(crate)` fn next to
  `sd_type_to_select_value`. The render body now calls it; `Btn { disabled: !sd_form_valid || *sd_saving.read() }`
  is unchanged.
- **`SpecialDayForm { date, ty, time }` + `special_day_form_after_create(before) -> SpecialDayForm`**
  — the post-create retention policy (Option 2, D-42-01/02) as a pure value fn returning
  `before.clone()`. Wired load-bearing into the create success arm: a `sd_form_before`
  snapshot is taken at closure entry, and after a successful POST the retained values are
  applied back to the three signals (so the fields stay filled, not emptied).
- **Reset block removed** — the three `sd_*_str.set(String::new())` / `sd_type.set(None)`
  calls and the stale WR-02 comment are gone. `sd_year.set(iso_year)` (WR-04) and
  `sd_resource.restart()` (list reload) are retained.
- **7 new unit tests** (prefix `special_day`): 5 predicate cases + retention-after-create
  + validity-stays-true-after-create (the central D-42-05 case).

## Behavior Result

- Button stays ENABLED after create → repeated create without dropdown toggle.
- Controlled-select D-06/D-08 intact: the type field stays FILLED after create (not
  emptied) → no signal↔DOM desync (D-25-06 class avoided by construction).
- Duplicate hint (`sd_is_duplicate`) remains informative, NOT coupled to `disabled`
  (D-42-03). „Gespeichert" persists until next submit (D-42-04). No i18n change.

## TDD Gate Compliance

- RED commit `b0fed7a` (`test(42-01)`) — 7 tests referencing not-yet-existing fns → module
  did not compile (expected RED).
- GREEN commit `e144847` (`feat(42-01)`) — added the two pure fns + struct; 10 `special_day`
  tests green.
- (Fixed a partial-move in one test: `SpecialDayTypeTO` is Clone-not-Copy, so `ty` is cloned
  when passed by value.)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] WASM build linker (`lld`) missing in the default shell**
- **Found during:** Task 2 verification (`cargo build --target wasm32-unknown-unknown`).
- **Issue:** default shell errored `linker 'lld' not found` → build could not link `tracing-wasm`.
- **Fix:** ran the WASM build inside `nix develop -c` (per plan/CLAUDE.local.md guidance); build
  finished warning-free. No code change; tooling-only.
- **Files modified:** none.

**2. [Rule 1 - Bug] Test partial-move on non-Copy enum**
- **Found during:** Task 1 GREEN.
- **Issue:** `is_special_day_form_valid(&before.date, before.ty, &before.time)` moved `before.ty`
  (SpecialDayTypeTO is not Copy), later borrowed `before` → E0382.
- **Fix:** clone `ty` at the call sites in `special_day_form_valid_stays_true_after_create`.
- **Commit:** `e144847`.

## D-42-06 SSR Decision

Justified skip (Fall B), documented in `42-VALIDATION.md`: `SettingsPage` is not sensibly
mountable in a bare `VirtualDom` — it reads global `I18N`/`CONFIG`/`AUTH` signals, its admin
guard short-circuits to "Not authorized" (so the button never renders without an initialized
`auth_info`), and it starts two network `use_resource` calls on mount; no VirtualDom harness
exists in the crate. `special_day_form_valid_stays_true_after_create` covers the button-enabled
invariant at the predicate level (the exact `!sd_form_valid` input to `Btn`).

## Gate Results

- `cargo build --target wasm32-unknown-unknown` (via `nix develop`): **warning-free** (HYG-01 held).
- `cargo test -p shifty-dioxus special_day`: **10 passed**.
- `cargo test -p shifty-dioxus` (full): **752 passed, 1 failed** — the failure is the documented
  pre-existing `i18n::tests::i18n_impersonation_keys_match_german_reference` (deferred, unrelated).
- Backend untouched → `cargo clippy --workspace -- -D warnings` trivially unaffected (no backend files changed).
- `42-VALIDATION.md`: `nyquist_compliant: true`, `wave_0_complete: true`.

## Scope

FE-only: only `settings.rs` (+ `42-VALIDATION.md`). No API/TO change, no snapshot bump, no
migration, no new deps, i18n unchanged.

## Known Stubs

None.

## Self-Check: PASSED

- Files: 42-01-SUMMARY.md, 42-VALIDATION.md, settings.rs — all present.
- Commits: b0fed7a, e144847, b77394e, fcd6362, 1288083 — all present.
