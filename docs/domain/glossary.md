# Glossary — Domain Terms in Shifty

This glossary is the source of truth for terms. When two documents use the
same term differently, this one is right — or it is a contradiction that
needs to be resolved.

## A

**Absence.** Range-based absence (v1.0+), e.g. Vacation, SickLeave,
UnpaidLeave. Replaces single-day Extra Hours after cutover. Range is
**inclusive on both sides** `[from, to]`. Details: [F05](../features/F05-absence-system.md).

**Absence Period.** A concrete row in the `absence_period` table,
identifying a range + category + Sales Person.

**Aggregate.** Domain-level grouping of multiple entities that must remain
consistent together (classic DDD). See
[architecture diagrams](../architecture/diagrams/domain-aggregates.mmd).

**Authentication::Full.** Auth enum variant that lets all permission
checks pass. **Exclusively for internal aggregate reads** by
business-logic services. Details:
[04-auth.md](../architecture/04-auth.md).

## B

**Balance / Balance Hours.** The calculated difference between actual
worked hours and contractually expected hours, plus/minus extras
(vacation, sick leave, holidays). Formula:
`balance = worked − expected + carryover`. Details:
[time-accounting.md](./time-accounting.md).

**Basic Service.** Service class that manages exactly one domain object
and does not consume other domain services. See
[02-service-tiers.md](../architecture/02-service-tiers.md).

**Billing Period.** Billing time range in which the balance/vacation/hours
of a Sales Person are frozen into a Snapshot. Details:
[billing-period.md](./billing-period.md) and
[F08](../features/F08-billing-period.md).

**Block.** Time slice for reports (usually a calendar week).
`My Block` = user's own view. `Block Report` = HR view with aggregation.

**Booking.** Assignment *Sales Person × Slot × date* — the core of shift
planning. Details: [F03](../features/F03-booking.md).

**Booking Log.** Read-only audit trail on `bookings_view`, including
soft-deletes. Shows who created/modified a Booking and when.

**Business-Logic Service.** Service class that combines multiple
aggregates or maintains cross-entity invariants. May consume other
domain services.

## C

**Carryover.** Balance persisted at year-end (hours and/or vacation days)
that rolls into the following year. Updated by the scheduler weekly for
the previous and current year. Avoids recomputing historical time ranges.

**Contract.** A row in `employee_work_details` with weekly hours,
weekdays, valid-from/valid-to. A Sales Person can have multiple contract
rows over time.

**Custom Extra Hours.** Extra category defined by the operation in
addition to the standard enum values. Referenced per row.

## E

**Expected Hours.** Contractually expected hours per time range. Derived
from Contract × days − Special Days − UnpaidLeave.

**Extra Hours (Legacy).** Single-day time rows for overtime, vacation,
sick leave, etc. Replaced by the Absence system after cutover, but
continues to exist for historical data. Categories:
`ExtraWork`, `Vacation`, `SickLeave`, `Holiday`, `Unavailable`,
`UnpaidLeave`, `VolunteerWork`, `CustomExtraHours`.

## F

**Fat Backend, Thin Client.** Core principle: business logic lives
exclusively in the backend. Frontend only renders.

**Feature Flag.** Static / boolean-oriented switch, usually set by the
admin. Difference from Toggle: no effective date, no user context.
Details: [F13](../features/F13-system-infrastructure.md).

## G

**gen_service_impl!.** Macro (`service_impl/src/macros.rs`) that wires
service implementations with their typed dependencies.

## H

**HR Gate.** Auth rule that restricts an operation to users with the HR
role (e.g. creating Billing Periods, editing others' absences).

## I

**Impersonation.** Admin feature that lets a support user act as another
user in order to reproduce their view. Session holds an `impersonate`
flag.

## O

**OIDC.** OpenID Connect. Production auth mode.

## P

**Permission Service.** Central checkpoint for role-based authorization.
Core bypass: `Authentication::Full`.

## R

**RBAC.** Role-Based Access Control. Shifty's roles are defined in
migrations; details: [F12](../features/F12-auth-session.md).

**Report.** Aggregate of Bookings + Extra Hours + Absence + Carryover +
Special Days that yields a Balance and further metrics. Details:
[F07](../features/F07-reporting-balance.md).

**Re-Point.** Data move: Bookings are re-attached from one Slot to
another (e.g. on slot split). MUST run atomically in a single
transaction, otherwise double-counting occurs.

## S

**Sales Person.** Employee entity with contract history, color choice,
availability windows. Details: [F01](../features/F01-employee-management.md).

**Session.** User's login state. Cookie-based, optionally marked
`impersonate`. 365-day expiry.

**Shiftplan.** Aggregate of Slots + Special Days + catalog + editor. Not
a domain object but the application's view of "who works when".

**Shiftplan Edit.** Business-logic service for editing shift plans
including slot split, booking migration, week-lock check.

**Slot.** Time window with capacity (`min_resources`,
`max_paid_employees`) per weekday. Bookings fill slots.

**Snapshot.** Frozen view of Balance/hours/vacation in a Billing Period.
Write-once, versioned with `snapshot_schema_version`. The contract for
formula changes is strict — see
[billing-period.md](./billing-period.md).

**Snapshot Schema Version.** `pub const u32` in
`service_impl::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION`.
Currently **12**. Bump rules in
[F08](../features/F08-billing-period.md).

**Soft-Delete.** Deletion by setting a `deleted` timestamp column
instead of `DELETE FROM`. Readers filter `WHERE deleted IS NULL`.

**Special Day.** Public holiday or operational special day that
influences the Expected Hours calculation.

**Stichtag Rollout (Effective-Date Rollout).** Pattern for Toggle
features: from date X the new semantics apply; before that the old
semantics remain valid. Reporting must handle both semantics
consistently across the timeline.

## T

**Toggle.** User- and/or date-dependent switch, often used for
effective-date rollouts (e.g. D-51-07). Difference from Feature Flag:
time- and context-dependent.

**Transaction (Option\<Transaction\>).** Pattern in which every service
method accepts `Option<Self::Transaction>` — opens its own if `None`,
joins the outer transaction if `Some`.

## U

**UnpaidLeave.** Extra Hours category with special semantics:
**reduces expectation, adds nothing** to the actual side. Other
categories (Vacation, SickLeave, Holiday) do NOT reduce expectation but
instead add to the actual side.

## V

**Vacation Balance.** Current vacation balance: entitlement + carryover
− used − planned. Formula:
`balance = entitled + carryover(year−1) − (used + planned)`. Details:
[F06](../features/F06-vacation-management.md).

**Vacation Entitlement Offset.** Manual correction of the vacation
entitlement (bonuses, deductions). HR-only editable, HR-only visible.

**value_type.** Enum column in `billing_period_sales_person` that
identifies what kind of value a row carries (e.g. `WorkedHours`,
`VacationDaysUsed`, `Balance`). Extensions force a snapshot version
bump.

## W

**Week Message.** Info text per calendar week, shown in the Shiftplan.

**Week Status.** Release state of a week (`Unset`, `Planned`, `Locked`,
`Released`). Controls who is still allowed to make changes.

**Working Days.** Weekday flags in the contract that define on which
days the Sales Person fundamentally works.

**Working Hours.** Contractually expected hours per week.
