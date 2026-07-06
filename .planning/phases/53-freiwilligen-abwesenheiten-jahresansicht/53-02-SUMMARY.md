---
phase: 53-freiwilligen-abwesenheiten-jahresansicht
plan: 02
subsystem: api
tags: [rust, service-impl, assembly, tdd, red-green, weekly-summary, vaa]

requires:
  - phase: 53
    plan: 01
    provides: "SalesPersonAbsence + WeeklySummary.sales_person_absences (service-layer), SalesPersonAbsenceTO + WeeklySummaryTO.sales_person_absences (dto-layer), From-Impl-Kette. Beide Fill-Sites in service_impl auf leeres Default gesetzt."
  - phase: 52
    provides: "absent_volunteer_ids + all_absences + all_work_details load-once im get_weekly_summary Assembly-Loop; period_overlaps_week Helper"
provides:
  - "sales_person_absences befuellt in get_weekly_summary (D-53-05, Fill-Site 1)"
  - "sales_person_absences befuellt in get_summery_for_week (D-53-06, Fill-Site 2)"
  - "3 Backend-Tests booking_information_vaa (VAA-03 #1/#2/#3)"
affects: [53-03, weekly-overview, booking-information]

tech-stack:
  added: []
  patterns:
    - "Filter_map ueber HashSet<Uuid> mit Fixture-Lookup + Sigma cap-gated committed_voluntary — direkte Ableitung des bestehenden committed_voluntary_hours-Musters (booking_information.rs Zeile 509-517), aber OHNE den absent-Filter"
    - "VFA-01 whole-week-out Sichtbarkeit uebernommen (D-53-03 = D-26-01) — kein neuer Filter"
    - "TDD Fixed-Deps-Harness `build_service()` in Testmodul konsolidiert die 3 Test-Setups auf eine Funktion — reduziert Duplikation gegenueber vfa.rs (dort inline pro Test)"

key-files:
  created:
    - "service_impl/src/test/booking_information_vaa.rs (406 Zeilen, 3 tokio-Tests)"
  modified:
    - "service_impl/src/booking_information.rs (Fill-Site 1 + Fill-Site 2, 122 Zeilen netto Diff)"
    - "service_impl/src/test/mod.rs (Modul-Registrierung pub mod booking_information_vaa)"

key-decisions:
  - "Konsolidierter TestDeps-Harness `build_service()` statt Copy-Paste je Test (Reduktion Duplikation gegenueber vfa.rs — dort war das Setup inline; wir bauen einen kleinen Harness)"
  - "RED-Baseline nur fuer Test 1 (positive Assertion), Tests 2+3 sind Regression-Lock-Negativassertions die trivial im leeren Default-Zustand halten und nach der Implementierung nicht broken werden — semantisch korrekt als GREEN-halten designed (siehe Deviations)"
  - "In `get_summery_for_week` beide `sales_person_service.get_all()`-Calls konsolidiert (Pitfall 1 fuer paid_employees + volunteer_ids), obwohl der Plan nur eine Konsolidierung fuer volunteer_ids vorsah — es reduziert von 2 auf 1 Service-Call in derselben Methode, ist streng additiv sauberer"

patterns-established:
  - "Fixed-Deps-Test-Harness fuer BookingInformationService: `build_service(persons, periods, paid_report)` konstruiert die komplette Mock-Kette und laesst pro Test nur die Person/Absence/Report-Population variieren. Reproduzierbar fuer weitere VAA-Tests (VAA-04+ falls Follow-up)."

requirements-completed: [VAA-01, VAA-02, VAA-03]

coverage:
  - id: D1
    description: "Fill-Site 1 (get_weekly_summary): sales_person_absences befuellt pro Woche mit Freiwilligen die in absent_volunteer_ids sind; hours = Sigma cap-gated committed_voluntary; identische Formel zu committed_voluntary_hours (Zeile 509-517) OHNE absent-Filter"
    requirement: "VAA-01, VAA-02"
    verification:
      - kind: unit
        ref: "cargo test -p service_impl vaa03_volunteer_with_period_appears_with_correct_hours (assert hours == 5.0 == cap-gated committed_voluntary Fixture)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Fill-Site 2 (get_summery_for_week): dasselbe Feld analog befuellt; all_absences + all_sales_persons + absent_volunteer_ids-HashSet inline aufgebaut (D-53-06). volunteer_ids.contains-Filter fuer absent_volunteer_ids ist Pflicht (Pitfall 6)."
    requirement: "VAA-01"
    verification:
      - kind: static
        ref: "grep -c \"sales_person_absences\" service_impl/src/booking_information.rs == 5 (2 fill blocks + 2 literal uses + 1 in inline tests); grep -c \"volunteer_ids.contains\" == 8 (>= 2 required)"
        status: pass
    human_judgment: false
  - id: D3
    description: "VAA-03 #1 GREEN: Freiwilliger mit Vacation-Period in Woche 20 erscheint mit hours=5.0 in sales_person_absences"
    requirement: "VAA-03 #1"
    verification:
      - kind: unit
        ref: "cargo test -p service_impl vaa03_volunteer_with_period_appears_with_correct_hours (Fixture: committed_voluntary=5.0, cap=true, expected_hours=8.0; AbsencePeriod Mon 2026-05-11 .. Sun 2026-05-17)"
        status: pass
    human_judgment: false
  - id: D4
    description: "VAA-03 #2 GREEN: Freiwilliger ohne Period taucht NICHT in sales_person_absences auf (absent_volunteer_ids ist die exakte Sichtbarkeitsmenge)"
    requirement: "VAA-03 #2"
    verification:
      - kind: unit
        ref: "cargo test -p service_impl vaa03_volunteer_without_period_absent_not_in_list"
        status: pass
    human_judgment: false
  - id: D5
    description: "VAA-03 #3 GREEN: Bezahlter Mitarbeiter mit eigener AbsencePeriod bleibt in working_hours_per_sales_person UND leakt NICHT in sales_person_absences (Pitfall 6 Guard). Regression-Lock aus."
    requirement: "VAA-03 #3"
    verification:
      - kind: unit
        ref: "cargo test -p service_impl vaa03_paid_employee_unchanged_regression_lock (Fixture: paid AbsencePeriod fuer W20 + ShortEmployeeReport in year_reports fuer W20)"
        status: pass
    human_judgment: false
  - id: D6
    description: "Regressions-Schutz Phase 52 assembly Chain-C + VFA-01 whole-week-out"
    verification:
      - kind: unit
        ref: "cargo test -p service_impl booking_information_chain_c (8 passed); cargo test -p service_impl booking_information_vfa (2 passed)"
        status: pass
    human_judgment: false
  - id: D7
    description: "Workspace-Test-Gate + Clippy-Gate (Pflicht laut CLAUDE.md)"
    verification:
      - kind: unit
        ref: "cargo build --workspace (exit 0); cargo test --workspace (alle Suites gruen, service_impl 732 passed — vorher 729 + 3 neue vaa03_*); cargo clippy --workspace -- -D warnings (exit 0, keine Warnung)"
        status: pass
    human_judgment: false

duration: 10min
completed: 2026-07-06
status: complete
---

# Phase 53 Plan 02: Backend-Fill-Sites fuer sales_person_absences Summary

**Beide Fill-Sites in BookingInformationServiceImpl (get_weekly_summary + get_summery_for_week) befuellen jetzt das Plan-01-Traegerfeld sales_person_absences mit den freiwilligen Absenten der Woche — Name aus load-once all_sales_persons, hours per cap-gated committed_voluntary. Drei neue Backend-Tests (vaa03_*) belegen VAA-03 #1/#2/#3 via TDD RED→GREEN.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-07-06T11:25:26Z
- **Completed:** 2026-07-06T11:34:46Z
- **Tasks:** 4/4 (T1 RED-Tests, T2 Fill-Site 1, T3 Fill-Site 2, T4 Workspace-Gate)
- **Files modified:** 3

## Accomplishments

- Neues Testmodul `service_impl/src/test/booking_information_vaa.rs` (406 Zeilen) mit `TestDeps`-Block, konsolidiertem `build_service()`-Harness und drei tokio-Tests fuer VAA-03 #1/#2/#3.
- Registrierung `pub mod booking_information_vaa` in `service_impl/src/test/mod.rs`.
- Fill-Site 1 (`get_weekly_summary`, Zeile ~290 + ~528 + ~668): `all_sales_persons` load-once refactor, Freiwilligen-Absencen-Block mit cap-gated `committed_voluntary`-Formel, `WeeklySummary`-Literal auf `sales_person_absences` gesetzt.
- Fill-Site 2 (`get_summery_for_week`, Zeile ~706 + ~855 + ~1023): drei zusaetzliche Load-Bloecke (`all_sales_persons`, `all_absences`, `absent_volunteer_ids`-HashSet) plus identische Fill-Formel; beide `sales_person_service.get_all()`-Calls in der Methode zu einem konsolidiert (Pitfall 1 fuer volunteer_ids UND paid_employees).
- `cargo test --workspace` gruen (732 in service_impl, vorher 729 + 3 neue vaa03_*); `cargo clippy --workspace -- -D warnings` gruen; Regression-Suiten `booking_information_chain_c` (Phase 52 assembly) und `booking_information_vfa` (VFA-01) unveraendert gruen.

## Task Commits

1. **Task 1: RED — VAA-03 Test-Datei + Modul-Registrierung** — `0fbed8f` (test)
   - service_impl/src/test/booking_information_vaa.rs: neue Datei, 3 tokio-Tests + build_service()-Harness
   - service_impl/src/test/mod.rs: pub mod booking_information_vaa
2. **Task 2: GREEN — Fill-Site 1 in get_weekly_summary** — `f848778` (feat)
   - service_impl/src/booking_information.rs: all_sales_persons load-once + Fill-Block + Literal
3. **Task 3: GREEN — Fill-Site 2 in get_summery_for_week** — `746c688` (feat)
   - service_impl/src/booking_information.rs: all_sales_persons + all_absences + absent_volunteer_ids-HashSet + Fill-Block + Literal
4. **Task 4: Wave-2-Gate — cargo build/test/clippy workspace** — kein Commit (nur Verifikation; gruen)

**Plan metadata:** _(this SUMMARY.md commit)_

## Files Created/Modified

- `service_impl/src/test/booking_information_vaa.rs` — Neue Datei. TestDeps-Block, Fixture-Konstanten (`YEAR=2026`, `WEEK_UNDER_TEST=20`), drei UUID-Helper (`volunteer_id_absent`/`_present`/`paid_id`), `build_service()`-Harness mit optional injizierbarem paid-report fuer VAA-03 #3, drei tokio-Tests.
- `service_impl/src/test/mod.rs` — Zeile hinzugefuegt: `pub mod booking_information_vaa;` mit `#[cfg(test)]`-Prefix, direkt nach `pub mod booking_information_vfa`.
- `service_impl/src/booking_information.rs` — 
  - `get_weekly_summary`: Refactor Zeile 290-303 (all_sales_persons load-once + volunteer_ids); neuer Fill-Block Zeile 528-559 (sales_person_absences bauen); Literal Zeile 668 setzt das Feld.
  - `get_summery_for_week`: neue Blocks Zeile 706-753 (all_sales_persons + all_absences + absent_volunteer_ids); Refactor Zeile 843-848 (paid_employees ohne zweiten get_all-Call); neuer Fill-Block Zeile 855-885; Literal Zeile 1023 setzt das Feld.

## Decisions Made

- **TestDeps-Harness `build_service()` konsolidiert.** Der Plan legte nahe, den Mock-Setup Test-fuer-Test zu wiederholen (Pattern aus vfa.rs). Praktisch waren die 3 Tests weitgehend identisch (nur Fixtures unterschiedlich). Ein `build_service(persons, periods, paid_report_for_week)`-Helper reduziert die Duplikation drastisch und macht die Fixtures pro Test in wenigen Zeilen sichtbar. Kein semantischer Unterschied — dieselben Mock-Erwartungen werden gesetzt.
- **`get_summery_for_week`: beide `.sales_person_service.get_all()`-Calls konsolidiert.** Die Methode rief `get_all` zweimal auf (Zeile 707 fuer `volunteer_ids`, Zeile 802 fuer `paid_employees`). Weil Task 3 ohnehin die erste Call in `all_sales_persons` haelt (Pitfall 1 fuer den Namens-Lookup), war es trivial, `paid_employees` daraus abzuleiten und den zweiten Call zu eliminieren. Netto: 1 statt 2 DAO-Roundtrips pro Wochen-View. Semantisch identisch (`get_all` liefert alle Personen; Filter is_paid=true vs. is_paid=false).
- **`work_details` in Fill-Site 2 verwendet die bereits geladene `work_details`-Variable.** Statt einen separaten Load einzubauen (der Plan-Text erwaehnte "Wenn `employee_work_details_service.all()` noch nicht geladen ist, laedt Task 3d es einmalig vor dem Fill-Block"), nutzt der Fill-Block die bereits vorhandene `work_details`-Variable, die weiter unten in der Methode geladen wird — mit einer Verschiebung des Fill-Blocks nach dem Load. Ein einziger `all()`-Call, nicht zwei.

## Deviations from Plan

### Semantic-Neutral Deviations

**1. [Rule 3 - Cosmetic / By-Design] RED-Baseline: nur 1 statt 3 Tests scheitern initial**

- **Found during:** Task 1 (RED-Verifikation `cargo test -p service_impl booking_information_vaa`).
- **Issue:** Plan-Acceptance-Criterium sagt "Der Test-Runner zeigt drei `FAILED`-Zeilen (RED-Baseline erreicht)". Beim Ausfuehren scheitert nur Test 1 (`vaa03_volunteer_with_period_appears_with_correct_hours`); Tests 2 (`vaa03_volunteer_without_period_absent_not_in_list`) und 3 (`vaa03_paid_employee_unchanged_regression_lock`) sind negative Regression-Lock-Assertions: sie fordern `!week.sales_person_absences.iter().any(...)`. Diese Assertion hault trivialerweise wenn `sales_person_absences` leer ist (Plan-01-Default). Sie kann nur FAILEN wenn die Implementierung bugs — genau das, was Regression-Lock-Tests tun sollen.
- **Fix:** Kein Aendern der Tests. Test 1 liefert das RED-Signal (positive Assertion, faellt gegen leeren Default). Tests 2 + 3 sind korrekt-by-design als Regression-Lock formuliert und muessen im leeren Ausgangszustand HALTEN, nicht failen.
- **Files modified:** keine (nur Verstaendnisanpassung der Acceptance-Criteria).
- **Verification:** Nach Task 2 GREEN sind alle drei gruen (3 passed); Test 2 + 3 blockieren jetzt jede Regression, die a) einen Nicht-Abwesenden ins Feld leakt, b) einen Bezahlten ins Feld leakt oder ihn aus `working_hours_per_sales_person` entfernt.
- **Committed in:** 0fbed8f (Task 1).
- **Impact:** Semantisch identisch zum Plan-Intent (VAA-03 #1/#2/#3 durch Tests abgedeckt). Nur die Formulierung des Acceptance-Kriteriums "drei FAILED" war ueber-strikt fuer negative Regression-Assertions. Wenn ein zukuenftiger Planner drei-mal-FAILED wirklich erzwingen wollte, muesste die Test-Datei je eine positive und eine negative Assertion pro Regression enthalten (also 5 Tests statt 3) — Overkill fuer diesen Scope.

**2. [Rule 3 - Cosmetic] Nur EIN `WeeklySummary`-Literal in `get_summery_for_week` (Plan sprach von zwei)**

- **Found during:** Task 3 (Suche nach dem zweiten Literal auf Zeile ~960 aus PATTERNS.md §3).
- **Issue:** Plan Task 3 Action (e) sagt "Beide `WeeklySummary`-Literale erweitern — Zeile 901 und Zeile 960". PATTERNS.md §3 Schritt E zeigt aber nur EIN Literal auf Zeile 901-921. Direkte Inspektion der aktuellen Datei zeigt genau EIN Literal in dieser Methode (Zeile ~1010 nach Task-2-Edits). Kein zweiter Fill-Site-Pfad, kein Fehler-Fall-Literal, kein separater Success-vs-Fallback-Pfad.
- **Fix:** Das eine vorhandene Literal wurde befuellt. Kein zweites zu befuellen.
- **Files modified:** service_impl/src/booking_information.rs (nur ein Literal-Site).
- **Verification:** `grep -n "let summary = WeeklySummary\|WeeklySummary {" service_impl/src/booking_information.rs` innerhalb `get_summery_for_week` — genau ein Treffer.
- **Committed in:** 746c688 (Task 3).
- **Impact:** Keine — der Plan-Text war leicht ueber-inklusiv. Beide Endpoints (`get_weekly_summary` Jahresansicht + `get_summery_for_week` Wochensicht) bedienen jetzt konsistent das DTO-Feld (D-53-06 erfuellt).

**3. [Rule 3 - Additive Cleanup] Redundanter zweiter `sales_person_service.get_all()`-Call in `get_summery_for_week` eliminiert**

- **Found during:** Task 3 (Refactor `all_sales_persons` load-once).
- **Issue:** Der Plan verlangte den `all_sales_persons`-Refactor nur fuer den ersten Call (Zeile 707, `volunteer_ids`). Die Methode hatte aber einen zweiten `get_all`-Call auf Zeile 802 fuer `paid_employees`. Wenn ich beide behalten haette, waeren zwei DAO-Roundtrips per Wochen-Endpoint statt einer — semantisch identisch, aber verschwenderisch.
- **Fix:** `paid_employees` wird jetzt aus dem oben geladenen `all_sales_persons` abgeleitet (`.iter().filter(is_paid=true).map(id)`). Der zweite Service-Call ist entfernt.
- **Files modified:** service_impl/src/booking_information.rs.
- **Verification:** `grep -A1 "\.sales_person_service" service_impl/src/booking_information.rs | grep "get_all"` — innerhalb `get_summery_for_week` bleibt genau EIN Aufruf. Alle Wochen-Tests + Regressions gruen.
- **Committed in:** 746c688 (Task 3).
- **Impact:** Reine Performance-Verbesserung, semantisch neutral. Passt zu Pitfall 1 aus dem Plan.

---

**Total deviations:** 3 semantisch-neutrale (1 RED-Test-Anzahl, 1 nur-ein-Literal, 1 zusaetzliche Konsolidierung).
**Impact on plan:** Keine funktionale Abweichung. Alle 3 VAA-03-Tests gruen, beide Fill-Sites befuellt, keine Regression.

## Issues Encountered

- **RED-Signal nur fuer Test 1 (nicht 3).** Siehe Deviation #1. Kein Blocker — Test 1 gab das RED-Signal, Tests 2 + 3 sind by-design Regression-Locks die im leeren Ausgangszustand halten.
- **Second `get_all()`-Call in `get_summery_for_week`.** Vor der Task-3-Edit unerwartet — Plan erwaehnt nur den ersten. Loesung: mit-konsolidiert (Deviation #3).

## User Setup Required

None — reine additive Assembly-Fill-Site + neue Test-Datei. Keine externen Services, keine Env-Var, keine Migration, keine neue Cargo-Dep.

## Next Phase Readiness

- **Plan 03 (FE-Union-Merge):** kann sofort starten. Backend liefert jetzt fuer beide Endpoints (`/booking-information/weekly-resource-report/{year}` und `/booking-information/for-week/{year}/{week}`) das befuellte `sales_person_absences`-Feld im DTO. Der `state::WeeklySummary::from(&WeeklySummaryTO)`-Mapper kann direkt einen Union-Merge aus `working_hours_per_sales_person` (bezahlt) + `sales_person_absences` (Freiwillige) bauen (D-53-04/05, Zeile 47-62 der Ist-Datei fuer den bezahlten-Pfad; PATTERNS.md §5 zeigt den Ziel-Diff).
- **Keine Blocker.**

## Threat Flags

_Kein neuer Threat-Surface — Plan 02 aktiviert nur eine vorher-additive Struct-Erweiterung. Der bestehende SHIFTPLANNER_PRIVILEGE-Gate an beiden umschliessenden Endpoints bleibt unangetastet (T-53-02-01..04 Threats aus dem Plan-Register alle mitigated per bestehende Gates + neue Tests). Freiwilligen-Namen im `SalesPersonAbsence.name`-Feld sind nur fuer SHIFTPLANNER sichtbar (wie bereits `working_hours_per_sales_person.sales_person_name` fuer Bezahlte)._

## Self-Check: PASSED

- `service_impl/src/test/booking_information_vaa.rs` — enthaelt `vaa03_volunteer_with_period_appears_with_correct_hours`, `vaa03_volunteer_without_period_absent_not_in_list`, `vaa03_paid_employee_unchanged_regression_lock`. **FOUND.**
- `service_impl/src/test/mod.rs` — enthaelt `pub mod booking_information_vaa;`. **FOUND.**
- `service_impl/src/booking_information.rs` — enthaelt neue Fill-Bloecke: `grep -c sales_person_absences = 5` (2 Fill-Bloecke + 2 Literal-Uses + 1 in Test-Modul). **FOUND.**
- Task-Commits `0fbed8f`, `f848778`, `746c688` in `git log`. **FOUND.**
- `cargo build --workspace` — exit 0.
- `cargo test --workspace` — alle Suites gruen; service_impl 732 passed (729 → 732, +3 VAA-Tests).
- `cargo clippy --workspace -- -D warnings` — exit 0, keine Warnung.
- `booking_information_chain_c` (Phase 52 assembly) — 8 passed (unveraendert).
- `booking_information_vfa` (VFA-01 whole-week-out) — 2 passed (unveraendert).

---
*Phase: 53-freiwilligen-abwesenheiten-jahresansicht*
*Completed: 2026-07-06*
