## Context

The employees sidebar (`src/component/employees_list.rs`) is mounted via the `EmployeesShell` master/detail layout used by `employee_details.rs` (HR view). It loads its data with:

```rust
let employees =
    use_resource(move || loader::load_employees(config.to_owned(), *year.read(), week_until));
```

`use_resource` re-runs only when a reactive signal it reads changes. Today the only reactive dependency is `year` (changed via the year picker). When the user edits, deletes, or creates an extra hours entry on an employee detail page, only `EMPLOYEE_STORE` is repopulated — the sidebar's resource is not touched and continues to render the cached list.

The mutation paths that can change a sidebar-visible value (balance, target hours, name) are concentrated in `src/service/employee.rs`'s `EmployeeAction` arms and adjacent services:

- `EmployeeAction::UpdateExtraHours` (new — `frontend-extra-hours-edit`)
- `EmployeeAction::DeleteExtraHours`
- `EmployeeAction::DeleteCustomExtraHour`
- `EmployeeAction::Refresh` (catch-all — used by Save / Saved hooks across the page)
- `EmployeeWorkDetailsAction::Save` / `Update` / `Delete` (`src/service/employee_work_details.rs`)
- The create path for extra hours (currently a direct `api::add_extra_hour` call from `extra_hours_modal.rs`, not via a service action — relies on the page's `onsaved` callback dispatching `EmployeeAction::Refresh`)

## Goals / Non-Goals

**Goals:**
- After any mutation that affects an employee's sidebar-visible aggregates, the sidebar SHALL display the new values without a full page reload.
- The trigger mechanism SHALL be one narrow signal/store, not a per-call-site refresh ad-hoc patch — so future mutations only need to bump one thing.
- The sidebar SHALL preserve the user's local UI state (search term, scroll position) across the refresh.
- The fix SHALL apply uniformly to all current and future mutation paths via a single hook in the service layer (or one well-known signal).

**Non-Goals:**
- Optimistic UI updates. A full re-load round-trip is acceptable.
- Granular partial updates (only the changed employee). Re-loading the whole list is fine — the list is small (handful of employees).
- Backend changes.
- Refresh of any other page or component (this change is about the sidebar list only).

## Decisions

### Decision 1: Refresh-token signal, not a global employees store

Two designs were considered:

- **A: A global `EMPLOYEES_LIST_STORE`** that the sidebar reads and that the service repopulates after each mutation. Pro: explicit. Con: doubles the loading code (one in the sidebar's `use_resource`, one in the service refresh). Both must stay in sync. Also means the sidebar can no longer use `use_resource` ergonomics for `Loading…` / error states.

- **B: A global `EMPLOYEES_LIST_REFRESH: GlobalSignal<u64>` token** that the sidebar's `use_resource` reads (so the resource re-runs when the token changes), and that service mutation arms bump (`EMPLOYEES_LIST_REFRESH.write() += 1`). Pro: tiny surface, the sidebar keeps its existing loading/error rendering, and adding a new mutation requires a single line in the service. Con: less explicit, but cheap.

We pick **B** (refresh-token signal). It is a one-line change in `EmployeesList` (read the token inside the resource closure) and a one-line change at each mutation site.

### Decision 2: Where to bump the token

Two locations were considered:

- **Inside `refresh_employee_data()`** — the function that all `EMPLOYEE_STORE` repopulation goes through. Bumping the token there means: any caller that does `refresh_employee_data` automatically refreshes the sidebar too. Pro: single bump site. Con: ties the sidebar refresh to *every* per-employee data reload, including reads triggered by year navigation that does not change any aggregate (still fine — refresh is cheap, but it is over-eager).

- **In each mutation arm of `EmployeeAction`** that follows a mutation with a refresh — i.e. `UpdateExtraHours`, `DeleteExtraHours`, `DeleteCustomExtraHour`, `EmployeeWorkDetailsAction::Save`/`Update`/`Delete`. Pro: precise. Con: easy to forget when adding a new mutation.

We pick the **first option (inside `refresh_employee_data()`)**. Rationale: the over-eagerness is harmless (the loader call is cheap, the network round trip is local, and no UI state is lost), and the precision win of option 2 is illusory because new mutations would still need to remember to call `refresh_employee_data` (or `EmployeeAction::Refresh`) — which they already do for the per-employee view to update. Bumping the token there means any future mutation that follows the existing pattern gets sidebar refresh for free.

For `employee_work_details` mutations, we make the same choice: anywhere `refresh_employee_data` is invoked indirectly via `EmployeeAction::Refresh`, the token bump rides along.

### Decision 3: Signal type and starting value

`GlobalSignal<u64>` starting at `0`. Wraparound at `u64::MAX` is irrelevant for any realistic session. Using a counter rather than a `bool` toggle avoids the "two consecutive bumps in one frame look identical" hazard.

### Decision 4: Preserving sidebar UI state across refresh

The `search` signal and the `year` signal are local `use_signal` state inside `EmployeesList`. The refresh re-runs only the `use_resource` closure, not the component itself, so local signals survive the refresh. The `Loading…` placeholder briefly replaces the list during the round trip — acceptable, and matches existing behavior on year change.

### Decision 5: Tests

Two test layers:

- **Unit test for the bump**: A test inside `service/employee.rs` confirming that `refresh_employee_data` increments `EMPLOYEES_LIST_REFRESH`. The full async path through the API can't be exercised without a mock layer (see the existing gap in `frontend-extra-hours-edit`'s tasks 8.5), but the bump itself is a synchronous side effect we can isolate by extracting it into a small helper that the test calls directly.

- **Manual browser exercise**: Edit / delete / create an extra hours entry; confirm the sidebar value updates without a full page reload. Listed in tasks §4 as the verification step. The same flow we just used to discover the bug works as the verification.

## Risks / Trade-offs

- **Risk: Refresh thrash if many mutations happen in quick succession.** Mitigation: each refresh is a single HTTP request to a local backend; the listing endpoint is small. Not a real concern at our scale.

- **Risk: Resource re-runs on year change that does not change aggregates.** Mitigation: the resource already re-runs on year change today (year is its existing reactive dependency); bumping the refresh token does not regress this.

- **Risk: Forgetting to bump in a future mutation path that bypasses `refresh_employee_data`.** Mitigation: a comment on `EMPLOYEES_LIST_REFRESH` documenting the contract ("bump this from any service handler that mutates an employee aggregate") plus the centralization in `refresh_employee_data` mean the typical flow already gets it.

- **Trade-off: Eager refresh.** We refresh on year navigation and on per-employee re-read events that do not really need the sidebar to refresh. This is acceptable because the cost is one cheap GET. We can revisit if profiling shows a problem.

- **Trade-off: No optimistic update.** Brief `Loading…` between mutation and refresh. Acceptable for a v1; the list is small enough that the round-trip is barely visible.

## Migration Plan

1. Land the change in a single PR. It is mutually-supportive: sidebar reads the token, service bumps it.
2. Verify manually with the same browser flow used to discover the bug (`frontend-extra-hours-edit` §9.5): edit an extra hours entry, observe the sidebar balance update.
3. Verify the same with delete and add (the existing pre-bug paths) so we know the fix is uniform.
4. **Rollback**: revert the PR. The sidebar reverts to one-shot loading; no data is lost; no migration needed.
