---
phase: 32-admin-impersonation-frontend-audit-schicht-fe-be
reviewed: 2026-06-29T00:00:00Z
depth: deep
files_reviewed: 13
files_reviewed_list:
  - rest/src/session.rs
  - rest/src/impersonate.rs
  - rest/src/lib.rs
  - shifty_bin/src/integration_test/impersonation_audit.rs
  - shifty_bin/src/integration_test.rs
  - shifty-dioxus/src/api.rs
  - shifty-dioxus/src/service/impersonate.rs
  - shifty-dioxus/src/service/mod.rs
  - shifty-dioxus/src/app.rs
  - shifty-dioxus/src/component/impersonation_banner.rs
  - shifty-dioxus/src/component/mod.rs
  - shifty-dioxus/src/page/user_management.rs
  - shifty-dioxus/src/i18n/mod.rs
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: resolved
resolution_note: "No blockers; security architecture confirmed sound. All 3 warnings FIXED (re-verified: clippy --workspace -D warnings EXIT 0, impersonation_audit 3/3, rest session 11/11): WR-01 — start-impersonation tracing moved AFTER the service call succeeds (no false-positive audit on a failed start, matches stop); WR-02 — stop audit now includes target_user = session.impersonate_user_id (self-contained trail); WR-03 — corrected the integration-test doc comment to not overclaim end-to-end (the probe asserts the audit middleware's INPUTS Context+RealUser for a mutating request; the emit decision is covered by the should_audit_impersonated_write unit truth-table). INFO-01 (dead '<unknown>' fallback, unreachable) and INFO-02 (.expect('no window'), mirrors existing error.rs pattern) ACCEPTED as harmless/consistent."
---

# Phase 32: Code Review Report

**Reviewed:** 2026-06-29
**Depth:** deep
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Phase 32 implements admin-impersonation FE+BE with an audit layer (SC3), admin-gate hardening (SC5), and a non-closable amber banner (D-32-04).

**Security architecture verdict: sound.** The four critical security properties all hold:

- **SC3 – Audit completeness:** `audit_impersonated_writes` is correctly placed as the inner middleware (Tower applies layers outermost-last; `context_extractor` is last-added, runs first, then `audit_impersonated_writes` reads the populated extensions). All four mutating methods (POST/PUT/PATCH/DELETE) are covered. Both `context_extractor` variants (oidc and mock_auth) inject `RealUser` in every reachable session branch.
- **SC5 – Admin gate:** All three impersonate handlers read `session.user_id` (server-side, never client-supplied) for the privilege check — an impersonated non-admin Context cannot grant admin access.
- **No privilege escalation via nested impersonation:** a non-admin cannot start a new impersonation because `start_impersonate` gates on the raw `session.user_id`.
- **FE reload is not looping:** `LoadStatus` does not trigger a reload; only explicit `Start`/`Stop` actions do.

Three warnings and two info items were found — all quality/completeness defects, none a blocker. The most actionable are WR-01 (false-positive audit entry on service failure) and WR-02 (incomplete stop audit entry).

---

## Warnings

### WR-01: Start-impersonation audit log emitted before the service call succeeds

**File:** `rest/src/impersonate.rs:87-95`

**Issue:** The `tracing::info!("impersonation started")` fires at line 87 before `start_impersonate(...).await?` at line 94. If the service call returns an error (e.g., a DB transient failure), the error is propagated to the caller as HTTP 500 — the impersonation never actually starts — but the audit trail has already recorded it as started. This creates a false-positive entry that an analyst would need to correlate away.

The `stop_impersonate` handler correctly places its log *after* the service call (lines 142-150), so the two handlers are inconsistent.

**Fix:** Move the `tracing::info!` block to after the `.await?`:
```rust
rest_state
    .session_service()
    .start_impersonate(session.id.clone(), Arc::from(target_user_id.as_str()))
    .await?;

// D-32-01: audit start of impersonation — logged after success only
tracing::info!(
    real_user = %session.user_id,
    target_user = %target_user_id,
    "impersonation started"
);
```

---

### WR-02: Stop-impersonation audit log missing the impersonated target user

**File:** `rest/src/impersonate.rs:147-150`

**Issue:** The "impersonation stopped" log records `real_user` only:
```rust
tracing::info!(
    real_user = %session.user_id,
    "impersonation stopped"
);
```
`session.impersonate_user_id` is loaded at lines 128-132 and remains in scope at this point (the local `session` variable is not modified by `stop_impersonate`). Without the target user, a forensics query of the form "who was admin X impersonating when they stopped?" requires a correlated lookup against the corresponding start entry. If log retention is limited, or the start entry was rotated, the stop entry is uninterpretable in isolation.

**Fix:**
```rust
tracing::info!(
    real_user = %session.user_id,
    target_user = %session.impersonate_user_id.as_deref().unwrap_or("<none>"),
    "impersonation stopped"
);
```

---

### WR-03: Integration test claims SC3 end-to-end coverage that it does not provide

**File:** `shifty_bin/src/integration_test/impersonation_audit.rs:173-174`

**Issue:** The doc comment on `real_user_injected_under_impersonation` reads:

> "A POST probe is used so the mutating-method path (SC3) is exercised end-to-end (SC3)."

However, `build_probe_router` (lines 101-114) only includes `context_extractor` and `CookieManagerLayer`. `audit_impersonated_writes` is not in the probe router. The test verifies that extension injection is correct — a prerequisite for SC3 — but it does not exercise the actual `tracing::info!` emission inside `audit_impersonated_writes`. If the audit middleware were accidentally unregistered or incorrectly placed, this test would not catch it.

SC3 is currently only verified by unit tests of the pure `should_audit_impersonated_write` predicate in `session.rs`, which do not exercise the middleware wiring.

**Fix (option A — minimal):** Update the comment to accurately describe what is tested:
```
// This test verifies that context_extractor correctly populates Context = TARGET
// and RealUser = DEVUSER under an active impersonation session.  SC3 (audit
// emission) is covered by unit tests of should_audit_impersonated_write in
// session.rs; the full middleware wiring is verified by manual inspection of
// the layer order in lib.rs.
```

**Fix (option B — stronger):** Add `audit_impersonated_writes` to the probe router and use a `tracing_subscriber` test layer (e.g., `tracing-test` crate) to assert that the `"impersonated write"` event fires for a POST with an active impersonation session:
```rust
fn build_probe_router(test_setup: &TestSetup) -> axum::Router {
    axum::Router::new()
        .route("/probe", axum::routing::post(probe_handler))
        .layer(middleware::from_fn(rest::session::audit_impersonated_writes))
        .layer(middleware::from_fn_with_state(
            test_setup.rest_state.clone(),
            rest::session::context_extractor::<crate::RestStateImpl>,
        ))
        .layer(CookieManagerLayer::new())
}
```

---

## Info

### IN-01: Dead fallback string `"<unknown>"` in audit middleware

**File:** `rest/src/session.rs:66-71`

**Issue:** The `if` block is only entered when `request.extensions().get::<RealUser>().is_some()` is true (line 66). The second `get::<RealUser>()` call inside the block (line 69) therefore always returns `Some`, making the `.unwrap_or_else(|| Arc::from("<unknown>"))` on line 71 unreachable dead code. This is a minor mislead — the `"<unknown>"` fallback implies the extension could be absent, which it cannot be.

**Fix:** Extract the `RealUser` in a single lookup:
```rust
if let Some(real_user) = request.extensions().get::<RealUser>() {
    if should_audit_impersonated_write(&method, true) {
        let real_user_id = real_user.0.clone();
        let acting_as: Arc<str> = request
            .extensions()
            .get::<Context>()
            .and_then(|c| c.clone())
            .unwrap_or_else(|| Arc::from("<none>"));
        tracing::info!(...);
    }
}
```

---

### IN-02: `web_sys::window().expect("no window")` — latent panic path (mirrors existing pattern)

**File:** `shifty-dioxus/src/service/impersonate.rs:123, 141`

**Issue:** Both the `Start` and `Stop` arms call `web_sys::window().expect("no window").location().reload()`. `.expect()` panics if `window()` returns `None`. In a browser WASM deployment this cannot happen; the comment notes this mirrors the existing `src/error.rs:66` pattern.

However, the codebase has safer variants in adjacent code (`js.rs:78` uses `.ok_or(...)`, `dialog.rs:306` uses `if let Some(window) = web_sys::window()`). The inconsistency means any future SSR effort or cargo test that exercises the service coroutine directly would hit a panic rather than a graceful no-op.

**Fix:** Match the safer pattern:
```rust
if let Some(window) = web_sys::window() {
    let _ = window.location().reload();
}
```

---

## Pre-existing Note (out of Phase 32 scope)

`rest/src/session.rs:146` (oidc `context_extractor`):

```rust
tracing::info!("All cookies: {:?}", cookies.list());
```

`cookies.list()` renders all cookie name/value pairs including the `app_session` UUID session token. Anyone with read access to INFO-level logs can extract valid session tokens and impersonate any logged-in user. This predates Phase 32 and is not changed here, but the new audit log entries (real_user, acting_as) appear in the same log stream, increasing the value of that log access to an attacker.

---

_Reviewed: 2026-06-29_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
