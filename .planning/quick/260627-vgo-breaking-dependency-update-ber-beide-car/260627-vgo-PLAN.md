---
quick_id: 260627-vgo
slug: breaking-dependency-update-ber-beide-car
date: 2026-06-27
mode: quick (inline, no subagent — jj repo + escalation gate)
---

# Quick Task 260627-vgo: Dependency-Update (beide Cargo-Workspaces)

## Ziel
Dependencies in Backend-Root + `shifty-dioxus/` aktualisieren. Nix `flake.lock`
NICHT anfassen. User-Wunsch: „auch Major/Breaking".

## Eskalations-Gate (Abmachung)
Falls Breaking/Major eine große manuelle Migration erfordert → abbrechen und als
eigene `/gsd-phase` eskalieren statt durchzuwürgen.

## Befund beim Scope-Assessment
- Toolchain ist **stable cargo 1.95.0** (nix-gepinnt). `cargo update --breaking`
  ist unstable (braucht nightly `-Z unstable-options`) → nicht verfügbar.
- Kein `cargo-edit` (`cargo upgrade`), kein `cargo-outdated`, kein `+nightly`.
- Major/Breaking-Bumps wären daher rein manuell (pro Crate crates.io-Recherche +
  Cargo.toml-Constraint-Edits über 9 Member-Crates × 2 Workspaces + Compile-
  Migration). → **Gate ausgelöst: Breaking-Teil eskaliert, NICHT durchgewürgt.**

## Durchgeführt (sicherer Teil)
Semver-**kompatibles** `cargo update` (nur Cargo.lock, keine Cargo.toml-Edits,
reversibel) in beiden Workspaces, hinter allen Gates.

## Gates
- Backend: `cargo build --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
- Frontend: `cargo build --target wasm32-unknown-unknown` (nix-shell -p … lld), `cargo test`
