## 1. DAO Trait Change

- [x] 1.1 Add `all_ordered_desc` default method to `BillingPeriodDao` trait in `dao/src/billing_period.rs` that calls `all()` and sorts by `start_date` descending

## 2. Service Layer Update

- [x] 2.1 Update `get_billing_period_overview` in `service_impl/src/billing_period.rs` to call `all_ordered_desc` instead of `all`
- [x] 2.2 Remove redundant `deleted_at` filter in `get_billing_period_overview` (already handled by `all`)

## 3. Tests

- [x] 3.1 Add test verifying `all_ordered_desc` returns billing periods sorted by `start_date` descending
- [x] 3.2 Run `cargo build` and `cargo test` to verify everything compiles and passes
