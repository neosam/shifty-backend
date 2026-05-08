---
phase: 08-absence-crud-page-foundation
plan: 04
subsystem: ui
tags: [shifty-dioxus, absence, frontend, api, state, service-coroutine, i18n, dx-proxy, error-variant, vacation-balance]

# Dependency graph
requires:
  - phase: 08-absence-crud-page-foundation
    provides: VacationBalanceTO + AbsencePeriodTO from rest-types (Plan 08-01); /absence-period CRUD + /vacation-balance/{sp}/{year} + /vacation-balance/team/{year} REST endpoints (Plan 08-02); OpenAPI surface drift-detection (Plan 08-03)
provides:
  - api.rs CRUD + read functions for /absence-period and /vacation-balance
  - ShiftyError::Validation(String) variant for HTTP 422 self-overlap (D-11)
  - state::absence_period::AbsencePeriod + AbsenceCategory with side-join fields and From-TO conversions
  - state::vacation_balance::VacationBalance with From-TO conversion
  - loader.rs functions with HR-list cross-resolve (sales-person → name + color)
  - service::absence coroutine with ABSENCE_STORE, ABSENCE_REFRESH, ABSENCE_MODAL_EVENT side-channel and AbsenceAction enum
  - service::vacation_balance coroutine with VACATION_BALANCE_STORE, VACATION_TEAM_STORE and VacationBalanceAction enum
  - app.rs use_coroutine registrations for both services
  - 60 i18n Key-enum variants for absence-domain copy in src/i18n/mod.rs
  - 60 add_text calls in each of de.rs / en.rs / cs.rs (180 total)
  - 4 i18n tests (presence-in-all-locales + 3 reference matchers as Pitfall-2 guards)
  - Dioxus.toml proxy entries for /absence-period and /vacation-balance
affects: [08-05-page-modal-routing, future absence-domain UX]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Frontend State-with-Side-Join pattern for HR-list cross-resolve (mirrors loader::load_bookings)"
    - "Modal-Event Side-Channel via GlobalSignal<Option<Event>> (alternative to EventHandler-in-Action-Enum)"
    - "Defensive Uuid::nil at api-layer create-time (W-7 / Pitfall 9)"
    - "Per-locale reference-matcher tests as Pitfall-2 guards against Locale::En-instead-of-Locale::De drift"

key-files:
  created:
    - "shifty-dioxus/src/state/absence_period.rs (AbsencePeriod state + side-join fields)"
    - "shifty-dioxus/src/state/vacation_balance.rs (VacationBalance state)"
    - "shifty-dioxus/src/service/absence.rs (coroutine + 3 stores + AbsenceAction + AbsenceModalEvent)"
    - "shifty-dioxus/src/service/vacation_balance.rs (coroutine + 2 stores + VacationBalanceAction)"
  modified:
    - "shifty-dioxus/src/error.rs (added Validation(String) variant)"
    - "shifty-dioxus/src/state/mod.rs (pub mod absence_period + vacation_balance)"
    - "shifty-dioxus/src/api.rs (8 new functions for absence + vacation-balance)"
    - "shifty-dioxus/src/loader.rs (4 new loader functions)"
    - "shifty-dioxus/src/service/mod.rs (pub mod absence + vacation_balance)"
    - "shifty-dioxus/src/app.rs (2 use_coroutine registrations)"
    - "shifty-dioxus/src/i18n/mod.rs (60 Key variants + 4 tests)"
    - "shifty-dioxus/src/i18n/de.rs (60 add_text calls)"
    - "shifty-dioxus/src/i18n/en.rs (60 add_text calls)"
    - "shifty-dioxus/src/i18n/cs.rs (60 add_text calls)"
    - "shifty-dioxus/Dioxus.toml (2 proxy entries)"

key-decisions:
  - "ABSENCE_MODAL_EVENT side-channel chosen over EventHandler<Result<...>>-in-Action-Enum to keep AbsenceAction cheap to derive Debug; PATTERNS.md Z. 522-525 explicitly allows either"
  - "create_absence_period defensively zeroes id and version inside the function (W-7), independent of caller hygiene — backend rejects non-nil ids on POST with 422 IdSetOnCreate"
  - "Per-locale reference-matcher tests added as Pitfall-2 guards: i18n_absence_keys_match_{german,english,czech}_reference catch any future Locale::En-instead-of-De drift in the add_text calls"
  - "AbsenceCategory derives Copy + Hash for ergonomic use in HashMap keys / filter sets later"

patterns-established:
  - "Modal-Event side-channel pattern: service writes outcome to a GlobalSignal<Option<Event>>, page reads + acknowledges. Avoids threading EventHandler through Action-Enum payloads while still routing 409 / 422 / Forward-Warnings to modal-local UI rather than the global ERROR_STORE."
  - "Defensive id-zeroing at api-create-functions: function-body sets id and version to Uuid::nil() before send. Caller hygiene is no longer required for correctness."

requirements-completed: [FUI-A-01, FUI-A-02, FUI-A-03, FUI-A-04]

# Metrics
duration: ~85min
completed: 2026-05-08
---

# Phase 8 Plan 04: Frontend Foundation Summary

**Vollständige Dioxus-Frontend-Foundation für /absences: api/state/loader/coroutine-services/i18n/proxy — Plan 05 baut die Page darauf ohne Plumbing.**

## Performance

- **Duration:** ~85 min (incl. read-first phase + native cargo-test cycle through nix-shell)
- **Started:** 2026-05-08
- **Completed:** 2026-05-08
- **Tasks:** 6 (Task 1, Task 2, Task 3a, Task 3b, Task 3c, Task 3d)
- **Files modified:** 15 (4 new + 11 modified)

## Accomplishments

- API-Layer (8 neue async-Funktionen) mit Defensive Uuid::nil bei POST und 422-Validation/409-Conflict-Mapping (D-08, D-11).
- ShiftyError::Validation(String) Variant + error_handler-Branch.
- 2 neue State-Module (AbsencePeriod mit Side-Join + VacationBalance) mit From-TO + Tests.
- 4 neue Loader-Funktionen mit HR-Liste-Cross-Resolve (Mirror von load_bookings).
- 2 neue Service-Coroutines mit GlobalSignal-Stores + Action-Enums + Modal-Event-Side-Channel.
- app.rs Coroutine-Registrierung sichert dauerhaftes Service-Laufen.
- 60 i18n-Keys × 3 Locales = 180 add_text-Calls + 4 Tests (1 presence + 3 Pitfall-2-Guards).
- Dioxus.toml Proxy für /absence-period + /vacation-balance.
- WASM-Build (`cargo build --target wasm32-unknown-unknown`) bleibt grün; alle 492 Frontend-Tests grün.

## Task Commits

Each task was committed atomically (jj-managed, no git commit calls):

1. **Task 1: API-Layer + Error-Variant + State-Types + Loader** — `39c604a2` (feat)
2. **Task 2: Service-Coroutines (absence + vacation_balance) + app.rs-Registrierung** — `427bd5cd` (feat)
3. **Task 3a: Key-Enum-Variants + i18n-Test + Dx-Proxy** — `c5b947d5` (chore)
4. **Task 3b: Locale::De-Befüllung in de.rs** — `63807276` (feat)
5. **Task 3c: Locale::En-Befüllung in en.rs** — `9dcf3931` (feat)
6. **Task 3d: Locale::Cs-Befüllung in cs.rs + WASM-Build-Gate** — `4da8717d` (feat)

_Note: User commits manually via jj; per-plan metadata commit will be added by user when consolidating._

## Files Created/Modified

### Created (4)
- `shifty-dioxus/src/state/absence_period.rs` — `AbsencePeriod` (with side-join `person_name` / `background_color`), `AbsenceCategory` enum (Copy + Hash), bidirectional `From` impls for `AbsencePeriodTO` / `AbsenceCategoryTO`, 2 unit tests.
- `shifty-dioxus/src/state/vacation_balance.rs` — `VacationBalance` 1:1 mirror of `VacationBalanceTO` (read-only aggregate, no $version), 1 unit test.
- `shifty-dioxus/src/service/absence.rs` — `ABSENCE_STORE` (Rc<[AbsencePeriod]>), `ABSENCE_REFRESH` (u64 token), `ABSENCE_MODAL_EVENT` (Option<AbsenceModalEvent>), `bump_absence_refresh()`, `AbsenceAction` enum (LoadAll/LoadForSalesPerson/Create/Update/Delete/Refresh), `absence_service()` coroutine with 409/422/Forward-Warning routing, 2 unit tests.
- `shifty-dioxus/src/service/vacation_balance.rs` — `VACATION_BALANCE_STORE` (Self), `VACATION_TEAM_STORE` (Team), `VacationBalanceAction` enum (LoadSelf/LoadTeam), `vacation_balance_service()` coroutine.

### Modified (11)
- `shifty-dioxus/src/error.rs` — Added `ShiftyError::Validation(String)` variant + matching `error_handler` arm.
- `shifty-dioxus/src/state/mod.rs` — `pub mod absence_period; pub mod vacation_balance;`
- `shifty-dioxus/src/api.rs` — Added imports (`AbsencePeriodTO`, `AbsencePeriodCreateResultTO`, `VacationBalanceTO`) + 8 functions: `list_absence_periods`, `list_absence_periods_by_sales_person`, `get_absence_period`, `create_absence_period` (defensive Uuid::nil), `update_absence_period`, `delete_absence_period`, `get_vacation_balance`, `get_team_vacation_balance`. 422 → `ShiftyError::Validation`, 409 → `ShiftyError::Conflict`.
- `shifty-dioxus/src/loader.rs` — Added 4 functions: `load_absence_periods_all` (with sales-person side-join), `load_absence_periods_by_sales_person`, `load_vacation_balance`, `load_team_vacation`.
- `shifty-dioxus/src/service/mod.rs` — `pub mod absence; pub mod vacation_balance;`
- `shifty-dioxus/src/app.rs` — `use_coroutine(service::absence::absence_service); use_coroutine(service::vacation_balance::vacation_balance_service);`
- `shifty-dioxus/src/i18n/mod.rs` — 60 `Key` enum variants under `// Absence management (Phase 8)` + 4 tests (`i18n_absence_keys_present_in_all_locales`, `i18n_absence_keys_match_{german,english,czech}_reference`).
- `shifty-dioxus/src/i18n/de.rs` — 60 `add_text(Locale::De, …)` calls.
- `shifty-dioxus/src/i18n/en.rs` — 60 `add_text(Locale::En, …)` calls.
- `shifty-dioxus/src/i18n/cs.rs` — 60 `add_text(Locale::Cs, …)` calls.
- `shifty-dioxus/Dioxus.toml` — 2 proxy entries (`/absence-period`, `/vacation-balance`).

## Decisions Made

1. **`ABSENCE_MODAL_EVENT` Side-Channel statt EventHandler im Action-Enum.** Plan 04 Task 2 nennt explizit beide Optionen (PATTERNS.md Z. 522-525). Der Side-Channel-Ansatz hält `AbsenceAction` debug-able, vermeidet Lifetime-Probleme mit `EventHandler<Result<...>>` und ermöglicht der Page eine reaktive Beobachtung via `use_memo` / `use_effect` ohne die bei jedem Modal-Render neu allozierten EventHandler durch das Message-Bus-Layer zu schieben.

2. **Defensive Uuid::nil im api::create_absence_period-Body.** W-7 in `must_haves.truths` verlangt das. Das verhindert, dass ein Edit-Modal beim Mode-Switch von Edit zu Create vergisst, die `id` zu nullen, und der Backend dann 422 IdSetOnCreate liefert. Die Funktion ist jetzt selbstkonsistent — Caller-Hygiene ist nicht mehr Korrektheits-Voraussetzung.

3. **AbsenceCategory derives Copy + Hash.** Erleichtert spätere Filter-Implementierung in Plan 05 (HashMap-Keys, Filter-Sets ohne `.clone()`).

4. **Per-Locale-Reference-Matcher-Tests gegen Pitfall 2.** Über den `i18n_absence_keys_present_in_all_locales`-Test hinaus drei zusätzliche Tests (`match_{german,english,czech}_reference`), die je 4-5 Stichproben mit dem erwarteten Original-String prüfen. Das fängt z.B. ein versehentliches `add_text(Locale::En, …)` in `de.rs` (was historisch in v1.0 passiert ist).

## Deviations from Plan

None — plan executed exactly as written. All acceptance criteria met:

- All 6 tasks committed individually via jj.
- All 60 i18n keys present in all 3 locales (cargo test green).
- WASM build (`cargo build --target wasm32-unknown-unknown`) green.
- Backend `cargo check --workspace` green (no regression from rest-types-touchpoints — actually no rest-types changes in Plan 04).
- Pitfall-2 guard (no foreign Locale-tag in any locale-specific block) holds: 0 hits in De/En/Cs blocks.

The plan-suggested split of an "i18n mega-task" into 3a/3b/3c/3d (one per locale) worked exactly as designed: each sub-task is ~5-10 min implementation, the per-locale reference-matcher test is the GREEN gate, and the pitfall-2 risk is fully covered by the 4 i18n tests.

## Issues Encountered

1. **NixOS `nix develop` shell-eval failure.** As documented in `feedback_no_unauthorized_install.md` and `reference_local_dev_commands.md`, `nix develop` is broken at the moment (`nodePackages has been removed` upstream churn in nixpkgs). Workaround: cargo runs were either (a) `cargo check --target wasm32-unknown-unknown` (no openssl needed) or (b) `nix-shell -p openssl pkg-config --command "..."` for native cargo test, and (c) `nix-shell -p lld --command "..."` for the final WASM build gate (the wasm-target needs `lld`). All 3 modes were used at the appropriate gate; the broken `nix develop` was avoided.

2. **`cargo test --target wasm32-unknown-unknown` is incompatible with default-test-runner.** Mio refuses to build for wasm without explicit feature gating. Resolution: tests were run on the native target (which works once openssl is in the nix-shell), and the WASM gate is `cargo build --target wasm32-unknown-unknown` (not test). The Plan-02 verification command was understood and obeyed.

## User Setup Required

None — no external service configuration required. All work is internal to the frontend crate.

## Next Phase Readiness

**Plan 08-05** (Frontend Page + Modal + Routing + TopBar) can now be built without touching api/state/loader/service/i18n/proxy layers:

- `service::absence::AbsenceAction` enum is ready to receive `LoadAll(sales_persons)` / `LoadForSalesPerson(uuid)` / `Create(to)` / `Update(to)` / `Delete(uuid)` / `Refresh` from the page.
- `service::vacation_balance::VacationBalanceAction` is ready to receive `LoadSelf(sp_id, year)` / `LoadTeam(year)`.
- `ABSENCE_MODAL_EVENT` is the page-side modal-local event source for the version-conflict banner (D-08), self-overlap banner (D-11), and forward-warning list (D-12).
- `i18n.t(Key::AbsenceFoo)` works for all 60 keys in all 3 locales.
- `Dioxus.toml` proxies are wired for both backend resources.

No blockers. Plan 05 can route directly to `Route::Absences` once the router enum is extended (Plan 05 Task 1).

## Self-Check: PASSED

**Files verified to exist:**
- `shifty-dioxus/src/state/absence_period.rs` ✓
- `shifty-dioxus/src/state/vacation_balance.rs` ✓
- `shifty-dioxus/src/service/absence.rs` ✓
- `shifty-dioxus/src/service/vacation_balance.rs` ✓

**Commits verified to exist (jj log):**
- `39c604a2` Task 1 ✓
- `427bd5cd` Task 2 ✓
- `c5b947d5` Task 3a ✓
- `63807276` Task 3b ✓
- `9dcf3931` Task 3c ✓
- `4da8717d` Task 3d ✓

**Verification commands re-run during self-check:**
- `cargo build --target wasm32-unknown-unknown` (with `nix-shell -p lld`) — green
- `cargo test` (with `nix-shell -p openssl pkg-config`) — 492 tests green, including all 4 absence-i18n tests
- `awk` Pitfall-2 guard on all 3 locale files — 0 foreign Locale-tags in any block

---
*Phase: 08-absence-crud-page-foundation*
*Completed: 2026-05-08*
