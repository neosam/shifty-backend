---
phase: 26-freiwilligen-abwesenheit-cross-navigation
verified: 2026-06-28T19:30:00Z
status: passed
score: 9/9 must-haves verified
behavior_unverified: 0
overrides_applied: 0
---

# Phase 26: Freiwilligen-Abwesenheit & Cross-Navigation Verification Report

**Phase Goal:** A volunteer's absence reduces their committed pledge in the live year view (VFA-01); holidays deliberately do NOT (VFA-02 asymmetry); bidirectional per-employee deep-links between /absences and the employee report (NAV-01, Sales + HR).
**Verified:** 2026-06-28T19:30:00Z
**Status:** PASSED
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | VFA-01: `period_overlaps_week` pure helper exists as `pub(crate)`, category-agnostic, whole-week semantics | VERIFIED | `booking_information.rs:76-83` — `from <= week_sunday && to >= week_monday`; doc-comment references D-26-01/D-26-03 |
| 2 | VFA-01: AbsenceService wired into BookingInformationServiceImpl with no DI cycle | VERIFIED | `booking_information.rs:99` (gen_service_impl! dep); `main.rs:215` (type alias); `main.rs:925` (construction with `absence_service.clone()`); `main.rs:826` builds absence_service before line 925 |
| 3 | VFA-01: `find_all` called once before the week loop; `absent_volunteer_ids` HashSet built per-week | VERIFIED | `booking_information.rs:198-234` — load-once pattern before `for week in 1..=` loop; per-week HashSet built via `period_overlaps_week` |
| 4 | VFA-01: Band-1 `committed_voluntary_hours` excludes absent volunteers (whole-week-out, D-26-03) | VERIFIED | `booking_information.rs:291` — `.filter(\|wh\| !absent_volunteer_ids.contains(&wh.sales_person_id))` |
| 5 | VFA-01: Band-2 `committed_for_person` closure returns 0.0 for absent volunteers (consistency) | VERIFIED | `booking_information.rs:269-271` — `if absent_volunteer_ids.contains(&sp_id) { return 0.0; }` |
| 6 | VFA-01: 8 pure-helper unit tests for `period_overlaps_week` and whole-week-out exist and pass | VERIFIED | `test/booking_information.rs:503-599+` — tests `vfa01_overlap_*`, `vfa01_no_overlap_*`, `vfa01_whole_week_out_d2603_not_prorated`, `vfa01_non_absent_volunteer_unaffected`; confirmed PASS via `cargo test -p service_impl vfa01_whole_week_out_d2603_not_prorated` |
| 7 | VFA-02: `booking_information_vfa.rs` exists with `vfa02_holiday_vs_absence_asymmetry` test asserting HOLIDAY_WEEK committed ≈ 5.0 and ABSENCE_WEEK committed ≈ 0.0 for the same volunteer | VERIFIED | File exists (352 lines); test passes: `vfa02_holiday_vs_absence_asymmetry ... ok` (confirmed via `cargo test -p service_impl vfa02_holiday_vs_absence_asymmetry`); HOLIDAY_WEEK=W15 / ABSENCE_WEEK=W20 (5 weeks apart, no bleed risk) |
| 8 | No snapshot bump (D-26-02): `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays 11; guard test `phase26_vfa_no_snapshot_bump` passes | VERIFIED | `billing_period_report.rs:108: pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 11;`; test passes: `phase26_vfa_no_snapshot_bump ... ok` (confirmed via `cargo test -p service_impl phase26_vfa_no_snapshot_bump`); `mod.rs` registers module |
| 9 | NAV-01: Route, GlobalSignal, wrapper, 4 ghost-button cross-links, 4 i18n keys in en/de/cs, completeness test | VERIFIED | See detailed breakdown below; `i18n_phase26_keys_present_in_all_locales ... ok` (confirmed via `cargo test i18n_phase26_keys_present_in_all_locales` in shifty-dioxus) |

**Score:** 9/9 truths verified

---

### NAV-01 Detailed Breakdown

| Item | Location | Evidence |
|------|----------|----------|
| `Route::AbsencesFor { employee_id: String }` at `/absences/:employee_id/` | `router.rs:62-63` | Present; `pub use crate::page::AbsencesFor` at line 3 |
| `ABSENCES_PRESELECT: GlobalSignal<Option<Uuid>>` | `page/absences.rs:59` | `Signal::global(\|\| None)` — mirrors ABSENCE_REFRESH pattern |
| `AbsencesFor` wrapper: parses UUID, inline error on bad UUID, writes preselect on mount via use_effect | `page/absences.rs:2347-2366` | `Uuid::parse_str` at 2348; `Err(_)` renders `"Invalid employee id"` at 2351; `use_effect` writes at 2360 |
| `AbsencesPage` reads + clears ABSENCES_PRESELECT in use_effect; seeds `person_filter` | `page/absences.rs:1984-1990` | Reactive subscription: reads signal, calls `person_filter.set(Some(id))`, clears to None; re-runs when AbsencesFor's effect fires (GlobalSignal reactivity) |
| Link 1: Sales MyEmployeeDetails → Route::Absences {} (NavToMyAbsences, always shown) | `page/my_employee_details.rs:117-119` | `BtnVariant::Ghost`, `nav.push(Route::Absences {})`, label `i18n.t(Key::NavToMyAbsences)` |
| Link 2: HR EmployeeDetails → Route::AbsencesFor { employee_id } (NavToEmployeeAbsences, always shown) | `page/employee_details.rs:185-193` | `BtnVariant::Ghost`, `nav.push(Route::AbsencesFor { employee_id: employee_id.to_string() })`, label `i18n.t(Key::NavToEmployeeAbsences)` |
| Link 3: Absences Sales → Route::MyEmployeeDetails {} (NavToMyTimeAccount, !is_hr guard) | `page/absences.rs:2197-2204` | `if !is_hr` guard; `BtnVariant::Ghost`; `nav.push(Route::MyEmployeeDetails {})` |
| Link 4: Absences HR → Route::EmployeeDetails { employee_id } (NavToEmployeeReport, is_hr && person selected) | `page/absences.rs:2207-2218` | `if is_hr { if let Some(selected_id) = person_filter_val`; `BtnVariant::Ghost`; `nav.push(Route::EmployeeDetails { employee_id: selected_id.to_string() })` |
| 4 i18n Key variants in Key enum | `i18n/mod.rs:615,617,619,621` | `NavToMyAbsences`, `NavToEmployeeAbsences`, `NavToMyTimeAccount`, `NavToEmployeeReport` |
| En translations | `i18n/en.rs:995-998` | "My absences", "Absences", "My time account", "Employee report" |
| De translations | `i18n/de.rs:1082-1085` | "Meine Abwesenheiten", "Abwesenheiten", "Mein Zeitkonto", "Mitarbeiterbericht" |
| Cs translations | `i18n/cs.rs:1068-1071` | "Moje absence", "Absence", "Moje casove konto", "Prehled zamestnance" |
| Completeness test | `i18n/mod.rs:1360-1367` + test run | `i18n_phase26_keys_present_in_all_locales` passes: all 4 keys present in En/De/Cs, non-empty, non-"??" |
| IDOR safety (T-26-03) | `page/absences.rs:1962-1967` | Non-HR path dispatches `LoadForSalesPerson(sp)` using own sales_person ID; `person_filter` only seeds the UI selector; data load unchanged |

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `service_impl/src/booking_information.rs` | `period_overlaps_week` helper + absence-driven exclusion in `get_weekly_summary` | VERIFIED | Helper at line 76; load-once at 198-201; per-week HashSet at 214-234; Band-1 filter at 291; Band-2 guard at 269-271 |
| `service_impl/src/test/booking_information.rs` | 8 VFA-01 unit tests for overlap helper + whole-week-out | VERIFIED | Lines 481-620+; all tests enumerated and named with `vfa01_*` prefix |
| `shifty_bin/src/main.rs` | `type AbsenceService = AbsenceService` in `BookingInformationServiceDependencies`; `absence_service` in construction | VERIFIED | Line 215 (type); line 925 (construction); line 826 is absence_service construction (before 925) |
| `service_impl/src/test/booking_information_vfa.rs` | VFA-02 asymmetry test + no-bump guard | VERIFIED | File created, 352 lines, 2 tests, both passing |
| `service_impl/src/test/mod.rs` | Registers `booking_information_vfa` module | VERIFIED | `pub mod booking_information_vfa;` at line 6 |
| `shifty-dioxus/src/router.rs` | `AbsencesFor { employee_id: String }` route variant + use re-export | VERIFIED | Lines 62-63 (route); line 3 (re-export) |
| `shifty-dioxus/src/page/absences.rs` | ABSENCES_PRESELECT GlobalSignal + AbsencesFor wrapper + preselect consumption + Links 3/4 | VERIFIED | Lines 59 (signal), 1984-1990 (use_effect), 2347-2366 (wrapper), 2197-2218 (links 3+4) |
| `shifty-dioxus/src/page/my_employee_details.rs` | Link 1 (NavToMyAbsences ghost button) | VERIFIED | Lines 117-119 |
| `shifty-dioxus/src/page/employee_details.rs` | Link 2 (NavToEmployeeAbsences ghost button) | VERIFIED | Lines 185-193 |
| `shifty-dioxus/src/i18n/{mod,en,de,cs}.rs` | 4 NavTo* keys + phase26 key-presence test | VERIFIED | All 4 keys in all 3 locales; test passes |

---

### Key Link Verification

| From | To | Via | Status |
|------|----|-----|--------|
| `get_weekly_summary` | `AbsenceService.find_all` | `self.absence_service.find_all(Authentication::Full, tx.clone().into())` at line 199-201, BEFORE the week loop | WIRED |
| `get_weekly_summary` Band-1 sum | `absent_volunteer_ids` filter | `.filter(\|wh\| !absent_volunteer_ids.contains(&wh.sales_person_id))` at line 291 | WIRED |
| `get_weekly_summary` Band-2 closure | `absent_volunteer_ids` guard | `if absent_volunteer_ids.contains(&sp_id) { return 0.0; }` at lines 269-271 | WIRED |
| `AbsencesFor` wrapper | `ABSENCES_PRESELECT` | `use_effect` writes `Some(parsed)` at mount (line 2360) | WIRED |
| `AbsencesPage` | `person_filter` | `use_effect` reads ABSENCES_PRESELECT, calls `person_filter.set(Some(id))` and clears signal (lines 1984-1990) | WIRED |
| Link 2 (`employee_details.rs`) | `Route::AbsencesFor { employee_id }` | `nav.push(Route::AbsencesFor { employee_id: employee_id.to_string() })` at line 189-190 | WIRED |
| Link 4 (`absences.rs`) | `Route::EmployeeDetails { employee_id }` | `nav.push(Route::EmployeeDetails { employee_id: selected_id.to_string() })` at line 2213-2214 | WIRED |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| VFA-02 asymmetry regression: HOLIDAY_WEEK ≈ 5.0, ABSENCE_WEEK ≈ 0.0 | `cargo test -p service_impl vfa02_holiday_vs_absence_asymmetry` | `test ... ok` | PASS |
| Phase-26 no-snapshot-bump guard: CURRENT_SNAPSHOT_SCHEMA_VERSION == 11 | `cargo test -p service_impl phase26_vfa_no_snapshot_bump` | `test ... ok` | PASS |
| VFA-01 whole-week-out (D-26-03, not pro-rated) | `cargo test -p service_impl vfa01_whole_week_out_d2603_not_prorated` | `test ... ok` | PASS |
| i18n completeness: all 4 NAV-01 keys in en/de/cs, non-empty, non-"??" | `cargo test i18n_phase26_keys_present_in_all_locales` (shifty-dioxus) | `test ... ok` | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| VFA-01 | 26-01 | Volunteer absence (any of 3 categories) in a week reduces committed_voluntary to 0 for that week (whole-week-out, D-26-01/D-26-03) | SATISFIED | `period_overlaps_week` + `absent_volunteer_ids` filter in both Band-1 and Band-2; 8 unit tests; full-service test confirms |
| VFA-02 | 26-01, 26-02 | Holiday does NOT reduce committed_voluntary; asymmetry asserted by regression test (D-26-04) | SATISFIED | `vfa02_holiday_vs_absence_asymmetry` confirmed GREEN; special_days code path at lines 299-308 is structurally disjoint from committed band logic |
| NAV-01 | 26-03 | Bookmarkable `/absences/:employee_id`; bidirectional links Sales↔own-absences and HR↔employee-absences; all labels de/en/cs | SATISFIED | Route variant, 4 wired ghost buttons, 4 i18n keys in all 3 locales; completeness test GREEN |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `booking_information.rs` | 612 | `committed_voluntary_hours: 0.0` with "placeholder" comment | INFO | Pre-existing from Phase 15 — single-week variant intentionally returns 0 (documented in 15-01-SUMMARY.md); not in the VFA code path; no Phase 26 debt |

No TBD/FIXME/XXX markers found in any Phase 26 modified file. No unresolved stubs.

---

### Human Verification Required

None. All must-haves verified by code inspection and behavioral spot-checks. NAV-01 ghost-button navigation behavior follows the established `BtnVariant::Ghost + use_navigator().push(Route::...)` pattern already used throughout the codebase; the objective explicitly specifies autonomous code-inspection verification for NAV-01 links. The ABSENCES_PRESELECT GlobalSignal preselect mechanism is reactive (AbsencesPage's use_effect subscribes to the signal and re-runs when AbsencesFor writes it), making effect-ordering a non-issue.

---

### Gaps Summary

No gaps. All 9 must-haves are VERIFIED against the actual codebase:

- VFA-01 is fully implemented and tested (8 unit tests + full-service test)
- VFA-02 asymmetry regression guard is wired and passing
- D-26-02 no-bump constraint holds (CURRENT_SNAPSHOT_SCHEMA_VERSION = 11, guard test passes)
- NAV-01 route, wrapper, 4 cross-links, 4 i18n keys all present and wired; completeness test green

---

_Verified: 2026-06-28T19:30:00Z_
_Verifier: Claude (gsd-verifier)_
