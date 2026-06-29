---
created: 2026-06-29T14:45:00.000Z
title: Eigener Urlaub markiert nicht als Nicht-Verfügbar im Schichtplan
area: shiftplan / absence
files:
  - shifty-dioxus/src/page/shiftplan.rs:1120-1123
  - shifty-dioxus/src/component/week_view.rs:975-1065
  - shifty-dioxus/src/component/day_aggregate_view.rs
  - service_impl/src/booking_information.rs:70-99
---

## Problem

Trägt eine Person für sich Urlaub (Absence, Kategorie Vacation) ein, wird sie im
Schichtplan für diesen Zeitraum NICHT als „Nicht Verfügbar" markiert. Laut User war
das eigentlich Teil des Scopes.

**Ursache (verifiziert):** Im Schichtplan-Grid kommt die „Nicht Verfügbar"-Markierung
ausschließlich aus dem `discourage`-Flag. Dessen Quelle ist `discourage_weekdays`,
das in `shifty-dioxus/src/page/shiftplan.rs:1120-1123` ausschließlich aus
`unavailable_days` (= `sales_person_unavailable`, wiederkehrende Unverfügbarkeit pro
Wochentag) abgeleitet wird:

```rust
discourage_weekdays: unavailable_days
    .iter()
    .map(|unavailable_day| unavailable_day.day_of_week)
    ...
```

Absence/Urlaub-Zeiträume (Datumsbereiche) fließen hier NICHT ein. Das sind zwei
getrennte Konzepte:
- `sales_person_unavailable` → treibt den `discourage`-Marker im Grid
  (`week_view.rs:975-1065`, rote Zelle).
- `absence` (Vacation/SickLeave/Unpaid, Datumsbereich) → fließt nur in
  Reporting/Wochensummary-Stunden (`service_impl/src/booking_information.rs:70-99`,
  VFA-01).

Teil-Support existiert bereits: Bucht man jemanden auf einen Urlaubstag, gibt es eine
`WarningTO::BookingOnAbsenceDay`-Warnung (`shiftplan.rs:1463-1467`). Es fehlt aber die
PROAKTIVE Unverfügbar-Markierung der Urlaubstage im Grid.

## Solution

Absence-Zeiträume (mindestens Vacation, ggf. auch SickLeave/Unpaid) in die
Verfügbarkeitsanzeige des Schichtplans einbeziehen, sodass die betreffenden
Tage/Zellen als „Nicht Verfügbar" (discourage) erscheinen. TBD:

1. **Frontend-Seite:** Im Schichtplan zusätzlich die Absences der editierenden Person
   für die angezeigte Woche laden und in `discourage` einrechnen — analog zum
   bestehenden `reload_unavailable_days`-Pfad, aber datum- statt wochentagbasiert
   (eine Absence trifft konkrete Daten, nicht „jeden Montag"). `discourage_weekdays`
   müsste dafür ggf. um konkrete Datums-Treffer erweitert werden, da das aktuelle
   Modell pro Wochentag arbeitet.
2. **Klären:** Soll nur die eigene Person betroffen sein (Editier-Sicht) oder generell
   jede Person mit Urlaub in der Woche? Gilt es für alle Absence-Kategorien oder nur
   Vacation?
3. Prüfen, ob die Datenquelle schon vorhanden ist — Absences werden bereits backendseitig
   pro Jahr/Woche aggregiert (`booking_information.rs`); ggf. genügt ein Frontend-Join.

Scope-Check: Mit dem User abgleichen, ob das ursprünglich geplant war (er ging davon
aus) — dann eher Bug/Lücke als Neufeature.
