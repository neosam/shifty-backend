---
phase: 24-paid-capacity-enforcement-config
reviewed: 2026-06-27T00:00:00Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - service/src/lib.rs
  - rest/src/lib.rs
  - rest/src/shiftplan_edit.rs
  - migrations/sqlite/20260627000000_seed-paid-limit-toggle.sql
  - service_impl/src/shiftplan_edit.rs
  - service_impl/src/test/shiftplan_edit.rs
  - shifty_bin/src/main.rs
  - shifty-dioxus/src/i18n/mod.rs
  - shifty-dioxus/src/i18n/en.rs
  - shifty-dioxus/src/i18n/de.rs
  - shifty-dioxus/src/i18n/cs.rs
  - shifty-dioxus/src/page/settings.rs
  - shifty-dioxus/src/page/mod.rs
  - shifty-dioxus/src/router.rs
  - shifty-dioxus/src/component/top_bar.rs
  - shifty-dioxus/src/api.rs
  - shifty-dioxus/src/loader.rs
  - shifty-dioxus/src/page/shiftplan.rs
findings:
  critical: 3
  warning: 4
  info: 2
  total: 9
status: issues_found
---

# Phase 24: Code Review Report

**Reviewed:** 2026-06-27
**Depth:** standard
**Files Reviewed:** 18
**Status:** issues_found

## Summary

Phase 24 adds paid-capacity hard enforcement via a `paid_limit_hard_enforcement` toggle. The backend adds `ServiceError::PaidLimitExceeded` (HTTP 409), a seed migration, and a pre-persist block guard inside `book_slot_with_conflict_check`. The frontend adds an admin-gated Settings page and inline 409 surfacing.

The implementation has three blockers: (1) the migration is not idempotent and will error on redeployment; (2) the hard-enforcement guard makes two redundant round-trips per booking (full week + paid persons fetched separately for the guard, then fetched again for the soft-warning path); and (3) `block_error` is not cleared when navigating to a different week, leaving the "paid limit reached" banner stale. There are also four warnings around the Settings page lacking its own privilege gate in the route, a double permission call in the guard, a missing `'static` lifetime in `api.rs`, and an unguarded `is_admin_target` classification that may need attention.

---

## Critical Issues

### CR-01: Migration is not idempotent — fails on re-run or re-deployment

**File:** `migrations/sqlite/20260627000000_seed-paid-limit-toggle.sql:1`

**Issue:** The migration uses a plain `INSERT INTO toggle (...) VALUES (...)` with no `ON CONFLICT` clause or `INSERT OR IGNORE`. SQLx's migrate runner marks each migration as applied in the `_sqlx_migrations` table after the first run, so on a fresh DB this works once. However, in any scenario where the migration is run twice (e.g., a developer drops and recreates only the migrations table, rolls back to re-apply, runs `sqlx migrate run` against a database that already has the row from a manual seed, or CI uses a shared DB) the statement will produce `SQLITE_CONSTRAINT_UNIQUE` if `name` is a primary/unique key. The correct form is `INSERT OR IGNORE` (SQLite) or `INSERT INTO ... ON CONFLICT DO NOTHING`.

**Fix:**
```sql
INSERT OR IGNORE INTO toggle (name, enabled, description, update_process)
VALUES (
    'paid_limit_hard_enforcement',
    0,
    'When ON, booking over a slot/week paid-employee limit is blocked for non-shiftplanners. Default OFF (soft, warning-only).',
    'phase-24-migration'
);
```

---

### CR-02: Hard-enforcement guard calls `count_paid_bookings_in_slot_week` (pre-persist) then calls the same two DAOs again in the soft-warning path (post-persist) — logic inconsistency between pre- and post-persist counts

**File:** `service_impl/src/shiftplan_edit.rs:436-483` and `service_impl/src/shiftplan_edit.rs:589-608`

**Issue:** When hard-enforcement is enabled **and** the actor is a shiftplanner, the guard is bypassed and the booking is persisted. Then the soft-warning path runs and calls `count_paid_bookings_in_slot_week` again with the same `(slot_id, year, week)`. This second call fetches `get_for_week` + `get_all_paid` from the DB — these are now post-persist counts (the booking is already in the DB). This is correct for the soft-warning case.

The real bug is in the hard-enforcement guard: `count_paid_bookings_in_slot_week` at line 455 fetches **all bookings for the week** (`get_for_week`) and then filters by `slot_id`. This is the same query used by the soft-warning path. There is no problem with correctness in isolation, **but** when enforcement is ON and the actor is **not** a shiftplanner, the guard invokes `count_paid_bookings_in_slot_week` (which calls `get_for_week` + `get_all_paid`) and also separately calls `sales_person_service.get_all_paid` (line 466-468) to determine `booked_is_paid`. This means `get_all_paid` is called twice for the same transaction: once inside `count_paid_bookings_in_slot_week` (line 718) and once at line 466. The returned data is the same, but it imposes an unnecessary additional DAO round-trip and makes the logic harder to maintain/audit.

More critically: the `booked_is_paid` determination (lines 466-471) and the `existing_paid` count (lines 455-463) each go to the DAO separately. If the booking service is ever called concurrently (e.g., two simultaneous requests), both reads could return a count of `N` and both could succeed, resulting in `N+2` paid employees where only `N+1` is allowed. This is a TOCTOU (time-of-check/time-of-use) race: the guard reads a count, returns `Ok`, and then the insert happens separately — no row-level lock covers this gap in SQLite WAL mode.

**Fix (minimum viable):** Unify the paid-person lookup: pass the `paid_persons` list from `count_paid_bookings_in_slot_week` back to the caller (or restructure the helper to return `(count, paid_person_ids)` so `booked_is_paid` can be computed from the same fetch):

```rust
// Return (count, paid_ids_set) from the helper to avoid the second get_all_paid call.
async fn count_paid_bookings_in_slot_week(
    &self, slot_id: Uuid, year: u32, week: u8, tx: Deps::Transaction,
) -> Result<(u8, std::collections::HashSet<Uuid>), ServiceError> {
    let bookings = ...;
    let paid_persons = ...;
    let paid_ids: HashSet<Uuid> = paid_persons.iter().map(|sp| sp.id).collect();
    let count = bookings.iter()
        .filter(|b| b.slot_id == slot_id && b.deleted.is_none())
        .filter(|b| paid_ids.contains(&b.sales_person_id))
        .count();
    Ok((count.min(u8::MAX as usize) as u8, paid_ids))
}
```

The TOCTOU issue requires a serialized write path (e.g., a DB-level UNIQUE constraint or an advisory lock), which is a broader architectural concern.

---

### CR-03: `block_error` signal is never cleared on week/year navigation — stale 409 banner persists across week changes

**File:** `shifty-dioxus/src/page/shiftplan.rs:487-488` (NextWeek) and `shifty-dioxus/src/page/shiftplan.rs:519-520` (PreviousWeek)

**Issue:** The `block_error: Signal<Option<Uuid>>` signal (line 206) is cleared on a successful booking (line 427) and on a removal action (line 460). However, when the user navigates to the next or previous week (actions `NextWeek` and `PreviousWeek`), `block_error` is not reset. If a user hits a 409 for a slot in week 10, then navigates to week 11, the hard-block banner for that slot_id will still be rendered. In week 11, a slot with the same UUID may or may not exist; if it does, the user sees a false "Paid limit reached" warning for a slot that was not blocked in week 11.

`booking_warnings` is correctly cleared on navigation (line 487 and 519), but `block_error` is not.

**Fix:**
```rust
ShiftPlanAction::NextWeek => {
    ...
    booking_warnings.set(WarningsList::empty());
    block_error.set(None);  // <-- add this line
    update_shiftplan();
    ...
}
ShiftPlanAction::PreviousWeek => {
    ...
    booking_warnings.set(WarningsList::empty());
    block_error.set(None);  // <-- add this line
    update_shiftplan();
    ...
}
```

---

## Warnings

### WR-01: Double `check_permission(SHIFTPLANNER_PRIVILEGE)` call — permission checked at booking gate and again inside the enforcement guard

**File:** `service_impl/src/shiftplan_edit.rs:413-421` and `service_impl/src/shiftplan_edit.rs:446-450`

**Issue:** `book_slot_with_conflict_check` first calls `check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())` as part of the parallel permission gate (lines 413-421: shiftplanner OR self). Then, inside the hard-enforcement guard (lines 446-450), it calls `check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())` again to decide whether to bypass the block. This is redundant: if the outer gate result from the parallel join (`sp_perm`) is `Ok(())`, the actor is already known to be a shiftplanner. The result is discarded and re-queried.

This is wasteful (extra DAO round-trip on every booking when enforcement is enabled) and creates a subtle correctness risk: the two calls use the same `context`, so they should produce the same result, but if the permission system has any caching subtleties or the future introduction of context mutation between calls changes that, the two checks could diverge.

**Fix:** Capture the result of the first shiftplanner check and reuse it:
```rust
let (sp_perm, self_perm) = join!(
    self.permission_service.check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
    self.sales_person_service.verify_user_is_sales_person(...),
);
let is_shiftplanner = sp_perm.is_ok();
sp_perm.or(self_perm)?;

// Inside enforcement guard:
if !is_shiftplanner {
    // ... enforcement check ...
}
```

---

### WR-02: `SettingsPage` route has no server-side access gate — client-side `settings: has("admin")` nav hide is the only barrier

**File:** `shifty-dioxus/src/router.rs:61-62` and `shifty-dioxus/src/component/top_bar.rs:57`

**Issue:** The `/settings/` route is registered in the Dioxus router without any privilege check inside `SettingsPage` itself. The nav item is hidden for non-admins in `nav_visibility` (top_bar.rs:57: `settings: has("admin")`), but a user who knows the URL can navigate directly to `/settings/` and see the toggle UI. The `PUT /toggle/{name}/enable` backend endpoint is presumably protected by the toggle service's own permission check, so the backend is safe — but the frontend renders a functional-looking toggle page for users who should not have access to it.

More importantly, if `loader::get_toggle_enabled` or `loader::set_toggle` do not validate server-side privilege before returning data, a non-admin user who navigates directly to `/settings/` would see the current state of the toggle and could attempt to flip it.

**Fix:** Add a privilege guard inside `SettingsPage`:
```rust
let auth_info = AUTH.read().auth_info.clone();
let is_admin = auth_info.as_ref()
    .map(|a| a.has_privilege("admin"))
    .unwrap_or(false);
if !is_admin {
    return rsx! { /* 403 / redirect */ };
}
```

---

### WR-03: `api::get_toggle_enabled` does not call `error_for_status_ref()` — a non-200 HTTP response is silently deserialized as JSON, producing a misleading error

**File:** `shifty-dioxus/src/api.rs:1590-1593`

**Issue:**
```rust
pub async fn get_toggle_enabled(config: Config, name: &str) -> Result<bool, reqwest::Error> {
    let url = format!("{}/toggle/{}/enabled", config.backend, name);
    Ok(reqwest::get(url).await?.json().await?)
}
```

If the backend returns 401, 403, or 500, the function attempts to deserialize the error body as `bool`, which will produce a `reqwest::Error` with kind `Decode` rather than an HTTP-status error. The caller in `SettingsPage` (via `loader::get_toggle_enabled`) matches `Some(Ok(enabled))` and falls through to the `None` arm on error, silently defaulting to `false` (hard enforcement OFF). An admin who cannot load the toggle state would not know it failed; the page would silently show "Soft mode" even if the server actually has hard mode enabled.

**Fix:**
```rust
pub async fn get_toggle_enabled(config: Config, name: &str) -> Result<bool, reqwest::Error> {
    let url = format!("{}/toggle/{}/enabled", config.backend, name);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    Ok(response.json().await?)
}
```

---

### WR-04: `block_error` is not cleared when `ToggleChangeStructureMode` is entered — stale 409 banner visible while editing slot structure

**File:** `shifty-dioxus/src/page/shiftplan.rs:200-206` and `ShiftPlanAction::ToggleChangeStructureMode` handler

**Issue:** When the user switches into "change structure mode" (slot editing), the `block_error` signal is not cleared. If a 409 hard-block was triggered just before entering structure mode, the inline error banner for that slot remains visible in the slot editor view. Since the slot editor does not surface bookings, this banner is misleading — the user cannot take any action to resolve it from the slot editor.

**Fix:** Clear `block_error` in the `ToggleChangeStructureMode` handler:
```rust
ShiftPlanAction::ToggleChangeStructureMode => {
    block_error.set(None);
    change_structure_mode.set(!*change_structure_mode.read());
}
```

---

## Info

### IN-01: `i18n` test suite does not cover Phase 24 placeholder substitution for `ShiftplanPaidOverageRow` and `BookingBlockedPaidLimit`

**File:** `shifty-dioxus/src/i18n/mod.rs:1291-1315`

**Issue:** The `i18n_phase24_keys_present_in_all_locales` test (line 1291) only asserts that each Phase 24 key is non-empty and not `"??"`. It does not verify that `ShiftplanPaidOverageRow` (which has `{slot}`, `{current}`, `{max}` placeholders) substitutes correctly via `t_m`, matching the pattern established for other placeholder-bearing keys (e.g., `shiftplan_filled_of_need_substitutes_placeholders` at line 757, `shiftplan_delete_confirm_body_interpolates_name` at line 932). If a future locale accidentally drops a placeholder or misspells it, the existing test will not catch it.

**Fix:** Add placeholder-substitution tests for the two Phase 24 keys that use `{...}` placeholders:
```rust
#[test]
fn i18n_shiftplan_paid_overage_row_substitutes_placeholders() {
    for locale in [Locale::En, Locale::De, Locale::Cs] {
        let i18n = generate(locale);
        let result = i18n.t_m(
            Key::ShiftplanPaidOverageRow,
            [("slot", "Mon 09:00"), ("current", "3"), ("max", "2")].into(),
        );
        assert!(result.contains("Mon 09:00") && result.contains('3') && result.contains('2'),
            "missing substituted values in {:?}: got `{}`", locale, result);
    }
}
```

---

### IN-02: Commented-out code block in `shiftplan.rs` (dead code)

**File:** `shifty-dioxus/src/page/shiftplan.rs:267-276` and `390-405`

**Issue:** There are two substantial blocks of commented-out Rust code (`//let (current_sales_person, ...` at lines 267-276 and `//if let Some(sales_person) = sales_person` at lines 390-405). These are left over from a refactor. They make the file harder to read and could confuse future contributors about the intended data-flow.

**Fix:** Remove the dead commented-out blocks; the surrounding live code makes the intent clear.

---

_Reviewed: 2026-06-27_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
