---
phase: 23-frontend-slot-paid-capacity-ui
plan: 01
subsystem: ui
tags: [dioxus, wasm, week-view, tailwind, ssr-tests, slot-capacity]

# Dependency graph
requires:
  - phase: 06 (backend paid-capacity state mirror)
    provides: "Slot.max_paid_employees + Slot.current_paid_count already populated by the week-view loader"
provides:
  - "Week-view slot cells tint bg-bad-soft (red) when current_paid_count exceeds max_paid_employees"
  - "Paid-overage coloring outranks orange understaffing (bg-warn-soft) and is computed for all roles"
  - "3-arg cell_background_class(missing, discourage, paid_overage) decision fn"
affects: [23-02 (slot-edit paid-capacity form + overage banner)]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Combined-arm priority decision fn (discourage || paid_overage -> red) to satisfy clippy::if_same_then_else while keeping documented precedence"
    - "is_some_and over map_or(false, ..) for Option<u8> threshold checks (clippy gate)"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/component/week_view.rs

key-decisions:
  - "Combined discourage and paid_overage into a single red arm (clippy::if_same_then_else); documented priority preserved in the doc comment and tests"
  - "Used is_some_and instead of the plan-specified map_or(false, ..) because the clippy -D warnings gate flags unnecessary_map_or"
  - "Ran clippy from the backend-root nix shell (matching rustc/clippy 1.93.0) with OPENSSL_LIB_DIR/INCLUDE overrides because the dioxus dev shell ships mismatched clippy 0.1.93 vs rustc 1.95.0 (E0514)"

patterns-established:
  - "Pattern 1: per-cell state-background priority — discourage/paid_overage (red) > missing (orange) > none"
  - "Pattern 2: SSR overage assertion reuses the existing week_cell_slot_render_tests harness (make_slot + render/render_with_tooltip)"

requirements-completed: [D-23-03, D-23-04, D-23-05]

# Metrics
duration: ~35min
completed: 2026-06-26
---

# Phase 23 Plan 01: Week-View Paid-Capacity Coloring Summary

**Week-view slot cells now tint `bg-bad-soft` (red) when a slot's live paid-booking count exceeds its `max_paid_employees` limit, overriding the orange understaffing tint, visible to all roles.**

## Performance

- **Duration:** ~35 min (incl. a clean rebuild forced by a dev-shell toolchain/openssl mismatch)
- **Completed:** 2026-06-26T21:56:29Z
- **Tasks:** 3
- **Files modified:** 1 (`shifty-dioxus/src/component/week_view.rs`)

## Accomplishments
- Extended `cell_background_class` to a 3-arg fn taking `paid_overage`, with paid-overage outranking understaffing (D-23-03 / D-23-04).
- Wired `paid_overage` at the `WeekCellSlot` call site UNCONDITIONALLY (not gated on `is_shiftplanner`), so every role sees the red tint (D-23-05).
- Added 3 new unit tests + 2 new SSR tests; updated the 4 existing unit tests to the new signature. All green.
- Verified all three gates: unit/SSR tests, WASM build, and clippy (zero new clippy lints from this change).

## Task Commits

Per VCS policy (jj-managed repo, `commit_docs: false`) NOTHING was committed by the executor — all edits are left uncommitted in the working copy for the user to review and commit with jj.

1. **Task 1: Extend cell_background_class + unit tests** — uncommitted (TDD: tests updated/added then fn extended)
2. **Task 2: Wire paid_overage at call site + SSR tests** — uncommitted
3. **Task 3: Plan-wide gate (test + WASM build + clippy)** — uncommitted

## Files Created/Modified
- `shifty-dioxus/src/component/week_view.rs` — 3-arg `cell_background_class` (combined red arm); call-site `paid_overage` computed via `is_some_and`; 4 existing unit tests updated, 3 new unit tests (`cell_background_class_paid_overage_is_bad_soft`, `cell_background_class_paid_overage_overrides_missing`, `cell_background_class_discourage_overrides_paid_overage`), 2 new SSR tests (`week_cell_slot_paid_overage_is_bad_soft`, `week_cell_slot_understaffed_no_overage_is_warn_soft`). Existing `filled/need` badge unchanged (D-23-03).

## Decisions Made
- **Combined-arm fn shape.** The plan specified separate `if discourage / else if paid_overage` branches both returning `"bg-bad-soft"`. clippy `-D warnings` flags this as `if_same_then_else`, so the two were merged into `if discourage || paid_overage`. Priority semantics and the documented order are unchanged; all 7 unit tests (including the all-flags-set regression test) still pass.
- **`is_some_and` over `map_or(false, ..)`.** The plan/PATTERNS specified `slot.max_paid_employees.map_or(false, |n| slot.current_paid_count > n)`. clippy `-D warnings` flags `unnecessary_map_or`; applied the clippy-suggested `is_some_and`. Identical behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] clippy::if_same_then_else in cell_background_class**
- **Found during:** Task 3 (clippy gate)
- **Issue:** The plan's literal branch structure (`if discourage { "bg-bad-soft" } else if paid_overage { "bg-bad-soft" }`) trips `clippy::if_same_then_else`, which is a hard gate (`-D warnings`).
- **Fix:** Combined into `if discourage || paid_overage { "bg-bad-soft" }`; documented priority preserved in the doc comment.
- **Files modified:** shifty-dioxus/src/component/week_view.rs
- **Verification:** `cargo clippy --workspace -- -D warnings` no longer reports this lint; all 7 unit tests pass.

**2. [Rule 1 - Bug] clippy::unnecessary_map_or at the WeekCellSlot call site**
- **Found during:** Task 3 (clippy gate)
- **Issue:** The plan-specified `map_or(false, |n| slot.current_paid_count > n)` trips `clippy::unnecessary_map_or` under `-D warnings`.
- **Fix:** Replaced with `is_some_and(|n| slot.current_paid_count > n)` (clippy's own suggestion).
- **Files modified:** shifty-dioxus/src/component/week_view.rs
- **Verification:** clippy clean for this line; SSR + unit tests still green.

---

**Total deviations:** 2 auto-fixed (both Rule 1 — clippy-gate correctness on this plan's own code).
**Impact on plan:** Both are mechanical clippy fixes on the lines this plan introduced. Behavior is identical (proven by the unchanged passing test suite). No scope creep, no logic change to the documented priority.

## Issues Encountered

- **Dev-shell toolchain mismatch (environment, not code).** The `shifty-dioxus/` `nix develop` shell ships `clippy 0.1.93` (clippy-driver rustc 1.93.0) but `rustc 1.95.0`. Running clippy there fails wholesale with `E0514` ("crate compiled by an incompatible version of rustc") for every dependency. Resolved by running clippy from the **backend-root** `nix develop` shell (matching rustc 1.93.0 + clippy 0.1.93), supplying openssl via `OPENSSL_LIB_DIR` / `OPENSSL_INCLUDE_DIR` / `OPENSSL_NO_VENDOR=1` env overrides (the backend shell lacks openssl for the dioxus crate by default). A `cargo clean` was required to drop stale 1.95-compiled artifacts. Logged in `deferred-items.md` with a recommendation to align the dioxus flake's clippy/rustc versions + openssl env.

## Deferred Issues

- **199 pre-existing crate-wide clippy lints** in `shifty-dioxus` (across ~30 files: api.rs, loader.rs, js.rs, component/atoms/*, component/*, page/*, state/*). These predate Phase 23 and are unrelated to this plan's single-file change — out of scope per the executor scope boundary. The clippy error count was 201 with this plan's raw code and 199 after fixing the 2 lints this plan introduced; the residual 199 are entirely pre-existing. Documented in `.planning/phases/23-frontend-slot-paid-capacity-ui/deferred-items.md` with a recommendation to schedule a dedicated crate-wide clippy-cleanup plan.

## Gate Results

- `cd shifty-dioxus && cargo test cell_background_class` — **PASS** (7 unit tests; 0 failed)
- `cd shifty-dioxus && cargo test week_view` — **PASS** (34 tests incl. 2 new SSR; 0 failed)
- `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` — **PASS** (Finished, exit 0)
- `cargo clippy --workspace -- -D warnings` (run from backend-root shell, see Issues) — **this plan's code introduces 0 lints**; 199 pre-existing crate-wide lints remain (deferred, out of scope).

## TDD Gate Compliance

Tasks 1 & 2 were `tdd="true"`. The executor commits nothing (user commits via jj), so RED→GREEN are not split into separate commits; however the discipline was followed: failing-test additions/updates were authored together with the minimal fn extension/call-site wiring needed to make them pass, and the full suite is green. No separate `test(...)`/`feat(...)` commit gates exist because all changes remain uncommitted by design (jj-only policy).

## Next Phase Readiness
- Plan 23-02 (slot-edit paid-capacity form + non-blocking overage banner) can proceed; the week-view coloring half of the phase is complete.
- Reviewer should commit the working-copy changes with jj before/after 23-02.

## Self-Check: PASSED

- `week_view.rs` 3-arg signature with `paid_overage: bool` — FOUND (:979)
- combined `if discourage || paid_overage` arm — FOUND (:984)
- call site `cell_background_class(missing, props.discourage, paid_overage)` — FOUND (:1063)
- `is_some_and(|n| slot.current_paid_count > n)` (unconditional, not gated) — FOUND (:1062)
- 3 new unit tests — FOUND (:707, :712, :717)
- 2 new SSR tests — FOUND (:1530, :1560)
- existing `filled/need` badge unchanged — FOUND (:1071, :1098)
- no new `bg-red-`/`text-red-` palette classes (only inside the legacy-class guard test list) — confirmed (guard test passes)
- SUMMARY.md — FOUND
- deferred-items.md — FOUND

---
*Phase: 23-frontend-slot-paid-capacity-ui*
*Completed: 2026-06-26*
