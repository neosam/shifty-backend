---
created: 2026-07-03T10:32:28.095Z
title: Absence-Warnbanner beim Booken entfernen (redundant zur Konfliktliste)
area: frontend
files:
  - shifty-dioxus/src/page/shiftplan.rs (BookingWarnings-Store + AddUserToSlot handler)
  - shifty-dioxus/src/component/warning_list.rs (WarningsList-Rendering)
---

## Problem

Beim Eintragen einer Person in einen Slot, während die Person im gleichen
Zeitraum eine Absence-Periode (Vacation / SickLeave / UnpaidLeave) hat,
zeigt das Frontend aktuell ZWEI Hinweise gleichzeitig:

1. Ein wegklickbarer NOTICE-Banner oben im Schichtplan
   („NOTICE · 1 CONFLICT — {Name} is absent on {date} as Vacation").
   Kommt aus dem Backend als `Warning` im
   `BookingCreateResult.warnings`-Array und wird via `booking_warnings`-
   Store durch `WarningsList` als dismissible Banner gerendert.
2. Die persistente „Fehlerhafte Zuweisungen"-Sektion
   (`booking_information/conflicts/for-week`), die seit v2.2.1 auch
   Absence-Periode-Überlappungen als Konflikt listet (nicht mehr nur
   manuell markierte Unavailable-Weekdays).

Der wegklickbare Banner ist damit redundant. Er verschwindet sowieso beim
nächsten Wochenwechsel, während die „Fehlerhafte Zuweisungen"-Liste den
Konflikt persistent zeigt bis der Slot gelöst wird.

## Solution

Backend-Seite: die Absence-bezogene Warning-Variante NICHT mehr im
`BookingCreateResult.warnings` emittieren, wenn sie sowieso von
`get_booking_conflicts_for_week` gefangen wird. Kandidat ist der Emit-Pfad
in `ShiftplanEditService::book_slot_with_conflict_check` — grep nach
„absent" / „Absence" / dem konkreten Warning-Variant-Namen dort.

Alternative (falls anderes Backend die Warning stimuliert): Frontend
filtert die Absence-Variante beim Setzen von `booking_warnings` aus
(schwächer, weil der Backend-Vertrag verlässlicher wäre).

Vorher zu klären: gibt es Kontexte, in denen die Absence-Warning WICHTIG
und die Konfliktliste NICHT sichtbar ist? Falls ja (z.B. eingeschränktes
Rollen-Setup ohne Zugriff auf Konfliktliste), Frontend-Filter statt
Backend-Removal. Sonst Backend-Removal ist sauberer.

Tests:
- Regression: Booking auf einem manuellen Unavailable-Weekday zeigt weiterhin
  keine doppelte Warnung (der Fall war schon vorher OK, aber neu absichern).
- Grep-Guard, dass die Absence-Warning-Variante nicht mehr emittiert wird
  aus `book_slot_with_conflict_check`.

Aufwand: klein (backend 1 Filter-Zeile + Regression-Test).
