## Why

Billing periods on the employees page are displayed with the oldest first and the newest at the bottom. Users expect the most recent billing period at the top, as that's the one they interact with most frequently.

## What Changes

- Add `all_ordered_desc` default method to the `BillingPeriodDao` trait that returns billing periods sorted by `start_date` descending (newest first)
- Update `get_billing_period_overview` in the billing period service to use `all_ordered_desc` instead of `all`
- Remove the redundant `deleted_at` filter in the service method (already handled by `all` in the DAO trait)

## Capabilities

### New Capabilities
- `billing-period-ordering`: Default descending sort for billing period listings via `all_ordered_desc` DAO trait method

### Modified Capabilities
<!-- No existing specs to modify -->

## Impact

- **DAO trait** (`dao/src/billing_period.rs`): New default method `all_ordered_desc`
- **Service** (`service_impl/src/billing_period.rs`): `get_billing_period_overview` uses new method
- **REST API**: `/billing-periods` endpoint returns periods in descending order — this is a behavior change for API consumers but aligns with expected UX
- **Frontend**: No changes needed — already iterates in order received from API
