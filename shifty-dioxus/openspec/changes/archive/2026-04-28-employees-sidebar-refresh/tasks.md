## 1. Refresh-Token Signal

- [x] 1.1 In `src/service/employee.rs`, add `pub static EMPLOYEES_LIST_REFRESH: GlobalSignal<u64> = Signal::global(|| 0);` with a doc comment documenting the contract
- [x] 1.2 In `refresh_employee_data`, after successful `load_employee_data(...)`, bump the signal via the `bump_employees_list_refresh()` helper
- [x] 1.3 Verify with `cargo check` — passes

## 2. Sidebar Reads The Token

- [x] 2.1 In `src/component/employees_list.rs`, import `EMPLOYEES_LIST_REFRESH`
- [x] 2.2 Inside the `use_resource` closure, read the signal as a reactive dependency
- [x] 2.3 Pattern matches the existing `year` reactive read pattern

## 3. Service Mutation Audit

- [x] 3.1 `EmployeeAction` arms: `UpdateExtraHours` calls `refresh_employee_data` directly. `DeleteExtraHours` and `DeleteCustomExtraHour` rely on the page's `onupdate` / `on_custom_delete` callbacks, which dispatch `EmployeeAction::Refresh` → `refresh_employee_data`. All ride along through the bump.
- [x] 3.2 `EmployeeWorkDetailsAction` arms (`Save`, `Update`, `Delete`) all call `reload_employee_work_details`, which itself calls `super::employee::refresh_employee_data()` (line 61). All ride along through the bump.
- [x] 3.3 Extra hours create path: `extra_hours_modal.rs` calls `api::add_extra_hour` directly, then `on_saved` → page → `ExtraHoursSaved` → `EmployeeAction::Refresh` → `refresh_employee_data` → bumps. Confirmed.
- [x] 3.4 No audited path bypasses `refresh_employee_data` — no additional patches needed.

## 4. Tests

- [x] 4.1 Added `bump_employees_list_refresh_increments_observable_signal` test in `src/service/employee.rs` test module. Drives the read+bump from inside a `VirtualDom` (required because `GlobalSignal` needs an active Dioxus runtime).
- [x] 4.2 Manual browser exercise (Chromium via Playwright, 2026-04-28):
  - Edit extra hours entry 15.5h → 20h: sidebar `Tom Bauer -618.5 → -614.0`, main `Balance -618.50 → -614.00` (synchron, ohne Page-Reload)
  - Type search "Tom" in sidebar → list filters to one row
  - Edit extra hours entry 20h → 25h: sidebar refreshes to `-609.0`, main to `-609.00`, **search term "Tom" survives** the refresh, filter still applied
  - Network: single PUT, single GET on the employees aggregate endpoint per submit — no thrash

## 5. Verification

- [x] 5.1 Run `cargo fmt`
- [x] 5.2 Run `cargo clippy` — no new warnings introduced
- [x] 5.3 Run `cargo check`
- [x] 5.4 Run `cargo test` — 461 passed
- [x] 5.5 Run `openspec validate employees-sidebar-refresh --strict` — valid
