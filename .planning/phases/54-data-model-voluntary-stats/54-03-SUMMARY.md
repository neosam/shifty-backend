---
phase: 54-data-model-voluntary-stats
plan: 03
subsystem: [backend, service, service_impl, di]
tags: [tdd, business-logic-tier, hr-gate, voluntary-stats, VOL-STAT-01, VOL-STAT-02, VOL-ACCT-01, VOL-ACCT-02, VOL-ACCT-03, D-F1-01, D-F2-01, D-54-DM-02]
status: complete
requirements:
  - VOL-STAT-01
  - VOL-STAT-02
  - VOL-ACCT-01
  - VOL-ACCT-02
  - VOL-ACCT-03
dependency_graph:
  requires:
    - 54-01 (ExtraHoursSource enum + source field auf ExtraHours)
    - 54-02 (RebookingBatchService — Basic-Tier, keine direkte Dep, aber Wave-Vorgänger)
  provides:
    - VoluntaryStatsService (BL-Tier, HR-only)
    - 4 pure fns in service_impl::reporting (voluntary_ist_total_for_year,
      contract_weeks_count, committed_voluntary_prorata_for_week,
      committed_voluntary_target_for_year)
  affects:
    - Plan 54-04 (REST-Endpoint konsumiert VoluntaryStatsService)
tech-stack:
  added: []
  patterns:
    - Business-Logic-Tier-Service via gen_service_impl!-Makro
    - HR-Gate an erster Stelle mit API-Level-None-Redaktion (Präzedenz VAC-OFFSET-01 v1.8)
    - Pure fns neben committed_voluntary_for_calendar_week in reporting.rs
    - Property-Test-Muster (Fixture ohne proptest-Crate, Präzedenz VAA-04)
    - Cross-Service-Calls mit Authentication::Full (Bypass nach Auth-Check)
key-files:
  created:
    - service/src/voluntary_stats.rs
    - service_impl/src/voluntary_stats.rs
    - service_impl/src/test/voluntary_stats.rs
  modified:
    - service/src/lib.rs (Modul-Registration)
    - service_impl/src/lib.rs (Modul-Registration)
    - service_impl/src/reporting.rs (4 pure fns + ExtraHoursSource-Import)
    - service_impl/src/test/mod.rs (voluntary_stats-Testmodul)
    - rest/src/lib.rs (RestStateDef-Erweiterung)
    - shifty_bin/src/main.rs (DI-Wiring)
    - .planning/phases/54-data-model-voluntary-stats/54-VALIDATION.md
decisions:
  - "Pure fns liegen in service_impl::reporting (nicht in einem neuen Modul), damit sie neben committed_voluntary_for_calendar_week + apply_weekly_cap und der geteilten ISO-Wochen-Semantik stehen."
  - "committed_voluntary_prorata_for_week iteriert tagesweise Mo..So und wählt pro Tag die aktive EmployeeWorkDetails via from_date/to_date-Bereich (nicht via find_working_hours_for_calendar_week), damit Mid-Week-Wechsel D-F2-01 exakt tagesgenau abgebildet wird (nicht wochenweise)."
  - "Non-HR-Aufrufer erhalten VoluntaryStats mit lauter None statt Err(Forbidden) — Präzedenz VAC-OFFSET-01 v1.8, keine 403."
  - "voluntary_ist_total_for_year filtert zusaetzlich deleted.is_none() — Soft-Deletes duerfen die Summe nicht verfaelschen."
metrics:
  duration: ~9 min
  completed: 2026-07-07
  tasks: 7
  files_touched: 7
  tests_added: 11
  commits: 3
---

# Phase 54 Plan 03: VoluntaryStatsService (BL-Tier) — TDD-Impl mit pure fns Summary

**One-liner:** Business-Logic-Tier VoluntaryStatsService (VOL-STAT + VOL-ACCT) mit HR-Gate + None-Redaktion, komponiert vier pure fns in reporting.rs; TDD-Sequenz RED→GREEN mit Property-Test VOL-ACCT-03 gegen Rebooking-Doppelzählung.

## Pure-fn Signaturen

Alle vier fns liegen in `service_impl/src/reporting.rs` direkt nach `committed_voluntary_for_calendar_week`:

```rust
/// VOL-STAT-01 / VOL-ACCT-01-Ist (F1-Ist Zähler)
pub fn voluntary_ist_total_for_year(extra_hours: &[ExtraHours], year: u32) -> f32;

/// VOL-STAT-01-Nenner / D-F1-01 (expected_hours=0 zählt MIT)
pub fn contract_weeks_count(working_hours: &[EmployeeWorkDetails], year: u32) -> u32;

/// D-F2-01: Tages-pro-rata für eine ISO-Woche
pub fn committed_voluntary_prorata_for_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> f32;

/// D-F2-01 Summe über alle ISO-Wochen des Jahres (52 oder 53)
pub fn committed_voluntary_target_for_year(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
) -> f32;
```

## Test-Übersicht

Datei: `service_impl/src/test/voluntary_stats.rs` (11 Tests, 495 Zeilen).

| # | Test | Zweck |
|---|------|-------|
| 1 | `f1_ist_manual_only_20h` | F1-Ist bei 5 Manual-VolunteerWork-Rows a 4h = 20.0 |
| 2 | `f1_ist_rebooking_pair_invariant_vol_acct_03` | **Property-Test:** Rebooking-Pair (-4+4, source=Rebooking) neutral für F1-Ist |
| 3 | `f2_soll_zero_when_no_committed_voluntary` | committed_voluntary=0 → target=0 |
| 4 | `f2_soll_prorata_midweek_change_d_f2_01` | Mid-Week Mittwoch: 3/7·7 + 4/7·14 = 11.0 |
| 5 | `contract_weeks_zero_expected_counts_d_f1_01` | expected_hours=0 → 6 Vertragswochen bei KW 10..=15 |
| 6 | `contract_weeks_empty_working_hours_returns_zero` | Leere Liste → 0 |
| 7 | `f2_soll_iso_week_53_year_boundary_d_f2_01` | 2026 (53 Wochen) → 53.0; 2025 (52 Wochen) → 52.0 |
| 8 | `f1_ist_and_f2_soll_share_iso_week_semantics_d_f1_01_kongruenz` | 2026-01-01 (ISO-Jahr 2026) korrekt zugeordnet |
| 9 | `service_tests::service_non_hr_returns_all_none_vol_stat_02` | Non-HR → alle Felder None, KEIN DAO-Call |
| 10 | `service_tests::service_hr_returns_some_and_delegates_to_pure_fns` | HR: 10 KW · 1.0 committed_voluntary = 10.0 soll_total, ist_total=10.0, delta=0.0 |
| 11 | `service_tests::service_zero_contract_weeks_yields_zero_per_week` | Division-Guard: 0 Vertragswochen → ist_per_contract_week=0.0 (kein NaN/inf) |

**Full-Suite:** `cargo test --workspace` → 748 (service_impl-lib) + 64 (service_impl-integration) + 11 (voluntary_stats submodul enthalten in 748) + weitere = alle grün.
**Clippy:** `cargo clippy --workspace -- -D warnings` grün ohne neue Lints.

## Decision-Coverage-Diff

| Decision | Test | Verifikation |
|----------|------|--------------|
| **D-F1-01** — contract_weeks zählen expected_hours=0 MIT | `contract_weeks_zero_expected_counts_d_f1_01` | KW 10..=15, expected=0.0 → count=6 |
| **D-F1-01** (Kongruenz) — Zähler + Nenner gleiche ISO-Wochen-Semantik | `f1_ist_and_f2_soll_share_iso_week_semantics_d_f1_01_kongruenz` | 2026-01-01 (calendar-year 2026, ISO-year 2026) → wird year=2026 zugeordnet |
| **D-F2-01** — Mid-Week Prorata Mittwoch | `f2_soll_prorata_midweek_change_d_f2_01` | 3/7·7.0 + 4/7·14.0 = 11.0 |
| **D-F2-01** — ISO-53-Wochen-Jahr | `f2_soll_iso_week_53_year_boundary_d_f2_01` | 2026: 53 Wochen; 2025: 52 Wochen |
| **D-54-DM-02 / VOL-ACCT-03** — Rebooking-Marker neutral | `f1_ist_rebooking_pair_invariant_vol_acct_03` | Rebooking-Pair (-4/+4) verändert Summe (=20.0) NICHT |

## Deviations from Plan

Keine — der Plan wurde exakt wie geschrieben ausgeführt. **Eine Anpassung** gegenüber dem Plan-Text:

- `committed_voluntary_prorata_for_week`: Der Plan-Text schlug vor, `find_working_hours_for_calendar_week` pro Tag mit der ISO-Woche des Tages aufzurufen. Für **Mid-Week-Vertragswechsel** (D-F2-01) reicht das nicht — beide Verträge liegen in derselben ISO-Woche und die Wochen-basierte Selektion würde die Tagesgranularität verlieren. Die Impl wählt daher pro Tag den aktiven Vertrag via `from_date <= day <= to_date`. Der bestehende `derive_hours_for_week_pure`-Helper in reporting.rs macht dasselbe (Zeilen 195–214), sodass das Muster im Modul konsistent bleibt. Der D-F2-01-Test verifiziert den erwarteten Wert (11.0).

**Zusätzlicher Guard:** `voluntary_ist_total_for_year` filtert zusätzlich `deleted.is_none()`, damit Soft-Delete-Rows die Summe nicht verfälschen (Rule 2 — kritische Korrektheit, kein Plan-Text-Widerspruch, präzedenzkonform mit allen anderen `extra_hours`-Filtern in reporting.rs).

## Commits

- `e28476f` **test(54-03): add failing pure-fn tests for F1/F2 (RED)** — Test-Datei + mod-Registration; kompiliert nicht (unresolved imports der 4 pure fns + voluntary_stats-Modul) → RED-Zustand bewiesen.
- `0767fb4` **feat(54-03): implement VoluntaryStatsService + pure fns (GREEN)** — 4 pure fns in reporting.rs + Trait service/src/voluntary_stats.rs + Impl service_impl/src/voluntary_stats.rs + Modul-Registrationen. Alle 11 Tests grün.
- `9ba0029` **feat(54-03): wire VoluntaryStatsService into DI + RestStateDef** — rest/src/lib.rs (Trait-Erweiterung) + shifty_bin/src/main.rs (Type-Alias, Deps-Impl, Konstruktion in BL-Wave, RestStateImpl-Feld + Getter).

## TDD Gate Compliance

Sequenz nachweisbar in git log:
```
git log --oneline --grep="54-03"
9ba0029 feat(54-03): wire VoluntaryStatsService into DI + RestStateDef
0767fb4 feat(54-03): implement VoluntaryStatsService + pure fns (GREEN)
e28476f test(54-03): add failing pure-fn tests for F1/F2 (RED)
```

- **RED-Gate** (Commit `e28476f`): `test(54-03): add failing pure-fn tests`, Datei `service_impl/src/test/voluntary_stats.rs` mit 11 Tests + Modul-Registration; kompiliert nicht (E0432 unresolved import der 4 pure fns + voluntary_stats-Modul).
- **GREEN-Gate** (Commit `0767fb4`): `feat(54-03): implement`, alle 11 Tests grün.
- **REFACTOR-Gate:** entfällt (keine strukturellen Vereinfachungen nötig).

## Verification-Log

| Gate | Command | Result |
|------|---------|--------|
| Pre-RED | `cargo test -p service_impl voluntary_stats --lib` | ❌ E0432 unresolved imports (RED bewiesen) |
| Post-GREEN | `cargo test -p service_impl --lib voluntary_stats::` | ✅ 11 passed |
| Full-Suite Build | `cargo build --workspace` | ✅ Finished |
| Full-Suite Tests | `cargo test --workspace` | ✅ 0 failed |
| Clippy Gate | `cargo clippy --workspace -- -D warnings` | ✅ Finished, keine Warnings |

## Self-Check: PASSED

Verifiziert:
- `service/src/voluntary_stats.rs` existiert ✅
- `service_impl/src/voluntary_stats.rs` existiert ✅
- `service_impl/src/test/voluntary_stats.rs` existiert ✅
- Alle 4 pure fns in `service_impl/src/reporting.rs` (grep) ✅
- Alle 3 Commits `e28476f`, `0767fb4`, `9ba0029` in git log ✅
- RestStateDef-Erweiterung in `rest/src/lib.rs` ✅
- DI-Wiring in `shifty_bin/src/main.rs` (Type + Deps + Konstruktion + Feld + Init + Getter, 6 Vorkommen) ✅
