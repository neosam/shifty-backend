---
title: Weekly-Overview Performance-Analyse
date: 2026-07-05
context: explore session — Hotspot-Analyse für /weekly_overview/ vor nächster Milestone-Planung
---

# Weekly-Overview Performance-Analyse

## Trigger

`/gsd-explore`-Session am 2026-07-05. User-Beobachtung: Die Jahresübersicht
(`/weekly_overview/` im Frontend) ist "ein paar Sekunden" langsam, während
andere Endpoints in Millisekunden antworten.

Zweck dieses Notes: **Hotspot-Analyse + Nutzungsprofil festhalten**, damit die
spätere `discuss-phase` mit klaren Fakten startet.

Siehe Seed [[weekly-overview-perf]].

## Hotspot

**Datei:** `service_impl/src/booking_information.rs:311`
**Funktion:** `BookingInformationServiceImpl::get_weekly_summary(year, ...)`
**Endpoint:** `GET /booking-information/weekly-resource-report/{year}`

## Query-Zählung (Ist-Zustand)

Die Methode iteriert über `weeks_in_year(year) + 3` Wochen (also ~55). **Pro
Woche** ruft sie sequenziell drei Domain-Services:

1. `reporting_service.get_week(year, week, ...)`
2. `special_day_service.get_by_week(year, week, ...)`
3. `shiftplan_report_service.extract_shiftplan_report_for_week(year, week, ...)`

Ergibt **~165 sequenzielle Service-Calls** pro Jahresabruf. Jeder Service-Call
löst mindestens einen DAO-Roundtrip aus; `reporting_service.get_week` ist der
teuerste, weil er intern Balance-Berechnung + Cap-Gating (CVC-06) macht.

**Load-once ist teilweise schon vorhanden** (`all_work_details`, `all_absences`,
`active_from` Toggle) — das Muster ist etabliert, aber für die drei genannten
Services nicht durchgezogen.

## Nutzungsprofil

- **Self-Service:** Jeder Mitarbeiter öffnet die Ansicht, um vor einem
  Urlaubs-/Absence-Eintrag zu prüfen, ob er dem Laden schadet.
- **Planner-intensiv:** Schichtplaner nutzen sie regelmäßig.
- **Live-Korrektheit gefordert:** Wenn ein User eine Absence einträgt und
  sofort die Übersicht öffnet, muss die aktuelle Zahl stehen.
  → Cache-/Snapshot-Layer wurde in der Explore-Session verworfen.

## Entschieden: Route A (algorithmisch), Scope groß

- **Kein Cache-Layer.** Live-korrekt bleibt Pflicht.
- **Bulk-Load für `special_days` und `shiftplan_reports`** — analog zum
  existierenden `all_work_details`-Muster (Zeile 291 in derselben Methode).
- **`reporting_service.get_year`-Variante** — der größere Umbau. Zentraler
  Service, betrifft Balance-Formel, CVC-06-Cap, Chain-C-Legacy-Filter unter
  `shortday_gate.active_from`.
- **Nicht verhandelbar:** Ergebnis muss byte-identisch zur aktuellen
  Wochen-Iteration bleiben.

## Risiken / Regressionsflächen

- `reporting.rs` — Balance-Berechnung, Chain-C-Legacy-Filter (D-51-06/07)
- CVC-06 Cap-Gating in `booking_information.rs`
- Chain-C-Tests (`test/booking_information_chain_c.rs`) — müssen grün bleiben
- ShortDay-Slot-Clipping (v2.4 gerade geshipped) — Slot-Filter muss über die
  neue `get_year`-Aggregation gleich wirken wie über die alte Wochen-Iteration

## Nicht (mehr) diskutiert

- HTTP-Caching / ETag / Snapshot-Cache — verworfen wegen Live-Korrektheit.
- Parallelisierung via `join_all` — SQLite serialisiert intern, marginal.
- Pagination der Übersicht — Jahres-View ist konzeptionell atomar.

## Verweise

- Codebase-Anker: `service_impl/src/booking_information.rs:311`
- REST-Handler: `rest/src/booking_information.rs:58` (`get_weekly_summary`)
- Frontend-Konsument: `shifty-dioxus/src/service/weekly_summary.rs` +
  `shifty-dioxus/src/loader.rs:496` (`load_weekly_summary_for_year`)
- Load-once-Vorbild in derselben Methode: Zeile 291 (`all_work_details`),
  Zeile 300 (`all_absences`)
