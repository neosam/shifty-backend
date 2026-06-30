# Phase 35: Slot-Werte nur für eine Woche ändern - Research

**Researched:** 2026-06-30
**Domain:** Rust backend service extension (ShiftplanEditService) + Dioxus frontend UI mode toggle
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-35-01 (Approach B: Split + Re-Merge):** "only this week" = same cut as `modify_slot` today PLUS a third segment restoring original values from KW+1 on. Three slot versions: Segment 1 (original `valid_to = Sunday KW-1`), Segment 2 (exception `valid_from = Monday KW`, `valid_to = Sunday KW`, new values), Segment 3 (restore `valid_from = Monday KW+1`, original values, original `valid_to`). Approach A (new override data model) explicitly rejected.
- **D-35-02 (UI choice, Approach C):** Explicit mode switch in slot editor — "only this week" vs "from this week". "from this week" = current 2-segment behavior. "only this week" = D-35-01 (3 segments). Concrete UI element (toggle/radio) = Claude's Discretion.
- **D-35-03 (Split bookings):** Bookings fetched by `get_for_slot_id_since(change_week)` split into two groups: `calendar_week == change_week` → Segment 2; `calendar_week > change_week` (or year > change_year) → Segment 3. Same delete+create re-point pattern per booking.
- **D-35-04 (One transaction):** The existing `use_transaction`/`commit` bracket in `modify_slot` is reused. Third segment + split re-points stay within the same `tx`. No intermediate commits.
- **D-35-05 (Mandatory re-point/no-double-count tests):** Mandatory tests in `service_impl/src/test/shiftplan_edit.rs`. Minimum: 3-segment structure correctness, each booking lands exactly once on correct segment, report/balance non-duplication, rollback, edge cases (first KW, unbounded valid_to, no bookings in exception KW).
- **D-35-06 (Gate `shiftplan.edit`):** Consistent with `modify_slot` today.

### Claude's Discretion

- Backend wiring: new `single_week: bool` param on `modify_slot` vs separate `modify_slot_single_week` method + REST route.
- Editor UI layout for mode switch; state field in `SlotEdit`/`SlotEditItem`.
- Exact date arithmetic for KW boundaries (ISO week Monday/Sunday) — reuse helpers from `modify_slot`.
- i18n text (de/en/cs) for the mode choice and any hint text.

### Deferred Ideas (OUT OF SCOPE)

- Approach A (week-specific override data model) — rejected, potential future re-evaluation if many exceptions + cleanup needed.
- Merging/cleanup of redundant slot versions — future idea, not in v1.10.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SWO-01 | Mode switch in slot editor — "only this week" vs "from this week" | Frontend: new `single_week` field in `SlotEdit` state; toggle/radio in `slot_edit.rs` component; new i18n keys |
| SWO-02 | Mechanics = Split + Re-Merge on `modify_slot` — three slot versions, booking split | Backend: extend `modify_slot` in `service_impl/src/shiftplan_edit.rs:51`; create Segment 3 + partition booking loop |
| SWO-03 | One transaction, full rollback on error | Reuse `use_transaction`/`commit` bracket (lines 59+141); no intermediate commits |
| SWO-04 | Hard tests — no double-count, no orphan bookings; `shiftplan.edit` gate; i18n de/en/cs | Test base: `service_impl/src/test/shiftplan_edit.rs`; permission already at line 61 |
</phase_requirements>

---

## Summary

Phase 35 extends the existing `ShiftplanEditService::modify_slot` with a third "restore" segment so that a Shiftplanner can change slot values for exactly one calendar week as a one-time exception. The entire mechanism is already atomic and proven; the extension is surgical: (1) close Segment 2 at Sunday KW instead of leaving it unbounded, (2) create Segment 3 with original values valid from Monday KW+1, (3) partition the existing booking re-point loop by `calendar_week == change_week` vs `> change_week`. The frontend adds a mode toggle ("only this week" / "from this week") to the existing slot editor dialog and routes the save action to the appropriate API call.

No new data models, no schema migrations, no `cargo sqlx prepare` changes (no new SQL), no snapshot schema version bump (slot versioning does not touch `BillingPeriodValueType`). All six ASVS-relevant controls (auth gate, transaction, soft-delete filter) are already in place and merely need extension.

**Primary recommendation:** Implement as a new `modify_slot_single_week` service method (separate from `modify_slot`) plus a new REST route `PUT /shiftplan-edit/slot/{year}/{week}/single-week`. This keeps the existing `modify_slot` signature and its mock tests completely unchanged, avoids a bool-flag in the service trait, and makes the API intent explicit for future callers.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| 3-segment slot split (create/update/delete) | API/Backend — `ShiftplanEditService` | — | Mutation of slot versioning rows; owns transaction boundary |
| Booking re-point (split by week) | API/Backend — `ShiftplanEditService` | — | Must stay inside same transaction as slot split to guarantee atomicity |
| ISO-week date arithmetic (Monday/Sunday boundaries) | API/Backend | Frontend (display only) | Authoritative calculation is in backend; frontend only computes `valid_from` for new-slot creation (existing pattern) |
| Mode toggle UI ("only this week" / "from this week") | Frontend (Dioxus) | — | Pure UI state; routes to one of two REST calls |
| Permission gate (`shiftplan.edit`) | API/Backend | Frontend (soft-gate, hides UI) | Hard gate on backend; frontend shows/hides editor per privilege already |
| i18n text for new mode toggle | Frontend (Dioxus) | — | Three-locale keys in `src/i18n/{de,en,cs}.rs` |

---

## Standard Stack

No new external libraries. Phase is pure Rust internal extension.

### Core (already in workspace)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `time` crate | workspace pin | ISO-week date arithmetic (`from_iso_week_date`, `Duration::days`) | Already used in `modify_slot` at lines 78-79; no import needed |
| `uuid` | workspace pin | `Uuid::nil()` for new slot/booking IDs | Already used |
| `mockall` | workspace pin | Mock service for unit tests | `ShiftplanEditDependencies` already uses it |
| `async-trait` | workspace pin | Trait method signatures | `ShiftplanEditService` is already `#[async_trait]` |

### No Installation Required

```bash
# No new packages — all dependencies already present in Cargo workspace
cargo build  # verify workspace compiles
```

---

## Package Legitimacy Audit

> **Skipped** — Phase installs zero external packages. All code uses existing workspace dependencies.

---

## Architecture Patterns

### System Architecture Diagram

```
Frontend (Dioxus WASM)
  SlotEdit component
    Mode Toggle ("nur diese Woche" | "ab dieser Woche")
          │
          │ SlotEditAction::SaveSlot (with mode)
          ▼
  save_slot_edit() / new save_slot_single_week()
          │
          │ PUT /shiftplan-edit/slot/{year}/{week}        (existing — "from this week")
          │ PUT /shiftplan-edit/slot/{year}/{week}/single-week  (new — "only this week")
          ▼
Backend REST layer (rest/src/shiftplan_edit.rs)
  edit_slot handler (existing)
  edit_slot_single_week handler (new)
          │
          ▼
ShiftplanEditService
  modify_slot()          (existing — 2 segments)
  modify_slot_single_week()   (new — 3 segments)
          │
  ┌───────┴────────────────────────────────────────┐
  │  use_transaction(tx) ─── ONE tx bracket ────   │
  │                                                 │
  │  1. get_slot → check version                    │
  │  2. get_for_slot_id_since(change_week) → bookings│
  │  3. Segment 1: update_slot (valid_to=Sun KW-1)  │
  │     OR delete_slot if Seg1 would be empty       │
  │  4. Segment 2: create_slot (Mon KW→Sun KW, new) │
  │  5. Segment 3: create_slot (Mon KW+1→orig, old) │◄─ NEW
  │  6. For each booking:                           │
  │       delete(old_booking)                       │
  │       create(new_booking, slot_id=seg2 or seg3) │◄─ SPLIT NEW
  │  7. commit(tx)                                  │
  └─────────────────────────────────────────────────┘
          │
          ▼
SlotService (Basic tier) → DAO → SQLite
BookingService (Basic tier) → DAO → SQLite (deleted IS NULL filter everywhere)
```

### Recommended File Changes

```
service_impl/src/shiftplan_edit.rs        # Add modify_slot_single_week() method
service/src/shiftplan_edit.rs             # Add trait method + update #[automock]
rest/src/shiftplan_edit.rs                # Add edit_slot_single_week handler + route
service_impl/src/test/shiftplan_edit.rs   # Add D-35-05 mandatory tests

shifty-dioxus/src/state/slot_edit.rs      # Add single_week: bool to SlotEdit
shifty-dioxus/src/service/slot_edit.rs    # Route SaveSlot to correct API
shifty-dioxus/src/component/slot_edit.rs  # Add mode toggle UI
shifty-dioxus/src/api.rs                  # Add update_slot_single_week()
shifty-dioxus/src/i18n/mod.rs             # New Key variants
shifty-dioxus/src/i18n/de.rs              # German translations
shifty-dioxus/src/i18n/en.rs              # English translations
shifty-dioxus/src/i18n/cs.rs              # Czech translations
```

### Pattern 1: modify_slot Today — Exact Code Flow

**What:** Two-segment "from this week" split, atomic, in `service_impl/src/shiftplan_edit.rs` lines 51-143. [VERIFIED: codebase read]

```rust
// Source: service_impl/src/shiftplan_edit.rs:51-143

// ATOMICITY BRACKET — line 59
let tx = self.transaction_dao.use_transaction(tx).await?;

// PERMISSION GATE — line 61
self.permission_service
    .check_permission("shiftplan.edit", context)
    .await?;

// VERSION CONFLICT CHECK — lines 64-75
let mut stored_slot = self.slot_service
    .get_slot(&slot.id, Authentication::Full, tx.clone().into())
    .await?;
if stored_slot.version != slot.version {
    return Err(ServiceError::EntityConflicts(...));
}

// DATE ARITHMETIC — lines 77-79
let new_slot_valid_from =
    time::Date::from_iso_week_date(change_year as i32, change_week, time::Weekday::Monday)?;
let old_slot_valid_to = new_slot_valid_from - time::Duration::days(1);  // Sunday KW-1

// BOOKINGS SINCE CHANGE_WEEK — lines 80-89
let bookings = self.booking_service
    .get_for_slot_id_since(slot.id, change_year, change_week, ...)
    .await?;

// SAVE ORIGINAL VALID_TO (before mutating stored_slot) — line 90
let original_valid_to = stored_slot.valid_to;

// SEGMENT 1 SHRINK — lines 92-102
stored_slot.valid_to = Some(old_slot_valid_to);
if stored_slot.valid_to.unwrap() < stored_slot.valid_from {
    self.slot_service.delete_slot(&stored_slot.id, ...).await?;  // first KW edge case
} else {
    self.slot_service.update_slot(&stored_slot, ...).await?;
}

// SEGMENT 2 CREATE — lines 104-117
let mut new_slot = stored_slot;   // copies remaining fields (from, to, min_resources, etc.)
new_slot.valid_from = new_slot_valid_from;   // Monday KW
new_slot.valid_to = original_valid_to;       // UNBOUNDED (existing behavior)
new_slot.id = Uuid::nil();
new_slot.version = Uuid::nil();
new_slot.min_resources = slot.min_resources;       // from INPUT slot
new_slot.max_paid_employees = slot.max_paid_employees;  // from INPUT slot
new_slot.from = slot.from;
new_slot.to = slot.to;
let new_slot = self.slot_service.create_slot(&new_slot, ...).await?;

// BOOKING RE-POINT — lines 119-139 (ALL bookings → new_slot.id)
for booking in bookings.iter() {
    self.booking_service.delete(booking.id, ...).await?;
    let mut new_booking = booking.clone();
    new_booking.id = Uuid::nil();
    new_booking.version = Uuid::nil();
    new_booking.slot_id = new_slot.id;          // <- target segment
    new_booking.created = None;
    new_booking.created_by = None;              // system stamp; original in soft-deleted row
    self.booking_service.create(&new_booking, ...).await?;
}

// SINGLE COMMIT — line 141
self.transaction_dao.commit(tx).await?;
```

### Pattern 2: modify_slot_single_week — New Logic (3 Segments)

**What:** Extension of Pattern 1 with closed Segment 2, new Segment 3, split booking loop. [ASSUMED — design derived from codebase analysis; not yet implemented]

```rust
// Source: derived from service_impl/src/shiftplan_edit.rs:51-143

async fn modify_slot_single_week(
    &self,
    slot: &Slot,
    change_year: u32,
    change_week: u8,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Slot, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    self.permission_service
        .check_permission("shiftplan.edit", context)
        .await?;

    let mut stored_slot = self.slot_service
        .get_slot(&slot.id, Authentication::Full, tx.clone().into())
        .await?;
    if stored_slot.version != slot.version {
        return Err(ServiceError::EntityConflicts(...));
    }

    let new_slot_valid_from =
        time::Date::from_iso_week_date(change_year as i32, change_week, time::Weekday::Monday)?;
    let old_slot_valid_to = new_slot_valid_from - time::Duration::days(1);    // Sunday KW-1
    let seg2_valid_to = new_slot_valid_from + time::Duration::days(6);         // Sunday KW (NEW)
    let seg3_valid_from = new_slot_valid_from + time::Duration::days(7);       // Monday KW+1 (NEW)

    let bookings = self.booking_service
        .get_for_slot_id_since(slot.id, change_year, change_week, ...).await?;
    let original_valid_to = stored_slot.valid_to;

    // CAPTURE ORIGINAL VALUES before mutating stored_slot (for Segment 3)
    // (new compared to modify_slot)
    let original_snapshot = stored_slot.clone();   // NEW: needed for Segment 3

    // SEGMENT 1 — same as modify_slot
    stored_slot.valid_to = Some(old_slot_valid_to);
    if stored_slot.valid_to.unwrap() < stored_slot.valid_from {
        self.slot_service.delete_slot(&stored_slot.id, ...).await?;
    } else {
        self.slot_service.update_slot(&stored_slot, ...).await?;
    }

    // SEGMENT 2 — closed at Sunday KW (changed from modify_slot)
    let mut seg2 = stored_slot;
    seg2.valid_from = new_slot_valid_from;
    seg2.valid_to = Some(seg2_valid_to);    // CLOSED: Sunday KW (not original_valid_to)
    seg2.id = Uuid::nil();
    seg2.version = Uuid::nil();
    seg2.min_resources = slot.min_resources;
    seg2.max_paid_employees = slot.max_paid_employees;
    seg2.from = slot.from;
    seg2.to = slot.to;
    let seg2_slot = self.slot_service.create_slot(&seg2, ...).await?;

    // SEGMENT 3 — NEW: restore original values from Monday KW+1
    let mut seg3 = original_snapshot;
    seg3.valid_from = seg3_valid_from;
    seg3.valid_to = original_valid_to;     // original bound (None = unbounded)
    seg3.id = Uuid::nil();
    seg3.version = Uuid::nil();
    // seg3.min_resources, .max_paid_employees, .from, .to remain ORIGINAL (from snapshot)
    let seg3_slot = self.slot_service.create_slot(&seg3, ...).await?;

    // BOOKING RE-POINT — split by week (changed from modify_slot)
    for booking in bookings.iter() {
        self.booking_service.delete(booking.id, ...).await?;
        let target_slot_id =
            if booking.year == change_year && booking.calendar_week == change_week as i32 {
                seg2_slot.id   // booking is IN the exception week → Segment 2
            } else {
                seg3_slot.id   // booking is AFTER exception week → Segment 3
            };
        let mut new_booking = booking.clone();
        new_booking.id = Uuid::nil();
        new_booking.version = Uuid::nil();
        new_booking.slot_id = target_slot_id;   // SPLIT assignment (NEW)
        new_booking.created = None;
        new_booking.created_by = None;
        self.booking_service.create(&new_booking, ...).await?;
    }

    self.transaction_dao.commit(tx).await?;
    Ok(seg2_slot)   // return Segment 2 (the exception slot, as the edit target)
}
```

### Pattern 3: Date Arithmetic — KW Boundaries

**What:** ISO-week Monday/Sunday helpers already in codebase. [VERIFIED: codebase read]

```rust
// Source: service_impl/src/shiftplan_edit.rs:77-79 (existing pattern)
let monday_kw = time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday)?;
let sunday_kw_minus1 = monday_kw - time::Duration::days(1);   // Segment 1 valid_to
let sunday_kw       = monday_kw + time::Duration::days(6);    // Segment 2 valid_to (NEW)
let monday_kw_plus1 = monday_kw + time::Duration::days(7);    // Segment 3 valid_from (NEW)

// Year-boundary safety: +7 days works correctly because ISO weeks are exactly 7 days.
// Example: change_year=2026, change_week=53 → monday_kw = 2026-12-28;
//          monday_kw + 7 days = 2027-01-04 (= 2027-W01-Monday) ✓
```

### Pattern 4: Booking Partition Condition

**What:** Split `bookings` (returned by `get_for_slot_id_since`) into Segment 2 vs Segment 3. [VERIFIED: codebase read — `Booking.calendar_week: i32`, `Booking.year: u32`]

```rust
// Booking struct (service/src/booking.rs:13-24):
//   calendar_week: i32  (note: i32, not u8)
//   year: u32

// Partition condition — exact change_week only:
let goes_to_seg2 = booking.year == change_year
    && booking.calendar_week == change_week as i32;

// The DAO query for get_for_slot_id_since uses:
//   WHERE slot_id = ? AND year * 100 + calendar_week >= ? AND deleted IS NULL
// (dao_impl_sqlite/src/booking.rs:104)
// This returns bookings from change_week onward, cross-year safe.
// The Rust partition above correctly handles year-boundary bookings:
//   year=2027, week=1 → goes_to_seg3 ✓ (year != change_year)
```

### Pattern 5: Frontend Mode Toggle Integration

**What:** Add `single_week: bool` to `SlotEdit` state; pass to `save_slot_edit`. [ASSUMED — current state has no mode field; design derived from existing component structure]

```rust
// state/slot_edit.rs — add field:
pub struct SlotEdit {
    // ... existing fields ...
    pub single_week: bool,   // NEW: "only this week" mode
}

// service/slot_edit.rs — route save action:
pub async fn save_slot_edit() -> Result<(), ShiftyError> {
    let store = SLOT_EDIT_STORE.read();
    match store.slot_edit_type {
        SlotEditType::Edit => {
            if store.single_week {
                // NEW path: calls PUT /shiftplan-edit/slot/{year}/{week}/single-week
                loader::save_slot_single_week(
                    CONFIG.read().clone(),
                    store.slot.clone(),
                    store.year,
                    store.week,
                ).await?;
            } else {
                // Existing path unchanged
                loader::save_slot(CONFIG.read().clone(), store.slot.clone(), store.year, store.week)
                    .await?;
            }
        }
        SlotEditType::New => { /* unchanged */ }
    }
    // ...
}
```

### Pattern 6: Existing Test DI Setup (Base for D-35-05 Tests)

**What:** The `ShiftplanEditDependencies` / `build_dependencies` in `service_impl/src/test/shiftplan_edit.rs` provides the DI scaffold. [VERIFIED: codebase read — lines 150-295]

Key test helper already in place:
- `build_dependencies(permission_grants_shiftplanner, verify_grants_self)` returns a `ShiftplanEditDependencies` with all mocks pre-wired
- `deps.slot_service.expect_get_slot().returning(...)` — override per test
- `deps.slot_service.expect_update_slot()`, `expect_delete_slot()`, `expect_create_slot()` — set expectations
- `deps.booking_service.expect_get_for_slot_id_since()` — returns bookings for the test scenario
- `deps.booking_service.expect_delete()`, `expect_create()` — verify re-point
- `deps.transaction_dao.expect_use_transaction()` + `.expect_commit()` — verify atomicity
- The existing test `test_modify_slot_carries_max_paid_employees` (lines 1157-1209) shows the exact mock pattern for `modify_slot` — the new tests follow this exact pattern

### Anti-Patterns to Avoid

- **Intermediate commit inside the 3-segment operation:** `transaction_dao.commit(tx)` must appear exactly once at the end, after ALL three segments and ALL booking re-points. Any earlier commit breaks D-35-04.
- **Mutating `stored_slot` before cloning for Segment 3:** `stored_slot` is mutated at line 92 (`valid_to = Some(old_slot_valid_to)`) before being used as the base for Segment 2 at line 104. For Segment 3, capture the snapshot BEFORE line 92 (`let original_snapshot = stored_slot.clone()`).
- **Using `stored_slot.valid_to` for Segment 2 in single-week mode:** In the new method, Segment 2 must be closed at `Sunday KW` (not `original_valid_to`). Using `original_valid_to` for Segment 2 would make it unbounded, overwriting the recurring slot permanently.
- **`get_for_slot_id_since` call with wrong week:** The DAO fetches from `change_week` onward. Do NOT call it twice with different weeks. Partition the single result set in Rust.
- **Double-counting via soft-deleted bookings:** The DAO query has `AND deleted IS NULL` (dao_impl_sqlite/src/booking.rs:104). Re-pointed bookings are soft-deleted (not hard-deleted). The DAO never returns them in subsequent reads. Safe by construction.
- **`from` and `to` time fields disabled in Edit mode in the frontend:** `component/slot_edit.rs:121` sets `time_disabled = props.slot_edit_type == SlotEditType::Edit`. But `modify_slot` DOES copy `slot.from` and `slot.to` from the input (lines 112-113). The time fields are currently disabled in the UI for Edit but the backend supports changing them. If the phase decides to allow time changes in single-week mode, the component's `time_disabled` logic needs updating. Otherwise leave consistent with current behavior.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ISO-week Monday arithmetic | Custom week-start calculator | `time::Date::from_iso_week_date(year, week, Weekday::Monday)` | Already used in `modify_slot:78`; handles year boundaries |
| Sunday-of-week calculation | `monday + 6` custom | `monday_kw + time::Duration::days(6)` | Direct and already tested in existing code |
| Cross-year week arithmetic | Manual year rollover | `monday_kw + time::Duration::days(7)` | `time` crate handles calendar correctly |
| Slot atomicity | Custom rollback logic | `transaction_dao.use_transaction` / `commit` existing pattern | Pattern is tested and battle-proven |
| Booking soft-delete | Hard-delete or custom flag | `booking_service.delete(id, ...)` existing method | DAO already handles; reports filter `deleted IS NULL` |
| Mock DI for new method | Custom test harness | `ShiftplanEditDependencies::build_service()` existing scaffold | Lines 150-295 of `test/shiftplan_edit.rs` |

**Key insight:** The entire 3-segment operation is achievable by reordering and supplementing existing service calls. No custom transaction handling, no raw SQL, no new DAO methods.

---

## Runtime State Inventory

> Skipped — this is a greenfield extension, not a rename/refactor/migration phase. No existing stored data is renamed or migrated.

---

## Common Pitfalls

### Pitfall 1: Segment 2 Left Unbounded in Single-Week Mode
**What goes wrong:** `new_slot.valid_to = original_valid_to` in the existing `modify_slot` makes Segment 2 unbounded. If copied to `modify_slot_single_week` without change, Segment 2 will run indefinitely — Segment 3 will then conflict with it (overlapping `valid_from`).
**Why it happens:** The code pattern at lines 104-117 is correct for "from this week" but wrong for "only this week".
**How to avoid:** In `modify_slot_single_week`, set `seg2.valid_to = Some(new_slot_valid_from + Duration::days(6))` (Sunday KW) BEFORE calling `create_slot`.
**Warning signs:** `SlotService::create_slot` likely has an overlap check; Segment 3 creation would return a conflict error if Segment 2 overlaps.

### Pitfall 2: Snapshot Captured After stored_slot Mutation
**What goes wrong:** `stored_slot.valid_to` is mutated at line 92 for Segment 1. If `original_snapshot = stored_slot.clone()` is done after line 92, Segment 3 inherits the wrong `valid_to` (the Segment 1 `valid_to` instead of the true original).
**Why it happens:** `stored_slot` is mutated in-place; the variable is then reused as the base for `new_slot`.
**How to avoid:** `let original_snapshot = stored_slot.clone();` BEFORE line 92 (before any mutation).
**Warning signs:** Segment 3 has `valid_to = Some(Sunday KW-1)` which makes it a zero-duration or negative-duration slot — would fail validation.

### Pitfall 3: Booking Partition Uses Wrong Type Comparison
**What goes wrong:** `booking.calendar_week` is `i32` (not `u8`). If compared as `booking.calendar_week == change_week` where `change_week: u8`, Rust will type-error or need explicit cast.
**Why it happens:** Booking entity uses `i32` for `calendar_week` (service/src/booking.rs:17) while service methods use `u8`.
**How to avoid:** `booking.calendar_week == change_week as i32`. Likewise `booking.year == change_year` works directly (both `u32`).
**Warning signs:** Compile error: `can't compare i32 with u8`.

### Pitfall 4: Booking Re-Point Loop Assigns Segment 3 ID to Exception-Week Bookings
**What goes wrong:** In `modify_slot`, all bookings go to `new_slot.id`. Copy-paste error assigns `seg3_slot.id` to ALL bookings — exception-week bookings land on Segment 3 (wrong values) not Segment 2.
**Why it happens:** Pattern from `modify_slot` uses a single `new_slot.id` for all.
**How to avoid:** Mandatory D-35-05 test: assert that a booking with `calendar_week == change_week` has `slot_id == seg2_slot.id`.
**Warning signs:** Test `test_booking_in_exception_week_lands_on_seg2` fails.

### Pitfall 5: Edge Case — First KW of Slot (Segment 1 Vanishes)
**What goes wrong:** When `change_week == slot's first week`, Segment 1 would have `valid_to < valid_from`. The existing code at lines 94-102 handles this for `modify_slot` by calling `delete_slot`. In `modify_slot_single_week`, after deleting Segment 1, Segments 2 and 3 must still be created.
**Why it happens:** The delete-slot branch short-circuits nothing — the code continues after line 102 in both cases.
**How to avoid:** In the new method, the delete-or-update branch (Segment 1) is followed unconditionally by Segment 2 and 3 creation. The pattern is already correct — just verify the edge case is tested.
**Warning signs:** Slot count after operation is 2 (not 3) when Segment 1 was deleted — this is correct behavior.

### Pitfall 6: Clippy Warning on Unused Variable or Unnecessary Clone
**What goes wrong:** `original_snapshot = stored_slot.clone()` before line 92. If clippy detects the clone is "unnecessary" in the non-single-week path, it may fail the build.
**Why it happens:** `cargo clippy -- -D warnings` is a hard gate (CLAUDE.md).
**How to avoid:** In the separate `modify_slot_single_week` method, the clone is always necessary. No clippy issue because the method doesn't have a non-single-week path.
**Warning signs:** `warning: unused variable` or clippy lint on redundant clone.

### Pitfall 7: No `cargo sqlx prepare` Needed — But Verify
**What goes wrong:** Assuming no new SQL queries are added. If any code path accidentally introduces a new `sqlx::query!` macro, the offline query cache (`sqlx-data.json`) becomes stale.
**Why it happens:** New SQL in DAO layer triggers SQLx compile-time check failure.
**How to avoid:** The 3-segment logic uses ONLY existing service calls (`get_slot`, `update_slot`, `delete_slot`, `create_slot`, `get_for_slot_id_since`, `delete`, `create`). No new DAO methods needed. Verify before commit: `grep -r "sqlx::query!" service_impl/src/shiftplan_edit.rs` should show 0 matches.
**Warning signs:** `error: no data found for query` at compile time.

---

## Code Examples

### Example 1: Existing modify_slot Atomicity Bracket (to extend)
```rust
// Source: service_impl/src/shiftplan_edit.rs:59,141 [VERIFIED: codebase read]
let tx = self.transaction_dao.use_transaction(tx).await?;
// ... all slot operations with tx.clone() ...
self.transaction_dao.commit(tx).await?;
```

### Example 2: Date Arithmetic Already Proven in Codebase
```rust
// Source: service_impl/src/shiftplan_edit.rs:78-79 [VERIFIED: codebase read]
let new_slot_valid_from =
    time::Date::from_iso_week_date(change_year as i32, change_week, time::Weekday::Monday)?;
let old_slot_valid_to = new_slot_valid_from - time::Duration::days(1);

// New for single-week:
let seg2_valid_to    = new_slot_valid_from + time::Duration::days(6);  // Sunday KW
let seg3_valid_from  = new_slot_valid_from + time::Duration::days(7);  // Monday KW+1
```

### Example 3: Booking Soft-Delete Filter at DAO Layer (double-count protection)
```rust
// Source: dao_impl_sqlite/src/booking.rs:104 [VERIFIED: codebase read]
// "SELECT ... FROM booking WHERE slot_id = ? AND year * 100 + calendar_week >= ?
//  AND deleted IS NULL"
//
// Implication: soft-deleted predecessor bookings are INVISIBLE to all subsequent
// DAO reads. No double-counting is possible via DAO queries.
```

### Example 4: Mock Test Pattern for modify_slot (model for D-35-05 tests)
```rust
// Source: service_impl/src/test/shiftplan_edit.rs:1158-1209 [VERIFIED: codebase read]
#[tokio::test]
async fn test_modify_slot_carries_max_paid_employees() {
    let mut deps = build_dependencies(true, true);
    deps.slot_service.expect_update_slot().returning(|_, _, _| Ok(()));
    deps.booking_service
        .expect_get_for_slot_id_since()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<Booking>::new())));
    deps.slot_service.expect_create_slot()
        .returning(|slot, _, _| {
            assert_eq!(slot.max_paid_employees, Some(7));
            Ok(slot.clone())
        });
    let service = deps.build_service();
    let input = Slot { min_resources: 4, max_paid_employees: Some(7), ..monday_slot() };
    let result = service.modify_slot(&input, 2026, 26, ().auth(), None).await.expect("...");
    assert_eq!(result.max_paid_employees, Some(7));
}
```

### Example 5: Frontend api.rs Pattern to Model New Endpoint Call
```rust
// Source: shifty-dioxus/src/api.rs:153-165 [VERIFIED: codebase read]
pub async fn update_slot(
    config: Config,
    slot: SlotTO,
    year: u32,
    week: u8,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/shiftplan-edit/slot/{}/{}", config.backend, year, week);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&slot).send().await?;
    response.error_for_status_ref()?;
    Ok(())
}

// New function (same pattern, different route):
pub async fn update_slot_single_week(
    config: Config,
    slot: SlotTO,
    year: u32,
    week: u8,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/shiftplan-edit/slot/{}/{}/single-week", config.backend, year, week);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&slot).send().await?;
    response.error_for_status_ref()?;
    Ok(())
}
```

---

## Backend Wiring: Trade-off Analysis (Claude's Discretion)

### Option A: `single_week: bool` param on `modify_slot`

**Changes:** `service/src/shiftplan_edit.rs` trait signature + `service_impl` impl + REST handler + all mock tests that call `modify_slot`.

**Pros:**
- Single method, single REST route
- Minimal surface area

**Cons:**
- Bool flag in service trait = code smell; trait becomes aware of two fundamentally different behaviors
- All existing `modify_slot` mock tests need updating to pass the new parameter (mockall `expect_modify_slot` in other test files)
- Violates Open/Closed: extending existing method rather than adding new one

### Option B: Separate `modify_slot_single_week` method (RECOMMENDED)

**Changes:** New method in trait + new impl + new REST route `PUT .../slot/{year}/{week}/single-week` + new handler. Existing `modify_slot` signature untouched.

**Pros:**
- Existing tests for `modify_slot` compile and pass without modification
- Clear REST API semantics — two distinct operations visible in OpenAPI
- Follows the project's pattern: `remove_slot` is separate from `modify_slot` even though they share mechanics
- `#[automock]` on the trait generates mock for both methods; tests for the new method are isolated

**Cons:**
- More boilerplate (new method in trait, new handler, new route)
- Trait grows by one method

**Decision guidance for planner:** Recommend Option B. The precedent is `remove_slot` (lines 145-197), which shares nearly identical setup with `modify_slot` but is a separate method with a separate REST route (`DELETE .../slot/{slot_id}/{year}/{week}`).

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single permanent slot | Versioned slot with `valid_from`/`valid_to` | Pre-existing (Phase 1) | Enables modify-from-week pattern; Phase 35 extends with third segment |
| All bookings re-pointed to single new slot | Bookings partitioned by target week | Phase 35 (this phase) | Enables exception-week isolation |

**No deprecated patterns in scope** — Phase 35 only extends, does not replace.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Separate `modify_slot_single_week` method is recommended; planner must confirm vs bool-flag approach | Backend Wiring Trade-off | If bool flag chosen instead, the task sequence changes (modify existing method + test fixture updates rather than adding new method) |
| A2 | New REST route `PUT /shiftplan-edit/slot/{year}/{week}/single-week` (path segment approach) | Architecture Diagram, API fn example | If query-param approach chosen (`?single_week=true`), `api.rs` example changes; backend handler changes; same behavior |
| A3 | `single_week: bool` in `SlotEdit` state is sufficient; no new `SlotEditType` variant needed | Frontend Pattern 5 | If a third `SlotEditType::EditSingleWeek` variant is preferred, the service/slot_edit.rs routing logic differs slightly |
| A4 | `time_disabled` in slot editor stays unchanged (from/to time fields remain disabled in Edit mode) | Anti-Patterns | If the feature spec decides to allow time-field changes in single-week mode, `component/slot_edit.rs:121` needs updating |

---

## Open Questions

1. **Backend wiring method vs param (Claude's Discretion)**
   - What we know: Both Option A (bool flag) and Option B (separate method) are technically sound
   - What's unclear: Team preference; long-term API versioning needs
   - Recommendation: Option B (separate method), consistent with `remove_slot` precedent

2. **Frontend mode toggle widget type (Claude's Discretion)**
   - What we know: Editor dialog uses Dioxus RSX; existing fields use `SelectInput` and `input` atoms
   - What's unclear: Whether a radio group, a toggle switch, or a checkbox is preferred
   - Recommendation: Radio group (two explicit labels) — makes the distinction clear; consistent with the i18n explanation text already in the dialog

3. **`cargo sqlx prepare` needed?**
   - What we know: No new SQL queries are added — all DAO calls reuse existing service methods
   - What's unclear: Whether any planner task touches DAO layer
   - Recommendation: Verify with `grep -r 'sqlx::query' service_impl/src/shiftplan_edit.rs` before finalizing plan; if zero, no prepare step needed

---

## Environment Availability

> Phase is pure Rust code + Dioxus frontend. No new external services.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Backend build | ✓ | workspace | — |
| `cargo test` | D-35-05 tests | ✓ | workspace | — |
| `cargo clippy` | Hard gate | ✓ | workspace | — |
| `dx` (Dioxus CLI) | Frontend build check | ✓ (pinned 0.6.x via flake) | 0.6.x | — |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[tokio::test]` + `mockall` |
| Config file | None — workspace-level `Cargo.toml` |
| Quick run command | `cargo test -p service_impl shiftplan_edit` |
| Full suite command | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SWO-01 | Mode toggle renders in editor; i18n keys present | unit (SSR) | `cargo test -p shifty-dioxus slot_edit` | ✅ (slot_edit.rs has SSR tests) |
| SWO-02 | 3-segment structure: correct valid_from/valid_to on each segment | unit (mock) | `cargo test -p service_impl modify_slot_single_week` | ❌ Wave 0 |
| SWO-02 | Bookings in exception KW → seg2; bookings after → seg3 | unit (mock) | `cargo test -p service_impl booking_split` | ❌ Wave 0 |
| SWO-03 | Rollback: error mid-operation → no commit → state unchanged | unit (mock) | `cargo test -p service_impl rollback` | ❌ Wave 0 |
| SWO-04 | No double-count: deleted bookings invisible to DAO | unit (mock) | `cargo test -p service_impl no_double_count` | ❌ Wave 0 |
| SWO-04 | Edge: exception KW == first KW → Segment 1 deleted, 2+3 created | unit (mock) | `cargo test -p service_impl first_kw_edge` | ❌ Wave 0 |
| SWO-04 | Edge: slot has `valid_to = None` → Segment 3 unbounded | unit (mock) | `cargo test -p service_impl unbounded_seg3` | ❌ Wave 0 |
| SWO-04 | Edge: no bookings in exception KW | unit (mock) | `cargo test -p service_impl no_bookings_edge` | ❌ Wave 0 |
| SWO-04 | Permission: `shiftplan.edit` required | unit (mock) | `cargo test -p service_impl forbidden` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p service_impl shiftplan_edit` + `cargo clippy --workspace -- -D warnings`
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Phase gate:** Full suite green + WASM build gate `cargo build --target wasm32-unknown-unknown` (from backend shell) before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `service_impl/src/test/shiftplan_edit.rs` — add `modify_slot_single_week` test module (all D-35-05 tests)
- [ ] Verify WASM compile: `cargo build --target wasm32-unknown-unknown` in `shifty-dioxus/` from backend nix-shell

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes (indirect) | OIDC / mock-auth already in place; `modify_slot_single_week` inherits same auth context flow |
| V3 Session Management | no | Stateless REST |
| V4 Access Control | yes | `check_permission("shiftplan.edit", context)` — must be first call inside `use_transaction` bracket, same as `modify_slot:61` |
| V5 Input Validation | yes | Version conflict check (`stored_slot.version != slot.version → EntityConflicts`) already present; same check required in new method |
| V6 Cryptography | no | No secrets, no encryption in this path |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unauthorized slot modification | Elevation of Privilege | `check_permission("shiftplan.edit")` gate — must be called before any mutation |
| Version collision (concurrent edit) | Tampering | `stored_slot.version != slot.version → EntityConflicts` — already present, must be in new method |
| Partial state on error (torn write) | Tampering | Single `tx` bracket with one `commit` at end; any error → implicit rollback |
| Orphaned bookings after partial re-point | Tampering | All re-points in same `tx`; if loop errors mid-way → rollback → zero re-points committed |

---

## Project Constraints (from CLAUDE.md)

- **Clippy hard gate:** `cargo clippy --workspace -- -D warnings` — every commit. Run BEFORE committing.
- **Transaction pattern:** `use_transaction(tx)` at start, single `commit(tx)` at end. No intermediate commits.
- **Service tier:** `ShiftplanEditService` is Business-Logic tier — may consume `SlotService` (Basic) and `BookingService` (Basic). No circular deps.
- **OpenAPI annotation:** Any new REST handler needs `#[utoipa::path(...)]` annotation. Add to `ShiftplanEditApiDoc` in `rest/src/shiftplan_edit.rs:210-229`.
- **Snapshot schema version:** NO bump required — this phase does not add/change/remove `BillingPeriodValueType` entries. Slot versioning changes do not flow through `billing_period_report.rs`. Confirmed by REQUIREMENTS.md "Out of Scope" section.
- **sqlx prepare:** NOT required — no new SQL queries. All DAO calls reuse existing methods.
- **i18n:** New UI text must appear in all three locales: `de.rs`, `en.rs`, `cs.rs`.
- **VCS:** GSD auto-commit via git (co-located jj). Do not run `git commit` manually.
- **Tests:** Always run `cargo test` after implementing new features.

---

## Sources

### Primary (HIGH confidence)

- `service_impl/src/shiftplan_edit.rs:51-143` — `modify_slot` exact implementation [VERIFIED: codebase read]
- `service_impl/src/shiftplan_edit.rs:145-197` — `remove_slot` (analogue reference) [VERIFIED: codebase read]
- `service/src/booking.rs:13-24` — `Booking` struct field types (`calendar_week: i32`, `year: u32`) [VERIFIED: codebase read]
- `service/src/slot.rs:13-25` — `Slot` struct fields [VERIFIED: codebase read]
- `service/src/shiftplan_edit.rs:41-121` — `ShiftplanEditService` trait signature [VERIFIED: codebase read]
- `dao_impl_sqlite/src/booking.rs:104` — `get_for_slot_id_since` SQL (`AND deleted IS NULL`) [VERIFIED: codebase read]
- `service_impl/src/test/shiftplan_edit.rs:150-295` — DI scaffold for new tests [VERIFIED: codebase read]
- `service_impl/src/test/shiftplan_edit.rs:1157-1209` — `test_modify_slot_carries_max_paid_employees` (exact test pattern to follow) [VERIFIED: codebase read]
- `rest/src/shiftplan_edit.rs:20-82` — REST routes and handlers [VERIFIED: codebase read]
- `shifty-dioxus/src/state/slot_edit.rs` — `SlotEdit`/`SlotEditItem` structs [VERIFIED: codebase read]
- `shifty-dioxus/src/service/slot_edit.rs` — `save_slot_edit`, `SlotEditAction` [VERIFIED: codebase read]
- `shifty-dioxus/src/component/slot_edit.rs` — editor dialog, `SlotEditProps` [VERIFIED: codebase read]
- `shifty-dioxus/src/api.rs:153-165` — `update_slot` API call pattern [VERIFIED: codebase read]
- `shifty-dioxus/src/loader.rs:705-713` — `save_slot` loader [VERIFIED: codebase read]
- `shifty-dioxus/src/i18n/de.rs:322-338` — existing slot edit i18n keys [VERIFIED: codebase read]

### Secondary (LOW confidence)

- `[ASSUMED]` — Recommended architecture (separate method, separate route, `single_week: bool` state field) derived from codebase analysis and project conventions, not yet validated by implementation.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all in workspace
- Architecture patterns: HIGH — derive directly from verified codebase lines
- Pitfalls: HIGH — identified from actual code structure (mutation order, type mismatch, SQL filter)
- Backend wiring choice: LOW — Claude's Discretion; one of two valid approaches

**Research date:** 2026-06-30
**Valid until:** 2026-07-30 (stable Rust ecosystem; no fast-moving dependencies)
