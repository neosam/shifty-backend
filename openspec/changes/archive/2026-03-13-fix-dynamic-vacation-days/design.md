## Context

The `hours_per_week` function in `service_impl/src/reporting.rs` builds `GroupedReportHours` structs for each calendar week. For dynamic employees, `weight_for_week` returns `0.0` as the first tuple element (the "expected hours for balance"), which causes the fallback at line 837 to set `expected_hours = shiftplan_hours + extra_work_hours`. This value is then assigned to `contract_weekly_hours`, which is used by `hours_per_day()` and subsequently by `vacation_days()`, `sick_leave_days()`, `holiday_days()`, and `absence_days()`.

The result: for any week where a dynamic employee has no shift plan hours (e.g., a pure vacation week), `contract_weekly_hours` becomes 0, `hours_per_day()` returns 0, and all day calculations return 0.

## Goals / Non-Goals

**Goals:**
- Dynamic employees show correct vacation/sick/holiday/absence days in reports
- Balance calculation for dynamic employees remains unchanged (still forced to 0)

**Non-Goals:**
- Changing how dynamic employees' balance is calculated
- Modifying the REST API or DTO structures
- Changing database schema

## Decisions

### Use `dynamic_working_hours_for_week` for `contract_weekly_hours`

The second return value of `weight_for_week` (`dynamic_working_hours_for_week`) already contains the real contract hours weighted for the week — it is calculated identically for both dynamic and non-dynamic employees (line 730: `employee_work_details.expected_hours * relation`).

**Current code (line 866):**
```rust
contract_weekly_hours: expected_hours,
```

**Fix:**
```rust
contract_weekly_hours: dynamic_working_hours_for_week,
```

This decouples the "contract hours for day calculations" from the "expected hours for balance calculations". The `expected_hours` variable continues to be used for the balance formula (line 870), so the balance-zeroing behavior is preserved.

**Alternative considered:** Adding a separate field for "real contract hours" to `GroupedReportHours`. Rejected because `contract_weekly_hours` already semantically represents the contract hours — it was just being assigned the wrong value for dynamic employees.

## Risks / Trade-offs

- **[Low] Non-dynamic employees also affected by the change**: For non-dynamic employees, `working_hours_for_week == dynamic_working_hours_for_week` (both use the same formula in `weight_for_week`), and the fallback branch (`working_hours_for_week == 0.0`) only triggers when there are no contract hours at all. So the change is a no-op for non-dynamic employees in the normal path.
- **[Low] Edge case: dynamic employee with no contract in a given week**: If `dynamic_working_hours_for_week` is also 0, `hours_per_day()` still returns 0 and vacation days will be 0 — but this is correct behavior (no contract = can't calculate days).
