## Context

The weekly overview page (`src/page/weekly_overview.rs`) displays a year's worth of weekly staffing data in a table. The data is already loaded via `WeeklySummaryStore` and includes `paid_hours`, `volunteer_hours`, and `required_hours` per week. No additional backend work is needed.

The frontend uses Dioxus with Tailwind CSS. SVG elements can be rendered directly in RSX. The project has no charting dependencies and should stay that way.

## Goals / Non-Goals

**Goals:**
- Provide a visual overview of staffing levels across the year
- Show paid vs. volunteer hour composition per week (stacked bars)
- Show required-hours as a reference line for quick gap detection
- Work well on desktop (full chart visible) and mobile (horizontal scroll)

**Non-Goals:**
- Interactive tooltips or click-to-navigate (can be added later)
- Animations or transitions
- Printing support for the chart
- Replacing the existing table — the chart supplements it

## Decisions

### 1. Pure SVG in RSX (no chart library)

All chart rendering is done with SVG elements (`rect`, `line`, `text`, `polyline`) directly in Dioxus RSX.

**Rationale**: The chart is simple enough (bars + line) that a library would be overkill. SVG in RSX keeps the dependency footprint at zero and gives full control over styling.

**Alternatives considered**:
- Chart.js via JS interop — adds JS dependency, complicates build
- CSS-only div bars — cannot draw the required-hours reference line elegantly
- `plotters` crate with SVG backend — heavy dependency for a simple chart

### 2. Hybrid responsive layout

On viewports where all bars fit (typically desktop), the SVG scales to fill the container width via `viewBox`. When the calculated chart width exceeds the container (many weeks on small screens), a minimum bar width is enforced and the container scrolls horizontally with `overflow-x-auto`.

**Rationale**: This avoids both extremes — bars that are too thin to read on mobile, and a chart that wastes space on desktop.

**Implementation**: The component calculates `total_svg_width = num_weeks * bar_step`. If this fits the container, `width="100%"` with a matching `viewBox`. The outer `div` always has `overflow-x-auto` so scrolling activates only when needed.

### 3. Component architecture

A new `WeeklyOverviewChart` component in `src/component/weekly_overview_chart.rs` receives `&[WeeklySummary]` as a prop. All scaling math (max value, bar heights, line points) is computed in Rust inside the component function.

**Rationale**: Keeps the page file clean and the chart self-contained and testable.

### 4. Color scheme

| Element | Color | Tailwind equivalent |
|---------|-------|---------------------|
| Paid hours bar | `#3B82F6` | blue-500 |
| Volunteer hours bar | `#10B981` | emerald-500 |
| Required-hours line | `#EF4444` | red-500 |

These colors provide sufficient contrast and align with the existing Tailwind palette used in the app.

### 5. Y-axis scaling

The Y-axis maximum is derived from the data: `max(paid + volunteer, required)` across all weeks, rounded up to the next multiple of 10 for clean grid lines. This ensures the chart always fits the data without hardcoded limits.

## Risks / Trade-offs

- **[52 bars on small screens]** → Mitigated by horizontal scroll with minimum bar width. Users on mobile can swipe through weeks.
- **[SVG text rendering varies across browsers]** → Use simple font sizes and avoid rotation. Week labels use short format (e.g., "W1").
- **[No interactivity]** → Acceptable for v1. The table below provides exact numbers. Tooltips can be added later.
- **[Maintenance of hand-rolled SVG]** → The chart logic is simple (bars + line). If charting needs grow significantly, revisit the library decision.
