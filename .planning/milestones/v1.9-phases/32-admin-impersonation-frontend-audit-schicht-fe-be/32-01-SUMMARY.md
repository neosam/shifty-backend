---
phase: 32-admin-impersonation-frontend-audit-schicht-fe-be
plan: "01"
subsystem: backend-audit
tags: [impersonation, audit, security, middleware, integration-tests]
dependency_graph:
  requires: []
  provides: [RealUser-extension, audit-middleware, impersonate-tracing, two-path-doc]
  affects: [rest/session.rs, rest/impersonate.rs, rest/lib.rs, shifty_bin/integration_test]
tech_stack:
  added: []
  patterns: [tower-middleware, axum-extensions, tracing-structured-logging]
key_files:
  created:
    - shifty_bin/src/integration_test/impersonation_audit.rs
  modified:
    - rest/src/session.rs
    - rest/src/impersonate.rs
    - rest/src/lib.rs
    - shifty_bin/src/integration_test.rs
    - shifty_bin/Cargo.toml
decisions:
  - "RealUser newtype placed in rest/src/session.rs alongside context_extractor (module discretion, D-32-01)"
  - "audit_impersonated_writes uses from_fn (no state needed) and reads extensions captured before calling next"
  - "pub mod session + pub mod impersonate added to lib.rs to expose them for integration tests"
  - "tower-cookies 0.10.0 added as dev-dep to shifty_bin for CookieManagerLayer in probe tests"
metrics:
  duration: "~45 minutes"
  completed: "2026-06-29"
  tasks_completed: 3
  tasks_total: 3
  files_changed: 5
status: complete
---

# Phase 32 Plan 01: Backend Impersonation Audit Layer Summary

**One-liner:** RealUser newtype injected by context_extractor + central `audit_impersonated_writes` tower middleware logging real_user+acting_as for every mutating impersonated request, with start/stop tracing and two-path/P10 documentation.

## Objective

Add the backend audit schicht for admin impersonation satisfying IMP-03 (writes under impersonation auditable with the real admin identity) and IMP-01 gate guarantee (non-admin cannot impersonate), with regression tests pinning SC3 and SC5.

## Tasks Completed

### Task 1: RealUser newtype + pure helpers + context_extractor injection + start/stop tracing

**Files:** `rest/src/session.rs`, `rest/src/impersonate.rs`

- Added `pub struct RealUser(pub Arc<str>)` (Clone + Debug + Send + Sync + 'static) to `rest/src/session.rs`
- Added `pub fn real_user_extension(session)` returning `Some(RealUser(session.user_id))` when `impersonate_user_id.is_some()`
- Added `pub fn should_audit_impersonated_write(method, real_user_present)` returning true for POST/PUT/PATCH/DELETE when impersonating
- Added `pub async fn audit_impersonated_writes(request, next)` tower middleware emitting `tracing::info!` with real_user + acting_as + method + path fields
- Updated OIDC `context_extractor` (1 site) and mock_auth `context_extractor` (3 sites) to inject `RealUser` after `resolve_session_user_id` when impersonating
- Added `tracing::info!("impersonation started", ...)` before `start_impersonate()` call in `impersonate.rs`
- Added `tracing::info!("impersonation stopped", ...)` after `stop_impersonate()` call in `impersonate.rs`
- Added 11 unit tests in `session::tests` covering `real_user_extension` and `should_audit_impersonated_write` truth table

Verification: `cargo test -p rest --lib session` → **11/11 pass**

### Task 2: Central audit middleware mount + two-path/P10 documentation

**Files:** `rest/src/session.rs`, `rest/src/lib.rs`

- Changed `mod session;` to `pub mod session;` and `mod impersonate;` to `pub mod impersonate;` to allow integration tests to access route generators and `context_extractor`
- Added `pub use session::RealUser;` next to `pub use session::Context;` for `rest::RealUser` access in integration tests
- Extended `use session::{...}` import to include `audit_impersonated_writes`
- Mounted `.layer(middleware::from_fn(audit_impersonated_writes))` IMMEDIATELY BEFORE `context_extractor` layer in the chain — comment explains tower's reverse-source-order semantics so `context_extractor` runs first at request time
- Added doc comment at `/admin/impersonate` route nest documenting:
  - D-32-02 two-path invariant: handlers check raw `session.user_id`, never the impersonated Context
  - D-32-02a P10 limitation: while impersonating a non-admin, admin is locked out of admin-only endpoints but STOP works via raw session.user_id

Verification: `cargo build -p rest && cargo clippy -p rest -- -D warnings` → **CLEAN**; `grep -q "raw session.user_id" rest/src/lib.rs` → **FOUND**

### Task 3: Backend integration tests (SC5 gate, RealUser inject, P10 stop)

**Files:** `shifty_bin/src/integration_test/impersonation_audit.rs`, `shifty_bin/src/integration_test.rs`, `shifty_bin/Cargo.toml`

- Created `impersonation_audit.rs` with 3 tests and helpers:
  - `sc5_non_admin_cannot_start_impersonation`: NOBODY (no role) → 403; DEVUSER admin → 200
  - `real_user_injected_under_impersonation`: POST probe with impersonating session → context=TARGET + real_user=DEVUSER; plain session → context=DEVUSER + real_user=<none>
  - `p10_stop_works_while_impersonating_non_admin`: start impersonating NOBODY → 200; DELETE /admin/impersonate → 200 (raw session.user_id path)
- Added `mod impersonation_audit;` to `integration_test.rs`
- Added `tower-cookies = "0.10.0"` as dev dependency (already in lockfile as transitive dep from `rest`; needed for `CookieManagerLayer` in probe/impersonate test routers)

Verification: `cargo test -p shifty_bin impersonation_audit` → **3/3 pass**

## Full Verification Gate Results

| Gate | Result |
|------|--------|
| `cargo build --workspace` | CLEAN |
| `cargo clippy --workspace -- -D warnings` | CLEAN |
| `cargo test --workspace` | GREEN (64 integration tests in shifty_bin incl. 3 new; 11 session unit tests) |
| `cargo test -p rest --lib session` | 11/11 pass |
| `cargo test -p shifty_bin impersonation_audit` | 3/3 pass |
| `grep -q "raw session.user_id" rest/src/lib.rs` | FOUND |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] audit_impersonated_writes dead code during Task 1 isolation**
- **Found during:** Task 1 compilation check
- **Issue:** `audit_impersonated_writes` was defined in `session.rs` as part of the plan but caused a `dead_code` lint error because it wasn't yet used in `lib.rs` (that was Task 2's work). With `warnings = "deny"` in the workspace, this prevented `cargo test -p rest --lib session` from running.
- **Fix:** Tasks 1 and 2 were batched (lib.rs changes done before running the Task 1 verification gate). No code change was needed — just execution order adjusted.
- **Files modified:** None (ordering only)

**2. [Rule 2 - Missing] pub mod session + pub mod impersonate**
- **Found during:** Task 3
- **Issue:** The plan's Task 3 action requires `context_extractor` (from `rest::session`) and `generate_route` (from `rest::impersonate`) to be accessible from `shifty_bin` integration tests, but both modules were private (`mod session;`, `mod impersonate;`).
- **Fix:** Changed both to `pub mod session;` and `pub mod impersonate;` in `rest/src/lib.rs`. This is an additive, non-breaking change; no existing code was modified, only the visibility was widened.
- **Files modified:** `rest/src/lib.rs`

**3. [Rule 3 - Blocking] tower-cookies dev dependency**
- **Found during:** Task 3
- **Issue:** Integration tests need `CookieManagerLayer` to parse the `app_session` cookie header for the `context_extractor` and `impersonate` handler probes. `tower_cookies` was not listed as a direct dependency in `shifty_bin/Cargo.toml` (only a transitive dep via `rest`).
- **Fix:** Added `tower-cookies = "0.10.0"` to `[dev-dependencies]` in `shifty_bin/Cargo.toml`. This is already present in the workspace lockfile (zero new downloads), consistent with the "zero new deps" constraint for production code. A comment explains the reason.
- **Files modified:** `shifty_bin/Cargo.toml`

## Known Stubs

None — all functionality is fully wired: RealUser is injected from the real session store, audit middleware reads real extensions, integration tests use the real middleware stack.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes were introduced. The audit layer is purely additive (read-only access to existing extensions; no state mutation). The `pub mod session;` and `pub mod impersonate;` changes expose internal types to consumers of the `rest` crate but do not create new attack surface — the underlying handlers and session logic are unchanged.

## Self-Check: PASSED

All required files exist. All verification gates passed. No `Authentication<Context>` signature changes, no DB migration, no snapshot version bump.
