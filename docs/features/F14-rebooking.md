# F14 — Voluntary Rebooking

> **In short:** Capped employees with committed voluntary hours must
> not fall into a permanent balance deficit. F14 introduces the
> data-model foundation, the audit-traceable batch structure, and the
> HR-only "Voluntary Stats" read view (F1/F2) that shows how far the
> employee is from the annual voluntary target. The full end-to-end
> rebooking pipeline is spread across milestone v2.6, phases 54..56.

**Cluster ID:** F14
**Status:** shipped (Phase 54 = F1/F2 baseline; Phase 55 = F3 manual
rebooking + F5 HR-alert approve/reject; Phase 56 = F4 auto-cron
scheduler + rollout backfill endpoint).
**First introduced:** Milestone v2.6, Phase 54 (2026-07-07). F3 (manual
one-shot write path) and F5 (Claim-on-Suggest lifecycle: suggest /
approve / reject) shipped in Phase 55 (2026-07-23). F4 (auto-cron
scheduler + backfill) shipped in Phase 56 (2026-07-26).
**Responsible crates:** `service::{rebooking_batch, voluntary_stats,
rebooking_reconciliation, voluntary_rebooking_scheduler}`,
`service_impl::{rebooking_batch, voluntary_stats, rebooking_reconciliation,
voluntary_rebooking_scheduler, voluntary_rebooking_gate}`,
`dao::rebooking_batch`, `dao_impl_sqlite::rebooking_batch`,
`service_impl::reporting` (four new pure fns + the category-based
aggregation with its single `source`-aware exception in `assemble_weeks`
+ the `has_pending_rebooking` alert flag),
`rest::{report, rebooking}` (voluntary-stats + F3/F5 routes + the F4
`/admin/rebooking/backfill` route),
`rest-types::{VoluntaryStatsTO, rebooking}`,
`shifty-dioxus::component::voluntary_stats_row`.

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
| F3 (manual one-shot rebooking) | v2.6 Phase 55 | shipped | HR rebooks a chosen number of hours in one shot; batch lands as `kind = Manual`, `state = Approved`, paired `extra_hours` written atomically. |
| F4 (auto-cron scheduler + backfill) | v2.6 Phase 56 | shipped | `VoluntaryRebookingScheduler` (Monday 03:15 cron) processes the previous ISO week for every capped person via `run_for_week` (`kind = AutoCron`), plus the rollout `POST /admin/rebooking/backfill` endpoint (`kind = AutoCronBackfill`). Stichtag-toggle-gated (`voluntary_rebooking_auto_active_from`); capped at balance 0 since Phase 57 (REB-AUTO-06). **Snapshot schema bumped 12 → 13 in Phase 57** (D-57-10; the Phase-56 "no bump" branch is void — see §9.4). See §9. |
| F5 (HR-alert suggest / approve / reject) | v2.6 Phase 55 | shipped | Claim-on-Suggest (Model B): `suggest` claims a `Pending` `HrSuggestion` batch; `approve` writes the paired rows, `reject` only transitions state. |

**Rule of thumb:** F3 (manual) and F5 (HR-alert) ship in Phase 55 (see
§8 below); F4 (auto-cron + backfill) ships in Phase 56 (see §9 below).

## 3. Marker-Filter Rule ([D-54-DM-02])

`extra_hours` gets an additive column `source TEXT NOT NULL DEFAULT
'manual'`. The active domain values are `manual` and `rebooking`.

- **`manual`** — every row written by the pre-existing UI paths
  (HR CRUD, absence-convert vacation writer, dev seed, REST TO → service
  mapper). Rows migrated in by the additive `ALTER TABLE` land on
  `manual` via the column DEFAULT.
- **`rebooking`** — the paired rows written by F3/F4/F5 since Phase 55.
  Since Phase 57 the value is a pure **provenance** marker: UI labelling,
  audit queries, and a future undo capability. It no longer carries any
  blanket aggregation effect.

**Reader rule (state after D-57-06, VOL-ACCT-04).** The domain logic is
carried by the **category semantics**, not by the marker:

- `ExtraHoursCategory::as_report_type()` decides balance relevance —
  `ExtraWork` maps to `WorkingHours` (counts into the paid balance),
  `VolunteerWork` maps to `Documented` (does not).
- `ExtraHoursCategory::availability()` decides presence.

A rebooking pair over `h` hours therefore raises the balance by `h`
(the `+h ExtraWork` leg) and lowers the voluntary Ist by `h` (the
`−h VolunteerWork` leg) — exactly the effect the feature exists for.

**Exactly one `source`-aware exception remains.** In `assemble_weeks`
(the weekly-summary path, `service_impl/src/reporting.rs`) the
balance-relevant sum is built over `availability()`, and
`VolunteerWork.availability()` is `Available`. There the combination
`VolunteerWork` **and** `source = Rebooking` is deliberately excluded —
otherwise the `−h` leg would cancel the lift produced by the `+h` leg
and the pair would net to zero at that one site. In the voluntary-Ist
sum of the same function (`manual_volunteer_hours`) the very same row
stays in on purpose: without it the voluntary Ist would not drop.

**Pre-existing inconsistency, deliberately untouched (RISK-01).**
Ordinary (non-rebooking) `VolunteerWork` hours count into
`balance_hours`/`overall_hours` in the weekly summary but not in the
annual report — because the two paths derive the balance differently
(`availability()` vs. `as_report_type()`). Phase 57 did **not** touch
this; the Site-1 exception above is restricted to `source = Rebooking`
precisely so that the behaviour for ordinary voluntary rows stays
exactly as it was.

**Historical note.** Up to Phase 56 an unconditional marker filter
(`source == ExtraHoursSource::Manual`, keep `Manual`, exclude
`Rebooking`) sat at the aggregation entry of all four report paths
(`get_week`, `get_reports_for_all_employees`, `hours_per_week`,
`overall_extra_work_hours`, D-55-A1). It was justified with "double
counting" and derived from the original wording of VOL-ACCT-03, which
demanded balance neutrality. That requirement was itself the bug: the
filter made the **entire** rebooking transfer ineffective — a pair
changed nothing anywhere. Re-drafted in Phase 57 (VOL-ACCT-04); the
filter is gone at three sites and narrowed to the one exception above at
the fourth.

**Audit rule:** `rebooking`-sourced rows stay in the database and stay
visible in *audit* queries — they are how F5 explains "why did the
balance change on that date".

**What stays neutral (VOL-ACCT-03, re-drafted in Phase 57):** only the
**Soll**. `voluntary_soll` comes from `committed_voluntary` on the
contract and reads no `extra_hours` at all, so a rebooking pair leaves it
bit-identical. This is the one remaining neutrality guarantee, nailed by
`test_neutrality_f2_soll`
(`service_impl/src/test/rebooking_neutrality.rs`). The effect on balance
and Ist is nailed by `test_neutrality_balance`, `test_neutrality_f1_ist`,
`rebooking_neutrality_single_manual`
(`service_impl/src/test/rebooking_reconciliation.rs`),
`rebooking_effective_in_weekly_summary` (the Site-1 guard) and the
end-to-end reference scenario in
`service_impl/src/test/rebooking_reference_scenario.rs`
(8 h paid / 12 h voluntary / balance −2 → 10 / 10 / 0).

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
| `RebookingBatchService` | Basic | HR-gated CRUD (find_by_id / find_by_sales_person_year_week / create) on `rebooking_batch` + entries, plus the Phase-55 F5-Model-B lifecycle: `create_pending_suggestion` (Claim-on-Suggest producer — INSERTs a `HrSuggestion`/`Pending` batch and claims the UNIQUE week-slot; a taken slot → `EntityAlreadyExists`), `find_pending_for_sales_person` (reader for `pending_rebooking_id`, returns only `Pending`), `update_state` (race-safe state-conditional transition `Pending → Approved\|Rejected`; a second/already-decided transition → `EntityAlreadyExists` via `rows_affected()==0`, no version-guard; takes `approved_by` for the approver audit), and `create_entries_for_batch` (appends entries to an **existing** batch — used by F5-`approve` to close the audit chain after the pair-write, since `create` INSERTs a new batch). Deps: `RebookingBatchDao`, `PermissionService`, `ClockService`, `UuidService`, `TransactionDao`. No domain-service dep. |
| `VoluntaryStatsService` | Business-Logic | Read-only F1/F2. Deps: `ExtraHoursService`, `EmployeeWorkDetailsService`, `SalesPersonService`, `PermissionService`, `TransactionDao`. HR-only via API-level None-redaction (not 403). |
| `RebookingReconciliationService` | Business-Logic | Phase 55. The F3/F5 write orchestrator. Owns the single internal pair-write-path shared by F3 and F5-approve. Deps: `ExtraHoursService`, `RebookingBatchService`, `ReportingService`, `VoluntaryStatsService`, `PermissionService`, `TransactionDao`. All four methods HR-gated (T-55-05). See §8. |

**Consumer wiring (Phase 55).** `RebookingBatchService` (Basic) gained
its first consumers: `RebookingReconciliationService` (F3/F5 lifecycle)
and `ReportingService` (reads `find_pending_for_sales_person` for the
alert banner). `RebookingReconciliationService` is DI-constructed
**after** all Basic services and after `ReportingService` +
`VoluntaryStatsService` in `shifty_bin/src/main.rs` (order:
`extra_hours` → `rebooking_batch` → `reporting` → `voluntary_stats` →
`rebooking_reconciliation`, last — no `OnceLock` / forward-decl).
`VoluntaryStatsService` is still consumed by
`rest/src/report.rs::get_voluntary_stats`; the four F3/F5 routes in
`rest/src/rebooking.rs` consume `RebookingReconciliationService`.

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
detail page. Because the Ist is delegated, the rebooking effect flows in
automatically: since Phase 57 the `−h VolunteerWork` leg lowers
`volunteer_hours` by `h` (§3) — this service holds no filter of its own.

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
- `ist_total` — F2 Ist (absolute voluntary sum for the range).
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
  table; since Phase 57 it is provenance only (see §3).
- **F07 Reporting / Balance** — the read aggregates no longer filter by
  provenance; the category semantics decide (`as_report_type()` for
  balance relevance, `availability()` for presence), with the single
  `assemble_weeks` exception described in §3. A rebooking pair raises the
  balance by `h` and lowers the voluntary Ist by `h`; only the
  VOL-ACCT-01 Soll stays neutral (VOL-ACCT-03/04). See F07 §2.6.
- **F08 Billing Period Snapshot** — **bumped 12 → 13 in Phase 57**
  (D-57-10). With the marker filter gone, rebooking rows reach the
  persisted values, so the `value_type`s `Overall`, `Balance`,
  `ExtraWork` and `Volunteer` shift by `±h` for a period containing a
  rebooking week. This is a "change the input set the computation reads
  from" in the sense of the CLAUDE.md clause and therefore a mandatory
  bump. The Phase-56 REB-AUTO-05 "no bump" branch is void — see §9.4 and
  F08 §7.4.
- **F13 System Infrastructure** — the toggle
  `voluntary_rebooking_auto_active_from` (seeded Phase 54, `enabled = 0`,
  `value = NULL`) will gate the F4 cron in Phase 56. In Phase 54 it
  is dormant.

## 8. F3 Manual Rebooking + F5 HR-Alert (Phase 55)

Phase 55 attaches the write side. The orchestrator is the
Business-Logic `RebookingReconciliationService`
(`service::rebooking_reconciliation`,
`service_impl::rebooking_reconciliation`).

### 8.1 Single pair-write-path (shared by F3 and F5-approve)

There is exactly **one** internal pair-writer,
`rebook(direction, hours, week)` — a private impl method — used by both
F3 (`rebook_manual`) and F5 (`approve`). It writes the `−N` (source
category) / `+N` (target category) `extra_hours` pair, both stamped
`source = 'rebooking'`, inside the caller's transaction. Sharing the
writer prevents F3/F5 drift (Pitfall 15). `RebookingDirection` is
explicit (D-55-06): `VolunteerToExtra` (−N VolunteerWork / +N ExtraWork,
the standard deficit offset) or `ExtraToVolunteer` (correction of
over-committed pledges). F5-`approve` always uses `VolunteerToExtra`.

### 8.2 F3 — one-shot manual rebooking (REB-MANUAL-01/02/03)

`rebook_manual(sales_person_id, direction, hours, iso_year, iso_week,
…)` HR-gates first, then **validates the amount server-side** (`hours >
0`, else `ValidationError` → HTTP 422 — the client-side modal guard is
not authoritative; a direct API call with `0`/negative hours is rejected
before any row is written), then in **one** transaction: writes the pair
(§8.1), creates a `rebooking_batch` with `kind = Manual`, `state =
Approved`, and a `rebooking_batch_entry` carrying the two pair ids.
There is **no** suggest/pending intermediate step. A taken UNIQUE
week-slot → `EntityAlreadyExists` → HTTP 409.

### 8.3 F5 — Claim-on-Suggest lifecycle (Model B, D-55-B)

The F5 lifecycle is **Claim-on-Suggest** (Model B): the pending batch is
created at *suggest* time, not at approve time.

- **`suggest(sales_person_id, iso_year, iso_week, …)`** — HR-gated;
  claims the UNIQUE week-slot via
  `RebookingBatchService::create_pending_suggestion`, which INSERTs a
  `kind = HrSuggestion`, `state = Pending` batch. Idempotent on an
  already-open pending (returns the same id); a **decided** slot →
  `EntityAlreadyExists` → HTTP 409 (D-55-05, HR-ALERT-04). Returns the
  backend-computed IST/DANN preview `RebookingSuggestion`:
  `balance`, `voluntary_ist` (IST), `balance_after = balance +
  proposed_hours`, `voluntary_ist_after = voluntary_ist −
  proposed_hours` (DANN), and `proposed_hours = min(|balance|,
  voluntary_ist)` (D-55-03). The suggestion struct carries **only**
  these backend-computed fields — there is no `voluntary_soll` /
  `voluntary_delta` on it.
- **`approve(rebooking_batch_id, hours, …)`** — HR-gated. The **amount
  is backend-authoritative** (Fat-Backend / Model B): the client-supplied
  `hours` is **discarded**; `approve` recomputes `proposed_hours =
  min(|balance|, voluntary_ist)` for the batch's ISO week with exactly
  the same computation as `suggest` (report balance + voluntary stats
  over the Mon..=Sun range). If the recomputed amount is `<= 0` (nothing
  to rebook), it returns `ValidationError` → HTTP 422 — **no** state
  transition, **no** pair. Otherwise it transitions state **first** via
  `RebookingBatchService::update_state` — a state-conditional UPDATE `…
  WHERE id = ? AND state = 'pending' AND deleted IS NULL` with a
  `rows_affected() == 1` guard (Pitfall 12), passing the requesting HR
  principal as `approved_by` (audit) — and only on success writes the
  pair (§8.1, `VolunteerToExtra`, the backend `proposed` amount) in the
  same transaction. A double-approve loser (`rows_affected() == 0`) →
  `EntityAlreadyExists` → HTTP 409. `approve` does **not** create a
  second batch; it transitions the existing pending one and, after the
  pair-write, appends a `rebooking_batch_entry` to that **existing**
  batch via `RebookingBatchService::create_entries_for_batch` carrying
  the two pair ids + the audit snapshots (`hours`, `balance_before`,
  `voluntary_actual`, `voluntary_committed`) — closing the audit chain
  Batch → Entry → ExtraHours symmetrically to F3.
- **`reject(rebooking_batch_id, …)`** — HR-gated; transitions state
  `Pending → Rejected` via the same conditional UPDATE. **No** pair
  rows are written.

**Claim-on-Suggest / Phase-56 cron consequence:** because the pending
batch already claims the UNIQUE `(sales_person, iso_year, iso_week)`
slot (partial index, §4), the F4 auto-cron (Phase 56) cannot race a
second `AutoCron` batch into a week that HR has an open suggestion for —
`state = pending` blocks the cron slot.

### 8.4 HR-Alert flag on the short report (HR-ALERT-01, D-55-01)

`ReportingService::get_reports_for_all_employees` computes a per-employee
alert flag surfaced as `ShortEmployeeReport.has_pending_rebooking` +
`pending_rebooking_id` (mirrored on `ShortEmployeeReportTO`,
`#[serde(default)]`, wire-additive). The alert predicate is
`balance <= -0.5 ∧ voluntary_ist > 0 ∧ cap_active`
(`alert_predicate`, `service_impl/src/reporting.rs`, boundary-tested at
−0.49 / −0.50 / −0.51). The banner rule (D-55-B): the flag is `true`
only when the predicate holds **and** there is no *decided* batch in the
slot — a decided (approved or rejected) batch suppresses the banner, so
the alert extinguishes after **both** approve and reject (D-55-07).
`pending_rebooking_id = Some(pending.id)` when an open pending batch
exists. There is **no undo** (D-55-04). To keep the read path cheap the
batch lookup runs only for predicate candidates.

**Clarification (D-57-04) — the banner behaviour is unchanged.** The
alert still extinguishes on approve, on reject, and on a taken UNIQUE
week slot. It is a **working tool for HR**, not a watchdog against the
rebooking transfer becoming ineffective again. Decoupling D-55-07 was
explicitly rejected by the user ("once HR has clicked accept or reject,
HR has decided and there is no question left in that week"). The
watchdog role is carried structurally by the property tests in
`service_impl/src/test/rebooking_neutrality.rs` and
`service_impl/src/test/rebooking_reference_scenario.rs`, which turn red
as soon as a rebooking pair stops moving the balance — not by a visual
inspection of the banner.

### 8.5 REST routes (Phase 55, `rest/src/rebooking.rs`)

All four routes are HR-gated (at the service) and OpenAPI-documented;
status mapping (409 / 403 / 422) lives centrally in `error_handler`.

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/rebooking/manual` | F3 one-shot manual rebooking. |
| `POST` | `/rebooking-suggestions` | F5 suggest / claim (a deliberate mutation, Model B). |
| `POST` | `/rebooking-suggestions/{id}/approve` | F5 approve → writes the pair. |
| `POST` | `/rebooking-suggestions/{id}/reject` | F5 reject → state only. |

The optional read-only `GET /rebooking-suggestions` list was **not**
implemented (no list-pending service/DAO exists in 55-01/02; adding one
would be a Rule-4 architecture change).

## 9. F4 Auto-Cron + Rollout Backfill (Phase 56)

Phase 56 adds the **automation surface** on top of the Phase-55 write
path. The one-shot F3 write and the F5 approve lifecycle require a human;
F4 books the same rebooking pairs automatically, week by week, for every
capped person — plus a one-time backfill command that sweeps historical
weeks at rollout. Since Phase 57 the automation is **capped at balance 0**
(REB-AUTO-06, §9.1).

### 9.1 The trigger-agnostic engine — `run_for_week`

`RebookingReconciliationService::run_for_week(sales_person_id, iso_year,
iso_week, kind, …)` is the **single write-path** shared by the cron and
the backfill (Plan 02). It is trigger-agnostic: the caller passes the
`RebookingBatchKind` (`AutoCron` for the weekly cron, `AutoCronBackfill`
for the backfill), and the engine performs all the gating internally so
the two triggers can never drift:

- **Excess (capped since Phase 57, REB-AUTO-06)** — the automation
  settles the **paid hour account**, but only as far as the employee
  still fulfils their voluntary pledge. The balance never rises above 0
  through the automation. Formula (`week_excess`,
  `service_impl/src/rebooking_reconciliation.rs`):

  ```
  min( max(ist − committed_voluntary, 0), max(−balance, 0) )
  ```

  Non-finite inputs yield `0.0` (NaN/Inf guard on all four parameters).
  Nothing to rebook, or a balance ≥ 0 → `Skipped { NoExcess }`.

  **Balance source (D-57-07):** the cumulative **year-to-date** balance
  from the **Monday of ISO week 1** of `iso_year`
  (`ShiftyDate::new(iso_year, 1, DayOfWeek::Monday)`) up to and including
  the Sunday of the week being processed, **with**
  `include_carryover = true` — *not* the balance at run time and *not*
  the isolated week range that `suggest()`/`approve()` use. Rationale: a
  backfill sweeping several historical weeks would otherwise hand out the
  same headroom over and over.

  **Why the ISO anchor and not January 1 (CR-01):**
  `ShiftyDate::first_day_in_year(y)` goes through `from_ymd(y, 1, 1)` and
  stores the result as an ISO triple. When January 1 falls on Fri/Sat/Sun
  (2027, 2028, 2021, 2022, 2023) it belongs to the **previous** ISO year
  — `first_day_in_year(2027)` is ISO week 53/2026.
  `get_report_for_employee_range` reads `from_date.year() - 1` for the
  carryover lookup and iterates `hours_per_week` from
  `from_date.as_shifty_week()`, so a calendar anchor loads the wrong
  carryover year and pulls a full previous-year week into the balance.
  Since the value drives `max(-balance, 0)` and therefore the amount
  written, this is a write bug. Covered by
  `voluntary_rebooking.rs::run_for_week_balance_range_is_iso_anchored_in_2027`
  and `rebooking_backfill.rs::backfill_balance_range_is_iso_anchored_in_2027`.

  **Delimitation from the HR proposal (D-57-01, invariant I-5):** the
  HR-alert proposal in §8.3 stays at `proposed_hours = min(|balance|,
  voluntary_ist)` and is deliberately **not** capped by the pledge — it
  may undercut it, which is precisely the point of the feature; its lower
  bound is the annual voluntary Ist of 0. Two lower bounds, two
  responsibilities: the automation stops at balance 0, the human-approved
  proposal stops at voluntary Ist 0.

  **Reference scenario (D-57-02):** contract 5 h paid + 5 h voluntary per
  week, two weeks; 3 h worked in week 1, 17 h in week 2 → 8 h paid /
  12 h voluntary / balance −2, and a transfer of 2 h lands on 10 / 10 / 0.
  Counter-check for over-delivery: 23 h worked against 20 h pledged still
  transfers only **2**, not 5 — the cap binds. Both halves are executable
  in `service_impl/src/test/rebooking_reference_scenario.rs`, and the
  write side with the same numbers in
  `voluntary_rebooking.rs::run_for_week_balance_cap_binds`.
- **Idempotency (REB-AUTO-03):** a week that already carries an active
  `rebooking_batch` (an earlier cron run, a manual F3 batch, or an open
  pending HR suggestion) is skipped — `Skipped { AlreadyProcessed }`.
  The UNIQUE `(sales_person, iso_year, iso_week)` partial index (§4) is
  the enforcement point; Claim-on-Suggest (§8.3) means an open pending
  batch blocks the cron slot.
- **Week-lock (D-04):** a `WeekStatus::Locked` ISO week writes a
  `state = SkippedLocked` **audit batch** (no pair) and returns
  `Skipped { WeekLocked }` — the balance of a closed week is never
  mutated, but the skip stays visible in the audit trail.
- **Stichtag toggle (write-side self-guard, D-06):** the engine reads
  `voluntary_rebooking_auto_active_from` fresh each run
  (`voluntary_rebooking_gate::should_run_for_week`, the ISO-week Monday
  ≥ `active_from` cutoff, mirroring `shortday_gate`). Toggle unset, or
  the week's Monday strictly before `active_from` → `Skipped { ToggleOff }`
  — a full no-op, no batch written.

### 9.2 The weekly scheduler — `VoluntaryRebookingScheduler`

`VoluntaryRebookingScheduler` (Business-Logic tier,
`service::voluntary_rebooking_scheduler` /
`service_impl::voluntary_rebooking_scheduler`) is a thin
`tokio-cron-scheduler` job that mirrors `PdfExportSchedulerImpl` minus
`reload_from_db`:

- **Hardcoded 6-field cron `0 15 3 * * 1`** — Monday 03:15
  (`VOLUNTARY_REBOOKING_CRON`), conflict-free against the pdf-export
  Monday-06:00 job (D-03). Six fields are mandatory under
  `tokio-cron-scheduler` (a 5-field expression silently produces a
  dormant job — T-56-09).
- **`run_once_now`** computes the **previous** completed ISO week
  (year-boundary-safe: `ShiftyDate::from_date(clock.date_now()).as_shifty_week()`
  + a `previous_week` step-back — never a naive `year()`/`week()`), then
  iterates every capped person from `EmployeeWorkDetailsService::all_for_week`
  and delegates each to `run_for_week(…, AutoCron, …)`. A per-person
  error is logged and the loop continues (T-56-07 — no panic propagates
  out of the cron closure); it logs a `{booked} booked, {skipped}
  skipped, {failed} failed` summary.
- **No cron string compiled from the toggle:** unlike a stichtag-in-cron
  approach, the toggle is read fresh inside `run_for_week` every run, so
  flipping `active_from` takes effect on the next Monday without a
  reload. That is why the scheduler has no `reload_from_db`.
- **DI wiring:** minimal dep set — `RebookingReconciliationService`
  (the engine), `EmployeeWorkDetailsService` (the capped-person source),
  `PermissionService`, `ClockService`, `TransactionDao`. Booted in
  `shifty_bin/src/main.rs` at the end of `main()` **right after**
  `pdf_export_scheduler.start()`. It has **no** REST surface (the backfill
  endpoint is a separate route on `RebookingReconciliationService`), so
  it is a plain `RestStateImpl` field, not a `RestStateDef` accessor.

### 9.3 The rollout backfill endpoint

`POST /admin/rebooking/backfill` (`rest/src/rebooking.rs::trigger_backfill`,
nested under `/admin/rebooking`) is a thin delegate to
`RebookingReconciliationService::backfill_range`:

| Query param | Meaning |
| --- | --- |
| `from` | ISO week `YYYY-Www` (e.g. `2026-W20`); malformed / week `0` / week > `weeks_in_year` → **HTTP 400** *before* the service runs (T-56-11 DoS bound — no expensive iteration on garbage input). |
| `dry_run` | `true` = preview only (no write); `false` (or absent) = apply. Any other value → HTTP 400. |
| `page`, `per_page` | Optional 1-based pagination of the candidate list (multi-year backfills are not dumped in one response, D-05). |

- **HR-gated:** `backfill_range` checks `HR_PRIVILEGE` first
  (`service_impl/src/rebooking_reconciliation.rs`); a non-HR caller gets
  **HTTP 403**. The route lives under the `/admin/rebooking` nest.
- **Single write-path:** the apply branch delegates each candidate week
  to the same `run_for_week` engine with `kind = AutoCronBackfill`, so a
  backfilled week is gated (idempotency, week-lock, toggle) exactly like
  a cron week — the only difference is the batch `kind`, which lets audit
  queries tell a rollout sweep apart from the ongoing weekly cron.
- **Response** is a `BackfillPreviewTO` (candidate items with the
  computed `voluntary_ist` / `voluntary_soll` / `committed_voluntary` /
  `excess` / `would_book` / `skipped`), identical shape for dry-run and
  apply so the HR-admin can preview before committing.
- **Preview == apply (invariant I-7, since Phase 57).**
  `BackfillPreviewItem.excess` applies the **same balance cap** as the
  write path, so the previewed number is exactly the amount a
  `dry_run=false` run would book. This is not self-evident: the preview
  computes the excess itself and does **not** go through `run_for_week`,
  so it needs its own year-to-date balance read (same range, same
  `include_carryover = true`) — a `0.0` placeholder there would silently
  make the preview lie. Two mutually contradicting tests pin this
  (`backfill_dry_run_writes_nothing` at balance −8 → 3.0,
  `backfill_dry_run_preview_respects_balance_cap` at balance −2 → 2.0).

  **Headroom accounting across weeks (CR-02).** The apply path consumes
  the headroom implicitly: every written pair raises the balance, and
  SQLite sees those rows within the week loop's shared `read_tx`. The
  dry run writes nothing and would therefore see the same unconsumed
  headroom in **every** week. `backfill_range` consequently carries a
  `HashMap<sales_person_id, f32>` in dry-run mode that credits the
  already-previewed transfer against the balance it reads
  (`balance = report.balance_hours + consumed`). Without that accounting
  preview and apply diverge linearly with the number of weeks — at
  balance −10 with 4 h of excess each in weeks 10/11/12 the preview
  reports 12 h while 10 h get booked. Pinned by
  `backfill_dry_run_preview_matches_apply_over_multiple_weeks`, which
  puts the preview sum and the apply sum of the same scenario against
  each other.

### 9.4 Snapshot-schema decision (REB-AUTO-05 — flipped in Phase 57, D-57-10)

**`CURRENT_SNAPSHOT_SCHEMA_VERSION` was bumped 12 → 13 in Phase 57.**

Phase 56 answered the REB-AUTO-05 question ("is the F4-cron
`Rebooking`-source `ExtraHours` writer an *input-set change* in the sense
of the CLAUDE.md snapshot-versioning clause?") with **no**, and pinned
the "no bump" branch. That answer rested entirely on rebooking being
balance-neutral — the read aggregates filtered `source == Manual`, so no
`Rebooking` row ever reached a snapshot chain. **That premise was itself
the bug** (§3): the filter made the whole transfer ineffective. With the
filter re-drafted (VOL-ACCT-04, D-57-06), rebooking rows now flow into
the persisted `value_type`s `Overall`, `Balance`, `ExtraWork` and
`Volunteer` — `Overall` moves along because
`report_delta.overall_hours = shiftplan_hours_by_week +
overall_extra_work_hours` and the `+h ExtraWork` leg lifts the second
summand. Not affected: `ExpectedHours`, `VacationEntitlement`,
`VacationDays` and the absence categories — `rebook()` only ever writes
the `VolunteerWork` and `ExtraWork` categories, and the Soll comes from
the contract.

Under the CLAUDE.md clause "Change the input set the computation reads
from" the bump is therefore **mandatory**, and it was carried out in
Phase 57 (D-57-10). Existing v12 rows are left untouched (D-57-12, no
recompute, no repair migration) — validators only ever compare within the
same `snapshot_schema_version`, which is exactly what the version field
is for.

The proof is empirical and now runs the other way round: the straddling
golden-snapshot fixture
(`service_impl/src/test/billing_period_straddling.rs`) still proves that
the *toggle* does not move a single byte (the Stichtag only steers the
write side, invariant I-8), and additionally
`test_straddling_snapshot_rebooking_shifts_values` proves that the
*dataset* does — the same period with and without a rebooking pair
differs by exactly `±h` on the four value types above. A shift of `0`
there is the alarm: it would mean some chain filters the pair out again.
See F08 §7.4 for the bump history and F07 §2.6.

---

**Conclusion.** Phase 54 delivered the read side of F14 (F1/F2, audit
tables, marker column). Phase 55 attached the write side: F3 manual
one-shot rebooking and the F5 Claim-on-Suggest HR-alert lifecycle
(suggest / approve / reject), and the `has_pending_rebooking` alert flag.
Phase 56 added the F4 auto-cron scheduler (weekly Monday 03:15) and the
rollout backfill endpoint (§9). **Phase 57 made the transfer actually
work:** up to then an unconditional `source == Manual` filter kept every
rebooking pair out of the read aggregates, so a rebooking changed
nothing. Since Phase 57 the category semantics carry the domain logic
(VOL-ACCT-04), the automation is capped at balance 0 while the
HR-approved proposal deliberately is not (REB-AUTO-06, D-57-01), and the
snapshot schema stands at **13** (D-57-10). The only thing that is still
neutral across a rebooking is the VOL-ACCT-01 Soll.

*Last verification against code:* see git blame of this file.
