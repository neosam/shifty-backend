---
phase: 54-data-model-voluntary-stats
plan: 07
subsystem: reporting/voluntary-stats
tags: [gap-closure, backend, reporting, voluntary-stats, range-semantics, docs-freshness]
requirements: [VOL-STAT-01, VOL-STAT-02, VOL-ACCT-01, VOL-ACCT-02]
requires:
  - Plan 54-03 (VoluntaryStatsService baseline)
  - Plan 54-04 (REST endpoint + integration test seed)
provides:
  - voluntary_ist_total_in_range (pure fn)
  - contract_weeks_count_in_range (pure fn)
  - committed_voluntary_target_in_range (pure fn)
  - VoluntaryStatsService::get_voluntary_stats(from_date, to_date, …)
  - GET /report/{id}/voluntary-stats?from_date=YYYY-MM-DD&to_date=YYYY-MM-DD
affects:
  - shifty-dioxus (breaks temporarily until Plan 54-08 in Wave 2)
tech-stack:
  added: []
  patterns:
    - "Range-based aggregation analog ReportingService::get_report_for_employee_range"
    - "ISO date parsing precedent from rest/src/toggle.rs (Length + Delimiter check → HTTP 400)"
key-files:
  created: []
  modified:
    - service_impl/src/reporting.rs
    - service/src/voluntary_stats.rs
    - service_impl/src/voluntary_stats.rs
    - service_impl/src/test/voluntary_stats.rs
    - rest/src/report.rs
    - shifty_bin/src/integration_test/voluntary_stats.rs
    - docs/features/F14-rebooking.md
    - docs/features/F14-rebooking_de.md
    - docs/features/F07-reporting-balance.md
    - docs/features/F07-reporting-balance_de.md
decisions:
  - "Range-Iteration is day-based (each range-day contributes committed_voluntary/7.0 if covered by an active contract); edge-weeks contribute pro-rata for the days that fall inside the range"
  - "contract_weeks_count_in_range counts an ISO-week if at least one range-day of that week is covered by an active contract (expected_hours=0 counts, per D-F1-01)"
  - "REST handler parses ISO YYYY-MM-DD in-line (Precedent: rest/src/toggle.rs); invalid format or from_date > to_date → HTTP 400"
  - "ExtraHours loading iterates from_year..=to_year for range-straddle support; per-row date filter is done in voluntary_ist_total_in_range"
  - "committed_voluntary_prorata_for_week (per-week helper) is retained as an internal per-week building block used by the mid-week-change test"
metrics:
  duration_seconds: 791
  duration_human: "~13 minutes"
  completed_utc: "2026-07-10T05:21:50Z"
  tasks_completed: 5
  files_modified: 10
  commits: 1
  tests_added_or_updated: 12
status: complete
---

# Phase 54 Plan 07: Voluntary-Stats Range-Semantik (Gap-Closure G1) Summary

**One-liner:** VoluntaryStatsService rechnet ab jetzt uber eine echte Date-Range (from_date, to_date) statt eines vollen ISO-Jahres — das 177h-Full-Year-Ubersoll aus 54-UAT.md Gap G1 ist damit geschlossen.

## Objective

Gap G1 (Blocker aus 54-UAT.md) schliessen: Die alte `year`-basierte Aggregation lieferte fur einen Mitarbeiter mit 5h/Woche committed_voluntary ab KW 18 einen "Freiwillige Sollstunden"-Wert von 177h uber das ganze ISO-Jahr — waehrend die Employee-Report-Chain nur bis zur aktuellen KW rechnet. Der Range-Cutoff bringt beide Ketten wieder in Sync: fur einen Range Jan–07-10 sind das ~54h statt 177h.

User-Zitat aus 54-UAT.md: *"Das muss immer so sein beim Report. Alleine schon wegen dem Abrechnungszeitraum."*

## Pure-fn-Signaturen (Range-fahig)

Alte Full-Year-fns entfernt, 3 neue Range-fns direkt unter `committed_voluntary_prorata_for_week` (der als per-week Baustein bleibt):

```rust
/// Phase 54 Gap-Closure G1 (VOL-STAT-01 / VOL-ACCT-01-Ist, D-54-DM-02)
pub fn voluntary_ist_total_in_range(
    extra_hours: &[ExtraHours],
    from_date: ShiftyDate,
    to_date: ShiftyDate,
) -> f32;

/// Phase 54 Gap-Closure G1 (VOL-STAT-01-Nenner, D-F1-01)
pub fn contract_weeks_count_in_range(
    working_hours: &[EmployeeWorkDetails],
    from_date: ShiftyDate,
    to_date: ShiftyDate,
) -> u32;

/// Phase 54 Gap-Closure G1 (VOL-ACCT-01-Soll, D-F2-01)
pub fn committed_voluntary_target_in_range(
    working_hours: &[EmployeeWorkDetails],
    from_date: ShiftyDate,
    to_date: ShiftyDate,
) -> f32;
```

Semantik:
- **voluntary_ist_total_in_range**: filtert `deleted.is_none()`, `category=VolunteerWork`, `source=Manual`, UND `eh.date_time.date()` in `[from ..= to]`.
- **contract_weeks_count_in_range**: iteriert Range Tag-fur-Tag, gruppiert per ISO-Woche via `to_iso_week_date()`, zaehlt jede ISO-Woche mit mindestens einem Vertragstag im Range-Overlap.
- **committed_voluntary_target_in_range**: iteriert jeden Tag im Range, sucht aktiven Vertrag (tages-basiert), summiert `committed_voluntary / 7.0` je Tag. Edge-Weeks tragen so tages-genau bei (D-F2-01 Pro-Rata bleibt).

## Trait- + Impl-Signatur-Diff

**service/src/voluntary_stats.rs — Trait:**

```diff
- async fn get_voluntary_stats(
-     &self,
-     sales_person_id: Uuid,
-     year: u32,
-     context: Authentication<Self::Context>,
-     tx: Option<Self::Transaction>,
- ) -> Result<VoluntaryStats, ServiceError>;
+ async fn get_voluntary_stats(
+     &self,
+     sales_person_id: Uuid,
+     from_date: ShiftyDate,
+     to_date: ShiftyDate,
+     context: Authentication<Self::Context>,
+     tx: Option<Self::Transaction>,
+ ) -> Result<VoluntaryStats, ServiceError>;
```

**service_impl/src/voluntary_stats.rs — Impl:**
- HR-Gate-First-Check unveraendert (D-AVG-05).
- Non-HR-None-Redaktion unveraendert (VOL-STAT-02 / VOL-ACCT-02).
- ExtraHours-Load: `for year in from_year..=to_year` iterativ, danach Filter auf `sales_person_id`. Range-Straddle wird so korrekt behandelt.
- Aggregation ruft die 3 neuen Range-fns; `contract_weeks == 0 → ist_per_contract_week = 0.0` (Divisions-Guard bleibt).

## REST-Query-Vertrag

`GET /report/{id}/voluntary-stats?from_date=YYYY-MM-DD&to_date=YYYY-MM-DD`

- Beide Parameter sind inklusive ISO-8601-Daten.
- Parser: Length-Check (10) + Delimiter-Check (`-` an Position 4 und 7) + `time::Date::from_calendar_date` (Praezedenz `rest/src/toggle.rs` Zeilen 350–377).
- Ungueltiges Format oder `from_date > to_date` → HTTP 400 mit `text/plain`-Body.
- `#[utoipa::path]` neu dokumentiert (params + 400-Response).

## Test-Uebersicht

Insgesamt **18 Unit-Tests** im `service_impl/src/test/voluntary_stats.rs` + **2 Integration-Tests** in `shifty_bin/src/integration_test/voluntary_stats.rs`.

### Pure-fn-Tests (Regressions umgestellt + neue Range-Tests)

| Test | Assertion | Ergebnis |
| --- | --- | --- |
| `f1_ist_manual_only_20h` | 5 Manual-Rows × 4h in Full-Year 2026 → 20.0h | ✓ |
| `f1_ist_rebooking_pair_invariant_vol_acct_03` | Rebooking-Pair neutral fuer F1-Ist | ✓ |
| `f2_soll_zero_when_no_committed_voluntary` | committed=0 → 0.0 | ✓ |
| `f2_soll_prorata_midweek_change_d_f2_01` | Mid-Week Vertragswechsel 3/7·7 + 4/7·14 = 11.0 | ✓ |
| `contract_weeks_zero_expected_counts_d_f1_01` | expected_hours=0 zaehlt trotzdem (6 Wochen) | ✓ |
| `contract_weeks_empty_working_hours_returns_zero` | leere Liste → 0 | ✓ |
| `f2_soll_iso_week_53_year_boundary_d_f2_01` | Full-Year 2026 = 365/7 ≈ 52.14; 2025 = 362/7 ≈ 51.71 | ✓ |
| `f1_ist_and_f2_soll_share_iso_week_semantics_d_f1_01_kongruenz` | Kongruenz-Test | ✓ |

### Neue Gap-Closure-G1-Range-Tests

| Test | Range | Erwartung | Ergebnis |
| --- | --- | --- | --- |
| `range_regression_full_year_2025_matches_old_semantics` | 2025-01-01..=2025-12-31, committed=1.0 KW1..=52 | soll ≈ 51.71 (362 aktive Tage / 7); contract_weeks=52 | ✓ |
| `range_regression_full_year_2026_matches_old_semantics` | 2026-01-01..=2026-12-31, committed=1.0 KW1..=53 | soll ≈ 52.14 (365 aktive Tage / 7); contract_weeks=53 | ✓ |
| `range_edge_week_start_midweek_wednesday_kw21_2026` | 2026-05-20 (Mi KW21) .. 2026-05-31 (So KW22), committed=7.0 | soll = 5.0 + 7.0 = 12.0; contract_weeks=2 | ✓ |
| `range_edge_week_end_midweek_thursday_kw21_2026` | 2026-05-18 (Mo KW21) .. 2026-05-21 (Do KW21), committed=7.0 | soll = 4.0; contract_weeks=1 | ✓ |
| `range_five_h_per_week_since_may_scenario_2026_until_kw28` | 2026-01-01..=2026-07-10, committed=5.0 ab KW18 | soll ≈ 5·75/7 = 53.57; **soll < 60 (Regression-Gate gegen 177h)**; contract_weeks=11 | ✓ |
| `range_before_contract_start_returns_zero` | 2026-01-01..=2026-01-07, Vertrag ab KW18 | soll=0.0, contract_weeks=0 | ✓ |
| `range_full_year_shows_full_annual_target_regression_lock_177` | Full-Year 2026, committed=5.0 ab KW18 | soll ≈ 5·245/7 = 175.0 (Referenz fur den alten Full-Year-Wert) | ✓ |

### Service-Tests (mockall)

| Test | Assertion | Ergebnis |
| --- | --- | --- |
| `service_non_hr_returns_all_none_vol_stat_02` | Non-HR → alle Felder None, kein DAO-Call | ✓ |
| `service_hr_returns_some_and_delegates_to_pure_fns` | HR Range KW10..=13, 10.0h ist, 4.0h soll, delta=6.0, ist/week=2.5 | ✓ |
| `service_zero_contract_weeks_yields_zero_per_week` | contract_weeks=0 → ist/week=0.0 (kein NaN) | ✓ |

### Integration-Tests (`shifty_bin`)

| Test | URL | Assertion | Ergebnis |
| --- | --- | --- | --- |
| `rest_voluntary_stats_hr_returns_populated_fields` | `?from_date=2026-01-01&to_date=2026-12-31` | contract_weeks=4, ist=8.0, soll=8.0 (Float-Toleranz 1e-3), delta=0.0, ist/week=2.0 | ✓ |
| `rest_voluntary_stats_non_hr_returns_all_null` | `?from_date=2026-01-01&to_date=2026-12-31` | alle Felder null (kein 403) | ✓ |

## Verification-Log

### `cargo build --workspace`

```
$ SQLX_OFFLINE=true cargo build --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 33.95s
```

### `cargo test -p service_impl --lib voluntary_stats::`

```
running 18 tests
test test::voluntary_stats::f1_ist_manual_only_20h ... ok
test test::voluntary_stats::f1_ist_rebooking_pair_invariant_vol_acct_03 ... ok
test test::voluntary_stats::f1_ist_and_f2_soll_share_iso_week_semantics_d_f1_01_kongruenz ... ok
test test::voluntary_stats::f2_soll_prorata_midweek_change_d_f2_01 ... ok
test test::voluntary_stats::f2_soll_zero_when_no_committed_voluntary ... ok
test test::voluntary_stats::f2_soll_iso_week_53_year_boundary_d_f2_01 ... ok
test test::voluntary_stats::contract_weeks_zero_expected_counts_d_f1_01 ... ok
test test::voluntary_stats::contract_weeks_empty_working_hours_returns_zero ... ok
test test::voluntary_stats::range_regression_full_year_2025_matches_old_semantics ... ok
test test::voluntary_stats::range_regression_full_year_2026_matches_old_semantics ... ok
test test::voluntary_stats::range_edge_week_start_midweek_wednesday_kw21_2026 ... ok
test test::voluntary_stats::range_edge_week_end_midweek_thursday_kw21_2026 ... ok
test test::voluntary_stats::range_five_h_per_week_since_may_scenario_2026_until_kw28 ... ok
test test::voluntary_stats::range_before_contract_start_returns_zero ... ok
test test::voluntary_stats::range_full_year_shows_full_annual_target_regression_lock_177 ... ok
test test::voluntary_stats::service_tests::service_non_hr_returns_all_none_vol_stat_02 ... ok
test test::voluntary_stats::service_tests::service_hr_returns_some_and_delegates_to_pure_fns ... ok
test test::voluntary_stats::service_tests::service_zero_contract_weeks_yields_zero_per_week ... ok

test result: ok. 18 passed; 0 failed; 0 ignored
```

### `cargo test -p shifty_bin voluntary_stats`

```
running 2 tests
test integration_test::voluntary_stats::rest_voluntary_stats_hr_returns_populated_fields ... ok
test integration_test::voluntary_stats::rest_voluntary_stats_non_hr_returns_all_null ... ok

test result: ok. 2 passed; 0 failed; 0 ignored
```

### `cargo test --workspace`

Alle 17 Test-Suites gruen, insgesamt 906 Tests (davon 755 in `service_impl`, 66 in `shifty_bin`).

### `cargo clippy --workspace -- -D warnings`

```
    Checking service v2.5.1-dev
    Checking rest-types v2.5.1-dev
    Checking service_impl v2.5.1-dev
    Checking rest v2.5.1-dev
    Checking shifty_bin v2.5.1-dev
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.97s
```

Zero warnings. Pflicht-Gate (MEMORY `feedback_clippy_gate`) gruen.

### Grep-Sanity

```
$ grep -rn "voluntary_ist_total_for_year|committed_voluntary_target_for_year|contract_weeks_count\b" \
    service/ service_impl/ rest/ shifty_bin/ rest-types/
(0 Treffer)

$ grep -c "?from_date=" docs/features/F14-rebooking.md docs/features/F14-rebooking_de.md
docs/features/F14-rebooking.md:1
docs/features/F14-rebooking_de.md:1
```

## Docs-Freshness-Diff

### `docs/features/F14-rebooking.md` (EN)

- Section 3 (Marker-Filter Rule): Reader-Regel und Balance-Neutralitat-Guarantee referenzieren jetzt `voluntary_ist_total_in_range` statt `voluntary_ist_total_for_year` (mit Erwahnung der Umbenennung in Plan 54-07 Gap G1).
- Section 5 (Pure functions in `service_impl::reporting`): 3 alte Full-Year-Signaturen durch 3 Range-Signaturen ersetzt; `committed_voluntary_prorata_for_week` als "internal per-week building block for debug tests" gekennzeichnet.
- **Neuer Rationale-Absatz:** "Range-based aggregation (Phase 54 Gap G1): consistent with `ReportingService::get_report_for_employee_range`; edge weeks contribute pro-rata for the days that fall inside the range. Without the cutoff, a 5h/week voluntary commitment starting in May yielded a full-year target that overshot the actual reporting range by ~4x (~177h vs. the realistic ~54h for a Jan–July window). See 54-UAT.md gap G1."
- Section 6 (REST): Tabellen-Zeile `?year=YYYY` → `?from_date=YYYY-MM-DD&to_date=YYYY-MM-DD`; neuer "Query contract"-Absatz erklärt HTTP-400-Verhalten.

### `docs/features/F14-rebooking_de.md` (DE)

- Analoge Anpassungen an denselben Sektionen, deutschsprachig.
- Rationale-Absatz: "Range-basierte Aggregation (Phase 54 Gap G1): konsistent mit `ReportingService::get_report_for_employee_range`; Edge-Weeks tragen pro-rata für die Tage im Range bei. Ohne Cutoff lieferte eine 5h/Woche-Zusage ab Mai ein Full-Year-Ziel, das den tatsächlichen Report-Zeitraum um ~4x überschoss (~177h vs. realistisch ~54h für Jan–Juli). Siehe 54-UAT.md Gap G1."
- Section 6: Query-Vertrag-Absatz erklärt HTTP-400-Verhalten.

### `docs/features/F07-reporting-balance.md` + `F07-reporting-balance_de.md`

Ein Reference-Bullet, der `voluntary_ist_total_for_year(..)` als ersten `source='manual'`-Reader erwähnt, wurde auf `voluntary_ist_total_in_range(..)` mit Rename-Hinweis aktualisiert. Damit gibt es keine `_for_year`-Referenzen in Rust-Code und keine `?year=`-URLs mehr in Docs.

## Deviations from Plan

**1. [Rule 1 – Bug] Float-Precision-Bounce im Integration-Test**
- **Found during:** Task 4 verification.
- **Issue:** Der bestehende `rest_voluntary_stats_hr_returns_populated_fields` erwartete via `assert_eq!(to.soll_total, Some(8.0))` einen bit-exakten Match. Die tages-basierte Range-Summe (28 × 2.0/7.0) landet bei `7.9999986` — mathematisch identisch, floating-point-technisch ≠ 8.0.
- **Fix:** Assertions auf `(x - expected).abs() < 1e-3` mit `Option::expect` umgestellt (Praezedenz existierende Pure-fn-Tests).
- **Files modified:** `shifty_bin/src/integration_test/voluntary_stats.rs`.
- **Rechtfertigung:** Rule 1 (Bug im nachfolgenden Testcode, direkt caused durch die neue tages-basierte Semantik). Kein User-Kontakt.

**2. [Rule 2 – Missing Critical Functionality] F07-Docs-Freshness**
- **Found during:** Task 5 grep-sanity.
- **Issue:** `docs/features/F07-reporting-balance.md` + `_de.md` referenzierten in ihrer Description der `source`-Migration den alten fn-Namen `voluntary_ist_total_for_year(..)` als ersten Live-Konsument des `source='manual'`-Filters. Nach dem Rename wäre das Drift (MEMORY `feedback_docs_always_current_no_followup`).
- **Fix:** Beide F07-Docs-Files aktualisiert (EN + DE synchron): `voluntary_ist_total_in_range(..)` mit "Plan 54-03 introduced as … renamed in Plan 54-07 Gap G1".
- **Files modified:** `docs/features/F07-reporting-balance.md`, `docs/features/F07-reporting-balance_de.md`.
- **Rechtfertigung:** Rule 2 (Docs-Freshness ist harte Regel, kein Follow-up).

Keine anderen Deviations.

## Auth-Gates

Keine Auth-Gates aufgetreten — komplett offline (SQLX_OFFLINE=true, in-memory SQLite).

## Threat Flags

Keine neuen Threat-Flags — die Query-Parameter sind mit explizitem Length-/Delimiter-Check + `time::Date::from_calendar_date` gehardened (T-54-07-01 mitigate). Keine neuen Netzwerk-Endpunkte, keine neuen DB-Tabellen, keine neuen Cargo-Deps.

## Commits

```
7aefad3 feat(54-07): voluntary-stats — accept date range instead of ISO year (Gap G1)
```

Einziger Commit, alle 5 Tasks in einem atomaren Wrap-Commit inklusive Docs-Freshness (F14 EN+DE + F07 EN+DE). Präzedenz: MEMORY `feedback_docs_always_current_no_followup` — Docs im gleichen Commit wie Code.

## Wave-1-Zwischenzustand

Der Frontend-Aufrufer in `shifty-dioxus/src/api.rs` verwendet nach diesem Plan noch die alte `?year=YYYY`-URL. Er faellt jetzt auf HTTP 400 zurueck; der bestehende `unwrap_or_default()`-Pfad in `service/employee.rs` fuellt den Store mit `VoluntaryStats::default()`, was aequivalent zur Non-HR-Redaktion ist (kein UI-Crash). Plan 54-08 zieht in Wave 2 nach.

## Self-Check: PASSED

- `service_impl/src/reporting.rs` — 3 neue Range-fns present: bestätigt via grep (siehe Verification-Log).
- `service/src/voluntary_stats.rs` — Trait-Signatur (from_date, to_date): bestätigt via cargo build (kein E0308).
- `service_impl/src/voluntary_stats.rs` — Impl ruft die 3 Range-fns: bestätigt via cargo build + cargo test.
- `service_impl/src/test/voluntary_stats.rs` — 18 Tests grün (10 alte + 7 neue Range + 3 Service, davon 1 auf Range umgestellt).
- `rest/src/report.rs` — VoluntaryStatsRequest { from_date, to_date }: bestätigt via cargo build + Integration-Test.
- `shifty_bin/src/integration_test/voluntary_stats.rs` — 2 Tests grün mit `?from_date=…&to_date=…`.
- `docs/features/F14-rebooking.md` + `_de.md` — Range-Signaturen + REST-Vertrag + Rationale synchron: bestätigt via grep.
- `docs/features/F07-reporting-balance.md` + `_de.md` — fn-Rename dokumentiert: bestätigt via grep.
- Commit `7aefad3` exists: bestätigt via `git log --oneline -1`.
