---
phase: 35-slot-einzelwoche-aenderung
plan: 02
subsystem: ui
tags: [dioxus, i18n, state, api, loader, frontend]

# Dependency graph
requires:
  - phase: 35-slot-einzelwoche-aenderung/35-01
    provides: PUT /shiftplan-edit/slot/{year}/{week}/single-week backend-Route
provides:
  - "SlotEdit.single_week: bool Zustandsfeld (Default false)"
  - "api::update_slot_single_week — ruft PUT /single-week-Route auf"
  - "loader::save_slot_single_week — delegiert an update_slot_single_week"
  - "SlotEditAction::SetSingleWeek(bool) — setzt single_week im Store"
  - "save_slot_edit-Routing — verzweigt bei single_week==true auf neuen Pfad"
  - "4 i18n-Keys (SlotEditModeScopeLabel/FromThisWeek/ThisWeekOnly/ThisWeekOnlyHint) in de/en/cs"
affects: [35-03-PLAN]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "single_week-Reset in new_slot_edit und load_slot_edit — saubere Rücksetzsemantik beim Öffnen des Editors"
    - "borrow-sicheres Auslesen von store-Feldern vor await in save_slot_edit"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
    - shifty-dioxus/src/state/slot_edit.rs
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/service/slot_edit.rs

key-decisions:
  - "[D-35-02] single_week: bool Default false — 100% Backward-Compat; bestehender save_slot-Pfad unverändert erreichbar"
  - "[D-35-02] set_single_week als eigenständige Hilfsfunktion extrahiert (nicht als Closure im Dispatcher) für bessere Testbarkeit"
  - "[D-35-02] borrow-sicheres Muster in save_slot_edit: Felder (single_week, config, slot, year, week) vor dem await aus store lesen"

patterns-established:
  - "Reset-Muster: SlotEditAction-Handler, der den Editor öffnet (new_slot_edit, load_slot_edit), setzt alle boolean-Flags zurück"
  - "API-Funktionspaar: update_slot / update_slot_single_week teilen sich dasselbe Payload-Format, unterscheiden sich nur in der URL"

requirements-completed: [SWO-01, SWO-04]

coverage:
  - id: D1
    description: "4 i18n-Keys (SlotEditModeScopeLabel, SlotEditModeFromThisWeek, SlotEditModeThisWeekOnly, SlotEditModeThisWeekOnlyHint) in allen drei Locales (de/en/cs) mit korrekten Übersetzungen und {week}/{year}-Platzhaltern im Hint-Key"
    requirement: SWO-04
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/i18n/mod.rs#i18n_slot_edit_mode_keys_present_in_all_locales"
        status: pass
    human_judgment: false
  - id: D2
    description: "SlotEdit.single_week: bool Feld im State (Default false) — setzt sich bei new_slot_edit und load_slot_edit zurück"
    requirement: SWO-01
    verification:
      - kind: unit
        ref: "cargo build shifty-dioxus: compiliert ohne Fehler"
        status: pass
    human_judgment: false
  - id: D3
    description: "api::update_slot_single_week und loader::save_slot_single_week — neuer API-Pfad auf PUT /shiftplan-edit/slot/{year}/{week}/single-week"
    requirement: SWO-01
    verification:
      - kind: unit
        ref: "cargo build --target wasm32-unknown-unknown: compiliert sauber"
        status: pass
    human_judgment: false
  - id: D4
    description: "SlotEditAction::SetSingleWeek(bool) und save_slot_edit-Routing: single_week==true → save_slot_single_week, sonst → bestehender save_slot-Pfad"
    requirement: SWO-01
    verification:
      - kind: unit
        ref: "cargo build --target wasm32-unknown-unknown: compiliert sauber, Routing im Code verifiziert"
        status: pass
    human_judgment: false

duration: 14min
completed: 2026-06-30
status: complete
---

# Phase 35 Plan 02: Frontend-Plumbing für Slot-Bearbeitungsmodus-Wahl Summary

**Single-week-Modus-Plumbing vollständig: 4 i18n-Keys (de/en/cs), single_week-Zustandsfeld, update_slot_single_week/save_slot_single_week-API+Loader und SetSingleWeek-Action mit save_slot_edit-Routing**

## Performance

- **Duration:** 14 min
- **Started:** 2026-06-30T18:08:46Z
- **Completed:** 2026-06-30T18:22:34Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- 4 neue i18n-Keys (SlotEditModeScopeLabel, SlotEditModeFromThisWeek, SlotEditModeThisWeekOnly, SlotEditModeThisWeekOnlyHint) in allen drei Locales inklusive {week}/{year}-Platzhalter im Hint; Presence-Test grün
- `SlotEdit.single_week: bool` (Default false) im State-Struct plus saubere Resets beim Öffnen des Editors; WASM-Build sauber
- `api::update_slot_single_week` (PUT …/single-week) und `loader::save_slot_single_week` als direktes Analog zu update_slot/save_slot; `SlotEditAction::SetSingleWeek(bool)` mit borrow-sicherem Routing in save_slot_edit

## Task Commits

1. **Task 1: 4 i18n-Keys (de/en/cs) + Locale-Presence-Test** — `1b84fd1` (feat)
2. **Task 2: single_week-State-Feld + api + loader** — `3862356` (feat)
3. **Task 3: SetSingleWeek-Action + Routing + Resets** — `757b435` (feat)

## Files Created/Modified

- `shifty-dioxus/src/i18n/mod.rs` — 4 neue Key-Varianten im Enum + i18n_slot_edit_mode_keys_present_in_all_locales-Test
- `shifty-dioxus/src/i18n/de.rs` — 4 deutsche Übersetzungen im Slot-edit-Abschnitt
- `shifty-dioxus/src/i18n/en.rs` — 4 englische Übersetzungen im Slot-edit-Abschnitt
- `shifty-dioxus/src/i18n/cs.rs` — 4 tschechische Übersetzungen im Slot-edit-Abschnitt
- `shifty-dioxus/src/state/slot_edit.rs` — `pub single_week: bool` in SlotEdit + Default false in new_edit()
- `shifty-dioxus/src/api.rs` — `update_slot_single_week` mit URL .../single-week
- `shifty-dioxus/src/loader.rs` — `save_slot_single_week` delegiert an api::update_slot_single_week
- `shifty-dioxus/src/service/slot_edit.rs` — SetSingleWeek(bool)-Variant, save_slot_edit-Routing, single_week=false-Resets

## Decisions Made

- `single_week: bool` Default false = "ab dieser Woche" — 100% Backward-Compat; bestehender save_slot-Pfad unverändert erreichbar (D-35-02)
- `set_single_week` als eigenständige Hilfsfunktion (kein Lambda im Dispatcher) — konsistentes Muster mit anderen Aktionshandlern
- Borrow-sicheres Muster in save_slot_edit: Felder vor dem await aus dem write-borrow lesen, um Borrow-Konflikte zu vermeiden

## Deviations from Plan

Keine — Plan exakt wie beschrieben umgesetzt. Einzige Abweichung ist die Extraktion von `set_single_week` als eigene Funktion (statt Inline-Lambda im Dispatcher), was das Muster der anderen Handler widerspiegelt und die Lesbarkeit verbessert.

## Issues Encountered

- WASM-Build benötigt `nix develop` für den lld-Linker (bekannt aus Umgebung). `cargo build --target wasm32-unknown-unknown` ohne nix-Shell schlägt fehl. Build aus nix-develop-Shell erfolgreich.
- Pre-existing-Testfehler `i18n_impersonation_keys_match_german_reference` (erwartet "Als diese Person agieren", de.rs liefert "🥸 Agieren") — besteht vor diesem Plan, kein Rückschritt.

## Next Phase Readiness

- Plan 03 (Radiogruppen-Komponente) kann sofort starten: alle Typen und Aktionen sind verfügbar
- `SLOT_EDIT_STORE.single_week`, `SlotEditAction::SetSingleWeek(bool)` und die 4 i18n-Keys sind einsatzbereit
- Backend-Route (Plan 01) und Frontend-Plumbing (dieser Plan) vollständig — Komponenten-Verdrahtung ist der letzte Schritt

---
*Phase: 35-slot-einzelwoche-aenderung*
*Completed: 2026-06-30*
