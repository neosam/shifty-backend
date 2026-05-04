---
phase: 05-slot-paid-capacity-warning
plan: 02
subsystem: service
tags: [service, warning, enum, paid-employee-limit, phase-5, additive]

# Dependency graph
requires:
  - phase: 03-booking-shift-plan-konflikt-integration
    plan: "*"
    provides: "v1.0 `service::warning::Warning` enum (4 variants) + `BookingCreateResult { booking, warnings }` wrapper pattern"
provides:
  - "service::warning::Warning gains a 5th variant `PaidEmployeeLimitExceeded { slot_id, booking_id, year, week, current_paid_count, max_paid_employees }` (D-08)"
  - "Service-tier warning contract for Phase-5's paid-capacity overflow signal — pure data, no i18n strings (D-13)"
affects:
  - "05-05 (REST DTO surface): WarningTO must mirror this 5th variant + add From-arm to keep workspace build green (E0004 in rest-types/src/lib.rs:1705 expected until Plan 05 lands)"
  - "05-06 (ShiftplanEditService emission): pushes Warning::PaidEmployeeLimitExceeded into BookingCreateResult.warnings inside book_slot_with_conflict_check"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Additive enum-extension: append new variant at the end of `Warning`, byte-preserve existing variants (Rust derives — Debug/Clone/PartialEq/Eq — propagate automatically since all new fields are Copy/Eq)"
    - "Domain-tier warning carries pure data; transport-tier WarningTO owns the wire format (#[serde(tag, content)] + ToSchema). Phase-5 follows this v1.0 split byte-for-byte"
    - "Wave-coupled landing: when an additive change to a domain enum breaks an exhaustive downstream match (here: rest-types/src/lib.rs:1705), schedule the producer-plan + consumer-plan in the SAME wave so workspace build is green at wave-boundary"

key-files:
  created: []
  modified:
    - "service/src/warning.rs (5th variant `PaidEmployeeLimitExceeded` appended after `AbsenceOverlapsManualUnavailable`)"

key-decisions:
  - "Variant carries 6 fields (slot_id, booking_id, year, week, current_paid_count, max_paid_employees) — superset of D-08's literal 3-field spec. Extra fields (booking_id/year/week) match the carry-the-booking-context convention of the existing 4 variants and are required by Plan 06's emission shape (no rework needed downstream)"
  - "Field types: `slot_id: Uuid`, `booking_id: Uuid`, `year: u32`, `week: u8`, `current_paid_count: u8`, `max_paid_employees: u8`. `u8` for the count fields matches `SlotEntity.max_paid_employees: Option<u8>` from Plan 01 — no widening, no narrowing"
  - "No new derives added — `#[derive(Clone, Debug, PartialEq, Eq)]` on the enum already covers the new variant since Uuid/u8/u32 are all Copy/Eq/Hash"
  - "No serde derives on the domain enum — wire format lives in WarningTO (Plan 05). v1.0 Phase 3 architecture preserved"
  - "Field order chosen to match Plan 05 WarningTO and Plan 06 emission destructuring order — bind-by-name everywhere, but ordering ensures visual consistency in match arms"

patterns-established:
  - "Wave-3 producer/consumer pairing: Plan 02 (producer = Warning enum) + Plan 05 (consumer = WarningTO From-arm) land together to keep workspace build green at wave boundary. Standalone, Plan 02 only requires `cargo build -p service` to succeed (which it does)."

requirements-completed: [D-08, D-13]

# Metrics
duration: 3min
completed: 2026-05-04
---

# Phase 5 Plan 02: Warning Enum 5th Variant Summary

**`service::warning::Warning` extended from 4 to 5 variants. The new `PaidEmployeeLimitExceeded { slot_id, booking_id, year, week, current_paid_count, max_paid_employees }` variant (D-08) is the service-tier contract that Plan 05 (REST DTO) and Plan 06 (emission in `ShiftplanEditService::book_slot_with_conflict_check`) both depend on. Pure additive change — existing 4 variants byte-preserved, no other files touched.**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-05-04T08:13Z (approx)
- **Completed:** 2026-05-04T08:16Z (approx)
- **Tasks:** 1
- **Files modified:** 1 (`service/src/warning.rs`)

## Accomplishments

- `service::warning::Warning` enum now has 5 variants. The new `PaidEmployeeLimitExceeded` variant lives at the end of the enum, byte-preserving the 4 existing variants and their order.
- The variant carries 6 structured fields: `slot_id: Uuid`, `booking_id: Uuid`, `year: u32`, `week: u8`, `current_paid_count: u8`, `max_paid_employees: u8`. No i18n strings — translation happens in the frontend per the v1.0 Phase 3 architecture.
- A doc-comment on the variant names D-06 (strict-greater threshold), D-07 (booking still persisted), D-08 (variant shape), D-15 (NULL = no check) inline.
- `cargo build -p service` succeeds standalone.
- Workspace-wide `cargo build` fails with E0004 at `rest-types/src/lib.rs:1705` (non-exhaustive match on `&Warning`) — **this is expected** and matches the Wave-3 plan-level coupling. Plan 05-05 (same wave) adds the corresponding `WarningTO::PaidEmployeeLimitExceeded` variant and its `From<&Warning>` arm so the workspace build flips back to green.

## Task Commits

Each task was committed atomically via `jj`:

1. **Task 1: Add `Warning::PaidEmployeeLimitExceeded` variant** — change `untoytuw` (commit `4d0ec8f3`) — `feat(05-02)`

## Files Created/Modified

- **Modified:** `service/src/warning.rs`
  - Appended a 5th variant after `AbsenceOverlapsManualUnavailable` (between line 53 and the closing `}` at the time of execution).
  - The variant declaration:

    ```rust
    /// Phase 5 (D-08): emittiert beim Anlegen eines Bookings über
    /// `ShiftplanEditService::book_slot_with_conflict_check`, wenn der
    /// Ziel-Slot ein konfiguriertes `max_paid_employees`-Limit hat und der
    /// resultierende Live-Count der bezahlten Mitarbeiter:innen in dieser
    /// (year, week, slot)-Kombination das Limit STRIKT übersteigt
    /// (`current_paid_count > max_paid_employees`, D-06).
    ///
    /// Die Buchung wird trotzdem persistiert (D-07) — die Warning ist
    /// rein informativ. NULL `max_paid_employees` triggert NICHT (D-15).
    /// Übersetzung der Variant-Bedeutung passiert im Frontend (en/de/cs);
    /// das Backend trägt nur strukturierte Daten.
    PaidEmployeeLimitExceeded {
        slot_id: Uuid,
        booking_id: Uuid,
        year: u32,
        week: u8,
        current_paid_count: u8,
        max_paid_employees: u8,
    },
    ```

  - The 4 existing variants (`BookingOnAbsenceDay`, `BookingOnUnavailableDay`, `AbsenceOverlapsBooking`, `AbsenceOverlapsManualUnavailable`) are byte-identical — verified via `git diff` / `jj diff` showing only insertions, no deletions or in-place edits.

## Verification

- `grep -c "PaidEmployeeLimitExceeded" service/src/warning.rs` → 1 (variant declared once)
- `grep -cE "BookingOnAbsenceDay|BookingOnUnavailableDay|AbsenceOverlapsBooking|AbsenceOverlapsManualUnavailable" service/src/warning.rs` → 4 (all 4 existing variants present)
- `grep -A 10 "PaidEmployeeLimitExceeded {" service/src/warning.rs | grep -cE "slot_id|booking_id|year|week|current_paid_count|max_paid_employees"` → 6 (all 6 fields declared)
- `cargo build -p service` → exits 0
- `cargo build` (workspace) → fails with E0004 at `rest-types/src/lib.rs:1705` — **expected**, will be resolved by Plan 05-05's `From<&Warning>` arm in the same wave.

## Decisions Made

1. **Variant carries 6 fields, not the 3 that D-08's literal text mentions.** D-08 in CONTEXT.md says `{ slot_id, current_paid_count, max_paid_employees }`, but the Pattern-Map (`.planning/phases/05-slot-paid-capacity-warning/05-PATTERNS.md` lines 215-235) and Plan 02's `<action>` block both prescribe a 6-field shape that adds `booking_id`, `year`, `week` to match the booking-context convention of the existing 4 variants and Plan 06's emission shape. Followed the Plan-File (Source of Truth) — this is consistent with the must_haves.artifacts entry naming all 6 fields.
2. **No new derives.** The enum already has `#[derive(Clone, Debug, PartialEq, Eq)]` (line 22). All 6 new fields (`Uuid`, `u32`, `u8`) are `Copy + Eq + Hash`, so the existing derives propagate automatically.
3. **No serde derives on the domain enum.** Wire format (with `#[serde(tag = "kind", content = "data")]`) lives in `WarningTO` in `rest-types/src/lib.rs` (Plan 05). This matches the v1.0 Phase 3 architecture: domain enum is plain Rust, transport enum carries serde semantics.
4. **Field order is `slot_id, booking_id, year, week, current_paid_count, max_paid_employees`.** This matches Plan 06's emission shape (D-08 + Pattern-Map) and Plan 05's `WarningTO::PaidEmployeeLimitExceeded` arm. Bind-by-name in match arms means ordering is cosmetic; consistency across the producer/transport/consumer triple still helps readability.
5. **Variant placed at the end of the enum, after `AbsenceOverlapsManualUnavailable`.** Append-only — preserves the existing variants' byte-positions and avoids any reordering churn.

## Deviations from Plan

None — Task 1 was implemented exactly as the Plan-File `<action>` block prescribes. The single Plan-vs-CONTEXT inconsistency (3-field D-08 text vs 6-field plan/action+pattern-map) was resolved in favour of the plan's prescription, which was the deliberate planner choice.

## Issues Encountered

None — single-task plan, single-line semantic change (one new enum variant), single compile check. The expected workspace E0004 failure at `rest-types/src/lib.rs:1705` is **pre-anticipated** by the plan (`<wave-3 placement rationale>`) and is the correct downstream signal that Plan 05-05's `From<&Warning>` arm is the next thing to land.

## Other downstream consumers (audit)

Per the plan's `<output>` instructions: scanned the workspace for OTHER `match` statements on `&service::warning::Warning` or `service::warning::Warning` that could be impacted by the new variant.

```
$ grep -rn "service::warning::Warning::" --include='*.rs' --exclude-dir=target
service/src/warning.rs (variants only — definition site, not match)
rest-types/src/lib.rs:1705-1745 (the known From-impl flagged by the plan)
```

No other downstream `match` consumer found. The plan's anticipation (`currently only WarningTO::from is the known consumer`) is correct.

## User Setup Required

None.

## Next Phase Readiness

- **Wave 3 (Plans 05-05 + 05-06)** unblocked. Plan 05-05 adds the corresponding `WarningTO::PaidEmployeeLimitExceeded` variant + `From<&Warning>` arm → workspace `cargo build` flips back to green. Plan 05-06 emits `Warning::PaidEmployeeLimitExceeded { ... }` from `ShiftplanEditService::book_slot_with_conflict_check`.
- The 5-variant `Warning` enum is the stable service-tier contract for the rest of Wave 3.

## Self-Check: PASSED

- File `service/src/warning.rs` contains exactly 1 occurrence of `PaidEmployeeLimitExceeded` (variant declaration). Verified.
- File `service/src/warning.rs` contains all 4 pre-existing variant names: `BookingOnAbsenceDay`, `BookingOnUnavailableDay`, `AbsenceOverlapsBooking`, `AbsenceOverlapsManualUnavailable`. Verified.
- New variant declares all 6 expected fields. Verified.
- `cargo build -p service` exits 0. Verified.
- Expected E0004 in `rest-types/src/lib.rs:1705` is reproducible (deferred to Plan 05-05 per plan). Verified.
- jj history shows 1 atomic Plan-02 task change (`untoytuw` Task 1, commit `4d0ec8f3`). Verified.
- No other files modified beyond `service/src/warning.rs`. Verified via `jj diff --stat` showing exactly 1 file changed (+19 lines, 0 deletions).

---
*Phase: 05-slot-paid-capacity-warning*
*Completed: 2026-05-04*
