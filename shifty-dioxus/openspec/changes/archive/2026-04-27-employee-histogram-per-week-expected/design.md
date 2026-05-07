## Context

The employee detail page (`src/page/employee_details.rs` → `EmployeeView` → `EmployeeWeeklyHistogram`) renders one bar per ISO week of the selected year and a horizontal dashed reference line representing the contracted expected hours per week.

Today the reference value is a single scalar:

```rust
// src/component/employee_view.rs
fn most_recent_expected(work_details: &[EmployeeWorkDetails]) -> f32 {
    work_details.iter().max_by_key(|d| d.from).map(|d| d.expected_hours).unwrap_or(0.0)
}
let expected_per_week = most_recent_expected(&work_details_list);
```

That scalar is passed as a prop into the histogram and used both for the dashed reference line and for the bar coloring. The same scalar is shown in a "X h/Woche" pill in the header.

Contracts (`EmployeeWorkDetails`) change over time. Each `WorkingHours` row already carries the correct per-week expected value:

```rust
// src/state/employee.rs (existing)
pub struct WorkingHours {
    pub from: time::Date,
    pub expected_hours: f32,    // already the right value for that week
    pub overall_hours: f32,
    ...
}
```

The table view directly below the histogram already reads `week.expected_hours` correctly. Only the histogram and the header pill ignore it.

## Goals / Non-Goals

**Goals:**
- Histogram bar color and reference line per week reflect the contract that applied to that ISO week.
- Header pill reflects the contract that applies to today's real-world ISO week.
- No change to backend, REST types, or i18n.
- Keep the visual language consistent: dashed line in `var(--ink-muted)`, bar tokens `var(--warn)` / `var(--accent)`.

**Non-Goals:**
- No change to the table-row view below the histogram (already correct).
- No change to other charts (e.g. `weekly_overview_chart.rs`).
- No new contract-history visualization beyond the stepped reference line.
- No backend changes — we trust `WorkingHoursTO.expected_hours` as source of truth.

## Decisions

### Decision 1: Per-week expected value comes from `WorkingHours`, not from contracts

The frontend already receives `WorkingHours.expected_hours` per week from the backend. We use it directly instead of reconstructing it from `EmployeeWorkDetails` entries.

**Why:** Single source of truth, no duplication of "which contract applies to which week" logic, no risk of frontend/backend disagreement. The table view already does this.

**Alternative considered:** Compute per-week expected hours frontend-side from `work_details_list` by walking contract `from` dates. Rejected: duplicates backend logic, more code, more bugs.

### Decision 2: Drop `expected_per_week` from `EmployeeWeeklyHistogramProps`

The histogram needs no scalar — it can derive everything from `WorkingHours`. The `expected_per_week` prop is removed.

**Why:** Removing the prop forces every call site to pass per-week data, preventing accidental regressions where a scalar gets re-introduced.

**Alternative considered:** Keep the prop optional and only use it as a fallback. Rejected: dead-code path, easy to misuse.

### Decision 3: Stepped reference line as `<polyline>`

Replace the single `<line>` with a `<polyline>` whose points form a step function over the weeks: a horizontal segment at the week's `expected_hours` for each week, with vertical segments at week boundaries where the value changes.

Concretely, each week of width `bar_width + BAR_GAP` produces two points: `(x_left, y_for_expected)` and `(x_right, y_for_expected)`. Points across week boundaries naturally produce vertical segments when expected changes, because successive points share the same x at boundaries only when the value is identical.

When `expected_hours == 0` for a week, the segment for that week sits at `y == BAR_AREA_HEIGHT` (the chart floor). The line is drawn for the full year regardless of zero spans.

**Why:** Stepped line is faithful to reality (contracts are piecewise-constant over time). `<polyline>` keeps the SVG simple, no extra elements, easy to test in SSR.

**Alternative considered:** Many small `<line>` elements per week. Rejected: more DOM, harder to test.

**Alternative considered:** Skip zero-expected weeks (no segment). Rejected per discussion — dropping to zero is more honest and visually obvious.

### Decision 4: Header pill uses today's ISO week, hidden when not in view

The header pill (`X h/Woche`) is sourced from the `WorkingHours` entry whose ISO week equals today's `current_year` / `current_week` (from `js::get_current_week()`).

If no such entry exists in the loaded year (e.g. user is viewing 2025 but today is 2026 KW 17), the pill is hidden entirely instead of showing a misleading 0 or fallback value.

**Why:** "X h/Woche" reads as "what is this employee's current contract", which only makes sense for the present moment. Hiding it in past-year views is honest.

**Alternative considered:** Fall back to the last week of the loaded data. Rejected: subtle and confusing — looks like a real value but is actually arbitrary.

**Alternative considered:** Show the value of the currently selected week in the histogram. Rejected for now — would couple the header to histogram selection state and change the meaning of the pill.

### Decision 5: Bar color rule unchanged

`value < expected → var(--warn)`, otherwise `var(--accent)`. When `expected == 0` for a week, every value `>= 0` so the bar is `var(--accent)`. This matches the existing rule applied per week.

**Why:** Keeps the comparison rule simple and consistent. A week with no contract has no expectation to fall short of, so coloring it as "ok" is acceptable.

## Risks / Trade-offs

- **Risk:** Backend `WorkingHoursTO.expected_hours` returns a stale value for some week. → Mitigation: out of scope here; if observed, file a backend bug. The frontend's job is to render what the backend says.
- **Risk:** Stepped line with many transitions could look noisy. → Mitigation: in practice contracts change rarely (a few times per year at most); the line will mostly be flat with a few steps. Render order keeps the line above bars (drawn last) so transitions stay visible.
- **Risk:** Header pill disappearing in past-year views may surprise users who expect "always something there." → Mitigation: the pill currently shows a misleading number; absence is strictly better. If a label is desired, a follow-up can add a placeholder.
- **Trade-off:** Removing `expected_per_week` is a (minor) breaking change to the histogram component's prop API. No external consumers — only one call site in this repo — so the cost is low.
