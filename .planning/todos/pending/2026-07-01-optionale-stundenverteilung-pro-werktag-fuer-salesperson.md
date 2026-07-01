---
created: 2026-07-01T04:50:00.000Z
title: Optionale Stundenverteilung pro Werktag für SalesPerson
area: working-hours
files: []
---

## Problem

Aktuell hat eine SalesPerson nur die Gesamt-Sollstunden (Working-Hours-Vertrag).
Es fehlt die Möglichkeit, **optional** einzutragen, wie viele Stunden die Person an
einem bestimmten Werktag arbeitet (z. B. Mo 8h, Di 4h, Mi 0h, …).

Anforderungen:
- Pro Werktag ein optionaler Stundenwert.
- Die Summe der Werktags-Stunden muss den gesamten Sollstunden entsprechen (Validierung).
- **Feiertage und Urlaube sollen sich an dieser Tagesverteilung orientieren**: Fällt ein
  Feiertag/Urlaubstag auf einen bestimmten Werktag, wird die für diesen Werktag
  hinterlegte Stundenzahl angerechnet (statt einer pauschalen Aufteilung).
- Das Ganze ist **optional** — ohne Eintrag bleibt das bisherige Verhalten
  (pauschale Verteilung der Sollstunden).

## Solution

TBD — grobe Richtung:

- Datenmodell: pro SalesPerson (bzw. pro Working-Hours-Vertrag) eine optionale
  Werktags-Stunden-Tabelle (Wochentag → Stunden). Zeitliche Gültigkeit analog zum
  bestehenden Working-Hours-Vertrag beachten (Versionierung ab KW/Datum).
- Validierung: Summe der Werktags-Stunden == Gesamt-Sollstunden, sonst Fehler.
- Berechnung: Feiertags-/Urlaubs-Anrechnung (Absence + Special Days) auf die
  Werktags-Stunden umstellen, wenn eine Verteilung hinterlegt ist; sonst Fallback
  auf die bisherige Logik.
- Reporting/Balance-Hours entsprechend anpassen und auf Doppelzählung prüfen.
- REST + rest-types erweitern, Frontend-UI zum Eintragen der Verteilung.

Verwandt: [[2026-05-05-warnung-eintrag-ausserhalb-vertragszeiten]] (nutzt ebenfalls
Vertragszeiten je Wochentag).
