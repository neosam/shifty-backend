---
phase: 35-slot-einzelwoche-aenderung
plan: "01"
subsystem: api
tags: [rust, axum, service, tdd, mockall, transaction, shiftplan]

requires:
  - phase: 34-feiertag-soll-schichtplan
    provides: ShiftplanEditService-Basisimplementierung mit modify_slot / remove_slot

provides:
  - ShiftplanEditService::modify_slot_single_week (Trait + Impl)
  - REST-Route PUT /shiftplan-edit/slot/{year}/{week}/single-week
  - MockShiftplanEditService::expect_modify_slot_single_week (via #[automock])

affects: [35-02, 35-03, phase-36-frontend]

tech-stack:
  added: []
  patterns:
    - "3-Segment-Slot-Split: Original schrumpfen + Ausnahme-Slot + Restore-Slot in EINER Transaktion"
    - "original_snapshot vor Mutation klonen (Pitfall 2 in modify_slot_single_week)"
    - "Booking-Partition nach calendar_week == change_week as i32 (Typ-Cast Pitfall 3)"

key-files:
  created: []
  modified:
    - service/src/shiftplan_edit.rs
    - service_impl/src/shiftplan_edit.rs
    - service_impl/src/test/shiftplan_edit.rs
    - rest/src/shiftplan_edit.rs

key-decisions:
  - "D-35-01: Option B (separate Methode modify_slot_single_week, kein bool-Flag) — modify_slot bleibt unverändert"
  - "D-35-03: Booking-Partition in Rust-Code (calendar_week == change_week), nicht durch zweiten DAO-Aufruf"
  - "D-35-04: Genau EINE Transaktion, ein commit am Ende — kein Zwischen-commit"
  - "D-35-05: 9 TDD-Tests (3-Segment-Daten, KW26→Seg2, KW27→Seg3, je-genau-einmal, Rollback, Erste-KW-Edge, unbegrenzt, keine Buchungen, Forbidden)"
  - "D-35-06: check_permission('shiftplan.edit') als erster Aufruf in modify_slot_single_week"
  - "REST: kein #[utoipa::path] auf edit_slot_single_week — konsistent mit edit_slot (Phase-1-Slots bewusst ohne Annotation)"

patterns-established:
  - "original_snapshot = stored_slot.clone() VOR stored_slot.valid_to-Mutation"
  - "Segment 2 valid_to = Some(seg2_valid_to) [geschlossen!], NICHT original_valid_to"
  - "Segment 3 aus original_snapshot, id/version = Uuid::nil()"

requirements-completed: [SWO-02, SWO-03, SWO-04]

coverage:
  - id: D1
    description: "modify_slot_single_week erzeugt 3 korrekte Slot-Segmente (Seg1 valid_to=Sonntag KW-1, Seg2 Mon-Son KW, Seg3 ab Montag KW+1)"
    requirement: SWO-02
    verification:
      - kind: unit
        ref: "service_impl/src/test/shiftplan_edit.rs#test_msw_three_segment_structure"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/shiftplan_edit.rs#test_msw_unbounded_valid_to_seg3"
        status: pass
    human_judgment: false
  - id: D2
    description: "Buchungen werden partitioniert: calendar_week==change_week→Seg2, sonst→Seg3; keine Doppelzählung"
    requirement: SWO-02
    verification:
      - kind: unit
        ref: "service_impl/src/test/shiftplan_edit.rs#test_msw_booking_partition_and_each_exactly_once"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/shiftplan_edit.rs#test_msw_no_bookings_no_booking_mutations"
        status: pass
    human_judgment: false
  - id: D3
    description: "Atomarität: Fehler mitten im Vorgang führt zu keinem commit (Rollback)"
    requirement: SWO-03
    verification:
      - kind: unit
        ref: "service_impl/src/test/shiftplan_edit.rs#test_msw_rollback_no_commit_on_error"
        status: pass
    human_judgment: false
  - id: D4
    description: "Permission-Gate check_permission('shiftplan.edit'); Erste-KW-Edge (delete_slot statt update)"
    requirement: SWO-04
    verification:
      - kind: unit
        ref: "service_impl/src/test/shiftplan_edit.rs#test_msw_forbidden"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/shiftplan_edit.rs#test_msw_first_kw_edge_delete_slot"
        status: pass
    human_judgment: false
  - id: D5
    description: "REST-Route PUT /shiftplan-edit/slot/{year}/{week}/single-week exponiert modify_slot_single_week"
    requirement: SWO-02
    verification:
      - kind: unit
        ref: "cargo build -p rest (Kompilierung + Clippy clean)"
        status: pass
    human_judgment: false

duration: 25min
completed: 2026-06-30
status: complete
---

# Phase 35 Plan 01: modify_slot_single_week Summary

**3-Segment-Slot-Split `modify_slot_single_week` mit atomarer Transaktion, Booking-Partition nach Ausnahme-KW und REST-Route PUT /shiftplan-edit/slot/{year}/{week}/single-week**

## Performance

- **Duration:** 25 min
- **Started:** 2026-06-30T17:38:54Z
- **Completed:** 2026-06-30T18:04:39Z
- **Tasks:** 3 (RED + GREEN + REST-Glue)
- **Files modified:** 4

## Accomplishments

- `ShiftplanEditService::modify_slot_single_week` Trait-Methode + vollständige Implementierung (3-Segment-Split + Booking-Partition + Atomarität)
- 9 D-35-05-Pflicht-Tests (alle grün, RED→GREEN TDD-Zyklus korrekt durchlaufen)
- REST-Handler `edit_slot_single_week` + Route `PUT /slot/{year}/{week}/single-week` ohne utoipa-Annotation (konsistent mit `edit_slot`)
- Keine neuen SQL-Queries, kein `cargo sqlx prepare`, kein Snapshot-Schema-Bump
- `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` grün

## Task Commits

1. **Task 1 (RED):** Trait + Stub + 9 Tests — `633fc26` (test)
2. **Task 2 (GREEN):** Vollständige Implementierung — `0d01585` (feat)
3. **Task 3 (REST):** Handler + Route — `e4af6c8` (feat)

## Files Created/Modified

- `service/src/shiftplan_edit.rs` — `modify_slot_single_week` Trait-Methode mit Dokumentation
- `service_impl/src/shiftplan_edit.rs` — Vollimplementierung: 3-Segment-Split, Booking-Partition, Atomarität, Permission-Gate
- `service_impl/src/test/shiftplan_edit.rs` — 9 TDD-Tests (D-35-05) + 2 Helper-Funktionen + Mutex-Import
- `rest/src/shiftplan_edit.rs` — Route `/slot/{year}/{week}/single-week` + Handler `edit_slot_single_week`

## Decisions Made

- **Option B (separate Methode):** `modify_slot_single_week` als eigene Methode statt bool-Flag auf `modify_slot` — bestehende Tests bleiben unverändert
- **Snapshot-Strategie:** `original_snapshot = stored_slot.clone()` VOR der Mutation `stored_slot.valid_to = Some(old_slot_valid_to)` (Pitfall 2 verhindert)
- **Segment-2-valid_to:** `Some(seg2_valid_to)` [Sonntag KW] statt `original_valid_to` (Pitfall 1 verhindert — Seg2 darf nicht unbegrenzt laufen)
- **Keine utoipa-Annotation:** `edit_slot_single_week` ohne `#[utoipa::path]`, konsistent mit `edit_slot` (Phase-1-Slots bewusst nicht in OpenAPI-Bündel)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy-Fehler: `unused-mut` in `test_msw_forbidden`**
- **Found during:** Task 1 (RED build)
- **Issue:** `let mut deps = build_dependencies(false, false)` — `deps` wurde nicht mutiert (kein checkpoint/expect nötig)
- **Fix:** `let mut deps` → `let deps`
- **Files modified:** `service_impl/src/test/shiftplan_edit.rs`
- **Verification:** `cargo clippy -p service_impl --tests -- -D warnings` clean

**2. [Rule 1 - Bug] Clippy-Fehler: `type_complexity` für `Arc<Mutex<Vec<(time::Date, Option<time::Date>)>>>`**
- **Found during:** Task 1 (RED clippy)
- **Fix:** `#[allow(clippy::type_complexity)]` auf `test_msw_three_segment_structure` gesetzt
- **Files modified:** `service_impl/src/test/shiftplan_edit.rs`
- **Verification:** Clippy clean

---

**Total deviations:** 2 auto-fixed (beide Rule 1, Clippy-Warnungen)
**Impact on plan:** Minimale Korrekturen ohne Scope-Änderung.

## Issues Encountered

Keine — Plan exakt wie spezifiziert ausgeführt.

## Known Stubs

Keine — alle implementierten Methoden sind vollständig verdrahtet.

## Threat Flags

Keine neuen Sicherheitsflächen jenseits des Plan-Threat-Modells:
- T-35-01 (Elevation): `check_permission("shiftplan.edit")` als erster Aufruf ✓
- T-35-02 (Torn write): Eine tx, ein commit, alle Fehler → Rollback ✓
- T-35-03 (Doppelzählung): Partition + soft-delete + `deleted IS NULL` im DAO ✓

## Next Phase Readiness

- Plan 35-02 (Frontend-API-Anbindung) kann `modify_slot_single_week` via `PUT /shiftplan-edit/slot/{year}/{week}/single-week` aufrufen
- Mock `MockShiftplanEditService::expect_modify_slot_single_week` durch `#[automock]` automatisch generiert

---
*Phase: 35-slot-einzelwoche-aenderung*
*Completed: 2026-06-30*

## Self-Check: PASSED
