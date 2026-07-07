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
