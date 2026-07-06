---
phase: 53-freiwilligen-abwesenheiten-jahresansicht
plan: 03
subsystem: frontend
status: complete
tags:
  - frontend
  - dioxus
  - weekly-overview
  - union-merge
  - vaa-01
  - vaa-04
requirements:
  - VAA-01
  - VAA-04
dependencies:
  requires:
    - 53-01
    - 53-02
  provides:
    - "Union-Merge Freiwilliger + Bezahlter in state::WeeklySummary.sales_person_absences"
    - "case-insensitive Name-Sort der Absencen-Liste"
  affects:
    - "shifty-dioxus Rendering-Layer (page/weekly_overview.rs:126 konsumiert Union unveraendert)"
tech_stack:
  added: []
  patterns:
    - "Union-Merge im From<TO>-Mapper (Block-Expression → Vec::sort_by → Struct-Feld)"
key_files:
  created: []
  modified:
    - shifty-dioxus/src/state/weekly_overview.rs
decisions:
  - "D-53-04: Union-Vec case-insensitive nach name via sort_by(to_lowercase().cmp)"
  - "D-53-04: >= 0.1-Filter gilt fuer beide Quellen (Randfall Zusage=0 raus)"
  - "VAA-04-Lock: Rendering-Zeile in page/weekly_overview.rs:126 woertlich unveraendert"
metrics:
  duration_seconds: 416
  completed_date: 2026-07-06
  commits_new: 3
  files_modified: 1
  tests_added: 2
---

# Phase 53 Plan 03: Frontend-Union-Merge Summary

Der FE-Mapper `impl From<&WeeklySummaryTO> for state::WeeklySummary` baut die Anzeige-Liste `sales_person_absences` jetzt als Union aus zwei Quellen: (a) bestehender Bezahlten-Loop ueber `working_hours_per_sales_person` bleibt inhaltlich unveraendert (Regression-Lock VAA-03 #3), (b) neu iteriert er ueber das in Plan 02 befuellte DTO-Feld `WeeklySummaryTO.sales_person_absences`. Anschliessend case-insensitive Name-Sort. Rendering-Zeile in `page/weekly_overview.rs:126` bleibt woertlich unveraendert (VAA-04-Lock).

## What Was Built

- **Union-Refactor `From<&WeeklySummaryTO> for WeeklySummary`** (`shifty-dioxus/src/state/weekly_overview.rs` Zeilen 30-89): Statt Struct-Literal mit inline-`filter_map` jetzt Block-Expression mit `let mut v: Vec<SalesPersonAbsence>` → Bezahlten-Loop (unveraendert) → `v.extend(...)` fuer Freiwilligen aus DTO-Feld → `v.sort_by(|x, y| x.name.to_lowercase().cmp(&y.name.to_lowercase()))` → v ins Struct-Feld.
- **`make_to()`-Fixture erweitert** (Zeile ~100): neues Feld `sales_person_absences: Vec::new().into()` (Pitfall 5 gefixt — Struct-Literal-Compile-Break nach DTO-Erweiterung aus Plan 01).
- **Zweiter Fixture-Helper `make_to_with_paid_and_volunteer(...)`**: liefert ein `WeeklySummaryTO` mit einem Bezahlten via `working_hours_per_sales_person` und einem Freiwilligen via neuem DTO-Feld — Testbaustein fuer Union-Szenario.
- **Neuer Test `sales_person_absences_union_merges_paid_and_volunteers_sorted_by_name`**: baut Bezahlten "Anna" (absence 8.0) + Freiwilligen "Bob" (hours 5.0), verifiziert `.len() == 2`, `[0].name == "Anna"`, `[1].name == "Bob"` — belegt VAA-01 + D-53-04.
- **Neuer Test `bezahlter_bleibt_via_working_hours_pfad_sichtbar`**: Bezahlter "Anna" (absence 8.0), leeres DTO-Feld, verifiziert `.len() == 1`, `[0].name == "Anna"` — belegt Regression-Lock VAA-03 #3 aus FE-Sicht.

## TDD Cycle

- **RED (Task 1, Commit 1f42842)**: Fixture erweitert (Compile-Fix), Union-Test schreibt Assertion `.len() == 2`, faellt gegen den bezahlten-only Mapper (Bob aus DTO-Feld fehlt → `left: 1, right: 2`). Regression-Test bezahlter-bleibt gruen (bestehendes Verhalten). Bestehende Tests `committed_voluntary_hours_maps_from_to` + `available_hours_maps_from_overall_available_hours` gruen.
- **GREEN (Task 2, Commit 4da9a6b)**: Union-Refactor macht den Union-Test gruen. Bezahlten-Formel `effective_absence = absence_hours - holiday_hours + unavailable_hours >= 0.1` inhaltlich unveraendert. Alle 4 Tests gruen.
- **REFACTOR/GATE (Task 3)**: keine Code-Aenderung — nur Gate-Verifikation. Rendering-Lock grep gruen (genau eine Zeile in page/weekly_overview.rs:126), WASM-Build gruen, FE-Test-Suite komplett gruen (802 Tests), Backend-Regression (build + test + clippy) gruen.

## Commits

- `1f42842` — `test(53-03/T1): RED — make_to() fixture + Union-Merge test`
- `4da9a6b` — `feat(53-03/T2): GREEN — Union-Merge im From<&WeeklySummaryTO>-Mapper`
- (SUMMARY-Commit folgt: `docs(53-03): SUMMARY`)

## Gate Status

| Gate | Kommando | Status |
|------|----------|--------|
| VAA-04 Rendering-Lock | `grep -n 'format!("{}: {} {hours_short}", absence.name, format_hours(absence.absence_hours, 2))' shifty-dioxus/src/page/weekly_overview.rs` | ✓ genau 1 Match (Zeile 126) |
| WASM-Build | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` | ✓ green, no warnings |
| FE-Tests | `cd shifty-dioxus && cargo test` | ✓ 802 passed, 0 failed |
| Backend-Build | `cargo build --workspace` | ✓ green |
| Backend-Tests | `cargo test --workspace` | ✓ all suites green (732 + 11 + 64 + smaller) |
| Backend-Clippy | `cargo clippy --workspace -- -D warnings` | ✓ green, no warnings |

## Deviations from Plan

### Skipped Steps

**1. FE-Clippy-Gate NICHT ausgefuehrt (Runtime-Instruction-Override)**
- **Plan Task 3(c) forderte**: `cd shifty-dioxus && cargo clippy -p shifty-dioxus -- -D warnings` als hartes Gate.
- **Runtime-Instruction (spawn-time Prompt): "Dioxus Clippy NOT gated by CI — shifty-dioxus is excluded from backend Clippy and has ~198 pre-existing lints. Do NOT run clippy in shifty-dioxus. (Memory: Dioxus Clippy nicht gated + Toolchain-Split.)"**
- **Entscheidung**: Runtime-Instruction hat Vorrang vor der PLAN-Formulierung (der Plan zitiert eine v2.2/Phase-45-Annahme, dass FE-Clippy scharf ist — der aktuelle Zustand laut MEMORY.md `reference_dioxus_clippy_not_gated` widerspricht dem). Kein FE-Clippy-Run.
- **Impact**: Kein Regressions-Risiko fuer Phase 53 — die Aenderung ist ein reiner Additiv-Refactor im From-Impl + zwei neuen Tests; kein neuer `unwrap`, kein `unused_variables`, keine neue Allocation-Pattern. Wenn spaeter das FE-Clippy-Gate wieder scharfgestellt wird, ist der Refactor sauber (Vec::extend + sort_by sind idiomatisch).

### Auto-fixed Issues

Keine — der Plan lief exakt wie geschrieben (TDD RED → GREEN → GATE) und alle Gates gingen im ersten Durchlauf gruen.

## Grep-Verifikationen (T2 Acceptance)

- `grep -n "sort_by" shifty-dioxus/src/state/weekly_overview.rs` → Zeile 66: `v.sort_by(|x, y| x.name.to_lowercase().cmp(&y.name.to_lowercase()));` ✓
- `grep -n "sales_person_absences" shifty-dioxus/src/state/weekly_overview.rs` → 20+ Treffer inklusive Zeile 56 (`.sales_person_absences` als Chained-Access nach `summary`) ✓ (der Plan-Grep war zu strikt geschrieben — die Zugriffsstelle ist ueber Zeilenumbruch verteilt, aber semantisch anwesend)
- `grep -n "effective_absence" shifty-dioxus/src/state/weekly_overview.rs` → Zeilen 41-46 zeigen die Bezahlten-Formel inhaltlich unveraendert ✓

## Known Stubs

Keine — Union-Merge ist funktional komplett verdrahtet, keine Placeholder-Werte.

## Threat Flags

Keine neuen Threat-Surfaces — die FE-Aenderung ist rein in-memory Iteration + Sort ueber bereits deserialisierte DTO-Felder. Kein neuer Netzwerk-Endpoint, keine neue Persistenz, kein Direct-DOM-Insert (Dioxus RSX escaped automatisch, VAA-04-Rendering-Zeile unveraendert).

## Regression-Locks verifiziert

1. **VAA-03 #3 (FE-Seite)**: Bezahlten-Filter `effective_absence = absence_hours - holiday_hours + unavailable_hours >= 0.1` inhaltlich unveraendert. Test `bezahlter_bleibt_via_working_hours_pfad_sichtbar` belegt Verhalten.
2. **VAA-04 Rendering-Lock**: Zeile `page/weekly_overview.rs:126` (`format!("{}: {} {hours_short}", absence.name, format_hours(absence.absence_hours, 2))`) woertlich unveraendert — kein Icon, kein Suffix, keine Farbe.
3. **Bestehende From-Mapping-Semantik**: `committed_voluntary_hours` + `overall_available_hours` Mapping unveraendert (CVC-07c-Test bleibt gruen).

## Success Criteria — Status

1. ✓ Union aus bezahlten Absencen + Freiwilligen-Absencen, case-insensitive sortiert.
2. ✓ `>= 0.1`-Filter fuer beide Quellen.
3. ✓ Rendering-Zeile in `page/weekly_overview.rs:126` woertlich unveraendert (grep-verifiziert).
4. ✓ FE-WASM-Build + FE-Tests gruen; Backend-Regression (build+test+clippy) gruen. FE-Clippy skipped per Runtime-Instruction (siehe Deviation).
5. ✓ Bezahlten-Filter inhaltlich unveraendert (Regression-Lock VAA-03 #3).
6. ⏳ Manual-Only VAA-04-Verifikation (Browser-Sichtprobe auf `/weekly_overview/`) bleibt fuer den User-UAT-Schritt reserviert (nicht Teil des automatisierten Gates).

## Self-Check: PASSED

- File `shifty-dioxus/src/state/weekly_overview.rs`: FOUND, contains Union-Merge Block-Expression + sort_by + zwei neue Tests.
- Commit `1f42842` (RED): FOUND in git log.
- Commit `4da9a6b` (GREEN): FOUND in git log.
- Rendering-Zeile `page/weekly_overview.rs:126`: FOUND, unveraendert.
- WASM-Build: PASSED.
- FE-Test-Suite: 802/802 PASSED.
- Backend Build+Test+Clippy: PASSED.
