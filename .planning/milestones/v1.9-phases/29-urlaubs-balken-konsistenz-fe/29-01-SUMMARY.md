---
phase: 29-urlaubs-balken-konsistenz-fe
plan: "01"
subsystem: frontend
tags: [vacation-bar, bugfix, pure-function, unit-tests, tailwind-static-classes]
requirements: [VAC-01]

dependency_graph:
  requires: []
  provides: [compute_vacation_bar helper, PersonVacationCard bar-fix]
  affects: [shifty-dioxus/src/page/absences.rs]

tech_stack:
  added: []
  patterns:
    - pure helper extracted from component for testability (mirrors compute_status pattern)
    - static Tailwind class match (Pitfall 5 preserved)

key_files:
  modified:
    - path: shifty-dioxus/src/page/absences.rs
      description: "Added compute_vacation_bar pure helper + 5 unit tests; replaced inline bar formula in PersonVacationCard with helper call"

decisions:
  - "D-29-01: Numerator changed from used_days to (used_days + planned_days) — bar and remaining_days now measure the same quantity"
  - "D-29-02: Clamp to 0..=100 retained; overflow signalled by amber color (bg-warn) + negative remaining number, not physical overflow"
  - "D-29-03: Single low = remaining_days <= 3.0 flag drives both bar color and number color; covers overdraw (remaining < 0) as a subset"
  - "Pitfall 5: static-class match (bg-warn/bg-good, text-warn/text-good) preserved as literal strings, never interpolated"

metrics:
  duration_minutes: 8
  completed_date: "2026-06-29"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 1

status: complete
---

# Phase 29 Plan 01: Urlaubs-Balken-Konsistenz (FE) Summary

## One-liner

Fixed vacation bar formula from `used_days/total` to `(used_days+planned_days)/total` via extracted pure helper `compute_vacation_bar`, with 5 unit tests covering overdraw, good-path, boundary, negative-remaining, and zero-total fixtures.

## Tasks Completed

| Task | Name | Status | Key Changes |
|------|------|--------|-------------|
| 1 | Extract pure helper compute_vacation_bar + unit tests (RED→GREEN) | done | Added `fn compute_vacation_bar`, 5 `#[cfg(test)]` tests + fixture builder |
| 2 | Wire compute_vacation_bar into PersonVacationCard render | done | Replaced inline `let low` + `let total` + `let used_pct` block with single helper call |

## Changes Made

### `shifty-dioxus/src/page/absences.rs`

**New pure helper** (placed just before `PersonVacationCardProps`):

```rust
fn compute_vacation_bar(b: &VacationBalance) -> (u32, bool) {
    let total = b.entitled_days + (b.carryover_days as f32);
    let fill_pct: u32 = if total > 0.01 {
        ((b.used_days + b.planned_days) / total * 100.0).clamp(0.0, 100.0) as u32
    } else { 0 };
    let low = b.remaining_days <= 3.0;
    (fill_pct, low)
}
```

**PersonVacationCard render** — replaced the three-binding inline block with:

```rust
let (used_pct, low) = compute_vacation_bar(&props.balance);
```

**Unit tests** added to existing `#[cfg(test)] mod tests`:
- `compute_vacation_bar_overdraw_fills_and_sets_low` — 19/18 clamped → 100 + low=true (D-29-01/02)
- `compute_vacation_bar_good_case_33_pct_not_low` — 6/18 → 33 + low=false (D-29-01/03)
- `compute_vacation_bar_boundary_remaining_3_is_low` — remaining=3.0 → low=true (≤ threshold)
- `compute_vacation_bar_negative_remaining_is_low` — remaining=-5 → low=true (D-29-03)
- `compute_vacation_bar_zero_total_guard` — entitled=0, carryover=0 → fill=0, no panic (T-29-02)

## Verification Gates

| Gate | Command | Result |
|------|---------|--------|
| Task 1 unit tests | `cd shifty-dioxus && cargo test compute_vacation_bar` | 5/5 passed |
| FE WASM build | `nix develop --command cargo build --target wasm32-unknown-unknown` | SUCCESS |
| Backend clippy hard-gate | `cargo clippy --workspace -- -D warnings` (backend root) | CLEAN (0 warnings) |
| Full FE test suite | `cd shifty-dioxus && cargo test` | 683 passed, 0 failed |

## Deviations from Plan

None — plan executed exactly as written. The `lld` linker is not on bare PATH (NixOS), so the WASM gate was run via `nix develop` as documented in CLAUDE.local.md.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced. Only client-side arithmetic on already-present FE state fields.

## Self-Check: PASSED

- [x] `fn compute_vacation_bar` exists in absences.rs (grep returns 6 lines: 1 definition + 5 test functions with matching prefix)
- [x] `compute_vacation_bar(&props.balance)` wired in PersonVacationCard (grep returns 1)
- [x] Static class literals `bg-warn`/`bg-good`/`text-warn`/`text-good` preserved at lines 884/886
- [x] All 5 new tests pass; full suite 683/683 green
- [x] FE WASM build succeeds (via nix develop)
- [x] Backend clippy clean
- [x] No commits made; working tree left dirty for user's jj commit
- [x] STATE.md / ROADMAP.md untouched
