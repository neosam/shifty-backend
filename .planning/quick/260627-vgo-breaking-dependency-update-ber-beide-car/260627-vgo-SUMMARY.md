---
quick_id: 260627-vgo
slug: breaking-dependency-update-ber-beide-car
date: 2026-06-27
status: complete
scope: partial — compatible-only delivered; breaking/major escalated
committed: false (jj repo — user commits manually)
---

# Quick Task 260627-vgo — Dependency-Update (beide Cargo-Workspaces)

One-liner: Semver-kompatibles `cargo update` in Backend + `shifty-dioxus/` (nur Cargo.lock), alle Gates grün; Breaking/Major bewusst eskaliert (kein `--breaking`/cargo-edit/nightly auf der stable-Toolchain).

## Was gemacht wurde

**Geändert:** nur `Cargo.lock` (Backend-Root) + `shifty-dioxus/Cargo.lock`. Keine `Cargo.toml`-Edits. `flake.lock` unberührt. Voll reversibel.

- Backend: ~168 Crates auf neueste semver-kompatible Versionen (u. a. axum 0.8.7→0.8.9, hyper 1.7→1.10.1, chrono 0.4.42→0.4.45, sqlx bleibt 0.8.6, indexmap, icu_*, zlib-rs 0.5→0.6.4 …).
- Frontend: kompatible Bumps; **dioxus bleibt 0.6.3** (Pin unberührt — kompatibles Update fasst keinen Major an).

## Gates — alle grün

| Gate | Ergebnis |
|------|----------|
| Backend `cargo build --workspace` | ✅ Finished (3m59s) |
| Backend `cargo clippy --workspace -- -D warnings` | ✅ 0 warnings (hartes nix-build-Gate) |
| Backend `cargo test --workspace` | ✅ ~583 grün (476 service_impl + 61 integration + Rest), 0 failed |
| Frontend `cargo build --target wasm32-unknown-unknown` (nix-shell -p … lld) | ✅ exit 0 (46 pre-existing dioxus-Warnings, nicht gated) |
| Frontend `cargo test` | ✅ 669 grün, 0 failed |

## Eskaliert (NICHT durchgewürgt) — Breaking/Major

Per Abmachung gestoppt, weil die Toolchain Breaking-Automation nicht hergibt:
- stable cargo 1.95.0 → `cargo update --breaking` ist unstable (nightly-only)
- kein `cargo-edit`/`cargo upgrade`, kein `cargo-outdated`, kein `+nightly`

Major/Breaking-Bumps wären reine Handarbeit (pro Crate crates.io-Recherche +
Cargo.toml-Constraints anheben über 9 Member-Crates × 2 Workspaces +
Compile-/API-Migration). Empfehlung: eigene `/gsd-phase` (Maintenance), idealerweise
mit nightly-Toolchain im flake für `cargo update --breaking` oder cargo-edit.

## Commit

NICHT committet (jj-Repo, User committet manuell). Working-Copy-Änderung:
`Cargo.lock` + `shifty-dioxus/Cargo.lock` (+ diese Quick-Doku).
