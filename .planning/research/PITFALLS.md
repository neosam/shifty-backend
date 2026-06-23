# Pitfalls Research

**Domain:** Time-versioned contract field + non-double-counting reporting input + snapshot-versioned billing computation (Shifty backend + Dioxus frontend) — v1.4 "committed voluntary capacity"
**Researched:** 2026-06-22
**Confidence:** HIGH (grounded in direct reads of `reporting.rs`, `booking_information.rs`, `billing_period_report.rs`, `employee_work_details.rs`, the v1.3 Phase-8.4 additive-merge lessons in STATE.md, and the snapshot-versioning rules in CLAUDE.md)

> Orchestrator note: this feature is *almost entirely* about not double-counting volunteer hours and not silently breaking snapshot validators / paid-employee assumptions. Three pitfalls (P1 double-count, P2 snapshot drift, P5 unpaid-volunteer record) are load-bearing; everything else is supporting. Read those three first.

---

## The two reporting surfaces you must reconcile (context for P1)

There are **two independent code paths** that already attribute volunteer hours, and they disagree about *who* a volunteer is:

1. **`ReportingService::get_week`** (`service_impl/src/reporting.rs:686-880`) iterates **everyone who has an `EmployeeWorkDetails` row for the week** (`all_for_week`, line 696-700 — NOT a paid filter). For each it computes `volunteer_hours = manual_volunteer_hours + auto_volunteer_hours` (line 854), where `auto_volunteer_hours` is the *cap overflow* from `apply_weekly_cap` (line 94-107). This is **reactive** volunteer hours — overflow above expected, only when `cap_planned_hours_to_expected` is set.

2. **`BookingInformationService::get_summery_for_week` / year** (`service_impl/src/booking_information.rs:118-218, 220-459`) computes its OWN `volunteer_hours` = sum of **shiftplan hours of `!is_paid` sales persons** (lines 118-125, 141-153, 243-267). It then sets `paid_hours = Σ report.dynamic_hours` over `get_week`'s output (line 176-178, 288-290) and `overall_available_hours = volunteer_hours + paid_hours` (line 197, 309).

`committed_voluntary` is a **third** notion of volunteer capacity (pre-committed, not reactive, not shiftplan-derived). It must be displayed *separately* (D-01 / requirement 4) and folded into availability **without** re-adding hours already represented by (1) `auto_volunteer_hours`, (2) the `booking_information` `volunteer_hours` term, or (3) the `dynamic_hours` of a person who *also* shows up as a `!is_paid` volunteer.

---

## Critical Pitfalls

### Pitfall 1: Double-counting committed-voluntary against already-counted volunteer hours

**What goes wrong:**
The same volunteer hour is counted twice in `overall_available_hours` (and in the year view). Concretely, an unpaid volunteer (requirement 5: they now get an `EmployeeWorkDetails` record so they can hold `committed_voluntary`) is `is_paid = false`. In `booking_information.rs` their *actual shiftplan hours* are already summed into `volunteer_hours` via the `volunteer_ids` filter (line 123/248). If the new feature also adds `committed_voluntary` (their pre-commitment) into available capacity *additively*, and the person actually worked their committed hours, the year view shows `committed_voluntary` **plus** the shiftplan hours that fulfilled exactly that commitment → up to 2× the real capacity. This is the exact failure class the v1.3 Phase-8.4 additive merge had to defend against (STATE.md: "extra_hours + absence_period... keine Doppelzaehlung per-row").

The required formula is a **replacement-not-addition** rule: `available = expected + committed_voluntary`, and realised volunteer work only adds capacity *above* the commitment: `surplus = max(0, actual_volunteer − committed_voluntary)`. The trap is implementing it as `available + actual_volunteer + committed_voluntary` somewhere.

**Why it happens:**
- There are two pre-existing volunteer terms (reactive `auto_volunteer_hours` in `get_week`, and shiftplan-of-unpaid in `booking_information`). A developer fixes one path and forgets the other.
- `apply_weekly_cap` already produces `auto_volunteer_hours` from overflow; committed capacity overlaps conceptually with that overflow but is computed from a totally different source (contract field, not shiftplan).
- `get_week` and `booking_information` attribute "volunteer" to *different person sets* (work-details-holders vs `!is_paid`), so the merge boundary is non-obvious.

**How to avoid:**
- Pick **one** authoritative reconciliation site. The cleanest is `booking_information::get_summery_for_week`/`get_summary_for_year` where `overall_available_hours = volunteer_hours + paid_hours` is assembled (lines 197, 309) — this is where committed capacity should enter as a **separate, max-clamped term**, not inside `get_week`.
- Implement the clamp literally: for each capped person, `committed_capacity_contribution = committed_voluntary`; `realised_volunteer = actual_volunteer_shiftplan_hours`; `volunteer_surplus = max(0.0, realised_volunteer − committed_voluntary)`. The capacity shown = `committed_voluntary + volunteer_surplus`, NOT `committed_voluntary + realised_volunteer`.
- Decide explicitly whether `committed_voluntary` replaces or supplements the existing `booking_information` `volunteer_hours` term for capped persons, and write that decision into the plan. Do not let both flow through untouched.
- Keep `committed_voluntary` out of `balance_hours` / `expected_hours` entirely (it is *capacity*, not *obligation* — see P6).

**Warning signs:**
- A test where `committed=5, actual=3` shows available `8` instead of `5` (commitment under-fulfilled but actual added on top).
- A test where `committed=5, actual=7` shows `12` instead of `7` (`5 + 7`) or `5` instead of `7` (`max` applied wrong direction / surplus dropped).
- `overall_available_hours` for a week increases when you *add* a committed value to someone who was already volunteering the same hours.

**Concrete tests to add:**
- `committed_voluntary_under_fulfilled`: committed=5, actual_volunteer=3 → capacity contribution = 5 (covered, no surplus). Boundary `actual == committed` (5/5) → 5.
- `committed_voluntary_over_fulfilled`: committed=5, actual=7 → 5 + 2 surplus = 7.
- `committed_voluntary_no_double_count_in_year_view`: a person with `committed=5` who works exactly 5 volunteer shiftplan hours contributes 5 (not 10) to `overall_available_hours`. Assert the year total is unchanged vs. the same person with `committed=0` + actual=5 only if that is the intended invariant; otherwise pin the exact delta.
- `committed_voluntary_zero_actual`: committed=5, actual=0 → still shows 5 available capacity (the whole point of pre-commitment).

**Phase to address:** The reporting/year-view phase (the phase that touches `booking_information.rs` + `reporting.rs`). This is the highest-risk phase and likely warrants a dedicated research/design note before planning.

---

### Pitfall 2: Forgetting to bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` in the same commit as the reporting change

**What goes wrong:**
`committed_voluntary` changes the **input set and computation** of volunteer/capacity reporting. If the persisted billing-period value types (`Volunteer`, `Balance`, `ExpectedHours`, `Overall`) shift even slightly and `CURRENT_SNAPSHOT_SCHEMA_VERSION` (currently `7`, `billing_period_report.rs:74`) is not bumped in the *same commit*, then validators that re-run the live computation and diff against old snapshots cannot distinguish "schema changed" from "real data bug". CLAUDE.md flags this explicitly and STATE.md records it as a v1.0 lesson ("MUSS gebumpt werden im selben Commit wie Reporting-Switch"). The v1.3 history (versions v3→v7 in the doc-comment) shows every volunteer/absence computation change required a bump.

**Why it happens:**
- The bump is a one-line const change far from the reporting code; easy to forget when the visible work is in `reporting.rs`/`booking_information.rs`.
- It is non-obvious whether `committed_voluntary` actually changes a *persisted* `value_type`. It does **if** it changes `report_delta.volunteer_hours` (which feeds `BillingPeriodValueType::Volunteer`, lines 240-250) or any of `Balance`/`ExpectedHours`/`Overall`.

**How to avoid:**
- Trace whether the new formula alters any of `report.volunteer_hours`, `report.expected_hours`, `report.balance_hours`, `report.overall_hours` (the ReportingService outputs that `build_billing_period_report_for_sales_person` reads, lines 147-250). If **any** change → bump to `8` and add a `// - v8: Phase v1.4 — committed_voluntary ...` history entry in the doc-comment (lines 38-73 pattern).
- If `committed_voluntary` is *purely* a separate display value that never touches a persisted `value_type` (e.g. it lives only in `WeeklySummaryTO`, never in `BillingPeriodValue`), then **document the no-bump decision explicitly in the plan** with the reasoning — CLAUDE.md lists "new fields on unrelated tables / frontend changes" as no-bump cases.
- Make the bump (or the explicit no-bump justification) a checklist item in the reporting phase's success criteria.
- Keep old snapshots valid: never *remove* or renumber existing version history entries; only append. Old v7 snapshots must still validate as "older schema" — the validator compares `snapshot_schema_version` and treats `< CURRENT` as non-revalidatable, which is correct.

**Warning signs:**
- Billing-period validator suddenly reports drift on historical periods after the v1.4 merge.
- A new persisted `value_type` (e.g. a `CommittedVoluntary` variant) appears in `billing_period_sales_person` with no version bump.
- The `if report_delta.volunteer_hours != 0.0` guard (line 240) now fires for people it didn't before (committed capacity leaking into volunteer delta).

**Phase to address:** Same phase as the reporting change. The bump and the computation change are atomic by rule.

---

### Pitfall 3: Snapshot reads old data — but committed_voluntary is time-versioned, and old rows have no value

**What goes wrong:**
Billing-period snapshots and reports re-read `EmployeeWorkDetails` via `derive_hours_for_range` and `vacation_days_for_year`-style time-window logic. If `committed_voluntary` is added as a NOT NULL column without a safe default, the migration fails on existing rows; if added nullable / defaulted wrong, existing reports drift (a row that historically had no commitment must read as `0.0`, contributing nothing). The cap-flag precedent (`20260426120000_add-cap-flag-to-employee-work-details.sql`) is the exact template: `ADD COLUMN ... INTEGER NOT NULL DEFAULT 0`.

**Why it happens:**
- `EmployeeWorkDetails` rows are **time-versioned** (logical_id pattern, STATE.md v1.0): every contract change rotates a physical row. A naive default could attach a non-zero commitment to historical contract versions, retroactively inflating past capacity.
- The field is decoupled from `expected_hours` (D-01) but lives on the *same* row, so a contract-version rotation copies *all* fields including `committed_voluntary` — which is the desired behaviour only if the rotation logic copies it forward intentionally.

**How to avoid:**
- Migration: `ALTER TABLE employee_work_details ADD COLUMN committed_voluntary REAL NOT NULL DEFAULT 0;` (REAL for `f32`, matching `expected_hours REAL`). Default `0` guarantees no-drift on all existing rows.
- Add the field to **every** DAO select (`dao_impl_sqlite/src/employee_work_details.rs` has ~4 SELECTs at lines ~107/157/209/263), the INSERT (line ~341) and UPDATE (line ~429), the entity struct (line ~17-26), both `From`/`TryFrom` conversions in `service/src/employee_work_details.rs` (lines 42-72 and 192-226), and `EmployeeWorkDetailsTO` + both conversions in `rest-types/src/lib.rs` (lines 597-700). Missing any one of these is a silent `0.0` or a compile error.
- When a contract version rotates (the `with_from_date`/`with_to_date` + create-new-row path), confirm `committed_voluntary` is carried via the `..self.clone()` spread (it will be, since `with_*_date` uses struct-update syntax — but verify the editor/update path in `service_impl/src/employee_work_details.rs` does not reset it to a default).
- Regenerate `.sqlx` after the migration (see P9).

**Warning signs:**
- `cargo build` fails with "missing field `committed_voluntary`" — good, that's the compiler catching an unconverted site. Fix all of them.
- A historical billing period's volunteer/capacity values change after deploy (drift on a row that should have read `0.0`).
- Editing a paid contract silently zeroes a previously-set commitment (rotation didn't carry the field).

**Phase to address:** The data-model phase (migration + DAO + service + rest-types). Must land before the reporting phase.

---

### Pitfall 4: Time-versioning skew — committed_voluntary window diverges from expected_hours window

**What goes wrong:**
D-01 deliberately decouples `committed_voluntary` from `expected_hours` so the paid contract can change without silently moving the volunteer commitment. But `committed_voluntary` still rides on the **same** time-versioned row (`from_year/from_calendar_week … to_year/to_calendar_week`). So in practice you *cannot* have a commitment window that differs from the contract window without rotating the whole row. The trap: a planner sets "5h committed from week 10" expecting it to apply from week 10, but the active `EmployeeWorkDetails` version covers weeks 1-52 → the commitment applies retroactively to weeks 1-9 too (or, if they rotate the row at week 10, the *paid* contract also visibly rotates, which D-01 was trying to avoid).

**Why it happens:**
- "Decoupled value, shared versioning row" is a subtle middle ground. Developers read "decoupled" and assume independent time windows exist; they don't — only the *value* is independent, the *time window* is shared.
- `find_working_hours_for_calendar_week` (`reporting.rs:77-86`) selects the row whose `[from, to]` window contains the week; whatever `committed_voluntary` that row holds applies to the whole window.

**How to avoid:**
- Treat `committed_voluntary` as "the commitment for this contract version's time window." Document that changing the commitment mid-window requires rotating the row (same mechanism as changing `cap_planned_hours_to_expected` mid-year).
- In reporting, read `committed_voluntary` through the **same** `find_working_hours_for_calendar_week` selection already used for `expected_hours` and `cap_planned_hours_to_expected` (`reporting.rs:264, 813, 1000`) — do not invent a parallel lookup, or the windows will drift.
- If true independent windowing is ever required, that is a separate (rejected-for-now) data-model change; keep it out of v1.4 and note it as future work.

**Warning signs:**
- A commitment appears to apply to weeks before it was "set."
- Two active `EmployeeWorkDetails` rows overlap a week and both carry `committed_voluntary` → which wins? (The `.any(...)` cap pattern at line 264 would need a defined aggregation: sum? max? first?). Pin this with a test.

**Phase to address:** Data-model phase (define semantics) + reporting phase (consume via the shared lookup). Add a CONTEXT decision: "committed_voluntary aggregation across overlapping active work-details rows = [sum|max]."

---

### Pitfall 5: Unpaid volunteer gets an EmployeeWorkDetails record → leaks into paid-only queries

**What goes wrong:**
Requirement 5 gives a *purely unpaid* volunteer an `EmployeeWorkDetails` record so they can hold `committed_voluntary` and appear in the "alle" filter. But `is_paid` lives on **`SalesPerson`, not `EmployeeWorkDetails`** (verified: `sales_person.rs:17`, DAO `all_paid = WHERE ... is_paid = 1`). Many reporting/aggregation sites assume **"has work-details ⇒ paid employee"** and iterate work-details directly. Creating a work-details row for an unpaid person breaks that assumption in several places:

- **`ReportingService::get_week`** (`reporting.rs:719`) iterates `all_for_week` work-details — **not** the paid filter. An unpaid volunteer with a work-details row now appears here, gets `expected_hours`/`dynamic_hours` computed, and is pushed into the `ShortEmployeeReport` list.
- **`booking_information::get_summery_for_week`** then does `paid_hours += report.dynamic_hours` over **every** `get_week` row (line 178/290). So the unpaid volunteer's `dynamic_hours` (from their new work-details `expected_hours`) is added to `paid_hours` — *while their shiftplan hours are simultaneously in `volunteer_hours`* (they're `!is_paid`). Double-attribution + miscategorisation. This is the convergence of P1 and P5.
- **`get_reports_for_all_employees`** (`reporting.rs:139-142`) filters `employee.is_paid.unwrap_or(false)` — so the unpaid volunteer is *excluded* here. Now the same person is **in** the year/week summary but **out** of the all-employees report → inconsistent person sets across reports.
- **Billing period** (`build_new_billing_period`, `billing_period_report.rs:315-332`) iterates `get_all` (no paid filter) and builds a snapshot row **per sales person**. An unpaid volunteer with work-details would get a billing snapshot row computed from their work-details `expected_hours` → a "balance" for someone who is not paid.
- **Day-level loop** in `get_summery_for_week` (`booking_information.rs:341`) iterates `paid_employees` only (filtered `is_paid == true`) → the unpaid volunteer is *excluded* from per-day available hours but *included* in the weekly total via `get_week`. Per-day and weekly totals disagree.
- **`vacation_balance`** (`vacation_balance.rs:146`) uses `get_all_paid` → unpaid volunteer excluded (probably correct), but if any vacation/entitlement code reads work-details directly it would now see them.

**Why it happens:**
- Historically "work-details exist" was a reliable proxy for "paid employee with a contract." v1.4 breaks that proxy on purpose.
- The two filters (`is_paid` on SalesPerson vs. presence of work-details) live in different layers and were never required to agree before.

**How to avoid:**
- **Enumerate and gate every work-details-iterating site.** The known set: `reporting::get_week` (line 719), `reporting::get_reports_for_all_employees` (paid-filtered, line 139), `booking_information::get_summery_for_week`/`get_summary_for_year` (paid_hours accumulation lines 176-178, 288-290, and day-level paid_employees line 341), `billing_period_report::build_new_billing_period` (line 315), `vacation_balance` (line 146), `generate_custom_report` is_paid join (line 422).
- Decide the inclusion rule per site and write it into the plan. Likely: an unpaid volunteer's `committed_voluntary` should add to the *committed/volunteer* bucket, but their work-details `expected_hours` must **not** add to `paid_hours` and must **not** generate a paid balance or billing row. Practically: in `get_week` / `booking_information`, branch on `sales_person.is_paid` (resolved per `sales_person_id`), not on presence of work-details.
- Keep `expected_hours = 0` for unpaid-volunteer work-details rows (their contract is "0 paid hours"), OR explicitly exclude `!is_paid` rows from the `paid_hours` accumulation. Prefer the explicit exclusion — relying on `expected_hours = 0` is a data-discipline assumption that will eventually be violated.
- For billing: filter `build_new_billing_period`'s loop to skip `!is_paid` persons unless they actually have a committed/volunteer value worth snapshotting — and if you snapshot them, ensure their `Balance`/`ExpectedHours` are `0`.

**Warning signs:**
- `paid_hours` in the year view jumps when an unpaid volunteer is given a work-details record.
- `current_paid_count` (`shiftplan.rs` / `shiftplan_edit.rs:533`) or the `max_paid_employees` warning fires for an unpaid person — though note `current_paid_count` filters `sb.sales_person.is_paid` (`shiftplan_edit.rs:662` uses `get_all_paid`), so it is *probably* safe; verify with a test anyway.
- A billing period suddenly has snapshot rows for people HR considers unpaid.
- Year-summary person count ≠ all-employees-report person count.

**Concrete tests to add:**
- `unpaid_volunteer_with_work_details_not_in_paid_hours`: create `is_paid=false` SalesPerson + work-details with `expected_hours=10`, assert `paid_hours` for the week is unchanged (their 10h does not become paid capacity).
- `unpaid_volunteer_excluded_from_billing_balance`: build a billing period; assert the unpaid volunteer either has no row or `Balance == 0`.
- `unpaid_volunteer_does_not_trigger_paid_count_warning`: book the unpaid volunteer into a slot with `max_paid_employees`, assert no `PaidEmployeeLimitExceeded` warning.
- `paid_count_consistency_year_vs_all_employees`: assert the set of `sales_person_id`s in the year summary matches expectations and does not silently gain the unpaid volunteer in the paid bucket.

**Phase to address:** Best handled as its own concern spanning the data-model phase (the record exists) and the reporting phase (gating). Flag for deeper research — the site enumeration is the deliverable.

---

### Pitfall 6: Scope-gate leakage — committed_voluntary affects non-capped persons' balance/expected

**What goes wrong:**
The feature is scoped to `cap_planned_hours_to_expected = true` persons only (PROJECT.md scope-grenze, requirement 2). If the new formula reads `committed_voluntary` unconditionally (not gated by the cap flag), a normal employee who happens to have a non-zero `committed_voluntary` (e.g. set by accident, or default-migrated wrong) gets their `expected_hours`/`balance_hours`/available capacity altered — exactly the "irrelevant for normal employees" case the scope rule forbids.

**Why it happens:**
- The cap check (`cap_active = find_working_hours_for_calendar_week(...).any(|wh| wh.cap_planned_hours_to_expected)`, `reporting.rs:264, 813, 1000`) is computed *near* where `committed_voluntary` would be read, so it's tempting to read the value first and gate later — or forget to gate at all.
- `committed_voluntary` is on the same row as the cap flag, so it's always *available* even when the cap is off.

**How to avoid:**
- Gate every read of `committed_voluntary` behind the same `cap_active` predicate already computed at `reporting.rs:264/813/1000` and `booking_information`. When `cap_active == false`, `committed_voluntary` contributes `0.0` to everything.
- Default migration value `0.0` (P3) makes leakage *quiet* even if gating is missed — but do not rely on that; a planner could set a value on a non-capped person.
- Add an assertion in the data-model layer or a validation: `committed_voluntary != 0.0 ⇒ cap_planned_hours_to_expected == true` (optional, but makes the invariant explicit). At minimum, test it.

**Warning signs:**
- A non-capped employee's `balance_hours` or available capacity changes when `committed_voluntary` is set on their contract.
- The capacity formula reads `committed_voluntary` outside a `cap_active` branch.

**Concrete test to add:**
- `committed_voluntary_ignored_when_cap_off`: `cap_planned_hours_to_expected=false`, `committed_voluntary=5`, assert `expected_hours`, `balance_hours`, and `overall_available_hours` are identical to the `committed_voluntary=0` baseline.

**Phase to address:** Reporting phase. The gate is one predicate reused at every read site.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Relying on `expected_hours = 0` to keep unpaid volunteers out of `paid_hours` instead of an explicit `is_paid` gate | Less branching code | First non-zero `expected_hours` on an unpaid row silently inflates paid capacity; the assumption is invisible | Never — gate on `is_paid` explicitly |
| Reading `committed_voluntary` in `get_week` (single site) instead of in `booking_information` reconciliation | One edit location | `get_week` feeds both `paid_hours` and the all-employees report; folding committed capacity there risks P1/P5 double-attribution | Never for the availability merge; OK only for pure display if proven not to feed balance |
| Skipping the snapshot version bump because "it's just a display field" | Avoids invalidating history | If it *does* touch `volunteer_hours`/`balance`, validators silently mis-attribute drift forever | Only with a written trace proving no persisted `value_type` changes |
| Not regenerating `.sqlx` (relying on a live DB) | Faster local loop | CI / other devs / NixOS `cargo build` fails offline-mode query check | Never — `.sqlx` is committed (155 files present) |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `WeeklySummaryTO` (frontend ↔ backend) | Adding `committed_voluntary` to backend struct but forgetting the `From<&WeeklySummaryTO>` mapping in `shifty-dioxus/src/state/weekly_overview.rs:29-63` → field exists but always default | Add field to `service::booking_information::WeeklySummary`, `rest-types::WeeklySummaryTO` (line 901), and the frontend `From` impl in the same phase; rustc enforces the rest-types side once the field is non-default |
| `EmployeeWorkDetailsTO` | Adding the field to the domain struct but not both `From` conversions in `rest-types/src/lib.rs` (lines 636-700) | Use `#[serde(default)]` on the new TO field for wire backward-compat; add to both conversion impls (the Wire-Tier-Mirror pattern from v1.1 Plan 05-05) |
| OpenAPI surface test | Adding a new TO field or a new `BillingPeriodValueType::CommittedVoluntary` variant without updating the surface-assertion test (STATE.md Plan 08-03: path/schema/tag lists via `assert!`, no insta) | If a new schema name or value-type string is introduced, update the OpenAPI surface test's expected lists; field-level churn is intentionally not pinned |

## "Looks Done But Isn't" Checklist

- [ ] **Migration:** Column added `REAL NOT NULL DEFAULT 0` — verify all ~4 DAO SELECTs + INSERT + UPDATE in `dao_impl_sqlite/src/employee_work_details.rs` reference it (compiler will catch, but `.sqlx` must be regenerated).
- [ ] **Snapshot version:** Either bumped to `8` with a history entry, OR an explicit written no-bump justification proving no persisted `value_type` changed.
- [ ] **Double-count:** A test proves `committed=5, actual=5` yields capacity `5` (not `10`) in the year view.
- [ ] **Unpaid-volunteer gating:** A test proves an unpaid volunteer's work-details `expected_hours` does NOT enter `paid_hours` or a billing balance.
- [ ] **Scope gate:** A test proves `committed_voluntary` is a no-op when `cap_planned_hours_to_expected = false`.
- [ ] **Separate display:** `WeeklySummaryTO` carries committed capacity in its **own** field (requirement 4), not summed into `paid_hours`/`volunteer_hours`; frontend `From` impl maps it.
- [ ] **i18n:** New frontend labels (committed capacity, "alle"/all filter) added to **all three** locales (en/de/cs) — STATE.md flags the recurring `Locale::En`-in-`de.rs` bug; add per-locale reference-matcher tests (Plan 08-04 pattern).
- [ ] **Overlap aggregation:** Defined + tested behaviour when two active work-details rows in one week both carry `committed_voluntary`.
- [ ] **Person-set consistency:** Year summary, all-employees report, and billing period agree on who is paid.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Double-count shipped | MEDIUM | Reports are derive-on-read for live data → fix formula + redeploy; only persisted billing snapshots are stuck. Re-run affected billing periods or accept them as "older schema" if version was bumped. |
| Snapshot version not bumped | HIGH | Cannot retroactively distinguish drift from bug on snapshots written under the un-bumped version. Bump now; treat the ambiguous window's snapshots as suspect; manually validate or regenerate them. |
| Unpaid volunteer leaked into paid balance | MEDIUM | Live reports self-heal after gating fix; billing snapshots that captured the bad balance need regeneration or version-quarantine. |
| Migration default wrong (non-zero) | LOW-MEDIUM | New migration to reset historical `committed_voluntary` to `0` where it should be; redeploy. No data loss since the field is additive. |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| P1 Double-count | Reporting / year-view phase | `committed_under/over_fulfilled` + `no_double_count_in_year_view` tests green |
| P2 Snapshot drift | Reporting phase (same commit) | Version bumped to 8 + history entry, OR written no-bump trace; billing validator clean on history |
| P3 Migration / forward-default | Data-model phase | Migration `DEFAULT 0`; all DAO/From sites compile; historical report unchanged test |
| P4 Time-version skew | Data-model + reporting phase | Overlap-aggregation decision in CONTEXT + test; committed read via shared `find_working_hours_for_calendar_week` |
| P5 Unpaid-volunteer leakage | Data-model + reporting phase (flag for research) | `unpaid_volunteer_not_in_paid_hours` + `excluded_from_billing_balance` + `paid_count` tests green |
| P6 Scope-gate leakage | Reporting phase | `committed_voluntary_ignored_when_cap_off` test green |
| i18n / OpenAPI / `.sqlx` | Frontend + cross-cutting | Per-locale reference tests; OpenAPI surface test updated; `.sqlx` regenerated and committed |

## i18n / OpenAPI / sqlx operational notes

- **i18n (de/en/cs):** New strings — committed-capacity label in the year view, the "alle"/all filter toggle, any employee-view label for the commitment value. Add to `shifty-dioxus/src/i18n/{en,de,cs}.rs` + the `Key` enum in `mod.rs`. STATE.md records two i18n guard patterns to reuse: `i18n_*_present_in_all_locales` plus per-locale `i18n_*_match_{german,english,czech}_reference` sample tests that catch a `Locale::En` accidentally used inside `de.rs`.
- **OpenAPI surface test:** If a new `BillingPeriodValueType` variant (e.g. a `committed_voluntary` string) or a new schema/field is introduced, update the path/schema/tag surface-assertion test (Plan 08-03 pattern; not insta). New TO *fields* are intentionally not pinned; new *schema names* / value-type strings are.
- **`.sqlx` regeneration on NixOS:** After the migration, run under `nix develop` (NOT `nix-shell` — `shell.nix` is broken, per STATE.md/CLAUDE.local.md). Use `sqlx migrate run` (additive) — **never** `sqlx database reset` (DESTRUCTIVE, requires explicit user confirmation). Then regenerate `.sqlx`/query metadata so offline `cargo build` passes (155 `.sqlx` files are committed and must include the new columns).

## Sources

- `service_impl/src/reporting.rs` (get_week, get_reports_for_all_employees, hours_per_week, apply_weekly_cap) — HIGH (direct read)
- `service_impl/src/booking_information.rs:100-459` (WeeklySummary computation, volunteer_ids / paid_hours / overall_available_hours) — HIGH (direct read)
- `service_impl/src/billing_period_report.rs` (CURRENT_SNAPSHOT_SCHEMA_VERSION=7, version history, build_new_billing_period) — HIGH (direct read)
- `service/src/employee_work_details.rs` + `dao_impl_sqlite/src/employee_work_details.rs` (struct, conversions, DAO columns) — HIGH (direct read)
- `service/src/sales_person.rs` + `dao_impl_sqlite/src/sales_person.rs` (is_paid lives on SalesPerson; all_paid filter) — HIGH (direct read)
- `migrations/sqlite/20260426120000_add-cap-flag-to-employee-work-details.sql` (migration default-0 template) — HIGH
- `shifty-dioxus/src/state/weekly_overview.rs`, `src/page/weekly_overview.rs` (frontend WeeklySummary mapping, volunteer display) — HIGH
- `.planning/STATE.md` Accumulated Context (Phase 8.4 additive-merge no-double-count lessons, logical_id versioning, i18n guard patterns, snapshot-bump-same-commit rule) — HIGH
- `shifty-backend/CLAUDE.md` § Billing Period Snapshot Schema Versioning (MUST-bump trigger list) — HIGH
- `.planning/todos/pending/2026-06-22-committed-voluntary-capacity-jahresansicht.md` + PROJECT.md v1.4 (formula, scope, D-01) — HIGH

---
*Pitfalls research for: v1.4 committed voluntary capacity*
*Researched: 2026-06-22*
