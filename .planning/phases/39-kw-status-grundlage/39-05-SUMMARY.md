---
phase: 39-kw-status-grundlage
plan: 05
subsystem: frontend
tags: [frontend, dioxus, wasm, week_status, badge, dropdown, shiftplan, i18n]

# Dependency graph
requires:
  - phase: 39-04-frontend
    provides: "WeekStatus enum + WEEK_STATUS_STORE + WeekStatusAction + week_status_service coroutine + i18n keys"
provides:
  - "component::atoms::week_status_badge::{WeekStatusBadge, should_show_badge, week_status_badge_class, week_status_label_key}"
  - "component::week_status_dropdown::WeekStatusDropdown (DropdownTrigger-based, no controlled select)"
  - "shiftplan.rs KW-status strip above WeekView + Load-per-KW wiring"
affects: [40-lock-enforcement]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pure-fn visibility gate should_show_badge(&WeekStatus) — unit-tested without VirtualDom (D-39-05)"
    - "Static match-arm Tailwind token classes per status (no format!, no palette literals) — D-39-08"
    - "Non-controlled dropdown on existing DropdownTrigger (D-39-06); reset entry incl. Unset (D-39-07)"

key-files:
  created:
    - shifty-dioxus/src/component/atoms/week_status_badge.rs
    - shifty-dioxus/src/component/week_status_dropdown.rs
  modified:
    - shifty-dioxus/src/component/atoms/mod.rs
    - shifty-dioxus/src/component/mod.rs
    - shifty-dioxus/src/page/shiftplan.rs

key-decisions:
  - "should_show_badge used in shiftplan.rs visibility matrix (not an inline != Unset) so the tested pure-fn is the single source of truth for the Unset->hidden rule"
  - "WeekStatusDropdown owns its own week_status_trigger_class helper (Unset=neutral, set states reuse badge token + gap-1) rather than importing week_status_badge_class — keeps the unreachable!(Unset) badge helper badge-only"
  - "shiftplan imports WeekStatusDropdown via the component/mod.rs re-export to keep the re-export non-dead"

requirements-completed: [WST-02]

# Metrics
duration: ~18min
completed: 2026-07-02
status: complete
---

# Phase 39 Plan 05: KW-Status Badge + Dropdown + Shiftplan-Integration Summary

**The visible KW-status layer: a color-coded `WeekStatusBadge` atom (all roles, only when set), a shiftplaner-only `WeekStatusDropdown` (built on the existing `DropdownTrigger`, no controlled `<select>`), and a status strip wired above the shiftplan week view that loads a fresh status per calendar week and sets it via the Wave-4 fresh-fetch store. Closes the Phase-39 vertical — the KW status is now visible for everyone and settable by shiftplaners; lock enforcement stays for Phase 40.**

## Performance
- **Duration:** ~18 min
- **Tasks:** 2 (Task 1 TDD RED→GREEN, Task 2 integration)
- **Files:** 2 created, 3 modified

## Accomplishments
- `component/atoms/week_status_badge.rs`: `should_show_badge(&WeekStatus)` (Unset→false, else true; D-39-05/WST-02), `week_status_badge_class(&WeekStatus)` with static token match arms (Locked=bad, Planned=good, InPlanning=warn; Unset=`unreachable!()`; D-39-08), `week_status_label_key`, and the `WeekStatusBadge` span component rendering token class + i18n label. Seven pure-fn tests incl. `no_legacy_classes_in_source` (employees.rs pattern).
- `component/week_status_dropdown.rs`: `WeekStatusDropdown { current_status, year, week, on_change }` on the existing `DropdownTrigger` (no controlled `<select>`, D-39-06). Four entries in ascending-commitment order Kein → In Planung → Geplant → Gesperrt, including Kein/Unset to reset (D-39-07); each forwards the chosen status via `on_change`. Trigger mirrors the current status (Unset=neutral `bg-surface-alt/border-border/text-ink-muted`; set states reuse the badge token + `gap-1` + caret). `aria-label` from `WeekStatusChangeAriaLabel`.
- `page/shiftplan.rs`: KW-status strip `div { class: "mb-3 flex items-center gap-2 print:hidden" }` above `WeekView` in `ViewMode::Week` (after `SlotEdit`). Visibility matrix: `is_shiftplanner` → `WeekStatusDropdown` (on_change sends `WeekStatusAction::Set`); else `should_show_badge(&week_status)` → `WeekStatusBadge`; else nothing. `WeekStatusAction::Load { year, week }` sent at coroutine init (after `set_selected_week`) and inside the `NextWeek`/`PreviousWeek` handlers for a fresh status per KW. Current status read from `WEEK_STATUS_STORE.read().status`.
- Module registration: `pub mod week_status_badge;` + `pub use WeekStatusBadge` in `atoms/mod.rs`; `pub mod week_status_dropdown;` + `pub use WeekStatusDropdown` in `component/mod.rs`.

## Task Commits
1. **Task 1 (RED): failing badge visibility + class-token tests** — `27c4eef` (test)
2. **Task 1 (GREEN): WeekStatusBadge atom + WeekStatusDropdown** — `792aec8` (feat)
3. **Task 2: wire KW-status strip into shiftplan week view** — `14867ad` (feat)

## Decisions Made
- **`should_show_badge` is the visibility SoT:** the shiftplan matrix calls the tested pure-fn instead of an inline `!= Unset`, so the D-39-05 rule has a single, unit-tested definition and the helper never goes dead.
- **Dropdown owns its trigger-class helper:** rather than reuse `week_status_badge_class` (which `unreachable!()`s on Unset), the dropdown has `week_status_trigger_class` covering all four states incl. the neutral Unset trigger — keeping the badge helper strictly badge-only.
- **Re-export import path:** `shiftplan.rs` imports `WeekStatusDropdown` via `crate::component::WeekStatusDropdown` (the `mod.rs` re-export) to avoid an unused-import warning on the re-export in the WASM build.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `no_legacy_classes_in_source` tripped on the module doc-comment**
- **Found during:** Task 1 (GREEN)
- **Issue:** The doc comment described the color mapping using a literal `bg-red-*/bg-green-*` phrase, which the `no_legacy_classes_in_source` guard scans the whole source prefix for — so it matched `bg-green-` and failed.
- **Fix:** Reworded the doc comment to name the semantic token classes (`bg-bad-soft`/`bg-good-soft`/`bg-warn-soft`) instead of palette literals.
- **Files modified:** shifty-dioxus/src/component/atoms/week_status_badge.rs
- **Commit:** 792aec8

**2. [Rule 3 - Blocking] Unused re-export warning for `WeekStatusDropdown`**
- **Found during:** Task 2 (WASM build)
- **Issue:** Importing the dropdown from its submodule path left the `component/mod.rs` re-export unused → warning in the WASM build.
- **Fix:** Switched the shiftplan import to the `crate::component::WeekStatusDropdown` re-export.
- **Files modified:** shifty-dioxus/src/page/shiftplan.rs, shifty-dioxus/src/component/mod.rs
- **Commit:** 14867ad

## Deferred Issues
- **Pre-existing i18n failure** `i18n::tests::i18n_impersonation_keys_match_german_reference` (de.rs `ImpersonateActAs` = "🥸 Agieren" from feat 37-02 vs. reference "Als diese Person agieren") still fails on the current tree, independent of this plan. Already tracked in `deferred-items.md` (logged during 39-04); untouched here (Scope Boundary — not a week_status file).

## Threat Mitigations Applied
- **T-39-01 (Elevation of Privilege):** the dropdown renders only under `is_shiftplanner` — a UX gate, not the security boundary. Server-side authorization (Wave 2/3) remains the real control; a non-shiftplaner PUT is rejected 403 and surfaces as the translated error banner.
- **T-39-05 (Spoofing/Consistency):** the strip reads `WEEK_STATUS_STORE.status`, which the Wave-4 store re-fetches after every mutation (fresh-fetch) — no optimistic/driftable status is displayed.

## Scope Guard (Phase 40)
- The Locked badge/trigger only **displays** — no lock enforcement, no read-only, no HTTP 423 was introduced. `assert_week_not_locked` and friends remain Phase-40 work.

## Gate Results
- `cargo test -p shifty-dioxus week_status_badge` — pass (7/7: visibility pure-fn + class tokens + no_legacy)
- `cargo test -p shifty-dioxus` — 740 pass, 1 pre-existing unrelated impersonation-reference failure (deferred above)
- `cargo build --target wasm32-unknown-unknown` (shifty-dioxus, via `nix develop`) — pass
- `cargo clippy --workspace -- -D warnings` (backend root, via `nix develop`) — pass (dioxus is a separate workspace excluded from backend clippy per project convention)

## Next Phase Readiness
- Phase 40 can build lock enforcement on top of the visible `Locked` status (assert_week_not_locked, HTTP 423, read-only shiftplan) — the display/set vertical is complete.
- No blockers.

## Self-Check: PASSED
- `shifty-dioxus/src/component/atoms/week_status_badge.rs` present on disk.
- `shifty-dioxus/src/component/week_status_dropdown.rs` present on disk.
- Commits 27c4eef, 792aec8, 14867ad exist in git history.

---
*Phase: 39-kw-status-grundlage*
*Completed: 2026-07-02*
