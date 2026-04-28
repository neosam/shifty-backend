# Capability: Employee Weekly Histogram

## Purpose
Provide a per-week comparison of worked hours against contract-defined expected hours on the employee detail page. Covers the histogram chart (one bar per ISO week of the loaded year), its stepped reference line of expected hours, per-bar coloring relative to the same week's expectation, and the current-week summary pill rendered in the page header.
## Requirements
### Requirement: Histogram bar color uses per-week expected hours

The employee weekly histogram SHALL color each bar based on the `expected_hours` value of the same week's `WorkingHours` entry, not a single scalar value derived from any contract.

A bar SHALL render in `var(--warn)` when its week's `overall_hours` is strictly less than the same week's `expected_hours`. A bar SHALL render in `var(--accent)` when its week's `overall_hours` is greater than or equal to the same week's `expected_hours`.

#### Scenario: Bar below per-week expected renders in warn

- **WHEN** a week has `overall_hours = 15.0` and `expected_hours = 20.0`
- **THEN** the bar for that week SHALL be styled with `fill: var(--warn)`

#### Scenario: Bar at or above per-week expected renders in accent

- **WHEN** a week has `overall_hours = 25.0` and `expected_hours = 20.0`
- **THEN** the bar for that week SHALL be styled with `fill: var(--accent)`

#### Scenario: Bar coloring respects contract change mid-year

- **WHEN** the loaded year contains weeks 1-10 with `expected_hours = 20.0` and weeks 11-52 with `expected_hours = 30.0`
- **AND** week 5 has `overall_hours = 22.0` and week 15 has `overall_hours = 22.0`
- **THEN** the bar for week 5 SHALL be `var(--accent)` (22 >= 20)
- **AND** the bar for week 15 SHALL be `var(--warn)` (22 < 30)

#### Scenario: Bar with zero per-week expected renders in accent

- **WHEN** a week has `overall_hours = 0.0` and `expected_hours = 0.0`
- **THEN** the bar for that week SHALL be styled with `fill: var(--accent)`

### Requirement: Reference line is a stepped polyline of per-week expected values

The employee weekly histogram SHALL render its reference line as a stepped line that follows each week's `expected_hours` value. The line SHALL be drawn as a single SVG `<polyline>` with `stroke-dasharray="4 3"` and `stroke: var(--ink-muted)`.

For each week, the polyline SHALL contain a horizontal segment at the y-coordinate corresponding to that week's `expected_hours`, spanning the full horizontal extent of that week's bar slot. Adjacent weeks with different `expected_hours` SHALL produce a vertical step segment at the week boundary.

When a week's `expected_hours` is `0`, the segment for that week SHALL sit at the chart's baseline (y == bar-area-height).

#### Scenario: Stepped line over a contract change

- **WHEN** weeks 1-2 have `expected_hours = 20.0` and weeks 3-4 have `expected_hours = 30.0`
- **THEN** the rendered SVG SHALL contain a `<polyline>` element
- **AND** the polyline SHALL include a vertical step between week 2 and week 3
- **AND** the segment over weeks 1-2 SHALL be at the y-coordinate for 20 hours
- **AND** the segment over weeks 3-4 SHALL be at the y-coordinate for 30 hours

#### Scenario: Stepped line drops to floor on zero-expected weeks

- **WHEN** week 1 has `expected_hours = 20.0`, week 2 has `expected_hours = 0.0`, and week 3 has `expected_hours = 20.0`
- **THEN** the polyline segment over week 2 SHALL sit at the chart baseline (y for value 0)
- **AND** the polyline SHALL include vertical steps at the week 1/2 boundary and the week 2/3 boundary

#### Scenario: Stepped line stays flat when expected does not change

- **WHEN** every loaded week has `expected_hours = 20.0`
- **THEN** the polyline SHALL render as one continuous flat line at the y-coordinate for 20 hours

#### Scenario: No reference line when all weeks have zero expected

- **WHEN** every loaded week has `expected_hours = 0.0`
- **THEN** the polyline MAY be omitted from the rendered SVG
- **OR** the polyline MAY render flat at the baseline

### Requirement: Histogram component does not accept a scalar expected value

The `EmployeeWeeklyHistogram` component SHALL NOT expose a scalar `expected_per_week` (or equivalent) prop. All expected-hours information SHALL be sourced from each `WorkingHours` entry's own `expected_hours` field.

#### Scenario: Component props omit a scalar expected value

- **WHEN** the `EmployeeWeeklyHistogramProps` struct is inspected
- **THEN** it SHALL NOT contain a field named `expected_per_week` or any other scalar representing a single expected-hours value

### Requirement: Header expected-hours pill reflects today's ISO week

The employee detail header SHALL display the "expected hours per week" pill using the `expected_hours` value from the `WorkingHours` entry whose ISO year and week match today's real-world ISO year and week.

If no such `WorkingHours` entry exists in the currently loaded data (for example, the user is viewing a past year), the header SHALL omit the expected-hours pill entirely.

#### Scenario: Pill shows expected hours for today's week in current year

- **WHEN** today is in ISO year 2026, week 17
- **AND** the loaded `working_hours_by_week` includes an entry for 2026 / week 17 with `expected_hours = 20.0`
- **THEN** the header SHALL display a pill with the value `20`

#### Scenario: Pill is hidden when today's week is not loaded

- **WHEN** today is in ISO year 2026, week 17
- **AND** the loaded `working_hours_by_week` covers only ISO year 2025
- **THEN** the header SHALL NOT render an expected-hours pill

#### Scenario: Pill reflects current contract after mid-year change

- **WHEN** today is in ISO year 2026, week 17
- **AND** the loaded data has `expected_hours = 20.0` for weeks 1-10 of 2026 and `expected_hours = 30.0` for weeks 11+ of 2026
- **THEN** the header pill SHALL display `30`

### Requirement: Histogram renders one bar per provided week
The `EmployeeWeeklyHistogram` component SHALL render an SVG element with a `<viewBox>` covering 340 width by 120 height and `preserveAspectRatio="none"`. For each entry in the `weeks` prop (a slice of `WorkingHours`), the component SHALL render one bar group at index `i`. The bar's `<rect>` SHALL be positioned at `x = i * bar_width` (with a 1 px gap), and its height SHALL be proportional to `(week.overall_hours / max_y) * 90`, where `max_y = max(expected_per_week, weeks.iter().map(|w| w.overall_hours).fold(0.0, f32::max), 1.0)`. The bar SHALL fill from the bottom of the chart (anchored at `y = 90 - bar_height`).

#### Scenario: One rect per week
- **WHEN** the component is rendered with 17 entries
- **THEN** the SVG SHALL contain exactly 17 `<rect>` elements representing bars

#### Scenario: max_y prevents divide-by-zero
- **WHEN** the component is rendered with `expected_per_week = 0.0` and all `overall_hours = 0.0`
- **THEN** the component SHALL render without panicking and all bars SHALL render with zero height (or no rect)

#### Scenario: Dashed expected-line at the right Y
- **WHEN** the component is rendered with `expected_per_week = 35.0` and `max_y` resolves such that the expected line sits at half-height
- **THEN** the SVG SHALL contain a `<line>` element with `stroke-dasharray="4 3"` whose `y1` and `y2` equal the computed line position, AND its `stroke` style SHALL resolve to `var(--ink-muted)`

### Requirement: Bars below the expected line render in warn color, others in accent color
For each week bar, when `week.overall_hours < expected_per_week`, the bar's fill SHALL resolve to `var(--warn)`. Otherwise, the fill SHALL resolve to `var(--accent)`. Color SHALL be applied via the SVG element's `style` attribute (e.g. `style: "fill: var(--warn)"`), not the `fill` attribute, so the CSS variable resolves through the cascade.

#### Scenario: Below-expected bar uses warn token
- **WHEN** a week has `overall_hours = 15.0` and `expected_per_week = 20.0`
- **THEN** that bar's `<rect>` SHALL include `fill: var(--warn)` in its inline style

#### Scenario: At-or-above expected bar uses accent token
- **WHEN** a week has `overall_hours = 25.0` and `expected_per_week = 20.0`
- **THEN** that bar's `<rect>` SHALL include `fill: var(--accent)` in its inline style

#### Scenario: No hardcoded hex remains in the histogram
- **WHEN** the non-test source of `src/component/employee_weekly_histogram.rs` is inspected
- **THEN** it SHALL NOT contain any 6-character hex color literals (matching `#[0-9A-Fa-f]{6}`) and SHALL NOT contain any 3-character hex color literals (matching `#[0-9A-Fa-f]{3}` outside of inline-style strings) for fill, stroke, or background

### Requirement: Bar selection is event-driven via `on_select`
Each bar group SHALL be wrapped in a `<g>` whose click target invokes the `on_select` event handler with the bar's `(year, week)` pair. The selected bar SHALL render at full opacity; non-selected bars SHALL render with `opacity: 0.85` when `selected_week` is `Some(_)`. When `selected_week` is `None`, all bars SHALL render at full opacity.

#### Scenario: Click emits the bar's (year, week)
- **WHEN** the user clicks a bar whose week is `(2026, 17)`
- **THEN** the `on_select` handler SHALL be called with `(2026, 17)`

#### Scenario: Selected bar is full opacity, others dimmed
- **WHEN** the component is rendered with `selected_week = Some((2026, 17))` for a week 17 entry plus a week 18 entry
- **THEN** the week-17 bar's group SHALL NOT include `opacity: 0.85` and the week-18 bar's group SHALL include `opacity: 0.85`

#### Scenario: No selection means all bars at full opacity
- **WHEN** the component is rendered with `selected_week = None`
- **THEN** no bar group SHALL include `opacity: 0.85`

### Requirement: X-axis labels render at sparse cadence with locale prefix
The component SHALL render an X-axis label for a week when `(week.week as usize - 1) % 4 == 0` OR `week.week == 52` OR `(week.year, week.week) == (current_year, current_week)`. Labels SHALL use the format `<WeekShort> <n>` where `<WeekShort>` is `i18n.t(Key::WeekShort)`. Label color SHALL resolve to `var(--ink-muted)` via the `style` attribute, font family SHALL be monospace (`ui-monospace, SFMono-Regular, Menlo, monospace`), and font size SHALL be 9.

#### Scenario: Label uses locale-aware prefix
- **WHEN** the component is rendered in German locale
- **THEN** each visible X-axis label SHALL begin with `KW ` followed by a digit

#### Scenario: Label cadence matches the rule
- **WHEN** the component is rendered with weeks 1, 2, 3, 4, 5 (and `current_week != 2..=4`)
- **THEN** an X-axis label SHALL render for week 1 and week 5, AND no X-axis label SHALL render for weeks 2, 3, or 4

#### Scenario: Current week always labeled
- **WHEN** the component is rendered with `current_year = 2026`, `current_week = 27`, and a week-27 entry whose `year = 2026`
- **THEN** an X-axis label for week 27 SHALL render even though `(27 - 1) % 4 != 0`

