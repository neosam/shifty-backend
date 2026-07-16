# 6. Runtime View

Four scenarios that together exercise every architecturally significant
mechanism: layered read fan-out, composable transactions, `Authentication::Full`,
scheduled jobs, snapshot freezing, and warning-based (non-blocking) validation.

## 6.1 Employee Balance Report (central read path)

`GET /report/{sales_person_id}?year=&until_week=` — the most important
computation in the system; also reused by scenarios 6.2 and 6.3.

```mermaid
sequenceDiagram
    participant H as rest::report
    participant R as ReportingService
    participant P as PermissionService
    participant S as sub-services
    participant D as DAOs / SQLite

    H->>R: get_report_for_employee(id, year, until_week, ctx, None)
    R->>P: HR privilege OR own sales person? (or-gate)
    P-->>R: ok
    Note over R: until_week clamped to weeks_in_year(year)
    R->>S: ShiftplanReport: bookings × slots (Full, tx)
    R->>S: ExtraHours: year range (Full, tx)
    R->>S: Absence: derive_hours_for_range → ResolvedAbsence (Full, tx)
    R->>S: SpecialDays + Toggle "holiday_auto_credit" (Full, tx)
    R->>S: Carryover: get_carryover(id, year−1) (Full, tx)
    S->>D: reads (shared tx)
    D-->>S: rows
    S-->>R: partial results
    Note over R: assemble_weeks: weekly cap,<br/>dynamic-contract & no-contract guards,<br/>manual-wins holiday
    R-->>H: EmployeeReport { balance, worked, expected, vacation… }
    H-->>H: → EmployeeReportTO (JSON)
```

Key points: the handler authenticates the *user*; all internal aggregate reads
then run with `Authentication::Full` on the same transaction (pure read, no
commit). Absence hours are **derived at read time** from the contract active on
each day — both the legacy `extra_hours` rows and range-based `absence_period`
rows are aggregated (post-cutover coexistence, see
[absence-system](../domain/absence-system.md)). Formula reference:
[time-accounting](../domain/time-accounting.md).

## 6.2 Booking Creation with Conflict Check (write path with warnings)

`POST /shiftplan-edit/booking` — the standard editor path (the raw
`POST /booking/` path skips the warning layer).

```mermaid
sequenceDiagram
    participant H as rest::shiftplan_edit
    participant E as ShiftplanEditService
    participant B as BookingService
    participant S as sub-services
    participant D as DAOs / SQLite

    H->>E: book_slot_with_conflict_check(booking, ctx, None)
    E->>E: assert_week_not_locked (needs shiftplan.edit)
    E->>S: paid-employee limit check (toggle-gated)
    alt limit hard-enforced and caller not shiftplanner
        E-->>H: Err(PaidLimitExceeded) → 409
    end
    E->>S: overlapping AbsencePeriods? Unavailability?
    S-->>E: warnings (non-blocking)
    E->>B: create(booking, ctx, Some(tx))
    B->>B: validate: week 1..=53, FKs exist,<br/>no duplicate, shiftplan eligibility
    B->>D: INSERT booking (created_by = user)
    E->>D: commit(tx)
    E-->>H: BookingCreateResult + warnings
```

Design intent: domain **conflicts warn instead of block** (booking on an
absence day may be deliberate), while *structural* violations (duplicate,
locked week, hard paid-limit) are errors. The basic `BookingService` stays
reusable; all cross-aggregate policy sits in the business-logic tier.
Existing diagram: [`sequence-booking-create.mmd`](../architecture/diagrams/sequence-booking-create.mmd).

## 6.3 Billing-Period Snapshot Creation (freeze)

`POST /billing-period` (HR) — payout stability mechanism.

```mermaid
sequenceDiagram
    participant H as rest::billing_period
    participant BR as BillingPeriodReportService
    participant SP as SalesPersonService
    participant R as ReportingService
    participant BP as BillingPeriodService
    participant D as DAOs / SQLite

    H->>BR: build_and_persist_billing_period_report(end_date, ctx, None)
    Note over BR: start = end of latest period + 1 day<br/>(first period: UNIX epoch)
    BR->>SP: get_all()
    loop every paid sales person (is_paid only)
        BR->>R: delta / ytd_from / ytd_to / full_year metrics (Full, tx)
    end
    Note over BR: stamp snapshot_schema_version =<br/>CURRENT_SNAPSHOT_SCHEMA_VERSION
    BR->>BP: create_billing_period(bp, ctx, Some(tx))
    BP->>D: INSERT billing_period + one row per (person, value_type)
    BR->>D: commit(tx)
    BR-->>H: BillingPeriodTO
```

The whole snapshot runs in **one transaction** — a consistent read-set against
concurrent writes. Rows are write-once; only the *latest* period may be
deleted (`NotLatestBillingPeriod` otherwise); periods chain seamlessly.
Version semantics: [billing-period](../domain/billing-period.md).

## 6.4 Carryover Update (scheduled job)

The cron-driven year-end rollover — no HTTP involved.

```mermaid
sequenceDiagram
    participant C as SchedulerService (cron)
    participant E as ShiftplanEditService
    participant SP as SalesPersonService
    participant R as ReportingService
    participant CO as CarryoverService
    participant D as DAOs / SQLite

    C->>E: update_carryover_all_employees(year−1, Full, None)
    E->>SP: get_all()
    loop every sales person
        E->>R: get_report_for_employee(id, year, last_week, Full, tx)
        R-->>E: report (balance, vacation)
        E->>CO: set_carryover({carryover_hours = balance, …}, Full, tx)
        CO->>D: UPSERT employee_yearly_carryover
    end
    E->>D: commit(tx) — all employees atomically
    C->>E: update_carryover_all_employees(year, Full, None)
    Note over C: current year re-run absorbs<br/>retroactive changes
```

This is where scenario 6.1's result becomes persistent state: the carryover
row for year *Y* stores the end-of-*Y* balance and is read back as the input
for year *Y+1* (`get_carryover(id, year − 1)`). Known limitation: retroactive
edits in a *closed* year are not auto-invalidated → see
[chapter 11](11-risks-and-technical-debt.md).
