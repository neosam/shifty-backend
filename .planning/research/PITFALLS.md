# Pitfalls Research

**Domain:** Adding week-locking (WST-01) and attendance-averaging (AVG-01) to an existing Rust/Axum/SQLite shift-planning system (Shifty v2.1)
**Researched:** 2026-07-01
**Confidence:** HIGH — derived from direct code inspection of the live codebase, known CI failure patterns from MEMORY.md, and established v2.1 project constraints from PROJECT.md and CLAUDE.md.

---

## Critical Pitfalls

### Pitfall 1: ISO-Week Year vs. Calendar Year Confusion in the Lock Key

**What goes wrong:**
The `(year, calendar_week)` key used across the codebase — `BookingEntity.year: u32`, `WeekMessageEntity.year: u32 / calendar_week: u8`, and the new week-status table — must store the **ISO week year**, not the Gregorian calendar year. Jan 1 of any year can fall in ISO week 52 or 53 of the *previous* ISO year. If week-status rows are keyed by Gregorian year, then:
- Dec 29–31, 2025 (which are in ISO week 1, **2026**) would be keyed `(2025, 53)` by one code path and `(2026, 1)` by another.
- The lock gate would fail to find the lock row for those days, silently allowing writes to a "locked" week.
- Queries on "week 53" in a year with only 52 ISO weeks will return zero rows rather than an error, masking bugs.

**Why it happens:**
`ShiftyDate::from_ymd` correctly calls `time::Date::to_iso_week_date()` which returns the ISO week year. But the raw `(year: u32, week: u8)` fields on REST DTOs and DAO entities are ambiguous — any caller that supplies `time::now().year()` (the Gregorian year) instead of the ISO week year when constructing the key will silently insert a wrong row. The existing `week_message` DAO uses the same `(year, calendar_week)` pattern, and that was the model WST-01 will follow. Week 53 only exists in some years (e.g., 2026 has 53 ISO weeks); a query for `(2027, 53)` against a 52-week year returns silently empty.

**How to avoid:**
- Derive the year field **exclusively** from `ShiftyDate::from_ymd(...).year()` or `ShiftyWeek::new(...)`, never from `date.year()` (Gregorian).
- Add a DB-level CHECK constraint on the migration: `CHECK (calendar_week >= 1 AND calendar_week <= 53)`, plus a SQLite trigger or application-level validation that rejects `calendar_week = 53` when `year` does not have 53 ISO weeks (`time::util::weeks_in_year(year) < 53`).
- Add a unit test: create status for (2020, 53) — a year with 53 weeks — and verify that `ShiftyWeek::new(2020, 53)` does not wrap to (2021, 1). Also test that `(2021, 53)` is rejected (2021 has only 52 ISO weeks).
- In the REST DTO, document that `year` is the ISO week year.

**Warning signs:**
- Lock gate passes for Dec 29–31 even though the week is locked.
- Unit tests with `year = 2025, week = 53` succeed even though 2025 has only 52 ISO weeks (the SQLite query returns 0 rows).
- Clippy or compiler does not catch this — it is a semantic, not syntactic, error.

**Phase to address:**
WST-01 data-model + migration phase (first WST-01 phase). Verification gate must test week-53 and year-boundary weeks explicitly.

---

### Pitfall 2: Lock Gate Check Outside the Write Transaction (Read-Check-Then-Write Race)

**What goes wrong:**
If the lock-status check is a separate DB read that happens *before* `BookingService::create` or `ShiftplanEditService::book_slot_with_conflict_check` opens/uses its transaction, two concurrent requests can both pass the lock check and both proceed to write — defeating the lock.

SQLite with the project's async pool (sqlx) serializes writes but only within a single connection. Two concurrent HTTP requests using separate pooled connections can interleave: both read `status = Planned`, both proceed, both write their bookings.

**Why it happens:**
The existing write paths pass `Option<Self::Transaction>` through all layers. A callee that does `self.lock_service.get_status(week, tx.clone().into()).await?` *before* passing `tx` to the actual write is safe — but only if the lock is read inside the *same* transaction that will do the write, using `BEGIN IMMEDIATE` or `BEGIN EXCLUSIVE` so SQLite serializes at the lock boundary. SQLite WAL mode (`BEGIN IMMEDIATE`) is the default for sqlx-sqlite; a plain `BEGIN` (deferred) still allows a TOCTOU window.

**How to avoid:**
- Enforce the lock check **inside** the business-logic service method, after `use_transaction(tx)` has been called, before any write — not in the REST layer before calling the service.
- For SQLite, use `BEGIN IMMEDIATE` (not `BEGIN DEFERRED`) for any transaction that includes a lock-gate check + write. Confirm the sqlx pool is configured with `PRAGMA journal_mode=WAL` and that transactions are opened with `BEGIN IMMEDIATE`.
- Alternatively, encode the lock as a DB-level constraint: a `week_status` row with `status = 'Locked'` could gate bookings via a SQLite trigger that raises an error if a booking insert targets a locked week and the actor is not shiftplanner. This moves atomicity to the DB engine.
- Add an integration test: lock week, fire two concurrent booking creates for the same employee/slot/week, assert exactly one succeeds (or that both fail if the week is locked for non-shiftplanners).

**Warning signs:**
- Lock check is implemented in the REST handler (`rest/src/`) rather than in `service_impl/src/shiftplan_edit.rs`.
- Transaction passed as `None` to the lock-status lookup while the booking write opens its own transaction.
- No `BEGIN IMMEDIATE` in the sqlx pool config.

**Phase to address:**
WST-01 service implementation phase. Must be part of the service-tier discussion (discuss-phase) decision log.

---

### Pitfall 3: Incomplete Write-Path Audit — `modify_slot_single_week` Bypass

**What goes wrong:**
The lock gate is added to `ShiftplanEditService::book_slot_with_conflict_check` and `modify_slot`, but `modify_slot_single_week`, `remove_slot`, and `copy_week_with_conflict_check` are left ungated. A non-shiftplanner can then modify a slot for one week (single-week override), delete a slot, or copy a week's bookings into a locked week without hitting the gate.

This exact class of bypass was previously flagged for the paid-capacity gate (Phase 24 found that `modify_slot` propagated `max_paid_employees` incorrectly for the single-week path), demonstrating that multi-path coverage is a genuine Shifty failure mode.

**Why it happens:**
Developers naturally gate the most obvious write path (`book_slot_with_conflict_check`) and forget secondary paths. `modify_slot_single_week` is architecturally distinct from `modify_slot` (it creates three slot segments), so it can be independently missed. `copy_week_with_conflict_check` already requires `shiftplan.edit` but the lock gate for the *destination* week is a separate concern.

**How to avoid:**
- During the discuss-phase, enumerate every write path that touches booking or slot data for a given `(year, week)`:
  1. `book_slot_with_conflict_check` — adds a booking
  2. `remove_slot` — affects all bookings on that slot
  3. `modify_slot` — splits a slot from a given week
  4. `modify_slot_single_week` — creates a week-specific override
  5. `copy_week_with_conflict_check` — adds bookings to the destination week
  6. `add_vacation` / extra-hours writes that touch the week indirectly
- Implement a shared internal helper `assert_week_not_locked(year, week, context, tx)` in `ShiftplanEditServiceImpl`, called at the top of every write method, so it cannot be forgotten on new paths.
- Write a test matrix: one test per write path, one locked / one unlocked case per path.

**Warning signs:**
- `modify_slot_single_week` does not call any lock-status service.
- The REST handler for slot modification (which calls `modify_slot` or `modify_slot_single_week`) skips a `week_status` lookup.
- Integration test coverage exists for booking creates but not for slot modifications on locked weeks.

**Phase to address:**
WST-01 service implementation phase. The "Looks Done But Isn't" checklist must enumerate all six write paths above.

---

### Pitfall 4: Permission Model Ambiguity — Who Locks, Who Unlocks, Who Bypasses

**What goes wrong:**
Three distinct permission questions are conflated:
1. Who can **change** week status (None → InPlanning → Planned → Locked)?
2. Who can **write booking/slot data to a locked week**?
3. Who can **unlock** a previously locked week?

If all three are answered with "shiftplanner," the model is consistent. But if, for example, HR can change status to Locked but cannot write to a locked week, or if the status setter role is undefined, then the discuss-phase decision is incomplete and the executor will guess — likely incorrectly.

There is also a secondary risk: "Gesperrt-Wochen nur noch vom Schichtplaner änderbar" (from PROJECT.md) implies shiftplanner CAN write to locked weeks. But the gate must not block the shiftplanner's own lock-management write (status change). If the gate checks `status = Locked → reject unless shiftplanner` and the status-change endpoint itself requires the same check, a shiftplanner trying to unlock a week would hit their own gate — a logic loop.

**Why it happens:**
RBAC gates are added incrementally. Each write path's gate is designed independently. The status-mutation path (setting `status = Locked`) and the data-mutation path (adding bookings to the week) share the same `(year, week)` but have different actors and different gate logic.

**How to avoid:**
- Decide in the discuss-phase and record in the decision log: (a) status changes require `SHIFTPLANNER_PRIVILEGE`; (b) writes to locked weeks are rejected unless caller has `SHIFTPLANNER_PRIVILEGE`; (c) the status-mutation path is gated by shiftplanner but is NOT subject to the "locked week" data-write gate (since it is a status change, not a booking/slot write).
- Use two separate gate functions: `check_week_writable(year, week, context)` (rejected for locked + non-shiftplanner) and `check_permission(SHIFTPLANNER_PRIVILEGE, context)` (for status mutation). Never apply `check_week_writable` to the status-change endpoint itself.

**Warning signs:**
- The discuss-phase CONTEXT block has no explicit decision on who sets status vs. who bypasses locks.
- `check_week_writable` is applied to the `update_week_status` service method.
- An integration test for "shiftplanner locks a week then edits a booking in it" fails because the shiftplanner is also blocked.

**Phase to address:**
WST-01 discuss-phase must produce explicit decision log entries for all three permission questions.

---

### Pitfall 5: Stale Lock State in the WASM Frontend — SelectInput D-25-06 Class

**What goes wrong:**
The week-status selector (None / In Planung / Geplant / Gesperrt) is a controlled `<select>` driven by a Dioxus signal. Two failure modes of the D-25-06 class apply here:

(a) **Stale cached status on navigation:** The user navigates away from the week view and back. If the status signal is not reloaded from the server on re-entry, the badge shows the old status while the server state has changed (e.g., another user locked the week).

(b) **Controlled-select desync after failed write:** A non-shiftplanner tries to edit a booking on a locked week. The server returns 403. The frontend signal rolls back, but the `<select>` DOM element does not, leaving the displayed status out of sync. The SDF-Desync fix pattern (Option 2: don't reset the form after create) avoids this in the Special-Days case by *not* resetting. But for week status, if the status selector resets its signal on success and the DOM has already moved to a new value, the controlled value attribute may not re-render because Dioxus diffing sees the same virtual DOM node.

**Why it happens:**
Dioxus's VDOM diffing does not always re-apply `value=` attribute to `<select>` if the attribute value is unchanged from the previous render cycle, even if the DOM has drifted. This is the documented D-25-06 constraint. Setting a `SelectInput value=Some(current_status_string)` on re-render works only if the signal value actually changes to trigger a diff.

**How to avoid:**
- Do not use `SelectInput` in controlled mode for the week-status selector if the status is only changed by a server call. Instead: display the status as a read-only badge + a separate action button (e.g., "Lock week") that fires a POST and on success reloads the week from the server. This avoids the desync entirely.
- If a dropdown is required: on every successful status change (and on every failed booking attempt), force a signal refresh that changes the status signal to a placeholder (`""`) then back to the correct value in two consecutive render cycles, to guarantee DOM re-application. This is fragile; the badge+button approach is safer.
- The SDF-Desync pattern from v2.1 (SDF item) applies: the safest fix is to never reset the controlled select — instead let the server response drive the next displayed value via a reload.
- Add a frontend component test (cargo test on `shifty-dioxus`) that verifies the status badge renders the server-returned value after a round-trip, not the last user selection.

**Warning signs:**
- The status selector is `SelectInput { value: Some(week_status_signal.read().to_string()) ... }` with a local signal that is mutated on user change before the server confirms.
- No explicit reload of week status from server on booking 403.
- No frontend component test for the lock badge.

**Phase to address:**
WST-01 frontend phase. Discuss-phase should explicitly decide: read-only badge + action button vs. controlled dropdown.

---

### Pitfall 6: AVG-01 Denominator Definition — Vacation vs. All Absence Categories

**What goes wrong:**
PROJECT.md specifies "Urlaub aus dem Nenner" (vacation excluded from denominator). The existing A-22-1 formula in `service/src/reporting.rs` excludes a week where `worked == 0.0 && (vacation + sick_leave + unpaid_leave + holiday) > 0.0`. These are different:

- A-22-1 (current): excludes fully-absent weeks for **any** category.
- AVG-01 (specified): excludes weeks with **vacation** only.

If AVG-01 reuses the existing `average_worked_hours_per_week` function directly, a week of sick leave where the employee worked 0 hours is excluded from the denominator — inflating the average. If the intent is "only vacation weeks are excluded from the denominator," sick leave weeks (where 0 hours were worked) would still count as 0 in the denominator, pulling the average down, which may be correct for attendance tracking but was not explicitly decided.

**Why it happens:**
The discuss-phase for AVG-01 was explicitly deferred ("viele offene Definitionsfragen"). An autonomous executor that finds an existing formula that "looks right" will use it without checking whether it matches the specification. The existing `average_worked_hours_per_week` is already in the service trait and already tested — the temptation to call it directly is high.

**How to avoid:**
- The discuss-phase MUST produce a decision on the exact exclusion rule: which absence categories exclude a week from the denominator, and whether the exclusion applies only when worked=0 or always.
- If the rule differs from A-22-1, implement a *separate* function rather than modifying `average_worked_hours_per_week` (modifying it would break STAT-01 behavior and could require a snapshot version bump).
- Write a parametric unit test covering: (1) pure vacation week excluded, (2) sick leave week — included or excluded per decision, (3) week with partial vacation + some hours worked — denominator inclusion rule, (4) empty weeks (no bookings, no absences) — always included as 0.

**Warning signs:**
- AVG-01 calls `average_worked_hours_per_week` without a new function or without verifying the exclusion set.
- No unit test distinguishes vacation-only exclusion from all-absence exclusion.
- The discuss-phase CONTEXT decisions block has no explicit statement on sick-leave week denominator treatment.

**Phase to address:**
AVG-01 discuss-phase (must produce explicit decision); AVG-01 service implementation phase (tests must cover all four scenarios above).

---

### Pitfall 7: AVG-01 Snapshot Version Bump — Silently Skipped for a Persisted Computation

**What goes wrong:**
If AVG-01 is implemented as a new `BillingPeriodValueType` (e.g., `AverageAttendanceHours`) written into billing-period snapshots, `CURRENT_SNAPSHOT_SCHEMA_VERSION` must be bumped from 12 to 13. If the executor skips the bump, old snapshots (written at version 12) will be compared by the billing-period validator against new live computations that include the new value type. The validator will see mismatches and flag drift for every historical billing period — but silently, since the schema version check is the guard that prevents the validator from running on mismatched snapshots.

Worse: the bumped constant is the *only* mechanism that tells the system "re-validate these snapshots under the new rules." Without the bump, old snapshots appear valid (same version) even though the computation has changed, defeating the validator's purpose entirely.

PROJECT.md already calls this out: "AVG-01 in discuss-phase prüfen (falls neue **persistierte** Berechnung → Bump nötig; reines Read-Aggregat → kein Bump)."

**Why it happens:**
The bump discipline requires the developer to recognize that adding a new `BillingPeriodValueType` variant is a persisted-computation change. An autonomous executor that implements AVG-01 as a live-compute REST endpoint with no new snapshot row correctly skips the bump. But if any refactoring also touches the billing-period report builder to write a new row, the bump becomes mandatory — and the two changes can happen in different commits without either commit triggering the bump.

**How to avoid:**
- The discuss-phase must explicitly decide: is AVG-01 a read-only aggregated REST endpoint (no new `BillingPeriodValueType`) or a persisted snapshot value? Record this as a decision in the CONTEXT block.
- If persisted: the plan phase must include a task "bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` from 12 to 13" as a non-optional checklist item, with a companion integration test that verifies the constant is > 12.
- If not persisted: the plan phase must explicitly state "no bump" with rationale, and the executor must not add any new `BillingPeriodValueType` variant.
- CI cannot auto-detect a missing bump — it is a semantic rule, not a compiler rule. The only automated guard is a test that compares computed values for a known billing period against a snapshot written at the current constant.

**Warning signs:**
- A new `BillingPeriodValueType::AverageAttendance` or similar variant appears in `billing_period_report.rs` but `CURRENT_SNAPSHOT_SCHEMA_VERSION` is still 12.
- The billing-period snapshot builder's `build_and_persist_billing_period_report` writes rows with the new type but the comment "KEIN value_type-Change → KEIN CURRENT_SNAPSHOT_SCHEMA_VERSION-Bump" is left unchanged.
- No test asserts the constant's current value.

**Phase to address:**
AVG-01 discuss-phase (decision); AVG-01 service implementation phase (bump or explicit no-bump in plan).

---

### Pitfall 8: Missing `.sqlx` Offline Cache After Adding New Queries — CI Breaks Silently

**What goes wrong:**
WST-01 requires a new `week_status` table and new `query!` / `query_as!` macros in `dao_impl_sqlite/src/`. SQLx compile-time checking requires a local database during `cargo build`. CI uses `SQLX_OFFLINE=true` and relies on `.sqlx/` metadata files committed to the repo. If the executor adds new `query!` macros but does not run `cargo sqlx prepare --workspace` and commit the generated `.sqlx/` files, the CI clean build fails even though `cargo test` on the developer's machine (with a live DB) is green. The overnight autonomous run cannot run `cargo sqlx prepare` without a live DB being available in CI.

This is a documented pattern in MEMORY.md: "nach jeder neuen query!/query_as! cargo sqlx prepare --workspace + .sqlx committen."

**Why it happens:**
`cargo sqlx prepare` requires both a running database and the `sqlx-cli` tool, neither of which is available in CI. The executor can run the full test suite locally against the test DB, which uses in-memory SQLite and does not go through SQLX_OFFLINE mode, so local tests pass but CI fails. The `.sqlx/` directory is checked in, but only contains entries for previously-prepared queries.

**How to avoid:**
- Every phase plan that adds any `query!` / `query_as!` / `query_scalar!` macro in `dao_impl_sqlite/` MUST include as the final step: "run `cargo sqlx prepare --workspace` in a nix-shell with sqlx-cli available, commit the updated `.sqlx/` directory."
- The phase's pre-commit checklist must verify: `git status .sqlx/` shows the new query entries.
- The gate command sequence for autonomous phases must be: `cargo test --workspace && cargo clippy --workspace -- -D warnings && (check .sqlx/ was updated)`.
- In practice: use `nix develop` (not `nix-shell`, see MEMORY.md), then run `cargo sqlx prepare --workspace` before committing.

**Warning signs:**
- New files in `dao_impl_sqlite/src/` contain `query!` or `query_as!` macros but `.sqlx/` directory was not modified in the same commit.
- CI fails with error: `error: failed to find data for query` or `error: offline mode is active but no offline data was found for the query`.
- `cargo build` succeeds locally but CI fails.

**Phase to address:**
WST-01 DAO implementation phase. Must be the last step before the commit. AVG-01 if it adds new DB queries.

---

### Pitfall 9: `cargo clippy --workspace -- -D warnings` Failures From Status Enum Patterns

**What goes wrong:**
Adding a new `WeekStatus` enum (None/InPlanning/Planned/Locked) introduces exhaustive match requirements. If existing match arms anywhere in the codebase pattern-match on a related enum or use `_` wildcards in ways that Clippy flags as unreachable or redundant after the new variant is added, the build fails with `-D warnings`. Common Clippy failures in this scenario:
- `unreachable_patterns` if existing match arms cover a superset after new variant addition.
- `dead_code` if the status DAO module is added but a public function is never called from tests.
- `clippy::match_wildcard_for_single_variants` if `_` covers only one remaining variant.
- Naming: `None` as a variant name shadows `Option::None`; Clippy may warn about `clippy::enum_variant_names` if the variant names don't follow a consistent prefix.

Note: The `shifty-dioxus` Clippy gate is broken (E0514 cross-compilation issue documented in MEMORY.md) and is not CI-gated. Only `cargo clippy --workspace -- -D warnings` in the backend workspace matters for CI.

**Why it happens:**
The backend workspace runs a strict Clippy gate via `nix build`. The executor runs `cargo test` and `cargo build` as verification gates but may skip the explicit `cargo clippy --workspace -- -D warnings` step — which is the only way to catch these warnings before CI.

**How to avoid:**
- Every phase gate MUST include `cargo clippy --workspace -- -D warnings` as a required step, not optional. This is stated in CLAUDE.md.
- Name the enum `WeekStatus` with variants `None` spelled as `Unset` or `Open` to avoid shadowing `Option::None` and triggering Clippy name warnings.
- After adding the enum, run a grep for all match arms that could be affected: `grep -r "WeekStatus\|week_status" --include="*.rs"`.
- Verify the test suite and `main.rs` DI wiring both reference the new service, so `dead_code` warnings are avoided.

**Warning signs:**
- Phase commit message says "cargo test green" but does not mention `cargo clippy`.
- The new `WeekStatus` enum has a variant named `None`.
- New DAO module functions that are only called from tests are `pub` but not referenced in the service layer yet (dead_code warning).

**Phase to address:**
All WST-01 and AVG-01 implementation phases. Clippy must be the last gate before every commit.

---

### Pitfall 10: AVG-01 Employee Scope Leak — Flexible vs. Fixed Employees

**What goes wrong:**
AVG-01 is described as "Durchschnitts-Anwesenheit bei flexiblen Stunden Mitarbeitern" (average attendance for flexible-hours employees). If the computation runs over all employees but the result is displayed only for flexible employees in the UI, a backend endpoint that returns aggregated averages across all employees exposes fixed-contract employees' data in a context they should not appear in. Conversely, if the filtering happens in the frontend but the backend returns all employees, a UI bug or future API consumer could show wrong data.

Additionally, the definition of "flexible" employee in the data model is ambiguous: is it `expected_hours = 0.0`? Is it a flag? Is it the absence of a fixed contract? If different code paths use different definitions, some employees will be double-counted or excluded incorrectly.

**Why it happens:**
The discuss-phase for AVG-01 was deliberately deferred. An executor implementing AVG-01 without a finalized definition will pick the most obvious heuristic (`expected_hours == 0.0` or `is_paid = false`), which may not match the intended semantics. The `SalesPersonService` does not currently have a "is_flexible" flag.

**How to avoid:**
- The discuss-phase must define: (a) exactly which field or combination of fields identifies a "flexible" employee; (b) whether the filtering is server-side (only those employees are returned in the AVG-01 endpoint) or client-side (endpoint returns all, UI filters).
- Server-side filtering is strongly preferred for access control: a HR-only average should not be computable by any non-HR client from raw per-employee data.
- If a new "is_flexible" flag is needed, it belongs in `EmployeeWorkDetails` (already time-versioned), not as a separate table. Add it there to keep the data model consistent.
- Unit tests must explicitly verify that a fixed-contract employee's weeks do not appear in the AVG-01 denominator or numerator.

**Warning signs:**
- AVG-01 REST endpoint returns data for all employees without filtering.
- The service implementation checks `expected_hours == 0.0` as the flexibility predicate without a recorded decision log entry.
- No unit test asserts that a fixed-contract employee is excluded from the aggregate.

**Phase to address:**
AVG-01 discuss-phase (define "flexible"); AVG-01 service implementation phase (tests must include a mixed-employee set).

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Reuse `average_worked_hours_per_week` for AVG-01 without reviewing exclusion logic | No new code | Wrong denominator if vacation-only exclusion differs from all-absence exclusion | Never — review first, then decide |
| Skip `cargo sqlx prepare` and commit without updated `.sqlx/` | Faster commit | CI breaks on clean build; autonomous run has no human to fix it | Never for autonomous overnight runs |
| Gate lock check in REST layer instead of service layer | Simpler service code | Bypassable if another client calls the service directly; violates layered-arch contract | Never |
| Add `WeekStatus::None` variant name | Obvious naming | Shadows `Option::None`; Clippy `-D warnings` fails CI | Never |
| Display lock badge without server reload on navigation | Simpler frontend state | Shows stale lock state; non-shiftplanner sees unlocked badge and attempts writes that fail with 403 | Never for production |
| Omit week-53 test | Faster test implementation | Silent year-boundary bug that manifests only once per 5-6 years | Never — add the test |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| SQLite lock gate | Read lock status before opening transaction, write booking in separate transaction | Read lock status and write booking inside one `BEGIN IMMEDIATE` transaction |
| `cargo sqlx prepare` in NixOS | Run from default shell (sqlx not on PATH) | Run inside `nix develop` (not `nix-shell`, shell.nix is broken per MEMORY.md) |
| Dioxus `SelectInput` for status | Drive with local signal mutated on user change, confirmed by server later | Drive with server-returned value; refresh signal after server confirms |
| Billing period snapshot validator | Add new `BillingPeriodValueType` without bumping `CURRENT_SNAPSHOT_SCHEMA_VERSION` | Bump constant and record in plan; test that constant > previous known value |
| `ShiftyWeek::new(year, week)` | Pass Gregorian year instead of ISO week year | Always derive year from `ShiftyDate::from_ymd(...).year()` or `date.to_iso_week_date().0` |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Loading all bookings for all weeks to check lock status | Slow week-view load when many historical bookings exist | Keep week-status as a separate lightweight table; look up status by `(year, week)` only | At ~10k+ bookings per year |
| AVG-01 scanning all weeks for all employees in one query | Slow billing-period report generation | Compute average on the already-loaded per-week slice (as A-22-1 does); do not add a new cross-week DB aggregation query | At ~50+ employees |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Lock gate enforced only in frontend (badge/disabled button) | Any HTTP client bypasses the gate and writes to locked weeks | Gate must be enforced in the service layer; frontend enforcement is only UX sugar |
| `check_week_writable` skips `Authentication::Full` requests | Internal service-to-service calls bypass the lock gate | The lock gate should only apply to `Authentication::Context` calls; `Authentication::Full` is used for internal system operations and should not be blocked by week locks |
| Week-status mutation endpoint lacks `SHIFTPLANNER_PRIVILEGE` check | Any authenticated user can lock or unlock any week | `check_permission(SHIFTPLANNER_PRIVILEGE, context)` required on all status-mutation paths |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No visual distinction between "locked by shiftplanner" and "planned (not locked)" | Non-shiftplanners attempt booking and receive opaque 403 error | Show clear locked badge; disable booking controls proactively when status is Locked and user is not shiftplanner |
| Status change confirmation requires page reload | Badge shows old status after shiftplanner locks a week; next booking from another tab succeeds (server-side blocked, but confusing) | After status change: immediately update local signal from server response, no full reload needed |
| AVG-01 displayed alongside fixed-contract employees without a label | HR cannot distinguish flexible vs. fixed averages | Display AVG-01 only in the flexible-employee section; add "flexible employees only" label |
| Week-53 displayed in the UI as a selectable week when the year has only 52 ISO weeks | Shiftplanner selects week 53 in a 52-week year; the status is saved to a nonexistent week key | Validate week numbers in the UI against `weeks_in_year(selected_year)` before allowing selection |

---

## "Looks Done But Isn't" Checklist

- [ ] **WST-01 lock gate:** Verify gate is applied to ALL six write paths: `book_slot_with_conflict_check`, `modify_slot`, `modify_slot_single_week`, `remove_slot`, `copy_week_with_conflict_check`, and any future slot-creating path. Not just the most obvious one.
- [ ] **WST-01 ISO week year:** Confirm the `year` field in the migration, DAO entity, and REST DTO is the ISO week year, not the Gregorian year. Test with Dec 29–31 of a year that belongs to ISO week 1 of the next year.
- [ ] **WST-01 week-53 validation:** Confirm the system rejects `(year, 53)` for years with only 52 ISO weeks.
- [ ] **WST-01 `.sqlx/` cache:** Confirm `cargo sqlx prepare --workspace` was run and `.sqlx/` was updated after new queries were added.
- [ ] **WST-01 Clippy:** Confirm `cargo clippy --workspace -- -D warnings` passes after all enum and match additions.
- [ ] **WST-01 i18n:** Confirm all four status values (None, InPlanning, Planned, Locked) have translations in `de.rs`, `en.rs`, and `cs.rs`. Missing Czech translation is a common omission.
- [ ] **AVG-01 denominator decision:** Confirm the discuss-phase produced an explicit decision on which absence categories exclude a week from the denominator.
- [ ] **AVG-01 employee filter:** Confirm "flexible employee" is defined with a recorded decision and that the filter is server-side, not only client-side.
- [ ] **AVG-01 snapshot version:** Confirm either (a) no new `BillingPeriodValueType` was added and `CURRENT_SNAPSHOT_SCHEMA_VERSION` is unchanged at 12, or (b) a new persisted type was added and the constant was bumped to 13.
- [ ] **AVG-01 unit tests:** Confirm tests cover pure-vacation week excluded, sick-leave week (whatever was decided), partial-vacation week with hours, and empty week.

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Wrong ISO year in lock key (data already written) | HIGH | Write a migration that re-keys affected rows by converting Gregorian year to ISO week year; verify with known boundary dates; test before running on prod DB |
| Missing `.sqlx/` cache breaks CI | LOW | Run `cargo sqlx prepare --workspace` in nix develop, commit updated `.sqlx/` — CI recovers on next push |
| Snapshot version not bumped (AVG-01 persisted) | MEDIUM | Bump constant, add a migration that marks existing billing-period rows as stale (force re-validation), re-run billing-period validator for all affected periods |
| Lock bypass via ungated write path | MEDIUM | Add missing gate call to the bypassed service method; add regression test for that specific path |
| Stale lock badge in frontend | LOW | Force re-fetch of week status on mount/navigation; no data migration needed |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| ISO-week year confusion (P1) | WST-01 data model + migration | Unit test for (2020, 53), (2021, 53 rejected), Dec 31 boundary |
| Race condition on lock gate (P2) | WST-01 service implementation | Integration test: concurrent booking creates on locked week |
| Incomplete write-path audit (P3) | WST-01 discuss-phase + service implementation | Test matrix: 6 write paths × locked/unlocked |
| Permission model ambiguity (P4) | WST-01 discuss-phase | Decision log entries for 3 permission questions; shiftplanner-locks-then-edits test |
| Stale lock state / SelectInput desync (P5) | WST-01 frontend phase | Component test: status badge after round-trip; reload after 403 |
| AVG-01 denominator trap (P6) | AVG-01 discuss-phase + service implementation | Unit test with vacation-only and sick-only absent weeks |
| Snapshot version bump (P7) | AVG-01 discuss-phase (decision) + service implementation | Assert `CURRENT_SNAPSHOT_SCHEMA_VERSION >= 12`; add test if bumped |
| Missing `.sqlx` cache (P8) | WST-01 DAO implementation (last step) | `git diff --name-only .sqlx/` must show new entries |
| Clippy failures from enum patterns (P9) | Every implementation phase | `cargo clippy --workspace -- -D warnings` in gate command |
| Flexible employee scope leak (P10) | AVG-01 discuss-phase + service implementation | Unit test with mixed flexible/fixed employee set |

---

## Sources

- Direct code inspection: `shifty-backend/shifty-utils/src/date_utils.rs` (ShiftyDate, ShiftyWeek ISO-week handling)
- Direct code inspection: `shifty-backend/service/src/shiftplan_edit.rs` and `service_impl/src/shiftplan_edit.rs` (existing write paths and permission gates)
- Direct code inspection: `shifty-backend/service/src/reporting.rs` (A-22-1 formula, EmployeeWeeklyStatistics)
- Direct code inspection: `shifty-backend/service_impl/src/billing_period_report.rs` (`CURRENT_SNAPSHOT_SCHEMA_VERSION = 12`, `BillingPeriodValueType` variants)
- Direct code inspection: `shifty-backend/shifty-dioxus/src/component/form/inputs.rs` (`SelectInput` controlled-mode implementation, D-05/D-07 props)
- Project charter: `shifty-backend/.planning/PROJECT.md` (v2.1 scope, snapshot version discipline, SDF-Desync decision, WST-01 / AVG-01 requirements)
- Project conventions: `shifty-backend/CLAUDE.md` (clippy hard gate, SQLx offline cache requirement, snapshot version bump rules)
- Known issues memory: MEMORY.md entries on SQLx prepare, Clippy gate, SDF-Desync, D-25-06 class, service tier conventions
- Prior milestone learnings: Phase 23 `modify_slot`/`max_paid_employees` single-week bypass; Phase 24 paid-capacity gate per-path discipline

---
*Pitfalls research for: Shifty v2.1 — WST-01 week locking + AVG-01 attendance averaging*
*Researched: 2026-07-01*
