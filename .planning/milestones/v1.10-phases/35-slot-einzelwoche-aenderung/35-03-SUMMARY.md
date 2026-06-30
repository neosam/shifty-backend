---
phase: 35-slot-einzelwoche-aenderung
plan: 03
subsystem: ui
tags: [dioxus, components, i18n, ssr-tests, radio-group, frontend]

# Dependency graph
requires:
  - phase: 35-slot-einzelwoche-aenderung/35-02
    provides: "SlotEditAction::SetSingleWeek(bool), SLOT_EDIT_STORE.single_week, 4 i18n-Keys"
provides:
  - "Sichtbare Modus-Radiogruppe im Slot-Editor (Edit-Modus): Ab dieser Woche / Nur diese Woche"
  - "Konditionaler Hinweis-Absatz bei single_week=true (text-small text-ink-muted)"
  - "SlotEditProps: single_week: bool + on_set_single_week: EventHandler<bool>"
  - "SlotEdit-Wrapper verdrahtet SetSingleWeek-Action"
  - "3 neue SSR-Tests: Edit-default, Edit-single-week, New-mode"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Visibility Gate per if-Block im RSX: Radiogruppe nur bei SlotEditType::Edit"
    - "Hint-Absatz innerhalb desselben if-Blocks, konditional auf single_week=true"
    - "Design-Token-only Klassen: accent-accent, border-border-strong, text-ink, text-ink-muted — kein Legacy-Palette"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/component/slot_edit.rs

key-decisions:
  - "[D-35-03] Radiogruppe als inline RSX-Block (kein eigenes Subkomponent) — minimal invasiv, kein neues Modul"
  - "[D-35-03] Hint-Abwesenheit-Assertion auf 'ausschließlich' (einzigartiger Token im Hint, nicht in der Erklärungsbullet)"

patterns-established:
  - "SSR-Test-Muster: SlotEditProps mit struct-update-Syntax (..props_with(…)) für Varianten"

requirements-completed: [SWO-01]

duration: 12min
completed: 2026-06-30
status: complete
---

# Phase 35 Plan 03: Modus-Radiogruppe im Slot-Editor Summary

**Radiogruppe "Ab dieser Woche (Standard) / Nur diese Woche" mit konditionalen Hinweis-Absatz im Edit-Modus, verdrahtet via SetSingleWeek-Action; 3 neue SSR-Tests + Legacy-Guard + WASM-Gate grün.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-06-30T18:20:00Z
- **Completed:** 2026-06-30T18:32:57Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- `SlotEditProps` erweitert: `single_week: bool` und `on_set_single_week: EventHandler<bool>` als Pflichtfelder
- Radiogruppe in `SlotEditInner` (nur bei `SlotEditType::Edit`): zwei Radio-Buttons mit Design-Token-Klassen (`accent-accent`, `border-border-strong`, `text-ink`, `text-ink-muted`), korrekte i18n-Bindings (SlotEditModeScopeLabel / SlotEditModeFromThisWeek / SlotEditModeThisWeekOnly / SlotEditModeThisWeekOnlyHint mit week/year-Interpolation)
- Konditionaler Hinweis-Absatz (`p.text-small.font-normal.text-ink-muted.mt-1`) bei `single_week=true` — kein Warn-Banner, rein informativ
- `SlotEdit`-Wrapper reicht `single_week` und `on_set_single_week` durch an `SlotEditInner`
- `props_with`-Hilfsfunktion in Tests um die neuen Pflichtfelder ergänzt (bestehende Tests wieder kompilierbar)
- 3 neue SSR-Tests: Edit-default (beide Labels, kein Hint), Edit-single-week (Hint mit week/year), New-mode (Radiogruppe absent)
- Alle 17 slot_edit-Tests grün inkl. `slot_edit_no_legacy_classes_in_source`
- WASM-Build via nix develop: sauber

## Task Commits

1. **Task 1: SlotEditProps + Radiogruppe + Hinweis + Wrapper-Verdrahtung** — `b909ce9` (feat)
2. **Task 2: SSR-Tests + Test-Helper-Update + Legacy-Guard + WASM-Gate** — `ccefa5e` (test)

## Files Created/Modified

- `shifty-dioxus/src/component/slot_edit.rs` — SlotEditProps erweitert, Radiogruppe + Hint in SlotEditInner (RSX), Wrapper-Verdrahtung, props_with-Update, 3 neue SSR-Tests

## Decisions Made

- Radiogruppe als inline RSX-Block in `SlotEditInner` ohne eigenes Subkomponent — minimal invasiv, kein neues Modul nötig (D-35-03)
- Hint-Abwesenheits-Assertion in SSR-Test nutzt "ausschließlich" als eindeutigen Token (im Hint, nicht in der bestehenden Erklärungsbullet) — robuster als Substring "Kalenderwoche 26" (D-35-03)

## Deviations from Plan

Keine — Plan exakt wie beschrieben umgesetzt. Einzige Abweichung war die Korrektur des Test-Assertions für die Hint-Abwesenheit: der erste Versuch nutzte "Kalenderwoche 26" als Prüfstring, was durch den bestehenden Erklärungstext fehlschlug. Korrigiert auf "ausschließlich" (unique im Hint-Text).

## Known Stubs

Keine — Radiogruppe ist vollständig verdrahtet, Hinweis-Text nutzt reale i18n-Übersetzungen.

## Threat Flags

Keine neuen Bedrohungsflächen eingeführt (rein UI-seitige Mutation, keine neuen Netzwerkendpunkte).

## Self-Check: PASSED

- [x] `shifty-dioxus/src/component/slot_edit.rs` vorhanden
- [x] Commit `b909ce9` existiert (feat: Task 1)
- [x] Commit `ccefa5e` existiert (test: Task 2)
- [x] `cargo test -p shifty-dioxus slot_edit`: 17/17 grün
- [x] `cargo build --target wasm32-unknown-unknown`: Finished dev profile

---
*Phase: 35-slot-einzelwoche-aenderung*
*Completed: 2026-06-30*
