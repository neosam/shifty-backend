## 1. Scaffolding — Types and Stubs

- [x] 1.1 Add `UnpaidLeave` variant to `ExtraHoursCategoryEntity` enum in `dao/src/extra_hours.rs`
- [x] 1.2 Add `UnpaidLeave` variant to `ExtraHoursCategory` enum in `service/src/extra_hours.rs` with `as_report_type() → AbsenceHours` and `as_availability() → Unavailable`
- [x] 1.3 Add `UnpaidLeave` variant to `ExtraHoursCategoryTO` enum in `rest-types/src/lib.rs`
- [x] 1.4 Update DAO SQLite serialization/deserialization in `dao_impl_sqlite/src/extra_hours.rs` to handle `"UnpaidLeave"` string
- [x] 1.5 Update conversion functions between DAO entity, service model, and REST transport object to handle `UnpaidLeave`
- [x] 1.6 Add `unpaid_leave_hours` field to `GroupedReportHours` in `service/src/reporting.rs`
- [x] 1.7 Add `unpaid_leave_hours` field to `ShortEmployeeReport` in `service/src/reporting.rs`
- [x] 1.8 Add `unpaid_leave_hours` field to `EmployeeReport` in `service/src/reporting.rs`
- [x] 1.9 Add `unpaid_leave_hours` field to `WeeklyHours` struct in `service_impl/src/reporting.rs` and stub the aggregation
- [x] 1.10 Update `absence_days()` in `GroupedReportHours` to include `unpaid_leave_hours`
- [x] 1.11 Verify project compiles with `cargo build`

## 2. Red — Write Failing Tests

- [x] 2.1 Add test that `UnpaidLeave` maps to `ReportType::AbsenceHours`
- [x] 2.2 Add test that `UnpaidLeave` maps to `Availability::Unavailable`
- [x] 2.3 Add test that unpaid leave hours are tracked separately in reporting (not mixed into vacation/sick/holiday)
- [x] 2.4 Add test that unpaid leave does not affect vacation days calculation
- [x] 2.5 Add test that unpaid leave is included in `absence_days()` calculation
- [x] 2.6 Add test that unpaid leave reduces expected hours (balance stays neutral)

## 3. Green — Implementation

- [x] 3.1 Wire up `unpaid_leave_hours` aggregation in the weekly reporting loop in `service_impl/src/reporting.rs`
- [x] 3.2 Wire up `unpaid_leave_hours` in the fold/accumulation step
- [x] 3.3 Wire up `unpaid_leave_hours` into `ShortEmployeeReport` and `EmployeeReport` construction
- [x] 3.4 Run `cargo test` and verify all tests pass
- [x] 3.5 Run `cargo run` and verify the server starts successfully
