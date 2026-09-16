# Feature Landscape — v2.6 Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter

**Domain:** HR / Time-Tracking — voluntary-vs-paid-hours reconciliation for capped employees
**Researched:** 2026-07-06
**Mode:** Ecosystem (project-internal — the "ecosystem" here is the existing Shifty backend + established GSD milestone patterns)
**Overall confidence:** HIGH (all upstream services and DTOs read from repo; industry patterns for reconciliation batches well-established, cross-checked against Phase 22 AVG-01 + v1.7/v2.4 ToggleService precedent)

---

## Executive summary

v2.6 introduces a **five-feature reconciliation stack** on top of already-shipped primitives:

- v1.4 shipped `EmployeeWorkDetails.committed_voluntary` (time-versioned pledge, `f32`, gated on `cap_planned_hours_to_expected`).
- v1.5 shipped the HR-only employee-year report `/employees/:sales_person_id` with balance rows (UV-04/05, YV-01..03).
- v2.1 shipped AVG-01 as **pure read-aggregate in `ReportingService`** with HR-gate + `is_dynamic` filter (the direct template for F1).
- v2.2 shipped RPT-01..03 (per-weekday attendance stats) — replaced v2.1 AVG-01 in the FE but kept the compute pattern; **F1 rides on this precedent again**.
- v2.5 shipped VAA-01..04, wiring the "cap-gated volunteer sees committed" formula (`filter(cap || expected == 0).map(committed_voluntary)`) at two fill-sites in `booking_information.rs`.
- v1.7 (HCFG-02), v2.4 (SHC-04) shipped `ToggleService`-gated admin cutoff dates — v2.6 will reuse this for the F4 cron rollout stichtag.
- v2.2 (EXP-01) shipped a weekly `tokio_cron_scheduler` job with admin-gated on/off + last-error surface in Admin panel — F4 rides on the same infrastructure.

**Net new work in v2.6:**
- Two new tables: `rebooking_batch` (parent) + `rebooking_batch_entry` (children per SalesPerson), unified for F4 (auto-cron) and F5 (HR suggestion) via a `kind` discriminator.
- One new **Business-Logic Service** `RebookingService` that assembles ExtraHours-pair-transactions and owns batch lifecycle.
- Extension of `ReportingService` (F1) and the employee-year DTO (F2, F5-account fields).
- One new REST family under `/rebooking-batch/*` (list/approve/reject) and `/employees/:id/voluntary-account` (read-model).
- Backfill CLI command for retroactive F4 processing at rollout.

**No new Cargo dependency required.** All primitives (transactions, cron, toggles, ExtraHours pair-writes, HR gate) are already in the codebase.

**Snapshot-Schema-Version:** likely **stays at 12** — F1/F2/F5 are read-aggregates; F3/F4 write to the existing `extra_hours` table (already versioned inside snapshot). *Only* a bump would be needed if we choose to persist a new `BillingPeriodValueType` for the account balance (e.g. `VoluntaryCommitmentDelta`) — **default = no**, discuss-phase should confirm the "no persist" path (matches v1.4 CVC-05 precedent: derive-on-read wins).

---

## Table stakes

Features users **expect** as soon as the domain acknowledges "voluntary commitment" as a first-class concept. Missing = the reconciliation story is incomplete.

| Feature | Why expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **F1 — Ø freiwillig geleistete Stunden / Vertragswoche / Jahr** | Every time-tracking tool that surfaces a pledge (`committed_voluntary`) has to also surface its *realization*. Users can already see AVG-01/RPT-01 for paid attendance; they will immediately ask for the mirror on voluntary. | Low | Read-aggregate on `ReportingService`, HR-only. Numerator: Σ ExtraHours(VolunteerWork).hours in period. Denominator: **weeks with a valid `working_hours` row for the person** (D-F1-01 to fix — probably matches AVG-01's "weeks with contract" nenner). No persistence. |
| **F2 — Freiwilliges Stundenkonto (Soll / Ist)** | Once F1 exists ("here's your average") the immediate follow-up is "here's your promise vs. reality". Industry: every overtime-account / vacation-balance widget in HRIS tools (BambooHR, Personio, Factorial) shows Soll/Ist next to each other. | Low | Soll = Σ over calendar-year of `working_hours(week).committed_voluntary` (i.e. weekly pledge × contract-weeks). Ist = F1 numerator. Account = Ist − Soll. **HR-only** (matches CVC-06 no-leak precedent). Rendered in existing employee-year report next to the paid balance. |
| **F3 — Manuelle Umbuchung Freiwillig ↔ Bezahlt (1-click)** | HR today does this by hand as two ExtraHours entries. The tool has to atomically write both or neither — a partial write breaks CVC-06 gating and balance semantics. 1-click is the whole point. | Medium | Single REST endpoint accepts `{ sales_person_id, hours, source_category, target_category, date }`, opens transaction, writes two `ExtraHours` rows (`-N source / +N target`), commits. Standard `ServiceError`. FE modal in employee-year report next to the account row. |
| **F5 — HR-Alert bei gedeckeltem Mitarbeiter mit negativem Konto** | Without a proactive alert, F2 is a passive stat that HR forgets to check. Every payroll/overtime system with reconciliation ships a "you need to act on this" banner. | Medium | Persistent warning row in the employee-overview list (top-level `/employees`). Condition: `sales_person.cap_planned_hours_to_expected == true` AND balance_account (paid) < 0 AND voluntary_account (F2) > 0 (there's something to rebook). Click opens **suggestion modal**: shows IST (current balance, F1 Ist, F2 Soll, F2 Konto) vs. DANN (post-rebooking projection). HR approves/rejects. Approve → persist as `rebooking_batch` with `kind = 'hr_suggestion'`, one entry per person. Reject → persist rejection state (audit trail). |

**F1 dependency chain:**
- Authoritative source for numerator: **`ExtraHoursService`** (already exposes `find_by_iso_year`, added in v2.5 Follow-up #3 for the boundary fix). Sum over category `VolunteerWork`.
- Authoritative source for denominator: **`EmployeeWorkDetailsService`** (weeks with a `working_hours` row for the person in the year). AVG-01 formula A-22-1 is the reference — Phase 22 pinned "Jahr bis heute; worked = shiftplan+extrawork+volunteer; full-absence weeks removed".
- Live in **`ReportingService`** (Business-Logic tier) as new pure fn `average_voluntary_hours_per_contract_week(sales_person_id, year)`, analogous to `average_hours_per_attendance_day`.
- **DTO extension:** `EmployeeReportTO` / attendance-statistics response gets one new field `avg_voluntary_hours_per_contract_week: f32` (additive, `#[serde(default)]` for legacy-wire compat — the same guard used in v2.5 VAA-01).

**F2 dependency chain:**
- Same DTO extension gets: `voluntary_account: { soll: f32, ist: f32, delta: f32 }` (or three flat fields).
- Soll compute: iterate `working_hours` rows in the report period, sum `committed_voluntary × weeks_covered_by_this_row` (same "period-slicing" pattern as `vacation_days_for_year` and CVC-04).
- Ist compute: exactly the F1 numerator (share the pure fn).
- **Renders in the existing `/employees/:sales_person_id` report** — no new page, just a new sub-block under the paid-balance block. Presedence: v1.5 YV-01..03 added new sub-blocks to the same page.

**F3 dependency chain:**
- Authoritative source: **`ExtraHoursService`** already handles create + delete under a `TransactionDao` (transaction-management pattern). F3 needs a new business-logic method `rebook_hours(sales_person_id, hours, source: ExtraHoursCategory, target: ExtraHoursCategory, date, reason, tx)` — either on `ExtraHoursService` itself or on a new `RebookingService` (Business-Logic tier).
- **Recommended: put F3 on the new `RebookingService`** because F4 + F5 both need it and both wrap it in a batch. Keeps single Rebooking-write-path, single audit-log source.
- FE: modal in employee-year report, dropdowns for source/target category (from `ExtraHoursCategory`), hours input, one confirm button.

**F5 dependency chain:**
- Read side rides on F2's account math (same balance formulas).
- Trigger: employee-overview list (top-level `/employees`) already shows all sales-persons — becomes overview-with-badge. Backend supplies `Vec<VoluntaryReconciliationAlertTO>` (or extends the existing list DTO with an optional field).
- Write side: HR approve → `RebookingService::persist_hr_suggestion(sales_person_id, entries, tx)` writes one `rebooking_batch(kind='hr_suggestion', state='approved')` + N `rebooking_batch_entry` rows + calls F3's `rebook_hours` in the same tx. Reject → same batch row but `state='rejected'`, no ExtraHours writes.
- **Read model for pending batches:** `GET /rebooking-batch?state=pending&kind=hr_suggestion` for a future queue view (not in v2.6 core scope, but the schema supports it).

---

## Differentiators

Features that raise the reconciliation story from "manual with tool assist" to "self-driving". Not expected — but each one meaningfully reduces HR toil.

| Feature | Value proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **F4 — Automatische Umbuchung (Wochen-Cron)** | Turns a manual reconciliation from "HR checks weekly, HR clicks per person" into "HR reviews a weekly summary". Real productivity multiplier. Nobody expects this on day one but everyone loves it once shipped. | High | Weekly cron (Vorwoche = last completed ISO week). For each capped person: compute `Ist_last_week = Σ VolunteerWork(hours) in ISO-week`, `Soll_last_week = working_hours(week).committed_voluntary`. If `Ist > Soll + committed_voluntary` (i.e. Überschuss über Zusage), rebook the excess `excess = Ist − (Soll + committed_voluntary)` per week as pair-ExtraHours in one batch (`kind='auto_cron'`). Batch persistence groups all persons in one commit per run. |
| **F4 rollout backfill CLI** | Without backfill, F4 only helps *going forward* — historical over-delivery stays broken. Standard practice for any "cutoff-date" feature (HCFG-02/SHC-04 precedent). | Medium | New Cargo binary or `shifty_bin` subcommand `backfill-voluntary-rebooking --from <date> --to <date> --dry-run`. Same F4 compute path, iterates historical ISO-weeks. Dry-run outputs what *would* be booked; live run persists as `kind='auto_backfill'`. Ships with rollout of the cron. |

**F4 dependency chain:**
- Cron scaffolding: **v2.2 `tokio_cron_scheduler`** (already registered in `shifty_bin/src/main.rs` for the WebDAV/PDF export). Add a second job — matches v2.2 precedent one-to-one.
- Gate: new `ToggleService` toggle `voluntary_auto_rebooking_active_from` (ISO date, precedent HCFG-02 v1.7 + SHC-04 v2.4). Off (unset) = cron doesn't run. Set to a date = cron runs and only processes weeks with `booking_date >= active_from`.
- Compute path: same pure fn as F1 (Ist per week) + `EmployeeWorkDetailsService` (Soll per week). Reuse — do not duplicate.
- Write path: `RebookingService::persist_auto_cron_run(entries, run_id, tx)` — same table as F5, `kind='auto_cron'`, `state='applied'` (no HR gate on auto-runs).
- **Idempotency:** F4 must not double-book if the cron runs twice for the same week. Enforce via **unique index** on `rebooking_batch_entry(sales_person_id, iso_year, iso_week, kind)` — a re-run for the same key is a no-op (SKIP existing). Aligns with atomicity precedent (feedback_atomic_repoint_no_double_count).

**F4 rollout backfill CLI:**
- Runs *once* at rollout, then archived. Same compute + persistence as F4.
- Idempotency guard: the unique index above catches double-inserts even if the CLI is run twice. Dry-run mode required (precedent: milestone tooling).

---

## Anti-features (explicit non-goals)

Explicit "we will NOT build this in v2.6" list, to prevent scope creep. Every one of these will get asked in discuss-phase; the answer is prepared.

| Anti-feature | Why avoid | What to do instead |
|--------------|-----------|--------------------|
| **Retro-editing existing ExtraHours as "part of a rebooking batch"** | The rebooking-batch model is prospective (batches wrap **new** ExtraHours writes). Retro-linking old orphan rows would require a schema-level foreign-key or a repair CLI — huge scope, low value. | Pre-v2.6 ExtraHours stays orphan (no batch). Reports treat them exactly as today. |
| **Employee-self-service view of the voluntary account** | Every existing voluntary/committed-related surface is HR-only (CVC-06 precedent). The account exposes commitment-vs-realization drift, which is inherently sensitive. | HR-only across F1, F2, F5. Add a self-view later as a separate milestone with a discuss-phase on privacy semantics. |
| **Notifications (email / iCal / push) on F4 completion or F5 alert** | v2.6 has no notification channel infrastructure. Adding one is a milestone of its own. | Alerts are visible in the app only. F4 writes to a log line + surfaces in Admin-panel status (analog EXP-03). |
| **Approval workflow / multi-role sign-off on F5 batches** | PROJECT.md notes: "Backend kennt heute keinen Approval-Schritt" (v1.4 out-of-scope). Two-step approval multiplies REST endpoints and state machine complexity. | Single-step HR approve/reject. Persist `approved_by = <HR user id>` + `approved_at` on the batch for audit only. |
| **Undo/rollback of an applied batch** | Batch is a series of ExtraHours pair-writes. Undoing means writing inverse pairs — doable, but multiplies REST surface + UI complexity. High probability of drift bugs. | For v2.6, undo is manual (HR uses the existing ExtraHours-editor to delete rows one by one). Design the schema so a future undo can be added (batch has a stable id; entries reference it — reverse-pair write is straightforward if ever needed). |
| **F4 compute over full year or over arbitrary range** | Weekly windowed compute is O(1) per person per run. Full-year every week is O(52) per person per run × cron cadence — waste. Also would double-book on partial-week edits. | F4 processes exactly last completed ISO week. Backfill CLI handles historical range. |
| **UI to visualize the rebooking-batch history** | Adds a whole new screen. Every HR system has one eventually, but nobody needs it on day one — the ExtraHours-editor already shows the resulting pair rows. | Deferred to v2.7+. Schema (batch table + entries) supports a queue view without migration. |
| **Alert on non-capped employees with voluntary Überschuss** | F5's alert is about *capped* employees whose paid balance is stuck at 0 (by cap) while they over-delivered voluntarily — that's the exact user-value case. Non-capped employees just get overtime paid normally; no rebooking needed. | Alert is gated on `sales_person.cap_planned_hours_to_expected == true`. Non-capped employees see nothing extra. |
| **F4 as source-of-truth for the voluntary account** | If F4 alters the ExtraHours ledger AND the account math reads from that ledger, the account trivially reads zero after F4 runs — which is *correct semantically* but requires the frontend to also show "was rebooked" separately. Doing that inline in v2.6 doubles the state model. | Account math reads from the raw `VolunteerWork` ExtraHours (pre-rebook), so the account keeps meaning across F4 runs. Rebooking writes cancel out **paid balance** minus, not voluntary Ist. Discuss-phase must confirm this: **the account is a live "not yet reconciled" figure**, and rebook operations shrink it. This is the semantic that matches how HR thinks about it. |

---

## Feature dependencies

```
                             ┌─────────────────────────────────────────┐
                             │  Existing (already shipped v1.4..v2.5)  │
                             │                                         │
                             │  committed_voluntary (EmployeeWorkDet.) │
                             │  ExtraHoursService + VolunteerWork cat. │
                             │  ReportingService (AVG-01 / RPT-01)     │
                             │  ToggleService (HCFG-02 / SHC-04)       │
                             │  tokio_cron_scheduler (EXP-01)          │
                             │  HR gate + is_dynamic filter            │
                             └─────────────────────────────────────────┘
                                              │
                                              │ new builds on
                                              ▼
                                    F1 (Ø voluntary / week)
                                              │
                                              │ shares pure fn
                                              ▼
                                    F2 (Soll/Ist/Konto)
                                              │
                                              │ account math feeds
                                              ▼
                          ┌──────────────┴──────────────┐
                          ▼                             ▼
                    F3 (manual 1-click)          F5 (HR alert + modal)
                          │                             │
                          │ pair-write primitive        │ writes batch entries
                          │                             │ that call...
                          ▼                             ▼
                    RebookingService (new)  ──────────────┐
                          ▲                             │
                          │                             │
                          │ writes same batch schema    │
                          │                             │
                    F4 (weekly cron)  + rollout backfill CLI
```

**Ordering implications for the roadmap:**

1. **First phase:** F1 + F2 read-side + DTO. Cheapest, unlocks F3/F5 UI immediately. No new tables yet.
2. **Second phase:** Table migration (`rebooking_batch` + `rebooking_batch_entry`) + `RebookingService` skeleton + F3 (manual pair-write). Isolated, testable, ships even if F4/F5 slip.
3. **Third phase:** F5 (HR alert + suggestion modal + approve/reject persistence) — needs F3's `rebook_hours` primitive.
4. **Fourth phase:** F4 (weekly cron + backfill CLI + toggle stichtag). Most complex; ships last so F1..F3+F5 can be verified against manual reality first.

This order also matches the risk profile: F1/F2 are pure read (near-zero regression risk), F3 is bounded pair-write (transaction guarantees), F5 is UX-heavy but read-safe once state machine settled, F4 is the highest-blast-radius change and benefits from all previous work being validated.

---

## MVP recommendation (if scope has to shrink)

If the milestone needs to be de-risked, ship **F1 + F2 + F3** as the MVP:

- HR sees the account (F1/F2).
- HR can act manually (F3).
- Value delivered end-to-end without cron or auto-batch semantics.
- No new toggle, no cron, no rollout stichtag, no backfill CLI.
- Snapshot version stays 12 (guaranteed — no persistence changes beyond the additive ExtraHours pair-writes F3 already uses).

Defer F4 + F5 to v2.7 — the schema (batch tables) can either be pre-created in v2.6 or deferred to v2.7 as well.

**Recommendation:** Do NOT pre-create the tables if F4/F5 slip. Empty tables are drift-fuel; ship them when they're used.

---

## Cross-milestone dependencies (traceability)

| v2.6 feature | Depends on shipped feature | Source-of-truth service | New DTO extension |
|--------------|---------------------------|-------------------------|-------------------|
| F1 | AVG-01 (v2.1) / RPT-01 (v2.2) pattern; `ExtraHoursService::find_by_iso_year` (v2.5) | `ReportingService` | `EmployeeReportTO.avg_voluntary_hours_per_contract_week` (additive, `#[serde(default)]`) |
| F2 | `committed_voluntary` (v1.4 CVC-01/02); AVG-01 nenner (v2.1); `EmployeeWorkDetailsService.get_by_year`-style batch (v2.5 Follow-up #3 pattern) | `ReportingService` + `EmployeeWorkDetailsService` | `EmployeeReportTO.voluntary_account: { soll, ist, delta }` (additive) |
| F3 | `ExtraHoursService::create` under existing `TransactionDao` | new `RebookingService` (Business-Logic tier) | new `RebookHoursRequestTO` (POST body) |
| F4 | `tokio_cron_scheduler` (v2.2 EXP-01); `ToggleService` stichtag (v1.7 HCFG-02); F3 pair-write primitive | new `RebookingService` + `Scheduler`-registration | none FE-facing (backend log only); new REST GET on batches for future UI |
| F5 | F2 account math; F3 pair-write; existing employee-overview list | new `RebookingService` | new `VoluntaryReconciliationAlertTO` + new `RebookingSuggestionTO` (approve/reject payload) |

---

## Confidence

**HIGH** on:
- Existing service/DTO map (read from repo directly, cross-checked against PROJECT.md + v2.5 REQUIREMENTS).
- Pattern precedent for each feature (AVG-01, RPT-01, VAA-01, HCFG-02, SHC-04, EXP-01 — all quoted with concrete file/phase references).
- Snapshot-version staying at 12 (matches CVC-05 revised decision from v1.4 — Achse-B / derive-on-read wins).

**MEDIUM** on:
- Exact denominator definition for F1 (AVG-01's "weeks with contract, absence-adjusted" vs. simpler "contract weeks in year"). **Decide in discuss-phase**, precedent A-22-1 is the reference.
- Whether to persist a new `BillingPeriodValueType` for the voluntary account. Default: NO. Discuss-phase should confirm.
- F4 idempotency exact key shape (`(sales_person_id, iso_year, iso_week, kind)` vs. `(sales_person_id, iso_year, iso_week)` — the latter lets HR-suggestion supersede an auto-cron; the former makes them independent). **Decide in discuss-phase.**
- F5 IST→DANN modal formula for the "DANN" column — is it the account after applying the exact excess-per-week from F4, or a summed one-shot? Recommend: match F4's per-week excess semantics so manual F5 reconciles the same way F4 would.

---

## Sources

- `.planning/PROJECT.md` (v2.6 charter section, line-verified 2026-07-06)
- `.planning/milestones/v1.4-REQUIREMENTS.md` (committed_voluntary, CVC-05 no-bump precedent)
- `.planning/milestones/v1.5-REQUIREMENTS.md` (employee-year report structure, YV-01..03, STAT-01/02, A-22-1 formula)
- `.planning/milestones/v2.1-REQUIREMENTS.md` (AVG-01 read-aggregate pattern, HR gate + is_dynamic filter, "no snapshot bump" precedent)
- `.planning/milestones/v2.2-REQUIREMENTS.md` (RPT-01..03 replaces AVG-01 in FE; EXP-01/02/03 for `tokio_cron_scheduler` + admin gate)
- `.planning/milestones/v2.5-REQUIREMENTS.md` (VAA-01..04 additive-DTO precedent, `#[serde(default)]` legacy-wire-guard, D-53-02 formula for cap-gated volunteer sum, `find_by_iso_year` pattern)
- `service/src/extra_hours.rs` (ExtraHoursCategory enum: `ExtraWork`, `Vacation`, `SickLeave`, `Holiday`, `Unavailable`, `UnpaidLeave`, `VolunteerWork`, `Custom(id)`)
- `service_impl/src/` layout (existing `RebookingService` slot to fill — no naming conflict with existing services)
- `feedback_atomic_repoint_no_double_count` (project MEMORY.md — atomicity + re-point-tests are precedent for F3/F4/F5 write paths)
- `feedback_stichtag_rollout_legacy_semantics` (project MEMORY.md — F4 toggle stichtag needs pre-stichtag semantic reconstruction, matches HCFG-02 shape)

**Confidence tags** on individual claims are inline; overall document = HIGH except where the four MEDIUM discuss-phase decisions above are called out.
