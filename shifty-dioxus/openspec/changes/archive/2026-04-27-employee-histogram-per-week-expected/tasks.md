## 1. Histogram component: per-week expected hours

- [x] 1.1 Remove `expected_per_week: f32` field from `EmployeeWeeklyHistogramProps` and `EmployeeWeeklyHistogramViewProps` in `src/component/employee_weekly_histogram.rs`
- [x] 1.2 Update `compute_max_y` to take `weeks: &[WorkingHours]` only, computing the max of `overall_hours` and `expected_hours` across all weeks (drop the scalar `expected` parameter)
- [x] 1.3 Update `bar_color_token` to take `value: f32, expected: f32` where `expected` is the per-week value (signature unchanged, semantics now per-week — adjust call site to pass `week.expected_hours`)
- [x] 1.4 In the `EmployeeWeeklyHistogramView` body, drive bar color from `week.expected_hours` for each week instead of the removed prop
- [x] 1.5 Replace the single dashed `<line>` with a `<polyline>` whose points form a stepped path: for each week emit `(x_left, y_for_week_expected)` and `(x_right, y_for_week_expected)` where `x_left` / `x_right` are the bar slot's horizontal extent including the leading gap. Use `stroke-dasharray="4 3"`, `stroke-width="1.5"`, `stroke: var(--ink-muted)`, `fill: none`
- [x] 1.6 Render the polyline with the chart's baseline y when a week's `expected_hours` is `0`
- [x] 1.7 Skip the polyline element entirely when no week has a positive `expected_hours` (allowed by spec)

## 2. Histogram component: tests

- [x] 2.1 Update existing tests to drop the `expected_per_week` argument and use per-week `expected_hours` set on each `WorkingHours` test fixture
- [x] 2.2 Add SSR test: stepped polyline renders one segment at the y for 20h over weeks 1-2 and one at 30h over weeks 3-4 (vertical step at the boundary)
- [x] 2.3 Add SSR test: a zero-expected week produces a polyline segment at the chart baseline
- [x] 2.4 Add SSR test: bar coloring reflects each week's own `expected_hours` — week with `overall=22, expected=20` is `var(--accent)`, week with `overall=22, expected=30` is `var(--warn)` rendered side-by-side
- [x] 2.5 Add SSR test: when every week has `expected_hours = 0.0`, no `<polyline>` is emitted (or it sits flat at baseline — assert one of the two)
- [x] 2.6 Update the `compute_max_y_*` unit tests to the new signature
- [x] 2.7 Keep the existing `no_hex_color_literals_in_source` test green

## 3. Employee view: header pill and call site

- [x] 3.1 Remove `most_recent_expected` helper from `src/component/employee_view.rs`
- [x] 3.2 Replace `let expected_per_week = most_recent_expected(&work_details_list);` with a lookup of today's `WorkingHours` entry: find the week whose ISO `(year, week)` equals `(js::get_current_year(), js::get_current_week())` in `employee.working_hours_by_week`
- [x] 3.3 In the header section (around `employee_view.rs:160`), render the pill only when the lookup found a `WorkingHours` entry; use that entry's `expected_hours` as the displayed value
- [x] 3.4 Update the `EmployeeWeeklyHistogram { ... }` call site (around line 367) to drop the `expected_per_week` argument
- [x] 3.5 Verify the table view below the histogram (which already reads `week.expected_hours`) still compiles and renders correctly — no change expected
- [x] 3.6 Run `cargo check` and fix any unused-import / dead-code warnings introduced by the removed helper

## 4. Employee view: tests

- [x] 4.1 Remove or update tests asserting `most_recent_expected_*` behavior
- [x] 4.2 Add unit test: header pill is rendered with today's week's expected value when that week is present in `working_hours_by_week`
- [x] 4.3 Add unit test: header pill is omitted when today's ISO week is not present in `working_hours_by_week` (e.g. all entries are from a past year)
- [x] 4.4 Add unit test: header pill shows the post-change value when contract changed mid-year and today is after the change point

## 5. Validation

- [x] 5.1 Run `cargo fmt`
- [x] 5.2 Run `cargo clippy` and address any warnings introduced by the change
- [x] 5.3 Run `cargo test` — all new and existing tests green
- [ ] 5.4 Run `dx serve --hot-reload` (with Tailwind in watch mode) and visually verify on the employee detail page: stepped reference line, correct per-week bar colors across a contract change, correct header pill for the current week
- [x] 5.5 Run `openspec validate employee-histogram-per-week-expected`
