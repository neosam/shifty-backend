# F14 — Voluntary Rebooking

> **In short:** Capped employees with committed voluntary hours must
> not fall into a permanent balance deficit. F14 introduces the
> data-model foundation, the audit-traceable batch structure, and the
> HR-only "Voluntary Stats" read view (F1/F2) that shows how far the
> employee is from the annual voluntary target. The full end-to-end
> rebooking pipeline is spread across milestone v2.6, phases 54..56.

**Cluster ID:** F14
**Status:** partially shipped (Phase 54 = F1/F2 baseline)
**First introduced:** Milestone v2.6, Phase 54 (2026-07-07). F3 lands
in Phase 55 (HR-suggest write path); F4 (auto-cron) and F5 (approval
UI) in Phase 56.
**Responsible crates:** `service::{rebooking_batch, voluntary_stats}`,
`service_impl::{rebooking_batch, voluntary_stats}`,
`dao::rebooking_batch`, `dao_impl_sqlite::rebooking_batch`,
`service_impl::reporting` (four new pure fns),
`rest::report` (voluntary-stats route),
`rest-types::VoluntaryStatsTO`, `shifty-dioxus::component::voluntary_stats_row`.

---

## 1. Purpose

Some employees have contracts with `has_hour_cap = true` (Phase 26)
plus a `committed_voluntary` value on `employee_work_details`
(Phase 34). The cap prevents the paid shiftplan-hours part of the
balance from paying them beyond the contract, but voluntary work
(category `Volunteer`) is booked additively and *does* count into the
balance. When a capped employee simultaneously accumulates paid deficit
(sick days, holidays with `holiday_auto_credit` off, absence reduction)
and voluntary surplus, the two must be netted — otherwise the balance
line stays permanently red even though the person has effectively
worked their contract plus extra volunteer hours.

**Milestone v2.6 delivers a three-stage pipeline:**

1. Show the employee (F1) how much voluntary they perform per contract
   week on average, and (F2) how far they are from the annual voluntary
   target (`committed_voluntary` pro-rata).
2. HR proposes rebooking (F3): a batch that converts a chosen number of
   `Volunteer` hours into an equal-and-opposite `Rebooking`-sourced pair
   inside `extra_hours` so the balance chain sees the offset without
   losing the audit trail.
3. Once approved (F5) — and, from Phase 56, automatically scheduled by
   an admin-controlled cron (F4) — the batch commits the paired
   `extra_hours` rows atomically, both stamped
   `extra_hours.source = 'rebooking'`.

## 2. Feature Slices

| Slice | Milestone / Phase | Status | Purpose |
| --- | --- | --- | --- |
| F1 (Ø voluntary per contract week) | v2.6 Phase 54 | shipped | HR-only average — Σ Volunteer / contract-weeks for the ISO year. |
| F2 (voluntary target + delta) | v2.6 Phase 54 | shipped | HR-only view of `committed_voluntary` pro-rata target vs. actual + delta. |
| F3 (HR suggest → pending batch) | v2.6 Phase 55 | planned | HR proposes a paired rebooking; batch lands as `state = Pending`. |
| F4 (auto-cron scheduler) | v2.6 Phase 56 | planned | Toggle-gated cron auto-creates `AutoCron` suggestions; bumps snapshot schema 12 → 13. |
| F5 (approval / UI) | v2.6 Phase 55 | planned | HR reviews Pending batches and either approves (writes the paired rows) or rejects. |

**Rule of thumb for Phase 54:** everything that a *reader* needs works
today. Everything that a *writer* touches (F3/F4/F5) is deferred to
Phase 55/56.

## 3. Marker-Filter Rule ([D-54-DM-02])

`extra_hours` gets an additive column `source TEXT NOT NULL DEFAULT
'manual'`. The active domain values are `manual` and `rebooking`.

- **`manual`** — every row written by the pre-existing UI paths
  (HR CRUD, absence-convert vacation writer, dev seed, REST TO → service
  mapper). Rows migrated in by the additive `ALTER TABLE` land on
  `manual` via the column DEFAULT.
- **`rebooking`** — reserved for the paired rows F3/F4/F5 will emit
  starting in Phase 55. In Phase 54 no writer sets this value — the
  Marker exists as a *reader-side filter target* only.

**Reader rule (planned for Phase 55):** every aggregate that must remain
balance-neutral in the presence of future rebooking pairs will filter
`source = 'manual'`. In Phase 54 the filter is not active yet — the
Voluntary-Stats Ist aggregate reads
`EmployeeReport::volunteer_hours` from `ReportingService` and inherits
whatever filter that central path applies. When Phase 55 lands the
`source == 'manual'` cutoff inside `ReportingService`, this chain picks
it up automatically; otherwise a voluntary hour would be counted twice
(once as its original `Volunteer` row, once as the `Rebooking`-source
row that neutralises it inside the paid chain).

**Audit rule:** `rebooking`-sourced rows stay in the database and stay
visible in *audit* queries — they are how F5 explains "why did the
balance change on that date". They are only invisible to end-user
aggregates.

**Balance-neutrality guarantee (VOL-ACCT-03) — planned for Phase 55:**
once `source == 'manual'` filtering lands inside `ReportingService`
(Phase 55), inserting an equal-and-opposite pair `(+h, -h)` both stamped
`source = 'rebooking'` will not change `EmployeeReport::volunteer_hours`
— the F1/F2 numbers stay stable across a rebooking event because the
Voluntary-Stats chain consumes `EmployeeReport::volunteer_hours`
directly. The property test is deferred to Phase 55 together with the
first live rebooking writer.

## 4. Batch Structure

Two SQLite tables, created in migration `20260707000000_create-rebooking-batch.sql`.

### `rebooking_batch` — parent row

| Column | Type | Notes |
| --- | --- | --- |
| `id` | BLOB(16) PK | UUID v4. |
| `sales_person_id` | BLOB(16) FK | Which employee this batch reconciles. |
| `iso_year` | INT | ISO-year of the reconciliation window. |
| `iso_week` | INT | ISO-week within `iso_year` (see UNIQUE below). |
| `kind` | TEXT | `Manual` \| `HrSuggestion` \| `AutoCron` \| `AutoCronBackfill` (Phase 55/56 writers). |
| `state` | TEXT | `Pending` \| `Approved` \| `Rejected` \| `SkippedLocked`. |
| `created`, `approved`, `approved_by` | TEXT | ISO timestamps + user-name; `approved*` are NULL until state = Approved. |
| `deleted` | TEXT nullable | Soft-delete marker. |
| `update_process`, `update_version` | audit columns |

**Constraint [D-54-DM-01]:** UNIQUE partial index
`rebooking_batch_week_unique_idx` on
`(sales_person_id, iso_year, iso_week) WHERE deleted IS NULL` —
*global across all kinds*. Rationale: a Claim-on-Suggest — once HR
opens a Pending batch for week X, the F4 cron (Phase 56) must not
race in a second AutoCron batch for the same week. The partial index
(soft-delete-aware) is the enforcement point.

### `rebooking_batch_entry` — per-slot payload

| Column | Type | Notes |
| --- | --- | --- |
| `id` | BLOB(16) PK |  |
| `batch_id` | BLOB(16) FK → `rebooking_batch(id)` | No CASCADE — soft-delete pattern. |
| `sales_person_id` | BLOB(16) | Denormalised for query performance. |
| `hours` | REAL | Absolute number of hours the entry proposes to rebook. |
| `balance_before` | REAL | Balance snapshot at suggestion time (audit). |
| `voluntary_actual` | REAL | Actual Ist voluntary hours at suggestion time. |
| `voluntary_committed` | REAL | Pro-rata target at suggestion time (F2 numerator). |
| `extra_hours_out_id`, `extra_hours_in_id` | BLOB(16) nullable | FKs into `extra_hours` — filled on state → Approved (F3/F5 writers, Phase 55). |
| `created`, `deleted`, `update_process`, `update_version` | audit columns |

**Rule:** `extra_hours_out_id` + `extra_hours_in_id` are `NULL` while
`state = Pending`. They are set atomically inside the same transaction
that flips `state = Approved` — that is how F5 guarantees the paired
`extra_hours` rows are consistent with the batch state.

## 5. Services (Phase 54 baseline)

| Service | Tier | Purpose |
| --- | --- | --- |
| `RebookingBatchService` | Basic | HR-gated CRUD (find_by_id / find_by_sales_person_year_week / create) on `rebooking_batch` + entries. Deps: `RebookingBatchDao`, `PermissionService`, `ClockService`, `UuidService`, `TransactionDao`. No domain-service dep. |
| `VoluntaryStatsService` | Business-Logic | Read-only F1/F2. Deps: `ExtraHoursService`, `EmployeeWorkDetailsService`, `SalesPersonService`, `PermissionService`, `TransactionDao`. HR-only via API-level None-redaction (not 403). |

**Consumer wiring (Phase 54):** `RebookingBatchService` has no
consumer inside the code yet — it is DI-wired in `shifty_bin/src/main.rs`
so Phase 55's `RebookingReconciliationService` can plug in without a
DI refactor. `VoluntaryStatsService` is consumed exactly once, by the
REST handler `rest/src/report.rs::get_voluntary_stats`.

**Service-tier note.** Per `shifty-backend/CLAUDE.md` conventions,
`RebookingBatchService` is Basic (only DAO + Permission + Clock + UUID +
Transaction). `VoluntaryStatsService` is Business-Logic (consumes three
other domain services). The distinction is enforced in the runtime
graph — see [`../architecture/diagrams/service-graph-runtime.mmd`](../architecture/diagrams/service-graph-runtime.mmd).

### Aggregation model in `VoluntaryStatsService`

`VoluntaryStatsService` is thin. Two responsibilities:

**Ist (VOL-STAT-01 / VOL-ACCT-01-Ist):** delegates to
`ReportingService::get_report_for_employee_range` and reads
`EmployeeReport::volunteer_hours` for the requested range. That aggregate
covers all three sources — manual VolunteerWork ExtraHours, Shiftplan
cap-overflow (`auto_volunteer_hours`), and no-contract Shiftplan hours —
consistent with the OVERALL "Ehrenamt" value displayed on the Employee
detail page. The Rebooking-neutrality filter (`source == 'manual'`) is not
active in this service in Phase 54; it lands centrally in
`ReportingService` from Phase 55 and automatically flows into this chain.

**Soll + contract-weeks:** two range-based pure fns beside
`committed_voluntary_prorata_for_week` (internal per-week building block)
in `service_impl/src/reporting.rs`:

```rust
/// F1 denominator / D-F1-01 — number of ISO weeks in the range with at
/// least one active-contract day inside the range. `expected_hours = 0`
/// still counts. Edge-weeks count as 1 (day-level dilution happens in
/// the numerator, not here).
///
/// v2.6.1 (D-54.5-02): a week with at least one Absence day for the
/// same salesperson is excluded from the count (whole-week-out).
pub fn contract_weeks_count_in_range(
    working_hours: &[EmployeeWorkDetails],
    from_date: ShiftyDate,
    to_date: ShiftyDate,
    absences: &[AbsencePeriod],
) -> u32;

/// D-F2-01 — day-level pro-rata for a single ISO week using per-day
/// active EmployeeWorkDetails (handles mid-week contract changes).
/// Kept as an internal per-week building block for debug tests.
pub fn committed_voluntary_prorata_for_week(
    working_hours: &[EmployeeWorkDetails], year: u32, week: u8) -> f32;

/// F2 target = Σ (committed_voluntary / 7.0) over every range-day
/// covered by an active contract. Edge weeks contribute pro-rata for
/// the days that fall inside the range (D-F2-01 stays day-based).
/// (Phase 54 Gap-Closure G1 — Range-based supersedes the earlier
/// full-year variant.)
///
/// v2.6.1 (D-54.5-01): any ISO week that overlaps with at least one
/// active Absence day of the same salesperson (Vacation, SickLeave,
/// UnpaidLeave — category-agnostic) contributes `0` to the target
/// (whole-week-out, not pro-rated per day).
pub fn committed_voluntary_target_in_range(
    working_hours: &[EmployeeWorkDetails],
    from_date: ShiftyDate,
    to_date: ShiftyDate,
    absences: &[AbsencePeriod],
) -> f32;
```

**Rationale — range-based aggregation (Phase 54 Gap G1):** consistent
with `ReportingService::get_report_for_employee_range`; edge weeks
contribute pro-rata for the days that fall inside the range. Without
the cutoff, a 5h/week voluntary commitment starting in May yielded a
full-year target that overshot the actual reporting range by ~4x
(~177h vs. the realistic ~54h for a Jan–July window). See 54-UAT.md
gap G1.

### Absence-aware whole-week-out (v2.6.1, D-54.5-01 / D-54.5-02 / D-26-03)

Both range-based pure fns take an additional `absences: &[AbsencePeriod]`
parameter — the per-salesperson list of active (`deleted.is_none()`)
Absence records — and apply a **whole-week-out** overlay:

- **Soll (`committed_voluntary_target_in_range`, D-54.5-01):** any ISO
  week whose Mo–Su calendar range overlaps with at least one Absence
  day of the salesperson contributes `0` to the target, regardless of
  category (Vacation, SickLeave, UnpaidLeave). Not pro-rated per
  absence day.
- **Contract-weeks denominator (`contract_weeks_count_in_range`,
  D-54.5-02):** the same overlap excludes the week from the
  denominator, so `ist_per_contract_week` measures the average over
  weeks that were actually available for volunteer work.
- **Overlap helper:** the check reuses `period_overlaps_week`
  (`service_impl/src/booking_information.rs:75`), the single source of
  truth shared with the Weekly display (VFA-01 / D-26-03).
- **Rationale — Ist/Soll symmetry:** the Weekly display
  (`WeeklySummary.committed_voluntary_hours`) has zeroed absence weeks
  since v2.6.0; `EmployeeReport::volunteer_hours` (the Ist source) is
  factually 0 during absences too (no shiftplan, no manual
  VolunteerWork). Aligning the Soll aggregation removes the systematic
  overshoot that made the delta look like a legitimate volunteer
  shortfall (~15 h per 3 absence weeks with a 5 h/week commitment).
- **Deliberate reversal of D-F1-01 for this consumer chain:** F1's
  original `expected_hours = 0`-counts-through rule stays intact;
  Absence weeks are excluded on top of it. The reversal is scoped to
  `VoluntaryStatsService`; other consumers of `contract_weeks` are not
  affected.
- **Non-HR path never loads Absences.** The `AbsenceService` load runs
  inside the HR path only; the Non-HR redaction (all fields `null`) is
  short-circuited before any data read (`service_non_hr_does_not_load_absences`
  regression-test).

**Changelog:** v2.6.1 — `committed_voluntary_target_in_range` +
`contract_weeks_count_in_range` are Absence-aware (whole-week-out,
D-54.5-01 / D-54.5-02). See phase `54.5-voluntary-soll-absence-fix`.

**v2.6.1 addendum (Quick-Task 260710) — Voluntary fulfillment ratio:**
`VoluntaryStats` (and its DTO mirror `VoluntaryStatsTO`) gained a
sixth field `ist_per_soll_pct: Option<f32>` = `ist_total / soll_total *
100` — the fulfillment ratio in percent. It is `None` when
`soll_total ≈ 0` (division-by-zero guard: non-volunteers or a range
that falls entirely into absence weeks). Values can exceed 100 % when
Ist > Soll (volunteer over-delivered). The FE row hides the cell when
the field is `None`.

## 6. REST (Phase 54)

| Method | Path | DTO In | DTO Out | Auth |
| --- | --- | --- | --- | --- |
| `GET` | `/report/{id}/voluntary-stats?from_date=YYYY-MM-DD&to_date=YYYY-MM-DD` | — | `VoluntaryStatsTO` | any authenticated; HR-only content — Non-HR gets all fields = `null`. |

`VoluntaryStatsTO` (5 fields, all `Option<f32>`/`Option<u32>`, serde
`#[serde(default)]` for wire compatibility):

- `ist_per_contract_week` — F1 (Ø voluntary / contract-week).
- `ist_total` — F2 Ist (absolute Manual Volunteer sum for the range).
- `soll_total` — F2 Soll (`committed_voluntary` pro-rata across the range).
- `delta` — `ist_total − soll_total`.
- `contract_weeks` — F1 denominator (audit).
- `ist_per_soll_pct` — fulfillment ratio in percent (`ist_total /
  soll_total * 100`), `None` when `soll_total ≈ 0`.

**Query contract:** both `from_date` and `to_date` are inclusive
ISO-8601 dates (`YYYY-MM-DD`). Invalid format or `from_date > to_date`
returns HTTP 400 (precedent `rest/src/toggle.rs`).

**Redaction rule:** the redaction happens **inside**
`VoluntaryStatsService::get_voluntary_stats`, not at the REST layer
(precedent VAC-OFFSET-01 v1.8). Non-HR receives HTTP 200 with all
fields = `null`. HR sees the concrete values.

**Prefix-proxy:** the route lives under `/report` in the Axum tree.
The existing `[[web.proxy]]` entry in `shifty-dioxus/Dioxus.toml` for
`/report` covers it — no new proxy entry needed.

## 7. Related Features

- **F04 Extra Hours** — new column `source` lives on the `extra_hours`
  table; the readers upstream in F07/F08 use the marker filter.
- **F07 Reporting / Balance** — Balance chain will filter
  `source = 'manual'` from Phase 55 onward (once a `Rebooking` writer
  exists). Phase 54 introduces the marker but no writer, so all
  existing rows continue to enter Balance identically.
- **F08 Billing Period Snapshot** — no version bump in Phase 54.
  `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays at 12 because Phase 54 adds
  neither a persisted `value_type` nor a computation change. The
  12 → 13 bump lives in Phase 56 (REB-AUTO-05, F4-Cron) — see
  `REQUIREMENTS.md`.
- **F13 System Infrastructure** — the toggle
  `voluntary_rebooking_auto_active_from` (seeded Phase 54, `enabled = 0`,
  `value = NULL`) will gate the F4 cron in Phase 56. In Phase 54 it
  is dormant.

## 8. Manual Rebooking (F3)

*Introduced Phase 55 (v2.6).* HR-triggered, one-shot pair-write that
turns some `Volunteer` hours into a balance-neutralising `+/-` pair,
directly stamped `Approved` — no pending step.

### 8.1 Trigger

HR navigates to `/employees/{id}` (the Employee Details page) and opens
the *"Manual Rebooking"* menu item in the TopBar / Action-Menu
([D-55-06]). Explicit design: no button lives inside the read-only
Voluntary-Stats row — the TopBar keeps the read view uncluttered and
forces the direction choice to be a conscious modal action.

### 8.2 Modal shape

The `manual_rebooking_modal` component shows four inputs plus a
preview:

- **ISO week** — `iso_year` (`u32`) + `iso_week` (`u8`), defaulting to
  the current KW. HR may pick any week, including retrospective ones
  ([D-55-05]).
- **Direction** — radio between `VolunteerToExtra` and `ExtraToVolunteer`
  ([D-55-06] — no delta-sign inference in the trigger context, so the
  modal MUST offer the direction explicitly).
- **Hours** — positive `f32`; validated by REB-MANUAL-03 (see 8.5).
- **Preview** — mirrors the backend-computed pair payloads before
  submit.

### 8.3 Submit flow

The modal POSTs `ManualRebookingRequestTO` to `POST /rebooking/manual`
(routed via `/rebooking` in the Axum tree; dev-proxy entry landed in
Plan 55-02). The REST handler forwards to
`RebookingReconciliationService::rebook_manual`, which:

1. **HR-gates** as its first `await` — Non-HR gets `Forbidden` without
   any DAO round-trip.
2. **Opens a transaction** (`TransactionDao::use_transaction`).
3. **Builds the paired payloads** via a `Direction`-driven helper
   (`build_pair_payloads`) — the enum eliminates the sign-bug class
   that a raw signed `hours` argument would open.
4. **Writes two `ExtraHours` rows** stamped `ExtraHoursSource::Rebooking`
   with categories dictated by direction:
   - `VolunteerToExtra` → `-h Volunteer`, `+h ExtraWork`.
   - `ExtraToVolunteer` → `-h ExtraWork`, `+h Volunteer`.
5. **Creates a `rebooking_batch`** (`kind=Manual, state=Approved`,
   `approved` + `approved_by` set immediately from the auth context).
6. **Creates one `rebooking_batch_entry`** with the two ExtraHours row
   ids populated as `extra_hours_out_id` / `extra_hours_in_id` — the
   invariant of §4 (FKs are `NULL` iff `state = Pending`) is
   preserved even for the Manual path because the write is atomic and
   already lands in `Approved`.

### 8.4 State machine

Manual batches never enter `Pending` — they are born `Approved`. The
only allowed transition is the one the writer performs during creation:
`⟂ → Approved`. [D-55-04] pins this as one-shot: no undo, no
`Approved → Rejected` reversal, no delete. The Anti-Feature
REB-UNDO-01 in `REQUIREMENTS.md` is the normative anchor.

### 8.5 Errors

| HTTP | Body                                                           | When |
|------|----------------------------------------------------------------|------|
| 400  | `{"error":"RebookingErrorHoursMustBePositive"}`                 | `hours <= 0` or non-finite (REB-MANUAL-03; validated in `rest/src/rebooking.rs::post_manual` before touching the service). |
| 409  | `{"error":"RebookingErrorSlotTaken"}`                           | UNIQUE-slot `(sales_person_id, iso_year, iso_week)` already occupied by *any* batch (Manual / HrSuggestion / AutoCron), regardless of state. The BL propagates `ServiceError::EntityAlreadyExists`; the REST handler manually maps it to the i18n key so the batch UUID never leaks into the wire body (T-4 mitigation). |
| 403  | plain `Forbidden`                                              | Non-HR caller (BL HR-gate). |

On `RebookingErrorSlotTaken` the modal stays open; HR can change the
week and re-submit. No auto-overwrite — that would silently erase a
previous manual correction.

### 8.6 Read-aggregate state after success

Two `ExtraHours` rows and one Batch+Entry are persisted. The pair
carries `source == Rebooking`, so it is invisible to `EmployeeReport::volunteer_hours`,
`balance_hours`, and `overall_hours` — [VOL-ACCT-03] holds by
construction. The central `source != Rebooking` filter lives in
`ReportingService` (Wave-1 owner) and covers every downstream consumer
(`VoluntaryStatsService`, `BookingInformationService`, F1/F2 rows,
Balance line). Property-test 55-03 gives the end-to-end guarantee.

---

## 9. HR-Alert + Suggestion Modal (F5)

*Introduced Phase 55 (v2.6).* Two-step, two-actor variant of §8:
HR-Alert flags the person automatically, and HR resolves it through
the Suggestion Modal with an Approve or Reject click.

### 9.1 Alert predicate ([D-55-01])

Pure fn in `service/src/rebooking_reconciliation.rs`:

```rust
pub fn alert_predicate(balance: f32, voluntary_ist: f32, cap_active: bool) -> bool {
    cap_active && balance <= -0.5 && voluntary_ist > 0.0
}
```

- **`cap_active`** — the target's contract has `has_hour_cap = true`.
  Without cap, the balance chain already reconciles paid hours end-of-year;
  the alert is meaningless.
- **`balance <= -0.5`** — Float-Noise-Tolerance. Strict `< 0` would
  trigger on `-0.0001`-rounding artefacts; the half-hour threshold aligns
  with the UI's one-decimal granularity and only surfaces real gaps.
- **`voluntary_ist > 0.0`** — nothing to rebook if the person has zero
  volunteer hours in the range.

Truth-table test lives in
`service_impl/src/test/rebooking_reconciliation.rs::predicate_truth_table`
and pins the boundary triple `balance = -0.49 / -0.5 / -0.51` explicitly.

### 9.2 Backend ripple ([D-55-02])

`ShortEmployeeReportTO` (in `rest-types`) is extended additively:

```rust
#[serde(default)]
pub has_pending_rebooking: bool,
#[serde(default)]
pub pending_rebooking_id: Option<Uuid>,
```

Wire-compat is preserved by `#[serde(default)]` (precedent VAA-04). The
value is set inside `ReportingService::enrich_reports_with_pending_rebooking`
using a *predicate-first* pattern:

1. Compute `alert_predicate(balance, voluntary_ist, cap_active)` from
   the already-assembled ShortEmployeeReport aggregates.
2. Only if `true`, query
   `RebookingBatchService::list_pending_for_sales_person(Some(sp_id))`
   via the new DAO verb `find_pending_for_sales_person`.
3. If a `state = Pending` batch exists, stamp
   `has_pending_rebooking = true` + `pending_rebooking_id = Some(batch_id)`.

Two guardrails:

- **HR-gate** — enrichment aborts before the DAO call when the caller
  is Non-HR (`check_permission(HR_PRIVILEGE)`). Non-HR receives the
  default (`false` / `None`).
- **Authentication::Full skip** — internal aggregate callers
  (`BookingInformationService`, `VoluntaryStatsService`, PDF-Scheduler)
  pass `Authentication::Full` and never need the alert flag; the
  enrichment early-returns for them so the ~40 `get_week` test-setup
  call-counts stay intact ([D-55-EXEC-03]).

### 9.3 Alert UI

`shifty-dioxus/src/component/rebooking_alert_banner.rs` (added in Plan
55-04) renders a **non-blocking inline banner** in the Employees
overview list — one row per person with `has_pending_rebooking = true`.
No modal-on-load, no confirmation dialog — deliberately follows the
project-wide MEMORY rule
[`feedback_warnings_inline_not_dialog`](../../.planning/PROJECT.md).
Clicking the banner opens the Suggestion Modal for
`pending_rebooking_id`.

### 9.4 Suggestion Modal — IST / DANN ([D-55-03])

`shifty-dioxus/src/component/rebooking_suggestion_modal.rs` displays
the `RebookingSuggestionTO` payload returned by
`GET /rebooking-suggestions` (or the single `/{id}` fetch). Two columns
side by side:

| Field                | IST (`_before`) — pre-approve | DANN (`_after`) — post-approve |
|----------------------|-------------------------------|--------------------------------|
| Balance              | `balance_before`              | `balance_after`                |
| Voluntary Ist        | `voluntary_ist_before`        | `voluntary_ist_after`          |
| Voluntary Soll       | `voluntary_soll_before`       | `voluntary_soll_after`         |
| Voluntary Delta      | `voluntary_delta_before`      | `voluntary_delta_after`        |
| Proposed hours       | —                             | `proposed_hours`               |

**Every one** of those numbers is Backend-computed
([D-55-03], MEMORY `feedback_fat_backend_thin_client`). The FE never
subtracts, min-caps, or offsets — it only renders. Notably
`voluntary_delta_before` and `voluntary_delta_after` are **their own
Backend fields**, not derived on the wire — the FE does not compute
`ist - soll`.

`proposed_hours = min(|balance|, voluntary_ist).max(0.0)` — the
central pure fn `proposed_rebooking_hours` from
`service/src/rebooking_reconciliation.rs`.

The modal exposes exactly two actions: **Approve** and **Reject**.

### 9.5 State machine

```
                    suggest_for_week          approve_suggestion
                   ┌──────────────┐          ┌───────────────────┐
        ⟂  ─────►  │  Pending     │  ─────►  │     Approved      │
                   │  (Claim held)│          │  (pair written)   │
                   └──────────────┘          └───────────────────┘
                            │                            
                            │ reject_suggestion          
                            ▼                            
                   ┌──────────────┐                      
                   │  Rejected    │                      
                   │  (slot held) │                      
                   └──────────────┘                      
```

**`suggest_for_week`** — creates `rebooking_batch`
`kind=HrSuggestion, state=Pending`. The two entry FKs
(`extra_hours_out_id`, `extra_hours_in_id`) are `NULL` per §4
invariant — **no `ExtraHours` rows are written during Pending**. The
UNIQUE-slot claim is acquired directly through the partial index
[D-54-DM-01]; this is Claim-on-Suggest ([HR-ALERT-04]) — it prevents
double-suggestions for the same person / same ISO week and also
prevents the F4 AutoCron (Phase 56) from racing in.

**`approve_suggestion`** — state-conditional `UPDATE ... WHERE state='pending'`
(rows-affected == 1 wins the race, otherwise `BatchAlreadyResolved`).
On success, the paired `ExtraHours` rows are written **atomically in
the same transaction** that flips the state, so the §4 FK-invariant
becomes true exactly when the state becomes `Approved`.

**`reject_suggestion`** — state-conditional `UPDATE ... WHERE state='pending'`
to `Rejected`. **No `ExtraHours` writes**. UNIQUE-slot stays held
until the next ISO week ([D-55-07]) — the same person cannot receive
another suggestion for the same week, and HR does not re-review the
same rejection.

### 9.6 Alert termination ([D-55-07])

Both terminal states end the alert:

- **Approve** — the pair-write moves `balance` above the -0.5h
  threshold; the predicate flips to `false`.
- **Reject** — the slot is claimed by a non-Pending batch; the
  `find_pending_for_sales_person` query returns none; the DAO-side
  gate stamps `has_pending_rebooking = false`.

Consistent with `REQUIREMENTS.md` HR-ALERT-03 wording *"sichtbar
vermerkt"* — the Rejected batch stays in the audit trail, just not in
the banner.

### 9.7 No-Undo ([D-55-04], Anti-Feature REB-UNDO-01)

Both `Approve` and `Reject` are one-shot. There is no
`Approved → Rejected` or `Rejected → Approved` transition, no delete,
no undo endpoint. Reversing a rebooking requires HR to issue a
fresh Manual-Rebooking (F3) in the *opposite direction* — a distinct
audit event.

### 9.8 Race semantics ([HR-ALERT-03], T-55-01)

Two HR users clicking Approve at the same instant hit the
state-conditional UPDATE simultaneously. Exactly one gets
`rows_affected = 1`; the other gets `rows_affected = 0` which the BL
maps to `ServiceError::BatchAlreadyResolved`. The REST layer maps that
to `HTTP 409 { "error": "RebookingErrorAlreadyResolved" }` — the FE
i18n contract is stable and does not depend on SQL details (T-4 leak
mitigation). Same protocol for the Approve/Reject cross-race.

---

**Conclusion.** Phase 54 delivers the read side of F14: HR sees F1/F2
in the employee report, the audit tables are in place, and the marker
column tells the future writers where the future rebooking rows will
live. Phase 55 attaches the two user-facing writers (F3 Manual +
F5 HR-Alert/Suggestion) on top of that baseline without another schema
change — the entire pair-write invariant, the pending Claim-on-Suggest,
and the state-machine live in `RebookingReconciliationService`
(Business-Logic tier) with backend-computed IST/DANN and no undo.
Phase 56 adds the F4 AutoCron.

*Last verification against code:* see git blame of this file.

_Last updated: 2026-07-10 — Phase 55 F3+F5 sections appended._
