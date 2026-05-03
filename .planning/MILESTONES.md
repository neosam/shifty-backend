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
