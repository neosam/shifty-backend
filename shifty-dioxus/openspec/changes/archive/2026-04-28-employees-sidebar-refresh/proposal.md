## Why

The employees sidebar (`EmployeesList` in `src/component/employees_list.rs`) loads its data once via `use_resource(move || loader::load_employees(...))` at component mount and caches it. Mutations on a single employee — adding, editing, or deleting an extra hours entry; saving a work-details contract; deleting a custom-extra-hours entry — refresh the per-employee detail view (`EMPLOYEE_STORE`) but do **not** invalidate the sidebar's cached list. The sidebar then displays stale balance/target numbers (e.g. `Tom Bauer -624.0/0`) while the main view shows the freshly recomputed value (e.g. `-618.50 hours`). This was confirmed during manual testing of the `frontend-extra-hours-edit` change.

The bug pre-dates the edit feature — it also affects delete and create — but it became visible in the manual test of the edit flow and warrants a dedicated fix that covers all mutation paths.

## What Changes

- Make the employees sidebar list re-fetch (or its store re-populate) after any mutation that can change an employee's aggregates: extra hours create / edit / delete, custom extra hours delete (and any future mutation that affects `EMPLOYEE_STORE` or the per-employee report).
- Avoid coupling each call site to the sidebar implementation: introduce a single, narrow signal/store that the sidebar listens to, and that mutation handlers bump.
- Cover both pages that mount the employee detail flow: the HR-facing `employee_details.rs` and the user's own `my_employee_details.rs`.

## Capabilities

### New Capabilities
- `employees-sidebar-refresh`: Defines when the employees sidebar list SHALL be refreshed in response to mutations elsewhere in the application, and the contract for triggering that refresh from service handlers without coupling to the sidebar component itself.

### Modified Capabilities
<!-- None — there is no pre-existing spec for the employees sidebar refresh behavior. -->

## Impact

- **Frontend code**:
  - `src/component/employees_list.rs`: switch from a one-shot `use_resource` to a resource that depends on a refresh-token signal (or move the data into a dedicated global store).
  - `src/service/employee.rs`: bump the refresh token (or repopulate the store) at the end of every mutation arm whose result can change a sidebar-visible aggregate (`UpdateExtraHours`, `DeleteExtraHours`, `DeleteCustomExtraHour`, plus the `Refresh` arm).
  - Possibly `src/service/employee_work_details.rs` and any other mutation entry points that touch employee aggregates — to be enumerated during design.
  - Tests: a unit test confirming the refresh-token signal is bumped by the relevant mutation arms. The end-to-end (sidebar updates after edit) verification is covered by manual exercise.
- **Out of scope**:
  - Optimistic updates (we accept a full re-load round trip).
  - Search / filter state preservation across the refresh — already handled by `EmployeesList`'s local `search` signal.
  - Sidebar performance optimizations (debouncing, partial updates).
  - Backend changes — the existing `GET /employees` aggregate endpoint already returns up-to-date numbers.
- **No backend impact.**
- **No i18n impact** — no user-facing strings change.
