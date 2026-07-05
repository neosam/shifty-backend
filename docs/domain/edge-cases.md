# Edge Case Reference

This file is the **central collection of sharp edges** in the Shifty
system. It is mandatory reading before touching the time account, the
balance computation, the absence system, any snapshot-producing operation,
or cross-cutting concerns (auth, transactions, time).

Every edge case is categorized:

- **[Verified]** — Behavior is in the code with a file reference.
- **[Convention]** — Not enforced by the code, but agreed by the team.
- **[To verify]** — Assumption; must be checked in the code before changes.
- **[Known gap]** — Currently **not** handled correctly.

For every feature-specific edge (e.g. "PDF export on empty slot"), see
the respective [feature doc](../features/README.md), section "Edge cases".

---

## Table of Contents

1. [Time Account (Balance Computation)](#1-time-account-balance-computation)
2. [Absence & Extra Hours](#2-absence--extra-hours)
3. [Billing Period & Snapshots](#3-billing-period--snapshots)
4. [Time & Timezone](#4-time--timezone)
5. [Rounding & Precision](#5-rounding--precision)
6. [Authentication & Authorization](#6-authentication--authorization)
7. [Transactions & Atomicity](#7-transactions--atomicity)
8. [Soft-Delete Consistency](#8-soft-delete-consistency)
9. [Feature Toggles & Cutover Rollouts](#9-feature-toggles--cutover-rollouts)
10. [Migrations & sqlx-Offline-Cache](#10-migrations--sqlx-offline-cache)
11. [Frontend-Backend Coupling](#11-frontend-backend-coupling)
12. [Clippy & Toolchain-Split](#12-clippy--toolchain-split)
13. [i18n & Text Changes](#13-i18n--text-changes)
14. [Export & External Integrations](#14-export--external-integrations)

---

## 1. Time Account (Balance Computation)

The time account (also "balance") computes the difference between actually
worked and contractually expected hours, enriched with extras (vacation,
sickness, holiday, …).

**Core formula (simplified):**

```
balance = worked_hours − expected_hours + extra_hours_count_positive
```

The devil is in `worked_hours`, `expected_hours`, and the question of what
"counts positive".

### 1.1 Carryover Boundary & Year Rollover

**[Verified]** Carryover hours persist the year-end balance so that
historical periods are not recomputed.
Triggered by `scheduler.rs:60,68` — `update_carryover_all_employees(year-1, Full)`
and `update_carryover_all_employees(year, Full)`.

- **Edge case — retroactive change in a closed year:**
  If a booking or extra-hours row is changed in a year for which a
  carryover value already exists, the carryover is **not automatically
  invalidated**. Live reporting shows the new truth; the persisted
  carryover value drifts. [To verify] whether `carryover.rs` has an
  explicit re-compute path.
  *Consequence:* Balance in the following year can appear inconsistent
  (the carryover value does not match the freshly computed prior-year
  balance).

- **Edge case — Sales Person starts mid-year:**
  There is no synthetic carryover=0 entry for new hires — the balance
  computation should respect the start date (`sales_person.from` or
  contract start from `employee_work_details`). [To verify] whether the
  reader actually credits no expectation before contract start.

- **Edge case — new holiday in a closed year:**
  If a `special_day` is entered into an already-closed year, the
  "computed expected" for that year changes. The carryover value stays
  static. → **Convention:** Never enter Special Days retroactively into
  closed years unless the carryover is manually recomputed.

### 1.2 Contract Change Mid-Period

- **Edge case — weekly hours change within a week:**
  A Sales Person had 20 h/week through Wednesday, 30 h/week from Thursday.
  How are expected hours computed for THIS week? Pro-rata (Mon–Wed with
  a 20 h distribution + Thu–Sun with a 30 h distribution) or flat (whichever
  contract was in effect at week start)?
  [To verify] convention in `employee_work_details.rs` — fields `from`/`to`
  on contract rows.

- **Edge case — retroactive contract change:**
  A contract row with `from: 2024-01-01` is entered in 2026. The live view
  for 2024 drifts away from persisted snapshots (billing period and
  carryover). Without a version bump, the diff is inexplicable to
  validators.

- **Edge case — contract gap:**
  No `employee_work_details` row covers a day. [To verify] What is the
  expected value? 0? Error? Fallback to the last valid row?

### 1.3 Sales Person Time Boundaries

**[To verify]** `sales_person.from` and `sales_person.to` bound activity.
The exact filter point (reader, writer, or both) must be checked in the
code.

- **Edge case — Booking before `sales_person.from`:**
  Rejected on creation? Visible on read? Silent filter?
- **Edge case — Booking after `sales_person.to`:**
  An employee who left, historical booking entered. Does it appear in
  reports?

### 1.4 Special Days & Holidays

- **[Verified]** Special Days influence `expected_hours` (holiday = 0
  expectation; half day = pro-rata).
- **Edge case — holiday on a weekend:**
  If no expectation exists anyway (no-work weekend), a holiday reduces
  nothing. But it may be incorrectly displayed as "holiday credited"?
  [To verify] `special_days.rs` reporting path.
- **Edge case — movable holidays:**
  Easter and friends are represented via `special_days` table entries,
  not algorithmically. → **Convention:** At the start of the year, movable
  holidays must be entered manually (or a script does it).
- **Edge case — Special Day after billing period snapshot:**
  The snapshot stays fixed (see [§3](#3-billing-period--snapshots)), the
  live view shows the new value. The diff view in the UI must make this
  case understandable.

### 1.5 Balance Perimeter — What Counts Toward the Balance?

Not all Extra Hours categories count equally into the balance. From
`service/src/extra_hours.rs`, the following categories exist:

**[Verified]** in `ExtraHoursCategory`:
`ExtraWork`, `Vacation`, `SickLeave`, `Holiday`, `Unavailable`,
`UnpaidLeave`, `VolunteerWork`, `CustomExtraHours(id)`.

And in `ExtraHoursReportCategory` (reporting layer) additionally:
`Shiftplan` (derived from Bookings).

- **Edge case — `UnpaidLeave` counts differently:**
  Unpaid leave reduces the *expectation* but adds nothing on the *actual*
  side. Other categories (Vacation, SickLeave) do both. Every new category
  must explain this semantic explicitly.
- **Edge case — custom category without semantic definition:**
  If a custom category is created without specifying its reporting
  treatment, the outcome is implementation-dependent. [To verify] defaults
  in `custom_extra_hours.rs`.

### 1.6 Round Numbers, Unround Reality

See [§5 Rounding & Precision](#5-rounding--precision).

---

## 2. Absence & Extra Hours

### 2.1 Cutover History

**[Verified via CLAUDE.md]** The Absence system (v1.0+) is range-based
and replaces single-day Extra Hours **after Cutover**. The cutover date
is a point in time from which new vacation/sick/unpaid rows land in the
`absence` aggregate, while earlier ones still exist as `extra_hours`.

- **Edge case — period spans the cutover:**
  Old rows lie in `extra_hours`, new ones in `absence`. The report must
  aggregate from **both** tables. Anyone who forgets one of the two paths
  shows too little / double.
- **[Verified via memory]** For toggle cutover rollouts (D-51-07, HCFG-02),
  each consumer chain must reconstruct the old semantic before feature
  introduction in the gate-off branch — do not blindly assume
  "None → raw".

### 2.2 Range Edge Cases in the Absence System

- **Edge case — Absence spans two billing periods:**
  How is it split? Pro-rata across both or fully into the start period?
  [To verify] `absence_conversion.rs` and reporting call.
- **Edge case — Absence spans year rollover:**
  The share before Dec 31 must flow into the carryover. If carryover was
  computed before the absence insert, the share is missing.
- **Edge case — two Absences overlap:**
  Vacation Jun 1–15, Sick Jun 10–12. What counts on the overlapping days?
  Sick usually takes precedence. [To verify] whether the conflict logic
  splits automatically or raises an error.
- **Edge case — Absence on a non-working day:**
  Vacation requested on Sunday. Counts as 0h? As expected hours (if the
  contract prescribes Sunday work)?
- **Edge case — Absence vs Booking conflict:**
  An existing booking on a day, then Absence for the same day. What
  happens? Booking stays and is ignored? Error on Absence? Both remain
  (double count)? [To verify] `absence.rs` service logic.

### 2.3 Legacy Extra Hours — Delete Semantics

- **Edge case — deleting an `extra_hours` row that is already in a snapshot:**
  Snapshot stays fixed (it persists the aggregate, not the individual
  rows). Live view drifts. Without a version bump, the diff cannot be
  identified as "delete".

---

## 3. Billing Period & Snapshots

### 3.1 The Snapshot Contract

**[Verified]** `service_impl::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION`
is a `pub const u32` (currently **12**, see `billing_period_report.rs:117`).

Every written `billing_period` row carries this value.
`build_and_persist_billing_period_report()` writes it (see
`billing_period_report.rs:390`).

### 3.2 Bump Rule

**Bump the version by 1 when you:**

- add a new persisted `value_type` to `billing_period_sales_person`
  (enum extension → row write),
- remove / rename an existing `value_type`,
- change the computation of an existing `value_type` (different formula,
  different inputs, different filtering),
- change the input set the computation reads (e.g. co-aggregating a new
  `extra_hours` category).

**Do NOT bump when you:**

- make purely additive changes that do not touch a `value_type` (new
  REST endpoints, frontend changes, new fields on unrelated tables).

### 3.3 Edge Cases Handling Snapshots

- **Edge case — old snapshot (v11) is read against code v12:**
  The validator (`billing_period_report.rs`) must detect this version
  discrepancy and either skip re-compute or emulate the old formula.
  [Verified via CLAUDE.md] That is the purpose of the version field.
- **Edge case — developer forgets the bump after a formula change:**
  Validator interprets the old snapshot as "same rules" and finds a diff.
  It is falsely reported as a data bug. → Pattern: PR review must watch
  for changes in `billing_period_report.rs` computation and actively look
  for a version bump.
- **Edge case — snapshot is produced during an ongoing period change:**
  Race between "booking is created" and "snapshot is generated".
  → Snapshot creation MUST run under a TX that keeps the read set
  consistent. [To verify] TX behavior in
  `build_and_persist_billing_period_report`.
- **Edge case — no snapshot exists but a report is requested:**
  Live computation kicks in. Does the UI make it clear that this is not
  a frozen value? [To verify] REST DTO field for "is_snapshot".
- **Edge case — snapshot with wrong version for feature-flag regime:**
  If a toggle changes the semantics, the snapshot version MUST be bumped
  — otherwise it is semantically ambiguous (see
  [§9](#9-feature-toggles--cutover-rollouts)).

---

## 4. Time & Timezone

- **Edge case — daylight saving time switch:**
  Mar/Oct DST switch. Booking from 02:00 to 04:00 on the spring night:
  either 1h effective (because 02:00 → 03:00 skipped) or 2h (naive
  calculation). [To verify] whether time arithmetic is UTC-based or
  naive-local.
- **Edge case — SQLite storage:**
  SQLite is timezone-less. The convention is [To verify] — presumably
  UTC or Berlin-Local. Important to know when a client displays the
  values.
- **Edge case — week across year rollover:**
  Week 1 of the new year often already starts in December (ISO 8601). If
  a report refers to "week 1/2026", the boundaries must be unambiguous.
  [To verify] use of `iso_week_number` in `datetime_utils.rs`.
- **Edge case — leap year Feb 29:**
  Annual calculations (expectation "per day" × 365) must handle 366.
- **Edge case — one-day Absence via DateRange:**
  Half-open vs closed range. `[from, to)` or `[from, to]`? Off-by-one on
  "3 days of vacation" trivially caused.

---

## 5. Rounding & Precision

- **Edge case — float precision:**
  `f32` (as in `ExtraHours::amount` and `WorkingHoursDay::hours`) loses
  bits when summing many rows. 100 × 0.1 = 9.9999… in f32. For precise
  display, prefer multiplying before summation or use rational arithmetic.
- **Edge case — rounding ≠ associative:**
  `round(a + b + c) ≠ round(a) + round(b) + round(c)`. If the UI sums
  the individually rounded display values, it will diverge from the
  backend sum. → **Convention:** Always show the backend total, never
  re-add rounded individual values on the client.
- **Edge case — display rounding vs persistence rounding:**
  Display with one decimal, persistence with f32/four digits. The user
  sees "1.2h", the snapshot stores "1.234h". When comparing across
  periods, small differences appear large.

---

## 6. Authentication & Authorization

### 6.1 `Authentication::Full` Bypass

**[Verified]** In `service_impl/src/permission.rs`, `Authentication::Full`
returns `Ok(())` early for all permission checks (`permission.rs:28,41,63,80,90`).

This is how **internal aggregates** (business-logic services) consume
basic services without every individual read call having to thread a
user context through the chain.

- **[Verified via memory]** Phase 51 (Toggle Full-Context-Bypass):
  The ToggleService (`service_impl/src/toggle.rs`) had a guard that
  prevented Full reads; this broke internal aggregate callers (reporting,
  booking_information call with Full). Reads are exempted for Full.
- **Edge case — a new service copies the read handling and forgets the
  Full bypass:**
  Internal aggregates fail silently (or with a permission error).
  Reporting drifts.
- **Edge case — a REST endpoint receives Full instead of user context:**
  Catastrophic bug. `Full` is **exclusively for internal calls**. All
  REST handlers MUST forward the user context provided by the session
  layer.

### 6.2 OIDC / Mock Split

- **Edge case — Mock in Dev, OIDC in Prod:**
  Test coverage gap: RBAC deny paths are never traversed in Dev because
  Mock is always Admin. → Explicit unit tests with "not admin, only role
  X" are mandatory.
- **Edge case — OIDC token expiry mid-request:**
  [To verify] `session.rs` reaction. Refreshed? 401 returned?
- **Edge case — role change during an active session:**
  User loses a role but the frontend still shows the corresponding
  buttons. Click → 403. [To verify] whether the frontend has a
  refresh-on-403 pattern.

### 6.3 User Invitation

- **Edge case — invitation link redeemed multiple times:**
  [To verify] `user_invitation.rs` — session-revoke semantics
  (`20251020000000_add-session-revoked-at-to-user-invitation.sql` exists).

---

## 7. Transactions & Atomicity

### 7.1 The `Option<Transaction>` Pattern

**[Verified via CLAUDE.md]** Every service method accepts
`Option<Self::Transaction>`. If `None`, the service itself opens a TX
and commits at the end.

```rust
async fn do_something(&self, tx: Option<Self::Transaction>) -> Result<T, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // ... business logic ...
    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

- **Edge case — nested `commit`:**
  If an outer caller owns the TX and an inner service naively commits,
  the TX is half-committed. `use_transaction` must catch this case
  (pattern: no internal commit if `tx` was `Some` before). [To verify]
  implementation in `transaction_dao`.
- **Edge case — rollback on panic:**
  If a panic path runs between `use_transaction` and `commit` (not a
  `Result::Err`), is there a clean rollback? [To verify] Drop impl.

### 7.2 Re-Point Atomicity

**[Verified via memory]** For data moves / re-points (Slot-Split,
Booking migration): everything in ONE transaction (rollback) + hard tests
against double-counting in reports/balance. Phase 23 learned this
painfully.

- **Edge case — slot split without booking migration in the same TX:**
  Intermediate state: Slot A has a "split" marker but bookings still hang
  on the old slot. Report shows bookings 2× or not at all.
- **Edge case — booking move fails mid-way:**
  Bookings 1–5 moved, booking 6 fails. Without rollback: inconsistent
  state. With rollback: all 6 remain at the source slot — consistent.

### 7.3 SQLite as Single Writer

- **Edge case — two parallel writes:**
  SQLite serializes. The second write may get `BUSY`/`LOCKED`.
  [To verify] retry behavior and timeout.
- **Edge case — long-running TX blocks all others:**
  A report running in a TX for a long time holds writers back.
  → **Convention:** Wrap read ops in a spanning transaction only when
  consistency requires it.

---

## 8. Soft-Delete Consistency

**[Verified via CLAUDE.md]** All reader queries filter
`WHERE deleted IS NULL`.

- **Edge case — new query without the filter:**
  Ghost rows in the report. The filter is a convention, not a structural
  lock. → **Review rule:** Check new `query!/query_as!` for
  `deleted IS NULL`.
- **Edge case — aggregate parent deleted, children remain:**
  Sales Person soft-deleted — what happens to their Bookings, Absences,
  Extra Hours? [To verify] whether cascade soft-delete exists. Without
  cascade: bookings hang orphaned, reports can still see them if the
  reader omits `sales_person.deleted IS NULL`.
- **Edge case — foreign key on a deleted row:**
  SQLite checks FKs only when enabled. On soft-deleted rows no FK
  violation triggers. Silent bad data possible.

---

## 9. Feature Toggles & Cutover Rollouts

**[Verified via memory]** Toggle cutover features (e.g. D-51-07,
HCFG-02) have a gate-off branch per consumer chain that reconstructs the
old semantic before feature introduction. Do not blindly assume
"None → raw".

### 9.1 Rollout Edge Cases

- **Edge case — old data under new toggle:**
  Toggle "on" — data before the cutover date must be computed under the
  old semantic, otherwise the past is falsified. The gate-off branch in
  every consumer chain is mandatory.
- **Edge case — toggle switched at runtime:**
  Snapshot was "off", now read as "on". Without a snapshot version bump
  the behavior is ambiguous.
- **Edge case — Toggle read under two different auth contexts:**
  Reporting calls with Full, REST handler with User. If the toggle read
  answers differently, the computation drifts from the view.
  → Phase 51 ensured that Full lets the read pass.

### 9.2 Feature Flags vs Toggles

There are two mechanisms:

- **`feature_flag`** (see `service/src/feature_flag.rs`) —
  presumably compile-time-oriented or boolean store.
- **`toggle`** (see `service/src/toggle.rs`) — with user and date
  context.

[To verify] exact semantic separation — see
[`../features/F13-system-infrastructure.md`](../features/F13-system-infrastructure.md).

---

## 10. Migrations & sqlx-Offline-Cache

**[Verified via memory]** CI uses `SQLX_OFFLINE=true` + `cargo test`.
After every new `query!/query_as!` usage, `cargo sqlx prepare --workspace`
MUST run and the `.sqlx/` cache must be committed. Incremental builds may
be green while `--doc` target / clean build / CI fails. Phase 33 found
this.

- **Edge case — query with dynamic SQL:**
  `sqlx::query_with(&format!(…))` is not covered by the offline check.
  Silent compile OK, runtime error.
- **Edge case — migration removes a column, a query still references it:**
  Compiles if the `.sqlx/` cache was not regenerated. → After a migration
  ALWAYS re-run `sqlx prepare`.
- **Edge case — two devs, different DB states:**
  The `.sqlx/` cache in the commit is authoritative. The local DB must
  match the state. → `sqlx migrate run`, do not mix multiple migrations.

---

## 11. Frontend-Backend Coupling

**[Verified via memory]** New backend routes need `[[web.proxy]]` in
`shifty-dioxus/Dioxus.toml` — otherwise 404 in the `dx serve` dev mode.
Phases 28 + 49 both forgot it.

- **Edge case — new endpoint without proxy entry:**
  Prod works (static bundle is proxied by the reverse proxy), Dev does
  not. Reproduction in Dev is impossible until the proxy is added.
- **Edge case — DTO field change without WASM rebuild:**
  Frontend holds cache of the old DTO. Deserialization fails silently or
  throws in the browser console log.
- **Edge case — dx-CLI version drift:**
  [Verified via memory] shifty-dioxus needs dx 0.6.x (crate dioxus
  0.6.3). nixpkgs rolled to 0.7.x → app does not start, design stripped.
  Pinned in `flake.nix`. Style path `Dioxus.toml`: `/assets/tailwind.css`.
- **Edge case — backend roundtrip not tested in frontend phases:**
  [Verified via memory] Frontend phases with a "backend already exists"
  assumption MUST be verified e2e in the browser. Create path ≠ edit path
  (Phase 23: `modify_slot` dropped `max_paid_employees`).

---

## 12. Clippy & Toolchain-Split

**[Verified via memory]** `nix build` enforces
`cargo clippy -- --deny warnings`; `cargo test`/`build` and local CI do
NOT. Every phase gate MUST additionally run
`cargo clippy --workspace -- -D warnings`.

- **Edge case — tests green locally, `nix build` fails:**
  Most common case: clippy finds warnings that `cargo build` ignores.
- **Edge case — frontend workspace clippy:**
  [Verified via memory] shifty-dioxus is its own workspace, excluded
  from CI clippy (~198 pre-existing lints). Clippy is broken in the
  dioxus shell (E0514) and must be run from the backend shell.
  → New lints in the dioxus area drift unnoticed.
- **Edge case — `#[allow(…)]` proliferation:**
  If clippy findings are suppressed rather than fixed, they accumulate.
  No gate catches this.

---

## 13. i18n & Text Changes

**[Verified via CLAUDE.md]** New text needs translation in all three
locales: **En, De, Cs**.

- **Edge case — new text in only one language:**
  Fallback display shows the key name or an empty string. [To verify]
  exact fallback behavior.
- **Edge case — plural forms:**
  German has different plural rules than English; Czech has multiple
  plural forms (1, 2–4, 5+). [To verify] whether the i18n framework
  supports plural rules.
- **Edge case — text variable in wrong order:**
  German orders subject/object differently. If the frontend concatenates
  segments, German/Czech breaks. → Always use full-sentence templates,
  never fragment concatenation.

---

## 14. Export & External Integrations

### 14.1 PDF Export

- **Edge case — Sales Person with a very long name:**
  Layout overflow? [To verify] PDF renderer behavior in `pdf_render.rs`.
- **Edge case — period with 0 bookings:**
  Empty page? No page at all? → Customer expects visual feedback, not an
  empty file.
- **Edge case — Special Days overlay:**
  Holiday in a slot; is it drawn as "empty" or with a holiday marker?
- **Edge case — scheduler-driven PDF export:**
  `pdf_export_scheduler.rs` runs on a schedule. What happens if a
  scheduler tick fails? [To verify] recovery behavior.

### 14.2 iCal

- **Edge case — timezone in iCal:**
  iCal is strict about TZ definitions. If the backend serializes UTC or
  local, the TZID block must match. Otherwise the calendar shows the
  event at the wrong time.
- **Edge case — recurring events:**
  Recurrence rules (RRULE) correct? [To verify] `ical.rs`.

### 14.3 WebDAV

- **Edge case — auth error on WebDAV upload:**
  `webdav_client.rs` transfers PDF exports to a cloud storage. Network
  errors, 401, 507 (Insufficient Storage). Retry? Log? User-facing
  error?

---

## Meta Edge Case: Finding New Edge Cases

If you come across another edge case that is not documented here:

1. Add it to the same section.
2. Mark it as **[To verify]** until you have verified it in the code.
3. Link to the feature doc where the handling actually lives, if
   relevant.

The edge case reference ages. Keep it alive with the code — otherwise it
becomes a trap.
