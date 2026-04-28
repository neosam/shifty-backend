## Context

`src/page/weekly_overview.rs` is the year-overview page. The current implementation:

- Wraps the page in `<div class="m-4">`, renders an `<h1 class="text-2xl font-bold mb-6">` heading.
- Renders the year navigation as two `<button class="border-2 border-solid border-black ...">` legacy buttons, with the year between them inside a plain `<span>`.
- Mounts the existing `WeeklyOverviewChart` between the year nav and a `<table class="table-auto w-full mt-4">` that lists all 52 ISO weeks.
- Colors the diff column inline using `text-green-500`, `text-red-500`, and `text-yellow-700` legacy classes, with three deficit thresholds (>20 h → red, >7 h → yellow, otherwise neutral).
- Renders an absences subtitle row using `text-gray-600` when a week has any absences.

`src/component/weekly_overview_chart.rs` is the SVG chart consumed by the page. The current implementation:

- Uses hardcoded hex colors: `#3B82F6` for paid hours, `#10B981` for volunteer, `#EF4444` for the required-hours line, `#e5e7eb` and `#6b7280` for grid lines, and `#374151` for legend text.
- Renders the required-hours line as a solid 1.5 px polyline.
- Treats every bar identically — no current-week emphasis.
- X-axis labels are bare numbers (`1`, `2`, ... `52`) at `font-size="7"`, with every-other-week culling for ≥ 26-week ranges.
- Bar `fill` is set on the `<rect fill="..."/>` SVG attribute directly.

The reference design (`design_handoff_shifty/README.md` § 4) reframes this page as two stacked cards:

1. A chart card with surface background, border, and a legend row above the SVG. Bars use design tokens; the current week pops, others dim. The required line is dashed.
2. A table card showing a 10-week window centered on the current week, with columns `Woche · Bezahlt/Freiwillig · Verfügbar/Benötigt · Differenz`. The current week's row is tinted `accent-soft`. The diff column collapses to two color tiers: `warn` (deficit) and `good` (surplus).

This page follows the same migration pattern as `redesign-05-page-myshifts`: token migration first, layout polish second, no backend changes.

### Data shape constraints

`WeeklySummary` (in `state::weekly_overview`) carries: `week`, `year`, `available_hours`, `required_hours`, `paid_hours`, `volunteer_hours`, per-day available hours, and `sales_person_absences`. The chart already consumes everything it needs. No new fields are required.

The current ISO week is available via `js::get_current_week() -> u8` and the current year via `js::get_current_year() -> u32`. Both are already imported in the page.

### CSS variable visibility in SVG

SVG element `fill="..."` and `stroke="..."` attributes do not resolve `var(--token)`. The workaround is to use the `style:` attribute (`style: "fill: var(--accent)"`), which the browser resolves via the standard CSS cascade. All chart elements migrate to `style:` for color attributes; geometry attributes (`x`, `y`, `width`, `height`, `points`) stay as plain attributes.

## Goals / Non-Goals

**Goals:**
- Refactor `WeeklyOverviewChart` to consume CSS variables (`var(--accent)`, `var(--ink-muted)`, `var(--good)`, `var(--warn)`, `var(--bad)`, `var(--border)`, `var(--surface)`).
- Render the required-hours line dashed using `stroke-dasharray="4 3"`, in `var(--ink-muted)` rather than red.
- Highlight the current week — full-opacity bars; other weeks render at opacity `0.85`.
- Migrate X-axis labels to `KW <n>` format in monospace font, with sparse rendering (every 4th week plus the first, last, and current week always).
- Rewrite `WeeklyOverview` page to a stacked card layout: page heading, year nav using `NavBtn`, chart card, table card.
- Limit the table to a 10-week window centered on the current week, clamped to `[1, 52]` of the displayed year.
- Tint the current week's table row `accent-soft`.
- Collapse diff column color rules to two tiers: `text-warn` for deficit, `text-good` for surplus, neutral for exactly zero.
- Preserve existing chart interactions (click a bar to navigate to `/shiftplan/{year}/{week}`, tooltip on hover).
- Preserve existing print styles (`print:hidden` on year nav).

**Non-Goals:**
- No changes to `WeeklySummary`, `loader.rs`, `api.rs`, or backend.
- No changes to chart data shape (still stacked paid/volunteer with required line).
- No new chart interactions (no per-bar context menu, no zoom).
- No restructuring into HTML divs — chart stays SVG (better print scaling).
- No carry-over of the 10-week window across year boundaries (December weeks 49–52 just clamp to weeks 43–52, not weeks 49–52 plus weeks 1–5 of the next year).
- No restoration of the legacy ">20 h deficit → red" tier (intentionally simplified per proposal).
- No expandable absences row redesign — keep the inline list but restyle to tokens.

## Decisions

### 1. Capability split: keep `weekly-overview-chart` and add `weekly-overview-page`

The proposal lists only `weekly-overview-chart` as Modified. But the page-level concerns (year-nav layout, 10-week table, card surfaces, diff color tiering, current-row tinting) are not chart concerns. Putting them under `weekly-overview-chart` would conflate the SVG component with its host page.

**Decision**: split spec deltas into two capabilities:
- MODIFY `weekly-overview-chart`: visual changes inside the SVG (tokens, dashed line, current-week highlight, mono labels).
- ADD `weekly-overview-page` (new capability): page-level concerns (year nav, card layout, 10-week table window, current-row tinting, diff color rules).

This deviates from the proposal's capability list but keeps each spec cohesive. Sister change `redesign-05-page-myshifts` set the same precedent by introducing `my-shifts-page` even though that capability did not previously exist. The deviation is recorded here so the next reviewer doesn't expect a single chart-only spec.

### 2. Current week detection: `(js::get_current_year(), js::get_current_week())` with type coercion

Read both once at the top of the chart component and once at the top of the page component. `WeeklySummary.year` is a `u32` and `WeeklySummary.week` is a `u8`, matching the JS helpers' return types. A bar / row matches the current week when both fields equal.

**Edge case**: when the displayed year (signal) differs from the calendar year (e.g. user navigated to 2025 while the real today is in 2026), no bar or row matches, and no highlight or tinting renders. The 10-week table in this case shows weeks 1–10 of the displayed year (the simplest deterministic fallback — see decision 4).

### 3. Required-line styling: dashed `var(--ink-muted)`

Old: solid 1.5 px `#EF4444`. The red made the required line as visually loud as the bars themselves, which the reference design moves away from. New: `stroke="..." stroke-dasharray="4 3" stroke-width="1.5"` with stroke set via `style:` attribute. Color choice: `var(--ink-muted)` — neutral grey, dashed pattern carries the "this is a target line, not data" signal without competing with the accent.

The legend swatch updates to match: a short horizontal line with the same dash pattern.

### 4. 10-week table window: centered on current week, clamped to `[1, 52]`

Logic:
- If displayed year == current year: window = `[max(1, current_week - 4), min(52, current_week + 5)]` (10 weeks, possibly fewer near year edges).
- Otherwise: window = `[1, 10]` of the displayed year.

Using `take(10)` after a `skip(start)` on the existing `weekly_summary.weekly_summary` iterator avoids any re-indexing or boundary math beyond the bounds. The "10 weeks" target is approximate at year edges (e.g. December shows weeks 43–52 = 10 weeks; week 51 of a 52-week year shows weeks 47–52 = 6 weeks). Acceptable trade-off — no spillover across years.

### 5. Diff column: two tiers, `text-warn` and `text-good`

Old: 4 tiers (surplus / >20 h deficit / >7 h deficit / minor deficit).
New per proposal: 2 tiers — `text-warn` if `diff < 0`, `text-good` if `diff > 0`. Diff exactly zero renders neutral `text-ink`.

`diff = available_hours - required_hours` (positive = surplus). Sign formatting: surplus shows `+ X.XX`, deficit shows `- X.XX`, zero shows `0.00`. Use `format!("{:.2}", diff.abs())` to avoid double-sign rendering.

The simplification drops the "alarmingly understaffed" red tier. Reasoning per proposal: the chart's dashed required-line and the bar heights already convey severity at a glance; the table is a secondary view. If users miss the third tier, a follow-up change can reintroduce a `text-bad` threshold.

### 6. Chart card layout

Outer card wraps the legend row + SVG container:
```
section { class: "rounded-md border border-border bg-surface p-4 md:p-[18px]",
    div { class: "flex items-center gap-4 mb-2", /* legend swatches */ }
    div { class: "overflow-x-auto", svg { ... } }
}
```

The legend moves out of the SVG and into HTML. Reasons:
- HTML legend uses tokenized text colors (`text-ink`, `text-ink-muted`) that automatically theme.
- HTML legend is keyboard-focusable and screen-reader-friendly (the previous SVG legend was decorative).
- Frees up `LEGEND_HEIGHT = 30 px` of vertical space inside the SVG.

The SVG still renders the bars, grid, required line, and X-axis labels. Y-axis values stay inside the SVG (geometry-bound to grid lines).

### 7. Table card layout

Same surface pattern as chart card:
```
section { class: "rounded-md border border-border bg-surface overflow-hidden",
    table { class: "w-full text-sm",
        thead { class: "bg-surface-alt text-ink-muted text-left", ... }
        tbody { class: "divide-y divide-border", ... }
    }
}
```

Headers in `text-ink-muted text-xs uppercase tracking-wide` for the design system's "table head" style. Rows use `divide-y divide-border` instead of `border-b border-black`.

Current-week row: append `bg-accent-soft` to the row's class. The tint reads in both light and dark themes via the `--accent-soft` token.

### 8. Year navigation row using `NavBtn`

Replace the legacy two-button row with:
```
div { class: "flex items-center gap-3 print:hidden",
    NavBtn { glyph: ImStr::from("‹"), aria_label: Some(ImStr::from("Previous year")), on_click: Some(...) }
    span { class: "font-mono text-lg text-ink min-w-[4ch] text-center", "{year}" }
    NavBtn { glyph: ImStr::from("›"), aria_label: Some(ImStr::from("Next year")), on_click: Some(...) }
}
```

Glyphs `‹` / `›` (single guillemets) match the reference design and what `NavBtn`'s docstring uses as canonical glyphs. `aria_label` strings are translated via two new i18n keys (`PreviousYear`, `NextYear`) added to all three locales.

`print:hidden` stays on the wrapper so the printed year overview shows just the data.

### 9. X-axis label format and density

Format: `KW <n>` (e.g. `KW 1`, `KW 13`). Font: monospace via the SVG `font-family` attribute (`ui-monospace, SFMono-Regular, Menlo, monospace` — match Tailwind's `font-mono`). Color: `var(--ink-muted)` via `style:` attribute. Size: `font-size="9"` (slightly larger than the old `7` because labels are now wider).

Density rule: render label when `(week.week - 1) % 4 == 0` (so KW 1, 5, 9, ..., 49) **OR** `week.week == 52` (always show the final week) **OR** `(year, week) == (current_year, current_week)` (always show the current week).

This produces a stable label cadence (~13 visible labels for a 52-week chart) plus the user's "you are here" marker. The bar itself still receives a `<title>` tooltip on hover, so dense inspection per bar still works.

### 10. Current-week opacity via inline `style:`

Bars for non-current weeks render with `style: "opacity: 0.85"` on the `<g>` wrapper around the rects. Current-week bars get `style: "opacity: 1"` (or omit the style entirely). The opacity multiplies the fill, so the accent token still reads as accent — just dimmer. Reference recommends `0.85` because lower values risk losing the paid/volunteer color separation in dark mode.

The current-week bar also receives a thin border to emphasize: `stroke: "var(--accent)"` `stroke_width: "1"` on its `<rect>`. (Skipped for non-current weeks to keep the SVG light.)

### 11. Color token mapping inside the SVG

| Element | Old | New |
|---|---|---|
| Paid hours bar | `#3B82F6` | `var(--accent)` |
| Volunteer hours bar | `#10B981` | `var(--good)` |
| Required line | `#EF4444` solid | `var(--ink-muted)` dashed |
| Grid line | `#e5e7eb` | `var(--border)` |
| Y-axis label | `#6b7280` | `var(--ink-muted)` |
| X-axis label | `#6b7280` | `var(--ink-muted)` |
| Legend text (now HTML) | `#374151` | `text-ink` (Tailwind class) |

The volunteer bar moves from `green-500` to `--good`, which in the project's tokens is `#0e7a4d` (light) and `#4ed59a` (dark). Slightly different green than before — accepted because consistency with the rest of the design system is more important than literal hex preservation.

### 12. i18n: minimal new keys

Three new translation keys cover the redesign:

- `Key::PreviousYear` — aria label for the prev-year `NavBtn` (`"Previous year"` / `"Vorheriges Jahr"` / `"Předchozí rok"`).
- `Key::NextYear` — aria label for the next-year `NavBtn` (`"Next year"` / `"Nächstes Jahr"` / `"Další rok"`).
- `Key::WeekShort` — short prefix for the X-axis label, used to build `KW <n>` (`"W"` / `"KW"` / `"T"`).

Existing keys (`Paid`, `Volunteer`, `ChartRequiredHours`, `WeekLabel`, `PaidVolunteer`, `AvailableRequiredHours`, `MissingHours`, `HoursShort`) are reused.

### 13. Absences row restyle (no logic change)

Keep the conditional `if !week.sales_person_absences.is_empty()` row but restyle:
- Wrapper `<tr>` row inherits the table's `divide-y divide-border` separation.
- Inner `<td colspan="4">` switches from `text-gray-600 pl-4 pb-2 text-sm` to `text-ink-muted px-3 py-2 text-xs`.
- Each absence span goes from inline `mr-4` to `mr-3` (tightens to match the denser table).

If the surrounding row is the current week (`bg-accent-soft`), the absences row inherits no tint — keep it neutral so the eye reads them as a footnote, not part of the week's primary line.

## Risks / Trade-offs

**[CSS variables in SVG attributes don't resolve via `fill="var(...)"`]** → Use `style: "fill: var(--accent)"` for every colored SVG element. Mitigation: the migration is mechanical and unit-testable. Verified by inspecting how the rest of the codebase tokenizes SVG (none currently — this is the first SVG that uses tokens). Manual smoke test in both light and dark themes after the migration.

**[`var(--good)` shifts the volunteer-bar hue]** → Old `#10B981` (Tailwind emerald) versus new `#0e7a4d` (light) / `#4ed59a` (dark). The shift is intentional and aligned with the design system. If users complain, add a `--volunteer-bar` token alias rather than reverting to the hardcoded hex.

**[10-week window at year edges shows fewer than 10 weeks]** → December (weeks 49–52) shows weeks 43–52 = 10 ok; weeks 51–52 shows weeks 47–52 = 6 weeks. Acceptable: the chart still shows all 52 weeks above. If this proves confusing in practice, the fallback could pad with the next year's weeks 1–N, but that mixes two years in one table — strictly worse for the print-friendly use case.

**[Sparse X-axis labels may obscure dense-inspection use]** → Showing 13 labels for 52 weeks means most bars have no immediate label. Mitigation: hover tooltip already shows the week number; the current-week label is always rendered, anchoring the user. Manual reviewer test: hover three random bars and confirm the tooltip lists the correct week.

**[Diff color simplification may surprise managers used to red alerts]** → The legacy `text-red-500` for >20 h deficit goes away. Mitigation: warn-orange is still distinguishable from neutral, and the chart's dashed required-line plus bar heights make severe under-staffing visible at a glance. If feedback demands the third tier, reintroduce as a `text-bad` band — non-breaking change.

**[Print styles need verification]** → `print:hidden` on the legacy year-nav `<button>` was on each individual button. New version puts `print:hidden` on the wrapping flex `<div>`. Same effect, slightly cleaner. Manual print preview confirms the chart and table render without the nav.

**[Dark-mode contrast of dimmed bars]** → Opacity `0.85` over `var(--accent)` in dark mode (`#8b97ff`) blends toward the dark surface. If contrast falls below 3:1 against `--surface` (`#16191f`), bump to `0.9`. Verified via the design tokens' WCAG matrix — `0.85` stays above 3:1 for both bar colors against both surface variants.
