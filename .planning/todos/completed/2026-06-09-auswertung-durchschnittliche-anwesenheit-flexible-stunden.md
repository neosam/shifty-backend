---
created: 2026-06-09 12:39
title: Auswertung durchschnittliche Anwesenheit bei flexiblen Stunden (ohne Urlaub)
area: reporting
resolves_phase: 41
files:
  - service_impl/src/reporting.rs (bestehende Reporting-Logik)
  - service/src/ (ggf. neue Reporting-Methode - TBD)
---

## Problem

Für Mitarbeiter mit **flexiblen Stunden** soll es eine Auswertung geben, die
zeigt, **wie viel ein Mitarbeiter im Schnitt da ist** — also die
durchschnittliche tatsächliche Anwesenheit / geleisteten Stunden.

Wichtige Einschränkung: **Urlaubszeiten sollen herausgerechnet werden.** Wenn
die Person gerade im Urlaub ist, soll das den Durchschnitt nicht nach unten
ziehen — es geht um die Anwesenheit in den Zeiträumen, in denen die Person
arbeiten würde / nicht abwesend ist.

Offene Fragen, die vor dem Planen geklärt werden müssen:
- **Bezugsgröße des Durchschnitts:** pro Woche? pro Monat? Über welchen
  Gesamtzeitraum wird gemittelt (Abrechnungsperiode / frei wählbar)?
- **Definition "Anwesenheit":** gebuchte/geleistete Stunden aus Bookings?
  Oder Anwesenheitstage? Stunden pro Anwesenheitstag vs. Stunden pro
  Kalenderwoche?
- **Was zählt als "Urlaub raus":** Nur Urlaub (vacation) oder auch
  Krankheit / unbezahlter Urlaub / Feiertage? → Abgleich mit den
  Absence-Kategorien (v1.0+ Absence Periods) und ggf. extra_hours.
- **Nur für flexible Mitarbeiter?** Wie wird "flexible Stunden" identifiziert
  (Vertragsmodell / working hours contract)? Gilt die Auswertung nur für
  diese Gruppe oder optional für alle?
- **Darstellung:** REST-Endpoint + Frontend-Ansicht? Teil des bestehenden
  Reportings oder eigene Sicht?

## Solution

TBD — Reporting-Erweiterung. Wahrscheinlich neue Berechnung im
`ReportingService` (Business-Logic-Tier), die Anwesenheits-/geleistete Stunden
über einen Zeitraum aggregiert und Abwesenheitszeiträume (mind. Urlaub) aus dem
Nenner herausrechnet, dann den Schnitt bildet.

Eignet sich als eigene Phase oder als Erweiterung einer Reporting-Phase —
mehrere Definitionsfragen offen (Bezugsgröße, welche Abwesenheiten zählen),
daher bei Aufgriff über `/gsd-discuss-phase` einsteigen.
