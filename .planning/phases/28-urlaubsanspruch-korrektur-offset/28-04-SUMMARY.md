---
phase: 28-urlaubsanspruch-korrektur-offset
plan: 04
subsystem: frontend-vacation-offset
tags: [frontend, dioxus, i18n, vacation, offset, hr]
status: complete
requires:
  - "28-02: VacationBalanceTO.offset_days/computed_entitled_days (Option, HR-only) + POST /vacation-entitlement-offset"
provides:
  - "FE inline HR offset editor (D-28-07) + user-side effective-only view (D-28-03)"
  - "save_vacation_entitlement_offset API + SaveOffset action (post → reload)"
  - "i18n VacationOffsetComputedLabel/VacationOffsetLabel (de/en/cs)"
affects:
  - shifty-dioxus/src/page/absences.rs
  - shifty-dioxus/src/service/vacation_balance.rs
tech-stack:
  added: []
  patterns:
    - "post(url).json(&body).send() + error_for_status_ref() (mirrors create_absence_period)"
    - "use_coroutine_handle::<VacationBalanceAction>() dispatch from a leaf component"
    - "Copy closure (commit) shared across onkeydown + onfocusout handlers"
key-files:
  created: []
  modified:
    - shifty-dioxus/src/state/vacation_balance.rs
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/service/vacation_balance.rs
    - shifty-dioxus/src/page/absences.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/cs.rs
decisions:
  - "D-28-07: HR-detail StatBox shows effective box number + 'berechnet {computed} + Offset [x]' signed inline input; saves on blur/Enter, year-scoped."
  - "D-28-03: self-view receives computed/offset == None from backend → plain effective StatBox, no breakdown line, no input (no client re-derivation)."
  - "D-28-08: new labels added to de (Locale::De), en, cs + guarded by the i18n coverage test."
  - "D-28-09: SaveOffset targets the overview's selected year + the displayed balance's sales_person_id."
  - "Loader wrapper save_vacation_entitlement_offset added for convention consistency with the load path."
metrics:
  duration: ~25m
  completed: 2026-06-29
  tasks: 2
  files: 9
---

# Phase 28 Plan 04: FE inline HR vacation-entitlement-offset editor Summary

Renders the HR inline signed offset editor ("berechnet {computed} + Offset [x]") in the person-detail "Vertragsanspruch" StatBox while the user self-view keeps showing only the effective number — backed by the new TO Option fields and the HR-gated save endpoint from Plan 28-02.

## What was built

**Task 1 — FE state/api/service plumbing (TDD):**
- `state::vacation_balance::VacationBalance` gained `offset_days: Option<i32>` and `computed_entitled_days: Option<f32>`, mapped 1:1 in `From<&VacationBalanceTO>`. The existing round-trip test now asserts `Some` values, and a new `vacation_balance_from_to_preserves_none_breakdown` test asserts `None` round-trips (the self-only case).
- `api::save_vacation_entitlement_offset(config, sales_person_id, year, offset_days)` POSTs `VacationEntitlementOffsetTO` to `{backend}/vacation-entitlement-offset` (mirrors `create_absence_period`: `post(url).json(&body).send()` + `error_for_status_ref()`), returning `Result<(), ShiftyError>`. A thin `loader::save_vacation_entitlement_offset` wrapper keeps parity with the load path.
- `VacationBalanceAction::SaveOffset(Uuid, u32, i32)` added; on save success the coroutine re-loads the team aggregate (the HR `forced_self` detail view reads from `VACATION_TEAM_STORE`) and, when the self store holds the same person, re-loads `VACATION_BALANCE_STORE` so the effective value re-computes from the backend. Errors write `ERROR_STORE`.

**Task 2 — Inline HR editor + i18n:**
- `is_hr: bool` threaded into `VacationEntitlementSelfBodyProps`, passed as `props.is_hr` from `VacationEntitlementCard` (only `true` on the HR `forced_self` detail path).
- New `VacationContractCell` component replaces the plain contract `StatBox`:
  - `computed.is_some()` (HR + backend supplied breakdown) → effective entitlement as the big box number + a "berechnet {computed} + Offset [x]" line with a signed `<input type=number>` seeded from `offset_days.unwrap_or(0)`. On blur (`onfocusout`) and Enter (`onkeydown`), the offset is parsed and dispatched via `SaveOffset(balance.sales_person_id, props.year, parsed)`. Static Tailwind classes only (no `format!` into class strings).
  - `computed.is_none()` (employee self-view, or fields absent) → byte-for-byte the original effective-only `StatBox`: no field, no breakdown line.
- i18n keys `VacationOffsetComputedLabel` (de "berechnet" / en "calculated" / cs "vypočteno") and `VacationOffsetLabel` ("Offset" in all three) added; de uses `Locale::De`. Both keys added to the i18n key-coverage test array so all three locales are guarded.

## User-view leak confirmation

The self-view does NOT leak the offset: the backend sends `offset_days == None` / `computed_entitled_days == None` to self-only callers (Plan 28-02 API-level hiding, D-28-03). `VacationContractCell` renders the editor branch only when `props.is_hr && computed.is_some()`; for the employee path `is_hr` is `false` AND the fields are `None`, so the plain effective `StatBox` is rendered with no breakdown line and no input. No offset is re-derived client-side.

## Gate results

- **WASM build (HARD gate):** PASS. Command: `cd shifty-dioxus && nix develop -c bash -c 'cargo build --target wasm32-unknown-unknown 2>&1 | tail -8'` → `Finished dev profile ... in 28.44s`, 46 pre-existing warnings, zero errors.
- **cargo test (HARD gate):** PASS. `cd shifty-dioxus && cargo test` → `678 passed; 0 failed` (Phase 27 baseline 677 + 1 new `None`-round-trip test). i18n subset: `43 passed; 0 failed`.
- **Clippy (SOFT for FE):** No new warnings. `cargo clippy 2>&1 | grep -c '^warning'` → 207, matching the documented pre-existing baseline (~207). No clippy warning references any new identifier (`VacationContractCell`, `save_vacation_entitlement_offset`, `SaveOffset`, `VacationOffset*`) or any touched state/service file; the only hit in a touched file (`i18n/mod.rs:4` module-inception) is pre-existing and unrelated.
- **i18n de Locale guard:** de labels added via `Locale::De` (historical-bug guard satisfied); coverage test extended with both new keys across En/De/Cs.

## Deviations from Plan

- **Added `loader::save_vacation_entitlement_offset` wrapper** (Rule 3 / convention): the load path goes through `loader`, so a thin save wrapper was added for consistency; the service calls the loader rather than `api` directly. Non-behavioral.
- Otherwise the plan executed as written.

## Known Stubs

None.

## Human-verify checkpoint (deferred to orchestrator)

Task 2's `<human-check>` browser smoke (HR detail of a person with computed 17, offset +1 shows "berechnet 17 + Offset [1]" and box 18; same person in user self-view shows only 18; edit+blur persists and updates after reload) is left for the orchestrator's browser smoke per execution instructions. All automated gates are green.

## Self-Check: PASSED

- Files exist: state/vacation_balance.rs, api.rs, loader.rs, service/vacation_balance.rs, page/absences.rs, i18n/{mod,en,de,cs}.rs — all modified and compiling.
- Commits exist: 6aeb6e56 (Task 1), 204fefc2 (Task 2) — present in `jj log`.
