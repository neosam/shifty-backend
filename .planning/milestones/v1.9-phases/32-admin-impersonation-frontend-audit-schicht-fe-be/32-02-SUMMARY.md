---
phase: 32-admin-impersonation-frontend-audit-schicht-fe-be
plan: "02"
subsystem: shifty-dioxus (frontend)
tags: [impersonation, api, service, frontend, wasm, phase32]
dependency_graph:
  requires: [32-01]
  provides: [impersonation-api-calls, impersonation-store, impersonation-service]
  affects: [shifty-dioxus/src/api.rs, shifty-dioxus/src/service/impersonate.rs, shifty-dioxus/src/service/mod.rs]
tech_stack:
  added: []
  patterns: [GlobalSignal+coroutine, pure-function-unit-tests, full-client-reload-teardown]
key_files:
  created:
    - shifty-dioxus/src/service/impersonate.rs
  modified:
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/service/mod.rs
decisions:
  - "D-32-03: user_id stored raw from ImpersonateTO — no name lookup, no DTO change."
  - "D-32-05: LoadStatus action fires GET /admin/impersonate so banner survives hard reload."
  - "D-32-06/IMP-04/SC1: both Start and Stop trigger window.location.reload() after success so all user-bound stores (including component-local current_sales_person) reinitialise — mirrors error.rs:66 pattern."
  - "403 FORBIDDEN from GET /admin/impersonate mapped to cleared loaded store (not an error) so non-admins see nothing."
metrics:
  duration: "~30 minutes"
  completed: "2026-06-29"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 3
status: complete
---

# Phase 32 Plan 02: FE Runtime — api.rs calls + service/impersonate.rs Summary

**One-liner:** Three impersonation api.rs client calls (start/stop/status) plus a GlobalSignal service with a pure unit-tested status-mapping helper and full-page-reload teardown on start and stop.

## Tasks Completed

| Task | Name | Status | Key Files |
|------|------|--------|-----------|
| 1 | Three api.rs client calls (start / stop / status) | Complete | shifty-dioxus/src/api.rs |
| 2 | service/impersonate.rs — store, pure status mapping, coroutine, start/stop reload | Complete | shifty-dioxus/src/service/impersonate.rs, shifty-dioxus/src/service/mod.rs |

## What Was Built

### Task 1 — api.rs additions

Added `ImpersonateTO` to the `rest_types` import block and three new async functions
at the bottom of `shifty-dioxus/src/api.rs`:

- `get_impersonate_status(config) -> Result<ImpersonateTO, reqwest::Error>` — GET `/admin/impersonate`. Used by `LoadStatus` at app-mount (D-32-05).
- `start_impersonate(config, user_id: ImStr) -> Result<ImpersonateTO, reqwest::Error>` — POST `/admin/impersonate/{user_id}` (user_id is a path segment, no body payload; D-32-03).
- `stop_impersonate(config) -> Result<ImpersonateTO, reqwest::Error>` — DELETE `/admin/impersonate`.

All three follow the existing reqwest conventions (`reqwest::get` for GET, `reqwest::Client::new()` for POST/DELETE, `error_for_status_ref()`, `.json().await?`, `info!` logging).

### Task 2 — service/impersonate.rs (new file)

Mirrors the `weekly_summary.rs` GlobalSignal+coroutine pattern:

**Store:**
```rust
pub struct ImpersonateStore { pub impersonating: bool, pub user_id: Option<ImStr>, pub loaded: bool }
pub static IMPERSONATE_STORE: GlobalSignal<ImpersonateStore> = GlobalSignal::new(ImpersonateStore::default);
```

**Pure helper (unit-tested):**
```rust
pub fn status_from_to(to: ImpersonateTO) -> ImpersonateStore { ... }
```
Maps `ImpersonateTO` → `ImpersonateStore` directly (D-32-03: `user_id` copied via `as_deref().map(ImStr::from)`, no lookup).

**Action enum:**
```rust
pub enum ImpersonateAction { LoadStatus, Start(ImStr), Stop }
```

**Service coroutine:**
- `LoadStatus` — calls `api::get_impersonate_status`; on Ok writes `status_from_to(to)` with `loaded=true`; on 403 FORBIDDEN writes a cleared loaded store (non-admin path, no error banner); on other errors routes to `ERROR_STORE`.
- `Start(user_id)` — calls `api::start_impersonate`; on Ok calls `window.location.reload()` (D-32-06/SC1); on Err routes to `ERROR_STORE`.
- `Stop` — calls `api::stop_impersonate`; on Ok calls `window.location.reload()` (D-32-06/IMP-04/SC4); on Err routes to `ERROR_STORE`.

Module doc comment explains the full-reload design rationale (component-local `current_sales_person` has no global handle).

**Module registered:** `pub mod impersonate;` added alphabetically between `i18n` and `slot_edit` in `service/mod.rs`.

## Verification Results

```
cargo test impersonate
  test service::impersonate::tests::status_from_to_defensive_impersonating_without_user ... ok
  test service::impersonate::tests::status_from_to_not_impersonating ... ok
  test service::impersonate::tests::status_from_to_impersonating_with_user ... ok
  test result: ok. 3 passed; 0 failed; 0 ignored

cargo build --target wasm32-unknown-unknown
  Finished `dev` profile [unoptimized + debuginfo] target(s)  (warnings only, no errors)
```

## Success Criteria — Verification

- [x] **IMP-02 / D-32-05**: `get_impersonate_status` + `LoadStatus` action loads status into `IMPERSONATE_STORE`; the pure `status_from_to` helper is unit-tested.
- [x] **IMP-04 / D-32-06 / SC4**: `Stop` action triggers `window.location.reload()` after successful DELETE, clearing all user-bound stores including component-local `current_sales_person`.
- [x] **D-32-06 / SC1**: `Start` action symmetrically triggers `window.location.reload()` so the impersonated view + banner re-initialize immediately.
- [x] **D-32-03**: `user_id` carried through unchanged; `ImpersonateTO` struct not modified.
- [x] **WASM build**: `cargo build --target wasm32-unknown-unknown` succeeds (warnings only, pre-existing).
- [x] **Unit tests**: 3/3 `status_from_to` tests green; `cargo test impersonate` passes.
- [x] **No new dependencies**: `reqwest`, `web_sys`, `dioxus` all already present.
- [x] **VCS**: No commits made; working tree left dirty for user review with `jj`.

## Deviations from Plan

None — plan executed exactly as written.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes introduced. The three new `api.rs` functions are client-side calls to existing backend endpoints (backend endpoints are admin-gated server-side, implemented in Plan 01). No new threat surface.

## Self-Check: PASSED

- `shifty-dioxus/src/api.rs` — modified (ImpersonateTO import + 3 new functions): confirmed
- `shifty-dioxus/src/service/impersonate.rs` — created (IMPERSONATE_STORE, ImpersonateAction, impersonate_service, status_from_to): confirmed
- `shifty-dioxus/src/service/mod.rs` — modified (`pub mod impersonate;` added): confirmed
- `cargo test impersonate`: 3/3 passed
- `cargo build --target wasm32-unknown-unknown`: succeeded
- `.planning/STATE.md`, `.planning/ROADMAP.md`: untouched (commit_docs:false, jj-managed)
- Backend (32-01) changes in `rest/` and `shifty_bin/`: untouched
