## ADDED Requirements

### Requirement: Employees sidebar reflects mutations without page reload

After any mutation that can change a value displayed in the employees sidebar (the master list inside the `EmployeesShell` master/detail layout), the sidebar SHALL re-load its data and render the up-to-date values without requiring the user to reload the page or navigate away and back.

The sidebar SHALL preserve the user's local UI state across the refresh: the search input value, the selected year, and the scroll position MUST NOT be reset.

#### Scenario: Sidebar updates after editing an extra hours entry
- **WHEN** the user edits an extra hours entry in the detail view such that the saved employee aggregate (e.g. balance) changes
- **THEN** the sidebar entry for that employee shows the new aggregate without a page reload

#### Scenario: Sidebar updates after deleting an extra hours entry
- **WHEN** the user deletes an extra hours entry such that the saved employee aggregate changes
- **THEN** the sidebar entry for that employee shows the new aggregate without a page reload

#### Scenario: Sidebar updates after creating an extra hours entry
- **WHEN** the user creates an extra hours entry via the modal in create mode
- **THEN** the sidebar entry for that employee shows the new aggregate without a page reload

#### Scenario: Sidebar updates after deleting a custom extra hours entry
- **WHEN** the user deletes a custom extra hours entry on the detail view such that the aggregate changes
- **THEN** the sidebar entry for that employee shows the new aggregate without a page reload

#### Scenario: Sidebar updates after editing a work-details contract
- **WHEN** the user creates, edits, or deletes a work-details contract on the detail view
- **THEN** the sidebar entry for that employee shows the new target hours without a page reload

#### Scenario: Search term survives the refresh
- **WHEN** the user has typed a search term into the sidebar
- **AND** any of the above mutations trigger a sidebar refresh
- **THEN** the search input still shows the typed term and the filtered list reflects it

### Requirement: Refresh trigger is a single, narrow signal

The mechanism for triggering the sidebar refresh SHALL be a single application-wide signal. Service handlers that perform a mutation affecting employee aggregates SHALL bump that one signal; they MUST NOT directly invoke sidebar-component code or import sidebar-internal symbols.

This keeps the contract one-line for both new mutation handlers ("bump the signal") and any future sidebar variant ("read the signal").

#### Scenario: A new mutation handler can opt in with one line
- **WHEN** a developer adds a new mutation handler that changes an employee aggregate
- **THEN** opting the sidebar into the refresh requires only bumping the single refresh signal — no edits to the sidebar component, no per-call-site coupling

#### Scenario: The sidebar opts in by reading the signal in one place
- **WHEN** the sidebar's data-loading code is read
- **THEN** there is exactly one place where it reads the refresh signal, and changing the signal is the only mechanism that triggers a re-load (besides existing reactive dependencies like the year picker)

### Requirement: Refresh is centralized in the per-employee refresh function

To minimize the chance that a future mutation handler forgets to bump the refresh signal, the bump SHALL live inside the existing per-employee refresh path (`refresh_employee_data` in `src/service/employee.rs`). Any code path that already calls `refresh_employee_data` (or sends `EmployeeAction::Refresh`) to update the detail view will then refresh the sidebar as a side effect.

Mutation handlers that have a "follow with a refresh" pattern (which is the existing convention in `EmployeeAction`) SHALL therefore not need any sidebar-specific code.

#### Scenario: Refresh ride-along
- **WHEN** any code path calls `refresh_employee_data` (or dispatches `EmployeeAction::Refresh`)
- **THEN** the sidebar refresh signal is bumped as part of that call

#### Scenario: Year navigation also refreshes (acceptable side effect)
- **WHEN** the user navigates to a different year on the detail view (which calls the refresh path)
- **THEN** the sidebar refresh signal is bumped and the sidebar re-loads
- **AND** this over-eager refresh is intentional — the cost is one cheap GET and prevents the "developer forgot to bump" class of bugs
