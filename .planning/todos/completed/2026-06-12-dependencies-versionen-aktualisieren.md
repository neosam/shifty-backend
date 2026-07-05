---
created: 2026-06-12T07:38:21+0200
title: Alle Dependency-Versionen aktualisieren
area: tooling
files:
  - Cargo.toml
  - shifty-dioxus/Cargo.toml
---

## Problem

Die Dependencies im gesamten Monorepo (Backend-Workspace + `shifty-dioxus/`) sind
länger nicht aktualisiert worden. Ein turnusmäßiges Update steht an, um Security-
Fixes, Bugfixes und neue Features der Upstream-Crates mitzunehmen und Drift zu
vermeiden, der grössere Sprünge später teuer macht.

Betrifft beide Cargo-Workspaces:
- Backend-Root (`shifty_bin`, `service`, `service_impl`, `dao`, `dao_impl_sqlite`,
  `rest`, `rest-types`, `shifty-utils`)
- Frontend `shifty-dioxus/` (eigener Workspace, inkl. WASM-Target)

## Solution

TBD — grobe Richtung:
- `cargo update` für Patch-/Minor-Bumps in beiden Workspaces.
- Major-Bumps (z. B. Axum, SQLx, Dioxus, utoipa) einzeln prüfen — Breaking Changes
  möglich, Changelogs lesen.
- Nach jedem Schritt Regression-Gates fahren: `cargo build`, `cargo test` (Backend-Root)
  und `cargo build --target wasm32-unknown-unknown` im `shifty-dioxus/`.
- SQLx-Bump ggf. mit `sqlx prepare` / lokaler DB abgleichen (compile-time checked queries).
- Auf Dioxus-Major besonders achten (RSX/Hooks-Breaking-History).
