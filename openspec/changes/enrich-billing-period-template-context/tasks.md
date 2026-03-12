## 1. Scaffolding

- [x] 1.1 Add `EmployeeWorkDetailsService` dependency to `BillingPeriodReportServiceImpl` via `gen_service_impl!` macro in `service_impl/src/billing_period_report.rs`
- [x] 1.2 Wire `EmployeeWorkDetailsService` into the dependency injection in `shifty_bin` (main executable)
- [x] 1.3 Populate missing value types (`ExtraWork`, `VacationHours`, `SickLeave`, `Holiday`, `VacationDays`, `VacationEntitlement`) in `build_billing_period_report_for_sales_person` using data from `ShortEmployeeReport`
- [x] 1.4 Enrich the JSON context in `generate_custom_report` with employee metadata: load all sales persons and employee work details, add `name`, `is_paid`, `is_dynamic` fields per sales person entry
- [x] 1.5 Add `values_map` dictionary to the JSON context alongside existing `values` array, using value type string as key and `{delta, ytd_from, ytd_to, full_year}` as value

## 2. Tests (Red)

- [x] 2.1 Add test: `is_dynamic` is `true` when any `EmployeeWorkDetails` entry has `is_dynamic = true`
- [x] 2.2 Add test: `is_dynamic` is `false` when all `EmployeeWorkDetails` entries have `is_dynamic = false`
- [x] 2.3 Add test: `is_dynamic` is `false` when no `EmployeeWorkDetails` entries exist for the sales person
- [x] 2.4 Add test: `is_dynamic` with mixed entries (multiple entries, some `true`, some `false`) to verify `any()` semantics
- [x] 2.5 Add test: `name` and `is_paid` fields are present in the rendered template context
- [x] 2.6 Add test: `values_map` provides direct access to value types (e.g., `values_map.overall.delta`)
- [x] 2.7 Add test: newly populated value types (`vacation_hours`, `sick_leave`, `holiday`, `extra_work`, `vacation_days`, `vacation_entitlement`) are accessible in template output
- [x] 2.8 Add test: `values_map` and `values` array contain consistent data (same value accessed via both paths yields identical result)
- [x] 2.9 Add test: `CustomExtraHours` key format in `values_map` (e.g., `values_map["custom_extra_hours:overtime"]`)
- [x] 2.10 Add test: enriched context (name, is_paid, is_dynamic, values_map) produces identical output for both Tera and MiniJinja engines
- [x] 2.11 Add regression test: existing Tera template using `values` array produces identical output
- [x] 2.12 Add regression test: existing MiniJinja template using `values` array produces identical output

## 3. Implementation (Green)

- [x] 3.1 Implement the enrichment logic: batch-load sales persons and employee work details, resolve `name`, `is_paid`, `is_dynamic` per sales person
- [x] 3.2 Implement `values_map` construction from the `BTreeMap<BillingPeriodValueType, BillingPeriodValue>`
- [x] 3.3 Ensure all tests pass with `cargo test`
- [x] 3.4 Verify `cargo build` succeeds and `cargo run` starts without errors
