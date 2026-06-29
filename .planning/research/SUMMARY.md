# Project Research Summary

**Project:** Shifty v1.9 — Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation
**Domain:** Brownfield HR/shift-planning SaaS — four targeted feature additions to existing Axum + SQLite + Dioxus/WASM stack
**Researched:** 2026-06-29
**Confidence:** HIGH

## Executive Summary

Shifty v1.9 delivers four independent UX and admin improvements to a production Rust application. Three of the four features are pure frontend changes requiring no backend modifications: the vacation bar percentage formula fix (1-line change), the stale week-summary race guard (generation token in three sibling Dioxus stores), and the absence-to-discourage-marker join in the shiftplan grid. All data these features need is already loaded; no new HTTP endpoints are required for any of them. The recommended approach is to ship these three in order from lowest to highest risk before tackling Admin-Impersonation.

The Admin-Impersonation feature is the most nuanced. The headline distinction that MUST be understood: the impersonation MECHANISM is already complete in the backend. `Session.impersonate_user_id`, `start_impersonate`/`stop_impersonate` service methods, three REST endpoints at `/admin/impersonate`, and the `context_extractor` middleware that substitutes the effective identity — all of this exists and works. Write operations transparently run as the impersonated user. What is NOT yet built is (a) the AUDIT layer (real admin identity is silently dropped; a second Axum `Extension<RealUser>` must be inserted in `context_extractor` when impersonation is active, so write sites can log `real_user + impersonating`), and (b) the entire frontend (banner, status polling on mount, start/stop UI, and explicit store tear-down on stop).

The key risks are all in the impersonation feature: audit trail loss if the audit layer is deferred (cannot be retrofitted cheaply), silent impersonation after page reload if the frontend banner only sets state from the start-callback rather than fetching `GET /admin/impersonate` on mount, and stale frontend stores after stop-impersonation if each user-scoped store is not explicitly reloaded. All other three features carry low implementation risk — they are mechanical formula and signal-guard changes on code whose root causes have been confirmed by direct source inspection.

---

## Key Findings

### Recommended Stack

No new Cargo dependencies are needed for any of the four v1.9 features, backend or frontend. The stack is entirely fixed. All four features are implementable with crates already present: `axum 0.8.7`, `tracing 0.1.41`, `tower-cookies 0.10.0`, `serde_json 1.0.145`, `rest-types` (path dep), `dioxus 0.6.1`, and `reqwest 0.12.15`.

**Core technologies (unchanged):**
- `Axum 0.8.7` — REST handlers and middleware; `Extension` type used for the new `RealUser` audit extension
- `tracing 0.1.41` — structured audit events for impersonation write-path logging; already in `rest/Cargo.toml`
- `Dioxus 0.6.1` — `GlobalSignal`, coroutines, RSX; `dx-CLI` pinned to 0.6.x in `flake.nix`
- `rest-types` (path dep) — `ImpersonateTO`, `AbsencePeriodTO`, `VacationBalanceTO` all exist
- `jj` — VCS; `commit_docs: false` in GSD config; all commits are manual

**Critical version note:** `dx-CLI` must remain on 0.6.x (nixpkgs has rolled to 0.7.x which breaks the app). Pin is in `flake.nix`. Use `nix develop` (not `nix-shell`).

### Expected Features

**Must have (table stakes) for v1.9:**
- Vacation bar and remaining-days number measure the same quantity — `(used + planned) / total` — users trust neither when they disagree
- Stale-week race guard: summary cards always show data for the currently-selected week; three stores must be fixed atomically (WEEKLY_SUMMARY_STORE, BOOKING_CONFLICTS_STORE, reload_unavailable_days)
- Absence dates (all categories or an explicit whitelist) appear as discouraged cells in the shiftplan grid before booking, not only as a post-hoc warning
- Impersonation banner: persistent, non-dismissible, amber/yellow, with a one-click stop button — present on every page reload (requires `GET /admin/impersonate` on app mount)
- Real admin identity preserved in structured logs for all writes under impersonation
- Frontend stores cleared and reloaded for the real user on stop-impersonation

**Should have (competitive differentiators, defer if needed):**
- Two-segment vacation bar (used solid + planned lighter) — more information per pixel; purely additive
- Hover tooltip on discouraged shiftplan cell showing absence type and dates
- Overdraft overflow visual (bar continues past 100% mark in warning color)

**Defer to v2+:**
- Impersonation session auto-timeout (server-side `impersonate_expires_at`)
- Absence approval workflow (pending vs approved distinction)
- Audit log UI (DB-persisted impersonation history visible to other admins)
- Impersonating another admin (Pitfall 10 — document as known limitation for v1.9)

### Architecture Approach

All four features integrate into the existing layered architecture without redesigning it. The backend is REST → Service-trait → DAO-trait → SQLite; the frontend is Dioxus Pages → Components → Coroutine Services → GlobalSignals → loader/API. The impersonation mechanism splits at the `context_extractor` middleware: effective identity flows into `Extension<Context>` for all normal handlers; real identity must be added as a second `Extension<RealUser>` when impersonation is active. This is the ONLY backend change needed for v1.9.

**Components involved:**
1. `rest/src/session.rs` `context_extractor` — MODIFIED: insert `Extension<RealUser>` when `impersonate_user_id` is Some; no change to `Authentication<Context>` signatures
2. `rest/src/impersonate.rs` — MODIFIED: add `tracing::info!` audit events at start/stop; no structural change
3. `shifty-dioxus/src/service/weekly_summary.rs` and `booking_conflict.rs` — MODIFIED: add `(loaded_year, loaded_week)` guard fields; guard store writes after await
4. `shifty-dioxus/src/page/shiftplan.rs` — MODIFIED: merge absence-derived weekdays into `discourage_weekdays`; add render guard for stale-week; guard `reload_unavailable_days` closure
5. `shifty-dioxus/src/page/absences.rs` — MODIFIED: `PersonVacationCard` bar formula to `(used + planned) / total`
6. `shifty-dioxus/src/service/impersonate.rs` (NEW) — `ImpersonateStore` + coroutine for start/stop/status
7. `shifty-dioxus/src/component/impersonation_banner.rs` (NEW) — banner component; driven by `ImpersonateStore`
8. `shifty-dioxus/src/app.rs` — MODIFIED: mount banner + init impersonation coroutine with startup `GET /admin/impersonate` call

### Critical Pitfalls

1. **Audit trail loss (P1 — CRITICAL)** — `Context = Option<Arc<str>>` has a single slot; when impersonation is active, the real admin identity is dropped in `context_extractor`. Solution for v1.9: add a `pub struct RealUser(pub Arc<str>)` newtype as a second Axum `Extension`, inserted in `context_extractor` when `impersonate_user_id.is_some()`. REST write-site handlers extract `Extension<RealUser>` and emit `tracing::info!(real_user = %..., impersonating = %..., "write-under-impersonation")`. Do NOT change `Authentication<Context>` signatures — that has a massive blast radius. This must be locked before any write path is wired up.

2. **Admin gate checks impersonated user (P2 — HIGH)** — Any new admin-only endpoint that gates via the `context_extractor`-injected context will check the IMPERSONATED user's privileges and return 403 while an admin is impersonating a non-admin. Prevention: all admin-management endpoints must read the raw session cookie and construct `Authentication::Context(Some(session.user_id.clone()))` directly, as the three existing impersonate handlers already do.

3. **Frontend stores stale after stop-impersonation (P3 — HIGH)** — Dioxus global stores hold the impersonated user's data. When stop-impersonation succeeds, nothing broadcasts reload to user-scoped stores. Prevention: the `ImpersonationService` `StopImpersonation` handler must explicitly broadcast reload actions to every user-scoped store before returning.

4. **Banner missing after page reload (P9 — HIGH)** — If `ImpersonateTO` signal defaults to `{ impersonating: false }` on app mount, a hard reload while impersonating shows no banner. Prevention: `GET /admin/impersonate` must be the FIRST call in the app init sequence, before any user-data loads.

5. **Stale-week race in all three loaders (P4+P5 — MEDIUM)** — `load_summary_for_week`, `load_booking_conflict_week`, and `reload_unavailable_days` all perform unconditional writes after `await`. Fix all three in the same phase with the `(loaded_year, loaded_week)` guard pattern.

---

## Implications for Roadmap

### Phase 1: Urlaubs-Balken-Konsistenz (Vacation Bar Formula Fix)
**Rationale:** Smallest possible change, zero risk of regressions elsewhere, immediately visible correctness fix.
**Delivers:** Vacation bar and remaining-days number measure the same quantity; overdraft visually signaled.
**Addresses:** FEATURES Feature B; Pitfall P6
**Implements:** Single expression change in `absences.rs:866`; `(used + planned) / total` formula; overdraft warning color via existing `remaining_days <= 3.0` threshold (fires for negative values naturally)
**Avoids:** Silent overdraft clamped to 100%; bar-number contradiction that breaks HR trust
**Research flag:** SKIP — confirmed formula bug, confirmed fix, zero design ambiguity

### Phase 2: Stale-Daten-Race Guard (Week-Summary Generation Token)
**Rationale:** Pure frontend, low risk, self-contained. Must be fixed before Phase 3 adds another async load path to shiftplan. Fix atomically across all three loaders.
**Delivers:** Summary cards always show data for the currently-selected week; no mixed-state from rapid navigation.
**Addresses:** FEATURES Feature C; Pitfall P4, P5
**Implements:** Add `loaded_year: u32`, `loaded_week: u8` to `WeeklySummaryStore` and `BOOKING_CONFLICTS_STORE`; guard writes after await; render guard in shiftplan; guard `reload_unavailable_days` closure
**Avoids:** Partial-fix antipattern (fixing only WEEKLY_SUMMARY_STORE, leaving sibling loaders racy)
**Research flag:** SKIP — root cause confirmed from direct source; fix pattern is clear

### Phase 3: Urlaub → Nicht-Verfügbar (Absence Discourage Marker)
**Rationale:** Frontend-primary; adds a new async data load. Order after Phase 2 so shiftplan page is already race-guarded before adding more coroutine paths.
**Delivers:** Absence dates appear as discouraged cells in the shiftplan grid proactively.
**Addresses:** FEATURES Feature A; Pitfall P7, P11
**Implements:** `reload_absence_days` closure; `person_absences` signal; `absence_periods_to_discourage_days` pure helper function; merge into `discourage_weekdays` at `shiftplan.rs:1120`
**Avoids:** P7 (define explicit category whitelist `const`); P11 (gate absence fetch on `current_sales_person` being `Some`)
**Key decision for discuss-phase:** Which `AbsenceCategory` variants produce a discourage marker? Recommendation: all three (Vacation + SickLeave + UnpaidLeave), matching existing `BookingOnAbsenceDay` behavior.
**Research flag:** NEEDS DISCUSS — category whitelist must be a D-NN decision in CONTEXT before planning

### Phase 4: Admin-Impersonation Frontend + Audit Layer
**Rationale:** Largest surface; backend mechanism is already complete. Build last so simpler fixes ship independently.
**Delivers:** Admin can start/stop impersonation; persistent banner on all pages; audit log entries in structured logs with real admin identity; stores tear down and reload on stop.
**Addresses:** FEATURES Feature D; Pitfall P1, P2, P3, P8, P9, P10
**Implements:**
  - Backend: insert `Extension<RealUser>` in `context_extractor`; `tracing::info!` at start/stop in `impersonate.rs`
  - Frontend: `service/impersonate.rs` (new), `component/impersonation_banner.rs` (new), `api.rs` (3 new calls), `app.rs` mount + startup status fetch, i18n strings in 3 locales
**Avoids:** P1 via `RealUser` extension (NOT `Authentication<Context>` change); P9 via startup `GET /admin/impersonate`; P3 via explicit store tear-down on stop
**Known limitation:** Admin is locked out of admin-only endpoints while impersonating a non-admin (P10) — document in banner text and DISCUSS; correct behavior, not a bug
**Research flag:** NEEDS DISCUSS — audit design (RealUser extension vs Context struct) must be a D-NN decision before any write-path handler is coded

### Phase Ordering Rationale
- Phases 1-2 have zero network changes and zero inter-phase dependencies; clean baseline first.
- Phase 3 introduces a new async load path; placing it after Phase 2 means the race guard is already in place.
- Phase 4's backend work is trivially small but the design decision must precede any code; the frontend is the largest single-phase surface.
- All four phases can be verified independently: `cargo clippy --workspace -D warnings` + `cargo test --workspace` + WASM build gate + manual smoke.

### Research Flags
Phases needing recorded decisions in discuss-phase:
- **Phase 3:** Absence category whitelist must be D-NN in CONTEXT before planning.
- **Phase 4:** Audit design (RealUser extension approach) must be D-NN in CONTEXT before any write-path handler code. Admin-gate two-path contract must be documented in `rest/src/lib.rs` before new handlers are added.

Phases with standard patterns (skip research-phase):
- **Phase 1:** Single expression change; no design ambiguity.
- **Phase 2:** Generation-guard pattern established; three store locations identified.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All integration points verified by direct file reads at HEAD `905980b`; no new dependencies needed |
| Features | MEDIUM | Table stakes consistent across 4+ external sources; Shifty-specific integration paths verified by direct source read |
| Architecture | HIGH | All component boundaries, file paths, and line numbers confirmed by direct source inspection |
| Pitfalls | HIGH | All pitfalls derived from direct source inspection of affected call sites; root causes confirmed |

**Overall confidence:** HIGH

### Gaps to Address
- **Absence category policy (Phase 3):** Which categories produce discourage markers — product decision, not technical. Must be confirmed in discuss-phase.
- **Audit design choice (Phase 4):** `RealUser` extension approach vs full `Context` struct refactor. Recommend extension approach; must be ratified as D-NN in CONTEXT before code ships.
- **Impersonation entry point UI (Phase 4):** Where in admin UI to place "Impersonate" action — UX decision for discuss-phase.
- **Impersonation session lifetime:** `impersonate_expires_at` deferred to v2+; must be explicitly recorded as known limitation in Phase 4 DISCUSS.

---

## Sources

### Primary — HIGH confidence (direct codebase inspection, HEAD `905980b`)
- `service/src/session.rs`, `service_impl/src/session.rs` — Session struct, impersonate service methods
- `rest/src/impersonate.rs` — three REST handlers, admin gate using real `session.user_id`
- `rest/src/session.rs` — `context_extractor`, `resolve_session_user_id`, `Context` type alias
- `rest/src/lib.rs` — route wiring
- `service_impl/src/permission.rs` — `check_permission` effective-context-only confirmed
- `service_impl/src/booking_information.rs` — `period_overlaps_week` helper
- `shifty-dioxus/src/service/weekly_summary.rs` — unconditional write-after-await at lines 37-42
- `shifty-dioxus/src/service/booking_conflict.rs` — same pattern
- `shifty-dioxus/src/page/shiftplan.rs:1120-1123` — `discourage_weekdays` source; `reload_unavailable_days` at lines 350-368
- `shifty-dioxus/src/page/absences.rs:866-871` — bar math confirmed `used_days/total` only
- `shifty-dioxus/src/state/vacation_balance.rs` — all balance fields confirmed present
- `rest-types/src/lib.rs:1591` — `ImpersonateTO` confirmed

### Secondary — MEDIUM confidence
- [PropelAuth User Impersonation](https://docs.propelauth.com/overview/user-management/user-impersonation) — audit trail + privilege non-escalation conventions
- [Small Improvements User Impersonation](https://intercomdocs.small-improvements.com/en/articles/9146194-user-impersonation) — banner UX
- [Yaro Labs Safe Impersonation](https://yaro-labs.com/blog/user-impersonation-tool-saas) — session timeout, security invariants
- [Deputy Leave Management](https://help.deputy.com/hc/en-au/articles/4658289483023-Manager-s-awareness-of-leave) — proactive grid marking
- [When I Work Interpreting Availability](https://help.wheniwork.com/articles/interpreting-availability-on-the-schedule-computer/) — discourage cell conventions
- [Dayforce Your Balances](https://help.dayforce.com/r/documents/Employee-Guide/Your-Balances) — bar consistency, overdraft display
- OWASP CD-SEC-02 — privilege escalation prevention

---
*Research completed: 2026-06-29*
*Ready for roadmap: yes*
