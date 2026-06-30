---
phase: 22-mitarbeiter-statistik-hr
plan: 02
subsystem: ui
tags: [dioxus, rust, wasm, i18n, hr, statistics, employee-view]

requires:
  - phase: 22-01
    provides: "GET /report/{id}/weekly-statistics backend endpoint returning EmployeeWeeklyStatisticsTO"

provides:
  - "HR-only statistics block in EmployeeView (/employees/:id) — visible with HR role, hidden without"
  - "api::get_employee_weekly_statistics fetch function hitting /report/{id}/weekly-statistics"
  - "EmployeeStore.weekly_statistics field (None for non-HR, Some for HR users)"
  - "should_show_hr_stats pure helper function with full 2×2 test coverage"
  - "i18n keys StatisticsHeading / AverageWorkedHoursPerWeek / StatisticsIncludedWeeks in De/En/Cs"
  - "SSR visibility tests: block present with is_hr=true+stats, absent with is_hr=false"

affects: [employee-detail, hr-visibility, statistics, phase-22]

tech-stack:
  added: []
  patterns:
    - "HR-only UI gating: AUTH.read().auth_info.as_ref().map(|a| a.has_privilege(\"hr\")).unwrap_or(false)"
    - "Defence-in-depth: backend 403s non-HR fetch → store gets None → block hidden client-side too"
    - "SSR isolation for non-wasm-safe component tests via minimal stub components"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/service/employee.rs
    - shifty-dioxus/src/component/employee_view.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/cs.rs

key-decisions:
  - "Stats fetch (get_employee_weekly_statistics) uses .ok() → None for non-HR (403) or network errors, so block is always hidden without real HR data"
  - "weekly_statistics stored on EmployeeStore (not Employee), avoiding changes to the report-to-Employee From impl"
  - "SSR tests use minimal stub components to avoid js::get_current_year()/week() panic on non-wasm — same pattern as delete-contract tests"
  - "should_show_hr_stats is a pure pub(crate) fn: is_hr && stats.is_some() — 4 unit tests cover all combinations"

requirements-completed: [STAT-01, STAT-02]

duration: 25min
completed: 2026-06-26
---

# Phase 22 Plan 02: HR Statistics Block in EmployeeView Summary

**HR-only average-worked-hours-per-week block wired into the employee detail page, fetched from the Phase-22 backend endpoint, gated by is_hr + should_show_hr_stats, with i18n in De/En/Cs and SSR visibility tests.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-06-26T00:00:00Z
- **Completed:** 2026-06-26
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Added `api::get_employee_weekly_statistics` fetch fn mirroring get_employee_reports, 403 → Err → .ok() = None
- Added `weekly_statistics: Option<Rc<EmployeeWeeklyStatisticsTO>>` to `EmployeeStore`; load_employee_data fetches it after the main report, swallows errors (non-HR gets None)
- Added three i18n keys (StatisticsHeading, AverageWorkedHoursPerWeek, StatisticsIncludedWeeks) in all three locales (En/De/Cs)
- Added `should_show_hr_stats(is_hr, stats)` pure helper and the HR-only block in `EmployeeViewPlain` rsx, gated by that helper
- `EmployeeView` wrapper reads AUTH signal for `is_hr` and EMPLOYEE_STORE for `weekly_statistics`, passes both as new props to `EmployeeViewPlain`
- SSR visibility tests: 4 pure-fn tests for should_show_hr_stats covering all 2×2 combos; 3 SSR tests (visible with HR+stats, hidden without HR, hidden without stats)
- All 656 cargo tests pass; WASM build exits 0

## Task Commits

No commits were made (per hard_rules: DO NOT run git or jj; the user controls commits via jj).

## Files Created/Modified
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/api.rs` — Added `EmployeeWeeklyStatisticsTO` import + `get_employee_weekly_statistics` async fn
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/service/employee.rs` — Added `weekly_statistics` field to `EmployeeStore`; fetch in `load_employee_data`; import `EmployeeWeeklyStatisticsTO`
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/component/employee_view.rs` — `should_show_hr_stats` helper; new `is_hr`/`weekly_statistics` props on `EmployeeViewPlainProps`; HR-only block in rsx; AUTH read in `EmployeeView`; 7 new tests
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/i18n/mod.rs` — 3 new Key variants (StatisticsHeading, AverageWorkedHoursPerWeek, StatisticsIncludedWeeks)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/i18n/en.rs` — English translations
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/i18n/de.rs` — German translations
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/i18n/cs.rs` — Czech translations

## Decisions Made
- Used `.ok()` on the stats fetch so any network error or 403 silently stores None — block stays hidden without extra error handling
- Stored `weekly_statistics` on `EmployeeStore` (not inside `Employee`) to avoid touching the `EmployeeReportTO → Employee` From impl
- SSR tests use isolated stub components (not EmployeeViewPlain directly) since `js::get_current_year()` / `js::get_current_week()` panic outside WASM

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None.

## Known Stubs
None. The HR statistics block is fully wired: fetch → store → component → display.

## Threat Flags
None — no new network endpoints or auth paths beyond what the plan's threat model already covers (T-22-04, T-22-05).

## Next Phase Readiness
- STAT-01 and STAT-02 are complete on the frontend
- Phase 22 Wave 2 is done; all requirements fulfilled
- The HR-gated statistics block is live on /employees/:id

---
*Phase: 22-mitarbeiter-statistik-hr*
*Completed: 2026-06-26*

## Self-Check: PASSED
- All modified files verified via cargo build (Finished dev profile) and cargo test (656 passed; 0 failed)
- WASM gate: `nix develop --command cargo build --target wasm32-unknown-unknown` exited 0
- i18n keys present in all 3 locales: grep -c "AverageWorkedHoursPerWeek" en.rs de.rs cs.rs → each = 1
- should_show_hr_stats and has_privilege("hr") present in employee_view.rs
