---
phase: 26-freiwilligen-abwesenheit-cross-navigation
plan: "03"
subsystem: ui
tags: [dioxus, routing, i18n, navigation, wasm]

requires:
  - phase: 26-freiwilligen-abwesenheit-cross-navigation
    provides: "Phase 26 context — absence and employee detail pages to link"

provides:
  - "Bookmarkable /absences/:employee_id/ route (D-26-05) with ABSENCES_PRESELECT GlobalSignal pre-filling person selector"
  - "AbsencesFor wrapper component (parse UUID, seed preselect on mount, render AbsencesPage)"
  - "4 bidirectional ghost-button cross-links (D-26-06) — Sales: MyEmployeeDetails ↔ Absences; HR: EmployeeDetails(:id) ↔ AbsencesFor(:id)"
  - "4 NAV-01 i18n keys (NavToMyAbsences/NavToEmployeeAbsences/NavToMyTimeAccount/NavToEmployeeReport) in en/de/cs"

affects:
  - "absences page (preselect + person-filter seeding)"
  - "employee_details page (HR → absences cross-link)"
  - "my_employee_details page (Sales → own absences cross-link)"

tech-stack:
  added: []
  patterns:
    - "GlobalSignal preselect pattern: wrapper writes on mount, page component reads+clears via use_effect"
    - "Ghost-button imperative navigation: BtnVariant::Ghost + use_navigator().push(Route::...)"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/router.rs
    - shifty-dioxus/src/page/mod.rs
    - shifty-dioxus/src/page/absences.rs
    - shifty-dioxus/src/page/employee_details.rs
    - shifty-dioxus/src/page/my_employee_details.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/cs.rs

key-decisions:
  - "D-26-05: ABSENCES_PRESELECT GlobalSignal used (vs. prop threading) — avoids restructuring AbsencesPage component interface, mirrors ABSENCE_REFRESH pattern"
  - "D-26-05: use_effect in AbsencesPage reads + clears ABSENCES_PRESELECT so a later param-less /absences visit is not sticky"
  - "T-26-03 IDOR-safe: route param only seeds UI selector; non-HR data load (LoadForSalesPerson) unchanged — backend enforces HR ∨ self on every fetch"
  - "T-26-04 UUID guard: AbsencesFor renders inline error on parse failure without seeding preselect"

requirements-completed: [NAV-01]

coverage:
  - id: D1
    description: "Bookmarkable /absences/:employee_id/ route with AbsencesFor wrapper; param-less /absences/ unchanged"
    requirement: NAV-01
    verification:
      - kind: unit
        ref: "cargo build --target wasm32-unknown-unknown (shifty-dioxus) — route compiles"
        status: pass
    human_judgment: false
  - id: D2
    description: "ABSENCES_PRESELECT GlobalSignal seeds person_filter in AbsencesPage on mount; cleared after use"
    requirement: NAV-01
    verification:
      - kind: unit
        ref: "cargo build --target wasm32-unknown-unknown (shifty-dioxus) — compiles without error"
        status: pass
    human_judgment: true
    rationale: "Preselect signal flow is runtime behavior (use_effect → signal write → re-fire) that requires browser verification to confirm person selector is pre-filled"
  - id: D3
    description: "4 NAV-01 i18n keys (NavToMyAbsences/NavToEmployeeAbsences/NavToMyTimeAccount/NavToEmployeeReport) in en/de/cs"
    requirement: NAV-01
    verification:
      - kind: unit
        ref: "i18n::tests::i18n_phase26_keys_present_in_all_locales — 1 passed"
        status: pass
    human_judgment: false
  - id: D4
    description: "4 ghost-button cross-links — Link 1 (MyEmployeeDetails→Absences), Link 2 (EmployeeDetails→AbsencesFor), Link 3 (Absences Sales→MyEmployeeDetails), Link 4 (Absences HR→EmployeeDetails)"
    requirement: NAV-01
    verification:
      - kind: unit
        ref: "cargo build --target wasm32-unknown-unknown (shifty-dioxus) — all 4 links compile"
        status: pass
    human_judgment: true
    rationale: "Navigation behavior (click → route transition) requires browser E2E to verify correct destination"

duration: 12min
completed: 2026-06-28
status: complete
---

# Phase 26 Plan 03: NAV-01 Cross-Navigation Summary

**Bidirectional per-employee deep-links between Absences and Employee report via bookmarkable /absences/:employee_id route, ABSENCES_PRESELECT GlobalSignal, 4 ghost-button cross-links, and 4 i18n keys in en/de/cs**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-06-28T~T16:45Z
- **Completed:** 2026-06-28
- **Tasks:** 3 of 3
- **Files modified:** 9

## Accomplishments

- New `Route::AbsencesFor { employee_id: String }` at `/absences/:employee_id/` — existing `/absences/` unchanged
- `ABSENCES_PRESELECT: GlobalSignal<Option<Uuid>>` seeds `person_filter` in `AbsencesPage` on mount; cleared afterward to prevent stickiness
- `AbsencesFor` wrapper: parses UUID (inline error on bad UUID per T-26-04), writes preselect on mount, renders `AbsencesPage`
- Link 1: `MyEmployeeDetails` → `Route::Absences {}` (Sales, always shown; label NavToMyAbsences)
- Link 2: `EmployeeDetails(:id)` → `Route::AbsencesFor { employee_id }` (HR, always shown; label NavToEmployeeAbsences)
- Link 3: `AbsencesPage` → `Route::MyEmployeeDetails {}` (shown when `!is_hr`; label NavToMyTimeAccount)
- Link 4: `AbsencesPage` → `Route::EmployeeDetails { employee_id: selected_id }` (shown when `is_hr && person_filter_val.is_some()`; label NavToEmployeeReport)
- 4 i18n keys in en/de/cs; `i18n_phase26_keys_present_in_all_locales` test passes

## Task Commits

1. **Task 1: /absences/:employee_id route + ABSENCES_PRESELECT + AbsencesFor wrapper** — `32e60df` (feat)
2. **Task 2: 4 NAV-01 i18n keys + phase26 key-presence test** — `eecb678` (feat)
3. **Task 3: 4 bidirectional ghost-button cross-links** — `faf1f9e` (feat)

## Files Created/Modified

- `shifty-dioxus/src/router.rs` — added `Route::AbsencesFor { employee_id: String }` + `pub use crate::page::AbsencesFor`
- `shifty-dioxus/src/page/mod.rs` — added `pub use absences::AbsencesFor`
- `shifty-dioxus/src/page/absences.rs` — ABSENCES_PRESELECT GlobalSignal, preselect use_effect, AbsencesFor wrapper, nav variable, Links 3+4
- `shifty-dioxus/src/page/employee_details.rs` — Link 2 (HR → AbsencesFor(:id))
- `shifty-dioxus/src/page/my_employee_details.rs` — added Btn/BtnVariant/Route/I18N/Key imports + nav/i18n variables + Link 1
- `shifty-dioxus/src/i18n/mod.rs` — 4 NavTo* Key enum variants + i18n_phase26_keys_present_in_all_locales test
- `shifty-dioxus/src/i18n/en.rs` — English translations for 4 NAV-01 keys
- `shifty-dioxus/src/i18n/de.rs` — German translations for 4 NAV-01 keys
- `shifty-dioxus/src/i18n/cs.rs` — Czech translations for 4 NAV-01 keys

## Decisions Made

- Used `ABSENCES_PRESELECT` GlobalSignal (mirrors `ABSENCE_REFRESH` pattern) instead of prop threading — avoids restructuring AbsencesPage's component interface.
- `AbsencesFor` writes preselect in `use_effect` (not synchronously) per plan spec; `AbsencesPage` subscribes to the signal so it re-fires when the value arrives.
- ABSENCES_PRESELECT cleared after consumption so navigating to `/absences/` later is not sticky.
- Link 4 (HR → EmployeeDetails) absent when no person is selected — no disabled/placeholder state, consistent with "link only meaningful when context is clear".

## Deviations from Plan

None — plan executed exactly as written. The `crate::router::Route` import was added to absences.rs (necessary for Links 3+4) as specified by the plan.

## Security Notes

T-26-03 (IDOR) mitigated: `AbsencesFor` only seeds the UI person selector. `AbsencesPage`'s data-load `use_effect` dispatches `LoadForSalesPerson(own_sp)` for non-HR users regardless of `person_filter` — a non-HR user deep-linking `/absences/:other_id` sees their own data, not the other person's.

T-26-04 (malformed UUID) mitigated: `AbsencesFor` calls `Uuid::parse_str`; on failure renders inline `"Invalid employee id"` and does not seed `ABSENCES_PRESELECT`.

## Verification Results

- `cargo build --target wasm32-unknown-unknown` (shifty-dioxus): **PASS** (47 warnings, 0 errors)
- `cargo test` (shifty-dioxus): **PASS** — 671 passed, 0 failed
- `i18n_phase26_keys_present_in_all_locales`: **PASS** (En/De/Cs all keys non-empty, non-"??")

## Known Stubs

None.

## Threat Flags

No new security-relevant surface beyond what is documented in the plan threat model (T-26-03, T-26-04).

## Next Phase Readiness

NAV-01 is fully implemented. Phase 26 Plans 01–03 are complete. The cross-navigation links are ready for browser E2E verification.

## Self-Check: PASSED

- Commits exist: 32e60df, eecb678, faf1f9e — confirmed via `git log`
- Files modified: router.rs, page/mod.rs, absences.rs, employee_details.rs, my_employee_details.rs, i18n/{mod,en,de,cs}.rs
- WASM build green, 671 tests pass

---
*Phase: 26-freiwilligen-abwesenheit-cross-navigation*
*Completed: 2026-06-28*
