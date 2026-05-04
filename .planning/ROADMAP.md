# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)

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

<details>
<summary>✅ v1.1 Slot Capacity & Constraints (Phase 5) — SHIPPED 2026-05-04</summary>

- [x] **Phase 5: Slot Paid Capacity Warning** (6/6 plans) — completed 2026-05-04
  Slots erhalten ein optionales `max_paid_employees: Option<u8>` Capacity-Limit. Backend emittiert nicht-blockierende `Warning::PaidEmployeeLimitExceeded` (a) im `BookingCreateResult.warnings` im Conflict-Aware-Booking-Flow und (b) als `current_paid_count` per Slot im Shiftplan-Week-View. Buchen bleibt erlaubt (D-07: kein Rollback). NULL = kein Limit. Limit-Check lebt im Business-Logic-Tier (`ShiftplanEditService`); Legacy `POST /booking` + `BookingService::create` UNVERÄNDERT (D-Phase3-18 Regression-Lock gehalten). 461 Tests green (+6); 16/16 D-decisions verified. Frontend (shifty-dioxus) out of scope.

**Full milestone archive:** [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)

</details>

### Next milestone

Noch nicht definiert. Nächster Schritt: `/gsd:new-milestone` — Scope-Optionen aus v1.1-Deferred-Backlog:

- **Frontend-Workstream (shifty-dioxus)** — UI-Anzeige `current_paid_count` / `max_paid_employees` und Capacity-Editor in Slot-Settings
- **Min-Paid-Capacity / Skill-Matching** — weitere Slot-Constraints
- Oder andere Backend-Themen (Performance, Reporting-UX, Permissions etc.)

---

## Progress

| Phase | Milestone | Plans Complete | Status   | Completed  |
|-------|-----------|----------------|----------|------------|
| 1 — Absence Domain Foundation | v1.0 | 5/5 | Complete | 2026-05-01 |
| 2 — Reporting Integration & Snapshot Versioning | v1.0 | 4/4 | Complete | 2026-05-02 |
| 3 — Booking & Shift-Plan Konflikt-Integration | v1.0 | 6/6 | Complete | 2026-05-02 |
| 4 — Migration & Cutover | v1.0 | 8/8 | Complete | 2026-05-03 |
| 5 — Slot Paid Capacity Warning | v1.1 | 6/6 | Complete | 2026-05-04 |

---

*Last updated: 2026-05-04 — v1.1 milestone closed. Phase 5 SHIPPED. Ready for next milestone planning via `/gsd:new-milestone`.*
