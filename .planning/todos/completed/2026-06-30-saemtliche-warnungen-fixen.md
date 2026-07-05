---
created: 2026-06-30T20:30:24.971Z
title: Sämtliche Compiler-/Clippy-Warnungen fixen
area: frontend
resolves_phase: 45
files:
  - shifty-dioxus/ (alle Crates)
  - shifty-dioxus/src/state/shiftplan.rs:315
---

## Problem

Der v2026.181.0-Release-Build hat **~45 Compiler-Warnungen im Frontend** (`shifty-dioxus`)
ausgegeben (z. B. ungenutzte Methode `has_sunday_slots` in `src/state/shiftplan.rs:315`;
14 davon via `cargo fix` auto-fixbar). `shifty-dioxus` ist ein **eigener Workspace mit
~198 pre-existing Lints** und vom **CI-Clippy-Gate ausgeschlossen** (siehe Memory
`reference_dioxus_clippy_not_gated`), daher sammeln sich die Warnungen an. Das Backend ist
sauber (CI erzwingt `cargo clippy --workspace -- -D warnings`).

Ziel: **warnungsfreier Build** über beide Workspaces.

## Solution

1. Frontend-Warnungen aufräumen:
   - `cargo build` (aus `shifty-dioxus/` via `nix develop`) für die `rustc`-Warnings;
     `cargo fix --bin shifty-dioxus -p shifty-dioxus` für die ~14 auto-fixbaren.
   - Clippy für den dioxus-Workspace ist im dioxus-Shell kaputt (E0514) → **clippy aus der
     Backend-nix-Shell** laufen lassen (Memory `reference_dioxus_clippy_not_gated`).
   - Rest manuell: ungenutzte Methoden/Imports/Variablen entfernen oder bewusst behalten
     (`#[allow(dead_code)]` + Begründung, falls geplante API wie evtl. `has_sunday_slots`).
2. Backend: bestätigen, dass `cargo clippy --workspace -- -D warnings` weiter sauber bleibt.
3. Optional: erwägen, den dioxus-Workspace ins CI-Clippy-Gate aufzunehmen, sobald die Lints
   abgebaut sind (verhindert Re-Akkumulation). Tangiert thematisch
   `reference_dioxus_clippy_not_gated`.

**Scope-Hinweis:** Größeres Aufräum-Paket (~198 Lints) — ggf. als eigene kleine Phase oder
gebündelt mit der Backlog-Dependency-Migration (999.1) planen.
