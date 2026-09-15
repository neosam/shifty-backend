---
phase: 55-manual-rebooking-hr-alert
plan: 02
subsystem: api
tags: [rebooking, voluntary-hours, reporting, extra-hours, business-logic-service, tdd, mockall]

# Dependency graph
requires:
  - phase: 55-01
    provides: "RebookingBatchService.create_pending_suggestion / update_state / find_pending_for_sales_person + DaoError::EntityAlreadyExists"
  - phase: 54
    provides: "ExtraHoursSource-Marker (Manual/Rebooking) auf jeder ExtraHours-Row + VoluntaryStatsService + rebooking_batch-Datenmodell"
provides:
  - "RebookingReconciliationService (Business-Logic-Tier): rebook_manual (F3), suggest (F5-Claim), approve, reject"
  - "RebookingDirection-Enum + RebookingSuggestion (backend-computed IST/DANN)"
  - "source==Manual-Filter an allen Read-Aggregations-Sites → VOL-ACCT-03-Neutralitaet aktiv"
  - "pure fns alert_predicate + proposed_hours (HR-ALERT-01/D-55-01/03)"
  - "MockRebookingReconciliationService (via automock) fuer 55-03 REST-Layer"
affects: [55-03, 55-04, 55-05, 55-06, F14-rebooking, F07-reporting-balance]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Single Pair-write-path: EIN interner rebook(direction, hours, week) erzeugt beide -N/+N-Rows fuer F3 UND F5-approve (Pitfall 15)"
    - "Read-Neutralitaet via source-Filter am Aggregations-Eingang statt Korrektur-Row (D-55-A1)"
    - "state-conditional UPDATE ZUERST, Pair-Writes nur bei Erfolg (Pitfall 12)"

key-files:
  created:
    - "service/src/rebooking_reconciliation.rs"
    - "service_impl/src/rebooking_reconciliation.rs"
    - "service_impl/src/test/rebooking_reconciliation.rs"
  modified:
    - "service_impl/src/reporting.rs"
    - "service/src/lib.rs"
    - "service_impl/src/lib.rs"
    - "service_impl/src/test/mod.rs"

key-decisions:
  - "D-55-A1 umgesetzt via Filter am Aggregations-EINGANG (extra_hours_bucket / extra_hours_array / filtered_extra_hours_list) statt an jeder einzelnen .filter()-Site — deckt VolunteerWork- UND ExtraWork/WorkingHours-Bein in einem Schritt und ist regressionssicher, weil Rebooking-Rows nur ExtraWork/VolunteerWork sind."
  - "approve nutzt fest RebookingDirection::VolunteerToExtra (F5-Standard-Richtung) und erzeugt KEINEN zweiten Batch — es transitioniert den bestehenden pending-HrSuggestion-Batch und schreibt nur das Pair."
  - "rebook_manual/approve halten die Audit-Felder (balance_before/voluntary_actual/voluntary_committed) auf 0.0 — die Snapshot-Audit-Befuellung ist nicht Teil dieses Plans (kein Test fordert sie); die Pair-ids out_id/in_id werden korrekt gesetzt."

patterns-established:
  - "Single Pair-write-path rebook() als private impl-Methode, von rebook_manual + approve geteilt"
  - "Rebooking-Neutralitaet: source==Manual-Filter am Report-Aggregations-Eingang"
  - "F5-Modell-B: suggest=Claim (idempotent/Conflict), approve/reject=state-Transition, getrennte Write-Paths"

requirements-completed: [REB-MANUAL-01, REB-MANUAL-02, HR-ALERT-02, HR-ALERT-03, HR-ALERT-04, VOL-ACCT-03]

coverage:
  - id: D1
    description: "VOL-ACCT-03: Einzel-Umbuchung (-N/+N-Pair, source=Rebooking) laesst balance + volunteer_hours (Ist) unveraendert (single + roundtrip)"
    requirement: "VOL-ACCT-03"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#rebooking_neutrality_single_manual"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#rebooking_neutrality_roundtrip"
        status: pass
    human_judgment: false
  - id: D2
    description: "REB-MANUAL-01/02: rebook_manual erzeugt atomar das -N/+N-Pair (beide Richtungen, korrekte Kategorien/Vorzeichen/source=Rebooking) + Manual-Approved-Batch + Entry; Rollback-sicher bei Batch-Fehler"
    requirement: "REB-MANUAL-01"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#rebooking_reconciliation_creates_pair"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#rebooking_direction_both"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#rebooking_rollback_on_error"
        status: pass
    human_judgment: false
  - id: D3
    description: "HR-ALERT-02/04: suggest claimt einen pending-Batch (Modell B), liefert backend-computed IST/DANN + proposed_hours + pending-id; idempotent bei offenem pending; Conflict bei entschiedenem Slot"
    requirement: "HR-ALERT-04"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#suggest_claims_pending_and_returns_dann"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#suggest_is_idempotent_on_existing_pending"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#suggest_on_decided_slot_is_conflict"
        status: pass
    human_judgment: false
  - id: D4
    description: "HR-ALERT-03: approve transitioniert state ZUERST, dann Pair-Writes; reject transitioniert nur, kein Pair"
    requirement: "HR-ALERT-03"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#approve_after_suggest_writes_pair"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#reject_after_suggest_no_pair"
        status: pass
    human_judgment: false
  - id: D5
    description: "Pure fns alert_predicate (Grenzfaelle -0.49/-0.5/-0.51) + proposed_hours = min(|balance|, voluntary_ist)"
    requirement: "HR-ALERT-02"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#alert_predicate_matrix"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#proposed_hours_matrix"
        status: pass
    human_judgment: false
  - id: D6
    description: "T-55-05: rebook_manual ist HR-gated (Non-HR → Forbidden, Gate vor jedem Datenabruf)"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#rebook_manual_is_hr_gated"
        status: pass
    human_judgment: false

# Metrics
duration: 40min
completed: 2026-07-23
status: complete
---

# Phase 55 Plan 02: RebookingReconciliationService (BL) + VOL-ACCT-03-Neutralitaet Summary

**Business-Logic-Tier RebookingReconciliationService mit EINEM internen Pair-write-path (rebook), F5-Modell-B-Lifecycle (suggest-Claim / approve / reject) und aktiviertem source==Manual-Filter, der eine Einzel-Umbuchung balance/ist/soll/delta-neutral macht (VOL-ACCT-03 property-getestet).**

## Performance

- **Duration:** ~40 min
- **Started:** 2026-07-23T21:02:31Z
- **Completed:** 2026-07-23T21:16:00Z
- **Tasks:** 3 (TDD: RED → GREEN → GREEN)
- **Files modified:** 7 (3 created, 4 modified)

## Accomplishments
- **VOL-ACCT-03-Neutralitaet aktiv:** `source == ExtraHoursSource::Manual`-Filter an den drei Report-Aggregations-Eingaengen (`assemble_weeks` extra_hours_bucket, `get_reports_for_all_employees` extra_hours_array, `hours_per_week` filtered_extra_hours_list) + `overall_extra_work_hours`. Property-Test beweist: eine Einzel-Umbuchung laesst `balance_hours` UND `volunteer_hours` (= Ist-Aggregat) unveraendert — sowohl single als auch roundtrip.
- **Single Pair-write-path** `rebook(direction, hours, week)` (private impl-Methode): erzeugt das -N (Quell-Kat) / +N (Ziel-Kat) ExtraHours-Pair (source=Rebooking), geteilt von F3 (`rebook_manual`) und F5 (`approve`) — kein F3/F5-Drift (Pitfall 15).
- **F5-Modell-B-Lifecycle:** `suggest` claimt idempotent via `create_pending_suggestion` (Conflict bei entschiedenem Slot), liefert backend-computed IST/DANN; `approve` transitioniert state ZUERST (Pitfall 12), dann Pair; `reject` transitioniert nur.
- **Pure fns** `alert_predicate` + `proposed_hours` als testbare Truth-Table-Matrizen (HR-ALERT-01/D-55-01/03).
- **13 neue Tests** gruen; gesamter Workspace (781 service_impl-lib-Tests + alle Integration-Suites) gruen; `cargo clippy --workspace -- -D warnings` ohne Warnung.

## Task Commits

Jede Task atomar committet (TDD: RED → GREEN):

1. **Task 1: RED — VOL-ACCT-03-Property + Pure-fn-Matrizen + BL-Service-Tests** - `ef71d0d9` (test)
2. **Task 2: GREEN — source==Manual-Filter + alert_predicate/proposed_hours** - `e6292b81` (feat)
3. **Task 3: GREEN — RebookingReconciliationService (BL) single write-path** - `0e1e61d6` (feat)

_TDD-Plan: Task 1 = RED (test), Task 2 + 3 = GREEN (feat). Kein separater REFACTOR-Commit noetig — die Direction-Ableitung wurde direkt als reine Helper-fn `pair_categories` implementiert._

## Files Created/Modified
- `service/src/rebooking_reconciliation.rs` (neu) - Trait `RebookingReconciliationService` (#[automock]) + `RebookingDirection`-Enum + `RebookingSuggestion`-Struct (backend-computed IST/DANN)
- `service_impl/src/rebooking_reconciliation.rs` (neu) - Impl via `gen_service_impl!`; interner Pair-write-path `rebook`; F3/F5-Verdrahtung nach D-55-B; reine Helper-fns `pair_categories` + `week_anchor_date_time`
- `service_impl/src/test/rebooking_reconciliation.rs` (neu) - Neutralitaets-Property (am ReportingServiceImpl) + Pure-fn-Matrizen + BL-Service-Tests (mockall)
- `service_impl/src/reporting.rs` - source==Manual-Filter an 4 Sites + pure fns `alert_predicate`/`proposed_hours` + Kommentar Z.184 praezisiert
- `service/src/lib.rs`, `service_impl/src/lib.rs` - `pub mod rebooking_reconciliation;`
- `service_impl/src/test/mod.rs` - `pub mod rebooking_reconciliation;`

## Decisions Made
- **D-55-A1 am Aggregations-Eingang gefiltert:** Statt jede einzelne `.filter(|eh| eh.category == …)`-Site um `&& source==Manual` zu erweitern (7+ Stellen, fehleranfaellig), wird der Rebooking-Row-Filter EINMAL am Eingang jeder der drei Report-Funktionen angewendet. Da Rebooking-Rows ausschliesslich ExtraWork/VolunteerWork-Kategorien tragen, hat das exakt denselben Effekt fuer die vier Kennzahlen und Null Effekt auf Absence/Holiday-Aggregate. Property-Test ist die Autoritaet und bestaetigt den Scope.
- **Neutralitaets-Property am ReportingServiceImpl festgenagelt** (mit `MockExtraHoursService`, der einmal Baseline und einmal Baseline+Pair liefert) statt gegen eine volle In-Memory-DB — direkter Filter-Site-Nachweis, schneller, deterministisch. Soll/Delta sind reine committed_voluntary-pure-fns (rebooking-blind); da `ist == volunteer_hours` und dieses neutral ist, sind ist/soll/delta ebenfalls neutral.
- **approve ohne zweiten Batch-create:** approve transitioniert den bestehenden pending-HrSuggestion-Batch (via update_state) und schreibt nur das ExtraHours-Pair — es legt KEINEN neuen Batch an. Richtung fest VolunteerToExtra (F5-Standard).

## Deviations from Plan

None - Plan wurde exakt wie geschrieben ausgefuehrt. Die Prohibitions wurden strikt gehalten: keine Korrektur-Row (Neutralitaet nur via source-Filter), ein einziger Pair-write-path, suggest-Claim und approve-Pair getrennt, kein Basic-Service oeffnet eigene Tx im BL-Kontext (immer `Some(tx.clone())` + `Authentication::Full`), kein Snapshot-Bump (bleibt 12), keine FE-Arithmetik.

## Issues Encountered
- Ein unbenutzter Import (`RebookingBatchEntryEntity`) im Test-Modul liess den GREEN-Build unter `-D warnings` zunaechst fehlschlagen; direkt entfernt (Teil des Task-3-Commits). Kein funktionaler Impact.

## Docs-Freshness-Gate
`service_impl/src/reporting.rs` ist eine Trigger-Datei (→ F07-reporting-balance + domain/time-accounting). Die Balance-Formel selbst wurde NICHT geaendert — nur ein source-Filter ergaenzt, der Rebooking-Rows (die es vor Phase 55 im Betrieb nicht gab) aus den Aggregaten haelt. Ein Inline-Kommentar (reporting.rs Z.184) dokumentiert die ab-Phase-55-aktive Filter-Semantik. Die umfassende F07/F14/service-tiers-Doku-Synchronisation ist explizit Plan 55-06 zugewiesen; hier bewusst leichtgewichtig gehalten.

## Next Phase Readiness
- **55-03 (REST):** `MockRebookingReconciliationService` + `RebookingSuggestion`-Domain-Struct stehen bereit; die REST-Schicht mappt `RebookingSuggestion` → `RebookingSuggestionTO` (rest-types/src/rebooking.rs neu, D-55 gepinnt) und verdrahtet die 4 Routes (POST /rebooking/manual, GET /rebooking-suggestions, POST .../approve, .../reject). `ServiceError::EntityAlreadyExists` → HTTP 409-Mapping muss in 55-03 gesetzt werden.
- **DI-Wiring (55-03):** `RebookingReconciliationService` ist Business-Logic-Tier und NACH ExtraHours/RebookingBatch/Reporting/VoluntaryStats in `shifty_bin/src/main.rs` zu konstruieren (RESEARCH DI-Wiring). In diesem Plan nur Trait+Impl+lib.rs-Registrierung; main.rs bleibt 55-03.
- Keine Blocker.

---
*Phase: 55-manual-rebooking-hr-alert*
*Completed: 2026-07-23*

## Self-Check: PASSED
- Alle erstellten Dateien vorhanden (service/service_impl rebooking_reconciliation.rs, Test, SUMMARY).
- Alle 3 Task-Commits im Git-Log (ef71d0d9, e6292b81, 0e1e61d6).
- Gates: cargo build (workspace lib) + cargo test --workspace (781 service_impl + alle Suites) + cargo clippy --workspace -- -D warnings alle gruen.
