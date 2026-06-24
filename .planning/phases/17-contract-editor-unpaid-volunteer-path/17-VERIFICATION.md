---
phase: 17-contract-editor-unpaid-volunteer-path
verified: 2026-06-24T16:00:00Z
status: passed
human_verification_result: passed (live Chrome browser test 2026-06-24 — all 3 items confirmed; see 17-HUMAN-UAT.md)
score: 8/8 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Open the contract editor for a paid employee with cap_planned_hours_to_expected=true. Verify the 'Freiwillige Zusage (h)' numeric input field is visible and accepts decimal values. Save and reopen — confirm the value is preserved."
    expected: "Field visible, editable, and value round-trips faithfully without silent reset to 0."
    why_human: "SSR tests verify conditional rendering logic but not the full live reactive dispatch cycle through the Dioxus coroutine/store pipeline in a running browser."
  - test: "Open the contract editor for an employee with cap=false and expected_hours=0.0. Verify 'Freiwillige Zusage (h)' field is visible."
    expected: "Field appears for rein-freiwillig path (D-01 second branch)."
    why_human: "Browser runtime needed to observe conditional signal evaluation in live rendering."
  - test: "In Mitarbeiteransicht: default view shows only paid employees. Click the 'alle' toggle. Verify that unpaid non-inactive volunteers (is_paid=false) become visible in the list."
    expected: "show_all=false: no unpaid persons. show_all=true: unpaid non-inactive persons appear. Inactive persons remain hidden regardless."
    why_human: "The show_all toggle is a browser-side Dioxus signal — its live behavior in a running app cannot be verified via unit tests alone."
---

# Phase 17: Contract editor + alle-Filter / unpaid-volunteer path Verification Report

**Phase Goal:** `committed_voluntary` als numerisches Feld im Vertrags-Editor editierbar (Round-Trip-bewahrend), einblendbarer „alle"-Filter in der Mitarbeiteransicht fur rein unbezahlte Freiwillige, und jede paid-only work-details-Site explizit auf `sales_person.is_paid` gegated (kein Leak in paid_hours/Billing/Year-Summary).
**Verified:** 2026-06-24T16:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `committed_voluntary` is a field on the frontend `EmployeeWorkDetails` state struct (SC-1) | VERIFIED | `pub committed_voluntary: f32` at line 64 of `shifty-dioxus/src/state/employee_work_details.rs` |
| 2 | Both `TryFrom` directions (`TO→State` and `State→TO`) thread `committed_voluntary` — no 0.0 hardcode in TryFrom blocks (SC-1) | VERIFIED | Two occurrences of `committed_voluntary: details.committed_voluntary` (lines 180 + 220); `committed_voluntary: 0.0` only in `blank_standard` init |
| 3 | Open→Save-unverändert-Round-Trip preserves the Backend value (SC-1) | VERIFIED | `committed_voluntary_round_trip` and `committed_voluntary_from_to_maps_field` tests pass (6/6 in employee_work_details suite) |
| 4 | D-05 read-gate in `booking_information.rs::get_weekly_summary` extended to `cap || expected_hours == 0.0` at BOTH filter points (Band-1 + Band-2) (SC-3 / D-05) | VERIFIED | Lines 212 and 224 in `service_impl/src/booking_information.rs`; `grep -c "expected_hours == 0.0"` = 2; d05 tests pass (3/3) |
| 5 | `CURRENT_SNAPSHOT_SCHEMA_VERSION` remains 7 — no bump from D-05 gate extension or billing person-set gate (SC-4 / CVC-05) | VERIFIED | `const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 7` at line 75; `snapshot_schema_version_unchanged_at_7` test passes |
| 6 | `get_week` (reporting.rs) gates on `sales_person.is_paid` — unbezahlte Personen not returned in ShortEmployeeReport (SC-3 / D-06) | VERIFIED | `if !sales_person.is_paid.unwrap_or(false) { continue; }` at line 889; `get_week_skips_unpaid_person` and `get_week_unpaid_no_paid_hours_leak` tests pass (4/4 in get_week suite) |
| 7 | `build_new_billing_period` (billing_period_report.rs) gates on `is_paid` — unbezahlte Personen skipped in Billing loop (SC-3 / D-GATING-STYLE) | VERIFIED | `if !sales_person.is_paid.unwrap_or(false) { continue; }` at line 328 in `billing_period_report.rs` |
| 8 | i18n De/En/Cs vollständig: `CommittedVoluntaryLabel` + `EmployeesShowAll` in all 3 locales with correct Locale tags; Per-Locale-Matcher tests green (SC-4 / CVC-08-style) | VERIFIED | Keys present in `de.rs:419-420`, `en.rs:372-373`, `cs.rs:403-404` with correct Locale tags; `i18n_phase17_keys_match_*` tests: 3/3 pass |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `service_impl/src/reporting.rs` | is_paid gate in get_week result loop | VERIFIED | Line 889: `if !sales_person.is_paid.unwrap_or(false) { continue; }` |
| `service_impl/src/billing_period_report.rs` | is_paid gate in build_new_billing_period loop | VERIFIED | Line 328: `if !sales_person.is_paid.unwrap_or(false) { continue; }` |
| `service_impl/src/test/reporting_additive_merge.rs` | get_week integration tests (CVC-10) | VERIFIED | Tests `get_week_skips_unpaid_person` (L1147) and `get_week_unpaid_no_paid_hours_leak` (L1233) both present and passing |
| `service_impl/src/booking_information.rs` | D-05 gate extension at both .filter sites | VERIFIED | Lines 212 and 224: `cap_planned_hours_to_expected \|\| wh.expected_hours == 0.0` |
| `service_impl/src/test/booking_information.rs` | D-05 gate fixture tests | VERIFIED | 3 tests (d05_*) at lines 402, 415, 427; all pass |
| `shifty-dioxus/src/state/employee_work_details.rs` | committed_voluntary field + both TryFrom + Round-Trip tests | VERIFIED | Field at L64, both TryFrom at L180/L220, tests at L275/L304 |
| `shifty-dioxus/src/component/contract_modal.rs` | conditional committed_voluntary TextInput + SSR tests | VERIFIED | show_committed signal at L173, TextInput at L368-389, 3 SSR tests at L580/L609/L634 |
| `shifty-dioxus/src/i18n/mod.rs` | CommittedVoluntaryLabel + EmployeesShowAll Key enum + Per-Locale tests | VERIFIED | Keys at L268-269, matcher tests at L858/L869/L879 |
| `shifty-dioxus/src/i18n/de.rs` | De locale translations with Locale::De tag | VERIFIED | Lines 419-420 with `Locale::De` |
| `shifty-dioxus/src/i18n/en.rs` | En locale translations with Locale::En tag | VERIFIED | Lines 372-373 with `Locale::En` |
| `shifty-dioxus/src/i18n/cs.rs` | Cs locale translations with Locale::Cs tag | VERIFIED | Lines 403-404 with `Locale::Cs` |
| `shifty-dioxus/src/component/employees_list.rs` | show_all signal + Toggle + employee_visible predicate + 3 filter tests | VERIFIED | show_all at L83, employee_visible predicate at L24, tests at L288/L303/L313 |
| `shifty-dioxus/src/loader.rs` | load_unpaid_volunteer_employees function | VERIFIED | Function at L348, filters `!sp.is_paid && !sp.inactive`, calls get_sales_persons |
| `shifty-dioxus/src/state/employee.rs` | Employee::unpaid_placeholder + 2 dummy tests | VERIFIED | unpaid_placeholder at L324, tests at L423/L441 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `reporting.rs::get_week` | `SalesPerson.is_paid` | `if !sales_person.is_paid.unwrap_or(false) { continue; }` before result.push | WIRED | Line 889 — SalesPerson fetched before push, gate applied |
| `billing_period_report.rs::build_new_billing_period` | `SalesPerson.is_paid` | `if !sales_person.is_paid.unwrap_or(false) { continue; }` first line in loop body | WIRED | Line 328 |
| `booking_information.rs::get_weekly_summary` (Band-1) | `committed_voluntary_hours` | `.filter(\|wh\| wh.cap_planned_hours_to_expected \|\| wh.expected_hours == 0.0)` | WIRED | Line 224 — both filter sites extended symmetrically |
| `booking_information.rs::get_weekly_summary` (Band-2) | `committed_voluntary_hours` | `.filter(\|wh\| ... && (wh.cap_planned_hours_to_expected \|\| wh.expected_hours == 0.0))` | WIRED | Line 212 |
| `contract_modal.rs` | `EmployeeWorkDetails.committed_voluntary` | `show_committed = cap \|\| expected_hours==0.0` + `next.committed_voluntary = n` dispatch | WIRED | Lines 173 + 384 |
| `state/employee_work_details.rs` (State→TO) | `EmployeeWorkDetailsTO.committed_voluntary` | `committed_voluntary: details.committed_voluntary` | WIRED | Line 220 — hardcode replaced |
| `employees_list.rs` | `SalesPerson.is_paid` | `employee_visible(e, show_all_val, &term)` predicate: `show_all \|\| e.sales_person.is_paid` | WIRED | Line 24/132 |
| `employees_list.rs` | `api::get_sales_persons` (GET /sales-person) | `loader::load_unpaid_volunteer_employees(config2)` in second use_resource | WIRED | Lines 79, 348-360 in loader.rs |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `contract_modal.rs` committed TextInput | `details.committed_voluntary` | `EmployeeWorkDetails` state loaded from Backend via TryFrom TO→State (line 180) | Yes — TryFrom maps the actual Backend value | FLOWING |
| `employees_list.rs` show_all merge | unpaid Employee dummies | `loader::load_unpaid_volunteer_employees` → `api::get_sales_persons` → GET /sales-person → real Backend data | Yes — real API call to /sales-person endpoint | FLOWING |
| `booking_information.rs` Band-1 committed_voluntary_hours | `wh.committed_voluntary` after extended filter | `find_working_hours_for_calendar_week(&all_work_details, ...)` from DB-backed service | Yes — DB-backed EmployeeWorkDetails | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| get_week skips unpaid person (D-06) | `cargo test -p service_impl get_week` | 4 passed (including get_week_skips_unpaid_person, get_week_unpaid_no_paid_hours_leak) | PASS |
| D-05 gate: expected_hours=0 flows into Band 1 | `cargo test -p service_impl d05` | 3 passed (d05_expected_hours_zero_flows_into_band1, d05_capped_person_still_counted, d05_uncapped_nonzero_excluded) | PASS |
| Snapshot version stays 7 | `cargo test -p service_impl snapshot_schema_version` | 3 passed | PASS |
| committed_voluntary round-trip | `cd shifty-dioxus && cargo test employee_work_details` | 6 passed including both round-trip tests | PASS |
| SSR visibility conditions (D-01) | `cd shifty-dioxus && cargo test committed` | 11 passed including 3 SSR visibility tests | PASS |
| i18n Phase 17 keys all 3 locales | `cd shifty-dioxus && cargo test i18n_phase17` | 3 passed | PASS |
| employees_list filter predicate | `cd shifty-dioxus && cargo test employees_list` | 9 passed including filter_default_hides_unpaid, filter_show_all_reveals_unpaid, filter_inactive_always_hidden | PASS |
| Unpaid dummy tests | `cd shifty-dioxus && cargo test unpaid_dummy` | 2 passed | PASS |
| Full backend workspace tests | `cargo test --workspace` | 445/445 (service_impl) + all other crates: 0 failed | PASS |
| Full frontend tests | `cd shifty-dioxus && cargo test` | 627 passed, 0 failed | PASS |
| WASM check gate | `cd shifty-dioxus && cargo check --target wasm32-unknown-unknown` | Finished dev profile, exit 0 (lld-linker limitation pre-exists — check gate is the accepted substitute per task instructions) | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| CVC-09 | 17-03 | `committed_voluntary` im Vertrags-Editor als numerisches Feld editierbar; Open→Save-unverändert-Round-Trip bewahrt Backend-Wert; beide TryFrom-Richtungen durchgezogen | SATISFIED | committed_voluntary field, both TryFrom, show_committed D-01, contract_modal TextInput all verified |
| CVC-10 | 17-01, 17-02, 17-04 | Mitarbeiteransicht show_all-Filter; rein unbezahlte Freiwillige sichtbar; alle work-details-iterierende paid-only-Sites auf is_paid gegated; get_week-Seiteneffekt-Integrationstest; kein Leak in paid_hours/Billing/Year-Summary | SATISFIED | is_paid gates in reporting.rs + billing_period_report.rs; D-05 gate extension in booking_information.rs; show_all toggle + employee_visible predicate in employees_list.rs; 8 integration/unit tests pin the no-leak invariant |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | No TODO/FIXME/placeholder comments in modified files; no 0.0 hardcode remaining in TryFrom blocks; no is_paid in contract_modal.rs; no Hex literals in employees_list.rs | — | — |

Note: The former Phase-17-scope Gap-Comment and `committed_voluntary: 0.0` hardcode in the `State→TO` TryFrom block were explicitly removed and replaced by the real field mapping. Confirmed by grep showing 0 occurrences of the old hardcode in TryFrom blocks.

### Human Verification Required

#### 1. Contract Editor — committed_voluntary live edit and round-trip

**Test:** Open the Shifty frontend. Navigate to a paid employee with `cap_planned_hours_to_expected=true`. Open their contract modal. Verify the "Freiwillige Zusage (h)" input field is visible and accepts a decimal number (e.g. 3.5). Save. Reopen the modal and confirm the field shows 3.5, not 0.
**Expected:** Field is visible, accepts decimal input, and the value survives a Save/Reopen cycle without silent reset.
**Why human:** The SSR tests simulate the `show_committed` boolean predicate and Field rendering primitives, but not the live Dioxus coroutine dispatch + store persistence pipeline. Round-trip correctness of the full dispatch cycle requires a running browser.

#### 2. Contract Editor — D-01 rein-freiwillig branch (expected_hours=0)

**Test:** Find or create an employee with `is_paid=false` and `expected_hours=0`. Open their contract modal. Verify "Freiwillige Zusage (h)" is visible even though `cap_planned_hours_to_expected=false`.
**Expected:** Field appears for the second D-01 branch (`expected_hours == 0.0`).
**Why human:** Browser runtime needed; SSR tests cover the `show_committed=true` code path but not the full signal evaluation in live rendering.

#### 3. Mitarbeiteransicht — show_all toggle reveals unbezahlte Freiwillige

**Test:** Open the Mitarbeiteransicht (employees list). Confirm only paid employees are visible by default. Click the "alle" checkbox. Confirm that unpaid non-inactive volunteers (`is_paid=false, !inactive`) now appear in the list. Confirm inactive persons are absent regardless.
**Expected:** Default: paid-only. show_all=true: paid + unpaid non-inactive. Inactive: always hidden.
**Why human:** The `show_all` Dioxus signal and the two-resource merge trigger a re-render that can only be observed in a live browser. Unit tests pin the `employee_visible` predicate but not the full resource-merge rendering.

### Gaps Summary

No automated gaps found. All 8 must-have truths are VERIFIED against actual codebase evidence:

- Backend is_paid gates exist and are tested (Plan 01)
- D-05 gate extension exists at both `.filter` sites and is tested (Plan 02)
- committed_voluntary threads through state struct and both TryFrom directions with round-trip tests (Plan 03)
- Conditional TextInput with D-01 visibility condition exists in contract_modal (Plan 03)
- i18n keys complete in all 3 locales with correct Locale tags and Per-Locale-Matcher tests (Plan 03)
- show_all toggle + employee_visible predicate + load_unpaid_volunteer_employees + Employee::unpaid_placeholder all present and tested (Plan 04)
- CURRENT_SNAPSHOT_SCHEMA_VERSION = 7, regression test passes (Plans 01 + 02)
- Full test suites green: backend 445/445, frontend 627/627, WASM check exits 0

3 human-verification items remain for live browser UX confirmation of the editor field, D-01 rein-freiwillig branch, and show_all toggle in action.

---

_Verified: 2026-06-24T16:00:00Z_
_Verifier: Claude (gsd-verifier)_
