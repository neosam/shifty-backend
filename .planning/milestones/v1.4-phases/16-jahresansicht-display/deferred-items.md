# Phase 16 — Deferred / Out-of-Scope Items

Discovered during execution of plan 16-02. Logged per the executor scope-boundary rule
(pre-existing issues in files unrelated to the current task are not silently absorbed).

## Pre-existing frontend build breakage: `EmployeeWorkDetailsTO.committed_voluntary` (HEAD)

**Discovered during:** Plan 16-02, Task 1 (WASM-build / test compile).

**Issue:** `EmployeeWorkDetailsTO.committed_voluntary` became a **required** field at HEAD
(committed in `85223cf feat(employee-work-details): thread committed_voluntary through all
backend layers`), but the frontend constructors that build `EmployeeWorkDetailsTO` were
NOT updated. At HEAD the following did not set the field, so `shifty-dioxus` did not compile
even before any Plan 16-02 change:

- `shifty-dioxus/src/state/employee_work_details.rs` — `impl TryFrom<&EmployeeWorkDetails> for EmployeeWorkDetailsTO` (~line 190)
- `shifty-dioxus/src/tests/volunteer_work_tests.rs` — `make_to(...)` test literal (~line 167)

Verified pre-existing: `git show HEAD:shifty-dioxus/src/state/employee_work_details.rs` and
`git show HEAD:shifty-dioxus/src/tests/volunteer_work_tests.rs` both contain **zero**
`committed_voluntary` occurrences, while `git show HEAD:rest-types/src/lib.rs` carries the
required field.

**Action taken (minimal blocking fix only):** Both constructors now initialize
`committed_voluntary: 0.0` so the WASM-build / test-compile gate of Plan 16-02 can run.
This is the wire-default (`#[serde(default)]`) value — semantically neutral.

**Properly belongs to:** Phase 17 — "Editor-Input für `committed_voluntary`
(`contract_modal.rs`)". The frontend `EmployeeWorkDetails` **state struct** does not carry
`committed_voluntary` at all yet; Phase 17 should add the field to the state struct and
thread the real editor value through this `TryFrom`, replacing the `0.0` placeholder.
