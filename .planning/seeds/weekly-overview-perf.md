---
title: Weekly-Overview Performance-Optimierung
trigger_condition: nächster /gsd-new-milestone-Zyklus (aktuell zwischen Milestones nach v2.4)
planted_date: 2026-07-05
---

# Weekly-Overview Performance-Optimierung

## Trigger

Sobald der nächste Milestone aufgemacht wird (`/gsd-new-milestone`): dieses
Feature als Kandidat einbringen. Kein Blocker — der Endpoint funktioniert
korrekt, ist nur zäh. Aber User-sichtbar und Self-Service-relevant.

## Skizze: Feature

**Problem:** `GET /booking-information/weekly-resource-report/{year}` braucht
"ein paar Sekunden", während der Rest des Systems in ms antwortet. Ursache:
~165 sequenzielle Service-Calls in der Wochen-Schleife von
`BookingInformationServiceImpl::get_weekly_summary`. Detail: siehe
[[weekly-overview-perf-analyse]].

**Ziel:** Latenz auf sub-Sekunde bringen, ohne Verhalten zu ändern.

## Fixierte Constraints (User-bestätigt in Explore 2026-07-05)

- **Live-Korrektheit ist Pflicht.** Kein Cache-Layer, kein Snapshot. Wenn ein
  User eine Absence einträgt und sofort die Übersicht öffnet, muss die
  aktuelle Zahl stehen.
- **Scope: groß.** Nicht nur die "billigen" Bulk-Loads (special_days,
  shiftplan_reports), sondern auch `reporting_service.get_week` →
  `get_year`-Variante. Der User will den echten Fix, nicht nur die 60-%-Lösung.
- **Ergebnis muss byte-identisch bleiben.** Keine Chance-Freiheit für
  Rundungs- oder Semantik-Drift.

## Skizze: Umbau

### Chirurgischer Teil (kleiner Risiko)
Analog zum existierenden `all_work_details`/`all_absences`-Muster in
`booking_information.rs:291/300`:
- `special_day_service` bekommt eine `get_by_year(year, ...)`-Variante (oder
  die Wochen-Schleife lädt einmal alle Special-Days des Jahres und filtert
  in-memory pro Woche).
- `shiftplan_report_service` bekommt eine
  `extract_shiftplan_report_for_year(year, ...)`-Variante (oder ebenfalls
  Load-once + Wochen-Filter in-memory).

### Grösserer Teil (Risiko: zentraler Service)
- `reporting_service` bekommt eine `get_year(year, ...)`-Variante, die
  intern **alle** Wochen-Berechnungen in einem Rutsch macht:
  - Balance-Formel
  - CVC-06 Cap-Gating
  - Chain-C-Legacy-Filter unter `shortday_gate.active_from`
  - ShortDay-Slot-Clipping (v2.4)
- Die Wochen-Schleife in `get_weekly_summary` iteriert dann über das
  vor-berechnete Jahres-Aggregat.

## Harte Korrektheits-Gates (nicht verhandelbar)

- **Property-Test:** Für ein generiertes Jahres-Setup gilt: Ergebnis der
  neuen `get_year`-Variante ist byte-identisch zur alten
  Wochen-Iteration — für jede Woche, jeden Wert, jeden Volunteer.
  Diff-Toleranz: 0. Über N zufällig generierte Datensätze.
- **Alle existierenden Tests bleiben grün**, insbesondere:
  - `service_impl/src/test/booking_information.rs`
  - `service_impl/src/test/booking_information_chain_c.rs`
  - `service_impl/src/test/reporting*.rs`
- **CVC-06 Cap-Semantik** unangetastet.
- **`CURRENT_SNAPSHOT_SCHEMA_VERSION`** bleibt unverändert (kein Snapshot-
  Impact).
- **Manueller Verify:** Nach Umbau in Dev-DB Jahresansicht öffnen +
  A/B-Vergleich Zahlen vor/nach; Absence eintragen → Live-Update prüfen.

## Offene Fragen (siehe Q-02 in `.planning/research/questions.md`)

- Kann `reporting_service` eine `get_year`-Aggregation liefern, die alle
  Wochen-Invarianten exakt reproduziert — oder gibt es Berechnungen, die
  zwingend pro-Woche isoliert bleiben müssen (z.B. Toggle-Reads,
  ShortDay-Grenzen)?
- Wo genau wird `reporting_service.get_week` sonst noch konsumiert? Wenn
  andere Call-Sites nur eine Woche brauchen, bleibt die alte Variante
  koexistierend — dann keine Breaking Change im Service-Trait.

## Skizze: mögliche Phasen-Struktur

Kandidaten für die Milestone-Planung:

- **Variante 1 — Eine Phase, iterativ:** In einer Phase erst die billigen
  Bulk-Loads (special_days + shiftplan_reports), messen, dann
  `reporting_service.get_year`. Vorteil: Zwischen-Verifikation.
- **Variante 2 — Zwei Phasen:** Phase A = Bulk-Loads (leicht,
  risikoarm, sofort spürbar). Phase B = Reporting-Refactor (groß, zentraler
  Service). Vorteil: klare Trennung nach Risiko.

Entscheidung in `/gsd-discuss-phase` zum passenden Zeitpunkt.

## Verweise

- Hotspot-Analyse: [[weekly-overview-perf-analyse]]
- Research-Question: Q-02 in `.planning/research/questions.md`
- Betroffener Code:
  - `service_impl/src/booking_information.rs:259` (`get_weekly_summary`)
  - `service_impl/src/reporting.rs` (`get_week`)
  - `service/src/*.rs` (Trait-Definitionen für `get_year`-Ergänzungen)
