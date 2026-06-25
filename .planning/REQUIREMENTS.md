# Requirements: Shifty — v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen

**Defined:** 2026-06-25
**Core Value:** Urlaubs-/Abwesenheitswerte sind überall konsistent, das Umwandeln
stundenbasierter Legacy-Einträge braucht für HR nur noch minimale Handarbeit, und
die Mitarbeiter-Jahresansicht ist schnell les- und zuordenbar.

## v1 Requirements

Requirements für Milestone v1.5. Jede mappt auf eine Roadmap-Phase.

### Umwandeln-Dialog (Convert-UX)

- [ ] **UV-01**: Beim Öffnen des „In Zeitraum umwandeln"-Dialogs (`AbsenceConvertModal`)
  wird das „bis"-Datum automatisch so vorbelegt, dass der Zeitraum den bereits
  berechneten Urlaubstagen (`derived_days`) entspricht — **arbeitstagbasiert**
  (Wochenenden und Feiertage werden übersprungen). „von" bleibt der ursprüngliche
  Eintragstag.
- [ ] **UV-02**: Entsprechen die eingetragenen Urlaubsstunden **exakt** dem
  Wochen-Soll des Vertrags der Person, wird der Eintrag als „1 Woche" dargestellt
  (statt „N Tage") und der Umwandeln-Dialog schlägt **Montag–Sonntag** der
  betroffenen Kalenderwoche als Zeitraum vor. Bei jedem anderen Wert gilt die
  Tage-Darstellung und das Verhalten aus UV-01 (keine Vielfachen, kein Teilwochen-
  Sonderfall).

### Absences-Anzeige

- [ ] **UV-03**: Noch nicht konvertierte, stundenbasierte Einträge
  (`HourlyMarkerRow`) zeigen auf der Absences-Seite einen Warn-Indikator (⚠️) am
  Zeilenanfang, der signalisiert, dass der Eintrag noch **kein echter Urlaub**
  (Absence Period) ist und konvertiert werden sollte.

### Report- & Balance-Korrektheit

- [ ] **UV-04**: Der Carryover-Resturlaub in der Vacation-Balance-/Absence-Ansicht
  stimmt mit dem Wert des Report-Service überein (gleiche Carryover-Quelle:
  Ende-von-`year-1`-Snapshot, wie `ReportingService.get_employee`). Die Abweichung
  in `VacationBalanceService` (nutzt `get_carryover(year)`) wird behoben und durch
  Tests abgesichert. Report-Service = Wahrheit.
- [ ] **UV-05**: Nach Konvertierung eines stundenbasierten Urlaubseintrags
  (`extra_hours`, Kategorie `Vacation`) in eine Absence Period zeigt der
  Detail-Employee-Report (`/employees/:id`) die **Urlaubstage** (`vacation_days`)
  weiterhin korrekt — nicht 0. **Root-Cause:** `hours_per_week`
  (`service_impl/src/reporting.rs:1227`) summiert die absence-derived Stunden nur in
  `absence_hours` (Expected-Reduktion), aber **nicht** in die per-Woche-Felder
  `vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours`, aus denen
  `vacation_days()`/`sick_leave_days()`/`absence_days()` (`service/src/reporting.rs:118`)
  dividieren. Die Tage-Felder müssen die derived Absences mitzählen (analog zur
  Display-Stunden-Summe Z. 719–724), **ohne Doppelzählung**; testabgesichert.
  Betrifft `vacation_days`, `sick_leave_days`, `absence_days`.

### Mitarbeiter-Jahresansicht-Lesbarkeit (`/employees/:id`)

> Betrifft `EmployeeWeeklyHistogram` (`component/employee_weekly_histogram.rs`) — den
> Wochen-Graph auf der Mitarbeiter-Detailseite — sowie die aufklappbare KW-Liste
> (`expand_weeks`) / `WeekDetailPanel` in `component/employee_view.rs`.
> **NICHT** `weekly_overview_chart.rs` (das ist die separate Team-Route
> `/weekly_overview/`, dort sind Datum + volunteer bereits vorhanden). Datenquelle:
> `state::employee::WorkingHours` (Felder `from`/`to`, `overall_hours`,
> `expected_hours`, `volunteer_hours` je Woche — alle vorhanden).

- [ ] **YV-01**: Im Wochen-Graphen (`EmployeeWeeklyHistogram`) zeigt der Hover-/
  Tooltip pro Balken die KW und das **von–bis Datum** der Kalenderwoche (`from`–`to`),
  damit eine Woche im Graphen eindeutig zuordenbar ist.
- [ ] **YV-02**: Wo in der Mitarbeiter-Jahresansicht aktuell nur die **KW-Nummer**
  steht (Histogramm-X-Achse / aufgeklappte KW-Liste in `employee_view.rs`), wird
  zusätzlich das **von–bis Datum** gezeigt — Format „KW XY" + Zeilenumbruch + „von–bis".
  Ziel: schnelleres Suchen/Zuordnen von Wochen.
- [ ] **YV-03**: Der Wochen-Graph (`EmployeeWeeklyHistogram`) stellt die
  **Freiwilligen-Stunden** (`volunteer_hours`) als eigenes **gestapeltes** Balken-Segment
  dar (statt nur `overall_hours` als Einzelbalken), und die aufgeklappte KW-Liste /
  `WeekDetailPanel` weist die Freiwilligen-Stunden als **separaten Wert** aus.

### Mitarbeiter-Statistik (HR-only)

> Setzt das geparkte Todo `AVG-01`
> (`todos/pending/2026-06-09-auswertung-durchschnittliche-anwesenheit-flexible-stunden.md`)
> um. Mehrere Definitionsfragen offen → **eigene `discuss-phase` vor dem Planen.**

- [ ] **STAT-01**: Eine pro-SalesPerson Statistik-Ansicht ist verfügbar, **nur**
  zugänglich/sichtbar mit HR-Rolle (Gating analog `is_hr` auf der Absences-Seite).
- [ ] **STAT-02**: Kern-Kennzahl der Statistik ist die **durchschnittlich
  gearbeitete Stunden-Zahl pro Woche** einer Person. Urlaubs-/Abwesenheitszeiträume
  werden so herausgerechnet, dass sie den Schnitt nicht nach unten ziehen (es zählt
  die Anwesenheit in den Zeiträumen, in denen die Person arbeiten würde).
  **In `discuss-phase` zu klären:** Bezugszeitraum (Jahr / Abrechnungsperiode /
  frei wählbar), Definition „gearbeitet" (Bookings/Shiftplan vs. ExtraWork),
  welche Abwesenheitskategorien aus dem Nenner fallen (nur Vacation, oder auch
  SickLeave/UnpaidLeave/Holiday), und ob die Auswertung nur für flexible Verträge
  oder für alle gilt.

### UI-Polish — Tabellen-Lesbarkeit

- [ ] **UI-01**: Die Stunden-Tabelle unterhalb des Schichtplans
  (`WorkingHoursMiniOverview` TableLayout, `component/working_hours_mini_overview.rs`,
  gerendert in `page/shiftplan.rs:1140`) bekommt eine **maximale Breite** und ein
  **Zebra-Layout** (abwechselnde Zeilen-Hintergründe), damit man auf großen
  Bildschirmen nicht in der Zeile verrutscht.
- [ ] **UI-02**: In der `/absences`-Tabelle (`AbsenceList`, Grid
  `grid-cols-[1.5fr_170px_140px_90px_70px]` an 3 Stellen: `absences.rs` Z. 1632/1725/1817)
  wird die **Mitarbeiter-Spalte deutlich schmaler** (aktuell `1.5fr` → enger);
  optional eine max-width der Tabelle, damit Zeilen besser lesbar sind.

## v2 Requirements

Aktuell keine für diesen Milestone vorgemerkt.

## Out of Scope

Explizit ausgeschlossen, um Scope-Creep zu verhindern.

| Feature | Reason |
|---------|--------|
| Vielfache Wochen („2 Wochen", „N Wochen") im Convert/Display | User-Entscheidung — ging früher nicht richtig; nur exakter 1-Wochen-Fall (UV-02) |
| Teilwochen-Darstellung („½ Woche") | User-Entscheidung — nur exakte Wochen-Soll-Übereinstimmung, sonst Tage |
| Bug „Vertrag landet beim falschen Mitarbeiter" | Bereits gefixt (Signal-Mirror `current_employee_id` + Regressionstest `FROZEN_CAPTURE` in `employee_details.rs`) — Debug-Session resolved |
| Krankheitstage auf Absences-Seite | Bewusst ausgeblendet (Konstante `SICK_LEAVE_ENABLED`, Quick-Task 260612-svs) — nicht Teil dieses Milestones |

## Traceability

Wird bei der Roadmap-Erstellung befüllt.

| Requirement | Phase | Status |
|-------------|-------|--------|
| UV-01 | Phase 19 | Pending |
| UV-02 | Phase 19 | Pending |
| UV-03 | Phase 20 | Pending |
| UV-04 | Phase 18 | Pending |
| UV-05 | Phase 18 | Pending |
| YV-01 | Phase 20 | Pending |
| YV-02 | Phase 20 | Pending |
| YV-03 | Phase 20 | Pending |
| STAT-01 | Phase 22 | Pending |
| STAT-02 | Phase 22 | Pending |
| UI-01 | Phase 21 | Pending |
| UI-02 | Phase 21 | Pending |

**Coverage:**
- v1 requirements: 12 total
- Mapped to phases: 12 ✓
- Unmapped: 0 ✓

---
*Requirements defined: 2026-06-25*
*Last updated: 2026-06-25 after initial definition*
