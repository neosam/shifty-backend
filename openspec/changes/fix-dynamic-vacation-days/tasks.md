## 1. Red — Write failing tests

- [x] 1.1 Add a test for a dynamic employee that takes a full week of vacation, asserting vacation_days > 0 (currently returns 0 — this test should fail)
- [x] 1.2 Add a test for a dynamic employee that takes partial vacation in a worked week, asserting correct vacation day count
- [x] 1.3 Add a test verifying the balance for a dynamic employee is still forced to 0
- [x] 1.4 Add a test for a non-dynamic employee confirming vacation days are unchanged

## 2. Green — Fix the bug

- [x] 2.1 In `service_impl/src/reporting.rs` `hours_per_week` function, change `contract_weekly_hours: expected_hours` to `contract_weekly_hours: dynamic_working_hours_for_week` (line ~866)
- [x] 2.2 Run all tests (`cargo test`) and verify all new tests pass and no existing tests regress

## 3. Verify

- [x] 3.1 Run `cargo build` to confirm compilation
- [x] 3.2 Run `cargo run` to confirm the server starts successfully
