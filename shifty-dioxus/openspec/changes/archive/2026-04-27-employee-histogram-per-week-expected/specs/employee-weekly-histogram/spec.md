## ADDED Requirements

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
