## 1. i18n Keys

- [x] 1.1 Add translation keys to `Key` enum in `src/i18n/mod.rs` (PaidHours, VolunteerHours, RequiredHours)
- [x] 1.2 Add English translations in `src/i18n/en.rs`
- [x] 1.3 Add German translations in `src/i18n/de.rs`
- [x] 1.4 Add Czech translations in `src/i18n/cs.rs`

## 2. Chart Component

- [x] 2.1 Create `src/component/weekly_overview_chart.rs` with `WeeklyOverviewChart` component
- [x] 2.2 Implement Y-axis scaling logic (max value rounded up to next multiple of 10, grid lines)
- [x] 2.3 Render stacked bars (paid bottom, volunteer top) with correct proportional heights
- [x] 2.4 Render required-hours polyline overlay
- [x] 2.5 Render X-axis week number labels
- [x] 2.6 Render Y-axis labels and horizontal grid lines
- [x] 2.7 Render legend with translated labels (paid, volunteer, required)

## 3. Integration

- [x] 3.1 Register the component module in `src/component/mod.rs`
- [x] 3.2 Embed `WeeklyOverviewChart` in `src/page/weekly_overview.rs` between year navigation and table

## 4. Responsive Layout

- [x] 4.1 Wrap chart SVG in `overflow-x-auto` container
- [x] 4.2 Calculate SVG width based on number of weeks with minimum bar step width
- [x] 4.3 Verify chart scrolls on narrow viewports and scales on wide viewports

## 5. Testing

- [x] 5.1 Add unit tests for Y-axis scaling logic (rounding, grid line calculation)
- [x] 5.2 Add unit tests for bar height calculation
- [x] 5.3 Verify all three locales have the new translation keys (compile check)
