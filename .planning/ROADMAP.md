# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- 🚧 **v1.1 Slot Capacity & Constraints** — Phase 5 (in planning)

## Phases

<details>
<summary>✅ v1.0 Range-Based Absence Management (Phasen 1–4) — SHIPPED 2026-05-03</summary>

- [x] **Phase 1: Absence Domain Foundation** (5/5 plans) — completed 2026-05-01
  Neue parallele `absence` Domain (DAO + Service + REST + Permission), additiv, ohne Reporting-Wirkung
- [x] **Phase 2: Reporting Integration & Snapshot Versioning** (4/4 plans) — completed 2026-05-02
  `derive_hours_for_range` + Reporting-Switch hinter Feature-Flag, `CURRENT_SNAPSHOT_SCHEMA_VERSION` 2 → 3 im selben Commit
- [x] **Phase 3: Booking & Shift-Plan Konflikt-Integration** (6/6 plans) — completed 2026-05-02
  Forward/Reverse Booking-Warnings + Shift-Plan-Anzeige aus AbsencePeriod ohne Doppel-Eintragung
- [x] **Phase 4: Migration & Cutover** (8/8 plans) — completed 2026-05-03
  Heuristik-Migration, Validierungs-Gate (< 0.01h Drift-Toleranz), atomarer Feature-Flag-Flip mit Carryover-Refresh, REST-Deprecation. Plus Bonus-Recovery von `extra_hours.update` mit logical_id-Versionierung.

**Full milestone archive:** [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)

</details>

### v1.1 Slot Capacity & Constraints (in planning)

- [ ] **Phase 5: Slot Paid Capacity Warning** (in progress, 2/6 plans)

---

### Phase 5: Slot Paid Capacity Warning

**Goal**: Slots erhalten ein optionales Capacity-Limit `max_paid_employees: Option<u8>`. Wenn der Live-Count an aktiven Bookings im Slot mit `sales_person.is_paid = true` das konfigurierte Limit übersteigt, emittiert das Backend nicht-blockierende Warnings (1) im `BookingCreateResult.warnings` analog zum v1.0-Phase-3-Pattern und (2) im Shiftplan-Week-View-Read-DTO als `current_paid_count` neben `max_paid_employees`. Buchen bleibt erlaubt — die Warning ist informativ, nicht blockierend. `NULL`-Konfiguration bedeutet „kein Limit" (kein Check, keine Warning, keine Read-Felder). Frontend ist out of scope (separater Workstream im shifty-dioxus Repo).

**Depends on**: Nothing (additive backend feature)
**Plans**: 6 plans

Plans:
- [x] 05-01-PLAN.md — Wave 1 — Migration + DAO: nullable `slot.max_paid_employees INTEGER` + `SlotEntity.max_paid_employees: Option<u8>` + all SQLite read/write sites — **completed 2026-05-04** ([SUMMARY](phases/05-slot-paid-capacity-warning/05-01-SUMMARY.md))
- [x] 05-03-PLAN.md — Wave 2 — Slot service wiring: `service::slot::Slot.max_paid_employees`, `From` impls, `SlotServiceImpl` create/update flow + 3 service tests + cross-file fixture migration in 5 test files (slot, booking, block, absence, shiftplan_edit) + 3 Rule-3 forward-compat shims for sequential-execution compile blockers — **completed 2026-05-04** ([SUMMARY](phases/05-slot-paid-capacity-warning/05-03-SUMMARY.md))
- [ ] 05-04-PLAN.md — Wave 2 — Shiftplan-View read aggregation: `ShiftplanSlot.current_paid_count: u8` computed inline in `build_shiftplan_day` + 4 read tests + Slot fixture migration in `test/shiftplan.rs` (Plan 04 owns this file's fixtures since it adds the new tests there)
- [ ] 05-02-PLAN.md — Wave 3 — Service-tier Warning enum: add 5th variant `Warning::PaidEmployeeLimitExceeded` (lands together with 05-05 to keep workspace build green: rest-types `From<&Warning>` arm is exhaustive without wildcard)
- [ ] 05-05-PLAN.md — Wave 3 — REST DTO surface: extend `SlotTO`, `WarningTO` (5th variant + From-arm), `ShiftplanSlotTO` in `rest-types/src/lib.rs`
- [ ] 05-06-PLAN.md — Wave 3 — `ShiftplanEditService` warning emission in `book_slot_with_conflict_check` + private `count_paid_bookings_in_slot_week` helper + 6 booking-pfad tests

---

## Progress

| Phase | Milestone | Plans Complete | Status   | Completed  |
|-------|-----------|----------------|----------|------------|
| 1 — Absence Domain Foundation | v1.0 | 5/5 | Complete | 2026-05-01 |
| 2 — Reporting Integration & Snapshot Versioning | v1.0 | 4/4 | Complete | 2026-05-02 |
| 3 — Booking & Shift-Plan Konflikt-Integration | v1.0 | 6/6 | Complete | 2026-05-02 |
| 4 — Migration & Cutover | v1.0 | 8/8 | Complete | 2026-05-03 |
| 5 — Slot Paid Capacity Warning | v1.1 | 2/6  | In progress | —          |

---

*Last updated: 2026-05-04 — Plan 05-03 (Wave 2) executed: `service::slot::Slot` carries `max_paid_employees: Option<u8>`, both `From` impls bridge entity↔domain, `SlotServiceImpl` create/update propagate the field via existing `..slot.clone()` spread (no production-code change), 5 owned test files migrated, 3 new tests verify create-with-limit / update-changes / update-clears semantics, permission Pflicht-Test covered transitively via existing `SHIFTPLANNER_PRIVILEGE` gate. Plan 05-01's 2 Rule-3 shims replaced with real flow. 3 new Rule-3 shims placed at out-of-scope compile blockers (`test/shiftplan.rs`, `rest-types/src/lib.rs::From<&SlotTO>`, `shifty_bin/.../booking_absence_conflict.rs`) — Plan 05-04 / 05-05 will replace. 451+ tests green workspace-wide. Next: Plan 05-04 (Shiftplan-View read aggregation).*
