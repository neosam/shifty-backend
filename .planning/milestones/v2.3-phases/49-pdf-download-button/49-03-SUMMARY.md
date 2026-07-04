---
phase: 49-pdf-download-button
plan: 03
subsystem: backend
tags: [scheduler, refactor, dry, di-wiring, phase-49, wave-2]
requires: [49-01]
provides:
  - PdfExportScheduler refactored to delegate PDF assemble to PdfShiftplanService (D-49-08)
  - RestStateDef trait extended with PdfShiftplanService assoc-type + accessor (unblocks Plan 02)
  - DI-Wiring in shifty_bin/src/main.rs (Basic → Business-Logic Order per CLAUDE.md Service-Tier-Konvention)
  - Q1-Verhaltensaenderung: Scheduler exportiert nur Weeks in {Planned, Locked} (per-Week-Skip auf ValidationError)
affects:
  - service_impl/src/pdf_export_scheduler.rs (Deps-Trim + Delegation)
  - service_impl/src/test/pdf_export_scheduler.rs (Mock-Migration + 2 neue Tests)
  - shifty_bin/src/main.rs (PdfShiftplanServiceDependencies + Konstruktion + RestStateDef-Impl-Feld/Accessor)
  - rest/src/lib.rs (RestStateDef-Trait: neuer assoc-type + accessor)
tech-stack:
  added: []
  patterns: [gen_service_impl, mockall, Arc<dyn Service> DI, jj-managed]
key-files:
  created: []
  modified:
    - rest/src/lib.rs
    - service_impl/src/pdf_export_scheduler.rs
    - service_impl/src/test/pdf_export_scheduler.rs
    - shifty_bin/src/main.rs
decisions:
  - "D-49-08 realisiert: EIN Assemble-Pfad (`PdfShiftplanService::render_week_pdf`) fuer REST-Handler und Scheduler"
  - "Q1 realisiert: ValidationError vom Service = per-Week-Skip (record_error + return Ok()); Scheduler exportiert nur Planned/Locked-Weeks"
  - "D-49-07 fuer Scheduler-Kontext: Aufrufer `Authentication::Full` (Cron-Callback ist trusted, kein User-Session)"
  - "D-49-09: DI-Order Basic → BL (shiftplan_view_service + sales_person_service + week_status_service + permission_service + transaction_dao → pdf_shiftplan_service → pdf_export_scheduler) folgt Service-Tier-Konvention"
  - "Rule-3-Fix: RestStateDef-Trait-Extension (assoc-type + accessor) auch in Plan 03 — sonst kompiliert der DI-Impl-Block in main.rs nicht standalone"
metrics:
  duration: ~30 minutes (incl. jj history recovery)
  completed: 2026-07-03
status: complete
---

# Phase 49 Plan 03: Scheduler-Refactor + DI-Wiring Summary

**One-liner:** PDF-Export-Scheduler delegiert das Assemble jetzt an den Wave-1-`PdfShiftplanService` — ein einziger DRY-Pfad fuer REST-Handler und Cron-Job, mit WeekStatus-Gate im Service-Kern und per-Week-Skip auf ValidationError.

## Objective (achieved)

Bestehende Assemble-Duplizierung im Scheduler (§347-400 in `pdf_export_scheduler.rs`) abgeraeumt — der Scheduler ruft jetzt ausschliesslich `PdfShiftplanService::render_week_pdf(shiftplan_id, y, w, Authentication::Full, None)` statt inline `shiftplan_view_service.get_shiftplan_week` + `sales_person_service.get_all` + `pdf_render::render_shiftplan_week_pdf` zu chainen. Deps aus `PdfExportSchedulerDeps` gestrichen, Tests umgebaut, DI-Wiring in `shifty_bin/src/main.rs` eingereiht.

Purpose: DRY-Aufloesung der Phase 49. Nach diesem Plan gibt es GENAU EINEN Ort fuer die PDF-Assemble-Logik (`PdfShiftplanService`), konsumiert vom REST-Handler (Plan 02) UND vom Scheduler.

## Files Changed

**Modified:**
- `service_impl/src/pdf_export_scheduler.rs` (-42 / +34 net) — `gen_service_impl!`-Block ohne `ShiftplanViewService` + `SalesPersonService`; `PdfShiftplanService` als neuer Dep-Slot. `run_once_now` ruft `pdf_shiftplan_service.render_week_pdf(...)` statt inline zu assemblen. Header-Doc-Kommentar dokumentiert Q1-Verhaltensaenderung + Verweis auf D-49-08.
- `service_impl/src/test/pdf_export_scheduler.rs` (-90 / +190 net) — `TestDeps`-Struct trimmed (2 Mock-Felder raus, 1 rein); alle 7 bestehenden Tests umgestellt auf `MockPdfShiftplanService::expect_render_week_pdf(...)`. 2 neue Tests: `scheduler_calls_pdf_shiftplan_service_with_full_auth` (D-49-07) + `scheduler_skips_week_on_validation_error` (Q1).
- `shifty_bin/src/main.rs` (+45 / -3 net) — `PdfShiftplanServiceDependencies`-Type-Alias + Konstruktion nach den Basic Services (shiftplan_view_service + sales_person_service + week_status_service + permission_service + transaction_dao) und VOR `PdfExportSchedulerService::new(...)`. `RestStateImpl`-Struct bekommt Feld `pdf_shiftplan_service: Arc<...>`. `RestStateDef`-Impl-Block bekommt `type PdfShiftplanService = ...` + `fn pdf_shiftplan_service(&self)`.
- `rest/src/lib.rs` (+10 net) — **Rule-3-Fix**: Der `RestStateDef`-Trait bekommt den neuen assoc-type `PdfShiftplanService` + accessor `pdf_shiftplan_service()`. Sonst waere Plan 03 nicht standalone kompilierbar (der `type PdfShiftplanService = ...` im `impl RestStateDef for RestStateImpl`-Block in main.rs braucht die Trait-Deklaration). Plan 02 hat denselben Trait-Bereich zusaetzlich um `mod pdf_shiftplan;`, ApiDoc-Nest + `.nest("/shiftplan", pdf_shiftplan::generate_route())` erweitert — merge-kompatibel, keine Konflikte.

**Created:** none.

**Deleted:** none.

## Tasks Executed

| # | Task | Status | Test |
|---|------|--------|------|
| 1 | Refactor Scheduler-Impl + Deps + Konstruktor | done | 9 Scheduler-Tests + `cargo build -p service_impl` green |
| 2 | Test-Anpassung `test/pdf_export_scheduler.rs` (Mock-Migration + neue Tests) | done | 9 Scheduler-Tests inkl. `boot_trigger_reload_flow` + 2 neue Tests green |
| 3 | DI-Wiring in `shifty_bin/src/main.rs` + `RestStateDef`-Trait-Extension | done | `cargo build --workspace` + `cargo test --workspace` (767/767) + `cargo clippy --workspace -- -D warnings` green |

## Verification

- `cargo build --workspace` green.
- `cargo test --workspace` — **767 passed, 0 failed** (629 service_impl inkl. 9 Scheduler-Tests + 2 neu; 11 rest + 24 rest-doctests + rest_types + service_impl integration).
- `cargo clippy --workspace -- -D warnings` green (0 Warnungen).
- Scheduler-File hat KEINE `shiftplan_view_service`/`sales_person_service`-Bezuege mehr (grep-verifiziert = 0 im non-comment-Body).
- `grep -c 'pdf_shiftplan_service' service_impl/src/pdf_export_scheduler.rs` = 5 (>= 3 required).
- `grep -c 'render_week_pdf' service_impl/src/pdf_export_scheduler.rs` = 2 (>= 1 required).
- `grep -c 'pdf_shiftplan_service' shifty_bin/src/main.rs` = 6 (>= 4 required).
- `grep -c 'PdfShiftplanServiceDependencies' shifty_bin/src/main.rs` = 4 (>= 2 required).

## Success Criteria (all achieved)

- [x] Ein einziger PDF-Assemble-Pfad: `PdfShiftplanService::render_week_pdf` konsumiert von REST-Handler (Plan 02) UND Scheduler (Plan 03).
- [x] Scheduler-Deps minimiert; DRY-Anspruch der Phase erfuellt (`ShiftplanViewService` + `SalesPersonService` restlos entfernt, `PdfShiftplanService` neu).
- [x] DI-Reihenfolge in main.rs konform mit CLAUDE.md Service-Tier-Konvention (Basic → Business-Logic).
- [x] Q1-Verhaltensaenderung (Scheduler exportiert nur Planned/Locked) dokumentiert im Scheduler-Header-Kommentar mit Verweis auf D-49-08.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] RestStateDef-Trait-Erweiterung in Plan 03**
- **Found during:** Task 3 (DI-Wiring standalone-build).
- **Issue:** Plan 03 sollte laut Plan-Text nur den `RestStateDef`-Impl-Block in main.rs mit `type PdfShiftplanService = ...` + accessor erweitern. Aber der Trait selbst (in `rest/src/lib.rs`) hatte die zugehoerige assoc-type-Deklaration + Accessor-Signatur noch NICHT. Der Impl-Block laesst sich ohne die Trait-Deklaration nicht kompilieren (`E0437: not a member of trait`).
- **Fix:** Trait-Extension in `rest/src/lib.rs` (assoc-type + accessor-signature) auch in Plan 03 landed — 10 Zeilen additiv, keine Kollateralaenderung. Plan 02 fuegt spaeter im gleichen File `mod pdf_shiftplan;` + ApiDoc-Nest + Router-Nest hinzu, das ist merge-kompatibel (disjoint patch regions).
- **Files modified:** `rest/src/lib.rs` (RestStateDef Zeile 430-438 assoc-type + Zeile 477 accessor).
- **Commit:** enthalten im `refactor(49-03): scheduler consumes PdfShiftplanService — DRY assemble path` (siehe git log).

### Deferred Items

- **Pre-existing Clippy-Lint in `service_impl/src/test/shiftplan_edit_lock.rs:6`** (doc_lazy_continuation) — nicht durch Plan 03 verursacht, tritt nur bei `cargo clippy --tests` auf. `cargo clippy --workspace -- -D warnings` (non-test targets) ist green. Ausserhalb Plan 03 Scope; sollte in einem separaten HYG-Task adressiert werden.

## Auth Gates

Keine — Plan 03 ist rein interne DI/Refactor-Arbeit ohne User-facing Auth-Flows.

## Threat Model Status

| Threat ID | Mitigation Status |
|-----------|-------------------|
| T-49-03 (Tampering / Input Validation, Scheduler-Loop) | **mitigated** — WeekStatus-Gate im `PdfShiftplanService` filtert Unset/InPlanning bereits im Service-Kern; per-Week-Skip via `record_error` + `return Ok(())` im Scheduler. Verifiziert per Test `scheduler_skips_week_on_validation_error`. |
| T-49-INFO (Deps-Streichung) | **accepted** — Entfernte `ShiftplanViewService`/`SalesPersonService`-Deps sind rein interne Refactor-Aenderung, keine externe Surface-Erweiterung. |

## Known Stubs

None. Plan 03 ersetzt Duplikat-Code durch Service-Delegation; alle Test-Assertions decken Verhalten deterministisch ab (kein hardcoded Mock-Data-Stub in Prod-Code).

## Self-Check

**Commits:**
- `8ce3781a` (git) / `xxwsylxr` (jj) `refactor(49-03): scheduler consumes PdfShiftplanService — DRY assemble path` — FOUND in `jj log --limit 10`.

**Files verified:**
- FOUND: `service_impl/src/pdf_export_scheduler.rs` (modified, 76 lines diff).
- FOUND: `service_impl/src/test/pdf_export_scheduler.rs` (modified, 280 lines diff).
- FOUND: `shifty_bin/src/main.rs` (modified, 54 lines diff).
- FOUND: `rest/src/lib.rs` (modified, 10 lines diff — Rule-3-Fix).

## Self-Check: PASSED
