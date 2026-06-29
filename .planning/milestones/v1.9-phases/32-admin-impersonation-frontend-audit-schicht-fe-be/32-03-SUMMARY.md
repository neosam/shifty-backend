---
phase: 32-admin-impersonation-frontend-audit-schicht-fe-be
plan: "03"
subsystem: frontend
tags: [impersonation, i18n, banner, user-management, dioxus]
dependency_graph:
  requires: [32-02]
  provides: [impersonation-banner-ui, impersonation-act-as-entry, impersonation-i18n]
  affects: [app.rs, user-management-page, i18n-all-locales]
tech_stack:
  added: []
  patterns:
    - prop-driven inner component for SSR testability (ImpersonationBannerView)
    - EventHandler threading from page to sub-component for coroutine isolation
key_files:
  created:
    - shifty-dioxus/src/component/impersonation_banner.rs
  modified:
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/cs.rs
    - shifty-dioxus/src/component/mod.rs
    - shifty-dioxus/src/app.rs
    - shifty-dioxus/src/page/user_management.rs
decisions:
  - "Used prop-driven ImpersonationBannerView as inner component to make SSR tests work without coroutine registration"
  - "Threaded on_impersonate EventHandler<ImStr> through UsersTabContentProps (consistent with on_request_delete pattern) to avoid use_coroutine_handle inside a testable sub-component"
  - "LoadStatus use_effect placed inside the config.backend non-empty if-block (consistent with existing title effect pattern)"
metrics:
  duration: ~45m
  completed: "2026-06-29"
  tasks_completed: 3
  tasks_total: 3
status: complete
---

# Phase 32 Plan 03: FE UI — i18n + Banner + app.rs Mount + Users-Tab Entry Summary

Delivered the complete impersonation UI layer on top of Plan 02's runtime: four i18n keys in all three locales (D-32-08), a persistent non-closable amber banner mounted above the router (D-32-04 / SC1 / SC2), and the "Act as this person" entry point in the Users tab (D-32-07 / IMP-01).

## What was built

### Task 1 — i18n keys for all three locales (D-32-08)

Added four `Key` variants to `i18n/mod.rs`:
- `ImpersonateActAs` — "Act as this person" / "Als diese Person agieren" / "Jednat jako tato osoba"
- `ImpersonateBanner` — banner body with `{user}` placeholder, e.g. "You are acting as {user}."
- `ImpersonateStop` — "Stop impersonation" / "Impersonation beenden" / "Ukončit personifikaci"
- `ImpersonateP10Hint` — known-limitation note (D-32-02a / P10)

Translations added to `en.rs`, `de.rs`, `cs.rs`. Three new i18n tests guard against locale-swap (Pitfall 2) and missing `{user}` placeholder. wasm build green.

### Task 2 — ImpersonationBanner component + app.rs mount (D-32-04 / SC1 / SC2)

Created `component/impersonation_banner.rs`:
- **`ImpersonationBannerView`** (private, prop-driven): renders nothing when `!impersonating`; renders amber bar (`bg-warn-soft border-l-4 border-warn text-warn` — static classes, Pitfall 5) with interpolated username (D-32-03: raw `user_id`), P10 hint, and Stop button (`BtnVariant::Secondary`). No close/dismiss control (D-32-04).
- **`ImpersonationBanner`** (public): reads `IMPERSONATE_STORE`, holds `use_coroutine_handle::<ImpersonateAction>`, delegates visual rendering to `ImpersonationBannerView`.

The inner/outer split allows SSR unit tests without needing the service coroutine registered. Two tests added:
- `banner_hidden_when_not_impersonating` — no amber bar, no Stop button
- `banner_shown_when_impersonating` — amber classes, interpolated "alex", Stop present, P10 hint, no close/dismiss

Registered in `component/mod.rs` (`pub mod impersonation_banner` + `pub use ImpersonationBanner`).

`app.rs` updated:
- `let impersonate_init = use_coroutine(service::impersonate::impersonate_service)` added alongside the other service coroutines
- `use_effect(move || { impersonate_init.send(ImpersonateAction::LoadStatus); })` fires once on mount (D-32-05 / SC2)
- `ImpersonationBanner {}` mounted inside the `authenticated` branch ABOVE `Router::<Route> {}` (D-32-04 / SC1)

### Task 3 — "Act as this person" in Users tab (D-32-07 / IMP-01)

`page/user_management.rs` updated:
- `use crate::service::impersonate::ImpersonateAction` imported
- `let impersonate_service = use_coroutine_handle::<ImpersonateAction>()` in `UserManagementPage`
- `on_impersonate: EventHandler<ImStr>` added to `UsersTabContentProps` (consistent with `on_request_delete`)
- Per-row: `Btn { variant: Secondary, on_click → on_impersonate.call(username_for_impersonate) }` with label `Key::ImpersonateActAs`
- Backend path verified: `user.username` IS the auth identity the POST `/admin/impersonate/{username}` endpoint expects (passes straight through to `ImpersonateAction::Start`)
- New test: `users_tab_act_as_renders_for_user_row` asserts the localized label renders for a user row
- Existing `render_users_with` helper updated to pass `on_impersonate: |_: ImStr| {}`

## Verification gates

| Gate | Result |
|------|--------|
| `cargo test impersonation_banner` | 3 passed (2 banner + 1 i18n placeholder) |
| `cargo test user_management` | 32 passed (all existing + 1 new) |
| `cargo test` (full suite) | **705 passed, 0 failed** |
| `cargo build --target wasm32-unknown-unknown` | **Success** (46 pre-existing warnings, no new errors) |

## Deviations from Plan

### Auto-decision: prop-driven inner component for banner SSR tests

**Found during:** Task 2 (TDD)

**Issue:** The `ImpersonationBanner` public component calls `use_coroutine_handle::<ImpersonateAction>()`. In SSR tests using `VirtualDom::new(fn)` without the service registered, this would panic.

**Fix:** Introduced a private `ImpersonationBannerView(store, on_stop)` prop-driven component that the public `ImpersonationBanner` delegates to. Tests call `ImpersonationBannerView` directly with no coroutine dependency. This is the established testability pattern (mirrors `UsersTabContent` + callbacks pattern already in the codebase).

**Files modified:** `component/impersonation_banner.rs` (design decision, no extra files)

### Auto-decision: EventHandler threading in UsersTabContent

**Found during:** Task 3

**Issue:** Adding `use_coroutine_handle::<ImpersonateAction>()` directly inside `UsersTabContent` would break the existing SSR tests (same coroutine-not-registered problem as above).

**Fix:** Added `on_impersonate: EventHandler<ImStr>` to `UsersTabContentProps` and obtained the handle in the parent `UserManagementPage`. This is consistent with how `on_request_delete` already works. Existing tests updated to pass `on_impersonate: |_: ImStr| {}`.

## Known Stubs

None. All impersonation actions are wired to the Plan 02 service coroutine.

## Deferred UAT (non-blocking manual smoke)

The plan marks browser-based visual verification as optional/non-blocking (not pixel-automatable):
- Admin starts impersonation from Users tab → amber banner appears on all pages with username + Stop
- Hard reload during impersonation → banner returns (SC2 / D-32-05)
- Click Stop → page reloads, banner gone, real admin view restored (SC4 / D-32-06)

These require a live backend + frontend session and are deferred to manual UAT.

## Threat Flags

No new trust-boundary surface introduced. The `on_impersonate` handler in `UsersTabContent` dispatches `ImpersonateAction::Start(username)` which calls POST `/admin/impersonate/{username}` — the server re-verifies admin identity against the raw `session.user_id` (T-32-07, mitigated by backend gate from Plan 01). No new endpoints or auth paths added in this plan.

## Self-Check: PASSED

- `component/impersonation_banner.rs` — created and compiles
- `component/mod.rs` — `pub mod impersonation_banner` + `pub use ImpersonationBanner` added
- `app.rs` — coroutine registered, LoadStatus effect wired, `ImpersonationBanner {}` mounted above `Router::<Route> {}`
- `page/user_management.rs` — `on_impersonate` prop threaded, button renders "Act as this person"
- `i18n/mod.rs` — 4 Key variants + 3 i18n tests added
- `en.rs / de.rs / cs.rs` — all 4 keys translated in all three locales
- Full `cargo test`: 705 passed, 0 failed
- `cargo build --target wasm32-unknown-unknown`: success
