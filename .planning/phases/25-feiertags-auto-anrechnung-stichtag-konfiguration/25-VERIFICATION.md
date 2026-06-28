---
phase: 25-feiertags-auto-anrechnung-stichtag-konfiguration
verified: 2026-06-28T00:00:00Z
status: passed
score: 7/7 must-haves verified
behavior_unverified: 0
overrides_applied: 0
human_verified: 2026-06-28
human_verified_note: "Browser-verified live (DEVUSER granted admin role via sqlite for the check). Item 1: set 2026-01-15 -> Save -> backend value=2026-01-15, enabled=true, 'Saved.' -> reload shows date. Item 2: Clear -> backend 204/empty, enabled=false, 'Not set - automation is off.', no dialog. Item 4: admin-gate confirmed (non-admin -> 'Not authorized.', admin -> cards). Item 3 (i18n) verified by passing completeness test (5 keys x en/de/cs) + German label compiled into WASM; no in-app locale switcher to flip live."
human_verification:
  - test: "As admin, open /settings, pick a date in the 'Feiertags-Automatik aktiv ab' card and click 'Datum speichern'. Reload the page and confirm the same date is still shown."
    expected: "Saved date persists across page reload; inline 'Saved.' feedback appears on success."
    why_human: "Programmatic <input type=date> changes do not reliably fire Dioxus signals (WASM caveat D-25-06). The save/persist-after-reload loop requires real browser interaction. Classified as pending from 25-03 Task 3 (checkpoint:human-verify)."
  - test: "Click 'Loschen (deaktivieren)' — field clears, unset hint 'Nicht gesetzt — Automatik inaktiv.' appears. No confirmation dialog."
    expected: "Field clears inline without a modal dialog."
    why_human: "Same WASM input-signal caveat; clear behaviour requires real browser interaction."
  - test: "Switch locale (en/cs) on the settings page and verify all five Card 2 labels are translated."
    expected: "All five labels (heading, description, Save, Clear, unset hint) show in the selected locale."
    why_human: "Locale switching and label rendering require a live frontend."
  - test: "Open /settings as a non-admin user and confirm Card 2 is not editable / shows not-authorized."
    expected: "Non-admin users cannot set or clear the cutoff date."
    why_human: "Admin-gate behavior requires session context that cannot be verified by grep."
---

# Phase 25: Feiertags-Auto-Anrechnung & Stichtag-Konfiguration — Verification Report

**Phase Goal:** Holidays are automatically and correctly credited in the employee report — identical in effect to a manual ExtraHours(Holiday) — and an admin can set the activation cutoff date via a settings UI.
**Verified:** 2026-06-28
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth (Requirement) | Status | Evidence |
|---|---------------------|--------|----------|
| 1 | An employee with a contracted weekday gets exactly `holiday_hours()` credited automatically from special_day (HOL-01) | VERIFIED | `build_derived_holiday_map` in `service_impl/src/reporting.rs:151` calls `wh.has_day_of_week(...)` and `wh.holiday_hours()` (lines 231-232); `test_holiday_auto_credit_basic` passes with `holiday_hours == 8.0` |
| 2 | Derived credit produces identical holiday_hours, expected_hours, and balance as a manual ExtraHours(Holiday) (HOL-02) | VERIFIED | `test_holiday_auto_credit_equivalence` passes: Run A (derived) vs Run B (manual) asserts equality of all three values; injected into both `holiday_hours` AND `absense_hours` at all three injection points (lines 543, 593/543, 872) |
| 3 | booking_information year-view hours (paid_hours/committed_voluntary_hours/volunteer_hours) are untouched (HOL-03) | VERIFIED | `service_impl/src/booking_information.rs` has NO reference to `toggle_service` or `holiday_auto_credit`; `test_holiday_auto_credit_no_year_view_impact` passes by setting up MockSpecialDayService with no expectations, proving get_week never calls it for holiday credit |
| 4 | Cutoff gate: holiday credited only when value set AND holiday_date >= cutoff; no value = automation off (HCFG-01) | VERIFIED | `reporting.rs:179` returns empty map (automation off) when no toggle value; `reporting.rs:212` gates `holiday_date < cutoff`; `test_holiday_before_cutoff_skipped` passes: cutoff "2024-03-25" with holiday "2024-03-18" → 0h; cutoff "2024-03-18" (boundary) → 8h |
| 5 | Admin can set/change/clear the cutoff date via settings UI; all five labels translated in en/de/cs (HCFG-02 — code half) | VERIFIED (code); UI behavior PENDING human verify | `shifty-dioxus/src/page/settings.rs:108` wires `get_holiday_cutoff_date`/`set_holiday_cutoff_date`; `i18n_phase25_keys_present_in_all_locales` test passes; WASM build green. Browser set/change/persist verified only by human (25-03 Task 3 pending). |
| 6 | Manual ExtraHours(Holiday) wins over derived credit — no double count (HCFG-03) | VERIFIED | Conflict check at `reporting.rs:220`: skips derived credit when `eh.category == Holiday && eh.date_time.date() == holiday_date`; `test_holiday_manual_wins` passes: special_day + manual ExtraHours same day → `holiday_hours == 8.0` (not 16.0) |
| 7 | `CURRENT_SNAPSHOT_SCHEMA_VERSION == 11`; locking test asserts 11 (HSNAP-01) | VERIFIED | `service_impl/src/billing_period_report.rs:108` has `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 11;`; `test_snapshot_schema_version_pinned` passes |

**Score:** 6/7 truths verified (Truth 5 has code fully verified; UI-behavior portion is the pending human-verify item)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `migrations/sqlite/20260628000000_toggle-value-column.sql` | ALTER TABLE toggle ADD COLUMN value TEXT | VERIFIED | Contains exactly `ALTER TABLE toggle ADD COLUMN value TEXT;` |
| `migrations/sqlite/20260628000001_seed-holiday-auto-credit-toggle.sql` | Seeds holiday_auto_credit disabled, value NULL | VERIFIED | INSERT OR IGNORE with name=`holiday_auto_credit`, enabled=0, update_process=`phase-25-migration` |
| `dao_impl_sqlite/src/toggle.rs` | get_toggle_value / set_toggle_value DAO impls, value in all SELECT/UPDATE | VERIFIED | Both methods present; `pub value: Option<String>` on ToggleDb |
| `dao/src/toggle.rs` | `get_toggle_value` / `set_toggle_value` trait methods | VERIFIED | Both trait methods at lines 175 and 190 |
| `service/src/toggle.rs` | `Toggle.value: Option<Arc<str>>` + trait methods | VERIFIED | `pub value: Option<Arc<str>>` at line 16; both service trait methods present |
| `service_impl/src/toggle.rs` | set_toggle_value with toggle_admin gate | VERIFIED | `check_permission(TOGGLE_ADMIN_PRIVILEGE, ...)` at line 182 before delegating to DAO |
| `rest-types/src/lib.rs` | `ToggleTO.value: Option<Arc<str>>` | VERIFIED | `pub value: Option<Arc<str>>` at line 1536 in ToggleTO struct |
| `rest/src/toggle.rs` | GET/PUT/DELETE /{name}/value with utoipa + ISO validation | VERIFIED | Three handlers registered at lines 27-29 in generate_route; ISO date validation at lines 352-367; `#[utoipa::path]` annotations present; registered in ToggleApiDoc at lines 733-735 |
| `service_impl/src/reporting.rs` | SpecialDayService+ToggleService deps, derived-holiday precompute, 3 injection points | VERIFIED | gen_service_impl! block at lines 79-80; `build_derived_holiday_map` helper; injection points 1a (line 1320), 1b (line 534), 1c (line 872) all present and adding to both `holiday_hours` AND `absense_hours` |
| `shifty_bin/src/main.rs` | toggle_service constructed before reporting_service; 2 new ReportingServiceImpl fields | VERIFIED | Comment at line 884 confirms Phase 25 reordering; `special_day_service` and `toggle_service` fields added at lines 905-906 |
| `service_impl/src/billing_period_report.rs` | CURRENT_SNAPSHOT_SCHEMA_VERSION = 11 | VERIFIED | Line 108: `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 11;` |
| `service_impl/src/test/billing_period_snapshot_locking.rs` | Pinned assert updated to 11 | VERIFIED | Line 28: `CURRENT_SNAPSHOT_SCHEMA_VERSION, 11,`; module doc at line 7 references Phase 25 |
| `service_impl/src/test/reporting_holiday_auto_credit.rs` | 5 tests: basic, equivalence, cutoff-boundary, manual-wins, year-view guard | VERIFIED | All 5 test functions exist; all 5 pass (confirmed by running `cargo test -p service_impl holiday_auto_credit`) |
| `service_impl/src/test/mod.rs` | pub mod registration for reporting_holiday_auto_credit | VERIFIED | Line 28: `pub mod reporting_holiday_auto_credit;` |
| `shifty-dioxus/src/api.rs` | get_toggle_value / set_toggle_value / clear_toggle_value REST clients | VERIFIED | Three functions at lines 1601, 1615, 1632 |
| `shifty-dioxus/src/loader.rs` | get_holiday_cutoff_date / set_holiday_cutoff_date | VERIFIED | Lines 1030-1042; keyed to `"holiday_auto_credit"` |
| `shifty-dioxus/src/page/settings.rs` | Card 2 — holiday auto-credit date input + Save/Clear + feedback | VERIFIED | `use_resource(get_holiday_cutoff_date)` at line 108; save/clear handlers at lines 144/166; RSX renders all five i18n keys |
| `shifty-dioxus/src/i18n/mod.rs` | 5 new Key variants | VERIFIED | Lines 598-606: all 5 Key enum variants; completeness test at lines 1332-1336 |
| `shifty-dioxus/src/i18n/en.rs`, `de.rs`, `cs.rs` | Verbatim strings for all 5 keys in all 3 locales | VERIFIED | `i18n_phase25_keys_present_in_all_locales` test passes (confirmed by running `cargo test i18n` in shifty-dioxus) |

---

### Key Link Verification

| From | To | Via | Status |
|------|----|-----|--------|
| `rest/src/toggle.rs` | `service_impl/src/toggle.rs` | `set_toggle_value` handler calls `toggle_service().set_toggle_value` | WIRED |
| `service_impl/src/toggle.rs` | `dao_impl_sqlite/src/toggle.rs` | `set_toggle_value` delegates to `toggle_dao.set_toggle_value` | WIRED |
| `service_impl/src/reporting.rs` | `service::toggle::ToggleService` | `toggle_service.get_toggle_value("holiday_auto_credit", ...)` at line 165 | WIRED |
| `service_impl/src/reporting.rs` | `service::special_days::SpecialDayService` | `special_day_service.get_by_week` at line 190 | WIRED |
| `service_impl/src/reporting.rs` | `EmployeeWorkDetails::holiday_hours` | `wh.holiday_hours()` at line 232, gated by `has_day_of_week` at line 231 | WIRED |
| `shifty-dioxus/src/page/settings.rs` | `shifty-dioxus/src/loader.rs` | `loader::get_holiday_cutoff_date` / `loader::set_holiday_cutoff_date` | WIRED |
| `shifty-dioxus/src/loader.rs` | `shifty-dioxus/src/api.rs` | `api::get_toggle_value` / `api::set_toggle_value` / `api::clear_toggle_value` with key `"holiday_auto_credit"` | WIRED |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 5 holiday auto-credit tests pass | `SQLX_OFFLINE=true cargo test -p service_impl holiday_auto_credit` | 5 passed, 0 failed | PASS |
| Snapshot version locking test passes | `SQLX_OFFLINE=true cargo test -p service_impl test_snapshot_schema_version_pinned` | 1 passed | PASS |
| i18n completeness test passes (including phase25 keys) | `cargo test i18n` in shifty-dioxus | `i18n_phase25_keys_present_in_all_locales` passed; 42 total passed | PASS |
| Workspace clippy clean | `SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings` | Finished, no warnings | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Status | Evidence |
|-------------|------------|--------|---------|
| HOL-01 | 25-02, 25-04 | SATISFIED | `build_derived_holiday_map` derives hours via `holiday_hours()` + `has_day_of_week`; `test_holiday_auto_credit_basic` passes |
| HOL-02 | 25-02, 25-04 | SATISFIED | All 3 injection points add to both `holiday_hours` AND `absense_hours`; `test_holiday_auto_credit_equivalence` passes with identical holiday_hours/expected_hours/balance |
| HOL-03 | 25-02, 25-04 | SATISFIED | `booking_information.rs` has no `toggle_service` or `holiday_auto_credit` reference; `test_holiday_auto_credit_no_year_view_impact` passes |
| HCFG-01 | 25-01, 25-02 | SATISFIED | `value` column + seed (disabled/NULL); cutoff gate at `reporting.rs:179,212`; `test_holiday_before_cutoff_skipped` passes both boundary cases |
| HCFG-02 | 25-01, 25-03 | SATISFIED (code); UI pending human verify | GET/PUT/DELETE `/toggle/{name}/value` admin-gated (toggle_admin privilege check); Settings Card 2 fully coded + WASM build green + i18n test green; browser set/change/persist-after-reload is human-verify pending |
| HCFG-03 | 25-02, 25-04 | SATISFIED | Conflict check at `reporting.rs:220` skips derived credit when manual ExtraHours(Holiday) covers same day; `test_holiday_manual_wins` passes |
| HSNAP-01 | 25-02 | SATISFIED | `CURRENT_SNAPSHOT_SCHEMA_VERSION = 11` at `billing_period_report.rs:108`; `test_snapshot_schema_version_pinned` passes |

---

### Anti-Patterns Found

No `TBD`, `FIXME`, or `XXX` markers found in any file modified by this phase. No stubs, placeholder returns, or empty implementations found in the phase output.

---

### Human Verification Required

#### 1. Settings Card 2 — set/persist-after-reload

**Test:** Start backend (port 3000) and frontend (`dx serve`, port 8080). As admin, open `http://localhost:8080/settings`. In Card 2 "Feiertags-Automatik aktiv ab", pick a date and click "Datum speichern".
**Expected:** Inline "Saved." feedback appears; after page reload, the same date is still shown in the date field.
**Why human:** Programmatic `<input type=date>` changes do not reliably fire Dioxus signals (documented caveat D-25-06, memory `reference_dioxus_browser_test_date_inputs`). The full save-reload-verify cycle requires real browser interaction. This is a deliberate `checkpoint:human-verify` from 25-03 Task 3 that remained pending.

#### 2. Settings Card 2 — clear/unset hint

**Test:** With a date set, click "Loschen (deaktivieren)".
**Expected:** Field clears immediately; unset hint "Nicht gesetzt — Automatik inaktiv." appears; no confirmation dialog.
**Why human:** Clear behavior requires real browser interaction for the same WASM signal reason.

#### 3. Settings Card 2 — locale switching

**Test:** While on /settings, switch the UI locale to "en" and then "cs".
**Expected:** All five Card 2 labels (heading, description, Save button, Clear button, unset hint) are translated in each locale. The i18n completeness test confirms the keys exist; rendering in context requires a live frontend.
**Why human:** Label rendering in the live app cannot be verified by grep or WASM build alone.

#### 4. Settings Card 2 — non-admin guard

**Test:** Open `/settings` as a user without the `toggle_admin` privilege.
**Expected:** Card 2 is not editable or shows the "not-authorized" view. The server-side gate (toggle_admin privilege in `service_impl/src/toggle.rs`) is already code-verified; the frontend admin guard (reusing Phase 24 pattern) requires a live session to confirm.
**Why human:** Admin guard requires a session context and cannot be confirmed by static analysis.

---

### Gaps Summary

No code gaps found. All seven observable truths are either fully verified or supported by complete, passing code with only the browser-interaction layer pending human confirmation. The outstanding item is exclusively the 25-03 Task 3 human-verify checkpoint, which was explicitly designed as a blocking human gate due to the documented WASM programmatic date-input signal caveat.

---

_Verified: 2026-06-28_
_Verifier: Claude (gsd-verifier)_
