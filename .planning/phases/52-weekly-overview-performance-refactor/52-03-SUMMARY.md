---
phase: 52-weekly-overview-performance-refactor
plan: 03
subsystem: [service, service_impl, dao, dao_impl_sqlite]
tags: [WOP-01, WOP-05, D-52-06, year-batch, sqlx-prepare]
requires: [52-02]
provides:
  - "ShiftplanReportService::extract_shiftplan_report_for_year"
  - "ShiftplanReportDao::extract_raw_shiftplan_report_for_year"
  - "ExtraHoursService::find_by_year"
  - "ExtraHoursDao::find_by_year"
affects: [Wave 4 (Plan 05 get_weekly_summary Bulk-Loads)]
tech-stack:
  added: []
  patterns: [Additive Trait Extension, Year-Batch SQL, sqlx prepare workflow]
key-files:
  created: []
  modified:
    - service/src/shiftplan_report.rs
    - service_impl/src/shiftplan_report.rs
    - dao/src/shiftplan_report.rs
    - dao_impl_sqlite/src/shiftplan_report.rs
    - service/src/extra_hours.rs
    - service_impl/src/extra_hours.rs
    - dao/src/extra_hours.rs
    - dao_impl_sqlite/src/extra_hours.rs
    - service_impl/src/test/shiftplan_report.rs
    - service_impl/src/test/extra_hours.rs
    - service_impl/src/test/absence_conversion.rs
  new-sqlx:
    - .sqlx/query-b70283681856cbf62db33bb96ce86e4bfe08e271ebe679c4c495561d86b3d5dc.json
decisions:
  - "OQ-1 → Option a bestätigt: ExtraHoursService::find_by_year als neue Trait-Methode (symmetrisch zu find_by_week), NICHT find_all + Filter"
  - "find_by_year DAO-Impl reuses the find_by_week SQL text (identisch bis auf Bind-Werte) → nur 1 neuer .sqlx-Cache-File statt 2"
metrics:
  duration_min: ~35
  completed_at: 2026-07-05
  new_tests: 7
  new_sqlx_files: 1
status: complete
---

# Phase 52 Plan 03: Year-Batch Trait Methods (Additive) Summary

Zwei neue Batch-Trait-Methoden für Basic-Services, die die für Wave 4 nötigen
Jahres-Aggregate in EINEM DAO-Roundtrip statt 55×Wochen-Iteration liefern.
Rein additiv — bestehende `_for_week`/`find_by_week`-Signaturen unverändert.

## What was built

### 1. `ShiftplanReportService::extract_shiftplan_report_for_year`

- **Trait** (`service/src/shiftplan_report.rs`): neue async Methode direkt
  unter `extract_shiftplan_report_for_week`. Signatur:
  ```rust
  async fn extract_shiftplan_report_for_year(
      &self,
      year: u32,
      context: Authentication<Self::Context>,
      tx: Option<Self::Transaction>,
  ) -> Result<Arc<[ShiftplanReportDay]>, ServiceError>;
  ```
- **DAO-Trait + Impl** (`dao/src/shiftplan_report.rs`,
  `dao_impl_sqlite/src/shiftplan_report.rs`): neue Query
  `extract_raw_shiftplan_report_for_year(year, tx)` — identisch zu
  `_for_week`-Query, nur ohne `calendar_week = ?`-Filter, mit
  `ORDER BY booking.calendar_week, slot.day_of_week` für deterministische
  Reihenfolge.
- **Service-Impl** (`service_impl/src/shiftplan_report.rs`): Aggregation
  nach `(sales_person_id, calendar_week, day_of_week)`. `hours_for_row` und
  Stichtag-Gate 1:1 aus `_for_week` übernommen. SpecialDays werden
  EINMAL via `special_day_service.get_by_year(year, ...)` geladen (D-52-06)
  und in eine `HashMap<u8, Vec<SpecialDay>>` pro Woche gruppiert — kein
  N-Roundtrip.

### 2. `ExtraHoursService::find_by_year`

- **Trait** (`service/src/extra_hours.rs`): neue Methode unter `find_by_week`.
- **DAO-Trait + Impl**: neue Methode mit Filter
  `WHERE date_time >= '{year}-01-01T00:00:00' AND date_time < '{year+1}-01-01T00:00:00' AND deleted IS NULL`.
  Interessanter sqlx-Nebeneffekt: Die SQL-Text-Sequenz ist identisch zu
  `find_by_week` (nur andere Bind-Werte) → sqlx erzeugt keinen neuen
  `.sqlx/query-*.json`-File, der bestehende Cache-Eintrag wird
  wiederverwendet.
- **Service-Impl** (`service_impl/src/extra_hours.rs`): dünner Wrapper analog
  `find_by_week`. Auth-Gate `check_only_full_authentication` (T-52-05
  Mitigation, identisch zu Wochenvariante).

## sqlx prepare bestätigt

- Kommando: `nix develop --command cargo sqlx prepare --workspace` in
  `shifty-backend/` (nicht dioxus-Shell).
- Neue prepared-statement-Files: **1**
  (`query-b70283681856cbf62db33bb96ce86e4bfe08e271ebe679c4c495561d86b3d5dc.json`
  für `extract_raw_shiftplan_report_for_year`).
- `find_by_year` teilt sich den Cache-Eintrag mit `find_by_week` (identischer
  SQL-Text-Hash). Bestätigt via `SQLX_OFFLINE=true cargo build --workspace
  --tests` — clean.

## Additivität — git diff Beweis

`git diff bdbdc28~1 -- service/src/shiftplan_report.rs service/src/extra_hours.rs`
zeigt ausschließlich `+`-Zeilen im Signaturbereich. Bestehende
`extract_shiftplan_report_for_week`- und `find_by_week`-Signaturen sind
byte-exakt unverändert. Kein Removal, kein Rename.

## Tests

7 neue Unit-Tests, alle grün:

**`service_impl::test::shiftplan_report`** (4 Tests):
1. `test_extract_for_year_multi_week_ungated` — Aggregation pro
   `(sp, week, day)` über zwei Wochen (Gate off).
2. `test_extract_for_year_clips_only_in_shortday_week` — ShortDay-Cutoff
   nur in W31, W30 ungeklippt — beweist korrekte SpecialDay-Gruppierung
   pro Woche.
3. `test_extract_for_year_matches_for_week_sum` — Semantik-Äquivalenz-
   Sanity-Check: `_for_year` == `_for_week` für Ein-Woche-Datensatz.
4. `test_extract_for_year_tolerates_toggle_unauthorized` — Regression
   für Gap-Closure aus Phase 51: Toggle-Unauthorized → Legacy off, kein
   401.

**`service_impl::test::extra_hours`** (3 Tests):
5. `test_find_by_year_happy_path` — DAO liefert 2 Entities, Service
   mapped nach `ExtraHours::from`, Ordnung bleibt DAO-Ordnung.
6. `test_find_by_year_rejects_non_full_auth` — `check_only_full_authentication`
   → Forbidden greift (T-52-05).
7. `test_find_by_year_empty` — leeres Jahr → leeres `Arc<[]>`.

## Gates

| Gate                                            | Status |
| ----------------------------------------------- | ------ |
| `cargo build --workspace`                       | GREEN  |
| `cargo test --workspace` (64 integration + Unit)| GREEN  |
| `cargo clippy --workspace -- -D warnings`       | GREEN  |
| `SQLX_OFFLINE=true cargo build --workspace --tests` | GREEN  |
| Wave-1 Golden-Snapshot-Fixture-Test             | GREEN  |
| Additivität (`_for_week`/`find_by_week` byte-exact) | ERFÜLLT |

## Deviations from Plan

**Rule 3 auto-fix — Stub-Impl in `absence_conversion.rs`:**
- `service_impl/src/test/absence_conversion.rs::StubExtraHoursService` ist eine
  manuelle Trait-Impl (nicht automock). Nach dem Trait-Add fehlte dort
  `find_by_year`. Fix: `unimplemented!("not needed in integration test")`-Stub
  hinzugefügt — kein Test-Verhalten geändert.
- Files: `service_impl/src/test/absence_conversion.rs`
- Commit: `cd44dc7`

Sonst keine Abweichungen. Plan wurde exakt wie geschrieben ausgeführt.

## Decisions bestätigt (OQ-1)

RESEARCH OQ-1 (`.planning/phases/52-.../52-RESEARCH.md`):
- **Option a gewählt**: `ExtraHoursService::find_by_year` als eigene Trait-
  Methode.
- **Option b verworfen** (`find_all` + In-Memory-Filter): würde die gesamte
  ExtraHours-Historie pro `get_weekly_summary`-Call laden — skaliert bei
  wachsendem Datenbestand schlechter als der jahresgefilterte SQL-Filter,
  der den bestehenden Full-Scan-Pfad nutzt. Symmetrisch zum bestätigten
  D-52-06-Muster für `ShiftplanReportService`.

## Threat mitigations

| Threat ID | Status |
| --------- | ------ |
| T-52-05 (Information Disclosure — Auth-Bypass) | mitigated — Auth-Check-Kopie identisch zu `_for_week` (`check_only_full_authentication` in extra_hours, `read_active_from` in shiftplan_report). Verifikation via `test_find_by_year_rejects_non_full_auth`. |
| T-52-06 (DoS — Full-Scan ohne Index) | accepted — dokumentiert; die Ersparnis kommt aus 55×→2× Roundtrips, nicht aus Index. |
| T-52-07 (Tampering — sqlx prepare) | mitigated — `.sqlx`-Delta committet, `SQLX_OFFLINE=true cargo build` grün. |

## Handoff to Wave 4/5

Die neuen Methoden sind bereit zum Konsum:

- **Wave 3 Plan 04** (`ReportingService::get_year`): kann jetzt
  `extract_shiftplan_report_for_year` statt Wochen-Iteration nutzen.
- **Wave 4 Plan 05** (`get_weekly_summary`): kann `extract_shiftplan_report_for_year`
  + `find_by_year` in EINEM Bulk-Load pro Jahr aufrufen (plus 1× für
  Spillover in `year+1`), statt 55×`_for_week` + 55×`find_by_week`.

Spillover-Handling verbleibt beim Consumer (D-52-05) — die neuen Methoden
liefern KEINE `until_week` oder `year_range`-Sondersignatur, nur reines
`year`.

## Commits

- `bdbdc28` — feat(52-03): add ShiftplanReportService::extract_shiftplan_report_for_year (WOP-01)
- `cd44dc7` — feat(52-03): add ExtraHoursService::find_by_year (WOP-01)

## Self-Check: PASSED

- `.planning/phases/52-weekly-overview-performance-refactor/52-03-SUMMARY.md`: FOUND (this file)
- Commit `bdbdc28`: FOUND
- Commit `cd44dc7`: FOUND
- `.sqlx/query-b70283681856cbf62db33bb96ce86e4bfe08e271ebe679c4c495561d86b3d5dc.json`: FOUND
- Files created/modified: all present in commits
