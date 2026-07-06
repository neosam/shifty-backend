# Pitfalls Research — v2.6 Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter

**Domain:** Adding voluntary-hours reconciliation (F1–F5) to Shifty's existing payroll-adjacent balance/reporting stack (Rust/Axum/SQLite, snapshot-versioned billing periods, range-based absences, weekly cron infrastructure).
**Researched:** 2026-07-06
**Confidence:** HIGH — derived from direct inspection of existing precedents (HCFG-02 Stichtag, VAC-OFFSET-01 HR-only redaction, VFA-01 whole-week-out, CVC-06 cap gating, WOP-Follow-up #3 ISO/Gregorian bug), `.planning/PROJECT.md`, `CLAUDE.md` snapshot-bump contract, `v1.7-REQUIREMENTS.md` HCFG-02, `v2.5-REQUIREMENTS.md` VAA cap-gated formula, `v2.2-REQUIREMENTS.md` EXP WebDAV cron precedent.

The pitfalls below are grouped by severity. Every one is voluntary-reconciliation-specific — none are generic "Rust deadlock" boilerplate. Each names existing precedents the planner should reuse, and calls out which Requirement should exclude the trap and which Success Criterion should verify it.

---

## Critical Pitfalls

### Pitfall 1: Doppel-Zählung — F4-Rebooking-ExtraHours landen zusätzlich im F1-Ist und/oder im F2-Soll-Nutz

**What goes wrong:**
F4 automatically writes two `ExtraHours` rows per rebooking: `−N Kategorie VolunteerWork` und `+N Kategorie ExtraWork` (or similar). Downstream aggregates that scan `ExtraHours`:
- **F1 Ist-Statistik** (`Σ VolunteerWork hours / weeks_with_working_hours`): if the F4 `−N` correction row is *not* excluded, the average drops artificially every time HR "rescues" a capped employee — a person who legitimately worked 8h voluntary then rebooked 4h looks like they only did 4h.
- **Balance / reporting.rs**: the existing balance formula sums extra_hours by category; if F4 shifts hours between two categories that both feed into `overall_hours`, the balance change happens twice (once via `−N` and once via `+N` of a different category with a different sign convention).
- **F2 Soll-Nutz-Summierung**: F2's "committed_voluntary × Vertragswochen" is the Soll side. If F2 accidentally computes Ist by summing `VolunteerWork` extra_hours (including the F4 correction rows), the "verbleibendes Freiwilliges Konto" widens each time HR reconciles — the very act of "using" volunteer credit destroys the record of it having been given.

**Why it happens:**
`ExtraHours` is a single write-once ledger with a single `Category` enum. The v1.4 CVC precedent already showed this pattern is brittle: `committed_voluntary` was intentionally kept as a *separate* term on `EmployeeWorkDetails` (Variante B) precisely to avoid the trap of computing it via `extra_hours` sums. F4 breaks that decoupling if it stores rebooking deltas in the same ledger without a distinguishing marker. Precedents:
- v1.5 UV-01..05 double-count fix (Snapshot bump 9→10) — the derived-absences / by_week fix cost a snapshot version bump because `vacation_days` was silently being counted twice after extra_hours→Absence conversion.
- The v1.7 HOL-03 / VFA-02 asymmetry (holidays touch balance but not committed 🎯) is documented as a *regression guard* — with a CI test — because balance-formula authors reflexively forget which subset feeds each aggregate.

**How to avoid:**
- Store F3/F4/F5 rebookings in a **dedicated `rebooking_batch_entry` table** with a foreign key to `rebooking_batch(kind)`, and add a **join-only column** `extra_hours.rebooking_batch_entry_id: Option<i64>` (or a `source: TEXT NOT NULL DEFAULT 'manual'` marker column). Every downstream aggregate — `reporting.rs::balance`, `booking_information.rs::get_weekly_summary`, F1 statistic, F2 Nutz-Summierung — MUST explicitly filter this marker.
- Add three property tests up front, each of which mutates only via a rebooking API and asserts:
  1. `balance_before == balance_after` (rebooking is balance-neutral — this is the *definition* of a rebooking).
  2. `sum(F1_Ist)_before == sum(F1_Ist)_after` (rebooking preserves the historical volunteer record).
  3. `F2_Soll_Nutz_before == F2_Soll_Nutz_after` (rebooking does not artificially widen the freiwilliges Konto).
- Reuse the v1.5-Snapshot-9→10 lesson: single source of truth for each aggregate + explicit `by_week` split. Do **not** let two code paths compute F1 differently.

**Warning signs:**
- Balance drift in existing golden-snapshot fixtures (WOP-03 style, 8 fixtures) after touching `reporting.rs` or `booking_information.rs`.
- Any F4/F5 test that has to "compensate" or "reset" `ExtraHours` before asserting F1/F2 — that's a smell the aggregate isn't filtering rebookings out.
- Test cases where "run F4 twice" changes user-visible numbers (idempotency is broken → double-count is happening).

**Prevention:** Requirement should include a **Doppel-Zählungs-Ausschluss-Regel** ("F1 Ist-Statistik zählt `source != 'rebooking'` ExtraHours, F4/F5 Rebooking-Deltas sind für F1 unsichtbar; äquivalent für F2 Soll-Nutz"). Success Criterion: three property tests above, plus **golden-snapshot re-run** across all 8 existing WOP-03 fixtures (must remain byte-identical if no rebookings exist).

**Which Phase:** The dedicated table + marker column MUST be introduced in the *first* backend phase (data-model), before F1/F2/F3/F4/F5. Any later phase that touches an aggregate MUST prove the filter is in place.

---

### Pitfall 2: Snapshot-Vertragsbruch — F4 verändert die Balance-Formel-Input-Set ohne Bump 12→13

**What goes wrong:**
`build_and_persist_billing_period_report()` reads `ExtraHours` at write time and materialises per-value_type totals into `billing_period_sales_person`. If F4 runs a weekly cron that writes rebooking `ExtraHours` rows **between** the reporting week and the eventual billing-period-close, the persisted snapshot values will disagree with a fresh re-computation — a validator running the live formula against an old snapshot will see drift indistinguishable from a real data bug. `CLAUDE.md` § "Billing Period Snapshot Schema Versioning" is explicit: any change to the input set of an existing `value_type` REQUIRES a bump. F4 introduces exactly that: a new source of ExtraHours rows that feeds existing value_types (`extra_hours_by_category`, `overall_hours`, `balance`).

**Why it happens:**
The instinct is "F4 just automates what HR does manually — no new value_type, no bump." That reasoning **is wrong** if either:
1. Historical billing periods (before F4 rollout) would have looked different under the new logic → snapshot disagreement.
2. A new persisted value_type (e.g., `voluntary_reconciled_hours` for auditor traceability) is added.

The v1.5 precedent (UV/YV, Snapshot 9→10) bumped for exactly this reason — the derived-absences merge changed the input set for `vacation_days`. The v1.8 precedent (VAC-OFFSET-01, Snapshot 11→12) bumped for `VacationEntitlement`. Missing a bump makes historical validators lie.

**How to avoid:**
- **Stichtag guard (see Pitfall 3)**: F4 only runs for weeks with `booking_date >= voluntary_rebooking_active_from` (Toggle-Wert). Snapshots strictly before the Stichtag remain reproducible under old rules; snapshots strictly after use the new rules. Snapshots *straddling* the Stichtag are the danger zone.
- **Decide in discuss-phase**: does F4 write a new `BillingPeriodValueType` (e.g., `VoluntaryReconciled`)? If YES → **Bump 12→13**, migration adds the column, `CURRENT_SNAPSHOT_SCHEMA_VERSION` incremented, docs `F08-billing-period.md` + `_de.md` + `docs/domain/billing-period.md` synced (Docs-Freshness-Gate).
- **If NO new value_type**: still audit whether the Stichtag-guarded change of `reporting.rs`-input-set (F4 rebookings are visible to `balance`) triggers `CLAUDE.md` clause "Change the input set the computation reads from" — YES → **Bump 12→13 anyway**.
- Golden-snapshot regression fixtures MUST cover a period straddling the Stichtag (analog to v2.4 SHC-04 `shortday_gate.active_from` on/off fixtures).

**Warning signs:**
- Existing golden-snapshot tests still green but a fresh `build_and_persist_billing_period_report()` for a historical period produces different numbers.
- Reviewer question "sollten wir bump?" left unanswered in Plan or DISCUSS.
- No new fixture that crosses the F4 Stichtag.

**Prevention:** Requirement should include a **Snapshot-Impact-Entscheidung** (Yes/No + Justification, per Präzedenz v1.7 HSNAP-01). Success Criterion: "Snapshot version documented: bumped or explicitly not bumped with justification in phase's SUMMARY.md; docs `F08-billing-period.md` + `_de.md` synced if bumped." Cross-check against `service_impl/src/billing_period_report.rs::CURRENT_SNAPSHOT_SCHEMA_VERSION`.

**Which Phase:** Decision belongs in the F4 discuss-phase. Bump commit belongs in the same phase that lands F4 (v1.8 precedent: VAC-OFFSET-01 bump + logic in same phase).

---

### Pitfall 3: Stichtag-Rollout mit falscher Legacy-Semantik pro Chain

**What goes wrong:**
F4 (and possibly F3/F5) needs an admin-configurable `voluntary_rebooking_active_from: Option<Date>` toggle (analog HCFG-02 for holiday auto-crediting, SHC-04 `shortday_gate.active_from`, D-51-07). Without one, historical Balance-Views suddenly re-interpret past weeks — HR opens a 2024 employee report and sees numbers that changed because F4-logic now applies retroactively. Users perceive this as a data-corruption bug.

But the failure mode captured in MEMORY (`feedback_stichtag_rollout_legacy_semantics.md`) is subtler: when the toggle is *off* or the date is *before Stichtag*, the code must reconstruct **the pre-feature legacy semantic per consumption chain** — not blindly return raw input. Precedents:
- v1.7 HCFG-01/02/03: pre-Stichtag, `special_day` holidays are visible in the settings UI but NOT auto-credited in reporting. Blindly "None → raw" would have surfaced `special_day` rows to a downstream aggregate that had never seen them before.
- v2.4 SHC-04 `shortday_gate.active_from`: Chain A' (BlockService) + Chain B (WeekView/PDF) + Chain C (BookingInformation) + Chain D (ShiftplanReport) each had a legacy semantic; all four had to be threaded through the toggle, and Chain C in particular had a "legacy filter" branch that was NOT identical to Chain B pre-Stichtag.
- D-51-07 (Kurzer-Tag-Slot-Kürzung): the toggle-off branch had to reproduce the exact pre-clipping behaviour for each of four chains, not the naive "no clipping".

For F4, the affected chains are at least:
- **Balance-Chain (reporting.rs)**: pre-Stichtag → balance ignores F4 rebooking entries entirely.
- **F1-Ist-Statistik**: pre-Stichtag → F1 ignores F4 rebooking entries (they didn't exist as a concept then).
- **F2-Soll/Nutz-Anzeige**: pre-Stichtag → F2 shows *only* the Soll and the raw Ist without Nutz-column. Or hide entirely.
- **Cron F4 self-guard**: cron MUST NOT process weeks before Stichtag (backfill command may — but only intentionally).

**Why it happens:**
Toggles look like a single boolean, but each aggregate has its own pre-feature invariant that must be preserved to avoid breaking historical golden snapshots. Author usually threads the toggle into one chain and forgets the others.

**How to avoid:**
- **Chain audit** in the F4 planning phase: enumerate every code site that reads `ExtraHours` or writes to the balance formula, and specify per-site the toggle-off semantic. Precedent: v2.4-Phase-51 Chain A'/B/C/D matrix.
- Toggle via existing `ToggleService` (`voluntary_rebooking_active_from: Option<Date>`), consumed via **`Full` context bypass** where internal aggregates (reporting, booking_information) don't have per-user auth (see `reference_toggle_service_full_context_bypass.md` MEMORY).
- Property test: for a fixed dataset, `active_from = None` produces byte-identical result to a v2.5 snapshot (WOP-03-style fixture) — i.e., no rebookings applied.
- Property test: `active_from = 2026-08-01` applies rebookings only for weeks with `booking_date >= 2026-08-01` — pre-Stichtag weeks unchanged.

**Warning signs:**
- Any code branch that says "if toggle off → don't do anything special" without an explicit test that pre-Stichtag numbers match a saved baseline.
- A single `if let Some(cutoff)` check in only one file — usually means the other chains are missing it.

**Prevention:** Requirement should include an **HCFG-analog** ("voluntary_rebooking_active_from Toggle, pro Chain identifizierbare Legacy-Semantik, default = None → Feature aus"). Success Criterion: byte-identical golden-snapshot re-run with `active_from = None` (analog v2.4 SHC-04). Reference existing precedents HCFG-02, SHC-04, D-51-07 in the requirement text.

**Which Phase:** Introduce the toggle + settings UI in the **first** F4 phase, before any rebooking logic ships. Precedent: v1.7 introduced HCFG-02 in Phase 25 before HOL-01 auto-crediting used it in the same phase.

---

## Moderate Pitfalls

### Pitfall 4: Cron-Idempotenz — mehrfaches Ausführen der gleichen Vorwoche schreibt N-fach

**What goes wrong:**
The weekly cron restarts (server restart, systemd unit restarts, cron overlap on slow DB). It iterates every SalesPerson, computes "IF Ist > Soll + committed_voluntary → book auto-rebooking". Without a per-(SalesPerson, ISO-Woche) processed-marker, each run creates fresh `rebooking_batch_entry` rows and doubles the correction. The v2.2 EXP-03 WebDAV cron precedent gives a partial guide (`repeated upload on transient failure`) but the destination there (WebDAV file) is naturally idempotent; the destination here (ExtraHours ledger) is NOT.

**Why it happens:**
Cron-Job authors default to "just re-run" because idempotence is rarely the top-of-mind failure mode. Backfill command amplifies this — running `backfill --from 2024-01 --to 2026-06` after F4 has already processed some weeks live silently duplicates.

**How to avoid:**
- `rebooking_batch` table: UNIQUE index on `(kind, sales_person_id, iso_year, iso_week)` where kind=`auto_cron`. INSERT ... ON CONFLICT DO NOTHING pattern. Cron creates the batch row **first**; if the insert conflicts, skip this week for this person. Only after successful batch insert are `rebooking_batch_entry` + `ExtraHours` written, all in one transaction.
- Backfill command reads the same UNIQUE constraint and reports "N weeks already processed, M would be processed" as a dry-run before writing.
- Test: run cron twice back-to-back over the same fixture; assert exactly one `rebooking_batch` per (person, week).
- Test: run cron, then backfill over same range → backfill is no-op.

**Warning signs:**
- Balance drift after `systemctl restart shifty`.
- Backfill command shows "processed 500 weeks" the first time, "processed 500 weeks" the second time.
- `rebooking_batch_entry` row count > 1× total weeks × capped-employee count.

**Prevention:** Requirement should include **Cron-Idempotenz-Contract** ("F4-Cron is idempotent per (sales_person_id, ISO-Woche, kind=auto_cron); running twice produces at most one rebooking_batch row"). Success Criterion: dedicated test asserting UNIQUE-conflict behaviour.

**Which Phase:** In the same phase that introduces `rebooking_batch` schema — the UNIQUE constraint IS the idempotency mechanism.

---

### Pitfall 5: ISO-Woche vs. Kalender-Woche im Cron-"Vorwoche"-Semantik

**What goes wrong:**
The cron runs weekly and processes "die Vorwoche". If "Vorwoche" is computed via `date - 7.days().calendar_week()`, the year boundary triggers the exact `paid_hours/required_hours`-Drift KW 1/KW 53 bug that WOP-Follow-up #3 fixed in v2.5 with `_iso_year`-Varianten. `booking(year, calendar_week)` is ISO — the cron MUST use ISO-week arithmetic (`time::util::weeks_in_year`, `Date::to_iso_week_date()`), NOT `Date::year() + Date::iso_week()` naively.

**Why it happens:**
`time::Date::iso_week()` returns a `u8` but the corresponding year is `iso_week_year()`, NOT `year()`. Around Jan 1, these disagree. The v2.5 WOP fix cost 16 new regression gates in `reporting_year_boundary.rs` + `booking_information_weekly_summary_year_boundary*.rs`.

**How to avoid:**
- All F4 cron code uses `ShiftyWeek::previous()` semantic derived from `ShiftyDate` (analog to what `booking(year, calendar_week)` already uses).
- Reuse the WOP-Follow-up #3 test fixtures — extend them to cover F4 cron scheduled on Jan 1 processing "the previous ISO week" that lives in the previous ISO year.
- Test: cron runs on Mon 2027-01-04 → processes ISO week (2026, 53). Cron runs on Mon 2027-01-11 → processes (2027, 1).

**Warning signs:**
- Any use of `chrono::Datelike::year()` or `time::Date::year()` in cron time arithmetic — should be `iso_week_year()`.
- Missing test around Jan 1 / Dec 31 boundary.

**Prevention:** Requirement should call out **ISO-Wochen-Semantik pro WOP-Follow-up #3 Präzedenz**. Success Criterion: regression test at year boundary.

**Which Phase:** F4 phase; ideally reuse existing `_iso_year` helpers from v2.5 rather than rolling new arithmetic.

---

### Pitfall 6: TOCTOU zwischen Cron und Manual-F3

**What goes wrong:**
HR clicks F3 "Manual rebooking for week 2026-W27" at 03:00:00. The weekly cron starts at 03:00:00 for the same person, same week. Both:
1. Read `Ist` and `Soll` for the week.
2. Both compute "overage exists".
3. Both write rebooking rows → double correction.

Or worse: HR clicks F5-approve while cron is mid-transaction — F5 sees stale numbers.

**Why it happens:**
No serialisation between manual and automatic paths. SQLite's transaction isolation helps but does not prevent read-compute-write races across two distinct transactions.

**How to avoid:**
- **The Pitfall-4 UNIQUE index does most of the work**: F3 and F4 both try to insert `rebooking_batch(kind, sales_person_id, iso_year, iso_week)` — one wins, one gets `UNIQUE violation`, which the service MUST translate into a user-visible `ServiceError::WeekAlreadyReconciled` (HTTP 409, analog v1.6 `PaidLimitExceeded`).
- F3 uses `kind='hr_manual'`, F4 uses `kind='auto_cron'`, F5 uses `kind='hr_suggestion'` — but the UNIQUE constraint is `(sales_person_id, iso_year, iso_week)` without kind, so a week can only be reconciled once by any path.
- Alternative if the "one reconciliation per week per person" invariant is not desired: composite UNIQUE `(kind, sales_person_id, iso_year, iso_week)` PLUS a pre-flight query that rejects "already reconciled by another kind this week" — same net effect, clearer error message.

**Warning signs:**
- Any F3/F4/F5 test that doesn't exercise the "concurrent write" case.
- Missing 409 mapping on rebooking endpoints.

**Prevention:** Requirement includes **Konfliktregel-Contract** ("A given (sales_person_id, iso_year, iso_week) can be reconciled at most once; second attempt returns 409 with error code `week-already-reconciled`"). Success Criterion: test that runs F3 + F4 concurrently (or sequentially in same transaction) and asserts exactly one succeeds.

**Which Phase:** F4 phase (introduces the constraint); F3 phase must consume the same constraint.

---

### Pitfall 7: F5-Vorschlag wird stale, weil F4 die Woche schon ausgeglichen hat

**What goes wrong:**
F5 pre-computes a "HR-Vorschlag: rebook 4h for week 2026-W27, IST 12h → DANN 8h". User opens the modal Monday 09:00. Overnight, F4 cron ran and already reconciled that week. HR clicks "approve" at 09:05 — the system either double-books (Pitfall 6, but the UNIQUE constraint catches it) or, worse, the F5 modal shows numbers that don't match reality.

**Why it happens:**
F5 is a read-side view; the underlying data (ExtraHours, rebooking state) is mutated by other paths (F3, F4). If F5 doesn't carry a version/hash of the state it saw, HR sees ghost suggestions.

**How to avoid:**
- F5 suggestion carries a `state_fingerprint` (e.g., latest `rebooking_batch.id` + latest `ExtraHours.id` for the person). On approve, backend checks fingerprint against current state; if different, return 409 `suggestion-stale` and the FE refreshes.
- Simpler alternative: F5's `pending` state is written to `rebooking_batch(kind='hr_suggestion', state='pending')` at *suggestion generation time*. That immediately claims the UNIQUE (sales_person_id, iso_year, iso_week) slot; F4 cron thereafter sees the row and skips the week (also solving Pitfall 6 from the other direction).
- Golden test: F5-suggest → F4-cron runs → F5-approve. Assert F4 was no-op and F5-approve succeeded (or F5 was cancelled with clear reason).

**Warning signs:**
- F5 modal shows numbers that don't match a fresh employee-report load.
- Approve button on F5 sometimes silently no-ops.

**Prevention:** Requirement includes **F5-Stale-Vorschlag-Erkennung** ("F5 suggestions are invalidated when the underlying state changes; approve returns 409 if stale"). Success Criterion: dedicated test for F4-cron-between-suggest-and-approve.

**Which Phase:** F5 phase — but the choice between "fingerprint" and "claim on suggest" strategy must be made in F5 discuss-phase.

---

### Pitfall 8: HR-Only Feld leakt via geteiltes DTO an Nicht-HR-User

**What goes wrong:**
F2 shows the Soll-Anzeige (`committed_voluntary × Vertragswochen`). This is HR-only. But `SalesPersonReportTO` is shared between HR-view (`/employees/{id}/report`) and Self-view (`/my-report`). If the field is present with a value on Self-view, non-HR users see it.

**Why it happens:**
The v1.5 STAT-01/02 pattern used FE-level gating (component checks role) — but the field was still in the DTO. Non-HR could inspect the network response. v1.8 VAC-OFFSET-01 fixed this properly with **API-level hiding**: `Option<f32> = None` in the DTO for non-HR responses. Same fix needed here.

**How to avoid:**
- Field `voluntary_committed_soll: Option<f32>` on the DTO. Backend fills it only for HR-role responses; Self-view gets `None`. FE hides the row when None (falls back gracefully).
- Test: hit `/my-report` as non-HR user → assert `voluntary_committed_soll: null` in JSON response. Hit same endpoint as HR user acting on their own record → assert value present.
- Do NOT rely on FE `role.is_hr` check alone.

**Warning signs:**
- Any non-HR test that only checks FE rendering, not JSON payload.
- Serde-serialisation of the DTO without a role-conditional path in the service layer.

**Prevention:** Requirement includes **HR-Only-Redaction pro VAC-OFFSET-01-Präzedenz** ("F2 Soll-Anzeige ist HR-only, DTO-Feld Option<f32>=None für Non-HR, kein FE-only-Gate"). Success Criterion: API-level test as non-HR user asserting null response.

**Which Phase:** F2 phase.

---

### Pitfall 9: Freiwilligen-Konto-Consistency — F1/F2/Balance schauen in verschiedene Wochen-Fenster

**What goes wrong:**
F2's "Freiwilliges Konto" = F2_Ist − F2_Soll. If F2_Ist counts weeks `[year-01-01, year-12-31]` (Gregorian) but F2_Soll counts weeks `[ISO(Y,1) - ISO(Y,weeks(Y))+1d]`, the delta drifts by up to 6 days of contract-time. Analog v2.5 Follow-up #3 bug.

Additionally: F1-Statistik (Ø freiwillig / Vertragswoche), F2-Konto (Ist − Soll), and Balance (via `reporting.rs`) each have their own "window" concept. They MUST agree on which week is in scope, otherwise the same person shows three different volunteer totals in the same UI.

**Why it happens:**
Three separate code sites (F1 in report-stats, F2 in employee-view, balance in reporting.rs) usually get written by three different sub-plans of the milestone. Without a shared pure helper, they drift.

**How to avoid:**
- Introduce a **shared pure helper** `voluntary_hours_for_person_in_range(person_id, start_iso, end_iso, extra_hours_slice) -> f32` in a common module (e.g., `service_impl/src/voluntary.rs`). F1, F2, and balance all call it. Analog to v2.5 Follow-up #1/#2 `derive_hours_for_week_pure` — the reason those helpers exist is exactly this class of drift.
- Property test: for a fixed dataset with 3 volunteer weeks in Dec 2026 (which straddle ISO 2026 end), `F1_Ist == F2_Ist == balance_volunteer_contribution` (each scaled by their respective denominator).
- Golden-snapshot: extend the existing 8 WOP-03 fixtures to include F1/F2/balance numbers; require byte-identity on re-run.

**Warning signs:**
- F1 average and F2 Ist show different totals for the same person in the same year in the UI.
- Any F1/F2/balance calculation that has its own bespoke `sum(extra_hours where category = VolunteerWork ...)` loop.

**Prevention:** Requirement includes **Freiwilligen-Consistency-Guard** ("F1, F2, and balance compute Ist-Freiwilligen-Stunden via a single pure helper; any deviation is a bug"). Success Criterion: property test asserting F1 × F2 × balance agreement across an ISO-year boundary.

**Which Phase:** F1 phase (introduces the helper); F2 and F4 phases consume it.

---

### Pitfall 10: Cap-Semantik-Grenzfall bei Wochenmitte-Vertragsänderung

**What goes wrong:**
`EmployeeWorkDetails` is time-versioned. A person's `committed_voluntary` changes from 4h to 6h on Wednesday 2026-07-08 (mid-week). What is F2's Soll for that week?
- Option A: pro-rata by weekday (4h × 3/7 + 6h × 4/7 = 5.14h).
- Option B: latest-active-in-week (6h).
- Option C: split into two "partial weeks" (4h for Mon-Tue, 6h for Wed-Sun) — each with its own Vertragswoche denominator, but then F1's "Vertragswochen"-Zähler explodes.

The v2.5 VAA formula (D-53-02) resolves ambiguity via `Σ filter(sales_person_id && (cap || expected==0)).map(committed_voluntary)` — but F2 across a whole year needs a stable per-week Soll.

**Why it happens:**
`WorkingHoursService` returns week-scoped values but the underlying `EmployeeWorkDetails` is date-scoped. Author picks one convention silently. Result: a mid-week contract change silently drops or double-counts.

**How to avoid:**
- **Decide in F2 discuss-phase** (list all three options + one more nobody has thought of); pin the choice as `D-F2-XX`.
- Pin the choice in a **CI-guard test** with the exact scenario: contract change Wed, assert Soll = decided-value.
- Reuse `working_hours_service::get_working_hours_for_week` semantics — whatever that already does, F2 MUST do too. If `get_working_hours_for_week` picks "latest active in week", F2 does too.
- Ideally, F2's Soll uses the SAME derivation path as the existing v1.4 CVC display of `committed_voluntary_hours` in the year overview — that path already made the choice.

**Warning signs:**
- Any code in F2 that has its own "which contract applies this week" logic separate from `WorkingHoursService`.
- No test with a contract-change-mid-week.

**Prevention:** Requirement includes **F2-Soll-Berechnung reuses existing `WorkingHoursService::get_working_hours_for_week` semantic** (Fat-Backend, no FE arithmetic). Success Criterion: test with contract-change-mid-week matches the pinned decision.

**Which Phase:** F2 phase.

---

### Pitfall 11: Frontend-Testbarkeit — Dioxus-Browser-Test unzuverlässig für Modal + Alert

**What goes wrong:**
D-25-06 established that Dioxus-Browser-Tests are unreliable for `<input type=date>` (programmatic set doesn't trigger signals) and for complex read-flows (screenshots time out on WASM report pages). F5's alert + modal + approve flow is exactly this class. If phases rely on browser tests, verification will stall.

**Why it happens:**
Verifier defaults to "run the browser test" for anything UI-facing. But per user precedent, "strukturell reicht" is acceptable for this class (D-25-06).

**How to avoid:**
- **Pure predicate tests**: extract `should_show_f5_alert(balance: f32, has_pending_suggestion: bool, is_capped: bool) -> bool` as a pure fn in `shifty-dioxus`. 8-case truth-table test analog to `should_show_pdf_button` (D-49-13).
- **SSR component test** where possible — Dioxus supports SSR of components, assert on rendered HTML strings, not on browser DOM.
- **Backend test doing the heavy lifting**: verifiy F5 numbers via `booking_information` or `reporting` service test, not via browser. FE test only asserts "the DTO field renders correctly".
- **Human INT-Sightcheck as final gate** for visual polish (analog VAA-04 in v2.5).
- Avoid programmatic `<input type=date>` setting; if F3/F5 modal has a date-input, use pure fn test for the state-transition + human sightcheck for the picker.

**Warning signs:**
- Any Plan that says "Dioxus browser test verifies F5-modal end-to-end".
- Missing pure-predicate for alert-visibility.

**Prevention:** Requirement includes **F5-Alert + F3/F5-Modal Testability-Strategy** ("F5 alert visibility is a pure predicate; approve flow verified by backend integration test + FE pure-predicate + human INT-Sightcheck"). Success Criterion: pure-predicate has ≥8-case truth-table + INT-Sightcheck logged in phase VERIFICATION.md.

**Which Phase:** F5 phase.

---

### Pitfall 12: HR-Approve/Reject-Concurrency — zwei HR-User klicken gleichzeitig approve

**What goes wrong:**
Two HR users open the same F5 suggestion. Both click Approve. Without a locking strategy, both trigger `ExtraHours` inserts → double reconciliation.

**Why it happens:**
F5 `pending → approved` state transition is a two-step read-then-write: "read state (pending), write extra_hours + state (approved)". Without SELECT ... FOR UPDATE or a state-conditional UPDATE, races happen.

**How to avoke:**
- The rebooking table Pitfall 6 UNIQUE constraint handles it: only one `ExtraHours` insert can succeed because only one `rebooking_batch` row can be created for the week. Second click gets 409.
- Additionally, use **state-conditional UPDATE**: `UPDATE rebooking_batch SET state = 'approved' WHERE id = ? AND state = 'pending'`. Check affected-row-count == 1; if 0, someone else beat you to it → return 409 `already-processed`.
- Reject flow same treatment: `UPDATE ... WHERE state = 'pending'` for reject.
- Test: spawn two concurrent approve requests over the same suggestion, assert exactly one succeeds with 200 and one gets 409.

**Warning signs:**
- Any approve/reject handler that reads state, then writes without a WHERE-state-clause.
- Missing 409-race test.

**Prevention:** Requirement includes **F5-Approve-Idempotenz** ("F5 approve/reject is single-shot; concurrent second click returns 409"). Success Criterion: concurrent-race test.

**Which Phase:** F5 phase.

---

## Minor Pitfalls

### Pitfall 13: Docs-Drift trotz hartem Gate

**What goes wrong:**
`docs/features/F0X-*.md` (+ `_de.md`) MUST be updated in the same phase if `service_impl/src/reporting.rs`, `billing_period_report.rs`, or `migrations/sqlite/*.sql` are touched. F4 touches all three. Missing docs drift blocks milestone close (v2.6 audit gate).

**Prevention:** Every F4/F2 phase plan checks the CLAUDE.md trigger-table. Add "docs synced (F07 + F08 + F0-new-freiwilligen-ausgleich, both EN + DE)" as Success Criterion literally on the phase.

**Which Phase:** Every phase that touches a trigger file.

---

### Pitfall 14: `Dioxus.toml` proxy fehlt für neue Backend-Routes

**What goes wrong:**
F3/F4/F5 REST endpoints (e.g., `/rebooking`, `/rebooking-suggestion/{id}/approve`) return 404 in `dx serve` dev mode because `[[web.proxy]]` in `shifty-dioxus/Dioxus.toml` was not extended. Precedent Phase 28 + 49 (twice forgotten in MEMORY).

**Prevention:** Success Criterion in every FE-touching phase: "Dioxus.toml `[[web.proxy]]` updated for new backend routes; dev-server sightcheck passes."

**Which Phase:** F3, F5 FE phases.

---

### Pitfall 15: Backend-Roundtrip nicht e2e verifiziert (create ≠ edit path)

**What goes wrong:**
Precedent Phase 23: `modify_slot` silently dropped `max_paid_employees` while `create_slot` kept it. In F4/F5, the "auto-cron creates rebooking" path and "HR-approve creates rebooking from suggestion" path may drop fields differently.

**Prevention:** Every rebooking-creating path (F3 manual, F4 auto, F5 approve) round-trips through the same `RebookingService::create_batch(kind, ...)` function — no path-specific ExtraHours construction. Test: three input flows, one output shape, assert equality.

**Which Phase:** F4 phase (introduces service); F3 and F5 phases consume it.

---

### Pitfall 16: WebDAV/Cron-Startup-Reihenfolge

**What goes wrong:**
Precedent v2.2 EXP: cron seed format was wrong (5-field vs. 6-field), post-ship hotfix v2.3.1. F4 introduces the second cron in Shifty. If the seed migration uses the wrong cron-syntax variant, cron silently doesn't run.

**Prevention:** Seed format matches the existing v2.2 WebDAV cron pattern exactly (6-field). Success Criterion: startup log shows both crons scheduled with correct next-run time.

**Which Phase:** F4 phase.

---

### Pitfall 17: Negative-Balance-Alert-Edge-Case

**What goes wrong:**
F5 shows alert "wenn negatives Stundenkonto UND `cap_planned_hours_to_expected=true`". Edge cases:
- Person has `cap=false` but negative balance → no alert (correct, but reviewer will ask).
- Person has `cap=true` but F2_Ist = 0 (no volunteer work ever) → alert would suggest rebooking 0h from nothing → nonsensical.
- Person has `cap=true`, F2_Ist > 0, but balance just slightly negative < 0.5h → suggest 0.5h min? Or don't alert?

**Prevention:** F5-alert-predicate is a pure fn with a truth-table test covering these edges (pinning decision from discuss-phase). Precedent D-49-13 8-case matrix.

**Which Phase:** F5 phase.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| **F1 Ist-Statistik** | Denominator "Vertragswochen" ambiguity when contract changes mid-year (Pitfall 10 sibling); shared helper drift (Pitfall 9) | Reuse existing `WorkingHoursService` semantics; introduce `voluntary_hours_for_person_in_range` helper up front |
| **F2 Freiwilliges Konto** | HR-Only leak via shared DTO (Pitfall 8); mid-week contract change (Pitfall 10); F1/F2/balance drift (Pitfall 9) | API-level `Option<f32>=None` redaction per VAC-OFFSET-01; pin week-derivation to `WorkingHoursService` |
| **F3 Manuelle Umbuchung** | Transactional atomicity of two ExtraHours rows (Pitfall 1 partial); TOCTOU with F4 (Pitfall 6); rebooking marker column (Pitfall 1) | Single service fn creates rebooking_batch + entries + extra_hours in one tx; UNIQUE constraint enforces mutual exclusion |
| **F4 Auto-Cron** | Idempotency (Pitfall 4); ISO-Woche (Pitfall 5); Snapshot bump (Pitfall 2); Stichtag legacy semantics per chain (Pitfall 3); backfill duplicates (Pitfall 4 sibling) | UNIQUE-conflict-first pattern; ISO-week helpers from v2.5; explicit bump decision in DISCUSS; chain audit + toggle-off golden snapshots |
| **F5 HR-Alert + Modal** | Stale suggestion after F4 (Pitfall 7); approve-race (Pitfall 12); FE testability (Pitfall 11); alert edge cases (Pitfall 17); Dioxus.toml proxy (Pitfall 14) | Claim-on-suggest via rebooking_batch(state=pending); state-conditional UPDATE; pure-predicate + INT-Sightcheck; explicit truth-table |
| **All phases** | Docs drift (Pitfall 13); backend roundtrip (Pitfall 15); Dioxus.toml proxy (Pitfall 14) | Add trigger-file→doc-file mapping to phase's Success Criteria upfront |

---

## Sources

- `.planning/PROJECT.md` — current milestone description (F1–F5 goals + rebooking_batch table sketch); v2.5 shipped notes (WOP-Follow-up #3 ISO-year bug, VAA D-53-02 formula); Snapshot version history (currently 12); Docs-Freshness-Gate contract. **Confidence: HIGH**.
- `shifty-backend/CLAUDE.md` § "Billing Period Snapshot Schema Versioning" — hard contract for `CURRENT_SNAPSHOT_SCHEMA_VERSION` bumps. **Confidence: HIGH**.
- `.planning/milestones/v1.7-REQUIREMENTS.md` — HCFG-01/02/03 Stichtag precedent; HSNAP-01 snapshot bump (10→11); HOL-03/VFA-02 asymmetry regression guard. **Confidence: HIGH**.
- `.planning/milestones/v2.5-REQUIREMENTS.md` — VAA D-53-02 cap-gated formula for `committed_voluntary` filtering; WOP-Follow-up #3 `_iso_year` helpers as reusable pattern. **Confidence: HIGH**.
- `.planning/milestones/v2.2-REQUIREMENTS.md` — EXP-01/02/03 WebDAV cron precedent (transient-failure retry, admin-gated, config via env vars); v2.3.1 post-ship hotfix (cron-seed 5- vs. 6-field). **Confidence: HIGH**.
- MEMORY notes referenced inline: `feedback_stichtag_rollout_legacy_semantics.md`, `feedback_atomic_repoint_no_double_count.md`, `reference_toggle_service_full_context_bypass.md`, `feedback_verify_backend_roundtrip_e2e.md`, `feedback_dioxus_proxy_for_new_backend_endpoints.md`, `reference_dioxus_browser_test_date_inputs.md`, `feedback_docs_always_current_no_followup.md`. **Confidence: HIGH** (direct user observations).
- Prior PITFALLS.md v2.1 (this file, overwritten) — pattern template for ISO-year vs Gregorian and lock-key traps. **Confidence: HIGH**.
