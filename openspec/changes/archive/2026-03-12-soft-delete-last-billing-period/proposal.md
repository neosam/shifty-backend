## Why

When a billing period is created by mistake (e.g., with the wrong end date), there is currently no way to remove it individually. The only option is `clear_all_billing_periods`, which soft-deletes every billing period. Users need the ability to undo the most recent billing period creation without affecting historical data.

## What Changes

- Add a new service method `delete_billing_period(id)` that soft-deletes a single billing period
- Enforce that only the latest (most recent by start date) billing period can be deleted — return a conflict error otherwise
- Cascade the soft delete to associated `billing_period_sales_person` records
- Add a new `DELETE /billing-periods/{id}` REST endpoint
- Add a new `ServiceError::NotLatestBillingPeriod` variant mapped to HTTP 409

## Capabilities

### New Capabilities
- `billing-period-soft-delete`: Ability to soft-delete the most recent billing period individually, with validation that only the latest period can be removed and cascade deletion of associated sales person entries.

### Modified Capabilities

## Impact

- **Service layer**: New `ServiceError` variant, new method on `BillingPeriodService` trait and implementation
- **DAO layer**: New `delete_by_id` method on `BillingPeriodDao`, new `delete_by_billing_period_id` method on `BillingPeriodSalesPersonDao`
- **REST layer**: New `DELETE /billing-periods/{id}` endpoint with utoipa annotation, error handler mapping for `NotLatestBillingPeriod`
- **API**: New endpoint visible in OpenAPI/Swagger documentation
