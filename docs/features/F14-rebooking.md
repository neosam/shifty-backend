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

**Reader rule:** every aggregate that must remain balance-neutral in
the presence of future rebooking pairs filters `source = 'manual'`.
The first consumer is Plan 54-03's
`voluntary_ist_total_in_range(extra_hours, from_date, to_date)`
(renamed from `voluntary_ist_total_for_year` in Plan 54-07 Gap-Closure G1),
which sums the Ist voluntary hours for F1/F2 and must exclude the future
`rebooking` noise; otherwise the same voluntary hour would be counted
twice (once as its original `Volunteer` row, once as the
`Rebooking`-source row that neutralises it inside the paid chain).

**Audit rule:** `rebooking`-sourced rows stay in the database and stay
visible in *audit* queries — they are how F5 explains "why did the
balance change on that date". They are only invisible to end-user
aggregates.

**Balance-neutrality guarantee (VOL-ACCT-03):** the property test in
`service_impl/src/test/voluntary_stats.rs` shows that inserting an
equal-and-opposite pair `(+h, -h)` both stamped `source = 'rebooking'`
does not change `voluntary_ist_total_in_range(..)` — the F1/F2 numbers
stay stable across a rebooking event.

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

### Pure functions in `service_impl::reporting`

`VoluntaryStatsService` is thin. The math lives in three range-based
pure fns beside `committed_voluntary_prorata_for_week` (internal per-week
building block) in `service_impl/src/reporting.rs`:

```rust
/// VOL-STAT-01 / VOL-ACCT-01-Ist — Manual-only sum of Volunteer hours
/// in the date range `[from_date ..= to_date]`. Filters source = Manual
/// + soft-deletes. (Phase 54 Gap-Closure G1 — replaces
/// `voluntary_ist_total_for_year(_, year)`.)
pub fn voluntary_ist_total_in_range(
    extra_hours: &[ExtraHours],
    from_date: ShiftyDate,
    to_date: ShiftyDate,
) -> f32;

/// F1 denominator / D-F1-01 — number of ISO weeks in the range with at
/// least one active-contract day inside the range. `expected_hours = 0`
/// still counts. Edge-weeks count as 1 (day-level dilution happens in
/// the numerator, not here).
pub fn contract_weeks_count_in_range(
    working_hours: &[EmployeeWorkDetails],
    from_date: ShiftyDate,
    to_date: ShiftyDate,
) -> u32;

/// D-F2-01 — day-level pro-rata for a single ISO week using per-day
/// active EmployeeWorkDetails (handles mid-week contract changes).
/// Kept as an internal per-week building block for debug tests.
pub fn committed_voluntary_prorata_for_week(
    working_hours: &[EmployeeWorkDetails], year: u32, week: u8) -> f32;

/// F2 target = Σ (committed_voluntary / 7.0) over every range-day
/// covered by an active contract. Edge weeks contribute pro-rata for
/// the days that fall inside the range (D-F2-01 stays day-based).
/// (Phase 54 Gap-Closure G1 — replaces
/// `committed_voluntary_target_for_year(_, year)`.)
pub fn committed_voluntary_target_in_range(
    working_hours: &[EmployeeWorkDetails],
    from_date: ShiftyDate,
    to_date: ShiftyDate,
) -> f32;
```

**Rationale — range-based aggregation (Phase 54 Gap G1):** consistent
with `ReportingService::get_report_for_employee_range`; edge weeks
contribute pro-rata for the days that fall inside the range. Without
the cutoff, a 5h/week voluntary commitment starting in May yielded a
full-year target that overshot the actual reporting range by ~4x
(~177h vs. the realistic ~54h for a Jan–July window). See 54-UAT.md
gap G1.

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

---

**Conclusion.** Phase 54 delivers the read side of F14: HR sees F1/F2
in the employee report, the audit tables are in place, and the marker
column tells the future writers where the future rebooking rows will
live. Milestones v2.6 Phase 55 + Phase 56 attach the writers and the
cron on top of this baseline without another schema change.

*Last verification against code:* see git blame of this file.
