# Redesign 07 — Page: Mitarbeiter

> **Status**: skeleton. Master plan: `../REDESIGN_PLAN.md`. Reference: `design_handoff_shifty/README.md` (Screens § 5).

## Why

Mitarbeiter is the largest layout overhaul: two-column master/detail with a new 3-column sub-grid in the detail and a brand-new weekly histogram. Also where `ContractModal` and `ExtraHoursModal` live.

## What Changes

- Rewrite `src/page/employees.rs`:
  - Two-column layout (list 280–360 px, detail flex-grow)
  - List: searchable, color-dot avatar (no initials, no text inside circle), name, hours/target mono, active row tinted `accent-soft` with 3 px `accent` left border
  - Mobile (`<720px`): single column with back button
- Rewrite `src/page/employee_details.rs` and/or `src/component/employee_view.rs`:
  - Header row: large color circle, name, type pill (Bezahlt/Freiwillig), target hours, year nav, `Sonstige Stunden` button, `Mehr ▾`
  - 3-column sub-grid (auto-fit, min 280 px):
    1. **Gesamtansicht** — `TupleRow` stacks: Stundenkonto, Gesamt, Soll, then dim breakdown
    2. **Arbeitsverträge + Stunden pro Woche** — clickable contract cards, then a 17-bar SVG histogram (`warn` color if under target). Clicking a bar opens an inline week-detail panel.
    3. **Zusatzarbeit** — list of extra-hours entries
  - **Important — week detail panel keeps current behavior**: shows hours-per-day with category, **not** time blocks (decided in plan; `WorkingHoursDay { date, hours, category }` stays as-is)
- New component: `EmployeeWeeklyHistogram` — 17 bars, dashed Soll line, click-to-select, inline detail panel below
- Implement `ContractModal` and `ExtraHoursModal` using `Modal` and form atoms from `04`

## Out of scope

- Block-based time view in week detail (explicitly out, see master plan)
- Backend changes

## Capabilities

### Modified
- `employees-page` / `employee-details-page` (create capability names as needed)

### New
- `employee-weekly-histogram`: clickable bar histogram with inline detail

## Impact

- Files: `src/page/employees.rs`, `src/page/employee_details.rs`, `src/component/employee_view.rs`, new `src/component/employee_weekly_histogram.rs`, modal sub-components
- Tests: histogram bar click → detail open, contract modal save round-trip, extra-hours form validation
