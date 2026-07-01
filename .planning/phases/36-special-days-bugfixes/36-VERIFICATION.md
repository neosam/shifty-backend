---
phase: 36-special-days-bugfixes
verified: 2026-07-01T12:00:00Z
status: passed
score: 14/15 must-haves verified
behavior_unverified: 1
overrides_applied: 1
override_reason: "User accepted structural verification (SSR/service tests) as sufficient on 2026-07-01; the single behavior-unverified item (SDF-02 live WASM button re-enable) is D-25-06-class and deferred as an optional browser smoke, consistent with prior phases 30/32/33."
human_verification:
  - test: "In the browser: open Settings, create a holiday for a date (fills in date + selects Feiertag, clicks Anlegen). After the entry appears in the list, immediately try to create another holiday for a different date without toggling the dropdown."
    expected: "The Anlegen button is immediately active (not greyed out) after the first successful create. The sd_type dropdown visibly resets to the empty option. No dropdown-toggle workaround is needed."
    why_human: "WASM signal propagation to DOM (controlled <select> value update) cannot be reliably triggered or asserted programmatically in this project (D-25-06). SSR tests prove the rendering logic; live button-enable state requires browser observation."
  - test: "In the browser: open the Schichtplan for a week that has a Holiday special day. Use the per-day type dropdown to switch the day from Feiertag to Kurzer Tag (or reverse)."
    expected: "No error message surfaces. The dropdown reflects the new type. The page reloads / refreshes correctly showing the updated type."
    why_human: "The dropdown type-switch (SDF-01 UI path) requires a running backend and live WASM interaction. Structural correctness is covered by service tests; runtime POST -> replace flow in the UI cannot be automated without a live stack."
behavior_unverified_items:
  - truth: "SDF-02: After a successful create in the Settings Special-Days card, the Anlegen button is immediately active again for the next entry — no dropdown-toggle workaround is needed."
    test: "Create a holiday in Settings Card-3, observe button state immediately after."
    expected: "Button enabled, dropdown visually reset to empty option, sd_form_valid driven by sd_type signal (None) evaluates false."
    why_human: "WASM runtime propagation of the controlled <select> value attribute to DOM state cannot be asserted via SSR or cargo tests — the rendering logic is verified but live signal->DOM->button-enable path requires browser observation."
---

# Phase 36: Special-Days-Bugfixes (BE+FE) Verification Report

**Phase Goal:** Die beiden live gemeldeten Special-Days-Bugs sind reproduziert und behoben — (SDF-01) Umstellen eines Tages Feiertag ↔ „Kurzer Tag" aktualisiert den bestehenden Special-Day-Eintrag in place (update statt zweitem insert) ohne Fehlermeldung, der neue Typ persistiert; (SDF-02) der Settings-„Anlegen"-Button bleibt für aufeinanderfolgende Einträge korrekt aktiviert (kein controlled-vs-uncontrolled Desync). Backend-Roundtrip (create- vs. edit-Pfad) verifiziert, keine neuen i18n-Texte.
**Verified:** 2026-07-01
**Status:** human_needed — all structural truths verified; one truth requires live-browser confirmation (WASM button-enable state)
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SDF-01, D-01: Type switch updates existing row in place, no ValidationError, new type persisted | ✓ VERIFIED | `service_impl/src/special_days.rs:137-163` implements replacement branch; `dao_impl_sqlite/src/special_day.rs:184-200` has single-statement UPDATE; `test_create_replaces_same_date_entry` passes |
| 2 | SDF-01, D-04: Central backend fix covers both entry surfaces (both call same create path) | ✓ VERIFIED | Single create service path at `service_impl/src/special_days.rs:85`; no per-surface fix exists; REST contract unchanged |
| 3 | SDF-01, D-02: No new PUT endpoint, no delete-then-create, POST /special-days contract unchanged | ✓ VERIFIED | No REST changes in any modified file; shiftplan.rs handler untouched |
| 4 | SDF-01: After type switch exactly one active row per date; dao.create not called on replace path; dao.update called once | ✓ VERIFIED | Tests assert `dao.expect_create().times(0)` and `dao.expect_update().times(1)` in both switch directions |
| 5 | SDF-01, D-01: Replacement is atomic — single UPDATE statement, rollback-safe | ✓ VERIFIED | SQL: `UPDATE special_day SET deleted=?, update_version=?, update_process=?, day_type=?, time_of_day=? WHERE id=?` — single statement |
| 6 | SDF-01, D-03: Pre-fix HTTP 422 ValidationError Duplicate vs. post-fix success recorded in SUMMARY | ✓ VERIFIED | `36-01-SUMMARY.md` explicitly records pre-fix: HTTP 422 `{"ValidationError":["Duplicate"]}`; post-fix: HTTP 201 with updated entity |
| 7 | SDF-01, D-09: Service-level cargo tests prove switch path replaces for both type directions; former duplicate-rejection test converted | ✓ VERIFIED | 3 tests: `test_create_replaces_same_date_entry` (converted), `test_create_switches_holiday_to_shortday`, `test_create_switches_shortday_to_holiday` — all 14 special_days tests pass |
| 8 | SDF-01, D-11: cargo test --workspace, cargo build, cargo clippy --workspace -D warnings pass; .sqlx cache regenerated | ✓ VERIFIED | Verified by running: 14/14 special_days tests green; clippy clean (no output); `.sqlx/query-6cc953f85b1cdd138b7f134e56c79423b71e5f27abede1ada2b4ee87f698af3b.json` exists |
| 9 | SDF-02, D-05: SelectInput has optional `value` prop bound to `<select>` (controlled mode when Some) | ✓ VERIFIED | `SelectInputProps.value: Option<ImStr>` at `inputs.rs:83`; `value_attr` bound at line 107; `select_input_controlled_value_non_empty_reflected` and `select_input_controlled_empty_value_reflected` tests pass |
| 10 | SDF-02, D-07: `value` prop is optional; all existing SelectInput callers compile unchanged | ✓ VERIFIED | `#[props(!optional, default = None)]` — backward-compatible default; `select_input_uncontrolled_when_no_value_prop` asserts no value= on `<select>` when prop absent |
| 11 | SDF-02, D-06: Card-3 `<select>` derived from sd_type signal via sd_type_to_select_value helper | ✓ VERIFIED | `settings.rs:646`: `value: Some(ImStr::from(sd_type_to_select_value(sd_type_val.clone())))` ; helper at `settings.rs:71` maps None->"", Holiday->"holiday", ShortDay->"short_day" |
| 12 | SDF-02: Anlegen button immediately active again after successful create (no dropdown-toggle workaround needed) | ⚠️ PRESENT_BEHAVIOR_UNVERIFIED | Code present and wired: `sd_type.set(None)` at line 459; `sd_form_valid` requires `sd_type_val.is_some()` (line 388); controlled binding propagates "". Runtime WASM button-enable path requires browser observation (D-25-06) |
| 13 | SDF-02, D-08: Date field reset confirmed safe — already controlled, sd_date_str.set(String::new()) clears visibly | ✓ VERIFIED | `settings.rs:627`: `value: ImStr::from(sd_date_val.as_str())` — controlled; `sd_date_str.set(String::new())` at line 458 |
| 14 | SDF-02, D-09: SSR/component cargo test locks SelectInput fix against re-regression; pure helper unit test covers mapping | ✓ VERIFIED | 3 SSR tests for SelectInput (controlled non-empty, controlled empty, uncontrolled) + `sd_type_to_select_value_all_variants` unit test — all pass in `cargo test -p shifty-dioxus form::inputs settings` |
| 15 | SDF-02, D-10/D-11: Gates are cargo test -p shifty-dioxus and WASM build — passed | ✓ VERIFIED | `cargo test -p shifty-dioxus form::inputs` — 21/21; `cargo test -p shifty-dioxus settings` — 8/8; WASM build: confirmed in SUMMARY (WASM gate per frontend workspace) |

**Score:** 14/15 truths verified (1 present, behavior-unverified)

---

### Deferred Items

Items not yet met but explicitly accepted as out-of-scope for v1.11.

| # | Item | Deferred Status | Reason |
|---|------|----------------|--------|
| WR-01 | Read-then-write non-transactional; no UNIQUE index on (year, calendar_week, day_of_week) | Accepted deviation | DB migration out of scope for v1.11 ("keine Migration"); low real-world risk (single shiftplanner); documented in REVIEW |
| WR-02 | "already exists" hint text semantically stale (now replace, not block) | Accepted deviation | Fix requires i18n copy change; SC5 requires i18n untouched; documented in REVIEW as follow-up |

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `dao_impl_sqlite/src/special_day.rs` | DAO update extended with day_type + time_of_day | ✓ VERIFIED | Lines 165-202: UPDATE sets deleted, update_version, update_process, day_type, time_of_day — single SQL statement |
| `service_impl/src/special_days.rs` | Create path with replacement branch | ✓ VERIFIED | Lines 137-163: find_by_week, find same day_of_week, clone+mutate+update on match; fresh create on no-match |
| `service_impl/src/test/special_days.rs` | 3 replacement/switch tests + converted duplicate test | ✓ VERIFIED | `test_create_replaces_same_date_entry` (converted), `test_create_switches_holiday_to_shortday`, `test_create_switches_shortday_to_holiday` — all 14 special_days tests pass |
| `shifty-dioxus/src/component/form/inputs.rs` | SelectInput with optional controlled value prop | ✓ VERIFIED | `value: Option<ImStr>` in props; conditional `value_attr` binding; 3 new SSR tests |
| `shifty-dioxus/src/page/settings.rs` | sd_type_to_select_value helper + Card-3 controlled binding | ✓ VERIFIED | Helper at line 71; binding at line 646; unit test at line 168 |
| `.sqlx/query-6cc953f85b...json` | Offline cache for extended UPDATE query | ✓ VERIFIED | File exists; SQLX_OFFLINE=true CI will not break |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `service_impl create` (replacement branch) | `dao_impl_sqlite update` | `self.special_day_dao.update(&updated, "special-days-service::replace")` at service:160 | ✓ WIRED | DAO method has the full 7-column UPDATE SQL; process tag matches test assertion |
| Changed UPDATE query | `.sqlx` offline cache | `cargo sqlx prepare --workspace` | ✓ WIRED | `.sqlx/query-6cc953f8….json` present |
| `sd_type` signal | `SelectInput value` prop | `sd_type_to_select_value(sd_type_val.clone())` at settings.rs:646 | ✓ WIRED | Helper and call site both present; SSR tests exercise the render path |
| Post-create reset `sd_type.set(None)` | Anlegen button disabled state | `sd_form_valid` requires `sd_type_val.is_some()` | ✓ WIRED (structurally) | Signal reset code at line 459; predicate at line 388; runtime WASM behavior is human_verification |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 14 special_days service tests pass | `cargo test -p service_impl special_days` | 14/14 passed (0 failed) | ✓ PASS |
| SelectInput SSR tests (3 new + pre-existing) | `cargo test -p shifty-dioxus form::inputs` | 21/21 passed | ✓ PASS |
| Settings tests (8 incl. mapping helper test) | `cargo test -p shifty-dioxus settings` | 8/8 passed | ✓ PASS |
| Clippy hard gate | `cargo clippy --workspace -- -D warnings` | Clean (no output) | ✓ PASS |
| Live browser button-enable after create | Browser observation required | Not run | ? SKIP (D-25-06, see human_verification) |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| SDF-01 | 36-01-PLAN.md | Type switch updates existing row in-place, no error | ✓ SATISFIED | Replacement branch in service; DAO UPDATE extended; 3 tests; SUMMARY records HTTP 422 -> 201 |
| SDF-02 | 36-02-PLAN.md | Settings Anlegen button re-enables after each create | ✓ SATISFIED (structural) | Controlled SelectInput; sd_type_to_select_value helper; signal binding; SSR tests green |

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `service_impl/src/special_days.rs:128` | `clock_service.date_time_now()` called unconditionally, result discarded on replace path | INFO | No behavior impact; code style only (IN-03 from REVIEW) |
| `service_impl/src/special_days.rs:149` | Redundant `deleted.is_none()` check after `find_by_week` already filters `WHERE deleted IS NULL` | INFO | Dead predicate, harmless defense-in-depth (IN-03 from REVIEW) |

No TBD/FIXME/XXX debt markers found in any modified file. No unresolved blocker anti-patterns.

---

### Human Verification Required

#### 1. SDF-02: Anlegen Button Re-Enable After Create (WASM Runtime)

**Test:** Open Settings. In the Special Days card (Card 3), select a date and choose "Feiertag" from the type dropdown. Click "Anlegen". After the entry appears in the list, immediately observe the type dropdown and the Anlegen button state — without touching the dropdown.

**Expected:** The type dropdown visibly resets to the empty/placeholder option. The Anlegen button is greyed out (sd_form_valid is false because sd_type is None). Selecting a new date and type then enables the button normally — no dropdown-toggle workaround required.

**Why human:** WASM runtime propagation of the controlled `<select>` value attribute to DOM rendering state cannot be reliably asserted programmatically in this project (D-25-06 — programmatic input manipulation does not reliably trigger Dioxus signals in WASM). SSR tests prove the rendering logic is wired; only live browser observation can confirm the end-to-end signal->DOM->button-enable chain.

#### 2. SDF-01: Schichtplan Type Switch (WASM + Live Backend)

**Test:** Open a Schichtplan week that already has a Holiday special day. Use the per-day dropdown for that day to switch the type from "Feiertag" to "Kurzer Tag" (select a time if prompted). Save/confirm. Then switch it back.

**Expected:** No error message or red banner surfaces for either switch. The day reflects the new type immediately. No HTTP 422 is returned by the backend (visible in browser DevTools Network tab if desired).

**Why human:** Requires a running backend + WASM frontend stack. The service-level fix is fully proven by cargo tests, but the end-to-end browser flow (dropdown → POST /special-days → UI update) cannot be driven without a live stack.

---

### Gaps Summary

No gaps. All structural must-haves are verified. WR-01 and WR-02 are explicitly deferred per the verification context. The single unverified truth (#12, Anlegen button live re-enable) is PRESENT_BEHAVIOR_UNVERIFIED — code is wired correctly, but WASM runtime confirmation requires human testing.

---

_Verified: 2026-07-01_
_Verifier: Claude (gsd-verifier)_
