## Context

The `generate_custom_report` method in `BillingPeriodReportServiceImpl` builds a JSON context that is passed to Tera or MiniJinja templates. Currently, each sales person entry only contains `sales_person_id` (UUID), a `values` array, and audit fields. Template authors must hardcode UUIDs to identify employees and loop through the values array to find specific value types. Additionally, several `BillingPeriodValueType` variants (VacationHours, SickLeave, Holiday, ExtraWork, VacationDays, VacationEntitlement) are defined but never populated from reporting data.

## Goals / Non-Goals

**Goals:**
- Enrich the template context with employee metadata (`name`, `is_paid`, `is_dynamic`) so templates can dynamically filter and display employees
- Populate all existing `BillingPeriodValueType` variants from reporting service data
- Provide a `values_map` dictionary alongside the existing `values` array for direct key-based access
- Maintain full backward compatibility with existing templates

**Non-Goals:**
- Changing the REST API for billing periods or text templates
- Modifying how `BillingPeriod` entities are persisted in the database
- Removing Tera support or changing template engine defaults
- Adding new REST endpoints

## Decisions

### 1. Add EmployeeWorkDetailsService as a dependency

**Decision**: Add `EmployeeWorkDetailsService` to the `gen_service_impl!` macro for `BillingPeriodReportServiceImpl`.

**Rationale**: The `is_dynamic` flag lives on `EmployeeWorkDetails`, not on `SalesPerson`. The service already has `SalesPersonService` for resolving names. Adding `EmployeeWorkDetailsService` follows the existing DI pattern.

**Alternative considered**: Query the DAO directly — rejected because it bypasses the service layer's permission checks and transaction management.

### 2. Determine is_dynamic with `any()` semantics

**Decision**: A sales person is considered `is_dynamic = true` if ANY of their `EmployeeWorkDetails` entries has `is_dynamic = true`.

**Rationale**: User preference — better to include too many employees in dynamic reports than to miss one. Avoids complex date-range matching logic. Simple and predictable.

**Alternative considered**: Use the `EmployeeWorkDetails` active at the billing period's end date — rejected as overly complex and could miss employees who were dynamic during part of the period.

### 3. Provide values_map alongside values array

**Decision**: Add a `values_map` field as a JSON object (dict) keyed by value type string, in addition to the existing `values` array.

**Rationale**: Non-breaking — existing templates use `values` array with loop+filter. New templates can use `values_map.overall.delta` for direct access. Works in both Tera and MiniJinja.

**Alternative considered**: Replace `values` array with dict — rejected as a breaking change for existing templates.

### 4. Populate value types from existing report fields

**Decision**: Map `EmployeeReport` fields to `BillingPeriodValueType` variants in `build_billing_period_report_for_sales_person`:

| Report Field | BillingPeriodValueType | Already populated? |
|---|---|---|
| `overall_hours` | `Overall` | Yes |
| `balance_hours` | `Balance` | Yes |
| `expected_hours` | `ExpectedHours` | Yes |
| `extra_work_hours` | `ExtraWork` | No → Add |
| `vacation_hours` | `VacationHours` | No → Add |
| `sick_leave_hours` | `SickLeave` | No → Add |
| `holiday_hours` | `Holiday` | No → Add |
| `vacation_days` | `VacationDays` | No → Add |
| `vacation_entitlement` | `VacationEntitlement` | No → Add |
| `custom_extra_hours` | `CustomExtraHours(name)` | Yes |

**Rationale**: The enum variants and the report data already exist. This is purely wiring them together.

### 5. Resolve employee metadata via batch loading

**Decision**: Load all sales persons and all employee work details once before iterating, then look up per sales person in-memory.

**Rationale**: Avoids N+1 queries. The `get_all` and `find_by_sales_person_id` methods already exist. Since billing period reports are HR-only operations, loading all data is acceptable.

## Risks / Trade-offs

- **[Performance]** Loading all `EmployeeWorkDetails` adds one extra query per report generation → Acceptable since this is a low-frequency HR operation, not a hot path.
- **[is_dynamic accuracy]** Using `any()` may mark a person as dynamic even if they are no longer dynamic → Acceptable per user preference (better too many than too few). Template authors can still filter by UUID if needed.
- **[Context size]** Adding 6 more value types per sales person increases JSON context size → Negligible for typical employee counts.
