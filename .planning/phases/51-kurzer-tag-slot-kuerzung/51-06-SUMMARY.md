---
phase: 51-kurzer-tag-slot-kuerzung
plan: 06
subsystem: reporting
tags: [rust, sqlx, shortday, shiftplan_report, dao, clip, gate, stichtag, snapshot-immunity]

# Dependency graph
requires:
  - phase: 51
    provides: "Slot::clip_to (P01), shortday_gate helpers (P02), ephemer-Slot pattern from BlockService (P04) + BookingInformation (P05)"
provides:
  - "ShiftplanReportRawRow entity (row-per-booking) + three raw-row DAO trait methods"
  - "Rust-Layer aggregation in ShiftplanReportServiceImpl (drop-in for existing Service signature)"
  - "Balance/Ist-Stunden respektieren pro-Row Slot::clip_to + Stichtag-Gate"
  - "Deletion of the SUM-based DAO trait+impl (with pre-existing /60.0 SQL bug — removed as dead code)"
  - "DI: SpecialDayService + ToggleService as ShiftplanReportServiceDeps"
affects: [reporting, booking_information, block_report, balance-views]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "DAO returns raw rows; Service aggregates + clips in Rust (D-51-08 canonical)"
    - "Toggle-Unauthorized-tolerance in service methods called from mock-auth (mirror of reporting.rs HCFG-02)"

key-files:
  created:
    - "service_impl/src/test/shiftplan_report.rs"
    - ".planning/phases/51-kurzer-tag-slot-kuerzung/51-06-SUMMARY.md"
  modified:
    - "dao/src/shiftplan_report.rs"
    - "dao_impl_sqlite/src/shiftplan_report.rs"
    - "service_impl/src/shiftplan_report.rs"
    - "service_impl/src/test/mod.rs"
    - "shifty_bin/src/main.rs"
    - ".sqlx/query-*.json (3 SUM queries deleted, 3 raw queries added)"

key-decisions:
  - "Delete-branch (Task 5 dual-branch decision): the three old SUM DAO methods are gone from trait+impl. Grep audit showed the ONLY in-workspace consumer was service_impl/src/shiftplan_report.rs, which this refactor rewrites. Deletion also disposes of the pre-existing /60.0 SQL bug in the two SUM queries at dao_impl_sqlite lines 114 + 147 as dead code."
  - "Ephemeral Slot per Row: the aggregation loop builds a Slot { from, to, day_of_week, ...dummies } per DAO row and feeds it through shortday_gate::clip_slot_for_week. Only from/to/day_of_week/valid_from are read by the clip path — all other fields are Uuid::nil / defaults."
  - "Aggregation-Key uses DayOfWeek::to_number() (u8) because DayOfWeek doesn't implement Hash."
  - "Unauthorized-tolerance: read_active_from helper maps Err(Unauthorized) → Ok(None), mirroring reporting.rs:164-172 HCFG-02 pattern. Without it, 7 integration tests running with Authentication::Full over mock-auth regressed."
  - "toggle_service construction in main.rs moved to before shiftplan_report_service to satisfy the new dep chain (Reporting/ShiftplanEdit reuse the same Arc handle)."

patterns-established:
  - "DAO raw-row + Service aggregation as the canonical approach for cross-entity time computations (replaces SQL SUM+GROUP BY + fragile STRFTIME math)"
  - "Snapshot-Immunität durch reine Live-Path-Refactors: persistente billing_period_sales_person-Rows werden NICHT neu berechnet, Version bleibt 12"

requirements-completed: [SHC-02, SHC-05, SHC-06]
completed: 2026-07-05
status: complete

metrics:
  duration: "~1h"
  tasks_completed: 6
  files_changed: 6
  tests_added: 7
  commits: 5
---

# Phase 51 Plan 06: Chain D — ShiftplanReportDao Rust-Layer-Refactor Summary

DAO liefert Roh-Zeilen pro Booking; `ShiftplanReportServiceImpl` aggregiert + clippt + gatet in Rust (D-51-06 / D-51-08). Balance-Views + Reporting sehen geclippte Ist-Stunden ohne Booking-Rewrite und ohne historisches Umschreiben vor `active_from`.

## Accomplishments

- **DAO Trait Delta (Task 1):** `ShiftplanReportRawRow { sales_person_id, booking_id, year, calendar_week, day_of_week, time_from, time_to }` plus die drei Methoden `extract_raw_shiftplan_report`, `extract_raw_quick_shiftplan_report`, `extract_raw_shiftplan_report_for_week`. Die alten `ShiftplanReportEntity` + `ShiftplanQuickOverviewEntity` und ihre drei SUM-Trait-Methoden wurden gelöscht (Task 5 Delete-Branch, siehe Konsumenten-Audit).

- **SQLite Impl (Task 2):** Drei raw-row Queries ohne `SUM`/`GROUP BY`. Beispiel-Skelett:
  ```sql
  SELECT sales_person.id as sales_person_id, booking.id as booking_id,
         booking.year, booking.calendar_week, slot.day_of_week,
         slot.time_from, slot.time_to
  FROM slot
  INNER JOIN booking ON (booking.slot_id = slot.id AND booking.deleted IS NULL)
  INNER JOIN sales_person ON booking.sales_person_id = sales_person.id
  LEFT JOIN shiftplan ON slot.shiftplan_id = shiftplan.id
  WHERE booking.year = ? AND booking.calendar_week = ?
    AND (shiftplan.is_planning = 0 OR shiftplan.is_planning IS NULL)
  ```
  `time_from`/`time_to` sind im SQLite als TEXT (`HH:MM:SS`) und werden in `TryFrom<&ShiftplanReportRawRowDb>` über `Time::parse(&s, &Iso8601::TIME)` in `time::Time` gehoben — kein `sqlx::Decode`-Cast (`time::Time` hat keinen direkten Decoder ohne Feature-Gate; slot.rs:40-41 hat das gleiche Muster).

- **Service Aggregation (Task 3):** Alle drei `extract_*`-Methoden fetchen den Toggle einmal, cachen `SpecialDayService::get_by_week` pro benötigter Woche, iterieren Roh-Rows und rufen pro Row `shortday_gate::clip_slot_for_week(&ephemer_slot, sds, year, week, active_from)` auf. Aggregations-Bucket ist `(sales_person_id, year, week, day_of_week_number)` (nur DoW-Nummer, weil `DayOfWeek` kein `Hash` hat). Kein `min_resources`-Faktor (Chain D zählt reine Personen-Stunden, D-51-06 Behavior-Note).

- **DI-Wiring (Task 3, main.rs):** `ShiftplanReportServiceDeps` bekommt `SpecialDayService` + `ToggleService`. `toggle_dao` + `toggle_service` in `shifty_bin/src/main.rs` sind ~60 Zeilen nach oben gerutscht (VOR `shiftplan_report_service`). Reporting + ShiftplanEdit greifen weiter denselben `toggle_service`-Handle ab, keine doppelte Instanziierung.

- **Tests (Task 4):** Sieben Tests in `service_impl/src/test/shiftplan_report.rs`, alle grün:
  - `test_extract_for_week_clips_at_shortday` — Clip greift (0,5h).
  - `test_extract_for_week_ungated_no_clip` — Gate off (1,0h Legacy).
  - `test_extract_for_week_post_cutoff_row_zero` — D-04 Zeile 3 (0h).
  - `test_extract_for_week_stichtag_boundary_inclusive` — SHC-06 inklusiv am Stichtag / Vortag exkludiert.
  - `test_extract_for_week_aggregates_multiple_bookings` — SHC-05: zwei Bookings summiert (2×0,5=1,0 mit Gate, 2×1,0=2,0 ohne).
  - `test_extract_range_multi_week_gate_mix` — pro-Woche-Gate.
  - `test_snapshot_schema_version_unchanged` — `CURRENT_SNAPSHOT_SCHEMA_VERSION == 12`.

- **Unauthorized-Tolerance (Task 6, follow-up fix):** `read_active_from`-Helper mappt `Err(ServiceError::Unauthorized)` → `Ok(None)`. Nach der ersten Runde von `cargo test --workspace` sind 7 integration-Tests regressed (`test_shiftyplan_hours_end_of_year`, `test_start_of_year`, `test_multiple_contracts`, `test_extra_hours_beginning_of_year`, `test_simple_shiftplan_entries`, `test_vacation_at_end_of_year`, `test_vacation_entitlement_calculcation`), weil `Authentication::Full` über mock-auth die User-ID-Anforderung von `ToggleService::get_toggle_value` nicht erfüllt. Semantisch ist das identisch zu einem unset Toggle → keine Cutoff-Konfiguration → Legacy. Muster kopiert aus `reporting.rs:164-172` (HCFG-02 holiday_auto_credit).

- **.sqlx/-Delta:**
  - Gelöscht: 3 alte SUM-Queries
    - `.sqlx/query-7181b8cc4b332a473228b2899284fd0c5fb1d19e976fc25156d2b5e0398a27d5.json`
    - `.sqlx/query-db614737853d8f53106b7074e0cc21572df2e7d71aacb74f5088a8361a94e64a.json`
    - `.sqlx/query-f34c28782d0ad962e3f474621feddac76a2ec0844bb3eff7d0ca09ffb6f3c5db.json`
  - Neu: 3 raw-row Queries
    - `.sqlx/query-3b2806e1fbdbbfb01b5dce843b50baf42e0861b6a338fa851c3acf82b7c97439.json`
    - `.sqlx/query-b68034abf0417724860dce161f6876c371aa3f483ae8dbe0f85294b94a85a479.json`
    - `.sqlx/query-f71e13426b396f8db58628fd0b52d04409065a85191825edd45a670cbda3408e.json`
  - `cargo sqlx prepare --workspace --check` ist grün (`.sqlx/` in sync mit den `query_as!`-Aufrufen).

## Konsumenten-Audit (Task 5 Delete-Branch, Evidenz)

Grep: `rg 'extract_shiftplan_report|extract_quick_shiftplan_report|extract_shiftplan_report_for_week' --type rust --glob '!shifty-dioxus/**'` VOR Task 1.

Ergebnisse mit direktem DAO-Zugriff (`.<method>()` an einem `ShiftplanReportDao`-Handle):

| Site | Kind | Aktion |
| --- | --- | --- |
| `dao/src/shiftplan_report.rs` (Trait-Def) | Trait selbst | ersetzt durch raw-Methoden |
| `dao_impl_sqlite/src/shiftplan_report.rs` (Trait-Impl) | Impl | ersetzt durch raw-Impl |
| `service_impl/src/shiftplan_report.rs` | Service ruft DAO | rewritten (Chain D) |

Alle anderen Treffer (`service_impl/src/reporting.rs:281,712,901`, `service_impl/src/booking_information.rs:369,537,709`, sowie die MockShiftplanReportService-Erwartungen in `test/reporting_*.rs`, `test/booking_information_*.rs`) rufen die **Service-Trait**-Methoden, nicht den DAO. Der Service-Trait bleibt unverändert (drop-in Signatur). Damit ist Deletion sicher — kein externer Konsument der SUM-DAO-Methoden bleibt zurück.

**Bonus:** Mit der Deletion verschwindet auch der pre-existente SQL-Bug in `dao_impl_sqlite/src/shiftplan_report.rs:114` und `:147` (fehlendes `/60.0` beim `time_from`-Minuten-Teil). PLAN-INDEX Warning 2 verzeichnet ihn — jetzt tot.

## Snapshot-Immunity

- `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt bei **12** (verifiziert via `grep -q` in `service_impl/src/billing_period_report.rs`; Test G asserted es zusätzlich).
- `billing_period_report.rs` — der Snapshot-Reader — wurde **nicht** angefasst. Persistierte `billing_period_sales_person`-Rows werden weiter unverändert gelesen.
- Chain-D-Refactor ist Live-Path-only: neu-live-berechnete Ist-Stunden für Zeiträume ab `active_from` sind geclippt. Historische Zeiträume vor `active_from` bleiben live-ungeclippt (Gate). Alte Snapshots bleiben es sowieso (der Reader konsumiert Rows, nicht Live-Aggregat).

## Perf-Note (D-51-08)

Row-Traffic steigt jetzt pro Booking (statt einem GROUP-BY-Bucket pro Person+DoW pro Woche liefert der DAO eine Row pro Booking). Für Balance-Historien über mehrere Jahre plus alle SalesPersons wird das quadratisch mit `bookings × weeks`. SQLite ist lokal und die Aggregation läuft in Rust in einem einzigen `HashMap`-Pass — für die aktuelle Größenordnung (kleines Team, Ein-Jahres-Balance) verschmerzbar. Ggf. spätere Materialisierung als `billing_period_snapshot`-adjacenter Zwischenkache — out-of-scope für Phase 51.

## Deviations from Plan

Keine Rule-4-Deviations. Zwei Rule-3-Fixes (blocking):

**1. [Rule 3 - Blocking] time::Time hat kein sqlx::Decode**
- **Found during:** Task 2 (`cargo sqlx prepare --workspace`).
- **Issue:** Die vorgeschlagene `time_from: time::Time as "time_from: time::Time"`-Cast-Notation kompilierte nicht — `time::Time` implementiert `sqlx::Decode<'_, Sqlite>` nicht ohne extra Feature-Gate.
- **Fix:** DB-Row liest `time_from`/`time_to` als `String`; `TryFrom<&ShiftplanReportRawRowDb>` parst über `Time::parse(&s, &Iso8601::TIME)`. Muster aus `dao_impl_sqlite/src/slot.rs:40-41` übernommen.
- **Files modified:** `dao_impl_sqlite/src/shiftplan_report.rs`.
- **Commit:** `6baf071`.

**2. [Rule 3 - Blocking] DayOfWeek impl weder Hash noch Eq**
- **Found during:** Task 3 (Service-Aggregation-Loop).
- **Issue:** `HashMap<(Uuid, u32, u8, DayOfWeek), f32>` compile-error — `DayOfWeek: Hash` fehlt.
- **Fix:** Aggregations-Key nutzt `day_of_week.to_number()` (`u8`); beim Emit zurück-konvertiert via `DayOfWeek::from_number(dow_num)`.
- **Files modified:** `service_impl/src/shiftplan_report.rs`.
- **Commit:** `79cad95` (Teil von Task 3).

**3. [Rule 3 - Blocking] Integration-Tests laufen ohne User-Kontext**
- **Found during:** Task 6 (`cargo test --workspace`).
- **Issue:** `ToggleService::get_toggle_value` unter mock-auth mit `Authentication::Full` = `Unauthorized`. 7 integration tests regressed.
- **Fix:** `read_active_from`-Helper mit HCFG-02-Pattern (Unauthorized → None).
- **Files modified:** `service_impl/src/shiftplan_report.rs`.
- **Commit:** `f654613`.

## Commits

- `a6de121` — feat(51-06): Chain D — raw-row DAO trait (delete SUM entities)
- `6baf071` — feat(51-06): Chain D — raw-row SQL impls + .sqlx regenerate
- `79cad95` — feat(51-06): Chain D — ShiftplanReport service aggregates + clips + gates
- `59497a6` — test(51-06): Chain D — Aggregation + Clip + Gate + Snapshot-Immunity
- `f654613` — fix(51-06): Chain D — tolerate Unauthorized on toggle read (HCFG-02 pattern)

## Verification

- `cargo test --workspace` — green (all crates including 64 integration tests).
- `cargo clippy --workspace -- -D warnings` — clean.
- `cargo sqlx prepare --workspace --check` — clean.
- `grep -q 'pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;' service_impl/src/billing_period_report.rs` — 1 match.
- Task-4-Tests: 7/7 green (`test::shiftplan_report::*`).

## Self-Check: PASSED

- `.planning/phases/51-kurzer-tag-slot-kuerzung/51-06-SUMMARY.md` — FOUND.
- `service_impl/src/test/shiftplan_report.rs` — FOUND.
- Commits `a6de121`, `6baf071`, `79cad95`, `59497a6`, `f654613` — all FOUND in `git log`.
