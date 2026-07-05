# Phase 52: weekly-overview-performance-refactor - Discussion Log

**Date:** 2026-07-05
**Mode:** discuss (Textform, User-Delegation aller Entscheidungen)

## Setup

- GSD-Tools (`gsd-tools.cjs`) beim Init crashen weiter (`fix-slash-commands.cjs`
  fehlt — bekannter Zustand aus Memory `reference_gsd_tools_roster_broken.md`).
  Workflow manuell durchgeführt.
- Phasen-Verzeichnis `.planning/phases/52-weekly-overview-performance-refactor/`
  angelegt (existierte noch nicht).

## Prior Context Geladen

- `.planning/PROJECT.md`, `.planning/STATE.md`
- `.planning/REQUIREMENTS.md` v2.5 (WOP-01..05 + VAA-01..04)
- `.planning/ROADMAP.md` (Phase 52 + 53 Definitionen)
- `.planning/notes/weekly-overview-perf-analyse.md` (Hotspot, Query-Zählung)
- `.planning/seeds/weekly-overview-perf.md` (Umbau-Skizze, Korrektheits-Gates)
- `.planning/research/questions.md` §Q-02
- Prior CONTEXT.md Vorbild: `50-CONTEXT.md`
- Code-Scan: `booking_information.rs:259` (Hotspot), `reporting.rs:884`
  (`get_week`), Trait-Signaturen für `_for_year`, `slot_service`, `rest/src/report.rs`
  (externer `get_week`-Konsument).

## Codebase-Scan-Findings

- `SpecialDayService::get_by_year` **existiert schon** — kein neuer Trait-Method.
- `ShiftplanReportService` hat **keine** `_for_year`-Variante.
- `ReportingService::get_week` liest den `shortday_gate`-Toggle **nicht**
  intern (Toggle-Read passiert nur in `booking_information.get_weekly_summary`
  für Slot-Clipping).
- `reporting.get_week` externer Konsument: nur `rest/src/report.rs:148`
  (`/report/week/{year}/{week}`). Interner: 2× in `booking_information.rs`.
- `slot_service.get_slots_for_week_all_plans` wird ebenfalls ~55× gerufen —
  nicht explizit in WOP-01 genannt, aber relevanter Anteil an der Latenz.

## Gray Areas (6 Stück) — alle vom User an Claude delegiert

**User-Antwort:** „Ich finde, du kannst alles entscheiden. Es soll einfach
schnell sein aber das Ergebnis sollte unverändert sein"

Entscheidungen (siehe `52-CONTEXT.md` §Implementation Decisions):

| # | Gray Area | Entscheidung | ID |
|---|-----------|-------------|-----|
| G1 | Scope Bulk-Load — Slot-Batching mit rein? | Ja, `slot_service.get_slots` einmal + In-Memory-Filter | D-52-01 |
| G2 | `get_year` Return-Shape | `Arc<[(u8, Arc<[ShortEmployeeReport]>)]>` (Vec sortiert nach week) | D-52-02, D-52-03 |
| G3 | Spillover-Wochen (weeks_in_year + 3) | 2× `get_year(year)` + `get_year(year+1)` | D-52-04, D-52-05 |
| G4 | Neue `_for_year`-Trait-Methoden | Nur `ReportingService::get_year` + `ShiftplanReportService::extract_shiftplan_report_for_year`; `SpecialDayService::get_by_year` existiert, `SlotService` bleibt unverändert | D-52-06, D-52-07 |
| G5 | `get_week` vs. `get_year` Duplikation | Gemeinsamer `assemble_weeks`-Helper (`pub(crate)`); `get_week` delegiert intern | D-52-08, D-52-09, D-52-10 |
| G6 | Property-Test Ansatz für Byte-Identität | Fixture-Tabelle (8 Szenarien) + `f32::to_bits()`-Vergleich; kein `proptest`-Dep | D-52-11, D-52-12, D-52-13, D-52-14 |

## Claude's Discretion (nicht als Gray Area präsentiert, aus Code-Kontext + Memory abgeleitet)

- **Frontend-Impact:** Keine (D-52-15). DTO unverändert, kein FE-Rebuild.
- **Messmethode WOP-04:** `curl -w "%{time_total}"` 5-Runs-Median gegen Dev-DB
  (D-52-16).
- **Docs-Freshness (F07):** Reine Refactor-Phase, Balance-Formel unverändert
  — Planner prüft am Ende ob `F07-reporting-balance.md/de.md` überhaupt
  angefasst werden muss.
- **sqlx prepare + Clippy-Gate** aus Memory heraus explizit in
  `<downstream_hooks>` verankert.

## Deferred Ideas (Noted for Later)

- VAA-01..04 → Phase 53 (bereits geplant).
- `SlotService::get_slots_for_year` — bewusst NICHT eingeführt.
- HTTP-Caching / ETag / Parallelisierung via `join_all` — verworfen.

## Next Step

`/gsd-plan-phase 52` — Planner konsumiert CONTEXT.md und erzeugt PLAN.md.
