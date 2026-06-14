---
created: 2026-06-14T08:45:00.000Z
title: Person per Doppelklick in "Per person"-Sektion als Filter auswählen
area: frontend
files:
  - shifty-dioxus/src/page/absences.rs:547-600
  - shifty-dioxus/src/page/absences.rs:1894-1965
  - shifty-dioxus/src/page/absences.rs:1281-1374
---

## Problem / Wunsch

In der globalen Abwesenheits-Übersicht (`/absences/`) gibt es die Sektion
„Per person · sorted by days remaining" (Komponente `VacationPerPersonList`,
`absences.rs:547`), die alle Personen mit verbleibenden Urlaubstagen auflistet.

Gewünscht: Per **Doppelklick** auf eine Person in dieser Sektion soll diese
Person als **Filter** für die darunterliegende Abwesenheitsliste gesetzt werden
— d. h. dasselbe Ergebnis wie die Auswahl im PERSON-Dropdown der Filter-Bar.

## Kontext / Bestandsaufnahme

- Per-Person-Sektion: `VacationPerPersonList` (`absences.rs:547-600`), rendert
  die nach `remaining_days` sortierten Zeilen (`VacationBalance` mit
  `sales_person_id`).
- Filter-State: Signal `person_filter: Option<Uuid>` in `AbsencesPage`
  (`absences.rs:1894`), fließt in die Listen-Filterung (`:1940`) und in die
  Filter-Bar / das PERSON-Dropdown (`AbsenceFilterBar`, props um `:1281-1374`).
- Die Liste reagiert bereits reaktiv auf `person_filter` — es muss also nur
  ein neuer Setz-Pfad ergänzt werden.

## Lösung (Skizze)

1. `VacationPerPersonList` um einen Callback-Prop erweitern, z. B.
   `on_person_select: EventHandler<Uuid>`, und auf jede Personen-Zeile einen
   `ondoubleclick`-Handler legen, der `sales_person_id` an den Callback gibt.
2. In `AbsencesPage` (`absences.rs:1894-1965`) den Callback verdrahten:
   `person_filter.set(Some(uuid))` setzen (reaktiv → Liste filtert sofort).
3. UX-Details klären:
   - Doppelklick auf eine **bereits ausgewählte** Person → Filter wieder
     aufheben (Toggle) oder ignorieren?
   - Visuelles Feedback in der Per-Person-Sektion, welche Person aktiv
     gefiltert ist (Highlight der Zeile), konsistent zum Dropdown-Zustand.
   - `cursor: pointer` / Hover-Affordance, damit erkennbar ist, dass die
     Zeilen klickbar sind.
4. Test: Render-/Interaktions-Test analog zu bestehenden Tests in
   `absences.rs`, der sicherstellt, dass nach Doppelklick `person_filter`
   gesetzt ist bzw. die gefilterte Liste nur Einträge der gewählten Person
   enthält.

**Hinweis:** Reines UX/Frontend-Feature, kein Backend-Touch nötig.
