---
phase: 08-absence-crud-page-foundation
plan: 08-09
subsystem: cutover
tags: [cutover, heuristic, lump-sum, absence, migration, iso-week]

# Dependency graph
requires:
  - phase: 04-migration-cutover
    provides: CutoverService.run + migrate_legacy_extra_hours_to_clusters + strict-match heuristic + QuarantineReason enum
  - phase: 08-absence-crud-page-foundation/08-08
    provides: CutoverQuarantineEntry + DriftRow.quarantined_entries (inline drift-report shape unchanged)
provides:
  - Weekly-Lump-Sum-Heuristik in der Cutover-Migration: 1× extra_hours-Row mit `amount = Σ(contract.hours_per_day) für Vertragstage der ISO-Woche` wird auf absence_period {Mo, So} der Woche gemappt — auch wenn der Eintrag-Tag ein Nicht-Vertragstag ist.
  - Backwards-compat: Strict-Match (1 Tag = hours_per_day) und Cluster-of-N (aufeinanderfolgende Vertragstage) bleiben unverändert; die Heuristik ist additive Ergänzung VOR den existing Quarantine-Pfaden.
  - Live-Szenario Max-Schmidt (3-Tage-Vertrag, 20h Vacation am Freitag) migriert sauber: gate_passed=true, drift=0, 1 Cluster, 0 Quarantäne.
affects: [Cutover-Operator-UX (weniger Quarantäne-Backlog für reale Datenmuster)]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ISO-Wochen-Boundary via `time::Date::from_iso_week_date()`-Roundtrip — nutzt das time-Crate-Native, keine eigene Mo-of-week-Math."
    - "Per-Weekday-Contract-Lookup (`contract_at(weekday)`-Closure) — Heuristik stützt sich nicht auf einen einzigen Vertrag pro Woche, kompatibel mit Mid-Week-Vertragswechseln."
    - "Heuristik-Pre-Check vor Quarantine-Pfaden: Migration-Loop-Order erweitert um (a.5) Lump-Sum-Detection; Match → expliziter 1-Row-Cluster mit überschriebenem Range; non-match → fallthrough zur Strict-Match-Logik unverändert."

key-files:
  created:
    - .planning/phases/08-absence-crud-page-foundation/08-09-SUMMARY.md
  modified:
    - service_impl/src/cutover.rs
    - service_impl/src/test/cutover.rs
    - shifty_bin/src/integration_test/cutover.rs

key-decisions:
  - "Helper-Funktion `detect_weekly_lump_sum(row, all_rows, contract_at)` als freistehende Funktion am Modul-Ende — kein State, easy zu testen, easy zu lesen vom Migration-Loop."
  - "Cluster-Range-Override per direkt-gepushtem `MigratedCluster` (statt Erweiterung der `InProgressCluster`-Struct um `range_override: Option<(Date, Date)>`). Hält den klassischen Cluster-Pfad zero-impact und macht den Lump-Sum-Pfad als Read-only-Pre-Check sichtbar im Code."
  - "Detection-Order: Lump-Sum-Check VOR Workday-Quarantine + Strict-Match-Quarantine. Begründung: Wochenpauschalen liegen oft auf Nicht-Vertragstagen (genau das Problem das wir lösen) und/oder mit amount > hours_per_day — sonst wäre die Heuristik nutzlos."
  - "ISO-Wochen-Boundary via `time::Date::to_iso_week_date()` + `from_iso_week_date()`. Cross-Year-Korrekt (KW 53 oder KW 1 spreading über Jahresgrenze) ohne manuelle Mo-of-week-Math."
  - "Per-Weekday-Contract-Lookup statt 'first contract of the week' — kompatibel mit Vertragswechseln mid-week (Test 7 deckt diesen Pfad ab)."
  - "Single-Row-Pro-Woche-Constraint linear gegen `all_rows` für die jeweilige (sp, cat) — kein zusätzlicher SQL-Roundtrip, kein zusätzlicher DAO-Call. `all_rows` ist sortiert, aber die Heuristik filter ist O(n) je Row; in der Praxis ist die Zahl der Rows pro (sp, cat) klein."
  - "Gate-Phase unverändert — `derive_hours_for_range` rekonstruiert die lump-sum-Stunden bereits korrekt (3×hours_per_day = expected_hours = legacy_sum). Plan-08-09 task 3 hat das verifiziert; ein Doc-Comment auf der Heuristik-Funktion hält die Begründung fest."
  - "Integration-Test #19 (Plan 08-08 inline drift report) wurde angepasst: amount 20.0 → 25.0 — exakt damit die Heuristik NICHT mehr trifft und der Test seinen ursprünglichen Zweck (failed-gate inline drift rendering) behält. Sonst wäre Test #19 mit Plan 08-09 Logik gebrochen."

patterns-established:
  - "Heuristik-Pre-Check-Pattern: Wenn eine bestehende Migration- oder Quarantine-Logik Edge-Cases falsch ablehnt, baue einen Pre-Check VOR den existing Pfaden ein — Match → bypass mit explizitem Output, Non-Match → fallthrough zur unveränderten Logik. Backwards-compat ohne Code-Duplication."
  - "ISO-Week-Range-Helper als freistehende Funktion (`iso_week_range(day) -> (Mo, So)`) — Cross-Year-korrekt via time-Crate-Roundtrip, kein eigener calendar-week-math."
  - "Live-UAT-Szenario als primärer Test-Target: ein expliziter Integration-Test (Test #20 in cutover-Integration) reproduziert den User-UAT-Bug (Max Schmidt 20h@Freitag) verbatim, gegen die echte SQLite + EmployeeWorkDetails. Die Unit-Tests sind die Detail-Coverage; der Integration-Test ist der Acceptance-Lock."

requirements-completed: []

# Metrics
duration: 50min
completed: 2026-05-08
---

# Phase 08 Plan 08-09: Cutover Wochenpauschalen-Heuristik Summary

**Erweitert die Cutover-Migration um eine Wochenpauschalen-Erkennung: 1 extra_hours-Row mit `amount` ≈ Σ(`hours_per_day` der Vertragstage in der ISO-Woche) wird auf `absence_period {Mo, So}` gemappt, auch wenn der Eintrag-Tag ein Nicht-Vertragstag ist. Live-Szenario Max-Schmidt (20h Vacation am Freitag bei 3-Tage-Mo/Di/Mi-Vertrag) migriert jetzt sauber.**

## Performance

- **Duration:** ~50 min
- **Started:** 2026-05-08T12:30Z (approx.)
- **Completed:** 2026-05-08T12:47Z
- **Tasks:** 5 (1 plan-setup + 1 heuristic-impl + 1 doc-only-fix + 1 test-pack + 1 closure)
- **Files modified:** 3 (service_impl/cutover.rs, service_impl/test/cutover.rs, shifty_bin/integration_test/cutover.rs)
- **Tests added:** 7 unit + 1 integration = 8 tests total. Plus 1 angepasster integration-test (#19).

## Accomplishments

### Heuristik-Implementation (`service_impl/src/cutover.rs`)

- `detect_weekly_lump_sum(row, all_rows, contract_at) -> Option<(Date, Date)>` — Helper-Funktion mit der dreistufigen Erkennungsregel:
  1. **Single-Row-Pro-Woche:** Lineare Suche über `all_rows` nach anderem `(sp, cat)` in derselben ISO-Woche → bei Konflikt: `None`.
  2. **Active-Contract-Required:** Mindestens ein Tag der Woche muss einen aktiven Vertrag haben → sonst `None` (das ist die `ContractNotActiveAtDate`-Domäne).
  3. **Amount-Match:** `|row.amount − Σ hours_per_day(d)| < CONTRACT_HOURS_EPSILON`, Summe über jeden Wochentag mit aktivem Vertrag der den Tag als Workday hat.
- `iso_week_range(day) -> (Mo, So)` — Cross-Year-korrekt via `time::Date::to_iso_week_date()` + `from_iso_week_date()`.
- `lookup_active_contract(work_details, day) -> Option<&EmployeeWorkDetails>` — Mirror der Per-Day-Vertragsabfrage aus dem Migration-Loop, jetzt als wiederverwendbarer Helper.
- Migration-Loop bekommt einen neuen Pre-Check (a.5) VOR den Workday- und Strict-Match-Quarantine-Pfaden. Match → `close_current_cluster()` + direkt-gepushter `MigratedCluster` mit explizitem `{Mo, So}`-Range + `continue`.

### Tests

**Unit-Tests** (`service_impl/src/test/cutover.rs`, 7 neue Tests + 1 neue Fixture):

- `fixture_3day_mo_tu_we_contract()` — Standardfixture für die Heuristik-Tests (3-day Mo/Tu/We, 20h/week).
- `test_weekly_lump_sum_at_workday_succeeds` — 20h Vacation am Mo (Vertragstag) → 1 Cluster {Mo, So}.
- `test_weekly_lump_sum_at_non_workday_succeeds` — **Live-Reproduce des Max-Schmidt-Bugs:** 20h Vacation am Freitag 2026-05-08 (Nicht-Vertragstag) → 1 Cluster {2026-05-04, 2026-05-10}, keine Quarantäne.
- `test_weekly_lump_sum_at_weekend_succeeds` — 20h Vacation am Sonntag → 1 Cluster {Mo, So}.
- `test_strict_match_per_day_still_works_after_pivot` — 6.667h (= hours_per_day) am Mo → 1-Day-Cluster (Mo-Mo) via existing strict-match-Pfad. Bewahrt Backwards-Compat.
- `test_two_rows_same_week_blocks_lump_sum` — 20h-Mo + 6.667h-Di → KEINE Heuristik (Single-Row-Constraint verletzt). Mo wird AmountAbove-Quarantäne, Di wird 1-Day-Cluster via strict-match.
- `test_partial_week_amount_falls_to_strict_match` — 13.33h (= 2 × hours_per_day) am Mo → KEINE Heuristik (amount ≠ Wochensumme). Strict-Match → AmountAbove-Quarantäne.
- `test_weekly_lump_sum_with_dynamic_contract_change_mid_week` — 3-Tage-Vertrag bis 2024-06-09, dann 4-Tage-Vertrag. 32h Vacation in Woche 24 → 1 Cluster (4 × 8h = 32h match unter dem 4-Tage-Vertrag).

**Integration-Test** (`shifty_bin/src/integration_test/cutover.rs`):

- `test_weekly_lump_sum_commit_succeeds_end_to_end` — Live-UAT-Reproduce gegen die echte SQLite + Migration-Pipeline. SP "Max Schmidt" + 3-Tage-Vertrag (Mo/Tu/We, 20h/week) + 1 extra_hours-Row (20h Vacation, Friday 2026-05-08). Cutover-Commit (NICHT dry-run) → `gate_passed=true`, `migrated_clusters=1`, `quarantined_rows=0`, `gate_drift_rows=0`. Verifiziert (a) Feature-Flag `absence_range_source_active=true` post-commit, (b) extra_hours-Row soft-deleted, (c) 1 Row in `absence_period` mit from=2026-05-04 / to=2026-05-10, (d) `derive_hours_for_range` rekonstruiert ≈ 20h Vacation für die Range.

**Angepasster Integration-Test (#19):**

- `test_failed_gate_returns_inline_drift_report_with_per_entry_details` (Plan 08-08) — Fixture-Amount 20.0 → 25.0, sodass die neue Heuristik den Friday-Eintrag NICHT mehr akzeptiert (25 ≠ Σ contract weekday hours = 24). Test behält seinen ursprünglichen Zweck (failed-gate liefert per-entry inline drift report) und verifiziert weiter den `ContractHoursZeroForDay`-Quarantäne-Pfad + DTO-Round-Trip + Reason-Mapping.

### Doc-only (Plan 08-09 Task 3)

Doc-Comment auf `detect_weekly_lump_sum`, der erklärt warum die Gate-Phase **keinen** parallelen Patch braucht: `derive_hours_for_range` iteriert pro Tag, prüft `has_day_of_week`, holiday-skip, contract-active-skip — und liefert `hours_per_day` pro Vertragstag. Bei `{Mo, So}`-Range mit 3-Tage-Vertrag (Mo/Tu/We) ist `Σ hours = 3 × 6.667h ≈ 20h = legacy_sum`. Drift = 0. Gate passes.

## Tests

- `cargo test --workspace` (nix develop): **alle grün** — 396 service_impl + 68 shifty_bin + alle anderen.
- `cargo test -p service_impl --lib cutover`: **19 grün** (12 existing + 7 neu).
- `cargo test --bin shifty_bin integration_test::cutover`: **20 grün** (19 existing inkl. angepasster #19 + 1 neu).
- `cargo build --bin shifty_bin`: grün.
- `cargo build --target wasm32-unknown-unknown` in `shifty-dioxus/`: grün (38 preexisting warnings, nicht Plan-08-09-related).

## Code-Pointer

- Heuristik: `service_impl/src/cutover.rs:485-493` (Migration-Loop-Pre-Check), `service_impl/src/cutover.rs:1023-1117` (Helper-Funktionen).
- Unit-Tests: `service_impl/src/test/cutover.rs:920-1196` (7 Tests + Fixture).
- Integration-Test: `shifty_bin/src/integration_test/cutover.rs:1505+` (`test_weekly_lump_sum_commit_succeeds_end_to_end`).

## Backwards-Compat

- **Bestehende Strict-Match-Tests** (`cluster_merges_consecutive_workdays_with_exact_match`, alle 5 Quarantine-Reason-Tests, `idempotent_rerun_skips_mapped`, beide Forbidden-Tests, beide Gate-Tolerance-Tests): grün ohne Modification.
- **Bestehende Cluster-of-N-Cases** (z.B. `test_idempotence_rerun_no_op` mit 5 Mo-Fr Vacation-Rows): grün — 5h-pro-Tag-Match ist KEINE Wochenpauschale (40h = expected_hours, aber 5 Rows in derselben Woche → Single-Row-Constraint verletzt) → fall-through zum existing Cluster-Builder.
- **Quarantine-Reasons unverändert:** `AmountBelow/AboveContractHours`, `ContractHoursZeroForDay`, `ContractNotActiveAtDate`, `Iso53WeekGap` — keine neuen Reasons, kein DTO-Bruch.

## Deviations from Plan

None of substance. Tasks 1+2 wurden in einem einzigen jj-Commit zusammengeführt, weil die `Cluster`-Range-Override durch den direkt-gepushten `MigratedCluster`-Pfad realisiert wurde (per Plan 08-09 explicit als Alternative erlaubt: "Falls in Task 1 alternativ eine Variante mit direkt-gepushtem Cluster gewählt wurde, entfällt Task 2"). Task 3 ist doc-only ausgefallen (verifikation, dass Gate-Phase keinen Patch braucht). Insgesamt 5 jj-commits: 1 plan-setup + 4 atomic content-commits + 1 docs-closure (= dieser Summary-Commit).

## Self-Check: PASSED

- service_impl/src/cutover.rs: existing
- service_impl/src/test/cutover.rs: existing (extended)
- shifty_bin/src/integration_test/cutover.rs: existing (extended + 1 modified)
- jj log shows: docs(08-09) plan + feat(08-09) heuristic + fix(08-09) gate-doc + test(08-09) tests
