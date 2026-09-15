---
phase: 54-data-model-voluntary-stats
plan: 09
status: complete
gap_closure: true
requirements:
  - VOL-STAT-01
  - VOL-STAT-02
  - VOL-ACCT-01
  - VOL-ACCT-02
tags: [gap-closure, verification, uat-rerun, manual-verify, human-checkpoint, ist-fix]
---

# Plan 54-09 SUMMARY — Manual UAT Round 2 + Ist-Fix

## Ergebnis

**Phase 54 Gap-Closure Sequenz abgeschlossen.** Alle 6 automatisierten Gates
grün, Manual-Roundtrip User-approved („Jetzt sieht es richtig aus"), Gaps
G1 + G2 resolved.

## Task 1 — Automated Regression Gates (grün)

| Gate | Ergebnis |
|------|----------|
| `cargo build --workspace` | green |
| `cargo test --workspace` | green, 0 failed |
| `cargo clippy --workspace -- -D warnings` | green, 0 warnings |
| `cargo build --target wasm32-unknown-unknown` (FE) | green |
| `cargo test -p shifty-dioxus` | 806/806 passed |
| `cargo clippy -p shifty-dioxus -- -D warnings` | green |

### Grep-Sanity (alle 0)

- Alte fn-Namen im Backend + Docs: 0
- `"Freiwillig"` in `shifty-dioxus/src/i18n/de.rs`: 0
- `"Volunteer"`/`"Volunteer Work"` in `shifty-dioxus/src/i18n/en.rs`: 0
- `?year=` in `get_voluntary_stats` (`shifty-dioxus/src/api.rs`): 0
- `shifty-dioxus/src/i18n/cs.rs` unangetastet (Round 2)

## Task 2 — Manual-Roundtrip Checkpoint

**Ergebnis:** approved.

Der User hat den Live-Roundtrip im Browser durchgeführt und nach dem Ist-Fix
bestätigt: „Jetzt sieht es richtig aus."

Verifikations-Punkte:
- **Ehrenamt Soll:** Range-basiert korrekt (nicht 177h).
- **Ehrenamt Ist:** entspricht jetzt dem OVERALL-„Ehrenamt"-Wert
  (deckt manuelle VolunteerWork-ExtraHours + Shiftplan-Cap-Überlauf +
  no_contract-Shiftplan-Stunden ab).
- **Ehrenamt Delta:** Ist − Soll, matcht den tatsächlichen Kontostand.
- **Terminologie DE:** durchgängig „Ehrenamt".
- **Terminologie EN:** durchgängig „Voluntary".
- **Non-HR:** die 3 Ehrenamt-Zeilen bleiben verborgen.

## Task 2b — Ist-Fix (nicht im Original-Plan)

**Deviation vom Plan.** Der Manual-Roundtrip in Task 2 zeigte einen zweiten
Bug, der im Original-Plan 54-09 nicht antizipiert war: das Ehrenamt-Delta
zeigte „Stundenkonto = Soll" (Ist = 0), weil die pure fn
`voluntary_ist_total_in_range` nur DB-Rows mit `category = VolunteerWork` +
`source = Manual` zählte — Shiftplan-Cap-Überlauf und no_contract-Volunteer
fehlten.

**Fix per User-Anweisung („verrechne doch einfach was auf der Seite steht;
diese ganzen neuen Datenstrukturen sollen noch gar nicht verwendet werden
und nur zusätzlich einfließen"):**

- `VoluntaryStatsServiceImpl` konsumiert jetzt `ReportingService` statt
  `ExtraHoursService`.
- `ist_total = report.volunteer_hours` aus
  `ReportingService::get_report_for_employee_range(from, to, carryover=false)`.
- Die pure fn `voluntary_ist_total_in_range` ist entfernt.
- `committed_voluntary_target_in_range` + `contract_weeks_count_in_range`
  bleiben (Soll + Contract-Weeks).
- DI-Konstruktion in `shifty_bin/src/main.rs`: `voluntary_stats_service`
  wird jetzt NACH `reporting_service` konstruiert (Business-Logic-Wave).
- Service-Tests mit `MockReportingService` neu geschrieben.
- Docs (F14 EN+DE, F07 EN+DE) aktualisiert: source-Filter für
  Rebooking-Neutralität ist explizit auf Phase 55 verschoben; VOL-ACCT-03
  Property-Test ebenfalls auf Phase 55 deferred (kein Live-Rebooking-
  Writer in Phase 54).

**Commit:** `77c9ec45` — `fix(54-09): voluntary-stats Ist reads EmployeeReport.volunteer_hours (Gap G1 Ist-Fix)`

## Task 3 — Doc-Updates

- `.planning/phases/54-data-model-voluntary-stats/54-VERIFICATION.md`:
  neue Sektion „Gap-Closure Round 2" mit Gates-Ergebnissen + Gap-Status
  (G1 + G2 = resolved).
- `.planning/phases/54-data-model-voluntary-stats/54-UAT.md`: neuer
  Abschnitt „Round 2 — Post-Gap-Closure" mit Test 1 (pass), Test 2
  (pass), Test 3 (skipped, User-Deferral).

## Commits (Round 2 Gap-Closure)

- `7aefad3` — feat(54-07): voluntary-stats — accept date range instead of ISO year (Gap G1)
- `ddfd3dc` — docs(54-07): SUMMARY — voluntary-stats Range-Semantik Gap-Closure G1
- `a4f72e5` — feat(54-08): voluntary-stats FE — call backend with date range (Gap G1)
- `00de0d8` — refactor(54-08): unify i18n de/en to Ehrenamt / voluntary (Gap G2)
- `ecfbfa6` — docs(54-08): complete gap-closure G1 FE + G2 i18n plan
- `85df384` — docs(54-09): remove old fn-names from F14 + F07 for regression-guard
- `77c9ec45` — fix(54-09): voluntary-stats Ist reads EmployeeReport.volunteer_hours (Gap G1 Ist-Fix)

## Deviations from Plan

1. **Ist-Fix (Task 2b) war nicht im Original-Plan.** Der 5h-Mai-Regression-
   Test in Plan 54-07 hatte den Bug nicht erwischt, weil er `voluntary_ist_total_in_range`
   isoliert getestet hat (nur ExtraHours-Fixture), nicht den Roundtrip
   durch das OVERALL-Aggregat. Manual-Roundtrip war der einzige Weg, den
   Bug zu finden.

2. **Property-Test VOL-ACCT-03 gelöscht.** Der Property-Test aus Plan
   54-03/07 verifizierte Rebooking-Neutralität über die entfernte pure
   fn `voluntary_ist_total_in_range`. Da der `source == 'manual'`-Filter
   in Phase 54 nicht mehr in dieser Kette wirkt (er landet ab Phase 55
   zentral im ReportingService), wandert der Property-Test in Phase 55
   mit dem ersten Live-Rebooking-Writer.

## Follow-ups

- **Phase 55:** Rebooking-Writer + `source == 'manual'`-Filter zentral
  im `ReportingService` einbauen; VOL-ACCT-03 Property-Test neu
  aufsetzen (jetzt via `EmployeeReport::volunteer_hours`).
- **Deferred (aus Plan 54-08):** `54-08-cs-rename` bleibt offen —
  User-Deferral aus Round 1 Test 3.

## Learnings (für Extract-Learnings)

- **UAT-Report-Werte vs isolierte Unit-Tests:** ein „5h-Mai-Regression-
  Test" gegen die pure fn hat den 177-Bug erwischt, aber nicht den
  Follow-up-Bug „Ist deckt nur ExtraHours ab". Isolierte pure-fn-Tests
  garantieren nicht Roundtrip-Korrektheit. Manual-UAT bleibt für
  Range/Aggregation-Änderungen im Report-Pfad unverzichtbar.
- **Doppelte Aggregations-Quellen:** wenn ein UI-Feld (OVERALL „Ehrenamt")
  einen Wert anzeigt, der aus mehreren Quellen kombiniert wird
  (`manual_volunteer_hours + auto_volunteer_hours + no_contract_volunteer`),
  darf eine daneben liegende „Ist/Soll/Delta"-Zeile nicht eine andere
  Aggregation nutzen — sonst ergibt Delta ≠ Ist − OVERALL.
- **Marker-Strukturen vs Live-Filter:** die `source`-Spalte in Phase 54
  ist ein reines Datenmodell-Vorbereitungs-Feature. Ihre Aktivierung als
  Reader-Filter gehört ins Phase-55-Reporting, nicht in einen
  Voluntary-Stats-Reader in Phase 54.
