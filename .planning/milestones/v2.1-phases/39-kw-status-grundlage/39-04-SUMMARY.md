---
phase: 39-kw-status-grundlage
plan: 04
subsystem: frontend
tags: [frontend, dioxus, wasm, i18n, week_status, store, coroutine, api-client]

# Dependency graph
requires:
  - phase: 39-03-rest
    provides: "WeekStatusTO + WeekStatusKindTO (rest-types) + GET/PUT /week-status/by-year-and-week/{year}/{week}"
provides:
  - "state::week_status::WeekStatus (FE domain enum, 4 variants, Default=Unset) + From<&WeekStatusTO>/<&WeekStatusKindTO>"
  - "i18n keys WeekStatusUnset/InPlanning/Planned/Locked + WeekStatusSetError + WeekStatusChangeAriaLabel (de/en/cs) + 4x3 completeness test"
  - "api::get_week_status/set_week_status FE REST client"
  - "service::week_status::{WEEK_STATUS_STORE, WeekStatusStore, WeekStatusAction, week_status_service} fresh-fetch store"
  - "app.rs coroutine registration + Dioxus.toml /week-status dev proxy"
affects: [39-05-frontend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Fresh-fetch store (D-39-06): Set -> server PUT -> GET -> store; no optimistic signal (T-39-05)"
    - "FE domain enum mirrors backend 4-variant WeekStatus with Default=Unset (D-39-04/D-39-03)"
    - "Set-error surfaces a translated ShiftyError::Validation(WeekStatusSetError) via ERROR_STORE"

key-files:
  created:
    - shifty-dioxus/src/state/week_status.rs
    - shifty-dioxus/src/service/week_status.rs
  modified:
    - shifty-dioxus/src/state/mod.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/service/mod.rs
    - shifty-dioxus/src/app.rs
    - shifty-dioxus/Dioxus.toml

key-decisions:
  - "Fresh-fetch implemented by calling load_week_status() directly after a successful PUT (no coroutine self-handle needed) — same PUT-then-GET semantics without handle plumbing"
  - "Set-error message translated via I18N.read().t(WeekStatusSetError) and stored as ShiftyError::Validation, mirroring the employee-service Conflict pattern"
  - "WeekStatusAction carries #[allow(dead_code)] — send-sites (badge/dropdown) arrive in 39-05; this plan is plumbing-only"

requirements-completed: [WST-02, WST-05]

# Metrics
duration: ~12min
completed: 2026-07-02
status: complete
---

# Phase 39 Plan 04: KW-Status Frontend-Fundament Summary

**FE state/data layer for the KW status: the `WeekStatus` domain enum (Default=Unset), full de/en/cs i18n with a 4x3 completeness test, the `get_week_status`/`set_week_status` REST client, and a fresh-fetch store coroutine that re-loads from the server after every mutation — no optimistic signal. UI rendering is deferred to Plan 39-05.**

## Performance
- **Duration:** ~12 min
- **Tasks:** 2
- **Files:** 2 created, 9 modified

## Accomplishments
- `state/week_status.rs`: `WeekStatus { Unset (default), InPlanning, Planned, Locked }` (D-39-04, D-39-03). `From<&WeekStatusKindTO>`, `From<&WeekStatusTO>`, and reverse `From<&WeekStatus> for WeekStatusKindTO`. Four unit tests (default=Unset, all-four mapping, round-trip, TO mapping). Re-exported via `pub mod week_status;` in `state/mod.rs`.
- i18n: six new `Key` variants (`WeekStatusUnset/InPlanning/Planned/Locked/SetError/ChangeAriaLabel`) with de/en/cs translations per D-39-09 (de: Kein/In Planung/Geplant/Gesperrt; en: None/In planning/Planned/Locked; cs: Žádný/V plánování/Naplánováno/Uzamčeno + diacritics on the error/aria copy). Added `i18n_week_status_keys_present_in_all_locales` (proves 4x3 = all labels non-empty, non-"??" across de/en/cs) plus a German-reference guard against the Locale::En/De pitfall.
- `api.rs`: `get_week_status` (GET, 404 -> Ok(None)) and `set_week_status` (PUT `WeekStatusTO { year, calendar_week, status }`, `error_for_status_ref`) on `/week-status/by-year-and-week/{year}/{week}`.
- `service/week_status.rs`: `WEEK_STATUS_STORE: GlobalSignal<WeekStatusStore>` (`status`, `loaded_year`, `loaded_week`), `WeekStatusAction { Load, Set }`, and `week_status_service` coroutine. Fresh-fetch flow (D-39-06): `Set` -> `api::set_week_status` -> on Ok run `load_week_status` (GET, None -> Unset); on Err leave store untouched + raise a translated `WeekStatusSetError` banner. `Load` -> GET -> write store; on Err store keeps its last value.
- `app.rs`: `use_coroutine(service::week_status::week_status_service)`. `Dioxus.toml`: `[[web.proxy]] backend = "http://localhost:3000/week-status"` (Phase 28 dev-proxy precedent).

## Task Commits
1. **Task 1: FE enum + i18n de/en/cs + 4x3 completeness test** — `29b0d0b` (feat)
2. **Task 2: API client + fresh-fetch store + app/proxy registration** — `373fce5` (feat)

## Decisions Made
- **Fresh-fetch without a coroutine self-handle:** rather than self-sending `Load` after a `Set`, the coroutine calls the extracted `load_week_status(year, week)` helper directly. Same PUT-then-GET server roundtrip, no optimistic write, and no need to thread a `Coroutine` handle into the async fn. Store is only ever written from the GET path.
- **Translated Set-error:** on PUT failure the coroutine reads `I18N.read().t(Key::WeekStatusSetError)` and stores it as `ShiftyError::Validation(msg)` in `ERROR_STORE` (mirrors the employee-service Conflict notice), while also logging the raw reqwest error via `log_shifty_error`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `#[allow(dead_code)]` on `WeekStatusAction`**
- **Found during:** Task 2 (`cargo build --target wasm32-unknown-unknown`)
- **Issue:** The action variants are constructed only by the 39-05 send-sites (badge/dropdown), so the WASM build emitted a `dead_code` warning for `Load`/`Set`.
- **Fix:** Added `#[allow(dead_code)]` on the enum with a comment pointing to 39-05, matching the existing `ErrorAction` precedent. Keeps the build warning-free without inventing a premature consumer.
- **Files modified:** shifty-dioxus/src/service/week_status.rs
- **Commit:** 373fce5

## Deferred Issues
- **Pre-existing i18n test failure** `i18n::tests::i18n_impersonation_keys_match_german_reference` fails on the current tree independently of this plan (de.rs `ImpersonateActAs` = "🥸 Agieren" from commit `83a0d91`/feat 37-02 vs. the reference "Als diese Person agieren"). Logged to `deferred-items.md`; out of scope for 39-04 (Scope Boundary rule — not a week_status file).

## Threat Mitigations Applied
- **T-39-05 (Spoofing/Consistency, optimistic FE signal):** the store performs no optimistic update. After a successful PUT the coroutine re-fetches (GET) and only that server value is written — a driftable/false-status display is impossible.
- **T-39-01 (Elevation of Privilege, PUT):** FE only transports; authorization stays server-side (Wave 2/3). A non-shiftplanner PUT is rejected server-side with 403 and surfaces as the translated error banner.

## Gate Results
- `cargo test -p shifty-dioxus week_status` — pass (6/6: 4 enum + 2 i18n week-status tests)
- `cargo test -p shifty-dioxus i18n` — new week-status 4x3 completeness + German-reference tests pass; one **pre-existing unrelated** impersonation-reference failure remains (deferred above)
- `cargo build --target wasm32-unknown-unknown` (shifty-dioxus) — pass, warning-free
- `cargo clippy --workspace -- -D warnings` (backend root) — pass (dioxus is a separate workspace excluded from backend clippy per project convention)

## Next Phase Readiness
- Plan 39-05 can build `WeekStatusBadge` + the status dropdown on top of `WEEK_STATUS_STORE`/`WeekStatus`, send `WeekStatusAction::Load` on shiftplan render/week-switch and `Set` from the dropdown, and translate labels via the new i18n keys.
- No blockers.

## Self-Check: PASSED
- `shifty-dioxus/src/state/week_status.rs` present on disk.
- `shifty-dioxus/src/service/week_status.rs` present on disk.
- Commits 29b0d0b, 373fce5 exist in git history.

---
*Phase: 39-kw-status-grundlage*
*Completed: 2026-07-02*
