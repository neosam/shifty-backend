---
created: 2026-06-22
title: Committed Voluntary Capacity — zweiter Stundenwert + Jahresansicht
area: reporting / frontend
files:
  - service/src/employee_work_details.rs
  - dao/src/employee_work_details.rs
  - rest-types/src/lib.rs
  - service_impl/src/reporting.rs
  - service_impl/src/billing_period_report.rs (CURRENT_SNAPSHOT_SCHEMA_VERSION)
  - shifty-dioxus/src/page/weekly_overview.rs
  - shifty-dioxus/src/state/weekly_overview.rs
related:
  - 2026-06-09-auswertung-durchschnittliche-anwesenheit-flexible-stunden.md
priority: später — eigene Phase nach Phase 10 (Unavailability-Marker)
---

## Problem

Manche Personen haben einen kleinen bezahlten Vertrag (z. B. 5 h/Woche) und wollen
sich zusätzlich **freiwillig im Voraus** auf weitere Stunden committen (z. B. 5 h
freiwillig). Diese **zugesagte** Kapazität soll als verfügbare/geplante Kapazität in
der Jahresansicht (`weekly_overview`) sichtbar einberechnet werden.

Heute existiert nur:
- das boolean Flag `cap_planned_hours_to_expected` auf `EmployeeWorkDetails`
- die `VolunteerWork`-Kategorie (balance-neutral, `ReportType::Documented`)

Diese erfassen freiwillige Stunden **reaktiv** (Auto-Cap-Überlauf *nachdem* gebucht
wurde) — es gibt **keinen Wert für eine im Voraus zugesagte Kapazität**.

## Geklärte Design-Entscheidung (D-01): Variante B — separater Wert

Neues Feld auf `EmployeeWorkDetails`, z. B. `committed_voluntary: f32` — **nur die
freiwillige Zusage obendrauf**, NICHT als Gesamtsumme inkl. Vertrag.

Begründung (technisch favorisiert):
- entkoppelt von `expected_hours` → keine Invariante `committed >= expected` nötig
- unabhängige Zeit-Versionierung (ändert sich der bezahlte Vertrag, verschiebt sich
  der freiwillige Anteil NICHT still mit)
- additive Reporting-Formel statt subtraktiv
- Doppelzählungs-Vergleich läuft direkt auf der freiwilligen Achse
  (`actual_volunteer` vs `committed_voluntary`), ohne Zwischenrechnung

Verworfen: Variante A „zugesagte Gesamtstunden" (committed_total inkl. Vertrag) —
bräuchte Invariante + impliziten Subtraktionsschritt beim Lesen.

## Solution (Anforderungen)

1. **Datenmodell:** Neues zeit-versioniertes Feld `committed_voluntary` auf
   `EmployeeWorkDetails` (Service + DAO + rest-types + SQLite-Migration).
2. **Scope:** Gilt NUR für gedeckelte/freiwillige Personen
   (`cap_planned_hours_to_expected = true`). Für normale Mitarbeiter ohne Cap
   irrelevant.
3. **Jahresansicht — Einrechnung OHNE Doppelzählung:**
   - Verfügbare Kapazität = `expected + committed_voluntary`
   - Überschuss = `max(0, actual_volunteer − committed_voluntary)`
   - Beispiel: committed=5, actual=3 → Anzeige 5 (gedeckt);
     committed=5, actual=7 → Anzeige 5 + 2 Überschuss
   - Die heute schon eingerechneten geleisteten Volunteer-Stunden dürfen NICHT
     zusätzlich zählen, solange sie `committed_voluntary` nicht übersteigen.
4. **Jahresansicht — Darstellung:** committed-Kapazität SEPARAT ausweisen (nicht mit
   `paid`/`volunteer` vermischen).
5. **Mitarbeiteransicht:** „alle"-Filter einblendbar machen — rein unbezahlte
   Freiwillige brauchen künftig einen `EmployeeWorkDetails`-Record (Arbeitsvertrag),
   um `committed_voluntary` festzuhalten, und müssen sichtbar/auswählbar sein.

## Hinweise

- **Snapshot-Schema-Version** (`CURRENT_SNAPSHOT_SCHEMA_VERSION` in
  `service_impl::billing_period_report`) vermutlich bumpen, da sich die
  Reporting-Berechnung der Volunteer-/Kapazitäts-Werte ändert (siehe CLAUDE.md
  „Billing Period Snapshot Schema Versioning").
- Bezug zur bestehenden `weekly-planned-hours-cap`-Spec
  (`openspec/specs/weekly-planned-hours-cap/spec.md`) — baut darauf auf.
- Ausarbeitung als **eigene Phase nach Phase 10**. **Nicht jetzt umsetzen.**
