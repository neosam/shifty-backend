---
created: 2026-09-15T00:00:00.000Z
title: Sollstunden fehlen bei dynamischen Verträgen in der Übersicht unterhalb des Schichtplans
area: backend / reporting (+ frontend Anzeige)
resolves_phase: 64
files:
  - service_impl/src/reporting.rs:2184-2188 (weight_for_week — Ursache)
  - rest-types (ShortEmployeeReportTO — expected_hours / dynamic_hours)
  - shifty-dioxus/src/component/working_hours_mini_overview.rs:113,216 (Anzeige-Target)
  - shifty-dioxus/src/loader.rs:414 (build_working_hours_mini)
---

## Problem

In der Übersicht unterhalb des Schichtplans (`WorkingHoursMiniOverview`, Tabellen-
und Karten-Layout) steht bei Personen mit **dynamischem Vertrag** kein Soll — die
Zeile zeigt `Ist / 0.0h`. Für die Planung braucht man dort aber das Sollstunden-
Ziel, auch wenn es bei diesen Verträgen bewusst **nicht** aufs Stundenkonto wirkt.

User-Meldung (2026-09-15): *„in der Tabelle (oder Karten) unterhalb des
Schichtplans sind bei dynamischen Verträgen die Sollstunden nicht eingetragen.
Nur die freiwillig zugesagten Stunden. Es sollte aber für die Planung schon die
Sollstunden drinstehen bzw. von diesem Backend-Service zurückgegeben werden, auch
wenn die Sollstunden bei diesen Verträgen nicht relevant für das Stundenkonto
ist."*

## Ursache (im Code verifiziert)

`weight_for_week` (`service_impl/src/reporting.rs:2184-2188`) gibt ein Tupel
zurück, dessen **erstes** Feld zu `expected_hours` und dessen **zweites** zu
`dynamic_hours` wird:

```rust
(
    if employee_work_details.is_dynamic { 0.0 } else {
        employee_work_details.expected_hours * relation
    },                                               // -> expected_hours
    employee_work_details.expected_hours * relation, // -> dynamic_hours
    ...
)
```

Bei `is_dynamic == true` wird `expected_hours` also **hart auf 0.0 gesetzt** —
absichtlich, damit das Soll die Balance nicht beeinflusst. Das Frontend nimmt
`expected_hours` als Anzeige-Target (`working_hours_mini_overview.rs:113` und
`:216`), folglich steht dort 0.

## Warum der naheliegende Fix falsch wäre

`dynamic_hours` trägt den echten Vertragswert und liegt bereits im DTO — es als
Target zu nehmen sieht nach einer Ein-Zeilen-Lösung im Frontend aus, ist aber
eine: der Kommentar an der Anzeige-Stelle begründet die heutige Wahl explizit:

> TARGET = expected_hours (das feiertags-/absence-reduzierte Soll), NICHT
> dynamic_hours: der Band-Guard (HSP-03) hält dynamic_hours frei vom automatisch
> angerechneten Feiertag, sodass ein derived Holiday sonst nie im angezeigten
> Soll erschiene.

Ein Umschalten auf `dynamic_hours` würde also bei **allen** Verträgen dazu
führen, dass abgeleitete Feiertage nicht mehr im angezeigten Soll landen — eine
Regression für den heute korrekten Normalfall.

Ebenso wenig taugt es, `expected_hours` bei dynamischen Verträgen einfach zu
füllen: das Feld wird an den bilanzrelevanten Stellen konsumiert, genau deshalb
steht dort 0.

## Lösungsrichtung

Weder `expected_hours` (0) noch `dynamic_hours` (nicht feiertagsreduziert) ist
der gesuchte Wert. Gebraucht wird ein **drittes Soll für die Planung**: das
feiertags-/absence-reduzierte Vertrags-Soll, unabhängig davon, ob es
bilanzwirksam ist.

Nach „Fat Backend, Thin Client" gehört das ins Backend und ins DTO — der User
formuliert es genauso („bzw. von diesem Backend-Service zurückgegeben werden").
Also ein zusätzliches Feld auf `ShortEmployeeReportTO` (Arbeitstitel
`planning_target_hours`), das

- bei nicht-dynamischen Verträgen dem heutigen `expected_hours` entspricht, und
- bei dynamischen Verträgen das gleich berechnete Soll trägt, statt 0,

während `expected_hours` als bilanzrelevantes Feld **unverändert** bleibt. Das
Frontend zeigt dann das neue Feld als Target, und die Balance-Pfade bleiben
unangetastet.

## Zu klären

- **Ist die betroffene Komponente wirklich `WorkingHoursMiniOverview`?** Der
  Befund „Soll = 0 bei dynamischen Verträgen" passt exakt; der Zusatz „nur die
  freiwillig zugesagten Stunden" ließ sich dort aber nicht verifizieren —
  `committed_voluntary` wird in dieser Komponente nicht gerendert (es erscheint
  im Vertragsmodal und im `weekly_overview_chart`). Vor dem Fix am Bildschirm
  bestätigen, welche Zahl heute danebensteht.
- **Balance-Anzeige:** die Zeile zeigt neben `Ist / Soll` auch die Balance. Bei
  dynamischen Verträgen bleibt die Balance-Semantik unverändert — die neue
  Soll-Zahl darf dort nicht einfließen.
- **Weitere Konsumenten** des neuen Feldes prüfen (Fortschrittsbalken,
  Farbschwellen `hours_text_class` / `progress_bar_class`, Summenzeile
  `total_target`) — die sollen vermutlich alle auf das Planungs-Soll umgestellt
  werden, das ist aber eine bewusste Entscheidung pro Stelle.

## Aufwand

Klein bis mittel: eine Berechnung in `reporting.rs` (bzw. deren
Durchreichung), ein DTO-Feld, FE-Anzeige plus Tests. Keine Migration, kein
Snapshot-Bump (`expected_hours` und alle persistierten `value_type`s bleiben
unverändert).
