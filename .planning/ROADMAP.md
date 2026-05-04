# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (completed 2026-05-04, ready for ship)

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

### v1.1 Slot Capacity & Constraints (completed 2026-05-04)

- [x] **Phase 5: Slot Paid Capacity Warning** (6/6 plans, completed 2026-05-04)

---

### Phase 5: Slot Paid Capacity Warning

**Goal**: Slots erhalten ein optionales Capacity-Limit `max_paid_employees: Option<u8>`. Wenn der Live-Count an aktiven Bookings im Slot mit `sales_person.is_paid = true` das konfigurierte Limit übersteigt, emittiert das Backend nicht-blockierende Warnings (1) im `BookingCreateResult.warnings` analog zum v1.0-Phase-3-Pattern und (2) im Shiftplan-Week-View-Read-DTO als `current_paid_count` neben `max_paid_employees`. Buchen bleibt erlaubt — die Warning ist informativ, nicht blockierend. `NULL`-Konfiguration bedeutet „kein Limit" (kein Check, keine Warning, keine Read-Felder). Frontend ist out of scope (separater Workstream im shifty-dioxus Repo).

**Depends on**: Nothing (additive backend feature)
**Plans**: 6 plans

Plans:
- [x] 05-01-PLAN.md — Wave 1 — Migration + DAO: nullable `slot.max_paid_employees INTEGER` + `SlotEntity.max_paid_employees: Option<u8>` + all SQLite read/write sites — **completed 2026-05-04** ([SUMMARY](phases/05-slot-paid-capacity-warning/05-01-SUMMARY.md))
- [x] 05-03-PLAN.md — Wave 2 — Slot service wiring: `service::slot::Slot.max_paid_employees`, `From` impls, `SlotServiceImpl` create/update flow + 3 service tests + cross-file fixture migration in 5 test files (slot, booking, block, absence, shiftplan_edit) + 3 Rule-3 forward-compat shims for sequential-execution compile blockers — **completed 2026-05-04** ([SUMMARY](phases/05-slot-paid-capacity-warning/05-03-SUMMARY.md))
- [x] 05-04-PLAN.md — Wave 2 — Shiftplan-View read aggregation: `ShiftplanSlot.current_paid_count: u8` computed inline in `build_shiftplan_day` + 4 read tests + Plan 05-03 Rule-3 shim resolution in `test/shiftplan.rs` — **completed 2026-05-04** ([SUMMARY](phases/05-slot-paid-capacity-warning/05-04-SUMMARY.md))
- [x] 05-02-PLAN.md — Wave 3 — Service-tier Warning enum: 5th variant `Warning::PaidEmployeeLimitExceeded { slot_id, booking_id, year, week, current_paid_count, max_paid_employees }` (D-08 + D-13). Pure additive; existing 4 variants byte-preserved; `cargo build -p service` green; workspace E0004 in `rest-types/src/lib.rs:1705` deferred to Plan 05-05 (same Wave 3) — **completed 2026-05-04** ([SUMMARY](phases/05-slot-paid-capacity-warning/05-02-SUMMARY.md))
- [x] 05-05-PLAN.md — Wave 3 — REST DTO surface: extend `SlotTO.max_paid_employees: Option<u8>` (D-10) with `#[serde(default)]`, `WarningTO::PaidEmployeeLimitExceeded` 5th variant + `From<&Warning>` arm (D-08, resolves Plan 05-02 wave-coupled E0004), `ShiftplanSlotTO.current_paid_count: u8` (D-09); Plan-05-03 Rule-3 shims in this file + integration test resolved (workspace shim catalog fully closed) — **completed 2026-05-04** ([SUMMARY](phases/05-slot-paid-capacity-warning/05-05-SUMMARY.md))
- [x] 05-06-PLAN.md — Wave 3 — `ShiftplanEditService::book_slot_with_conflict_check` emits `Warning::PaidEmployeeLimitExceeded` after persistence (D-07: no rollback) when `slot.max_paid_employees.is_some()` AND `current_paid_count > max` (D-06 strict, D-15); private `count_paid_bookings_in_slot_week` helper on `ShiftplanEditServiceImpl` (Business-Logic-Tier per CLAUDE.md + v1.0 D-Phase3-18 regression-lock); 6 service-tier tests covering D-04/D-05/D-06/D-07/D-15. Legacy `POST /booking` + `BookingService::create` UNVERAENDERT (D-16, D-Phase3-18) — **completed 2026-05-04** ([SUMMARY](phases/05-slot-paid-capacity-warning/05-06-SUMMARY.md))

---

## Progress

| Phase | Milestone | Plans Complete | Status   | Completed  |
|-------|-----------|----------------|----------|------------|
| 1 — Absence Domain Foundation | v1.0 | 5/5 | Complete | 2026-05-01 |
| 2 — Reporting Integration & Snapshot Versioning | v1.0 | 4/4 | Complete | 2026-05-02 |
| 3 — Booking & Shift-Plan Konflikt-Integration | v1.0 | 6/6 | Complete | 2026-05-02 |
| 4 — Migration & Cutover | v1.0 | 8/8 | Complete | 2026-05-03 |
| 5 — Slot Paid Capacity Warning | v1.1 | 6/6  | Complete | 2026-05-04 |

---

*Last updated: 2026-05-04 — Plan 05-06 executed (last in phase): `ShiftplanEditService::book_slot_with_conflict_check` emits `Warning::PaidEmployeeLimitExceeded` after persistence (D-07: no rollback) when `slot.max_paid_employees.is_some()` AND `current_paid_count > max` (D-06 strict, D-15 NULL-skip). Private helper `count_paid_bookings_in_slot_week` lives on `ShiftplanEditServiceImpl` (Business-Logic-Tier per CLAUDE.md + v1.0 D-Phase3-18 regression-lock); reuses `get_for_week` + `get_all_paid` (both already in deps). 6 service-tier tests in `service_impl/src/test/shiftplan_edit.rs` cover D-04 (paid-only count), D-05 (absence orthogonal), D-06 (strikt-größer; equal does NOT trigger), D-07 (kein Rollback), D-15 (NULL-skip). Legacy `POST /booking` + `BookingService::create` UNVERAENDERT (D-16, D-Phase3-18) — both verified via `grep -c "PaidEmployeeLimitExceeded"` returning 0. 2 atomic jj commits (`2e13be7d` Task 1, `ef2efbe0` Task 2). Workspace `cargo build` GREEN; 461 tests pass (376 service_impl + 56 shifty_bin + 11 cutover + 10 dao + 8 other; +6 over Plan 05-05 baseline 455). `cargo run` boots cleanly to `127.0.0.1:3000`. **Phase 5 — and Milestone v1.1 — complete (6/6 plans, 100%).** Frontend-Workstream (shifty-dioxus) bleibt out of scope.*
