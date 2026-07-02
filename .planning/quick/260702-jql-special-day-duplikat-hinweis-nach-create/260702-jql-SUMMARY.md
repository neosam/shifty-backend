---
quick_id: 260702-jql
title: Special-Day Duplikat-Hinweis nach Create ausblenden, erst bei Feld-Änderung wieder
status: complete
mode: quick
subsystem: shifty-dioxus/settings
tags: [frontend, dioxus, special-days, ux, tdd]
requires:
  - Phase 42 (D-42-01 retained-form policy)
provides:
  - should_show_duplicate_hint pure fn
  - sd_dup_hint_suppressed suppression signal
affects:
  - shifty-dioxus/src/page/settings.rs
tech-stack:
  patterns: [pure-fn-extraction, controlled-signal-gate]
key-files:
  created: []
  modified:
    - shifty-dioxus/src/page/settings.rs
decisions:
  - "260702-jql: Duplikat-Hinweis-Sichtbarkeit als reine fn should_show_duplicate_hint(is_duplicate, suppressed) gekapselt (bewusste, enge Umkehr von D-42-03)."
  - "Task 1 RED/GREEN zu einem atomaren Commit zusammengefasst: ein Rust-Test auf eine noch nicht existierende fn schlägt beim Kompilieren fehl, nicht beim Assert — ein separater RED-Commit ist hier nicht sinnvoll."
metrics:
  tasks: 2
  files: 1
  completed: 2026-07-02
---

# Quick Task 260702-jql: Special-Day Duplikat-Hinweis nach Create unterdrücken Summary

Der Inline-Hinweis „existiert bereits" in Card-3 (Special-Days) erscheint nicht mehr direkt nach einem erfolgreichen Create — die seit Phase 42 (D-42-01) gefüllt bleibenden Formularfelder matchen den gerade angelegten Feiertag selbst; dieses Self-Match wird jetzt bis zur nächsten echten Feld-Änderung unterdrückt.

## What Was Built

- **Task 1 (TDD):** Reine Funktion `pub(crate) fn should_show_duplicate_hint(is_duplicate: bool, suppressed: bool) -> bool { is_duplicate && !suppressed }` plus 4 Unit-Tests, die die vollständige Wahrheitstabelle abdecken (true/false × suppressed).
- **Task 2:** Suppress-Signal `sd_dup_hint_suppressed` (default `false`) verdrahtet:
  - Create-Success-Arm setzt `true` (nach dem Retain der drei Felder).
  - Die drei `on_change`-Handler (Datum/Typ/Zeit) setzen `false`.
  - Render-Gate in Row D nutzt jetzt `should_show_duplicate_hint(sd_is_duplicate, sd_dup_hint_suppressed())` statt `if sd_is_duplicate`.

## Deviations from Plan

None - plan executed exactly as written.

Anmerkung zu Task 1: Der Plan erlaubt explizit einen atomaren Commit statt getrenntem RED/GREEN, falls der RED-Test ohne die fn nicht kompiliert. Genau das ist in Rust der Fall (Referenz auf nicht existierende fn ⇒ Compile-Fehler, kein sauberer RED-Assert-Fail), daher ein atomarer `test(...)`-Commit für die fn + Tests.

## Gate Results

- `cargo test duplicate_hint` (shifty-dioxus): 4 passed.
- `cargo build --target wasm32-unknown-unknown` (shifty-dioxus): warnungsfrei (Finished, keine warnings/errors).
- `cargo test` (shifty-dioxus, full): 756 passed, 1 failed — ausschließlich der bekannte pre-existing `i18n_impersonation_keys_match_german_reference` (nicht Teil dieser Aufgabe, unberührt).
- Backend `cargo clippy --workspace -- -D warnings`: unberührt (kein Backend-Code angefasst; nicht neu ausgeführt, da FE-only Scope).

## Scope Guard Compliance

FE-only (`settings.rs`). Kein Backend, keine API/TO-Änderung, kein Snapshot-Bump, keine Migration, keine neuen Deps. Controlled-Select / gefüllte Felder (D-06/D-08/D-42) unverändert — nur die Sichtbarkeit des Hinweises wird gesteuert; `sd_save_result` / „Gespeichert" (D-42-04) unangetastet.

## Commits

- `20066fd` test(260702-jql): add should_show_duplicate_hint pure fn + 4 unit tests
- `b9d270b` feat(260702-jql): suppress special-day duplicate hint after create

## Self-Check: PASSED

- FOUND: shifty-dioxus/src/page/settings.rs (should_show_duplicate_hint + sd_dup_hint_suppressed)
- FOUND commit: 20066fd
- FOUND commit: b9d270b
