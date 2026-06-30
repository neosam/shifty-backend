---
created: 2026-06-30T00:00:00
title: Feiertags-Automatik reduziert das Soll in der Schichtplan-Tabelle nicht (nur im Stundenkonto)
area: reporting / shiftplan
resolves_phase: 34
files:
  - service_impl/src/reporting.rs
  - service_impl/src/booking_information.rs
---

## Problem

Die Feiertags-Auto-Anrechnung aus Phase 25 (`special_day` Typ `Holiday` →
`build_derived_holiday_map`) reduziert das Soll **nur im Mitarbeiter-Report /
Stundenkonto**, **nicht** in der Wochen-Tabelle unterhalb des Schichtplans.

User-Beobachtung (2026-06-30): Stundenkonto korrekt (Feiertag senkt expected /
hebt Balance), aber in der Tabelle unter dem Schichtplan wird der Feiertag
nicht vom Soll abgezogen.

### Ursache (verifiziert am Code)

Der `reporting_service` hat mehrere Einstiegspunkte. Die Feiertags-Automatik
(`build_derived_holiday_map`) wird nur in dreien aufgerufen:

- `get_reports_for_all_employees` — `reporting.rs:360`
- `get_report_for_employee_range` — `reporting.rs:753`
- `hours_per_week` (Injektionspunkt 1a) — `reporting.rs:1320`

→ Diese speisen das **Stundenkonto** und enthalten den Abzug korrekt.

**`get_week` (`reporting.rs:884`) ruft `build_derived_holiday_map` NICHT auf.**
Dort gilt:
- `holiday_hours` = nur manuelle `ExtraHours(Holiday)` (`reporting.rs:955`)
- `expected_hours = planned_hours - abense_hours_for_balance -
  absence_derived_balance_total` (`reporting.rs:1072`) — **kein** derived holiday.

Die Schichtplan-Tabelle kommt aus `booking_information` und liest genau diese
`get_week`-Werte:
- `available_hours: report.expected_hours` (`booking_information.rs:327`)
- `holiday_hours: report.holiday_hours` (`booking_information.rs:331`)

→ daher kein Feiertagsabzug in der Tabelle.

### Warum es so ist (bewusste Phase-25-Grenze)

Decision **D-25-08 / HOL-03** ("Regressions-Guard Jahresansicht"): Phase 25 ließ
den `get_week`/`booking_information`-Pfad bewusst unangetastet, um
`paid_hours`/`dynamic_hours`/`committed_voluntary_hours`/`volunteer_hours` nicht
zu verfälschen. Der Test `test_holiday_auto_credit_no_year_view_impact`
schreibt fest, dass `get_week` das SpecialDay-Service nicht für Holiday-Credit
aufruft.

Als **Nebeneffekt** blieb dabei auch die `available_hours`-Spalte (Mitarbeiter-
Soll) ungereduziert. Asymmetrie: Der **Bedarf** (`required_hours = slot_hours`)
wird an Feiertagen bereits reduziert (Slots an Feiertagen werden rausgefiltert,
`booking_information.rs:300-307`), die **Soll-Kapazität** der Mitarbeiter aber
nicht.

## Solution

TBD — grobe Richtung:

- `get_week` einen vierten Injektionspunkt für den derived Holiday geben
  (analog 1a/1b/1c), aber **sauber gegated**: nur `expected_hours` /
  `holiday_hours` / das daraus abgeleitete `available_hours` reduzieren.
- **Nicht** anfassen: `dynamic_hours` (→ `paid_hours` in
  `booking_information.rs:318`), `committed_voluntary_hours`, `volunteer_hours`
  — das ist der von D-25-08 geschützte Kern. Genau hier liegt das Risiko: das
  Soll reduzieren, ohne die Kapazitätsbänder zu verfälschen.
- Cutoff-/Konflikt-Logik (Stichtag, manueller Holiday gewinnt) gilt identisch —
  am besten denselben `build_derived_holiday_map`-Helper wiederverwenden.
- HOL-03-Regressionstest `test_holiday_auto_credit_no_year_view_impact` muss
  bewusst angepasst werden (er verbietet aktuell genau diesen Aufruf). Vorher
  klären, was er künftig genau garantieren soll (Bänder unverändert, aber
  expected/available reduziert).
- Snapshot-Versionierung: vermutlich **kein** Bump nötig, da
  `billing_period`-Snapshots aus dem `holiday_hours`-Pfad (`reporting.rs:872`)
  und nicht aus `get_week`/`booking_information` gespeist werden — vor
  Umsetzung verifizieren.
- Vorher entscheiden: Ist die Soll-Reduktion in der Schichtplan-Tabelle
  fachlich überhaupt gewünscht, oder soll die Tabelle bewusst das
  Brutto-Vertragssoll zeigen? (User tendiert zu reduzieren — analog
  Stundenkonto.)

Zugehörige Phase: noch nicht angelegt — erst diskutieren, ob eigener Slice oder
Nachzügler zu Phase 25.
