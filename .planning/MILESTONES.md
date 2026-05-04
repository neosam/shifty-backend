# Milestones — Shifty Backend

Historischer Index aller geshipten Milestones. Jeder Eintrag verlinkt auf die Detail-Archive in `.planning/milestones/`.

---

## v1.0 — Range-Based Absence Management

**Shipped:** 2026-05-03
**Phases:** 1–4 (4 phases, 23 plans, 22 SUMMARYs)
**Archive:** [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)

**Delivered:**
Zeitraum-basierte Abwesenheits-Domäne (Vacation/SickLeave/UnpaidLeave) ersetzt die wochenweise `ExtraHours`-Buchhaltung. Per-Tag-Stunden werden aus dem am jeweiligen Tag gültigen Vertrag abgeleitet (`derive_hours_for_range`); doppelte Eintragung zwischen ExtraHours und Shift-Plan entfällt. Komplette Bestandsdaten-Migration mit atomarer Cutover-Tx hinter Validierungs-Gate (Toleranz < 0.01h Drift).

**Key accomplishments:**
1. **Phase 1** — `absence_period`-Schema + DAO + Service + REST + 8 Integration-Tests; additiv ohne Reporting-Effekt; logical_id-Versionierung etabliert (später auch in `extra_hours` übernommen)
2. **Phase 2** — `derive_hours_for_range` (per-Tag-Vertrags-Lookup mit Feiertags-Orthogonalität), FeatureFlagService-Infrastruktur, atomarer Snapshot-Bump 2→3 + Reporting-Switch in einem Commit
3. **Phase 3** — Forward-/Reverse-Booking-Warnings über AbsencePeriod, ShiftplanView-Marker per-sales-person, BookingService-Files unangetastet (D-Phase3-18 Regression-Lock 0-Diff)
4. **Phase 4** — Heuristik-Cluster-Migration (Strict-Match + 5 Quarantäne-Reasons), atomarer Cutover (Backup → Carryover-Rebuild → Soft-Delete → Flag-Flip), 3 REST-Endpoints `/admin/cutover/{gate-dry-run,commit,profile}`, OpenAPI-Snapshot-Pin (160 KB, 3-Run-deterministic)
5. **Phase 4 Bonus** — `ExtraHoursService::update` mit logical_id-Rotation + REST `PUT /extra-hours/{id}` (recovered via jj-rebase nach fälschlicher phantom-Migration-Diagnose in Plan 04-02)

**Test verification:** 458+ tests green workspace-wide. OpenAPI snapshot deterministic. Cold-start smoke pass.

**Known deferred items:**
- 04-UAT Test 8 (idempotenter Re-Run nach Commit): manuell 403 erhalten, vermutlich Setup-Issue (kein cutover_admin-Grant in dev-DB); Code-Pfad abgedeckt durch passing Integration-Test `test_idempotence_rerun_no_op`.
- `/gsd:secure-phase 04` wurde nicht ausgeführt — als bewusstes Skip akzeptiert (Threats in Plan-SUMMARYs durchgängig als mitigated/accepted dokumentiert).

**Recovery note:**
Während Plan 04-02 wurde der frühere Commit `fe744df` (logical_id für extra_hours + PUT-Endpoint) fälschlich als "phantom from never-committed branch" interpretiert und seine Migration aus dem Workspace entfernt. Während des UAT-Reviews bemerkt, recovered via `jj rebase -r fe744dff -d @-`, Konflikte gelöst, OpenAPI-Snapshot re-akzeptiert. Im aktuellen v1.0-Lineup als Commit `psknryoq` (vor dem Phase-4-Verifikations-Commit) integriert.

---

## v1.1 — Slot Capacity & Constraints

**Shipped:** 2026-05-04
**Phases:** 5 (1 phase, 6 plans, 6 SUMMARYs)
**Archive:** [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)

**Delivered:**
Slots erhalten ein optionales Capacity-Limit `max_paid_employees: Option<u8>`. Wenn der Live-Count an aktiven Bookings im Slot mit `sales_person.is_paid = true` das konfigurierte Limit übersteigt, emittiert das Backend nicht-blockierende `Warning::PaidEmployeeLimitExceeded` über (a) `BookingCreateResult.warnings` im Conflict-Aware-Booking-Endpoint und (b) `current_paid_count` neben `max_paid_employees` per Slot im Shiftplan-Week-View. Buchen bleibt erlaubt — die Warning ist informativ (D-07: kein Tx-Rollback). `NULL`-Konfiguration bedeutet "kein Limit". Backend-additiv über alle Layer (SQLite-Migration, DAO, Service, REST). Frontend out of scope.

**Key accomplishments:**
1. **DAO Foundation** (Plan 05-01) — Nullable `slot.max_paid_employees INTEGER` ohne DEFAULT/NOT NULL, `SlotEntity.max_paid_employees: Option<u8>`, 4 SELECTs + INSERT + UPDATE in `dao_impl_sqlite/src/slot.rs`; Forward-Compat-Shims (Rule 3) für sequenzielle-Wave-2-Compile-Blocker.
2. **Service-Tier Wiring** (Plans 05-03, 05-04) — `service::slot::Slot.max_paid_employees`, `From` impls bridge DAO ↔ Service; `ShiftplanSlot.current_paid_count: u8` inline-derived in `build_shiftplan_day` aus bereits resolvten Bookings (Read-Aggregation-Pattern); Fixture-Migration in 5 Test-Files.
3. **Warning Surface** (Plans 05-02, 05-05) — 5. `Warning::PaidEmployeeLimitExceeded`-Variante mit 6 strukturierten Feldern (slot_id, booking_id, year, week, current_paid_count, max_paid_employees); REST-DTO-Mirror auf `SlotTO`/`WarningTO`/`ShiftplanSlotTO` mit `#[serde(default)]`-Backward-Compat (Wire-Tier-Mirror-Pattern); Wave-Coupling-Pattern für Producer-Consumer-Plans.
4. **Warning Emission** (Plan 05-06) — `ShiftplanEditService::book_slot_with_conflict_check` emittiert `Warning::PaidEmployeeLimitExceeded` nach Persistence (D-07: kein Rollback) wenn `current_paid_count > max` (D-06 strict, D-15 NULL-skip). Privater Helper `count_paid_bookings_in_slot_week` lebt im Business-Logic-Tier (D-12 + CLAUDE.md Service-Tier-Konvention + v1.0 D-Phase3-18 Regression-Lock).
5. **Regression-Lock gehalten** — Legacy `POST /booking` + `BookingService::create` UNVERÄNDERT (D-16, D-Phase3-18); verifiziert via `grep -c "PaidEmployeeLimitExceeded" service_impl/src/booking.rs` = 0 + `rest/src/booking.rs` = 0.

**Test verification:** 461 tests green workspace-wide (+6 über v1.0-Baseline 455). `cargo build --workspace` GREEN. `cargo run` boots cleanly to `127.0.0.1:3000`. `05-VERIFICATION.md`: 16/16 D-decisions verified, status: passed (gaps_remaining = []).

**Known deferred items:**
- **Frontend-Workstream (shifty-dioxus)** — bewusst out of scope. Capacity-Editor in Slot-Settings + UI-Anzeige `current_paid_count` / `max_paid_employees` folgen in separatem Repo.
- **Min-Paid-Capacity / Skill-Matching** — gemerkt für künftiges Backend-Milestone.

---
