# Feature Research — Committed Voluntary Capacity (v1.4)

**Domain:** HR / shift-planning reporting — pre-committed voluntary hours capacity
**Researched:** 2026-06-22
**Confidence:** HIGH (formula + side-effects verified directly against `reporting.rs` and `booking_information.rs`; line areas cited)

---

## TL;DR for the requirements step

1. **The no-double-count formula in the todo is stated against the WRONG data path.** The todo (req 3) describes `available = expected + committed_voluntary`, `surplus = max(0, actual_volunteer − committed_voluntary)` as if the year view reads `reporting.rs`. **It does not.** The Jahresansicht (`weekly_overview`) is fed by `booking_information.rs::get_weekly_summary` → `WeeklySummaryTO`, whose `volunteer_hours` is **already** the *actual booked hours of `is_paid=false` persons* (`booking_information.rs:141-153`), and `paid_hours` is the sum of paid persons' `dynamic_hours` (`:176-178`). The reactive cap-overflow `volunteer_hours` computed in `reporting.rs` **never reaches the year view at all.** This must be resolved before requirements are written. See [The two volunteer axes](#the-two-volunteer-axes-critical).
2. **Scope gate (`cap_planned_hours_to_expected = true`) is semantically clean but mechanically detached** from the year-view capacity number, because the year view keys voluntary capacity off `is_paid=false`, not off the cap flag. Pin down which gate actually drives display.
3. **Giving an unpaid volunteer an `EmployeeWorkDetails` record has concrete, enumerable side effects** — it flips them from the "volunteer" branch (`is_paid=false`) into multiple paid-only loops and the HR reporting list. See [Anti-Features](#anti-features-commonly-requested-often-problematic) and [unpaid-record side-effects](#unpaid-volunteer-record-side-effects-enumerated).
4. **Aggregation granularity is per-ISO-week, summed to year.** `committed_voluntary` is a flat weekly pledge from the time-versioned `EmployeeWorkDetails` record active for that week; it does **not** interact with absence/holiday/vacation. See [req 5 answer](#interaction-with-absence-vacation-flat-weekly-pledge).

---

## The two volunteer axes (CRITICAL)

There are **two completely separate "volunteer_hours" computations** in the backend. Conflating them is the single biggest risk in this milestone.

| Axis | Where | What it means | Reaches year view? |
|------|-------|---------------|--------------------|
| **A — reactive cap-overflow** (`auto_volunteer_hours`) | `reporting.rs::apply_weekly_cap` (`:94-107`), folded into `ShortEmployeeReport.volunteer_hours` (`:362-367`, `:854`, `:1121-1126`) | For a **paid** person with `cap_planned_hours_to_expected=true`: bookings above `expected` spill into volunteer instead of overtime. Plus manual `VolunteerWork` extra-hours. | **No** — consumed by `get_report_for_employee` / billing period, not by `get_weekly_summary`. |
| **B — unpaid-person booked hours** | `booking_information.rs::get_weekly_summary` (`:141-153`) and `:417-431` | Sum of `extract_shiftplan_report_for_week` hours for every sales person with `is_paid=false`. This is the number the Jahresansicht renders as `🤝 volunteer`. | **Yes** — this IS the year view's `volunteer_hours`. |

**Year-view capacity today** (`booking_information.rs:197`, `:309`):
```
overall_available_hours = paid_hours + volunteer_hours
                        = Σ(dynamic_hours of is_paid persons)   // contract capacity
                        + Σ(booked hours of !is_paid persons)   // reactive volunteer
```

So the year view's "available capacity" is **reactive on both axes**: paid capacity is contract-driven, volunteer capacity is *whatever unpaid people actually booked*. There is **no forward-looking pledge anywhere today** — which is exactly the gap v1.4 closes.

**Consequence for the formula:** The todo's `available = expected + committed_voluntary` and `surplus = max(0, actual_volunteer − committed_voluntary)` describes a *new* capacity axis that must be **added into `get_weekly_summary`'s `overall_available_hours`**, not retrofitted onto `reporting.rs`. The `reporting.rs` cap path (Axis A) and the year view (Axis B) should be treated as independent. Flag this for the requirements author as **D-FORMULA-PATH**.

---

## No-double-count formula — verified semantics

The intended forward-looking capacity model, expressed so it slots into `get_weekly_summary`:

```
committed_voluntary  := pledged weekly voluntary hours (new field on EmployeeWorkDetails, time-versioned)
actual_volunteer     := booked hours this person already has (Axis B, per week)

available_capacity   = expected_paid + committed_voluntary            // the pledge counts as available capacity
surplus              = max(0, actual_volunteer − committed_voluntary) // only bookings BEYOND the pledge add on top
counted_volunteer    = committed_voluntary + surplus
                     = max(committed_voluntary, actual_volunteer)
```

The closed form is **`max(committed_voluntary, actual_volunteer)`** — the pledge is a floor, actual bookings only matter once they exceed it. This is the no-double-count rule.

**Worked examples (from todo req 3, verified consistent):**

| committed | actual | counted_volunteer | display | interpretation |
|-----------|--------|-------------------|---------|----------------|
| 5 | 3 | `max(5,3)=5` | **5** (covered) | pledge not yet fulfilled; capacity still shown at pledge level, no surplus |
| 5 | 7 | `max(5,7)=7` → 5 + 2 | **5 + 2 surplus** | pledge fulfilled, 2h booked beyond pledge counts on top |
| 5 | 0 | 5 | **5** | pure forward pledge, nothing booked yet |
| 0 | 4 | 4 | **4** | no pledge → identical to today's reactive behaviour |

The `committed=0` row is important: it makes the new field **backward-compatible** — a person with no pledge behaves exactly as today (`counted = actual`).

### Ambiguity flagged: weekly vs period aggregation, partial weeks

- **Granularity is per-ISO-week.** `get_weekly_summary` already loops `for week in 1..=(weeks_in_year + 3)` (`booking_information.rs:126`) and the year view renders one row per week. `committed_voluntary` is read from the `EmployeeWorkDetails` record active for *that* week (same `from_year/from_calendar_week … to_year/to_calendar_week` versioning as `cap_planned_hours_to_expected`, per the cap spec). So the `max()` must be applied **per week, then summed** — never `max(Σcommitted, Σactual)` over the year, which would hide a week where the pledge was unmet behind another week of surplus.
- **Partial weeks / mid-week version boundaries:** `EmployeeWorkDetails` is week-granular for the cap flag (the spec treats a record as active for whole weeks). `committed_voluntary` should follow the **same week-granular activation** as `cap_planned_hours_to_expected` — do NOT pro-rate the pledge by `weight_for_week` the way `expected_hours` is pro-rated (`reporting.rs:240-254`). A pledge is a flat weekly number; pro-rating it would silently shrink it on a person's first/last partial week. **Flag D-PARTIAL-WEEK:** confirm pledge is flat-per-active-week, not weighted.
- **`weeks_in_year + 3` overscan:** `get_weekly_summary` deliberately computes 3 weeks into the next year (`:117`, `:126-131`). The committed pledge lookup must tolerate week>weeks_in_year roll-over identically.

---

## Feature Landscape

### Table Stakes (Users Expect These)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `committed_voluntary: f32` on `EmployeeWorkDetails`, time-versioned | The whole point of v1.4 (D-01 / Variante B) | LOW (data model) | Mirror `cap_planned_hours_to_expected` exactly: nullable→default 0.0 migration, same versioning range, Service+DAO+rest-types. `EmployeeWorkDetails` struct at `service/src/employee_work_details.rs:14-41`. |
| Pledge counts into year-view available capacity, per week | "Available capacity" must reflect committed hours, else feature is invisible | MEDIUM | Add `committed_voluntary` term into `overall_available_hours` in `booking_information.rs:197` and `:309`. This is the real integration point — NOT `reporting.rs`. |
| No double-count: `counted = max(committed, actual)` per week | Explicit todo req 3; prevents inflated capacity | MEDIUM | The `max()` must be evaluated per person per week before summing into `volunteer_hours`. |
| Committed capacity shown **separately** from paid & volunteer | Todo req 4 — must not be "vermischt" with `paid`/`volunteer` | LOW–MED (frontend) | Year-view row today shows `💰{paid} \| 🤝{volunteer}` (`weekly_overview.rs:103`, `:108`). Add a third token (e.g. `📌{committed}` / "zugesagt"). New field on `WeeklySummaryTO` + `WeeklySummary` state + i18n key in all 3 locales. |
| Backward-compat: `committed=0` ⇒ unchanged behaviour | Existing data must not shift | LOW | `max(0, actual)=actual`; guaranteed by formula. Default-0 migration (mirror cap-flag migration scenario in spec). |
| Snapshot schema version bump | CLAUDE.md mandate — volunteer/capacity computation input changes | LOW | `CURRENT_SNAPSHOT_SCHEMA_VERSION` is currently **7** (`billing_period_report.rs:74`). Bump to 8 **only if** the committed pledge changes a persisted `billing_period_sales_person` `value_type`. **Flag:** if v1.4 only touches the live year-view (`get_weekly_summary`) and NOT the billing snapshot writer, a bump may be unnecessary — verify whether `volunteer_hours` is a persisted snapshot value before bumping. |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Surplus highlighting (committed met + extra booked) | Planner instantly sees who over-delivered vs who only pledged | LOW | `surplus = actual − committed` already computed; render `5 + 2` like the example. Reuse `diff_color_and_sign` token logic (`weekly_overview.rs:22-30`). |
| Pledge-vs-delivered gap warning (committed but under-booked) | Surfaces unmet pledges early (committed=5, actual=2 ⇒ 3h gap to fill) | MEDIUM | Per project memory, prefer **inline banner, not blocking dialog**. Could reuse the existing absences sub-row pattern (`weekly_overview.rs:115-125`). |
| Chart inclusion of committed capacity | `WeeklyOverviewChart` consumes the same `weeks` data | MEDIUM | If committed feeds `available_hours`, chart updates for free; if shown as a separate band, chart needs a new series. Confirm with design. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Reuse `reporting.rs` cap-overflow path for the year-view number | "It already computes volunteer_hours" | That path (Axis A) is consumed by billing/employee-report, NOT the year view; wiring it in would double-count against Axis B's `is_paid=false` booked hours | Integrate the pledge in `booking_information.rs::get_weekly_summary` (Axis B) only |
| Give every unpaid volunteer a full paid `EmployeeWorkDetails` with `expected_hours>0` | "They need a work-details record to hold the pledge" | A paid-style record flips them into paid-only code paths: they leave the `is_paid=false` volunteer sum, enter HR reports, get contract paid_hours, get day-distributed capacity. Mass behaviour shift. | Create the record with `expected_hours=0` and keep `is_paid=false`; only `committed_voluntary>0`. Verify each downstream loop tolerates `expected_hours=0`. |
| Invariant `committed_voluntary >= expected_hours` | "Pledge should be the total" | D-01 explicitly rejected Variante A (committed_total). Adds a subtraction step and a brittle invariant. | Keep `committed_voluntary` as the *additive top-up only* (Variante B). No invariant. |
| Pro-rate the pledge by partial-week weight | Symmetry with `expected_hours` weighting | Silently shrinks the pledge on first/last partial weeks; a "5h pledge" becomes "2h" | Flat weekly pledge, week-granular activation like the cap flag |
| Make committed interact with absence/vacation (reduce pledge when on holiday) | "If they're on vacation they can't volunteer" | Adds cross-entity coupling the todo doesn't ask for; absence already handled on the paid/derived axis | Keep pledge a flat capacity number; let actual bookings (Axis B) naturally drop to 0 during absence |

---

## Scope gating (req 2) — answer

**Intended scope:** feature applies ONLY to `cap_planned_hours_to_expected = true` persons (PROJECT.md "Scope-Grenze", todo req 2).

**Reality check — two different gates exist and they don't coincide:**

- The **cap flag** (`cap_planned_hours_to_expected`) lives on `EmployeeWorkDetails` and gates Axis A (`reporting.rs:264-267`, `:813-814`, `:1000-1003`).
- The **year-view volunteer axis** (Axis B) gates on **`is_paid=false`** (`booking_information.rs:123`, `:248`), a `SalesPerson` field (`sales_person.rs:17`), *not* on the cap flag.

A "small paid contract + voluntary top-up" person (the todo's motivating example: 5h paid + 5h pledged) is `is_paid=true` AND `cap_planned_hours_to_expected=true`. But today such a person contributes to `paid_hours` (Axis A), **not** to the year-view `volunteer_hours` (Axis B, which is `is_paid=false` only). So their pledge has nowhere to land in the current year-view math.

**Behaviour to pin down (flag D-SCOPE-GATE):**
- For a **capped paid person** (`is_paid=true`, `cap=true`, `committed_voluntary=5`): the pledge should add 5 to `overall_available_hours` for that week, shown in the separate committed column. Their booked overflow beyond `expected` is Axis A volunteer in `reporting.rs` but does NOT reach the year view — so the year view needs the committed term added explicitly in `booking_information.rs`.
- For a **non-capped person** (`cap=false`): `committed_voluntary` is **ignored / not displayed**. The pledge field may physically exist on the record but the year-view integration reads it only when `cap_planned_hours_to_expected=true`. Recommended display: blank/`—` in the committed column, never `0` (so "no pledge" ≠ "pledged zero" is visually distinct). Confirm with design.
- For a **pure unpaid volunteer** (`is_paid=false`): handled by req 4 below — needs a record to hold the pledge.

---

## Year-view display (req 3/4) — table-stakes vs nice-to-have

Current row markup (`weekly_overview.rs:90-128`): three columns — Week, `💰paid \| 🤝volunteer`, `available/required`, `missing`. Plus an optional absences sub-row (`:115-125`).

**Table stakes (must ship):**
- New `committed_voluntary` field threaded: `WeeklySummaryTO` (`rest-types/src/lib.rs:901-915`) → `WeeklySummary` state (`state/weekly_overview.rs:11-27` + `From` impl `:29-62`) → render.
- Committed shown **separately** (third token, e.g. `📌{committed}` / a "zugesagt" / "committed" column), per todo req 4. New i18n key in En/De/Cs.
- `available_hours` (= `overall_available_hours`) includes the committed term so the `available/required` column and the `missing` diff (`weekly_overview.rs:87`) reflect the pledge.

**Nice-to-have (defer if needed):**
- Surplus rendering (`committed + surplus`, e.g. `5 + 2`).
- Pledge-unmet inline banner (committed > actual).
- Per-day distribution of the pledge (`monday_available_hours…`) — today only the day-level `get_summery_for_week` distributes (`:434-440`); the year view leaves day fields `0.0` (`:206-212`). A flat weekly pledge has no natural day split, so **omit from day columns** unless design wants even distribution.
- Chart band for committed.

---

## Unpaid-volunteer record side-effects (enumerated)

Req 4: pure unpaid volunteers need an `EmployeeWorkDetails` record to hold `committed_voluntary`, and must become visible/selectable via the "alle" filter.

**Today, an `is_paid=false` person with NO work-details record** is invisible to every paid loop and visible only as Axis-B booked hours. Giving them a record changes the following — **each must be verified to remain correct**:

| # | Downstream consumer | Today (no record) | After adding a record | Risk |
|---|---------------------|-------------------|------------------------|------|
| 1 | `reporting.rs::get_reports_for_all_employees` (HR list) | excluded — filters `is_paid==true` (`:141`) | **still excluded** if `is_paid` stays false | LOW — gate is `is_paid`, not record presence |
| 2 | `booking_information.rs` volunteer sum (Axis B) | included via `is_paid=false` (`:123`) | still included **iff** `is_paid` stays false | MED — if a record is mistakenly created with `is_paid=true`, they LEAVE the volunteer sum and ENTER paid_hours |
| 3 | `booking_information.rs` paid day-distribution loop (`:341-408`) | excluded — filters `is_paid=true` (`:326`) | excluded if `is_paid` stays false | LOW |
| 4 | `reporting.rs::get_week` (per-week report) | iterates over **everyone with work-details** (`:719`), keyed by work-details map, NOT `is_paid` | **now included** — a new `ShortEmployeeReport` row appears for this person | **HIGH — this is the main surprise.** `get_week` is called by `get_weekly_summary` (`:133`), so the new record makes the unpaid volunteer appear in `working_hours_per_sales_person` / absence sub-rows even though they were never there before |
| 5 | `paid_hours` accumulation (`booking_information.rs:176-178`) sums `report.dynamic_hours` over ALL `get_week` rows | the volunteer had no row | new row with `dynamic_hours` from `expected_hours` | **HIGH if `expected_hours>0`** — their contract hours would leak into `paid_hours`. **Mitigation: create record with `expected_hours=0`** so `dynamic_hours≈0`. |
| 6 | Billing period snapshot | excluded (no work-details / `is_paid=false`) | depends on whether billing keys off `is_paid` or work-details — **verify** | MED — possible snapshot drift ⇒ relates to the version-bump decision |

**Net recommendation for requirements:** the unpaid-volunteer record must be created with **`is_paid=false` preserved** and **`expected_hours=0`**, carrying only `committed_voluntary>0` (and likely `cap_planned_hours_to_expected=true` so the scope gate lets the pledge through). Then verify side-effect #4/#5: that a zero-expected record produces a benign `get_week` row (dynamic_hours 0, no spurious paid_hours, no absence noise). This is the highest-risk integration test in the milestone.

---

## Interaction with absence/vacation (req 5) — flat weekly pledge

**Answer: `committed_voluntary` does NOT interact with absence periods, holidays, or vacation. It is a flat weekly pledge.**

Rationale, verified:
- Absence/holiday/vacation reduce the **paid expected** axis via `expected = planned − absence − derived` (`reporting.rs:429-430`, `:851`, `:1089`) and the dynamic guard `if planned_hours <= 0.0 { 0.0 }` (`:845-850`).
- The pledge is a *capacity offer*, not contracted hours, so there is no expected/balance to erode. If the person is absent, their **actual** bookings (Axis B) naturally fall, and `counted = max(committed, actual)` keeps the pledge visible as offered-but-unfilled capacity — which is the correct planner signal.
- Adding absence interaction would re-introduce the cross-entity coupling D-01 was designed to avoid and pull `AbsenceService` into the pledge path (a Business-Logic dependency the cap flag deliberately avoids).

**One edge to flag (D-ABSENCE-DISPLAY):** if design later wants "don't show pledged capacity for a week the person is fully on holiday," that's a *display filter*, not a math change — keep it out of the core formula.

---

## Feature Dependencies

```
committed_voluntary field (EmployeeWorkDetails: Service + DAO + rest-types + migration)
    └──requires──> SQLite migration (default 0.0, mirror cap-flag migration)
    └──enables──> year-view capacity integration (booking_information::get_weekly_summary)
                       └──requires──> WeeklySummaryTO new field
                                          └──requires──> WeeklySummary state + From impl
                                                             └──enables──> separate committed column + i18n (En/De/Cs)
    └──enables──> "alle"-filter + unpaid-volunteer record (is_paid=false, expected_hours=0)
                       └──conflicts──> reporting::get_week including the new row (side-effect #4)

snapshot version bump ──depends-on──> whether volunteer_hours is a PERSISTED billing value_type (verify first)
Axis A (reporting cap-overflow) ──independent-of──> Axis B (year-view) — DO NOT merge
```

### Dependency notes
- **Field before everything:** the migration + struct field is the foundation; mirror `cap_planned_hours_to_expected` end-to-end (it already proves the time-versioned-flag pattern).
- **`booking_information.rs` is the real integration site**, not `reporting.rs`. Get this wrong and capacity double-counts.
- **The unpaid-record path conflicts with `get_week`'s record-keyed iteration** — the dependency that needs an explicit integration test.

---

## MVP Definition

### Launch With (v1.4)
- [ ] `committed_voluntary: f32` on `EmployeeWorkDetails` — time-versioned, default 0.0 migration (Service/DAO/rest-types). *Foundation.*
- [ ] Year-view capacity integration in `booking_information.rs::get_weekly_summary` with per-week `counted = max(committed, actual)`, gated on `cap_planned_hours_to_expected`. *The feature.*
- [ ] Separate committed column in `weekly_overview` + i18n (En/De/Cs). *Visibility (todo req 4).*
- [ ] "alle"-filter + unpaid-volunteer record path with `is_paid=false`, `expected_hours=0`, verified side-effects #4/#5. *Todo req 5.*
- [ ] Snapshot version decision (bump iff `volunteer_hours` is a persisted billing value_type).

### Add After Validation (v1.5)
- [ ] Surplus display (`committed + surplus`) and pledge-unmet inline banner.
- [ ] Committed capacity in `WeeklyOverviewChart`.

### Future Consideration (v2+)
- [ ] Approval workflow for pledges (explicitly out per PROJECT.md SC-01).
- [ ] Min-paid-capacity / skill matching (SC-02).
- [ ] Average-attendance evaluation (related todo, deferred).

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| `committed_voluntary` field + migration | HIGH | LOW | P1 |
| Year-view capacity integration (`max` per week) | HIGH | MEDIUM | P1 |
| Separate committed column + i18n | HIGH | LOW | P1 |
| Unpaid-volunteer record + side-effect guards | HIGH | MEDIUM | P1 |
| Snapshot version bump (conditional) | MEDIUM | LOW | P1 |
| Surplus display / unmet-pledge banner | MEDIUM | LOW | P2 |
| Chart band for committed | LOW | MEDIUM | P3 |

## Open questions for requirements author (decision flags)

- **D-FORMULA-PATH:** Confirm integration is in `booking_information.rs::get_weekly_summary` (Axis B), NOT `reporting.rs` (Axis A). The todo's formula wording implies reporting.rs and is misleading.
- **D-SCOPE-GATE:** For capped **paid** persons (`is_paid=true`, `cap=true`), where does the pledge add — the `paid_hours` term, the `volunteer_hours` term, or a new third term? Recommend a new third term `committed_hours` on `WeeklySummary` so it is genuinely "separate" (todo req 4) and never double-counts against Axis B.
- **D-PARTIAL-WEEK:** Confirm flat-per-active-week (no `weight_for_week` pro-rating).
- **D-UNPAID-RECORD:** Confirm `is_paid=false` + `expected_hours=0` for unpaid-volunteer records, and add an integration test for the `get_week` side-effect (#4/#5).
- **D-SNAPSHOT:** Verify whether `volunteer_hours` / capacity is a persisted `billing_period_sales_person` value_type before bumping `CURRENT_SNAPSHOT_SCHEMA_VERSION` (currently 7).
- **D-ABSENCE-DISPLAY:** Confirm pledge is flat (no absence interaction); any "hide on full-holiday week" is display-only.

## Sources

- `service_impl/src/reporting.rs` — `apply_weekly_cap` (`:94-107`); all-employees fold (`:222-446`, `is_paid` filter `:141`); `get_week` (`:686-879`, record-keyed iteration `:719`); per-week detail (`:980-1129`). HIGH (read directly).
- `service_impl/src/booking_information.rs` — `get_weekly_summary` Axis B (`:95-218`, volunteer sum `:141-153`, paid_hours `:176-178`, `overall_available_hours` `:197`); day-level `get_summery_for_week` (`:220-481`, paid filter `:326`, volunteer-by-day `:417-440`). HIGH.
- `rest-types/src/lib.rs` — `WeeklySummaryTO` (`:901-915`), `From` (`:919-940`). HIGH.
- `shifty-dioxus/src/page/weekly_overview.rs` (`:90-128` row markup) and `src/state/weekly_overview.rs` (`:11-62` TO mapping). HIGH.
- `service/src/employee_work_details.rs` (`:14-41` struct, `:26` cap flag) ; `service/src/sales_person.rs` (`:17` `is_paid`). HIGH.
- `service_impl/src/billing_period_report.rs:74` — `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7`. HIGH.
- `openspec/specs/weekly-planned-hours-cap/spec.md` — cap flag semantics, time-versioning, migration default. HIGH.
- `.planning/todos/pending/2026-06-22-committed-voluntary-capacity-jahresansicht.md` — D-01 / Variante B, reqs 1–5, worked examples. HIGH.

---
*Feature research for: committed voluntary capacity (v1.4 subsequent milestone)*
*Researched: 2026-06-22*
