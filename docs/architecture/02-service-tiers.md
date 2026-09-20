# Service Tiers — Basic vs Business-Logic

Shifty separates service implementations into two tiers. This separation
prevents cyclic DI coupling and keeps the construction order in
`shifty_bin/src/main.rs` deterministic.

## Basic Services (entity managers)

A **Basic Service** manages exactly one domain object:

- CRUD + validation + permission gates for its aggregate.
- Consumes **only** DAOs, `PermissionService`, `TransactionDao`.
- Consumes **no** other domain services.

**Examples:**

- `BookingService`
- `RebookingBatchService` — entity manager for `rebooking_batch` +
  `rebooking_batch_entry` (v2.6 Phase 54). HR-gated CRUD only; the
  first consumer arrives in Phase 55 (`RebookingReconciliationService`).
  Deps: `RebookingBatchDao`, `PermissionService`, `ClockService`,
  `UuidService`, `TransactionDao`. See feature
  [F14](../features/F14-rebooking.md).
- `PersonService` — entity manager for the `person` identity parent
  (v2.7 Phase 61). All four methods are HR-gated; two of them are the
  "unlinked candidate" reads that Phase 63 needs for the candidate picker.
  Deps: `PersonDao`, `PermissionService`, `ClockService`, `UuidService`,
  `TransactionDao` — deliberately no domain service, because one would close
  a DI cycle through `PermissionService`. Constructed in the basic block of
  `shifty_bin/src/main.rs`, strictly before the business-logic layer. Since
  v2.7 Phase 62 the first consumer context exists: `PersonDirectoryService`
  sits one tier above and reads the same tables through its own DAO — the
  basic service itself stays untouched and HR-gated. See
  feature [F01](../features/F01-employee-management.md) §4.5.
- `SalesPersonService`
- `SalesPersonUnavailableService`
- `SlotService`
- `ShiftplanService` (master data)
- `SpecialDayService`

## Business-Logic Services

A **Business-Logic Service** combines multiple aggregates or maintains
cross-entity invariants:

- Consumes Basic Services and other Business-Logic Services — as long as
  no cyclic coupling arises.
- Often aggregates read-only data from multiple Basic Services (typically
  with `Authentication::Full` internally, see
  [`04-auth.md`](./04-auth.md)).

**Examples:**

- `AbsenceService` — multi-day range, category logic, conflict lookups.
- `ShiftplanViewService` — read aggregate over Slot + Booking + Absence.
- `ShiftplanEditService` — write aggregate with booking migration on
  slot changes.
- `ReportingService` — balance calculation over Booking + ExtraHours +
  Absence + Carryover + SpecialDay.
- `BookingInformationService` — enriched booking views.
- `CarryoverService` — year-end snapshot with cross-year consistency.
- `WorkingHoursService` — expected-hours calculation.
- `BillingPeriodReportService` — snapshot creation.
- `VoluntaryStatsService` — read-only F1/F2 aggregate on top of
  `ExtraHoursService` + `EmployeeWorkDetailsService` +
  `SalesPersonService` (v2.6 Phase 54). HR-only via API-level
  None-redaction (Non-HR receives all-`None` fields, not 403).
  See feature [F14](../features/F14-rebooking.md).
- `RebookingReconciliationService` — F3/F5 rebooking write orchestrator
  (v2.6 Phase 55). Owns the single internal pair-write-path shared by
  the F3 manual one-shot and the F5-approve lifecycle. Consumes
  `ExtraHoursService`, `RebookingBatchService`, `ReportingService`,
  `VoluntaryStatsService` (+ `PermissionService`, `TransactionDao`) — a
  clear Business-Logic service (four domain-service deps). No cycle:
  none of those four know it back. All four methods HR-gated
  (T-55-05). DI-constructed in `shifty_bin/src/main.rs` **after** all
  Basic services and after `ReportingService` + `VoluntaryStatsService`
  (order: `extra_hours` → `rebooking_batch` → `reporting` →
  `voluntary_stats` → `rebooking_reconciliation`, last). See feature
  [F14 §8](../features/F14-rebooking.md).
- `VoluntaryRebookingScheduler` — F4 auto-cron scheduler (v2.6
  Phase 56). A thin `tokio-cron-scheduler` job (Monday 03:15, 6-field
  cron `0 15 3 * * 1`) that computes the previous ISO week and delegates
  every capped person to `RebookingReconciliationService::run_for_week`.
  Consumes `RebookingReconciliationService` (the engine) +
  `EmployeeWorkDetailsService` (the capped-person source) (+
  `PermissionService`, `ClockService`, `TransactionDao`) — a
  Business-Logic service (two domain-service deps). Mirrors
  `PdfExportScheduler` minus `reload_from_db` (the stichtag toggle is
  read fresh each run inside `run_for_week`). Booted in
  `shifty_bin/src/main.rs` **after** `pdf_export_scheduler.start()` at
  the end of `main()` — the DI-wiring trigger for this doc. No REST
  surface (the `/admin/rebooking/backfill` endpoint hangs on
  `RebookingReconciliationService`). See feature
  [F14 §9](../features/F14-rebooking.md).
- `PersonDirectoryService` — read aggregate behind the single person list
  that replaces the former tabs of the user administration (v2.7
  Phase 62). Deps: `PersonDirectoryDao`, `PermissionService`,
  `TransactionDao` — **no** domain service at all. It is Business-Logic
  nonetheless, because it merges the two carrier aggregates of a person
  (`sales_person` and `user`) into one view and owns the `PersonKind`
  invariant across them; see the clarification under rule 3 below. `list`
  is gated on the literal `"admin"` and **propagates** the error —
  deliberately unlike `VoluntaryStatsService`, which redacts to `None`
  fields: a redacted directory would be indistinguishable from an empty
  one. DI-constructed in `shifty_bin/src/main.rs` **after** the basic
  block, in particular strictly after `PersonService`, directly following
  `voluntary_stats_service`. See feature
  [F01](../features/F01-employee-management.md) §4.6.

## Rules

1. **If two services need each other:** one is Basic, one is
   Business-Logic. The Basic one does not know the Business-Logic
   service. If needed, the cross-entity operation moves into a
   third service one tier higher.
2. **DI construction in `main.rs`:** first all Basic Services, then the
   Business-Logic tier — no `OnceLock` / forward-declaration tricks.
3. **Rule of thumb for classification:** count dependencies.
   - Only DAOs + Permission + Transaction → Basic.
   - As soon as another domain service appears as a dep → Business-Logic.

   **The rule of thumb is an indicator, not the decision.** The substantive
   question decides: does the service manage exactly *one* aggregate? A read
   aggregate that merges several domain objects into one view is
   Business-Logic even when its dependency list looks like that of a Basic
   service. `PersonDirectoryService` (v2.7 Phase 62) is exactly that case:
   three dependencies, no domain service — and still Business-Logic, because
   it joins `sales_person` and `user` into one person view.

## Why two tiers?

**Without the separation** you quickly end up in cyclic dependencies:

- `BookingService` wants to check for an absence conflict on deletion →
  calls `AbsenceService`.
- `AbsenceService` wants to check on absence deletion whether a booking
  refers to it → calls `BookingService`.

With tiers the cycle is broken explicitly: `BookingService` stays Basic
and does **not** know `AbsenceService`. The cross-entity check moves into
a third service (e.g. `ShiftplanEditService`) that consumes both.

## The service graph

The actual DI wiring from `shifty_bin/src/main.rs` is generated as a
Mermaid diagram in
[`diagrams/service-graph-runtime.mmd`](./diagrams/service-graph-runtime.mmd).
The trait-declaration version (what every service **demands** as a
dependency, independent of ordering) lives in
[`diagrams/service-graph-traits.mmd`](./diagrams/service-graph-traits.mmd).

## History

The tier convention was formalized after the fact, after two refactoring
cycles failed due to hidden cycles. It is codified normatively in
`shifty-backend/CLAUDE.md` and is actively enforced during service
reviews.
