# Redesign 06 — Page: Jahresübersicht

> **Status**: skeleton. Master plan: `../REDESIGN_PLAN.md`. Reference: `design_handoff_shifty/README.md` (Screens § 4).

## Why

Year overview already has a working stacked SVG chart in `src/component/weekly_overview_chart.rs`. Redesign refactors it onto tokens and adds the polish from the reference (current-week highlight, dashed required-line, mono labels), plus the surrounding page layout and 10-week table.

## What Changes

- Refactor `src/component/weekly_overview_chart.rs`:
  - Replace hardcoded colors with tokens (`var(--accent)`, `var(--ink-muted)`, `var(--bad)`, `var(--good)`, `var(--warn)`)
  - Render required-hours line dashed (`stroke-dasharray`)
  - Highlight the current week — full accent fill, others slightly dimmed (opacity 0.85)
  - Mono labels (`KW 1`, `KW 13`, `KW 26`, …) with `var(--ink-muted)` color
  - Keep SVG (better than HTML divs for print and crisp scaling)
- Rewrite `src/page/weekly_overview.rs`:
  - Page header with prev/next year nav (using `NavBtn` from `02`)
  - Chart card: `surface` background, `border`, radius `lg`, padding 18 px, legend row above
  - Table card below: 10-week window around the current week, columns `Woche · Bezahlt/Freiwillig · Verfügbar/Benötigt · Differenz`, current row tinted `accent-soft`
  - Diff column color: `warn` if missing, `good` if surplus

## Out of scope

- Changing chart data shape or backend
- Adding click-to-week-detail interactions on bars

## Capabilities

### Modified
- `weekly-overview-chart`: token migration, current-week highlight, dashed required-line

## Impact

- Files: `src/component/weekly_overview_chart.rs`, `src/page/weekly_overview.rs`
- Tests: existing chart tests should still pass; add test for current-week highlight branch
