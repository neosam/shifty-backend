## Why

The weekly overview page currently displays data only as a table. Patterns like seasonal understaffing, paid-vs-volunteer ratios, and weeks that miss the required-hours target are hard to spot when scanning 52 rows of numbers. A visual chart above the table would let users grasp the full year's staffing picture at a glance.

## What Changes

- Add a stacked bar chart (SVG) rendered above the existing weekly overview table.
- Each bar represents one week, stacked into **paid hours** (bottom) and **volunteer hours** (top).
- A **required-hours reference line** overlays the bars so users can immediately see which weeks are under- or over-staffed.
- On desktop the chart scales to fit the viewport; on mobile a minimum bar width is enforced and the chart scrolls horizontally (`overflow-x: auto`).

## Capabilities

### New Capabilities
- `weekly-overview-chart`: SVG stacked bar chart component showing paid/volunteer hours per week with a required-hours reference line and responsive hybrid layout.

### Modified Capabilities
<!-- None — the existing table and data loading remain unchanged. -->

## Impact

- **New component**: `src/component/weekly_overview_chart.rs` (pure SVG in Dioxus RSX).
- **Page change**: `src/page/weekly_overview.rs` — embed the chart above the table.
- **i18n**: New translation keys for the chart legend (paid hours, volunteer hours, required hours).
- **No backend changes** — all data already available in `WeeklySummary`.
- **No new dependencies** — pure SVG, no external chart library.
