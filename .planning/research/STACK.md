# Stack Research — v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation

**Domain:** Brownfield additions to an existing Axum + SQLite + Dioxus/WASM app
**Researched:** 2026-06-29
**Confidence:** HIGH (derived from direct inspection of repo files at HEAD commit `905980b`)

> This is a **reuse map**, not a tech selection. The stack is fixed. For all four
> v1.9 features: **no new Cargo dependencies are needed** — backend or frontend.
> The following sections document exactly which existing primitives cover each
> feature and which integration points are already in place vs still to build.

---

## TL;DR — New Dependencies for v1.9

**Backend (workspace root): zero new crates.**
**Frontend (shifty-dioxus/): zero new crates.**

All four features are implementable with the crates already in `rest/Cargo.toml`
and `shifty-dioxus/Cargo.toml`.

---

## Feature-by-Feature Stack Analysis

### Feature 1 — Admin-Impersonation (READ + WRITE)

**Verdict: backend infrastructure is largely already implemented at HEAD.**

Direct code inspection confirms these pieces are in place:

| Component | File | Status |
|-----------|------|--------|
| `Session.impersonate_user_id: Option<Arc<str>>` | `service/src/session.rs:15` | **EXISTS** |
| `SessionService.start_impersonate()` / `stop_impersonate()` | `service/src/session.rs:50-55` | **EXISTS** |
| `SessionServiceImpl` with DB-level `update_impersonate` | `service_impl/src/session.rs:56-72` | **EXISTS** |
| REST handlers: `POST /{user_id}`, `DELETE /`, `GET /` | `rest/src/impersonate.rs` | **EXISTS** |
| Route mounted at `/admin/impersonate` | `rest/src/lib.rs:635` | **EXISTS** |
| `resolve_session_user_id()` propagates impersonated user as `Context` | `rest/src/session.rs:54-60` | **EXISTS** |
| Admin check uses REAL `session.user_id`, not impersonated | `rest/src/impersonate.rs:67-73` | **EXISTS** |
| `ImpersonateTO { impersonating, user_id }` DTO | `rest-types/src/lib.rs:1591` | **EXISTS** |

**What is NOT yet built for v1.9:**

1. **Audit identity propagation** — when impersonation is active, downstream handlers and
   tracing events should carry `real_user` so the audit trail does not lose the admin's
   identity. Implementation: in `context_extractor` (both mock and OIDC variants in
   `rest/src/session.rs`), insert a second Axum `Extension` for the real user when
   `impersonate_user_id` is Some:
   ```rust
   // new newtype to avoid clash with the existing Context (Option<Arc<str>>)
   pub struct RealUser(pub Arc<str>);
   // in context_extractor, when impersonation is active:
   request.extensions_mut().insert(RealUser(session.user_id.clone()));
   ```
   REST handlers that need to log writes can then extract `Extension<RealUser>` and emit:
   ```rust
   tracing::info!(
       real_user = %real_user.0,
       impersonating = %impersonated_as,
       "write-action-under-impersonation"
   );
   ```
   `tracing 0.1.41` is already in `rest/Cargo.toml`. No new crate.

2. **Frontend banner** — a Dioxus component that polls `GET /admin/impersonate` on mount
   and renders a prominent banner when `impersonating: true`. The `ImpersonateTO` DTO is
   already in `rest-types`. Use a `use_resource` or a new coroutine to fetch status on
   load. The banner provides start/stop buttons that call `POST /admin/impersonate/{user_id}`
   and `DELETE /admin/impersonate`. No new frontend crate.

**Write actions under impersonation: already work.** `context_extractor` resolves the
impersonated user as `Context = Option<Arc<str>>`. Every service call, including writes
(booking, absence, etc.), receives that context and operates as the impersonated user.
Nothing in the service layer needs to change.

**Privilege non-escalation: already correct.** All permission checks in
`service_impl/src/permission.rs` use `context` (which resolves to the impersonated user's
ID) to look up privileges in the DB. The admin does not gain the impersonated user's
privileges — the impersonated user's own privilege set is enforced.

**Crates used (all already present):**
- `axum 0.8.7` — `Extension`, middleware
- `tracing 0.1.41` — structured audit events
- `tower-cookies 0.10.0` — cookie extraction for session ID
- `serde_json 1.0.145` — `ImpersonateTO` serialization
- `rest-types` (path dep) — `ImpersonateTO` DTO

---

### Feature 2 — Stale-Daten-Race (week-switch coroutine guard)

**Verdict: pure Dioxus GlobalSignal state pattern, zero new crates.**

The race: `weekly_summary_service` in
`shifty-dioxus/src/service/weekly_summary.rs` processes `LoadWeek(y, w)` messages
sequentially (one at a time). After the HTTP fetch for week N completes, it writes to
`WEEKLY_SUMMARY_STORE` unconditionally — even if the user has already moved to week N+1
and that message is already queued. The store briefly shows stale data before the N+1
fetch completes.

Fix — generation token guard using existing `GlobalSignal`:

```rust
pub struct WeeklySummaryStore {
    pub weekly_summary: Rc<[WeeklySummary]>,
    pub data_loaded: bool,
    pub last_requested: Option<(u32, u8)>,  // NEW: (year, week) guard
}

async fn load_summary_for_week(year: u32, week: u8) -> Result<(), ShiftyError> {
    // Record what was requested BEFORE the async fetch
    (*WEEKLY_SUMMARY_STORE.write()).last_requested = Some((year, week));
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = false;

    let weekly_summary = loader::load_summary_for_week(..., year, week).await?;

    // Discard stale result if a newer request came in while we were fetching
    if WEEKLY_SUMMARY_STORE.read().last_requested != Some((year, week)) {
        return Ok(());
    }
    (*WEEKLY_SUMMARY_STORE.write()).weekly_summary = Rc::new([weekly_summary]);
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = true;
    Ok(())
}
```

The `LoadYear` path needs the same guard with `last_requested: None` or a separate
`last_requested_year: Option<u32>` sentinel.

**Crates used (all already present):**
- `dioxus 0.6.1` — `GlobalSignal`, coroutine `UnboundedReceiver`
- No new crates.

---

### Feature 3 — Urlaub → Nicht-Verfügbar (absence days as "discouraged" in shiftplan)

**Verdict: pure frontend rendering change, zero new crates.**

The data is already loaded. `absence_service` coroutine is registered in `app.rs:27`
and exposes absence periods in a global store. The shiftplan grid renders cells per
(employee, day). The fix is to cross-reference the employee's own absence periods
against the rendered cell date and apply a "discouraged" CSS class / visual indicator
when the date falls within a `Vacation` (and potentially other) absence category range.

No new HTTP calls, no new backend endpoints, no new crates. This is a rendering-logic
change in the shiftplan week view component and/or the cell rendering helper.

**Crates used (all already present):**
- `dioxus 0.6.1` — signal reads, RSX rendering
- `rest-types` — `AbsencePeriodTO`, `AbsenceCategoryTO`

---

### Feature 4 — Urlaubs-Balken-Konsistenz (vacation bar vs remaining-days number)

**Verdict: pure frontend calculation fix, zero new crates.**

The vacation balance data is already fetched by the existing `vacation_balance_service`
coroutine. The fix is in the progress-bar computation on the absences page: ensure the
bar reflects `used + planned` (with overage visually distinguishable) consistent with the
displayed remaining-days number.

No new HTTP calls, no backend changes, no new crates.

**Crates used (all already present):**
- `dioxus 0.6.1` — signal reads, RSX rendering
- `rest-types` — `VacationBalanceTO`

---

## Existing Stack (unchanged — for context)

| Layer | Technology | Version | Notes |
|-------|-----------|---------|-------|
| Backend web | Axum | 0.8.7 | REST handlers, middleware, extensions |
| Session/cookie | tower-cookies | 0.10.0 | `app_session` cookie extraction |
| Structured logging / audit | tracing | 0.1.41 | already used in `rest/src/lib.rs`, `rest/src/session.rs` |
| Serialization | serde + serde_json | 1.0.228 / 1.0.145 | all DTOs |
| Shared DTOs | rest-types (path dep) | — | includes `ImpersonateTO`, `AbsencePeriodTO`, `VacationBalanceTO` |
| Auth / RBAC | `Authentication<Context>` + `PermissionService` | — | `service/src/permission.rs`; impl in `service_impl/` |
| Session store | `SessionService` + `dao::session::SessionDao` | — | `service/src/session.rs`; SQLite-backed; already has `impersonate_user_id` column |
| Frontend framework | Dioxus | 0.6.1 | `GlobalSignal`, coroutines, RSX; dx-CLI pinned to 0.6.x in flake.nix |
| Frontend HTTP | reqwest | 0.12.15 | `features = ["json"]` |
| Dev shell | nix develop (flake) | — | provides sqlx-cli, sqlite; `shell.nix` is broken — use `nix develop` |
| VCS | jj (co-located git) | — | commit via jj, never `git commit` |

---

## What NOT to Add (explicit do-not list)

| Avoid | Why | Instead |
|-------|-----|---------|
| Any new Cargo dependency | All four features are covered by existing crates | Reuse existing stack |
| A dedicated `audit_log` service/table | The requirement is "audit keeps real admin identity" — `tracing` structured events on the REST layer satisfy this without a new DB table | Add `RealUser` Extension + `tracing::info!` at write sites |
| Storing `real_user` inside `Context` / changing `Authentication<Context>` | Changing the service-layer `Context` type from `Option<Arc<str>>` propagates to all service impls and mocks — massive blast radius for a REST-layer concern | Keep `Context` as `Option<Arc<str>>`; add a separate `RealUser` Axum Extension for REST-layer audit |
| Changing `PermissionService` to accept a dual-identity auth type | Overkill; non-escalation is already enforced by resolving impersonated user as `Context` | No change to permission service |
| `sqlx database reset` | Destructive; no new migration needed for v1.9 features | Not applicable — no schema changes in v1.9 |
| Snapshot version bump | None of the four features touch persisted `BillingPeriodValueType` computations | Do not bump — confirmed in PROJECT.md |

---

## Integration Points Summary

| Feature | Backend change | Frontend change |
|---------|---------------|-----------------|
| Admin-Impersonation READ+WRITE | `context_extractor`: insert `RealUser` Extension when impersonating; add `tracing::info!` audit events at write sites | New impersonation-banner component; poll `GET /admin/impersonate`; start/stop buttons |
| Stale-Daten-Race | None | Add `last_requested: Option<(u32, u8)>` to `WeeklySummaryStore`; guard writes in `load_summary_for_week` |
| Urlaub → Nicht-Verfügbar | None (data already in shiftplan view) | Rendering change in shiftplan cell component: cross-reference absence periods |
| Urlaubs-Balken-Konsistenz | None | Fix progress-bar math on absences page |

---

## Sources

- Repo files at HEAD (`905980b`), read directly — HIGH confidence:
  - `service/src/session.rs` (Session struct, SessionService trait)
  - `service/src/permission.rs` (Authentication<Context>, PermissionService)
  - `service_impl/src/session.rs` (SessionServiceImpl, start/stop_impersonate)
  - `service_impl/src/permission.rs` (check_permission, check_user)
  - `rest/src/impersonate.rs` (REST endpoints, admin gate logic)
  - `rest/src/session.rs` (context_extractor, resolve_session_user_id, Context type)
  - `rest/src/lib.rs` (RestStateDef, route wiring, error_handler)
  - `rest/Cargo.toml` (axum 0.8.7, tracing 0.1.41, tower-cookies 0.10.0)
  - `rest-types/src/lib.rs:1591` (ImpersonateTO)
  - `shifty-dioxus/src/service/weekly_summary.rs` (WeeklySummaryStore, coroutine pattern)
  - `shifty-dioxus/src/app.rs` (coroutine registrations)
  - `shifty-dioxus/Cargo.toml` (dioxus 0.6.1, reqwest 0.12.15)
  - `.planning/PROJECT.md` (v1.9 feature scope, snapshot-version note)
- Project memory: `reference_local_dev_commands`, `project_frontend_dx_version_pin`,
  `feedback_service_tier_convention` — HIGH confidence (user-confirmed conventions)

---
*Stack research for: v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation*
*Researched: 2026-06-29*
