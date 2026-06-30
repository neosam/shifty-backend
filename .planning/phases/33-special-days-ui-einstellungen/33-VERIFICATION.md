---
phase: 33-special-days-ui-einstellungen
verified: 2026-06-30T14:00:00Z
status: human_needed
score: 15/20 must-haves verified
behavior_unverified: 5
overrides_applied: 0
re_verification: false
behavior_unverified_items:
  - truth: "D-33-04: calendar date entered in the date picker is mapped via parse_date_to_iso_parts and POSTed via create_special_day (WASM datepicker oninput binding)"
    test: "In Settings as shiftplanner: pick a specific date in the date picker, choose Holiday, click Add"
    expected: "The entry appears in the year list with the correct date context string (DD.MM.YYYY / KW NN / year), and persists across reload"
    why_human: "project memory D-25-06: programmatic setting of <input type=date> does not trigger Dioxus signals; oninput binding correctness requires real browser interaction to confirm the signal fires"
  - truth: "D-33-06 (Settings): when type = ShortDay the Add button stays disabled until a valid time is entered; Holiday shows no time field"
    test: "In Settings Card-3: enter a date, leave type empty -> Add disabled. Select Holiday -> Add enabled, no time field. Select ShortDay -> Add disabled until time entered"
    expected: "Add button only activates when date + type + (time if ShortDay) are all filled"
    why_human: "WASM Dioxus signal-to-disabled binding requires browser to observe the actual button disabled state"
  - truth: "D-33-08 / SPD-02: existing special days render chronologically ascending each as 'DD.MM.YYYY (Weekday, KW NN, YYYY)' with Holiday/ShortDay badge; ShortDay shows time"
    test: "In Settings Card-3: create a Holiday and a ShortDay for the same year, check the list"
    expected: "Entries appear in ascending calendar_week/day_of_week order, with locale date format, correct weekday/KW context, and colored type badges (Holiday accent, ShortDay warn)"
    why_human: "Visual rendering with i18n date formatting and Tailwind badge styles requires browser verification"
  - truth: "D-33-03: weekday dropdown Feiertag/Nichts create/delete roundtrip in the shiftplan week grid"
    test: "In Shiftplan week view as shiftplanner: click Feiertag for Monday -> holiday dot appears; click Nichts -> dot disappears; navigate to next week and back -> state persists correctly"
    expected: "Holiday creates; Nichts deletes; both reload the week special-days and shift_plan_context; trigger reflects current state"
    why_human: "Dioxus use_resource restart + slot-filter re-render requires browser to observe state consistency across mutations"
  - truth: "D-33-06 (Shiftplan): choosing 'Kurzer Tag' opens inline time prompt; Save disabled until time entered; ShortDay created with parsed time"
    test: "In Shiftplan week view as shiftplanner: click Kurzer Tag for a day -> inline time field + Save/Cancel appear; Save disabled; enter 13:00 -> Save enabled; click Save"
    expected: "ShortDay (warn dot) appears for that day, persists after week navigation; Cancel makes no change"
    why_human: "Dioxus signal-driven conditional rendering and disabled state require browser interaction to confirm the ShortDay inline prompt flow"
human_verification:
  - test: "Settings Card-3 datepicker persist/display loop (D-33-04, D-25-06)"
    expected: "Picking a date in the Settings date picker triggers the on_change signal; creating a Holiday/ShortDay entry persists it and it appears correctly in the year list with derived context string"
    why_human: "WASM datepicker oninput signal behavior is not unit-testable in this project (project memory reference D-25-06)"
  - test: "Settings Card-3 disabled/enabled button state for Holiday vs ShortDay (D-33-06)"
    expected: "Add button only enables when date + type + (time for ShortDay) are all filled; no time field for Holiday"
    why_human: "Signal-to-disabled binding requires real browser observation"
  - test: "Settings Card-3 chronological list with context string and type badges (D-33-08)"
    expected: "Entries shown as 'DD.MM.YYYY (Weekday, KW NN, YYYY)' with accent badge (Holiday) or warn badge + time (ShortDay), sorted ascending"
    why_human: "Visual rendering and i18n date formatting need browser verification"
  - test: "Shiftplan per-weekday dropdown Holiday/Nichts create-delete roundtrip (D-33-03)"
    expected: "Holiday dot appears after create; Nichts removes it; both surface and shift_plan_context reload correctly"
    why_human: "Dioxus use_resource restart behavior needs browser to observe state updates"
  - test: "Shiftplan ShortDay inline time prompt and browser disable state (D-33-06)"
    expected: "Kurzer Tag opens inline prompt; Save disabled until time entered; entering time + Save creates ShortDay with warn dot; Cancel writes nothing"
    why_human: "Inline prompt signal-driven rendering + disabled state require browser"
  - test: "Shiftplanner gating on both surfaces as non-shiftplanner (SPD-04 / D-33-01)"
    expected: "Settings Card-3 hidden; shiftplan weekday dropdown absent — no 403 mismatch"
    why_human: "Role-based UI visibility requires a real user session with non-shiftplanner role"
---

# Phase 33: Special-Days-UI-Einstellungen Verification Report

**Phase Goal:** Ein Shiftplanner kann Special Days (Holiday/ShortDay) auf ZWEI Flächen voll-CRUD pflegen — interaktiv im Schichtplan-Wochenraster (Per-Tag-Dropdown Feiertag/Kurzer Tag/Nichts) UND über eine Settings-Sektion (Kalenderdatum-Picker + Jahres-Liste mit abgeleitetem Kontext) — verdrahtet gegen die bestehende REST-CRUD (POST/DELETE /special-days, for-week-Read) plus einen NEUEN Range/Jahr-Read-Endpoint (GET /special-days/for-year/{year}). Shiftplanner-gated, i18n de/en/cs.

**Verified:** 2026-06-30T14:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | D-33-05: GET /special-days/for-year/{year} returns full year [SpecialDayTO] array (DAO→Service→REST→ApiDoc) | VERIFIED | `rest/src/special_day.rs`: route at line 24, handler at line 81, ApiDoc at line 174; DAO SQL: `WHERE year = ? AND deleted IS NULL ORDER BY calendar_week ASC, day_of_week ASC` |
| 2 | D-33-05: get_by_year delegates to find_by_year, ungated (_context) | VERIFIED | `service_impl/src/special_days.rs:72-83`: uses `_context`, delegates to `find_by_year`; `test_get_by_year_delegates_and_maps` passes |
| 3 | D-33-01: create and delete shiftplanner-gated (Forbidden without privilege) | VERIFIED | `service_impl/src/special_days.rs:90-92, 170-172`: `check_permission(SHIFTPLANNER_PRIVILEGE)`; tests `test_create_forbidden_without_shiftplanner` and `test_delete_forbidden_without_shiftplanner` pass (12/12 special_days tests green) |
| 4 | SPD-01: create rejects non-nil id (IdSetOnCreate) and non-nil version (VersionSetOnCreate) | VERIFIED | `service_impl/src/special_days.rs:131-136`; tests `test_create_rejects_nonnil_id` and `test_create_rejects_nonnil_version` pass |
| 5 | SPD-03: delete of missing/deleted id returns EntityNotFound | VERIFIED | `service_impl/src/special_days.rs:174-182`; test `test_delete_not_found` passes |
| 6 | D-33-05 FE: get_special_days_for_year(config, year) fetches GET /special-days/for-year/{year} returning Rc<[SpecialDayTO]> | VERIFIED | `shifty-dioxus/src/api.rs:988-997`: URL `format!("{}/special-days/for-year/{}", config.backend, year)` |
| 7 | SPD-01 FE: create_special_day forces id=Uuid::nil() and version=Uuid::nil() before POST | VERIFIED | `shifty-dioxus/src/api.rs:1005-1006`: `body.id = Uuid::nil(); body.version = Uuid::nil();` before `client.post(url).json(&body)` |
| 8 | SPD-03 FE: delete_special_day issues DELETE /special-days/{id} and errors on non-2xx | VERIFIED | `shifty-dioxus/src/api.rs:1016-1022`: DELETE URL, `error_for_status_ref()?` |
| 9 | SPD-04: 18 new i18n Key variants exist in de/en/cs (compile-time exhaustive match) | VERIFIED | `i18n/mod.rs` lines 640-674: 18 keys; `grep -c` on de.rs/en.rs/cs.rs each returns 18; Dioxus host tests (7 tests) pass |
| 10 | D-33-02: Card-3 gated by inner has_privilege("shiftplanner"), not page-level admin gate | VERIFIED | `settings.rs:322-328`: `is_shiftplanner = AUTH.map(|a| a.has_privilege("shiftplanner"))`; Card-3 wrapped at line 553: `if is_shiftplanner {` |
| 11 | D-33-04: calendar date → parse_date_to_iso_parts → create_special_day POST (D-25-06 oninput binding) | PRESENT_BEHAVIOR_UNVERIFIED | `parse_date_to_iso_parts` exists and is tested (host test passes); wired via `on_add_special_day` at settings.rs:376; TextInput uses `on_change` (oninput); but WASM datepicker signal-fire requires browser |
| 12 | D-33-06 (Settings): ShortDay requires time; Add disabled until valid type+time; Holiday shows no time field | PRESENT_BEHAVIOR_UNVERIFIED | `settings.rs:357-359`: `sd_form_valid` includes `sd_type_val.is_some()` + ShortDay/time guard; `disabled: !sd_form_valid` at line 649; conditional time field at line 628; actual disabled state in WASM needs browser |
| 13 | D-33-07 (Settings): live duplicate hint shown when (year, calendar_week, day_of_week) already exists | VERIFIED | `settings.rs:362-364`: `sd_is_duplicate` computed via `is_duplicate_special_day`; hint rendered at line 656-659; `is_duplicate_special_day` tested (2 host tests pass) |
| 14 | D-33-08/SPD-02: year list renders chronologically ascending as 'DD.MM.YYYY (Weekday, KW NN, YYYY)' with type badges | PRESENT_BEHAVIOR_UNVERIFIED | `settings.rs:691-765`: list iterates resource, calls `special_day_iso_date`, `weekday_key`, i18n context string, badge classes; helper functions tested; actual browser rendering needs verification |
| 15 | SPD-03 (Settings): each row has Danger delete button calling delete_special_day then reloading, no confirmation dialog | VERIFIED | `settings.rs:745-756`: `api::delete_special_day(cfg, entry_id)`, then `sd_resource.restart()`; no confirm dialog in code |
| 16 | D-33-03: weekday column dropdown (Holiday/ShortDay/Nichts) in shiftplan week grid for Mo-So | PRESENT_BEHAVIOR_UNVERIFIED | `shiftplan.rs:782-912`: `weekday_sub_headers` Vec built for all 7 weekdays, DropdownEntry triples wired; but browser roundtrip (dot/trigger reflect state, reload after mutation) requires human |
| 17 | D-33-01/SPD-04: shiftplan dropdown renders only inside existing is_shiftplanner guard | VERIFIED | `shiftplan.rs:782`: `let weekday_sub_headers = if is_shiftplanner {`; non-shiftplanner gets empty vec |
| 18 | D-33-06 (Shiftplan): Kurzer Tag opens inline time prompt; Save disabled until time entered; ShortDay created with parsed time | PRESENT_BEHAVIOR_UNVERIFIED | `shiftplan.rs:915-998`: `if *shortday_prompt_day.read() == Some(day)` renders inline form; `disabled: shortday_time.read().is_empty()` at line 939; dual-format time parse WR-06 at lines 950-951; actual browser disabled state needs verification |
| 19 | SPD-01/03 (Shiftplan): after create or delete both special_days_for_week and shift_plan_context reload | VERIFIED | `shiftplan.rs:848-849, 895-897, 972-973`: `special_days_for_week.restart()` and `shift_plan_context.restart()` both called in every mutation branch (Holiday create, Nichts delete, ShortDay confirm) |
| 20 | D-33-07 (Shiftplan): no pre-check for duplicates; backend error shown inline under the day column | VERIFIED | `shiftplan.rs:852-853, 900-901, 976-977`: only sets `special_day_error` signal on `Err(_)`; no pre-check code; error rendered at lines 1009-1013 |

**Score:** 15/20 truths verified (5 present, behavior-unverified)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `dao/src/special_day.rs` | `find_by_year` trait method | VERIFIED | Line 36: `async fn find_by_year(&self, year: u32) -> Result<Arc<[SpecialDayEntity]>, DaoError>` |
| `dao_impl_sqlite/src/special_day.rs` | `find_by_year` SQLx impl | VERIFIED | Lines 108-126: SQL with `WHERE year = ? AND deleted IS NULL ORDER BY calendar_week ASC, day_of_week ASC` |
| `service/src/special_days.rs` | `get_by_year` trait method | VERIFIED | Lines 91-95: trait method with `_context` ungated |
| `service_impl/src/special_days.rs` | `get_by_year` impl, create validation, duplicate guard | VERIFIED | Lines 72-83 (get_by_year), 85-164 (create with validation + duplicate guard), 165-192 (delete) |
| `rest/src/special_day.rs` | handler + route + ApiDoc | VERIFIED | Route at line 24, handler at 81-102, ApiDoc at 167-180 |
| `service_impl/src/test/special_days.rs` | 11 tests in special_days module | VERIFIED | 11 test functions covering all plan scenarios; 12 total run under `special_days` filter (1 from shiftplan module matches); all pass |
| `service_impl/src/test/mod.rs` | `pub mod special_days` registered | VERIFIED | Line 74: `#[cfg(test)] pub mod special_days;` |
| `shifty-dioxus/src/api.rs` | 3 API functions | VERIFIED | Lines 988-1022: `get_special_days_for_year`, `create_special_day`, `delete_special_day` |
| `shifty-dioxus/src/i18n/mod.rs` | 18 new Key variants | VERIFIED | Lines 640-674: exactly 18 new keys |
| `shifty-dioxus/src/i18n/de.rs, en.rs, cs.rs` | 18 translations each | VERIFIED | `grep -c` returns 18 for each locale file |
| `shifty-dioxus/src/page/settings.rs` | Card-3 + date helpers + tests | VERIFIED | Lines 31-148 (helpers + tests); 319-765 (Card-3 RSX block) |
| `shifty-dioxus/src/page/shiftplan.rs` | per-weekday dropdown + ShortDay prompt | VERIFIED | Lines 250-252 (signals); 782-1015 (weekday_sub_headers + prompt) |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `rest/src/special_day.rs:24` | `get_special_days_for_year` handler | `generate_route` `.route("/for-year/{year}", get(...))` | WIRED | route line 23-26 |
| `rest/src/special_day.rs:174` | `get_special_days_for_year` | `SpecialDayApiDoc paths(...)` | WIRED | ApiDoc `paths(get_special_days_for_year, ...)` line 172-176 |
| `dao_impl_sqlite find_by_year SQL` | soft-delete filter + sort | `WHERE deleted IS NULL ORDER BY calendar_week ASC, day_of_week ASC` | WIRED | dao_impl_sqlite line 113-116 |
| `settings.rs Card-3` | `api::get_special_days_for_year` | `use_resource` at line 341-347 | WIRED | year signal drives resource |
| `settings.rs on_add_special_day` | `api::create_special_day` | `spawn(async move { api::create_special_day(cfg, body).await })` line 417 | WIRED | |
| `settings.rs delete button` | `api::delete_special_day` | `api::delete_special_day(cfg, entry_id)` line 745 | WIRED | |
| `shiftplan.rs Holiday entry` | `api::create_special_day` | spawn at line 846 | WIRED | both `special_days_for_week.restart()` + `shift_plan_context.restart()` at 848-849 |
| `shiftplan.rs Nichts entry` | `api::delete_special_day` | spawn at line 893 | WIRED | both restarts at 895-896 |
| `shiftplan.rs ShortDay confirm` | `api::create_special_day` | spawn at line 968 | WIRED | both restarts at 972-973 |
| `settings.rs Card-3 inner guard` | `has_privilege("shiftplanner")` | `AUTH.read().map(|a| a.has_privilege("shiftplanner"))` line 322-328 | WIRED | separate from page-level admin gate |
| `shiftplan.rs weekday_sub_headers` | `is_shiftplanner` guard | `if is_shiftplanner { ... }` at line 782 | WIRED | |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `settings.rs Card-3 year list` | `sd_list` (from sd_resource) | `api::get_special_days_for_year(config, *sd_year.read())` → `GET /special-days/for-year/{year}` → `find_by_year` DAO SQL | Yes — SQLx query with real DB | FLOWING |
| `shiftplan.rs weekday_sub_headers` | `loaded_special_days` | `api::get_special_days_for_week(config, year_val, week_val)` → `GET /special-days/for-week/{year}/{week}` | Yes — existing endpoint | FLOWING |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 12 special_days tests pass | `cargo test -p service_impl special_days` | 12 passed, 0 failed | PASS |
| 7 Dioxus host tests pass (settings helpers + component) | `cd shifty-dioxus && cargo test -- settings` | 7 passed, 0 failed (includes all 6 settings helper tests) | PASS |
| WASM build (i18n completeness gate) | Orchestrator confirmed: `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` | green | PASS |
| Workspace clippy | Orchestrator confirmed: `cargo clippy --workspace -- -D warnings` | clean | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| SPD-01 | Plans 01, 02, 03, 04 | Shiftplanner kann Special Day per Kalenderdatum anlegen (Holiday/ShortDay+Uhrzeit) | SATISFIED | Backend: nil-id/version guard + ShortDay validation; FE: api.rs forces nil; Settings: parse_date_to_iso_parts → create; Shiftplan: inline ShortDay prompt |
| SPD-02 | Plans 01, 02, 03 | Liste mit Datum + abgeleitetem Kontext (DD.MM.YYYY, KW NN, Wochentag) | SATISFIED | New `GET /special-days/for-year/{year}` endpoint; settings.rs year list with context string helpers; helpers tested |
| SPD-03 | Plans 01, 02, 03, 04 | Special Day löschen, Liste aktualisiert sich | SATISFIED | Backend delete gate tested; FE api.rs delete_special_day; settings.rs delete button; shiftplan Nichts entry; both restart resource + context |
| SPD-04 | Plans 01, 02, 03, 04 | Shiftplanner-gated auf beiden Flächen; alle Texte i18n de/en/cs | SATISFIED | Backend SHIFTPLANNER_PRIVILEGE gate (tested); Settings inner `has_privilege("shiftplanner")` guard; Shiftplan `if is_shiftplanner` guard; 18 keys × 3 locales (verified by grep + WASM compile) |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `shifty-dioxus/src/api.rs` | 1485 | `// TODO: Find a better way to convert serde error` | Info | Pre-existing in `list_user_invitations` function (line 1445) — NOT introduced by Phase 33 (Phase 33 functions are lines 988-1022); not a Phase 33 concern |

No TBD, FIXME, or XXX markers found in any Phase 33 modified files. No stubs (`todo!()`, `return null`, empty returns) in backend code.

**Note on post-review fixes:** The re-review (REVIEW.md iteration 2) documented two remaining issues. Both are fixed in the actual current code:
- **WR-01 (re-review):** "Add button enabled with no type" — `settings.rs:358` now includes `sd_type_val.is_some()` in `sd_form_valid`. Fixed.
- **IN-01 (re-review):** "Misleading uuid label 'booking-version' in delete" — `service_impl:185` now uses `"special-day-service::delete version"`. Fixed.

---

### Human Verification Required

The following items require browser testing due to Dioxus WASM signal/rendering behavior. Per project memory (D-25-06), datepicker signals and visual rendering cannot be exercised by unit tests in this project.

#### 1. Settings Card-3 Datepicker Persist/Display Loop (D-33-04, D-25-06)

**Test:** In Settings as shiftplanner: pick a date (e.g. 2026-08-15) in the date picker, select Holiday, click Add.
**Expected:** Entry appears in the year list as `15.08.2026 (Samstag, KW 33, 2026)` with accent badge; persists across page reload; year picker jumps to ISO week-year of the entry.
**Why human:** Project memory D-25-06: programmatic setting of `<input type=date>` does not trigger Dioxus signals. Real browser interaction required to confirm the `oninput` binding fires correctly.

#### 2. Settings Add Button Disabled State (D-33-06)

**Test:** In Settings Card-3: (a) enter date, leave type unselected → Add disabled; (b) select Holiday → Add enabled, no time field visible; (c) select ShortDay → Add re-disabled; (d) enter time → Add enabled.
**Expected:** Form validity enforced visually; no time field for Holiday.
**Why human:** Signal-to-disabled rendering in WASM Dioxus browser needs live observation.

#### 3. Settings Year List Visual Rendering (D-33-08 / SPD-02)

**Test:** After creating a Holiday and a ShortDay entry for the same year, review the list in Settings Card-3.
**Expected:** Entries sorted ascending (calendar_week, then day_of_week); Holiday entry shows accent badge + 'Feiertag' label; ShortDay entry shows warn badge + time; format is `DD.MM.YYYY (Weekday, KW NN, YYYY)`.
**Why human:** Visual rendering with locale date formatting and badge CSS requires browser.

#### 4. Shiftplan Dropdown Holiday/Nichts Roundtrip (D-33-03)

**Test:** In Shiftplan week view as shiftplanner: click Feiertag for Monday; observe trigger and slot filtering; then click Nichts; navigate away and back.
**Expected:** Holiday dot (accent) appears after create; slot filtering updates for that weekday; Nichts removes it; state persists after week navigation.
**Why human:** `use_resource.restart()` + `shift_plan_context.restart()` state propagation requires browser observation.

#### 5. Shiftplan ShortDay Inline Prompt (D-33-06)

**Test:** In Shiftplan as shiftplanner: click Kurzer Tag for any weekday; check Save button state; enter `13:00`; click Save.
**Expected:** Inline time input + Save/Cancel appear; Save disabled until time entered; after Save, warn dot appears and prompt closes; Cancel leaves no change; wrong time format shows inline error.
**Why human:** Signal-driven conditional element rendering and disabled state in WASM require browser.

#### 6. Shiftplanner Gating as Non-Shiftplanner (SPD-04 / D-33-01)

**Test:** Log in as a user without shiftplanner privilege; navigate to Settings and to Shiftplan week view.
**Expected:** Settings Card-3 is hidden; no weekday dropdown appears in shiftplan; no 403 errors triggered.
**Why human:** Role-based UI visibility requires a real authenticated session with non-shiftplanner role.

---

### Gaps Summary

No gaps found. All code-verifiable truths are VERIFIED. The 5 PRESENT_BEHAVIOR_UNVERIFIED items are all expected Dioxus/WASM browser-only behaviors documented in project memory (D-25-06) and in each plan's `<human-check>` blocks.

---

_Verified: 2026-06-30T14:00:00Z_
_Verifier: Claude (gsd-verifier)_
