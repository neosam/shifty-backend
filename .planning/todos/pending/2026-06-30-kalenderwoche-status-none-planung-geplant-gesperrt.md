---
created: 2026-06-30T18:25:27.061Z
title: Status für Kalenderwochen (None / In Planung / Geplant / Gesperrt)
area: shiftplan
files:
  - service/ (ShiftplanService / SlotService)
  - rest/
  - rest-types/
  - dao/ + dao_impl_sqlite/ + migrations/sqlite/
  - shifty-dioxus/ (Wochenansicht Schichtplan)
---

## Problem

Aktuell gibt es keinen expliziten Status für eine Kalenderwoche im Schichtplan.
Es soll möglich sein, einer KW (year + week) einen von vier Status zuzuweisen:

- **None** — kein Status / Default
- **In Planung** — Woche wird gerade geplant
- **Geplant** — Planung abgeschlossen
- **Gesperrt** — Woche ist gesperrt

**Berechtigungsregel:** Gesperrte (`Gesperrt`) Wochen dürfen **nur noch vom
Schichtplaner** geändert werden. Für alle anderen Rollen sind Buchungen/
Slot-Änderungen in einer gesperrten Woche blockiert (Permission-Gate).
Wer den Status selbst setzen/ändern darf (insb. das Setzen/Aufheben von
`Gesperrt`), ist beim Plan-Phase noch zu klären — vermutlich ebenfalls der
Schichtplaner.

## Solution

TBD — grobe Richtung:

1. **Datenmodell:** Neue Tabelle/Spalte für Wochen-Status, key = (year, week).
   Migration in `migrations/sqlite/`. Enum `WeekStatus { None, InPlanning,
   Planned, Locked }`.
2. **DAO + Service:** DAO-Trait + SQLite-Impl; Service-Methode zum Lesen/Setzen
   des Status. Klassifizierung Basic vs. Business-Logic Service prüfen
   (Status-Manager könnte Basic sein; das Permission-Gate für gesperrte Wochen
   greift dann in Booking-/Slot-Schreibpfaden = Business-Logic). Vgl.
   Service-Tier-Konvention.
3. **Permission-Gate:** In den Schreibpfaden für Bookings/Slots prüfen, ob die
   betroffene KW `Gesperrt` ist — wenn ja, nur Schichtplaner-Rolle zulassen.
   Auf `single-week`-Edit-Pfad achten (siehe verwandtes Todo
   2026-06-26 Slot-Einzel-KW).
4. **REST:** Endpoints zum Lesen/Setzen des Status, mit `#[utoipa::path]` +
   `ToSchema`-DTO in `rest-types`.
5. **Frontend (shifty-dioxus):** Status pro KW in der Wochenansicht anzeigen
   (Badge/Auswahl) + Setzen. Gesperrte Wochen für Nicht-Schichtplaner als
   read-only kennzeichnen. i18n in allen drei Locales (En/De/Cs).
6. **Tests:** Service-Tests für Status-Übergänge + Permission-Gate
   (gesperrte Woche nur durch Schichtplaner änderbar). Backend-Roundtrip e2e
   (create- vs. edit-Pfad).
