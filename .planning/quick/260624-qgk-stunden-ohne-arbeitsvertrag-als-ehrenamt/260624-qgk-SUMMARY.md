---
task: 260624-qgk
title: Ehrenamt ohne Arbeitsvertrag verbuchbar + Ehrenamt-Stunden unter "Soll" anzeigen (Schwelle >= 0.5)
type: quick
completed: 2026-06-24
snapshot_schema_version_bump: false
---

# Quick Task 260624-qgk Summary

## One-liner

`committed_voluntary_hours` durch den Short-Report-Pfad (Backend → TO → Frontend) durchgereicht; separate geschwellte (>= 0.5) Anzeige unter "Soll" in Karten- und Tabellenansicht; i18n in de/en/cs; Req-1-Befund mit Regressionstest gepinnt.

## What Changed

### Req 1 — Ehrenamt ohne Arbeitsvertrag verbuchbar (bestätigend, kein Verhaltens-Change)

**Befund bestätigt:** Eine `EmployeeWorkDetails`-Zeile mit `expected_hours = 0.0` und `cap_planned_hours_to_expected = false` (= kein bezahlter Vertrag) wird von `committed_voluntary_for_calendar_week` korrekt aggregiert — `committed_voluntary = 5.0` liefert `5.0`. Der D-05-Pfad war bereits korrekt; die Implementierung ist jetzt gegen Regression gepinnt.

Neuer Test: `committed_voluntary_bookable_without_paid_contract` in `service_impl/src/reporting.rs`.

### Req 2 — Datentransport `committed_voluntary_hours` Backend → Frontend

**Backend (`service/src/reporting.rs`):**
- `ShortEmployeeReport`: neues Feld `pub committed_voluntary_hours: f32` ergänzt.

**Backend (`service_impl/src/reporting.rs`):**
- `get_week`: berechnet `committed_voluntary_hours` per Person aus den person-gruppierten `working_hours` via `committed_voluntary_for_calendar_week(&working_hours, year, week)`.
- `get_reports_for_all_employees`: befüllt `committed_voluntary_hours` analog mit `committed_voluntary_for_calendar_week(&working_hours, year, until_week)`.

**Transport (`rest-types/src/lib.rs`):**
- `ShortEmployeeReportTO`: neues Feld `#[serde(default)] pub committed_voluntary_hours: f32` ergänzt.
- `From<&ShortEmployeeReport>`-Impl: mappt `committed_voluntary_hours` 1:1.
- Tests: Roundtrip (2.5 überlebt) + Backward-Compat (Legacy-JSON ohne Feld → 0.0).

**Frontend State (`shifty-dioxus/src/state/employee_work_details.rs`):**
- `WorkingHoursMini`: neues Feld `pub committed_voluntary_hours: f32` ergänzt.
- `Default`: `committed_voluntary_hours: 0.0`.

**Frontend Loader (`shifty-dioxus/src/loader.rs`):**
- `build_working_hours_mini`: mappt `committed_voluntary_hours: report.committed_voluntary_hours`.
- Test-Fixture `make_report`: um `committed_voluntary_hours: 0.0` ergänzt.

### Req 3 — Geschwellte Anzeige unter "Soll" (>= 0.5)

**Helper (`shifty-dioxus/src/component/working_hours_mini_overview.rs`):**
- `pub(crate) fn show_committed_voluntary(committed: f32) -> bool { committed >= 0.5 }` (Threshold-Gate).

**TableLayout:**
- Per-Zeile: unter `{target_str}h` wird bedingt (`if show_committed_voluntary(...)`) eine kleine Zeile `"+ {row_committed_str}h {row_committed_label}"` mit Klasse `text-small font-normal text-good` gerendert.
- Total-Zeile: analoge geschwellte Anzeige der Summe aller `committed_voluntary_hours`.

**CardsLayout:**
- `let i18n = I18N.read().clone()` hinzugefügt (CardsLayout hatte bisher keinen i18n-Zugriff).
- Per-Karte: unter der Stunden-Zeile wird bedingt eine `div` mit `text-micro font-normal text-good` gerendert.

**Die Soll-Zahl (`dynamic_hours` / `target_str`) bleibt vollständig unverändert** — ausschließlich additive Anzeige, kein Reinrechnen in die Soll-Zahl. Balance und Auslastung bleiben rechnerisch identisch.

### i18n

- `Key::CommittedVoluntaryShort` zur `Key`-Enum in `shifty-dioxus/src/i18n/mod.rs` hinzugefügt.
- de.rs: `Locale::De, Key::CommittedVoluntaryShort, "Ehrenamt"`
- en.rs: `Locale::En, Key::CommittedVoluntaryShort, "Volunteer"`
- cs.rs: `Locale::Cs, Key::CommittedVoluntaryShort, "Dobrovolnictví"`

## Test Counts

### Backend (`cargo test --workspace`)
- Vorher: 557 Tests
- Danach: 562 Tests
- Neu: 5 Tests (+1 Req-1-Bestätigungstest, +2 TO-Roundtrip-Tests, +2 weitere existing-committed-voluntary-Tests)
- Ergebnis: **0 failed**

### Frontend (`cargo test` in shifty-dioxus/)
- Vorher: 627 Tests
- Danach: 630 Tests
- Neu: 3 Tests (`show_committed_voluntary_threshold`, `committed_voluntary_line_rendered_when_at_or_above_threshold`, `committed_voluntary_line_hidden_below_threshold`)
- Ergebnis: **0 failed**

### WASM Build Gate
- `cargo build --target wasm32-unknown-unknown`: **exit 0**

## Snapshot Schema Version

`CURRENT_SNAPSHOT_SCHEMA_VERSION` wurde **nicht geändert**. Begründung: Req 2 und Req 3 berühren ausschließlich den Live-Short-Report-Pfad (`get_week` / `get_reports_for_all_employees`), der nicht persistiert wird. Kein `billing_period`-`value_type` wurde hinzugefügt, entfernt oder in seinem Input-Set verändert.

## Files Modified

| File | Change |
|------|--------|
| `service/src/reporting.rs` | `ShortEmployeeReport` + Feld `committed_voluntary_hours: f32` |
| `service_impl/src/reporting.rs` | `get_week` + `get_reports_for_all_employees` befüllen Feld; neuer Test |
| `rest-types/src/lib.rs` | `ShortEmployeeReportTO` + Feld; `From`-Impl; 2 neue Tests |
| `shifty-dioxus/src/state/employee_work_details.rs` | `WorkingHoursMini` + Feld + Default |
| `shifty-dioxus/src/loader.rs` | `build_working_hours_mini` + Mapping; Test-Fixture gepatcht |
| `shifty-dioxus/src/component/working_hours_mini_overview.rs` | `show_committed_voluntary` Helper; CardsLayout + TableLayout + Total-Row Anzeige; 3 neue Tests; alle bestehenden WorkingHoursMini-Struct-Literale um Feld ergänzt |
| `shifty-dioxus/src/i18n/mod.rs` | `Key::CommittedVoluntaryShort` |
| `shifty-dioxus/src/i18n/de.rs` | "Ehrenamt" |
| `shifty-dioxus/src/i18n/en.rs` | "Volunteer" |
| `shifty-dioxus/src/i18n/cs.rs` | "Dobrovolnictví" |
| `shifty-dioxus/src/tests/volunteer_work_tests.rs` | Test-Fixture um `committed_voluntary_hours: 0.0` ergänzt (Rule 3: rustc-geführt) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `ShortEmployeeReportTO`-Literal in `volunteer_work_tests.rs` fehlte**
- **Found during:** Task 2, beim Frontend-Test-Compile
- **Issue:** `src/tests/volunteer_work_tests.rs:147` konstruierte `ShortEmployeeReportTO` ohne `committed_voluntary_hours` — rustc-Fehler E0063
- **Fix:** `committed_voluntary_hours: 0.0` ergänzt
- **Files modified:** `shifty-dioxus/src/tests/volunteer_work_tests.rs`

**2. [Rule 3 - Missing dependency] CardsLayout hatte keinen i18n-Zugriff**
- **Found during:** Task 2, bei Implementierung der Ehrenamt-Anzeige in CardsLayout
- **Issue:** CardsLayout benötigte `i18n.t(Key::CommittedVoluntaryShort)` für das Label, hatte aber keinen `I18N`-Zugriff (anders als TableLayout)
- **Fix:** `let i18n = I18N.read().clone();` am Anfang von `CardsLayout` hinzugefügt
- **Files modified:** `shifty-dioxus/src/component/working_hours_mini_overview.rs`

**3. [Rule 1 - Bug Fix] Legacy-JSON-Test in rest-types verwendete rohen JSON-String mit `"version"` statt `"$version"`**
- **Found during:** Task 1, erster `cargo build`-Versuch
- **Issue:** `SalesPersonTO.version` ist als `#[serde(rename = "$version")]` definiert; der direkt eingebettete JSON-String in der Backward-Compat-Test würde nicht korrekt deserialisieren. Zudem: Raw-String-Literale mit `null` wurden vom Compiler falsch geparst.
- **Fix:** Test nutzt jetzt `serde_json::Value`-Manipulation (serialisieren → Feld entfernen → re-deserialisieren) statt hartkodiertem JSON-String
- **Files modified:** `rest-types/src/lib.rs`

## Known Stubs

None — alle neuen Felder sind voll verdrahtet (Backend-Report → TO → Frontend-State → Render).

## Threat Flags

None — keine neuen Netzwerk-Endpunkte oder Auth-Pfade eingeführt. Die Änderungen sind rein im Datentransport bestehender Report-Endpunkte und im Frontend-Rendering.

## Self-Check

### Files created/modified exist:
- `service/src/reporting.rs` — modified (ShortEmployeeReport + committed_voluntary_hours)
- `service_impl/src/reporting.rs` — modified (get_week + get_reports_for_all_employees + test)
- `rest-types/src/lib.rs` — modified (ShortEmployeeReportTO + From-Impl + tests)
- `shifty-dioxus/src/state/employee_work_details.rs` — modified (WorkingHoursMini + Default)
- `shifty-dioxus/src/loader.rs` — modified (build_working_hours_mini + test fixture)
- `shifty-dioxus/src/component/working_hours_mini_overview.rs` — modified (helper + display + tests)
- `shifty-dioxus/src/i18n/mod.rs` — modified (Key::CommittedVoluntaryShort)
- `shifty-dioxus/src/i18n/de.rs` — modified ("Ehrenamt")
- `shifty-dioxus/src/i18n/en.rs` — modified ("Volunteer")
- `shifty-dioxus/src/i18n/cs.rs` — modified ("Dobrovolnictví")
- `shifty-dioxus/src/tests/volunteer_work_tests.rs` — modified (test fixture)

### Verification results:
- `cargo build --workspace` (Backend): PASSED
- `cargo test --workspace` (Backend): PASSED (0 failed)
- `cargo test` (shifty-dioxus): PASSED (630 passed, 0 failed)
- `cargo build --target wasm32-unknown-unknown` (shifty-dioxus): PASSED (exit 0)

### VCS constraint:
No commits made by the executor. All changes are in the working copy (`@`) for the user to commit manually via `jj`.

## Self-Check: PASSED
