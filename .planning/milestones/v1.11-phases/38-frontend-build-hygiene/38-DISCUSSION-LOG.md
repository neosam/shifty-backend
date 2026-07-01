# Phase 38: Frontend-Build-Hygiene - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-01
**Phase:** 38-frontend-build-hygiene
**Areas discussed:** Dead-Code-Politik, Deprecated `time::parse`, Lint-Dokumentation, Scope/CI-Clippy-Gate, Verifikation

---

## A — Dead-Code-Politik (~34 warnings)

| Option | Description | Selected |
|--------|-------------|----------|
| Delete as default | Remove unused symbols; `#[allow(dead_code)]` only as justified exception | ✓ |
| Keep-list | Blanket-keep certain functions (register_user_to_slot etc.) for planned future use | |

**User's choice:** Delete as default. No blanket keep-list — named candidates are deletion
candidates unless a concrete reason (trait symmetry / planned API tied to open requirement /
avoid scope-blowing restructure) applies during implementation.
**Notes:** User stressed the frontend should actually be **fixed** (code removed), not just
suppressed where removal is cleaner.

---

## B — Deprecated `time::parse` (2 sites)

| Option | Description | Selected |
|--------|-------------|----------|
| Migrate to `parse_borrowed` | Pure API rename, no behavior change, removes warning cleanly | ✓ |
| `#[allow(deprecated)]` | Suppress instead of migrate | |

**User's choice:** Migrate to `parse_borrowed`. Fall back to allow only if migration
unexpectedly needs the format-version argument.

---

## C — Documentation of deliberately-kept lints

| Option | Description | Selected |
|--------|-------------|----------|
| Inline at symbol | `#[allow(...)] // reason: <why>` on the symbol + overview in log | ✓ |
| Central doc | Separate document listing kept lints | |

**User's choice:** Inline at the symbol, plus an overview list in CONTEXT/DISCUSSION-LOG.

---

## D — Scope / CI-Clippy-Gate

| Option | Description | Selected |
|--------|-------------|----------|
| dioxus stays out of CI clippy gate; rustc-only | Fix the 50 rustc warnings; do NOT touch the ~198 dioxus clippy lints | ✓ |
| Add dioxus to clippy gate | Clean all clippy lints + gate them | |

**User's choice:** dioxus stays out of the CI clippy gate; strict scope = only the 50 rustc
`cargo build` warnings; dioxus clippy untouched.
**Notes:** User asked what "dioxus bleibt draußen" meant → clarified: the backend
`cargo clippy --workspace -- -D warnings` gate doesn't cover the separate dioxus WASM
workspace; building an equivalent dioxus clippy gate would require clearing ~198 pre-existing
lints (own future phase). Phase 38 only needs dioxus rustc-warning-free.

---

## Verifikation (dx serve)

| Option | Description | Selected |
|--------|-------------|----------|
| Smoke-check as safeguard | Compiler+tests+WASM are the gate; run `dx serve` once as manual safeguard | ✓ |
| `dx serve` as hard gate | Mandatory browser check | |
| Compiler+tests+WASM only | No `dx serve` at all | |

**User's choice:** Safeguard is fine. Main thing: `cargo build` warning-free and the frontend
gets fixed.
**Notes:** Clarified WASM *build* only proves compilation, not runtime behavior; `dx serve`
smoke-check covers the unlikely case that "dead" code was reachable at runtime.

## Claude's Discretion

- Per-symbol delete-vs-keep judgment during implementation (apply the D-03 rule).
- Ordering: `cargo fix` for the 14 auto-fixable first, then manual dead-code + deprecated migration.

## Deferred Ideas

- dioxus clippy cleanup (~198 pre-existing lints) + adding dioxus to the CI clippy gate —
  out of scope, candidate for its own future phase.
