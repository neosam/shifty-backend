---
created: 2026-05-05T00:00:00
title: Warnung bei Eintrag außerhalb der Vertragszeiten bei bezahlter SalesPerson
area: shiftplan
files: []
---

## Problem

Wird ein Booking-Eintrag für eine bezahlte SalesPerson erstellt, der außerhalb
ihrer vertraglich vereinbarten Arbeitszeiten (Working Hours) liegt, soll das
System eine Warnung ausgeben — analog zum bereits in Phase 05-06
implementierten `PaidEmployeeLimitExceeded`-Warning, aber bezogen auf die
zeitliche Lage des Slots gegenüber dem Working-Hours-Vertrag, nicht auf die
Anzahl bezahlter Bookings pro Woche.

Heute prüft die Buchungs-/Shiftplan-Logik nicht, ob der Slot innerhalb der
Vertragszeiten liegt; ein außervertraglicher Einsatz wird stillschweigend
akzeptiert. Für die Lohn-/Stunden-Auswertung kann das zu Überraschungen
führen, da bezahlte Stunden außerhalb des Vertrags entstehen können, ohne
dass jemand vorher informiert wurde.

## Solution

TBD — grobe Richtung:

- Entlang derselben Warnungs-Pipeline wie `PaidEmployeeLimitExceeded`
  (siehe Phase 05-06: `count_paid_bookings_in_slot_week`-Helper +
  Service-Tier-Tests) eine zweite Warnung emittieren, z. B.
  `PaidBookingOutsideContractHours`.
- Datenquelle für die Vertragszeiten ist der Working-Hours-Vertrag
  der SalesPerson (Wochentag/Zeitfenster).
- Entscheiden, ob die Warnung pro Booking, pro Slot oder pro Woche
  aggregiert wird.
- Service-Tier-Tests + REST-Sichtbarkeit klären, bevor geplant wird.

Zugehörige Phase: noch nicht angelegt — erst diskutieren, ob das ein
eigenständiger Slice oder Teil eines größeren "Warnungs-Bundles" wird.
