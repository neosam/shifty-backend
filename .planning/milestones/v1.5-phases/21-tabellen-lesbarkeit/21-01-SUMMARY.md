---
phase: 21-tabellen-lesbarkeit
plan: 01
subsystem: frontend
tags: [ui, tailwind, css, tables, zebra, max-width]
dependency_graph:
  requires: []
  provides: [UI-01-max-width-zebra, UI-02-narrow-employee-column]
  affects: [shifty-dioxus/src/component/working_hours_mini_overview.rs, shifty-dioxus/src/page/absences.rs]
tech_stack:
  added: []
  patterns: [static-tailwind-class-strings, tdd-red-green]
key_files:
  created: []
  modified:
    - shifty-dioxus/src/component/working_hours_mini_overview.rs
    - shifty-dioxus/src/page/absences.rs
decisions:
  - "D-21-01: max-w-5xl mx-auto added to TableLayout container div class string"
  - "D-21-02: 1.5fr replaced with 200px at all three grid-cols locations in absences.rs"
  - "D-21-03: Pure Tailwind/CSS change only — no logic, props, or i18n"
metrics:
  duration: ~20min
  completed: 2026-06-26
  tasks_completed: 2
  tasks_total: 2
---

# Phase 21 Plan 01: Tabellen-Lesbarkeit Summary

Tailwind-only readability polish: `max-w-5xl mx-auto` + 3-way zebra striping on WorkingHoursMiniOverview TableLayout, and `200px` narrower employee column (replacing `1.5fr`) at all three consistent grid-cols sites in the absences page.

## Tasks Completed

| Task | Name | Files | Key Changes |
|------|------|-------|-------------|
| 1 | UI-01: max-width + Zebra-Striping | working_hours_mini_overview.rs | Added `max-w-5xl mx-auto` to container; `.enumerate()` + 3-way static match for zebra; 3 new tests |
| 2 | UI-02: Schmalere Mitarbeiter-Spalte | absences.rs | Replaced `1.5fr` with `200px` at Header (L.1632), HourlyMarkerRow (L.1732), AbsenceListRow (L.1836) + updated comment at L.1730; 1 new source-sweep test |

## Exact Classes Applied

**UI-01 — WorkingHoursMiniOverview TableLayout:**
- Container: `"bg-surface border border-border rounded-lg overflow-hidden select-none max-w-5xl mx-auto"` (added `max-w-5xl mx-auto`)
- Even rows (idx % 2 == 0): `"border-t border-border bg-surface-2 cursor-pointer hover:bg-surface-alt"`
- Odd rows: `"border-t border-border bg-surface cursor-pointer hover:bg-surface-alt"`
- Selected rows (wins over zebra): `"border-t border-border bg-accent-soft cursor-pointer"` (unchanged)
- Total row: unchanged (`bg-surface-alt`)

**UI-02 — absences.rs grid-cols (all three sites identical):**
- Before: `grid-cols-[1.5fr_170px_140px_90px_70px]`
- After: `grid-cols-[200px_170px_140px_90px_70px]`

## All Three Grid-Cols Sites Match

Confirmed: lines 1632, 1732, 1836 all carry `grid-cols-[200px_170px_140px_90px_70px]`.
Comment at line 1730 also updated from `1.5fr` to `200px`.

## Test Results

- `cargo test` (full frontend suite): **649 passed, 0 failed**
- New tests added: 3 (working_hours_mini_overview) + 1 (absences) = 4 total
- All existing tests remain green

## WASM Build Gate

`cargo build --target wasm32-unknown-unknown` fails with `linker 'lld' not found` — this is a **pre-existing environment issue** (lld not installed in current shell). The error is not related to the code changes (pure class string modifications). No `lld`-requiring change was introduced; the flake's devShell does not supply lld in the current environment. The compilation phase (up to linking) produces only unrelated pre-existing warnings.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test assertion for zebra row needed refinement**
- **Found during:** Task 1 RED phase
- **Issue:** Initial `table_layout_zebra_even_row_has_surface2` test checked `html.contains("bg-surface-2")` which passed even before implementation because `bg-surface-2` appears in the progress bar `<div>` inside each row.
- **Fix:** Changed to check `<tr>` opening tag attributes specifically using `split("<tr").skip(1).filter_map(|s| s.split('>').next())`.
- **Files modified:** working_hours_mini_overview.rs (test only)

**2. [Rule 1 - Bug] Test split for absences.rs production-code check needed refinement**
- **Found during:** Task 2 GREEN phase
- **Issue:** `source.split("#[cfg(test)]").next()` split at the first occurrence which appeared in a doc comment on line 16 (`//! ... \`#[cfg(test)]\` module ...`), stripping all production code.
- **Fix:** Changed to `source.split("\n#[cfg(test)]\nmod tests").next()` which splits at the actual test module declaration.
- **Files modified:** absences.rs (test only)

## Known Stubs

None.

## Threat Flags

None. Changes are purely CSS class string modifications with no new network endpoints, auth paths, or schema changes.

## Self-Check: PASSED

- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/component/working_hours_mini_overview.rs` — exists, contains `max-w-5xl`
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/page/absences.rs` — exists, contains 3x `grid-cols-[200px_170px_140px_90px_70px]` in production code, 0x `1.5fr` in production code
- `cargo test` — 649 passed, 0 failed
