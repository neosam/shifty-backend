## Context

The weekly report aggregation in `service_impl/src/reporting.rs` currently sums up all booked shiftplan hours per week and feeds them into the balance formula

```
balance = shiftplan_hours + extra_work_hours − expected_hours + carryover
```

without any per-person cap. Hours beyond `expected_hours` automatically become positive balance (overtime credit). This works well for ordinary staff, but excludes a class of contributors that the proposal addresses: people with a small genuine contract who additionally help on a voluntary basis and do **not** want the extra time credited as overtime.

The cap mechanism must therefore live exactly where this aggregation happens. The relevant function is `hours_per_week()` (Z. 760+ in `reporting.rs`), which already has access to the three inputs the cap depends on: weekly shiftplan hours, the active `EmployeeWorkDetails` for that week, and `expected_hours` for that week.

`expected_hours` is not a property of `SalesPerson`. It is stored on `EmployeeWorkDetails`, which is time-versioned via `from_year/from_calendar_week … to_year/to_calendar_week`. A person can have multiple records over their lifetime in the system. Whatever flag controls the cap must therefore live on the same time-versioned record so the cap can switch on or off across time without touching history.

The existing `ExtraHoursCategoryEntity` enum (`dao/src/extra_hours.rs:9–17`) holds the categories `ExtraWork`, `Vacation`, `SickLeave`, `Holiday`, `Unavailable`, `UnpaidLeave`, `Custom(Uuid)`. Its variants are mapped to one of three `ReportType` values via `as_report_type()` (`service/src/extra_hours.rs:50–70`):

- `WorkingHours` — counts toward `expected` (planned attendance)
- `ExtraWork` — adds to balance positively (overtime)
- `AbsenceHours` — reduces `expected` (vacation, unpaid leave, sick leave)

None of these match the semantics required for "documented but balance-neutral". Volunteer work is *attendance*, not absence, but it must not move the balance. A new `ReportType` variant is therefore necessary.

## Goals / Non-Goals

**Goals:**

- Cap shiftplan hours at `expected_hours` per week, on a per-`EmployeeWorkDetails` opt-in basis.
- Introduce a `VolunteerWork` `ExtraHoursCategory` that is documented but balance-neutral, usable both as the auto-attribution target of the cap and as a manually entered record.
- Preserve all existing behaviour for records where the new flag is `false` (default).
- Keep the cap logic localised: a single mutation point in the reporting service, no scatter across DAO/REST layers beyond the obvious enum/field plumbing.
- Make the volunteer-hour figure visible at the same granularity as other category aggregates (per week and aggregated per period).

**Non-Goals:**

- Automatic compensation across weeks or billing periods. If a capped person plans below `expected` in a week, the resulting negative balance is intentional (per proposal) and is corrected, if at all, by a human via an `ExtraWork` entry.
- Any change to how `ExtraHours` records are summed (they are never capped).
- Year-end carryover semantics specific to volunteer hours. Volunteer hours are documentation only; they do not roll over.
- Frontend changes (Dioxus toggle, report column, manual-entry form). These are tracked in `shifty-dioxus` once the backend lands.
- User-facing localisation labels for the new `VolunteerWork` category. The backend has no locale files (verified: no i18n directory, no translation crate in `Cargo.toml`); category labels are owned by the `shifty-dioxus` frontend and will be added in the same follow-up that adds the UI controls.
- A privilege-level distinction for who may set the cap flag. The flag is part of `EmployeeWorkDetails` and reuses whatever permissions already guard that record.

## Decisions

### 1. Cap flag lives on `EmployeeWorkDetails`

Add `cap_planned_hours_to_expected: bool` (DB column `INTEGER NOT NULL DEFAULT 0`) to `EmployeeWorkDetails`.

**Why on `EmployeeWorkDetails` and not on `SalesPerson`:** `expected_hours` itself lives here, the records are already time-versioned, and the cap is conceptually a property of the *contract instance* — not the person. A person whose contract is later upgraded to full pay can simply have a new `EmployeeWorkDetails` record without the flag, and historical weeks remain calculated correctly against the contract that was active then.

**Alternative considered:** A boolean on `SalesPerson`. Rejected because it would either retroactively change historical weekly balances (if the cap is computed live) or require freezing/snapshotting balances at flag-flip time. Both are worse than letting the existing time-versioning carry the flag.

### 2. New `ExtraHoursCategoryEntity` variant `VolunteerWork`

Add a parameterless variant to the enum. SQLite mapping uses the literal string `"VolunteerWork"` in the existing `category` TEXT column, matching the convention already used for `ExtraWork`, `Vacation`, `SickLeave`, `Holiday`, `Unavailable`, `UnpaidLeave`.

**Why match the existing string convention:** Pattern matches in `dao_impl_sqlite/src/extra_hours.rs:34–44, 160–161` already enumerate variants by string. Following the convention keeps the change additive — one extra arm in each match.

**Alternative considered:** Reusing `Custom(Uuid)` with a designated UUID. Rejected: `Custom` exists to let users define their own categories at runtime; co-opting it for a system-defined balance-neutral category would conflate two separate concepts.

### 3. New `ReportType::Documented` variant

Add a third sibling to `WorkingHours`, `ExtraWork`, `AbsenceHours`: `Documented`. Hours classified as `Documented` are recorded in the report output but contribute nothing to the balance formula.

**Why a new variant:** None of the existing three is balance-neutral. Trying to express volunteer hours as `AbsenceHours` would actively mislead the calculation (it would *reduce* `expected_hours`, awarding the person credit for being "absent" while they were actually present and working).

**Naming — why `Documented`:** Short, precise, and parallel in form to the other variants (single descriptive word). Considered `Recorded` (too generic), `BalanceNeutral` (describes the side-effect, not the meaning), `Informational` (suggests metadata rather than work). `Documented` reads naturally as "this happened and is on record, but it doesn't move balance".

`ExtraHoursCategoryEntity::VolunteerWork` maps to `ReportType::Documented`. `Availability` for the new variant is `Available` (the person is in fact present).

### 4. Cap logic placement: a single helper in `hours_per_week()`

Inside `hours_per_week()` in `service_impl/src/reporting.rs`, immediately after `shiftplan_hours` and `working_hours_for_week` are determined for the active week, apply:

```
if work_details_for_week.cap_planned_hours_to_expected
    && shiftplan_hours > expected_hours_for_week:
        auto_volunteer_hours = shiftplan_hours − expected_hours_for_week
        shiftplan_hours      = expected_hours_for_week
else:
        auto_volunteer_hours = 0
```

The resulting `auto_volunteer_hours` are added to whatever `VolunteerWork` extra-hour records exist for the same week and exposed under a single `volunteer_hours` figure in the per-week output.

**Why in `hours_per_week()` and not deeper (DAO) or shallower (REST):** It is the single point that both has access to the booking total *and* knows which `EmployeeWorkDetails` is active for the week. The DAO layer has no concept of `expected_hours`. The REST layer is downstream of the aggregation and would have to redo the per-week math. Putting the rule here means there is exactly one place where the cap is enforced, and it is the same place where the balance is composed.

**Alternative considered:** Pre-cap the bookings before `hours_per_week()` ever sees them. Rejected because the cap is per-week, but bookings are per-slot and per-day; pre-capping would require re-introducing weekly bucketing in a layer that doesn't otherwise need it. Aggregating first and capping second is simpler.

### 5. Hybrid sourcing of volunteer hours: auto + manual, single output

Volunteer hours can originate from two sources:

- **Auto-attributed:** computed by the cap logic above. *Not* persisted as an `ExtraHours` record.
- **Manually entered:** an `ExtraHoursCategoryEntity::VolunteerWork` record created via the existing extra-hours flow.

The report sums the two and exposes a single `volunteer_hours` figure per week (and aggregated per period). Consumers do not distinguish them.

**Why not persist the auto-attributed hours as synthetic `ExtraHours` rows:** It would create a hidden coupling between bookings and extra-hours records — every booking edit would have to recompute and rewrite synthetic rows, and the historical accuracy of synthetic rows would depend on whether bookings or the cap flag changed since. Computing on read keeps the bookings table the single source of truth for what was actually planned.

**Why not split the output into "auto" and "manual" separately:** No downstream consumer (planner UI, billing report) has been identified that needs the distinction. The proposal frames volunteer hours as a single concept. The split can be added later without breaking the field if a use case appears.

### 6. Manual `VolunteerWork` entries are allowed regardless of cap flag

A shift planner may enter a `VolunteerWork` extra-hours record for any sales person, including ones whose `EmployeeWorkDetails` does not have the cap flag set.

**Why:** The flag controls *automatic* attribution from over-cap bookings. It does not own the category. People with a regular contract may still occasionally do volunteer work (e.g. helping out at an event); forbidding manual entry would force the planner to either flip the flag temporarily or pick a misleading category.

**Alternative considered:** Validation that rejects `VolunteerWork` records when the person's currently active `EmployeeWorkDetails` does not have the cap flag. Rejected as needless coupling between two orthogonal concepts; the simpler rule (anyone can be marked as having volunteered) is also the more honest one.

### 7. Volunteer hours appear as a dedicated field in report TOs, not under the generic extra-hours list

Existing report TOs already break out specific categories (`vacation_hours`, `sick_leave_hours`, `unpaid_leave_hours`). Volunteer hours follow the same pattern: a dedicated `volunteer_hours` field on `ShortEmployeeReportTO`, `EmployeeReportTO`, `WorkingHoursReportTO`, and `GroupedReportHours`.

Volunteer entries also appear in the per-category extra-hours list as their own variant (`ExtraHoursReportCategoryTO::VolunteerWork`), consistent with how every other category is exposed.

**Why both:** Separate field for at-a-glance reading and balance-clarity in summaries; categorised list for callers that iterate over all categories generically.

### 8. YTD semantics for the new `"volunteer"` value_type in billing-period snapshots

`BillingPeriodSalesPersonEntity` carries four numeric fields for each `value_type` row: `value_delta`, `value_ytd_from`, `value_ytd_to`, `value_full_year`. For the new `"volunteer"` value_type these fields SHALL be populated using the same accumulation rule as the other value_types written by `build_and_persist_billing_period_report()`:

- `value_delta` — the volunteer hours accumulated within the billing period (cap-attributed plus manual `VolunteerWork` extra-hours records)
- `value_ytd_from` — the volunteer hours accumulated from the start of the same calendar year up to the day before the billing period starts
- `value_ytd_to` — the volunteer hours accumulated from the start of the same calendar year through the last day of the billing period (equals `value_ytd_from + value_delta`)
- `value_full_year` — the volunteer hours accumulated over the full calendar year that contains the billing period, frozen at snapshot creation time

**Why mirror the existing convention rather than zero out YTD fields:** Volunteer hours are conceptually periodic (they do not affect balance, do not roll over year-end), so the YTD figures have no consumer at this point. However, mirroring the established convention costs nothing extra at write time (the same aggregation infrastructure already produces these figures for every other value_type) and avoids a special-case write path in `build_and_persist_billing_period_report`. Future readers that wish to display "volunteer hours so far this year" get the data for free; readers that only care about the period delta simply ignore the YTD fields, as they do for every other value_type.

**Alternative considered:** Set `value_ytd_from`, `value_ytd_to`, and `value_full_year` all equal to `value_delta` (a "no YTD concept" marker) or to `0.0`. Rejected: a consumer reading these fields generically (e.g. a future report renderer that iterates value_types) would either crash on the inconsistency or display misleading numbers. The cost of populating them honestly is zero.

## Risks / Trade-offs

- **[Negative balance on capped persons who underplan a week]** → Documented in proposal as intentional; corrected manually via `ExtraWork`. Frontend will need to surface this clearly to avoid surprise.
- **[New `ReportType` variant requires touching every existing match arm]** → The alternative (overloading an existing variant) is worse. The compiler will catch missed arms. Acceptable cost.
- **[SQLx compile-time checks fail on stale local DB]** → Standard project workflow already requires running migrations before `cargo build`. Documented in `shifty-backend/CLAUDE.md`.
- **[Auto-attributed volunteer hours are computed on every report read]** → The math is trivially cheap (one comparison, one subtraction per week). No caching needed.
- **[Manual `VolunteerWork` entry on non-capped person can confuse]** → Surface friendly hint in frontend if and when the situation arises; backend stays permissive.
- **[Edge case: cap flag flips mid-period]** → Each week is evaluated against the `EmployeeWorkDetails` record active for that week (existing logic via `weight_for_week()`). A flip therefore affects only weeks from the new record onward, which is the desired behaviour and matches how `expected_hours` itself behaves on contract changes.
- **[Edge case: no `EmployeeWorkDetails` active for a week]** → Pre-existing condition unaffected by this change. Hours fall through with `expected = 0`, no cap applies. Documented for completeness.

## Migration Plan

1. **Schema migration:** add `cap_planned_hours_to_expected INTEGER NOT NULL DEFAULT 0` to `employee_work_details`. Existing rows automatically receive `0` (cap off). No data migration needed.
2. **Enum extension:** extend `ExtraHoursCategoryEntity` with `VolunteerWork`. Update SQLite mapping arms (read + write paths). Update `as_report_type()` and `availability()` mappings.
3. **`ReportType` extension:** add `Documented` variant. Compiler-driven sweep of all match arms across `service`, `service_impl`, `rest`, `rest-types`. Each arm decides explicitly whether `Documented` contributes (it never does to balance; it does contribute to the dedicated `volunteer_hours` aggregate).
4. **Reporting logic:** wire the cap helper into `hours_per_week()`. Sum auto-attributed + manual volunteer hours into the per-week and per-period `volunteer_hours` figures.
5. **TO updates:** add `volunteer_hours` field where `vacation_hours` and `sick_leave_hours` already exist; add `VolunteerWork` to the category-list TO variant; regenerate OpenAPI.
6. **i18n:** add labels for the new category in `En`, `De`, `Cs`.
7. **Tests:** integration tests covering (a) capped person with bookings = expected, (b) capped person with bookings > expected, (c) capped person with bookings < expected (negative balance), (d) capped person with manual `VolunteerWork` entry, (e) non-capped person with manual `VolunteerWork` entry, (f) cap flag flipping between two consecutive `EmployeeWorkDetails` records.
8. **Rollback:** the schema change can be reverted with a follow-up migration that drops the column. Existing volunteer-hour `ExtraHours` rows would have to be either deleted or re-categorised; document this as a one-way door for production data once the feature is live.

## Open Questions

None. All open questions from the proposal have been resolved in the Decisions section above (variant name → `Documented` in §3, filter behaviour → dedicated field plus categorised list in §7, validation on manual entry without cap flag → permissive in §6).
