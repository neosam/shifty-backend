# Phase 25: Feiertags-Auto-Anrechnung & Stichtag-Konfiguration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-28
**Phase:** 25-Feiertags-Auto-Anrechnung & Stichtag-Konfiguration
**Areas discussed:** Architektur, Konfliktregel (HCFG-03), Stichtag-Speicher (HCFG-01/02), Stunden-Formel (HOL-01)
**Mode:** Interactive (gsd-autonomous --interactive), Batch-Entscheidung mit Empfehlungen

---

## Architektur der Anrechnung

| Option | Description | Selected |
|--------|-------------|----------|
| Derive-on-read | Feiertagsstunden beim Report-Lesen aus special_day + Vertrag berechnet, in bestehende holiday_hours-Summe injiziert; keine DB-Zeilen; Stichtag/Konflikt = Read-Filter; Snapshot-Bump | ✓ |
| Materialize (echte ExtraHours-Rows) | Pro Feiertag×Mitarbeiter echte auto-markierte ExtraHours(Holiday)-Zeile; sichtbar/auditierbar; aber Generierungs-Zeitpunkt, Cleanup, Dedup, Backfill | |

**User's choice:** Derive-on-read (= Empfehlung)
**Notes:** Vermeidet Write-Side-/Cleanup-Komplexität; Stichtag- und Konfliktregel werden zu simplen Read-Filtern.

---

## Konfliktregel manuell vs. automatisch (HCFG-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Manuell gewinnt | Automatik überspringt Tage mit vorhandenem manuellem Holiday-Eintrag (pro Mitarbeiter+Tag); keine Doppelzählung | ✓ |
| Automatik ersetzt | Automatik hat Vorrang, manueller Eintrag ignoriert/überschrieben | |

**User's choice:** Manuell gewinnt (= Empfehlung)
**Notes:** Bewusste manuelle Eingaben bleiben unangetastet; verhindert Doppelzählung.

---

## Stichtag-Speicher (HCFG-01/02)

| Option | Description | Selected |
|--------|-------------|----------|
| Neuer Key-Value-Config-Store | Generische app_config(key, value)-Tabelle nach Toggle-Muster; wiederverwendbar (war meine Empfehlung) | |
| Toggle-Tabelle um value-Spalte erweitern | Bestehende toggle-Infra um nullable value-Spalte ergänzen (key-value-mit-Wert) | ✓ |
| Dedizierte Holiday-Config-Tabelle | Eigene Tabelle nur für die Feiertags-Automatik | |

**User's choice:** Toggle-Tabelle um value-Spalte erweitern (überstimmt meine Key-Value-Empfehlung)
**Notes:** Reuse der in Phase 24 etablierten Toggle-Infra; Stichtag als ISO-Datum-String im neuen value-Feld.

---

## Stunden-Formel (HOL-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Bestehenden Helper nutzen | EmployeeWorkDetails::holiday_hours() = expected_hours / potential_days_per_week(); Anrechnung nur wenn has_day_of_week(Feiertag) | ✓ |
| Nenner = workdays_per_week | expected_hours / workdays_per_week statt Anzahl gesetzter Wochentag-Flags | |

**User's choice:** Bestehenden Helper nutzen (= Empfehlung)
**Notes:** Reuse getesteter, bereits existierender Logik; Nenner = potential_days_per_week konsistent mit vorhandenem Helper.

---

## Claude's Discretion

- Exakter Toggle-Key-Name; ob `enabled` separater Master-Schalter oder aus `value`-Präsenz abgeleitet.
- Ableitung des absoluten Feiertags-Datums aus (year, calendar_week, day_of_week) für den Stichtag-Vergleich.
- Genaue Detektion „manueller Holiday-Eintrag deckt diesen Feiertag ab".
- i18n-Texte (de/en/cs) + Settings-UI-Layout (final in UI-/Plan-Phase).

## Deferred Ideas

- ShortDay/Kurztage-Automatik — Future-Story (Out-of-Scope bestätigt).
- Volle Urlaubsverwaltung für Freiwillige — Phase 26 liefert nur Jahresansicht-Reduktion.
- Ob Stichtag auch VFA-01 gated — Phase-26-Entscheidung (D-25-09).
