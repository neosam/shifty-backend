# Feature Research

**Domain:** Shift-planning / Workforce Management — v2.1 new features only
**Researched:** 2026-07-01
**Confidence:** MEDIUM (industry conventions well-established; tool-specific lock semantics vary; AVG-01 definitional choices are product-owner decisions, not researchable facts)

---

## Scope

This document covers only the two new v2.1 features:

- **WST-01** — Calendar-week workflow status (`None / In Planning / Planned / Locked`)
- **AVG-01** — Average attendance reporting for flexible-hours employees, vacation excluded

The existing Shifty capabilities (bookings, slots, absence periods, billing-period reports, carryover, volunteer hours, paid-capacity limits) are treated as given dependencies.

---

## WST-01 — Calendar-Week Status / Lifecycle Gate

### Background: How Tools Handle It

Most shift-scheduling products (Planday, When I Work, Deputy, Shiftboard, Workday, Dayforce) follow a two- or three-state model at the schedule level:

| Tool convention | Analogous WST-01 state |
|---|---|
| Draft / Unpublished | "In Planning" (only planner sees it) |
| Published | "Planned" (employees see, can be notified) |
| Payroll-period locked | "Locked" (no writes except planner) |
| (no status / unstarted) | "None" |

A **four-state** model (None → In Planning → Planned → Locked) as WST-01 proposes is slightly richer than the typical two-state draft/publish but well within industry convention. Adding an explicit "None" sentinel and a "Planned" state between draft and lock is a natural refinement for a tool that runs a weekly planning cycle.

### Table Stakes

Features users of any scheduling tool expect around a week-status model.

| Feature | Why Expected | Complexity | Notes |
|---|---|---|---|
| Visual week badge | Users need to know which week is in what state at a glance | LOW | Color-coded badge on week header: gray=None, blue/yellow=In Planning, green=Planned, red/orange=Locked. One badge per week in the week-selector or plan header. |
| Status transition by planner role | The shift planner drives the lifecycle; other roles are consumers | LOW | Only Shiftplanner role sets the status. Triggered manually, not automatically. |
| Locked week blocks booking/slot writes for non-planner roles | Core value of the lock: "this week is final for everyone else" | MEDIUM | Booking-create, booking-delete, slot-create, slot-edit paths check lock status pre-persist. Returns a distinct error (HTTP 409 or 403) with localized message. |
| Locked week still readable by all roles | Employees must still see the schedule they are locked into | LOW | Read paths unaffected by lock. |
| Planner can still write to Locked week | Planner needs to correct errors after lock | LOW | Permission gate: `is_locked AND role != Shiftplanner` → block. |
| Retroactive edits by planner remain possible | Real operations require post-lock corrections | LOW | No auto-expiry; planner manually moves state if needed. |
| i18n labels for all four states | Existing Shifty convention for all user-facing strings | LOW | de/en/cs translation keys needed. |

### Differentiators

Features that add value beyond baseline but are not expected by default.

| Feature | Value Proposition | Complexity | Notes |
|---|---|---|---|
| Explicit "In Planning" state visible to planner (not employees) | Mirrors the actual planning workflow; planner can work freely before committing to "Planned" | LOW | Industry tools call this "Draft". Employees see nothing in this state, but planner has full edit rights — same as today's behavior before publish. |
| Unlock to lower state | Allows correction: e.g., move Locked → Planned to allow employee changes, then re-lock | LOW | Reverse transitions must be permitted (any state to any lower state by planner). |
| Warning on booking-edit when week is Planned (not yet Locked) | Soft "this week is published; your change will be visible immediately" UX nudge | MEDIUM | Not a hard block — just a visual warning. Skip for MVP; add if user feedback calls for it. |
| State shown in week-selector dropdown | At-a-glance overview when navigating weeks | LOW | Useful for planners scanning multiple future/past weeks. |

### Anti-Features

| Feature | Why Requested | Why Problematic | Alternative |
|---|---|---|---|
| Auto-lock after billing-period close | "Payroll periods should lock automatically" | Timing varies, data corrections still needed post-close; removes planner control | Planner locks explicitly; billing-period close is independent |
| Require explicit unlock to edit Locked week as planner | "Safety net" | Adds friction for correction flows; planner role already signals authority | Role-based gate is sufficient; no extra confirmation step needed |
| Employee-facing status label (expose "Planned" to employees) | Transparency | Confusing to employees who cannot act on planning status | Employees see shifts or not — publication of individual shifts is enough |
| Multi-step approval chain for lock | Enterprise governance | Out of scope for a small-team tool; Shifty has a single Shiftplanner role | Single-role lock is the right model for current scope |

### State-Transition Map

```
None ──────────────────────────────────────────────────┐
  │ planner sets                                        │
  ▼                                                     │
In Planning ──────────────────────────────────────────►─┤ (any state can go back
  │ planner sets                                        │  to any lower state
  ▼                                                     │  by planner only)
Planned ──────────────────────────────────────────────►─┤
  │ planner sets                                        │
  ▼                                                     │
Locked ◄──────────────────────────────────────────────-┘
```

**Permission rules:**
- Any role: read in any state
- Shiftplanner: write in any state; set any state transition
- Non-Shiftplanner: write only when state is None, In Planning, or Planned
- Non-Shiftplanner attempting write on Locked week → error (HTTP 409 with localized message)

### Complexity Summary

| Area | Estimate | Notes |
|---|---|---|
| Data model | LOW | New table `week_status(year, week, status_enum)` with upsert |
| Migration | LOW | Single new SQLite table |
| DAO + Service | LOW | CRUD on a simple keyed record; no join complexity |
| Permission gate | MEDIUM | Must inject into all booking/slot write paths — identify all write endpoints |
| REST + OpenAPI | LOW | Two endpoints: GET status for week, PUT/POST status |
| Frontend badge | LOW | Inline badge in week header; no complex interaction |
| Frontend permission guard | MEDIUM | Frontend must also reflect lock (disable booking buttons) and show badge state |
| i18n | LOW | 4 state labels × 3 locales |

### Dependencies on Existing Shifty Features

- Booking create/delete paths (`ShiftplanEditService`, `BookingService`) — must add lock check
- Slot create/edit paths (`SlotService`, `ShiftplanEditService`) — must add lock check
- Shiftplanner role permission constant (already exists) — gate reuses existing RBAC
- Week-navigation component in frontend — hosts the new status badge + selector

---

## AVG-01 — Average Attendance for Flexible-Hours Employees

### Background: How Tools Handle It

"Average hours worked per week" or "average utilization" for variable-hours staff is a standard reporting metric in workforce analytics, but the **exact definition is not standardized** across tools. The key axes are:

1. What is the **reference period** (denominator's time unit)?
2. What goes in the **numerator** (hours booked, hours present, days present)?
3. What is the **denominator** (total weeks in period, total weeks minus vacation weeks, total scheduled hours minus vacation hours)?
4. Which **absence categories** are excluded?
5. Which **employees** are in scope (only flexible-hours employees, or all)?

The existing Shifty v1.5 feature "HR-only Ø-Stunden/Woche-Statistik pro Person (urlaubsbereinigt, Regel A-22-1)" is the direct predecessor. AVG-01 extends or formalizes this into a proper reporting view.

Industry standard for flexible/variable-hour workers: most WFM tools compute a **per-week rolling average** over a measurement window (typically 4 weeks, 12 weeks, or a billing period), using actual worked/booked hours as the numerator, and excluding authorized leave from the denominator to avoid diluting the metric.

### Open Definitional Decision Points (Product-Owner Decisions)

These are not researchable — the product owner must decide. They are listed here to drive the discuss-phase.

**D-AVG-01: Reference period**
Options:
- A. Per billing period (most natural fit with existing Shifty billing structure)
- B. Per calendar month (common in HR reporting)
- C. Rolling N weeks (e.g., last 12 weeks)
- D. Per calendar year (too coarse for operational decisions)
*Recommendation: A (billing period) — aligns with existing `ReportingService` data already computed per billing period.*

**D-AVG-02: Numerator definition**
Options:
- A. Sum of booked hours in the period (hours from `Booking` records linked to slots in paid weeks)
- B. Count of days present
- C. Sum of actual clocked hours (Shifty has no clock-in; N/A unless using bookings as proxy)
*Recommendation: A — booked hours are already computed in Shifty; consistent with existing balance reporting.*

**D-AVG-03: Denominator — what counts as a "present week"**
Options:
- A. All calendar weeks in the period (vacation drags the average down — the problem AVG-01 solves against)
- B. All calendar weeks minus weeks where the employee had any vacation day
- C. All calendar weeks minus weeks where the employee had a full-week vacation
- D. Per-week denominator = (contracted expected hours) − (vacation hours that week); summed over period
*Recommendation: B — simplest to explain and implement; most natural match to "exclude weeks you were on vacation".*

**D-AVG-04: Which absence categories are excluded from the denominator**
Options:
- A. Vacation (Urlaub) only
- B. Vacation + sick leave
- C. Vacation + sick leave + public holidays
- D. All authorized absence types (Vacation, Sick, Holiday, Unpaid Leave)
*Recommendation: A (Vacation only) — this is the stated feature intent ("Urlaub aus dem Nenner gerechnet"). Sick leave, holidays, and unpaid leave are edge cases the product owner must decide; default to A unless stated otherwise. Including sick leave in exclusion has a "rewards absence" perception risk.*

**D-AVG-05: Employee scope**
Options:
- A. Only employees with `expected_hours = 0` (pure flexible / no fixed contract)
- B. Only employees with a "flexible hours" flag (if such a flag exists)
- C. All employees (fixed + flexible) — show avg for everyone, filter in UI
- D. All employees with `is_paid = true`
*Recommendation: C — show for all employees but allow filtering by contract type. Avoids a separate scope concept; flexible employees just have more meaningful numbers.*

**D-AVG-06: Display location**
Options:
- A. Inside the billing period report per employee (existing reporting view)
- B. New standalone "Attendance" view
- C. In the employee year view alongside existing balance
- D. In multiple of the above
*Recommendation: A — least new surface area; billing period report already aggregates per-employee per-period data in `ReportingService`.*

**D-AVG-07: Minimum data threshold**
How many non-vacation weeks are required before showing the average? (A zero-weeks denominator would be division-by-zero; a 1-week average is not meaningful.)
*Recommendation: Show average only when ≥ 2 non-vacation weeks present in the period. Show "—" or "n/a" otherwise.*

**D-AVG-08: Snapshot persistence**
If the average attendance result is added to the billing period snapshot (`BillingPeriodValueType`), the `CURRENT_SNAPSHOT_SCHEMA_VERSION` must be bumped. If it is computed read-only (derive-on-read), no bump is needed.
*Decision: Derive-on-read preferred for first implementation; avoids snapshot versioning complexity. Revisit if performance becomes an issue.*

### Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---|---|---|---|
| Average hours per week (vacation weeks excluded) per employee per period | Core stated feature; Shifty v1.5 already has precursor (Regel A-22-1) | MEDIUM | `sum(booked_hours_in_non_vacation_weeks) / count(non_vacation_weeks_in_period)` |
| HR-only access gate | Attendance analytics are sensitive; existing Shifty convention is HR-gated | LOW | Reuse existing `PermissionService` HR role check |
| Display in billing period report | Natural home; avoids new surface | LOW | Add one new row/column to existing per-employee billing period table |
| i18n labels | Shifty convention | LOW | de/en/cs translation keys |
| Handle no-data case gracefully | Employee may have only vacation in the period | LOW | Return `null` / display "—" when ≤ 1 non-vacation week |

### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---|---|---|---|
| Side-by-side comparison: average vs. committed voluntary hours | Shows "promised" vs. "actual" attendance for flexible volunteers | MEDIUM | Committed voluntary is already in `EmployeeWorkDetails`; comparing gives gap visibility |
| Trend over multiple billing periods | Shows if a flexible employee is attending more or less over time | HIGH | Requires multi-period query; defer to later milestone |
| Configurable exclusion categories | Allow HR to choose whether sick leave is also excluded | MEDIUM | UI complexity high; hardcode for v2.1 per decision D-AVG-04 |

### Anti-Features

| Feature | Why Requested | Why Problematic | Alternative |
|---|---|---|---|
| Show average attendance for all employees regardless of contract | "Consistency" | Meaningless for fixed-hours employees whose expected hours are contractually defined | Filter/badge flexible employees; fixed employees already have balance hours metric |
| Count attendance as binary present/absent days | "Simpler" | Loses information for employees who book fewer hours some days | Use booked hours as numerator (more accurate for shift workers) |
| Include sick leave weeks in exclusion by default | "Fairness — employee can't control illness" | Creates a "bonus" effect that hides patterns; HR should see sick weeks in the denominator by default | Offer as a toggle (future enhancement); default to vacation-only exclusion per D-AVG-04 |
| Persist as new BillingPeriodValueType in snapshot | "Consistency with other report metrics" | Triggers mandatory snapshot-schema-version bump + backward-compat concerns | Derive-on-read for v2.1; add to snapshot if needed later |

### Computation Sketch (for requirements/planning reference)

```
For each employee E in billing period P:

  weeks_in_period = all ISO calendar weeks overlapping P

  for each week W in weeks_in_period:
    vacation_hours_W = sum of AbsencePeriod hours of type Vacation
                       that overlap week W for employee E
    booked_hours_W   = sum of Booking hours for employee E in week W

  non_vacation_weeks = { W : vacation_hours_W == 0 }
  // (or: < full_week_threshold — per D-AVG-03 decision B)

  avg_attendance = sum(booked_hours_W for W in non_vacation_weeks)
                  / count(non_vacation_weeks)
                  when count(non_vacation_weeks) >= 2, else null
```

**Dependencies on existing Shifty services:**
- `AbsencePeriod` data (already in DAO layer) — to identify vacation weeks
- `Booking` data per employee per week (already in `BookingInformationService`)
- Billing period date range (already in `ReportingService`)
- `ReportingService` (Business-Logic tier) — natural home for new computation
- Snapshot schema version: no bump if derive-on-read (see D-AVG-08)

### Complexity Summary

| Area | Estimate | Notes |
|---|---|---|
| Data model | NONE | No new tables if derive-on-read |
| Business logic | MEDIUM | New computation in `ReportingService`; requires joining booking + absence data per week |
| REST | LOW | New field on existing billing-period-report response DTO |
| Frontend display | LOW | New cell in existing billing-period-per-employee table |
| Definitional decisions | HIGH (discussion) | Seven open decision points (D-AVG-01 through D-AVG-08) must be resolved in discuss-phase |

---

## Feature Dependencies (on Existing Shifty Capabilities)

```
WST-01 week status
    └── requires ──> Booking write paths (ShiftplanEditService, BookingService)
    └── requires ──> Slot write paths (SlotService, ShiftplanEditService)
    └── requires ──> Shiftplanner RBAC role (exists)
    └── requires ──> Week-navigation frontend component (exists)

AVG-01 average attendance
    └── requires ──> AbsencePeriod DAO (exists, v1.0+)
    └── requires ──> Booking data per employee per week (exists, BookingInformationService)
    └── requires ──> Billing period date ranges (exists, ReportingService)
    └── enhances ──> Billing period report frontend (existing view, add new column)
    └── relates to ──> v1.5 Regel A-22-1 "Ø-Stunden/Woche-Statistik" (precursor; check for reuse)
```

---

## MVP Recommendation for v2.1

### WST-01 — Build in full (low risk, clear scope)
- Four states with simple badge
- Planner-only state transitions
- Lock gate on all booking/slot write paths for non-planner roles
- i18n de/en/cs

### AVG-01 — Resolve decisions first, then implement minimum viable version
- Resolve D-AVG-01 through D-AVG-08 in discuss-phase
- Implement derive-on-read computation in `ReportingService`
- Display in billing period report, HR-gated
- Vacation-only exclusion (D-AVG-04 option A)
- Defer: trend view, configurable exclusion, snapshot persistence

### Defer (not v2.1)
- Publish-notification system for week status changes (employees notified when week moves to Planned)
- Week-status bulk operations (lock all past weeks at once)
- AVG-01 snapshot persistence
- Side-by-side avg vs. committed hours trend

---

## Competitor Feature Analysis

| Feature | Planday | When I Work | Dayforce | Shifty v2.1 approach |
|---|---|---|---|---|
| Week/schedule states | Draft → Published → (Payroll Locked) | Draft → Published | Timesheet lock per pay period | Four states + explicit planner lock |
| Who owns the lock | Admin only | Manager | Payroll admin | Shiftplanner (existing role) |
| Retroactive planner edit after lock | No (requires support) | N/A | Via retroactive adjustment flow | Yes, Shiftplanner can always write |
| Average attendance metric | Not found in documentation | Not found | Via workforce analytics module | New in v2.1, HR-gated |
| Vacation excluded from avg denominator | Partial (PTO subtracted from utilization) | N/A | Configurable | Vacation weeks excluded from denominator |

---

## Sources

- [Planday Draft Shifts documentation](https://help.planday.com/en/articles/30569-how-to-use-draft-shifts-in-planday) — schedule lifecycle states
- [Planday Payroll Period Lock](https://help.planday.com/en/articles/30525-how-to-lock-a-payroll-period) — lock semantics, admin-only, visual lock icon
- [When I Work Scheduling Basics](https://help.wheniwork.com/articles/scheduling-basics/) — draft vs. published states, diagonal-line visual for drafts
- [Shiftboard Permission Levels](https://support.shiftboard.com/l/en/article/af94aqf3yk-permission-levels-overview) — coordinator/manager draft vs. publish permissions
- [Dayforce Retroactive Adjustments](https://help.dayforce.com/r/manager-guide/Retroactive-Adjustments) — locked period retroactive edit access patterns
- [Oyster HR — Absence Rate](https://www.oysterhr.com/glossary/absence-rate) — numerator/denominator definitions for attendance metrics
- [Patriot Software — Absenteeism Rate](https://www.patriotsoftware.com/blog/payroll/how-to-calculate-absenteeism-rate/) — vacation exclusion from authorized leave calculations
- [BASUSA — Variable Hour Employee Measurement Periods](https://www.basusa.com/blog/measurement-period-best-practices-for-variable-hour-employees) — 3–12 month measurement windows, ACA compliance lens
- [Hubstaff — Utilization Rate](https://hubstaff.com/workforce-analytics/utilization-rate) — utilization = worked hours / available hours; vacation subtracted from denominator
- [Zendesk — Workforce Management Metrics](https://www.zendesk.com/blog/workforce-management-metrics/) — standard WFM metrics catalogue

---

*Feature research for: Shifty v2.1 — WST-01 and AVG-01*
*Researched: 2026-07-01*
