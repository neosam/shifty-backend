## Context

The `BillingPeriodDao` trait follows a pattern where `dump_all` returns raw data from the database, and default trait methods (`all`, `find_by_id`, `find_latest_end_date`) filter and query on top of it. Currently, `all` filters out soft-deleted entries but does not sort. The service method `get_billing_period_overview` calls `all` directly, resulting in undefined order (insertion order in practice), which shows oldest billing periods first in the UI.

## Goals / Non-Goals

**Goals:**
- Add a `all_ordered_desc` default method to `BillingPeriodDao` that sorts by `start_date` descending
- Use `all_ordered_desc` in `get_billing_period_overview` so the REST API returns newest-first
- Remove redundant `deleted_at` filtering in the service layer (already done by `all`)

**Non-Goals:**
- Adding configurable sort direction or pagination to the API
- Changing the existing `all` method behavior
- Adding `ORDER BY` at the SQL level (sorting happens in the default trait implementation)

## Decisions

### 1. Default trait method over SQL-level ordering

Add `all_ordered_desc` as a default method on `BillingPeriodDao` that calls `all()` and sorts the result in Rust. This follows the existing convention where `dump_all` is the only method with a concrete SQL implementation, and all other methods are default implementations that filter/transform on top of it.

Alternative: `ORDER BY from_date_time DESC` in the SQL query. Rejected because it breaks the established DAO pattern.

### 2. Sort by `start_date`

Sort descending by `start_date` since billing periods are sequential and non-overlapping. `start_date` clearly identifies the chronological position of a period.

Alternative: Sort by `end_date`. Would yield the same result for non-overlapping periods but `start_date` is more intuitive.

### 3. Keep `all` unchanged

`all` remains as-is (filtered, unsorted) since `find_by_id` and `find_latest_end_date` depend on it and don't need ordering.

## Risks / Trade-offs

- **API behavior change** → The `/billing-periods` endpoint will return results in a different order. Since the frontend already consumes them in iteration order, this is the desired effect. No other known API consumers.
- **Performance** → Sorting in Rust rather than SQL is negligible for the expected number of billing periods (tens, not thousands).
