## 1. i18n Keys

- [x] 1.1 Add `Key::PreviousYear`, `Key::NextYear`, and `Key::WeekShort` variants to the `Key` enum in `src/i18n/mod.rs`
- [x] 1.2 Add English translations in `src/i18n/en.rs`: `PreviousYear = "Previous year"`, `NextYear = "Next year"`, `WeekShort = "W"`
- [x] 1.3 Add German translations in `src/i18n/de.rs`: `PreviousYear = "Vorheriges Jahr"`, `NextYear = "Nächstes Jahr"`, `WeekShort = "KW"`
- [x] 1.4 Add Czech translations in `src/i18n/cs.rs`: `PreviousYear = "Předchozí rok"`, `NextYear = "Další rok"`, `WeekShort = "T"`

## 2. Chart Color Token Migration

- [x] 2.1 In `src/component/weekly_overview_chart.rs`, replace `fill: "#3B82F6"` on the paid-hours `<rect>` with `style: "fill: var(--accent)"`
- [x] 2.2 Replace `fill: "#10B981"` on the volunteer-hours `<rect>` with `style: "fill: var(--good)"`
- [x] 2.3 Replace `stroke: "#e5e7eb"` on the grid `<line>` with `style: "stroke: var(--border)"`
- [x] 2.4 Replace `fill: "#6b7280"` on Y-axis labels with `style: "fill: var(--ink-muted)"`
- [x] 2.5 Replace `fill: "#6b7280"` on X-axis labels with `style: "fill: var(--ink-muted)"`
- [x] 2.6 Verify no hardcoded hex color (`#xxxxxx`) remains in non-test source of `weekly_overview_chart.rs`

## 3. Required-Hours Line Restyle

- [x] 3.1 Replace `stroke: "#EF4444"` on the required-hours `<polyline>` with `style: "stroke: var(--ink-muted)"`
- [x] 3.2 Add `stroke_dasharray: "4 3"` attribute to the required-hours `<polyline>`
- [x] 3.3 Keep `stroke_width: "1.5"` and `fill: "none"` unchanged

## 4. Current-Week Highlight in Chart

- [x] 4.1 At the top of `WeeklyOverviewChart`, read `let current_year = js::get_current_year();` and `let current_week = js::get_current_week();`
- [x] 4.2 Inside the bars `for` loop, compute `let is_current = week.year == current_year && week.week == current_week;`
- [x] 4.3 Wrap each week's `<rect>` elements in a `<g>` group; set `style: "opacity: 0.85"` on the group when `!is_current` and skip the style when `is_current`
- [x] 4.4 On the current-week paid `<rect>`, append `stroke: "var(--accent)"` and `stroke_width: "1"` to the `style` attribute (combined with the existing `fill: var(--accent)`)
- [x] 4.5 Ensure non-current bars do NOT receive the accent stroke

## 5. X-Axis Labels: Mono Font, Locale Prefix, Sparse Rendering

- [x] 5.1 Read `let week_short = i18n.t(Key::WeekShort);` once near the top of the component
- [x] 5.2 Update each `<text>` for the X-axis to render `format!("{week_short} {}", week.week)` instead of the bare week number
- [x] 5.3 Add `font_family: "ui-monospace, SFMono-Regular, Menlo, monospace"` and `font_size: "9"` to each X-axis `<text>` (replacing the previous `font_size: "7"`)
- [x] 5.4 Replace the existing `show_label` rule with: `let show_label = (week.week as usize - 1) % 4 == 0 || week.week == 52 || (week.year == current_year && week.week == current_week);`
- [x] 5.5 Verify only labels matching the new rule render; the rest are skipped via `if show_label`

## 6. Move Legend Out of SVG

- [x] 6.1 Delete the in-SVG legend block (the `let legend_y = ...` rsx block emitting `<rect>` and `<text>` legend elements)
- [x] 6.2 Reduce `LEGEND_HEIGHT` to `0.0` (or remove it from the `svg_height` calculation)
- [x] 6.3 In the chart component's outer `div`, render an HTML legend row above the `<div class="overflow-x-auto">` using design-token classes
- [x] 6.4 Legend row markup: a `<div class="flex items-center gap-4 mb-2 text-xs text-ink">` containing three inline-flex spans (paid swatch + label, volunteer swatch + label, dashed line swatch + required label)
- [x] 6.5 Paid/volunteer swatches: 12×12 px boxes with inline `style: "background: var(--accent)"` / `var(--good)` and `border-radius: 2px`
- [x] 6.6 Required swatch: an inline SVG (16×8 px) containing a horizontal `<line>` with `stroke="var(--ink-muted)"`, `stroke-dasharray="4 3"`, and `stroke-width="1.5"`

## 7. Page Wrapper and Heading

- [x] 7.1 In `src/page/weekly_overview.rs`, replace the outer `<div class="m-4">` with `main { class: "mx-auto max-w-5xl w-full px-4 py-6 md:py-8 space-y-4", ... }` rendered after `TopBar {}`
- [x] 7.2 Replace the legacy `<h1 class="text-2xl font-bold mb-6">` with `h1 { class: "text-xl font-semibold text-ink", "{title}" }`
- [x] 7.3 Replace the legacy `<div class="text-center">` loading message with `div { class: "text-ink-muted px-4 py-3", "Loading data..." }`

## 8. Year Navigation Row with NavBtn

- [x] 8.1 Replace the two legacy `<button class="border-2 border-solid border-black ...">` buttons with `NavBtn` atoms
- [x] 8.2 Wrap the year-nav row in `div { class: "flex items-center gap-3 print:hidden" }`
- [x] 8.3 Render the previous-year `NavBtn { glyph: ImStr::from("‹"), aria_label: Some(ImStr::from(i18n.t(Key::PreviousYear).as_ref())), on_click: Some(EventHandler::new(move |_| cr.send(WeeklyOverviewPageAction::PreviousYear))) }`
- [x] 8.4 Render the year span `span { class: "font-mono text-lg text-ink min-w-[4ch] text-center", "{year.read()}" }`
- [x] 8.5 Render the next-year `NavBtn` symmetrically with `›` and `Key::NextYear`
- [x] 8.6 Verify `print:hidden` is on the wrapping flex `<div>` and not duplicated on the buttons

## 9. Chart Card Wrapper

- [x] 9.1 Wrap the `WeeklyOverviewChart { ... }` invocation in `section { class: "rounded-md border border-border bg-surface p-4 md:p-[18px]" }`
- [x] 9.2 Verify the legend (now inside the chart component) renders above the SVG inside this card

## 10. Table Card and 10-Week Window

- [x] 10.1 Wrap the `<table>` in `section { class: "rounded-md border border-border bg-surface overflow-hidden" }`
- [x] 10.2 Replace the existing `<table class="table-auto w-full mt-4">` with `<table class="w-full text-sm">`
- [x] 10.3 Restyle `<thead class="text-left">` to `<thead class="bg-surface-alt text-ink-muted text-left">` and table headers to `<th class="px-3 py-2 text-xs uppercase tracking-wide font-semibold">`
- [x] 10.4 Restyle `<tbody>` to `<tbody class="divide-y divide-border">`; remove `border-b border-black` from header rows and `border-b` from data rows
- [x] 10.5 Compute the visible window: `let (start, len) = if *year.read() == current_year { let cw = current_week as usize; let s = cw.saturating_sub(5); (s, 10.min(52 - s)) } else { (0, 10) };` (then `.iter().skip(start).take(len)`)
- [x] 10.6 Verify only weeks in the window render rows (chart still renders all 52)

## 11. Current-Week Row Tinting and Diff Color Tiers

- [x] 11.1 For each visible row, compute `let is_current_row = week.year == current_year && week.week == current_week;`
- [x] 11.2 Build the `<tr>` class as `if is_current_row { "content-center bg-accent-soft" } else { "content-center" }`
- [x] 11.3 Replace the four-tier diff cell with two tiers: compute `let diff = week.available_hours - week.required_hours;`
- [x] 11.4 Render the diff cell as `<td class="px-3 py-2 {color_class} font-mono tabular-nums">{sign} {abs:.2}</td>` where `color_class` is `"text-good"` for positive, `"text-warn"` for negative, `"text-ink"` for zero
- [x] 11.5 Format sign as `"+"` for positive, `"-"` for negative, empty for zero — combined with `format!("{:.2}", diff.abs())`
- [x] 11.6 Remove all `text-green-500`, `text-red-500`, `text-yellow-700` references from the page source

## 12. Restyle Other Table Cells with Tokens

- [x] 12.1 Restyle the week link cell from `pb-2 pt-2 underline` to `px-3 py-2`; the inner `<Link>` keeps its underline behavior via the link's own styling, OR switch to `text-accent hover:underline` if no underline appears by default
- [x] 12.2 Restyle the inner `<div class="font-bold">` to `<div class="font-semibold text-ink">`
- [x] 12.3 Restyle the date-range `<div>` to `<div class="text-ink-muted text-xs">` (smaller and tokenized)
- [x] 12.4 Restyle the desktop paid/volunteer cell `<td class="hidden md:table-cell">` to `<td class="hidden md:table-cell px-3 py-2 text-ink">`
- [x] 12.5 Restyle the available/required cell to `<td class="px-3 py-2 text-ink font-mono tabular-nums">`; restyle the mobile-only paid/volunteer subline `<div class="text-sm text-gray-600 block md:hidden">` to `<div class="text-xs text-ink-muted block md:hidden mt-1">`

## 13. Restyle Absences Row

- [x] 13.1 Restyle the absences row's `<td class="pl-4 pb-2 text-sm text-gray-600">` to `<td class="px-3 py-2 text-xs text-ink-muted">`
- [x] 13.2 Restyle each absence span from `mr-4` to `mr-3`
- [x] 13.3 Verify the absences row's `<tr>` does NOT receive `bg-accent-soft` even when its parent week is the current week

## 14. Tests

- [x] 14.1 Update existing chart unit tests (`test_compute_max_hours_uses_larger_of_bar_or_required`, `test_compute_max_hours_empty`, `test_grid_lines_*`, `test_y_pos`) to match any helper signature changes — most should be unchanged
- [x] 14.2 Add SSR test `chart_uses_token_styles_not_hex`: render the chart and assert the produced HTML contains `var(--accent)`, `var(--good)`, and `var(--ink-muted)` and contains none of `#3B82F6`, `#10B981`, `#EF4444`, `#e5e7eb`, `#6b7280`
- [x] 14.3 Add SSR test `chart_required_line_is_dashed`: assert the rendered HTML contains `stroke-dasharray` on the required-line polyline
- [x] 14.4 Add SSR test `chart_current_week_has_full_opacity_others_dimmed`: render with a sample where `current_week == 17` and three weeks of data; assert the markup for week 17 lacks `opacity: 0.85` while the other weeks include it
- [x] 14.5 Add SSR test `chart_current_week_has_accent_stroke`: assert the current week's bar element includes a `stroke` resolving to `var(--accent)`
- [x] 14.6 Add SSR test `chart_x_axis_labels_use_locale_prefix`: render with German i18n active and assert visible labels contain the substring `KW ` followed by a digit
- [x] 14.7 Add SSR test `chart_x_axis_labels_are_sparse`: render 52 weeks with `current_week = 27` and assert the rendered labels include weeks 1, 5, 9, 13, 17, 21, 25, 27, 29, 33, 37, 41, 45, 49, and 52 only
- [x] 14.8 Add SSR test `chart_legend_in_html_not_svg`: assert no `<rect>` legend swatches appear inside the SVG; assert HTML legend swatches appear in a `flex items-center` container
- [x] 14.9 Add SSR test `page_uses_navbtn_for_year_nav`: assert no `border-2 border-solid border-black` substring in the page output and that two `NavBtn` instances render with the expected `aria-label` translations
- [x] 14.10 Add SSR test `page_table_window_centers_on_current_week`: with `current_week = 27`, displayed year == current year, and 52 weeks of mock data, assert exactly 10 `<tr>` data rows render with weeks 23–32
- [x] 14.11 Add SSR test `page_table_window_clamps_at_year_start`: with `current_week = 2`, assert rows render for weeks 1 through 7 (no padding from previous year)
- [x] 14.12 Add SSR test `page_table_window_clamps_at_year_end`: with `current_week = 51`, assert rows render for weeks 47 through 52
- [x] 14.13 Add SSR test `page_table_different_year_shows_first_10`: with displayed year = 2024 and current year = 2026, assert the table renders weeks 1–10 of 2024 with no `bg-accent-soft` on any row
- [x] 14.14 Add SSR test `page_current_row_has_accent_soft_tint`: with the displayed and current year matching and `current_week = 27`, assert the row for week 27 contains `bg-accent-soft` and the other rows do not
- [x] 14.15 Add SSR test `page_diff_column_two_tier_colors`: render rows with diff +3.00 / -5.00 / -25.00 / 0.00 and assert classes are `text-good`, `text-warn`, `text-warn`, neither (no `text-bad` / `text-yellow-700` / `text-red-500`)
- [x] 14.16 Add SSR test `page_absences_row_uses_tokens_and_no_tint`: with the current week containing absences, assert the absences `<td>` includes `text-ink-muted` and `text-xs` and the `<tr>` does NOT include `bg-accent-soft`
- [x] 14.17 Add SSR test `page_source_does_not_use_legacy_classes`: read the non-test source of `src/page/weekly_overview.rs` and assert it contains none of `bg-gray-100`, `bg-white`, `text-gray-500`, `text-gray-600`, `text-green-500`, `text-red-500`, `text-yellow-700`, `border-black`, `border-2 border-solid`
- [x] 14.18 Add unit test `i18n_week_short_returns_locale_value`: `WeekShort` returns `"W"` (En), `"KW"` (De), `"T"` (Cs); `PreviousYear` and `NextYear` are non-empty in all three locales

## 15. Verification

- [x] 15.1 `cargo check` passes in `shifty-dioxus`
- [x] 15.2 `cargo test --package shifty-dioxus` passes; all new tests in §14 are green
- [x] 15.3 `cargo clippy --no-deps --package shifty-dioxus` produces no new warnings in `src/component/weekly_overview_chart.rs` or `src/page/weekly_overview.rs`
- [x] 15.4 `cargo fmt -- --check` passes
- [x] 15.5 Manual smoke test: start the dev server (Tailwind watcher + `dx serve`); load `/weekly-overview` in light mode and dark mode; verify chart bars use the accent/good tokens, the required line is dashed grey, the current week is highlighted, X-axis labels show `KW <n>` (or locale equivalent), and the table shows ten rows centered on the current week with the row tinted
- [x] 15.6 Manual print preview: confirm year nav is hidden and chart + table render cleanly
- [x] 15.7 `openspec validate "redesign-06-page-overview" --strict` passes
