## 1. Service Error

- [x] 1.1 Add `NotLatestBillingPeriod(Uuid)` variant to `ServiceError` enum in `service/src/lib.rs`
- [x] 1.2 Add error handler mapping for `NotLatestBillingPeriod` → HTTP 409 in `rest/src/lib.rs`

## 2. DAO Layer

- [x] 2.1 Add `delete_by_id(id, process, tx)` method to `BillingPeriodDao` trait in `dao/src/billing_period.rs`
- [x] 2.2 Implement `delete_by_id` in `dao_impl_sqlite/src/billing_period.rs` (UPDATE SET deleted, deleted_by WHERE id = ?)
- [x] 2.3 Add `delete_by_billing_period_id(billing_period_id, process, tx)` method to `BillingPeriodSalesPersonDao` trait in `dao/src/billing_period_sales_person.rs`
- [x] 2.4 Implement `delete_by_billing_period_id` in `dao_impl_sqlite/src/billing_period_sales_person.rs`
- [x] 2.5 Add SQLx query metadata files for the new queries (run `cargo sqlx prepare` or build with live DB)

## 3. Service Layer

- [x] 3.1 Add `delete_billing_period(id, context, tx)` method to `BillingPeriodService` trait in `service/src/billing_period.rs`
- [x] 3.2 Implement `delete_billing_period` in `service_impl/src/billing_period.rs` with logic: check permission → find_by_id (404) → all_ordered_desc → check latest (409) → cascade delete sales person entries → delete billing period → commit

## 4. REST Layer

- [x] 4.1 Add `DELETE /billing-periods/{id}` handler function in `rest/src/billing_period.rs` with `#[utoipa::path]` annotation
- [x] 4.2 Register the new route in the billing period router
- [x] 4.3 Add the endpoint to the ApiDoc struct for Swagger UI inclusion

## 5. Testing

- [x] 5.1 Add unit test: successful deletion of the latest billing period
- [x] 5.2 Add unit test: deletion rejected when ID is not the latest billing period (409)
- [x] 5.3 Add unit test: deletion rejected when ID does not exist (404)
- [x] 5.4 Add unit test: deletion rejected without HR privilege (403)
- [x] 5.5 Add unit test: cascade soft-delete of billing_period_sales_person entries
- [x] 5.6 Run `cargo build`, `cargo test`, and `cargo run` to verify everything compiles and passes
