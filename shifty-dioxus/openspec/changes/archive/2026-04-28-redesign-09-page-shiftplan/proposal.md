# Redesign 09 — Page: Schichtplan

> **Status**: skeleton. Master plan: `../REDESIGN_PLAN.md`. Reference: `design_handoff_shifty/README.md` (Screens § 2).

## Why

The most complex screen — week grid with sticky time column, person chips, mini overview, booking log. Done last so all primitives (tokens, atoms, top bar, modals) are in place.

The interaction model (`Du bearbeitest:` dropdown drives a single +/− button per cell) is **already implemented**; only the rendering changes.

## What Changes

- Rewrite toolbar in `src/page/shiftplan.rs`:
  - Prev/next-week buttons (`NavBtn`) + week label (`KW 17 · 20.04 – 26.04`)
  - View toggle Woche/Tag — segmented control inside `surface-alt` pill
  - `Letzte Woche` copy button + iCal export (existing actions, restyled)
  - `Du bearbeitest:` select using new form atoms
- Refactor `src/component/shiftplan_tab_bar.rs`:
  - Flat underline tabs with accent active state
- **Refactor `src/component/week_view.rs` — main visual change**:
  - CSS Grid: `grid-template-columns: 76px repeat(N, minmax(140px, 1fr))`, `min-width: 920px` (replaces current flex/zoom approach where compatible — keep zoom dropdown for accessibility)
  - Header row: empty corner cell (`z-index: 3`) + day cells (long name + date + day total in mono)
  - Body cells: mono `filled/need` count, `PersonChip`s, **single +/− button** absolutely positioned `top: 6, right: 6`, 20×20
    - Show `+` (neutral) when current `editing` person is **not** in cell, show `−` (`bad`-tinted) when they **are**
    - Per-chip × buttons removed (chips become display-only)
  - Sticky time column: `position: sticky; left: 0; z-index: 2`
  - Cell padding: `6px 32px 6px 8px` (right padding reserved for absolute button)
  - State backgrounds: missing-staff `bg-warn-soft`; hover slightly darker; **no per-cell red** (column-wide red optional, decided OUT in master plan)
- Working-hours mini overview (`src/component/working_hours_mini_overview.rs`):
  - Auto-fit grid card per SalesPerson with color dot, name, current/target hours, thin progress bar (`warn` if under, `good` if at/over)
- Booking log table (`src/component/booking_log_table.rs`):
  - Restyle on tokens, deleted rows at 50% opacity with `bad`-tinted `Gelöscht` cell
- Adapt `src/component/slot_edit.rs` modal to use `Modal` + form atoms from `04`

## Out of scope

- Per-cell red conflict tinting (out by master-plan decision)
- Tag (single-day) view logic changes (only restyling)
- Backend changes

## Capabilities

### Added
- `shiftplan-page`: redesigned shiftplan week-grid page (toolbar, tab bar, week grid with single +/− cell button, working-hours mini overview, slot-edit dialog, booking-log table) — captures all page-level requirements introduced by the redesign

### Modified
- `shiftplan-day-view`: visual-only token sweep (no requirement-level changes)
- `shiftplan-edit-dialog`: switched to new `Dialog` + form atoms (no requirement-level changes; visual implementation only)
- `shiftplan-catalog`: tab bar restyled (no requirement-level changes; visual only)

## Impact

- Files: `src/page/shiftplan.rs`, `src/component/week_view.rs`, `src/component/shiftplan_tab_bar.rs`, `src/component/working_hours_mini_overview.rs`, `src/component/booking_log_table.rs`, `src/component/slot_edit.rs`
- Largest visual change in the project; QA pass on print stylesheet required
- Tests: cell single-button branching (in/out of editing person), sticky time column scroll behavior, conflict list still renders above grid
