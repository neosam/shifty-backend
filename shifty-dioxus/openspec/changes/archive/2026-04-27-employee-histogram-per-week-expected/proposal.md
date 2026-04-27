## Why

The employee detail page renders a weekly histogram of worked hours against a single "expected hours per week" value pulled from the most recent contract. Contracts change over time, so historical weeks are compared against a value that did not apply to them — bars get the wrong color and the dashed reference line is wrong wherever the contract differed from today's value. The header pill above the chart shows the same stale "most recent" value, which misrepresents the current state when viewing past years.

The backend already exposes the correct per-week expected hours on each `WorkingHours` entry, so the data is available — only the frontend is ignoring it.

## What Changes

- Use `WorkingHours.expected_hours` per week as the reference inside the employee weekly histogram, instead of a single scalar value derived from the most recent contract.
- Replace the flat dashed reference line with a stepped line that follows the per-week expected value (drops to 0 when no contract is active for a week).
- Per-bar color logic continues to use `value < expected → warn`, `value >= expected → accent`, but `expected` is now the per-week value rather than a single scalar.
- Header "expected hours per week" pill on the employee detail page sources its value from the `WorkingHours` entry for today's ISO week (the real-time current week), not from the most recent contract. When today's week is not present in the loaded data (e.g. user is browsing a past year), the pill is hidden.
- Remove the `most_recent_expected` helper and the `expected_per_week` prop on the histogram component — both become dead code.

## Capabilities

### New Capabilities
- `employee-weekly-histogram`: Per-week comparison of worked hours against contract-defined expected hours on the employee detail page, including the histogram chart, its reference line, bar coloring, and the header summary pill.

### Modified Capabilities
<!-- None — the existing histogram has no spec today. -->

## Impact

- `src/component/employee_weekly_histogram.rs` — drop `expected_per_week` prop, read per-week values from `WorkingHours`, render stepped reference line as a `<polyline>` instead of a single `<line>`.
- `src/component/employee_view.rs` — drop `most_recent_expected` helper, change header pill to look up today's `WorkingHours` entry, drop scalar prop on histogram call site.
- No backend changes — `WorkingHoursTO.expected_hours` already carries the per-week value.
- No i18n changes — labels stay the same.
