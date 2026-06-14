---
created: 2026-06-14T08:40:00.000Z
title: Stundenbasierte Marker in der Abwesenheitsliste in Tage umrechnen
area: frontend
files:
  - shifty-dioxus/src/page/absences.rs:1672-1703
  - shifty-dioxus/src/state/absence_period.rs:121-140
  - rest/src/absence.rs:42-56
  - rest-types/src/lib.rs
---

## Problem

In der globalen Abwesenheits-Übersicht (`/absences/`) werden die alten
stundenbasierten `extra_hours`-Marker (Legacy, Kategorie
Vacation/SickLeave/UnpaidLeave) als **rohe Stunden** angezeigt
(`absences.rs:1676`/`1703`: `"{:.2} hrs"`, z. B. „8.00 hrs", Badge
„hours-based").

Die **Range-Abwesenheiten** dagegen zeigen bereits genommene **Tage**
(`absences.rs:1758-1760`: `derived_days` + day/days-Label). Diese
Tages-Ableitung wurde in einem früheren Todo umgesetzt
(siehe `.planning/todos/completed/2026-06-12-urlaubstage-bei-abwesenheitszeitraeumen-aus-stunden-berechnen.md`).

Gewünscht: Auch die stundenbasierten Marker sollen (zusätzlich oder statt
der rohen Stunden) in **genommene Tage** umgerechnet angezeigt werden,
analog zu den Range-Einträgen.

## Kontext / Bestandsaufnahme

- Marker entstehen im Backend in `rest/src/absence.rs` über `map_to_marker`
  (`:42-56`); nur Kategorien {Vacation, SickLeave, UnpaidLeave} werden zum
  `ExtraHoursMarkerTO` (`is_absence_category`, `:57-63`). Das TO trägt aktuell
  `amount` (Stunden) + `when` (Datum), aber **kein** Tages-Feld.
- Frontend-State: `ExtraHoursMarker` in `state/absence_period.rs:121-140`
  (`amount: f32`, `when: time::Date`).
- Für Ranges liefert das Backend `derived_days` via
  `AbsenceService::derive_hours_for_range` (Single Source of Truth für die
  Stunden→Tage-Umrechnung pro Person, inkl. Wochen-Deckelung/Halbtag).

## Offene Frage (vor Umsetzung klären)

Die Umrechnung Stunden→Tage braucht die **Tagesstunden des Mitarbeiters am
`when`-Datum** (aktiver Arbeitsvertrag), genau wie `derive_hours_for_range`.
Zu entscheiden:
1. Umrechnung **backend-seitig** in `map_to_marker` (neues Feld
   `derived_days` am `ExtraHoursMarkerTO`, konsistent mit der Range-Logik /
   Single Source of Truth) — bevorzugt, vermeidet Logik-Duplizierung im
   Frontend.
2. ODER clientseitig — dann müsste das Frontend die Vertrags-Tagesstunden
   laden; Gefahr von Drift gegenüber der Backend-Ableitung.
3. Darstellung: nur Tage, oder „X Tage (Y hrs)"? Mit Range-Einträgen
   konsistent halten.

## Lösung (Skizze, bevorzugt Variante 1)

1. `ExtraHoursMarkerTO` (rest-types) um `derived_days: f32` erweitern.
2. In `rest/src/absence.rs` (`map_to_marker` bzw. an den beiden
   Marker-Lade-Stellen `:240-257` und `:412-...`) die Tage über dieselbe
   Quelle wie die Range-`derived_days` ableiten (Tagesstunden der Person am
   `when`-Datum aus dem aktiven Arbeitsvertrag).
3. Frontend `ExtraHoursMarker` (`state/absence_period.rs`) + Render in
   `absences.rs:1672-1703` auf Tage-Anzeige umstellen (Format konsistent zu
   `absences.rs:1758-1760`, `format_decimal` + day/days-Unit-Key).
4. i18n: ggf. neuen/angepassten Label-Key für Tage bei Markern (En/De/Cs).
5. Tests: pure Umrechnungsfunktion (Backend) + Render-/Format-Test im
   Frontend analog zu den bestehenden Marker-Tests in `absences.rs`.

**Hinweis:** Erst klären (Offene Frage), dann umsetzen — nicht raten.
