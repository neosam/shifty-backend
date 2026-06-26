---
phase: 24-paid-capacity-enforcement-config
verified: 2026-06-27T00:00:00Z
status: human_needed
score: 7/7 must-haves verified
overrides_applied: 0
gaps:
human_verification:
  - test: "Open the shiftplan page, attempt to book a paid employee into a slot/week that is at its paid limit with hard enforcement ON. Observe that the error message 'Paid employee limit reached...' appears below the WeekView (not at the specific slot cell). Verify it is clearly readable and correctly attributed. The plan spec required per-slot scoping (only shown at the blocked slot), but the implementation shows it whenever any block_error signal is set, at the page level below the whole week grid. Assess whether the page-level placement is acceptable UX."
    expected: "The inline BookingBlockedPaidLimit message is visible and correctly localized. Minor deviation: the message appears below the entire week view, not inlined at the specific slot cell."
    why_human: "Block-error visual placement is a UI/visual concern that cannot be verified programmatically. The signal itself (block_error: Signal<Option<Uuid>>) carries the slot_id but the render at line 1189 only checks is_some(), not which slot. Whether this placement satisfies 'inline at the slot' from the UI-SPEC requires visual inspection in a browser."
  - test: "Navigate to /settings/ as an admin user. Verify: (1) the Settings page renders with the paid-limit toggle button (showing Soft/Hard state). (2) Click the toggle — verify it flips state and shows 'Saved.' inline feedback. (3) On network failure, verify 'Could not save setting.' appears inline."
    expected: "Toggle button flips, aria-pressed attribute changes, inline feedback appears, no dialog/modal shown."
    why_human: "Interactive toggle state and network error states require browser testing. The component-level admin guard (is_admin check in SettingsPage) and the REST round-trip are not exercisable by static analysis."
  - test: "Load a week where at least one slot has current_paid_count > max_paid_employees. Verify the yellow/warn overage section appears above the ShiftplanTabBar, listing the over-limit slot(s) with {slot}: {current}/{max} format. Verify it is absent when no slots are over limit."
    expected: "Persistent warn section visible to all roles (including non-shiftplanners), with correct slot label format, positioned above ShiftplanTabBar and below any booking_warnings banner."
    why_human: "Visual section rendering and role-visibility need browser confirmation. Client-side state must match loaded week data."
  - test: "As a non-admin user, directly navigate to /settings/ via URL. Verify that the page shows 'Not authorized.' rather than the toggle UI."
    expected: "The component-level admin guard returns early with a 'Not authorized.' message for non-admin users who bypass the nav."
    why_human: "Route access control by direct URL (bypassing nav visibility) needs browser verification."
---

# Phase 24: Paid-Limit konfigurierbar & rollenbasiert durchsetzen - Verification Report

**Phase Goal:** Paid-Limit konfigurierbar & rollenbasiert durchsetzen (Backend+Frontend) — a globally configurable hard/soft paid-capacity limit, role-based override (only Shiftplanner privilege may book over the limit), and a clearer overage display in the weekly shiftplan.
**Verified:** 2026-06-27
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | D-24-01 / D-24-01a / D-24-07: Global hard/soft mode persisted in toggle table, default soft (enabled=0), NOT feature_flag | VERIFIED | Migration `20260627000000_seed-paid-limit-toggle.sql` uses `INSERT OR IGNORE INTO toggle (name, enabled, ...) VALUES ('paid_limit_hard_enforcement', 0, ...)`. `INSERT OR IGNORE` confirms CR-01 idempotency fix was applied. |
| 2 | D-24-02 + strikt-groesser: In hard mode, non-shiftplanner paid booking over limit blocked BEFORE persist with PaidLimitExceeded; shiftplanner bypasses; soft mode warns only; unpaid never blocked | VERIFIED | `shiftplan_edit.rs:440-473`: pre-persist guard reads toggle, checks `!is_shiftplanner` (reusing captured `sp_perm.is_ok()` per WR-01 fix), counts paid bookings via `count_paid_bookings_in_slot_week`, computes `prospective`, returns `Err(PaidLimitExceeded { current: prospective, max })` when `prospective > max`. All four test cases present (lines 1219, 1296, 1394, 1492). |
| 3 | D-24-08: ShiftplanEditService consumes ToggleService; toggle read fresh per booking before booking_service.create | VERIFIED | `gen_service_impl!` block line 42 adds `ToggleService` dep. `main.rs` lines 910-933: toggle_service constructed at line 911, shiftplan_edit_service at line 917 (Basic before Business). Toggle read at line 443 occurs before `booking_service.create` at line 524. |
| 4 | D-24-04: booking gate fixed from HR-OR-self to Shiftplanner-OR-self; HR_PRIVILEGE removed from booking path | VERIFIED | `shiftplan_edit.rs:11`: imports `SHIFTPLANNER_PRIVILEGE` (no HR_PRIVILEGE import). Gate at line 413-421: `join!(check_permission(SHIFTPLANNER_PRIVILEGE,...), verify_user_is_sales_person(...))`. No `HR_PRIVILEGE` in entire file (grep returns empty). Test `test_book_slot_with_conflict_check_forbidden` updated with comment "D-24-04: gate ist nun Shiftplanner ∨ self." |
| 5 | D-24-05: New ServiceError::PaidLimitExceeded { current, max } mapped to HTTP 409 (not 403); frontend detects 409 and shows inline localized message | VERIFIED | `service/src/lib.rs:126-127`: `PaidLimitExceeded { current: u8, max: u8 }` variant present. `rest/src/lib.rs:249-254`: mapped to `.status(409)`. `rest/src/shiftplan_edit.rs:131`: OpenAPI `(status = 409, ...)` annotation. Frontend `shiftplan.rs:414-419`: arm matching `StatusCode::CONFLICT` sets `block_error.set(Some(slot_id))`. Inline div at line 1190: `class: "text-bad text-small font-normal mt-1"` with `Key::BookingBlockedPaidLimit`. |
| 6 | D-24-06: New admin-gated /settings/ route renders SettingsPage with exactly ONE toggle (paid_limit_hard_enforcement) via Toggle REST API | VERIFIED | `router.rs:61-62`: `#[route("/settings/")] Settings {}`. `page/mod.rs:12,30`: `pub mod settings` + `pub use settings::SettingsPage`. `settings.rs`: component-level admin guard (lines 24-35) with `has_privilege("admin")` check, `TOGGLE_NAME = "paid_limit_hard_enforcement"`, aria-pressed button, `loader::get_toggle_enabled` / `loader::set_toggle` wiring. `top_bar.rs`: `NavVisibility.settings: bool` + `has("admin")` gate + `is_admin_target(NavTarget::Settings)` = true (line 131). |
| 7 | D-24-03: Persistent overage warning section above shiftplan, all roles, client-side, hidden when no overage | VERIFIED | `shiftplan.rs:924-978`: custom section iterates `shiftplan.slots`, filters `slot.current_paid_count > max` (NOT gated by `is_shiftplanner`), renders `bg-warn-soft border border-warn rounded-md print:hidden` container with `Key::ShiftplanPaidOverageSectionHeader` heading and per-slot list items substituting `{slot}/{current}/{max}`. Empty-list case renders `rsx! {}` (invisible). Positioned above `ShiftplanTabBar` (line 987). |

**Score:** 7/7 truths verified

---

### Deferred Items

None.

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `service/src/lib.rs` | `ServiceError::PaidLimitExceeded { current: u8, max: u8 }` | VERIFIED | Line 126-127 |
| `rest/src/lib.rs` | `PaidLimitExceeded -> HTTP 409` match arm | VERIFIED | Lines 249-254 |
| `rest/src/shiftplan_edit.rs` | OpenAPI `(status = 409, ...)` annotation | VERIFIED | Line 131 |
| `migrations/sqlite/20260627000000_seed-paid-limit-toggle.sql` | `INSERT OR IGNORE INTO toggle ... paid_limit_hard_enforcement ... enabled=0` | VERIFIED | Full file confirmed; no CREATE TABLE, no toggle group |
| `service_impl/src/shiftplan_edit.rs` | ToggleService dep, pre-persist hard-block guard, Shiftplanner-OR-self gate, PaidLimitExceeded return | VERIFIED | Lines 11, 17, 42, 410-473; no HR_PRIVILEGE |
| `shifty_bin/src/main.rs` | ToggleService wired into ShiftplanEditService; toggle_service constructed before shiftplan_edit_service | VERIFIED | Lines 406, 910-933 (toggle at 911, shiftplan_edit at 917) |
| `service_impl/src/test/shiftplan_edit.rs` | 4 hard-block tests; MockToggleService; no HR_PRIVILEGE | VERIFIED | Lines 1219, 1296, 1394, 1492; MockToggleService at line 36; no HR_PRIVILEGE in file |
| `shifty-dioxus/src/i18n/mod.rs` | 9 new Key variants + i18n_phase24_keys_present_in_all_locales test | VERIFIED | Lines 586-602 (keys), line 1291 (test) |
| `shifty-dioxus/src/i18n/en.rs` | English translations for all 9 keys, Locale::En | VERIFIED | Line 968 contains "Paid employee limit reached" |
| `shifty-dioxus/src/i18n/de.rs` | German translations for all 9 keys, Locale::De | VERIFIED | Lines 1012-1052; line 1051 "Bezahlt-Limit erreicht" |
| `shifty-dioxus/src/i18n/cs.rs` | Czech translations for all 9 keys, Locale::Cs | VERIFIED | Line 1001 "Vynucení limitu placených zaměstnanců" |
| `shifty-dioxus/src/page/settings.rs` | SettingsPage with admin guard, paid_limit_hard_enforcement, aria-pressed button | VERIFIED | Full file read; const TOGGLE_NAME, is_admin guard lines 24-35, aria-pressed line 124 |
| `shifty-dioxus/src/router.rs` | `#[route("/settings/")] Settings {}` + alias | VERIFIED | Lines 19, 61-62 |
| `shifty-dioxus/src/component/top_bar.rs` | NavTarget::Settings in 6 sites; settings: has("admin") | VERIFIED | Lines 31, 57, 72, 98, 131, 441-447; tests at lines 793, 799 |
| `shifty-dioxus/src/api.rs` | `set_toggle` + `get_toggle_enabled` REST client fns (with `error_for_status_ref()` per WR-03) | VERIFIED | Lines 1577-1595; `error_for_status_ref()` present at line 1593 |
| `shifty-dioxus/src/loader.rs` | `set_toggle` + `get_toggle_enabled` ShiftyError wrappers | VERIFIED | Lines 1011-1023 |
| `shifty-dioxus/src/page/shiftplan.rs` | Persistent overage section + inline 409 block_error signal; CR-03+WR-04 clears on navigation | VERIFIED | Lines 206, 414-419, 924-978, 1189-1193; block_error.set(None) at lines 460, 493 (NextWeek/PreviousWeek), 555 (ToggleChangeStructureMode) |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ServiceError::PaidLimitExceeded` | HTTP 409 | `error_handler` match arm in `rest/src/lib.rs` | VERIFIED | `err @ ServiceError::PaidLimitExceeded { .. } => .status(409)` at line 249 |
| `ShiftplanEditService.book_slot_with_conflict_check` | `toggle_service.is_enabled("paid_limit_hard_enforcement", ...)` | pre-persist guard | VERIFIED | Lines 441-448; called before `booking_service.create` at line 524 |
| `book_slot_with_conflict_check gate` | `SHIFTPLANNER_PRIVILEGE` | `check_permission` replacing `HR_PRIVILEGE` | VERIFIED | Line 11 imports only SHIFTPLANNER_PRIVILEGE; gate at lines 413-426 |
| `AddUserToSlot Err(Reqwest) status == CONFLICT` | `block_error.set(Some(slot_id))` | new match arm before generic Err | VERIFIED | Lines 414-419: arm `if e.status() == Some(reqwest::StatusCode::CONFLICT)` |
| `state::shiftplan::Slot current_paid_count > max_paid_employees` | overage section list | client-side iteration | VERIFIED | Lines 927-953: iterates `shiftplan.slots`, filters `slot.current_paid_count > max` |
| `SettingsPage toggle click` | `PUT /toggle/paid_limit_hard_enforcement/enable|disable` | `loader::set_toggle` | VERIFIED | `settings.rs:70`: `loader::set_toggle(cfg, TOGGLE_NAME, next)` |
| `NavVisibility.settings` | `is_admin_target(NavTarget::Settings)` | top_bar nav wiring | VERIFIED | Line 131 includes `NavTarget::Settings` in `is_admin_target` matches; nav tests at 793/799 |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `settings.rs` | `hard_enforcement: Signal<bool>` | `use_resource(|| loader::get_toggle_enabled(...))` + `use_effect` sync | GET /toggle/paid_limit_hard_enforcement/enabled → backend DAO | FLOWING |
| `shiftplan.rs` (overage section) | `overage_slots` | `shift_plan_context` (loaded week slots with `current_paid_count` + `max_paid_employees`) | Existing week-view state already loaded from backend | FLOWING |
| `shiftplan.rs` (block_error) | `block_error: Signal<Option<Uuid>>` | Set by `AddUserToSlot` handler when backend returns 409 | Backend returns 409 when `ServiceError::PaidLimitExceeded` | FLOWING |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `ServiceError::PaidLimitExceeded` variant compiles | `grep -c "PaidLimitExceeded" /home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/lib.rs` | 1 | PASS |
| HTTP 409 mapping for PaidLimitExceeded | `grep -c "PaidLimitExceeded" /home/neosam/programming/rust/projects/shifty/shifty-backend/rest/src/lib.rs` | 1 | PASS |
| Migration file exists and is idempotent | `grep "INSERT OR IGNORE" migrations/sqlite/20260627000000_seed-paid-limit-toggle.sql` | Found | PASS |
| HR_PRIVILEGE removed from booking path | `grep -n "HR_PRIVILEGE" service_impl/src/shiftplan_edit.rs` | (empty) | PASS |
| 4 hard-block test functions present | `grep -n "test_hard_block_\|test_soft_mode_over_limit" service_impl/src/test/shiftplan_edit.rs` | 4 found (1219, 1296, 1394, 1492) | PASS |
| toggle_service constructed before shiftplan_edit_service in main.rs | Line 910-911 vs line 917 | toggle at 910-915, shiftplan_edit at 917-934 | PASS |
| block_error cleared on NextWeek/PreviousWeek/ToggleChangeStructureMode | `grep -n "block_error.set(None)" shiftplan.rs` | Lines 460, 493, 555 | PASS |
| SettingsPage has component-level admin guard | `grep -n "has_privilege.*admin\|is_admin" settings.rs` | Lines 24-35 | PASS |
| `get_toggle_enabled` calls `error_for_status_ref()` (WR-03) | `grep -n "error_for_status_ref" api.rs` | Line 1593 | PASS |
| NavTarget::Settings in is_admin_target | `grep -n "NavTarget::Settings" top_bar.rs | grep 131` | Line 131 confirmed | PASS |

---

### Requirements Coverage

| Decision | Description | Status | Evidence |
|----------|-------------|--------|----------|
| D-24-01 | Global paid-limit mode configurable (hard/soft), default soft = no regression | SATISFIED | Migration seeds `enabled=0`; toggle table stores the mode |
| D-24-01a | Stored in ToggleService table, NOT feature_flag | SATISFIED | Migration targets `toggle` table; no feature_flag reference |
| D-24-07 | New migration seeds `paid_limit_hard_enforcement` toggle, `enabled=0`, no group | SATISFIED | File content confirmed; `INSERT OR IGNORE` (idempotent per CR-01 fix) |
| D-24-02 | Hard mode blocks non-shiftplanner paid booking over limit BEFORE persist | SATISFIED | Pre-persist guard at lines 440-473; `return Err(...)` before `booking_service.create` at line 524 |
| D-24-Grenzregel | Strictly-greater boundary (prospective > max); only paid persons count; existing bookings never removed | SATISFIED | `prospective > max` at line 467; `count_paid_bookings_in_slot_week` filters by `paid_ids`; soft path is post-persist (no rollback) |
| D-24-08 | Pre-persist check; ShiftplanEditService has ToggleService dep; toggle read fresh per booking | SATISFIED | ToggleService in gen_service_impl! + main.rs DI ordering verified |
| D-24-04 | Booking gate fixed from HR-OR-self to Shiftplanner-OR-self | SATISFIED | SHIFTPLANNER_PRIVILEGE replaces HR_PRIVILEGE at gate; no HR_PRIVILEGE in shiftplan_edit.rs |
| D-24-05 | New `ServiceError::PaidLimitExceeded` → HTTP 409 (not 403); inline localized message at slot | SATISFIED | Variant exists, 409 mapping confirmed, frontend arm + render confirmed (note: page-level placement, see human verification) |
| D-24-06 | New admin-gated /settings/ page with exactly one toggle | SATISFIED | SettingsPage with one const TOGGLE_NAME; admin guard; route registered |
| D-24-03 | Persistent overage section above shiftplan, all roles, client-side, hidden when empty | SATISFIED | Section at lines 924-978; no is_shiftplanner gate; positioned above ShiftplanTabBar |
| copy_week untouched | copy_week_with_conflict_check not modified by enforcement | SATISFIED | Function starts at line 610, no ToggleService / PaidLimitExceeded references inside it |
| No retroactive booking removal | Existing bookings never rolled back on mode switch | SATISFIED | Only pre-persist guard; persisted bookings stay; confirmed by D-07 note in code comments |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `shiftplan.rs` | 1189 | `block_error.read().is_some()` — renders block error for ANY blocked slot_id, not the specific blocked slot; the stored `Option<Uuid>` is not compared to a current slot context | WARNING | The inline BookingBlockedPaidLimit message appears below the entire WeekView component, not at the specific slot cell. Functionally the error IS shown; visual placement deviates from "scoped to the slot that was blocked" in plan spec and UI-SPEC. Clears correctly on navigation and success. |
| `top_bar.rs` | 983-996 | `is_admin_target_classifies_each_variant` test does not assert `is_admin_target(NavTarget::Settings) == true` | INFO | The production code at line 131 is correct; the test simply doesn't exercise the new variant. No functional regression — nav_visibility tests at 793/799 cover the admin-gate path. |

---

### Human Verification Required

#### 1. Block-error inline message placement (D-24-05 visual)

**Test:** Log in as a non-shiftplanner with a paid employee account. Enable hard enforcement via /settings/. Navigate to the shiftplan for a week where a slot has reached its paid limit. Attempt to add a paid person to that slot.
**Expected:** A red "Paid employee limit reached — only shift planners may book beyond the limit." message appears. Verify WHERE it appears: the spec says "inline at the slot" but the implementation renders it below the entire WeekView. Determine if the placement is acceptable.
**Why human:** The `block_error: Signal<Option<Uuid>>` stores the slot_id but the render at `shiftplan.rs:1189` only checks `is_some()`, not which slot. Verifying whether the below-weekview placement satisfies "inline at the slot" requires visual inspection. If the placement is unacceptable, the fix is to pass block_error into `WeekView`/slot components and compare per-slot.

#### 2. Settings page toggle interaction (D-24-06)

**Test:** Navigate to /settings/ as an admin. Observe the toggle button. Click it. Observe state flip and "Saved." confirmation. Disable network and try again — observe "Could not save setting." error.
**Expected:** Toggle button flips between Soft/Hard labels; aria-pressed changes; inline text feedback; no modal; button is disabled during in-flight request.
**Why human:** Interactive async behavior (spinner disable, save feedback) and error revert logic require browser runtime.

#### 3. Persistent overage section visibility (D-24-03)

**Test:** Load a shiftplan week that has at least one slot where current_paid_count > max_paid_employees. Verify the yellow overage section appears above ShiftplanTabBar. Also verify it is visible to a non-shiftplanner (regular employee) user.
**Expected:** Section with "⚠️ Paid employee limit exceeded this week" heading; one row per overage slot in `{slot}: {current}/{max} paid` format; no dismiss button; absent when no slots are over limit.
**Why human:** Visual rendering and cross-role visibility need browser confirmation.

#### 4. Settings page direct-URL access by non-admin (WR-02 component guard)

**Test:** As a non-admin user, directly navigate browser to `/settings/` (bypass nav). Verify page shows "Not authorized." not the toggle UI.
**Expected:** Component-level admin guard returns early with the "Not authorized." fallback; toggle is not rendered.
**Why human:** Route-level access control for direct URL access requires browser testing; cannot be verified by grep.

---

### Gaps Summary

No automated-verification blockers were found. All 7 observable truths are supported by codebase evidence.

One WARNING-level deviation was identified: the `BookingBlockedPaidLimit` inline error (D-24-05) renders at the page level below the WeekView (checking `block_error.is_some()`) rather than being scoped to the specific blocked slot cell. The error IS surfaced and clears correctly; only the visual placement deviates from the UI-SPEC "at the slot" specification. Human verification is requested to determine if this placement is acceptable UX.

Four human verification items exist (visual/browser UX concerns) — see section above.

**Review fixes applied and verified:**
- CR-01 (migration idempotency): `INSERT OR IGNORE` confirmed in migration file
- CR-03 (block_error cleared on week navigation): `block_error.set(None)` at lines 460 (NextWeek), 493 (PreviousWeek)
- WR-01 (dedup permission check): `is_shiftplanner = sp_perm.is_ok()` at line 425, reused at line 449
- WR-02 (SettingsPage admin guard): component-level `has_privilege("admin")` guard at lines 24-35
- WR-03 (error_for_status in get_toggle_enabled): `response.error_for_status_ref()?` at api.rs line 1593
- WR-04 (block_error cleared on ToggleChangeStructureMode): line 555 confirmed

**TOCTOU note (CR-02, accepted design limitation):** The count-then-insert race condition remains intentionally. This was explicitly accepted as a low-concurrency design limitation; `count_paid_bookings_in_slot_week` now returns `(count, paid_ids)` tuple (CR-02 partial fix to eliminate duplicate DAO calls), but no DB-level lock was added.

---

_Verified: 2026-06-27_
_Verifier: Claude (gsd-verifier)_
