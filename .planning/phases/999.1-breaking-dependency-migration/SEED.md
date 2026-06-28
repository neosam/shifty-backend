# Phase 999.1 — Breaking/Major Dependency-Migration (Backlog Seed)

**Created:** 2026-06-28
**Status:** Backlog — not planned. Plan with `/gsd-plan-phase 999.1`.
**Milestone:** none (off-theme to v1.6 Paid-Capacity). Promote into a v1.7-Maintenance milestone or plan directly.

## Why this exists

Quick-Task `260627-vgo` updated all **semver-compatible** dependencies across both
Cargo workspaces (Backend-Root + `shifty-dioxus/`) — Cargo.lock only, all gates green.
The **breaking/major** portion was escalated (not forced through) because the pinned
toolchain can't automate it:

- stable **cargo 1.95.0** → `cargo update --breaking` is unstable (nightly-only, needs `-Z unstable-options`)
- no `cargo-edit` (`cargo upgrade`), no `cargo-outdated`, no `+nightly` toolchain

So every major bump would otherwise be hand work (per-crate crates.io research +
Cargo.toml constraint edits + API migration) across 9 member crates × 2 workspaces.

## Goal

Establish reproducible breaking-update tooling, then migrate all tractable major
bumps with green gates over both workspaces — without touching the fragile pins
(dioxus 0.6.x) unasked.

## Rough task structure

1. **Toolchain enabler** — add a nightly toolchain and/or `cargo-edit` + `cargo-outdated`
   to `flake.nix`, so `cargo update --breaking` or `cargo upgrade --incompatible`
   run reproducibly in the dev shell.
2. **Major-bump inventory** — which direct deps, which jump, changelog/breaking risk,
   per workspace. Produces the migration work-list.
3. **Iterative per-major migration** with gates:
   - Backend: `cargo build` + `cargo clippy --workspace -- -D warnings` + `cargo test --workspace`
   - Frontend: `cargo build --target wasm32-unknown-unknown` (via `nix-shell -p openssl pkg-config lld`) + `cargo test`

## Constraints

- **dioxus major** (0.6.x pin) ONLY with explicit user approval — dx-CLI 0.7 incompatibility
  is documented (app won't start + styling stripped). See memory `project_frontend_dx_version_pin`.
- `flake.lock` Nix inputs are NOT part of this phase (separate maintenance job).
- jj repo: user commits manually, no git fallbacks.
- Clippy `-D warnings` is a hard gate (nix build enforces it); cargo test does NOT run clippy.

## Depends on

- Quick-Task `260627-vgo` — compatible baseline (done). Diff base for the breaking work:
  current `Cargo.lock` + `shifty-dioxus/Cargo.lock` after the compatible update.
