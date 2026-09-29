---
phase: 64-planungs-soll-f-r-dynamische-vertr-ge
plan: 01
subsystem: api
tags: [rust, reporting, dto, serde, mockall, tdd]

requires:
  - phase: 02-reporting-integration-snapshot-versioning
    provides: "reporting_phase2_fixtures.rs — Spiegel-Paar aus dynamischem und festem 40h-Vertrag plus Urlaubs-/Krankheits-Perioden"
  - phase: 08.4-absence-merge
    provides: "die gegateten Bilanz-Reduktionsterme in assemble_weeks, deren Gate-Kopplung dieser Plan bewusst NICHT wiederverwendet"
  - phase: 55-hr-alert
    provides: "has_pending_rebooking/pending_rebooking_id — Praezedenz fuer additive Felder an ShortEmployeeReport/TO"
provides:
  - "service::reporting::ShortEmployeeReport::planning_target_hours: f32"
  - "rest_types::ShortEmployeeReportTO::planning_target_hours: f32 (#[serde(default)]) inkl. From-Impl"
  - "berechnetes Planungs-Soll an BEIDEN Konstruktionsstellen (assemble_weeks und get_reports_for_all_employees)"
  - "service_impl/src/test/reporting_planning_target.rs — fuenf Tests (PSOLL-01, PSOLL-03, PSOLL-04, Zeichengleichheit, Reduktionswirksamkeit)"
affects: [64-02 Frontend-Anzeige, 64-03 Doku, 64-04 UAT]

tech-stack:
  added: []
  patterns:
    - "Parallele, eigenstaendig gegatete Reduktionsrechnung neben einem bestehenden, anders gegateten Bilanzpfad"
    - "Cross-Site-Paritaetstest, der zwei unabhaengig geschriebene Aggregationspfade gegeneinander haelt"

key-files:
  created:
    - service_impl/src/test/reporting_planning_target.rs
  modified:
    - service/src/reporting.rs
    - rest-types/src/lib.rs
    - service_impl/src/reporting.rs
    - service_impl/src/test/mod.rs
    - service_impl/src/test/reporting_phase2_fixtures.rs
    - service_impl/src/test/booking_information_weekly_summary_year_boundary.rs
    - service_impl/src/test/booking_information_vaa.rs
    - service_impl/src/test/person_directory_dao.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/tests/volunteer_work_tests.rs

key-decisions:
  - "Gate = has_contract_row && dynamic_hours > 0.0 — beide Bedingungen, gemessen an Element 1 des weight_for_week-Tupels, an BEIDEN Konstruktionsstellen zeichengleich"
  - "Kein .max(0.0) auf dem Ergebnis — der bestehende Bilanzpfad klemmt ebenfalls nicht; Klemmen wuerde die Zeichengleichheit unter schwerer Abwesenheit brechen"
  - "In get_reports_for_all_employees bekommen die Zielgroessen eigene Variablennamen (week_extra_hours_for_target etc.) statt die Normal-Zweig-Variablen zu teilen — die kleine Doppelberechnung haelt die Bilanzpfade Zeile fuer Zeile unveraendert"
  - "Kein Snapshot-Schema-Bump: billing_period_report.rs liest EmployeeReport, nicht ShortEmployeeReport — Version bleibt 13"

patterns-established:
  - "Zwei-Gate-Trennung: ein Anzeigefeld darf ein eigenes Gate haben, wenn das Bilanz-Gate fachlich anders gemeint ist"
  - "Paritaetstest statt zweier isolierter Tests, wenn dasselbe fachliche Konzept an mehreren Aggregationsstellen berechnet wird"

requirements-completed: [PSOLL-01, PSOLL-03, PSOLL-04]

coverage:
  - id: D1
    description: "Ein dynamischer Vertrag hat in get_week ein Planungs-Soll von 40.0, waehrend expected_hours 0.0 und dynamic_hours 40.0 bleiben"
    requirement: "PSOLL-01"
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_planning_target.rs#test_planning_target_dynamic_get_week"
        status: pass
    human_judgment: false
  - id: D2
    description: "Derselbe dynamische Vertrag hat in get_reports_for_all_employees ein Planungs-Soll von 80.0 (Vertragswochen 22 und 23)"
    requirement: "PSOLL-01"
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_planning_target.rs#test_planning_target_dynamic_all_employees"
        status: pass
    human_judgment: false
  - id: D3
    description: "Beide Konstruktionsstellen liefern fuer dieselben Ausgangsdaten denselben Wert (56.0), und Abwesenheiten reduzieren ihn nachweislich (< 80.0, > 0.0)"
    requirement: "PSOLL-03"
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_planning_target.rs#test_planning_target_parity_both_sites"
        status: pass
    human_judgment: false
  - id: D4
    description: "Fuer nicht-dynamische Vertraege ist das Planungs-Soll zeichengleich mit expected_hours — auch fuer eine Vertragszeile mit 0 Vertragsstunden"
    requirement: "PSOLL-04"
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_planning_target.rs#test_planning_target_matches_expected_for_fixed_contract"
        status: pass
    human_judgment: false
  - id: D5
    description: "available_hours in WorkingHoursPerSalesPerson liest weiterhin das bilanzrelevante Soll und bleibt fuer dynamische Vertraege 0.0"
    requirement: "PSOLL-04"
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_planning_target.rs#test_available_hours_unchanged_dynamic"
        status: pass
      - kind: other
        ref: "grep -c 'available_hours: report.expected_hours' service_impl/src/booking_information.rs == 2"
        status: pass
    human_judgment: false

duration: 45min
completed: 2026-09-29
status: complete
---

# Phase 64 Plan 01: Planungs-Soll im Backend Summary

**`ShortEmployeeReport` und `ShortEmployeeReportTO` tragen ein neues Feld `planning_target_hours`, das an beiden Konstruktionsstellen mit einem eigenen, zweiteiligen Gate berechnet wird — dadurch hat ein dynamischer Vertrag erstmals ein feiertags- und abwesenheitsreduziertes Planungsziel > 0, waehrend jede Bilanzgroesse byte-identisch bleibt.**

## Performance

- **Duration:** ~45 min
- **Tasks:** 3 von 3
- **Files modified:** 10 (1 neu, 9 geaendert)

## Accomplishments

- Additives Feld `planning_target_hours` an Modell, DTO und `From`-Impl; `#[serde(default)]` haelt die Wire-Kompatibilitaet additiv.
- Parallele, eigenstaendig gegatete Reduktionsrechnung an BEIDEN `ShortEmployeeReport`-Konstruktionsstellen, mit zeichengleich formuliertem Gate.
- In `get_reports_for_all_employees` wurden die Feiertags-/Abwesenheits-Bausteine vor die Dreiwege-Verzweigung gehoben — der dynamische Zweig kannte sie vorher gar nicht.
- Fuenf Tests, darunter ein Cross-Site-Paritaetstest, der die beiden unabhaengig geschriebenen Aggregationspfade gegeneinander haelt.

## Task Commits

1. **Task 1: Feld am Modell, am DTO und an allen bestehenden Literalen** — `4eb742f2` (feat)
2. **Task 2: RED — Testmatrix** — `37e9c146` (test)
3. **Task 3: GREEN — die Rechnung an beiden Konstruktionsstellen** — `4612f826` (feat)

## Das Gate — beide Bedingungen, an beiden Stellen, zitiert

**Stelle A — `service_impl/src/reporting.rs:1121` (`assemble_weeks`):**

```rust
let reduce_target = has_contract_row && dynamic_hours > 0.0;
```

`dynamic_hours` ist an dieser Stelle noch Element 1 des `weight_for_week`-Tupels,
also der ungezwungene Rohwert — die Ueberschattung durch
`dynamic_hours - abense_hours_for_balance - absence_derived_balance_total`
passiert erst weiter unten. Der Einfuegepunkt liegt direkt hinter
`let expected_hours = expected_hours_for_cap - holiday_derived_gated;`.

**Stelle B — `service_impl/src/reporting.rs:1476` (`get_reports_for_all_employees`):**

```rust
let planning_target_hours_for_week = if has_contract_row && dynamic_hours > 0.0
{
    dynamic_hours
        - manual_absence_for_target
        - derived_absence_for_target
        - derived_holiday_for_target
} else {
    0.0f32
};
```

Auch hier ist `dynamic_hours` Element 1; Element 0 heisst in dieser Funktion lokal
`expected_hours`. Gegen `expected_hours` zu pruefen waere die Falle gewesen: fuer jede
Woche eines dynamischen Vertrags ist dieser Wert 0, die Reduktion waere genau dort
ausgefallen, wo sie neu gebraucht wird.

**Warum die zweite Bedingung strukturell den 0-Stunden-Randfall schliesst:** Fuer
nicht-dynamische Vertraege sind Element 0 und Element 1 identisch. Das bestehende
Bilanz-Gate (`!has_contract_row || planned_hours <= 0.0`) und das neue Gate schalten
damit unter genau derselben Bedingung. Bei einer Vertragszeile mit 0 Vertragsstunden
oder einer Randwoche mit `relation == 0.0` ueberspringen also BEIDE Pfade die
Subtraktion; ohne die zweite Bedingung wuerde nur der neue Pfad von einer 0-Basis
abziehen und ein negatives Planungs-Soll erzeugen, waehrend `expected_hours` bei 0
bleibt. Der zweite Assertion-Block in
`test_planning_target_matches_expected_for_fixed_contract` ist genau dieser
Regressionstest.

## Gemessene Testwerte

Alle Werte wurden waehrend der Verifikation per temporaerer `eprintln!`-Sonde
abgelesen (Sonde danach vollstaendig zurueckgenommen, die Datei im Commit ist
sondenfrei).

| Test | gemessen |
|---|---|
| `test_planning_target_dynamic_get_week` | `planning_target_hours = 40`, `expected_hours = 0`, `dynamic_hours = 40` |
| `test_planning_target_dynamic_all_employees` | `planning_target_hours = 80`, `expected_hours = 0` |
| `test_planning_target_parity_both_sites` | `get_week(KW22) = 40`, `get_week(KW23) = 16`, Summe `= 56`; `all_employees = 56`; Differenz 0.0; `0 < 56 < 80` |
| `test_planning_target_matches_expected_for_fixed_contract`, Block 1 | `get_week`: Planungs-Soll 16 = `expected_hours` 16; `all_employees`: 56 = 56 |
| `test_planning_target_matches_expected_for_fixed_contract`, Block 2 (0h-Vertrag, nicht dynamisch) | `get_week`: 0 = 0; `all_employees`: 0 = 0 — kein negativer Wert |
| `test_available_hours_unchanged_dynamic` | `available_hours = 0` bei Planungs-Soll 40 im Report |

RED-Lauf vor Task 3 (zum Nachweis, dass die Tests fachlich und nicht durch einen
Kompilierfehler rot waren): `40 erwartet / 0 bekommen`, `80 / 0`, `> 0 / 0`,
`16 erwartet / 0 bekommen`. `test_available_hours_unchanged_dynamic` war bereits rot
nicht, sondern gruen — es ist ein Regressionsschloss, kein Treiber.

**Keine Erwartung musste nachgezogen werden.** Alle sechs Zahlen stimmen mit der
Herleitung aus dem Plan ueberein (40h je volle Vertragswoche; 8h je Werktag; drei
abwesende Werktage in KW 23 durch `fixture_vacation_period` Mo-Mi plus
`fixture_sick_period` Di nach BUrlG §9 => 24h Reduktion; 40 − 24 = 16; 40 + 16 = 56).

## Negative Planungs-Soll-Werte

**Keine beobachtet.** Kein Testlauf und kein Gate-Pfad hat in diesem Plan einen
negativen Wert erzeugt. Der pathologische Fall bleibt aber konstruierbar: bei einer
Woche mit positivem Rohwert und Abwesenheitsstunden groesser als der Rohwert wird das
Planungs-Soll negativ — genau wie das bilanzrelevante `expected_hours` unter derselben
Eingabe. Das ist gewollt und der Grund, warum bewusst KEIN `.max(0.0)` gesetzt wurde.
Wer es spaeter nachtraegt, bricht die Zeichengleichheit aus
ROADMAP-Erfolgskriterium 4.

## Files Created/Modified

- `service/src/reporting.rs` — Feld `planning_target_hours` an `ShortEmployeeReport` mit Zweck-Kommentar
- `rest-types/src/lib.rs` — Feld an `ShortEmployeeReportTO` (`#[serde(default)]`) plus Mapping im `From`-Impl
- `service_impl/src/reporting.rs` — Rechnung an beiden Konstruktionsstellen, `WeeklyHours` um ein Feld erweitert, Fold und Push ergaenzt
- `service_impl/src/test/reporting_planning_target.rs` — NEU, fuenf Tests plus Mock-Boilerplate fuer beide Reporting-Entry-Points und den `booking_information`-Pfad
- `service_impl/src/test/mod.rs` — Registrierung
- `service_impl/src/test/reporting_phase2_fixtures.rs` — Doc-Kommentar der dynamischen Fixture um die Phase-64-Erwartung ergaenzt
- `service_impl/src/test/booking_information_weekly_summary_year_boundary.rs`, `service_impl/src/test/booking_information_vaa.rs` — drei bestehende `ShortEmployeeReport`-Literale mitgezogen
- `shifty-dioxus/src/loader.rs`, `shifty-dioxus/src/tests/volunteer_work_tests.rs` — drei bestehende `ShortEmployeeReportTO`-Literale mitgezogen
- `service_impl/src/test/person_directory_dao.rs` — Rule-3-Fix, siehe Deviations

## Decisions Made

Keine neuen Entscheidungen — die eine nicht-triviale Entscheidung der Phase war im
Plan bereits festgeschrieben und wurde exakt so umgesetzt. Zwei Umsetzungsdetails,
die der Plan offen liess:

- Die Rohwerte fuer die Reduktion stammen an jeder Stelle aus deren eigenem,
  bestehenden Filter. An Stelle A ist `abense_hours` ueber
  `category.availability() == Availability::Unavailable` gebildet, an Stelle B ueber
  `category.as_report_type() == ReportType::AbsenceHours`. Diese Asymmetrie ist
  vorbestehend (sie betrifft heute schon die Bilanzpfade) und wurde bewusst NICHT im
  Rahmen dieses Plans vereinheitlicht — das waere eine Aenderung an den Bilanzpfaden
  gewesen. Der Paritaetstest laeuft ueber abgeleitete Abwesenheiten, nicht ueber
  manuelle `extra_hours`, und beruehrt die Asymmetrie deshalb nicht. **Fuer Plan 64-03
  oder eine Folgephase vormerken.**
- `test_planning_target_parity_both_sites` hat eine dritte Assertion
  (`all_employees > 0.0`) zusaetzlich zu den im Plan geforderten zwei. Ohne sie waere
  der Test im RED-Lauf gruen gewesen (0.0 == 0.0 erfuellt Paritaet, 0.0 < 80.0 erfuellt
  die Obergrenze) und damit kein Treiber.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Pre-existierende Clippy-Warnung blockierte das neue `--all-targets`-Gate**
- **Found during:** Task 2 (Verifikationsschritt `cargo clippy --workspace --all-targets -- -D warnings`)
- **Issue:** `service_impl/src/test/person_directory_dao.rs:149` (`fn row_of<'rows>(...)`, aus Phase 62) loeste `clippy::needless_lifetimes` aus. Die Warnung ist vorbestehend und in keinem CI-Gate sichtbar, weil `.github/workflows/rust.yml` und der `nix build` `cargo clippy --workspace -- -D warnings` OHNE `--all-targets` laufen — Lib-Test-Targets werden dort nicht geprueft. Task 2 und Task 3 fordern aber explizit `--all-targets`.
- **Fix:** Lifetimes elidiert: `fn row_of(rows: &[PersonDirectoryRow], person_id: Uuid) -> &PersonDirectoryRow`. Reine Signatur-Vereinfachung, kein Verhaltensunterschied.
- **Files modified:** `service_impl/src/test/person_directory_dao.rs`
- **Verification:** `cargo clippy --workspace --all-targets -- -D warnings` gruen; `cargo test --workspace` unveraendert gruen
- **Committed in:** `37e9c146` (Teil des Task-2-Commits)

---

**Total deviations:** 1 auto-fixed (1x Rule 3)
**Impact on plan:** Keine Scope-Ausweitung. Der Fix war noetig, um ein vom Plan gefordertes Gate ueberhaupt laufen lassen zu koennen.

## Issues Encountered

Keine. Der RED-Lauf war beim ersten Versuch fachlich rot (nicht durch Kompilierfehler),
der GREEN-Lauf beim ersten Versuch vollstaendig gruen, und keine Erwartung musste
korrigiert werden.

## Gate-Ergebnisse

| Gate | Ergebnis |
|---|---|
| `cargo build --workspace` | gruen |
| `cargo test --workspace` | gruen — 963 (service_impl lib, vorher 958) + 104 Integration + alle uebrigen, 0 failed |
| `cargo clippy --workspace -- -D warnings` | gruen |
| `cargo clippy --workspace --all-targets -- -D warnings` | gruen |
| `cargo test --manifest-path shifty-dioxus/Cargo.toml` | gruen — 858 passed (Baseline unveraendert) |
| `cargo clippy --manifest-path shifty-dioxus/Cargo.toml --all-targets -- -D warnings` | gruen |
| `cargo build --manifest-path shifty-dioxus/Cargo.toml --target wasm32-unknown-unknown` | gruen |
| Geruest-Marke `SCAFFOLD-64-01` verschwunden | ja |
| `planning_target_hours` in `booking_information.rs` / `billing_period_report.rs` | nicht vorhanden |
| `available_hours: report.expected_hours` in `booking_information.rs` | 2 (unveraendert) |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13` | 1 Treffer (kein Bump) |
| `weight_for_week`-Signatur (`num_potential_workdays_in_week as u8`) | 1 Treffer (unveraendert) |
| `git diff --name-only -- '*Cargo.toml'` | leer (keine neue Abhaengigkeit) |

## Threat Flags

Keine neue Angriffsflaeche. `T-64-01` bleibt `accept` (das Feld ist eine Ableitung aus
bereits ausgelieferten Werten im selben DTO, an dieselben Empfaenger). `T-64-02` und
`T-64-03` sind durch Test plus Quellcode-Gates mitigiert und oben belegt. `T-64-SC`
entfaellt mangels Paket-Installation.

## Known Stubs

Keine.

## User Setup Required

Keine — kein externer Dienst, keine Migration, keine neue Route.

## Next Phase Readiness

Plan 64-02 kann direkt aufsetzen: `ShortEmployeeReportTO::planning_target_hours` wird
von `GET /report` und `GET /report/week/{year}/{calendar_week}` ohne Routenaenderung
ausgeliefert, und `get_year` fuehrt das Feld gratis mit (derselbe
`assemble_weeks`-Helfer, D-64-02). Zu beachten fuer 64-02: das Feld fehlt noch in
`shifty-dioxus/src/state/employee_work_details.rs::WorkingHoursMini` und in
`build_working_hours_mini` — beides ist ausdruecklich Scope von 64-02.

Fuer Plan 64-03 (Doku) vormerken: die oben beschriebene Rohwert-Asymmetrie zwischen
`Availability::Unavailable` (Stelle A) und `ReportType::AbsenceHours` (Stelle B) ist
vorbestehend und betrifft auch die Bilanzpfade.

## Self-Check: PASSED

- `service_impl/src/test/reporting_planning_target.rs` — FOUND
- `.planning/phases/64-planungs-soll-f-r-dynamische-vertr-ge/64-01-SUMMARY.md` — FOUND
- Commit `4eb742f2` — FOUND
- Commit `37e9c146` — FOUND
- Commit `4612f826` — FOUND

---
*Phase: 64-planungs-soll-f-r-dynamische-vertr-ge*
*Completed: 2026-09-29*
