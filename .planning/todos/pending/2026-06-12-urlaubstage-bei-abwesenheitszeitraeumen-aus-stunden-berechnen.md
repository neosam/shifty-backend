---
created: 2026-06-12 18:18
title: Urlaubstage bei Abwesenheitszeiträumen aus Stunden berechnen (wie Report Service)
area: absence
files:
  - service_impl/src/absence.rs:387 (derive_hours_for_range — Per-Tag-Auflösung zu Stunden)
  - service_impl/src/absence.rs:519 (effective_hours mit day_fraction Half/Full)
  - service_impl/src/reporting.rs:584 (vacation_days/sick_leave_days/holiday_days fold)
  - service_impl/src/reporting.rs:513 (Kommentar: vacation_days = vacation_hours / hours_per_day)
  - service_impl/src/vacation_balance.rs:219 (CarryoverService::get_carryover → Carryover.vacation, carryover_days)
  - service_impl/src/reporting.rs:603 (previous_year_vacation → vacation_entitlement + previous_year_vacation)
  - shifty-dioxus/src/page/ (AbsencesPage — StatsGrid / VacationEntitlementCard Anzeige)
---

## Problem

Bei den **Abwesenheitszeiträumen** (Absence Periods, v1.0+) sollen die
**Urlaubstage anhand der Stunden** berechnet werden — **genauso wie es der
Report Service bereits macht** — statt Kalendertage naiv zu zählen.

Hintergrund / aktueller Stand:
- Der `ReportingService` leitet Urlaub bereits stundenbasiert ab: er resolved
  jeden Abwesenheitstag über den Arbeitsvertrag zu effektiven Stunden
  (`AbsenceService::derive_hours_for_range`, `absence.rs:387`) und rechnet dann
  `vacation_days = vacation_hours / hours_per_day` (vgl. Kommentar
  `reporting.rs:513` und der Tage-Fold `reporting.rs:584`).
- Diese Auflösung berücksichtigt bereits korrekt: Vertrags-Workdays
  (`has_day_of_week`), Feiertage (Holiday-Skip via `SpecialDayService`),
  `hours_per_day` aus dem Vertrag und `day_fraction` (Half = 0.5, Full = 1.0).
- In der Abwesenheits-Sicht selbst (AbsencesPage: StatsGrid /
  VacationEntitlementCard, zuletzt in 260612-o7t gebaut) werden die Urlaubstage
  aber vermutlich noch als reine Kalendertage / Range-Länge gezählt und decken
  sich damit nicht mit dem stundenbasierten Wert aus dem Reporting (Halbtage,
  Teilzeit mit < voller hours_per_day, übersprungene Feiertage/Nicht-Workdays
  weichen ab).

Ziel: Die in den Abwesenheitszeiträumen angezeigten/aggregierten Urlaubstage
sollen denselben stundenbasierten Wert liefern wie der Report Service, damit
beide Sichten konsistent sind.

**Zusatz (bei der Gelegenheit gleich mitnehmen): Vorjahres-Übertrag.** In der
Abwesenheits-/Entitlement-Sicht soll der **Übertrag aus dem Vorjahr** mit
einbezogen werden — genauso wie es bereits anderswo gemacht wird:
- `vacation_balance.rs:219` zieht den Übertrag via
  `CarryoverService::get_carryover` → `Carryover.vacation` (`carryover_days`)
  und rechnet `entitled_days + carryover_days - (used_days + planned_days)`.
- Das Reporting addiert `previous_year_vacation` auf das Entitlement
  (`reporting.rs:603` / `vacation_entitlement + previous_year_vacation`).
Die AbsencesPage / VacationEntitlementCard soll das verfügbare Urlaubskontingent
also als **Jahresanspruch + Vorjahres-Übertrag** ausweisen, nicht nur den reinen
Jahresanspruch — konsistent mit `vacation_balance` und Reporting.

Vor dem Planen zu klären:
- **Wo genau weicht es ab:** Backend-Aggregat im `AbsenceService` (eigene
  Tage-Zählung?) oder erst die Frontend-Darstellung (AbsencesPage)?
- **Single Source of Truth:** Bestehende stundenbasierte Logik aus
  `reporting.rs` extrahieren/wiederverwenden (gemeinsamer Helper), statt die
  Berechnung in der Absence-Sicht zu duplizieren.
- **Kategorien:** Gilt die stundenbasierte Day-Berechnung nur für Vacation oder
  analog auch für SickLeave / UnpaidLeave / Holiday (die im Reporting bereits
  alle stundenbasiert in Tage umgerechnet werden)?
- **Rundung/Darstellung:** Halbtage als 0.5 anzeigen? Auf welche Nachkommastelle?
- **Übertrag:** Bezugsjahr für `get_carryover` (laufendes Jahr → Vorjahr)?
  Vorhandene `vacation_balance`-Logik direkt wiederverwenden statt neu rechnen?

## Solution

TBD. Wahrscheinlich: die bereits vorhandene stundenbasierte
Tage-Ableitung (`derive_hours_for_range` → `hours / hours_per_day`) als
gemeinsamen Helper bereitstellen und in der Abwesenheits-Sicht
(Backend-Aggregat + AbsencesPage) statt der Kalendertag-Zählung verwenden, so
dass Urlaubstage 1:1 mit dem Report Service übereinstimmen.

Berührt Business-Logic-Tier (`AbsenceService` / ggf. geteilter Reporting-Helper)
und die Frontend-Anzeige. Bei Aufgriff über `/gsd-discuss-phase` einsteigen —
mehrere Definitionsfragen offen (Ort der Abweichung, Kategorie-Umfang).
Tests: Konsistenz Absence-Tage == Reporting-vacation_days für Halbtage,
Teilzeit, Feiertage, Nicht-Workdays.
