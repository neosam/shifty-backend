---
phase: 55-manual-rebooking-hr-alert
plan: 01
subsystem: api
tags: [rust, axum, sqlx, service-tier, business-logic, rebooking, hr-alert, mockall, tdd-lite]

requires:
  - phase: 54
    provides: "rebooking_batch-DAO + Basic-Service + UNIQUE-Slot-Index; ExtraHours.source-Marker (Manual/Rebooking)"

provides:
  - "service/src/rebooking_reconciliation.rs: Trait RebookingReconciliationService (6 async Methoden) + RebookingDirection enum + RebookingSuggestion Domain-Struct + 2 pure fns (alert_predicate, proposed_rebooking_hours)"
  - "service_impl/src/rebooking_reconciliation.rs: BL-Impl RebookingReconciliationServiceImpl mit gen_service_impl!, orchestriert ExtraHoursService + RebookingBatchService + ReportingService in einer Transaktion"
  - "dao/src/rebooking_batch.rs: DAO-Trait erweitert um find_pending_for_sales_person + list_all_pending + update_state_conditional"
  - "dao_impl_sqlite/src/rebooking_batch.rs: SQLx-Impls fuer die drei neuen DAO-Methoden inkl. .sqlx offline cache"
  - "service/src/rebooking_batch.rs + service_impl/src/rebooking_batch.rs: HR-gated Service-Wrapper fuer die drei neuen DAO-Methoden"
  - "service_impl/src/reporting.rs: zentraler `source == Rebooking`-Filter an allen vier extra_hours-Fetches (VOL-ACCT-03 Wave-1-Owner)"
  - "service/src/lib.rs: ServiceError::BatchAlreadyResolved (HR-ALERT-03 Race-Fehler)"
  - "rest/src/lib.rs: BatchAlreadyResolved -> HTTP 409 Conflict"

affects:
  - "Plan 55-02 REST-Endpoints (POST /rebooking/manual, GET /rebooking-suggestions, approve/reject) — konsumiert diesen Service"
  - "Plan 55-03 Property-Test Rebooking-Neutralitaet — nutzt VOL-ACCT-03-Filter als System-Under-Test"
  - "Plan 55-04/05 FE-Komponenten — indirekt via Plan 55-02 DTOs"
  - "shifty_bin/src/main.rs — DI-Wiring RebookingReconciliationServiceImpl kommt in Plan 55-02"

tech-stack:
  added: []
  patterns:
    - "BL-Service konsumiert 3 Domain-Services (Basic + BL) mit sauberer Konstruktions-Reihenfolge — kein Zyklus"
    - "state-conditional UPDATE als DAO-Verb mit rows_affected-Rueckgabe (kein Fehler bei 0 auf Basic-Tier)"
    - "Pair-ExtraHours-Semantik mit ExtraHoursSource::Rebooking-Marker + zentralem Filter in Reporting-Aggregat"
    - "Zwei pure fns (alert_predicate, proposed_rebooking_hours) als eigenstaendige Domain-Regeln, unit-testbar ohne Service-Mocks"

key-files:
  created:
    - "service/src/rebooking_reconciliation.rs"
    - "service_impl/src/rebooking_reconciliation.rs"
    - "service_impl/src/test/rebooking_reconciliation.rs"
    - ".sqlx/query-3274b39e1e957625bc786e7e720f17e6c39aa37c0f4ee1ee996db980363424c2.json"
    - ".sqlx/query-4fb6dc270b2a26e37bc2aea07de36208963d33f59851009a46f4d71ef51957c6.json"
    - ".sqlx/query-e234e4e1e518d843bd89c9c2756a9642f1b6e13d9fb7d13cae643e015031c948.json"
  modified:
    - "service/src/lib.rs"
    - "service/src/rebooking_batch.rs"
    - "service_impl/src/lib.rs"
    - "service_impl/src/rebooking_batch.rs"
    - "service_impl/src/reporting.rs"
    - "service_impl/src/test/mod.rs"
    - "dao/src/rebooking_batch.rs"
    - "dao_impl_sqlite/src/rebooking_batch.rs"
    - "rest/src/lib.rs"

key-decisions:
  - "D-55-01: alert_predicate ist `cap_active && balance <= -0.5 && voluntary_ist > 0.0` (Float-Noise-Tolerance)."
  - "D-55-03: proposed_rebooking_hours = `min(|balance|, voluntary_ist).max(0.0)`; RebookingSuggestion enthaelt voluntary_delta_before/after als Backend-computed Felder (Fat-Backend)."
  - "D-55-05: iso_year/iso_week sind HR-gewaehlt — retrospektive Buchungen erlaubt; UNIQUE-Slot-Kollision propagiert als EntityAlreadyExists."
  - "D-55-07: reject persistiert ohne ExtraHours-Schreiben; UNIQUE-Slot bleibt bis zur naechsten ISO-Woche belegt."
  - "Task 3 Trade-off: hydrate_pending_to_suggestion setzt voluntary_soll_before=0.0. Der vollstaendige Soll-Aggregator kommt aus VoluntaryStatsService (Business-Logic-Tier); den in RebookingReconciliationService als Dep zu ziehen wuerde einen Cross-BL-Cycle-Kandidaten schaffen. Plan 55-02 haengt VoluntaryStatsService im REST-Handler-Layer nach, wenn das FE echte Soll-Zahlen anzeigen will. Die Fat-Backend-Regel D-55-03 bleibt gewahrt: Delta-Felder werden weiterhin Backend-berechnet."
  - "Reporting-Filter (VOL-ACCT-03) greift zentral in ReportingService — kein per-Konsument-Filter noetig, weil VoluntaryStatsService/BookingInformationService bereits ueber ReportingService::get_report_for_employee* aggregieren."

patterns-established:
  - "Direction-Enum + Helper build_pair_payloads: Positive Menge + Direction-Enum vermeidet Vorzeichen-Bug-Klasse."
  - "Nicht-blockende BatchAlreadyResolved-Race-Semantik: DAO liefert rows_affected, Basic-Service reicht durch, BL entscheidet den Fehler-Modus."

requirements-completed: [REB-MANUAL-01, REB-MANUAL-02, HR-ALERT-03, HR-ALERT-04]

coverage:
  - id: D1
    description: "RebookingReconciliationService Trait + Pure fns (alert_predicate D-55-01, proposed_rebooking_hours D-55-03) — Domain-Regeln als eigenstaendige, unit-testbare Bausteine."
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#predicate_truth_table::*"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#proposed_hours::*"
        status: pass
    human_judgment: false
  - id: D2
    description: "DAO/Service-Erweiterungen (find_pending_for_sales_person, list_all_pending, update_state_conditional) — HR-gated Wrapper + sqlx offline cache aktualisiert."
    requirement: "HR-ALERT-03"
    verification:
      - kind: integration
        ref: "SQLX_OFFLINE=true cargo build --workspace && cargo sqlx prepare --workspace --check"
        status: pass
    human_judgment: false
  - id: D3
    description: "REB-MANUAL-01 Atomarität: rebook_manual schreibt 2 ExtraHours (Marker Rebooking) + 1 Batch + 1 Entry in einer Transaktion; UNIQUE-Kollision propagiert als 409."
    requirement: "REB-MANUAL-01"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#service::rebook_manual_writes_two_extra_hours_batch_and_entry_in_one_tx"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#service::rebook_manual_unique_collision_returns_entity_already_exists"
        status: pass
    human_judgment: false
  - id: D4
    description: "REB-MANUAL-02 Direction-Symmetrie: VolunteerToExtra + ExtraToVolunteer erzeugen inverse Kategorien-Paare."
    requirement: "REB-MANUAL-02"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#service::rebook_manual_writes_two_extra_hours_batch_and_entry_in_one_tx"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#service::rebook_manual_supports_reverse_direction"
        status: pass
    human_judgment: false
  - id: D5
    description: "HR-ALERT-03 State-Machine + T-55-01 Race-Schutz: approve/reject sind state-conditional Pending→(Approved|Rejected); Double-Approve-Race -> BatchAlreadyResolved."
    requirement: "HR-ALERT-03"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#service::approve_suggestion_updates_state_writes_pair_rows"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#service::approve_suggestion_double_approve_race_yields_error"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#service::reject_suggestion_updates_state_without_writing_extra_hours"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#service::reject_after_approve_yields_error"
        status: pass
    human_judgment: false
  - id: D6
    description: "HR-ALERT-04 UNIQUE-Slot-Claim: suggest_for_week / rebook_manual triggern die DB-UNIQUE-Kollision und bekommen sie als EntityAlreadyExists zurueck (Kollision-Test in D3)."
    requirement: "HR-ALERT-04"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#service::rebook_manual_unique_collision_returns_entity_already_exists"
        status: pass
    human_judgment: false
  - id: D7
    description: "VOL-ACCT-03 Rebooking-Neutralitaet: Reporting-Aggregation filtert `source == Rebooking`-Rows an allen vier extra_hours-Fetch-Pfaden — Pair-Nullsummen fliessen nicht ins Aggregat."
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#reporting_filter::reporting_aggregate_ignores_rebooking_marker_rows"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#reporting_filter::reporting_aggregate_is_zero_when_only_rebooking_pair"
        status: pass
      - kind: integration
        ref: "Property-Test 55-03 (Rebooking-Roundtrip-Neutralitaet) wird das end-to-end verifizieren."
        status: unknown
    human_judgment: false
  - id: D8
    description: "T-55-02 HR-Gate: jede public Trait-Methode gatet HR_PRIVILEGE als erste await-Operation; Non-HR-Aufrufer bekommt Forbidden ohne DAO-/Reporting-Aufruf."
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_reconciliation.rs#service::rebook_manual_forbidden_for_non_hr"
        status: pass
    human_judgment: false

duration: 25min
completed: 2026-07-10
status: complete
---

# Phase 55 Plan 01: Manual-Rebooking + HR-Alert Backend-Foundation Summary

**Business-Logic RebookingReconciliationService orchestriert Pair-ExtraHours + Batch/Entry in einer Transaktion mit state-conditional UPDATE fuer Approve/Reject-Race + zentralem Rebooking-Neutralitaets-Filter im Reporting-Aggregat.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-07-10T18:37:46Z
- **Completed:** 2026-07-10T19:02:47Z
- **Tasks:** 3
- **Files modified:** 12 (davon 3 neue Source-Files + 3 neue .sqlx-Cache-Files + Test-File)

## Accomplishments

- **Business-Logic-Foundation fuer F3+F5:** neuer `RebookingReconciliationService` (BL-Tier) mit 6 Trait-Methoden — HR-gated, atomarer Doppel-Eintrag (REB-MANUAL-01), state-conditional Approve/Reject (HR-ALERT-03), Pending-Claim ueber UNIQUE-Slot (HR-ALERT-04), phase-weite `list_pending_for_sales_person(None, ...)`.
- **Pure-fn Domain-Regeln** `alert_predicate` (D-55-01, `balance <= -0.5h` Float-Noise-Grenzfall) und `proposed_rebooking_hours` (D-55-03, `min(|balance|, voluntary_ist)`) — beide Truth-Table-getestet ohne Service-Mocks (12 Cases).
- **DAO-Erweiterungen** `find_pending_for_sales_person`, `list_all_pending`, `update_state_conditional` (mit `rows_affected`-Rueckgabe fuer Race-Schutz) plus HR-gated Service-Wrapper.
- **VOL-ACCT-03 Reporting-Filter Wave-1-Owner:** zentraler `source != Rebooking`-Filter an allen vier `extra_hours`-Fetches in `service_impl/src/reporting.rs` — Pair-Nullsummen fliessen nicht in `balance_hours`/`volunteer_hours`/`overall_hours`.
- **23/23 rebooking_reconciliation Tests gruen; 783/783 service_impl-Tests gruen; `cargo clippy --workspace -- -D warnings` gruen; sqlx offline cache regeneriert und committed.**

## Task Commits

Jede Task wurde atomar committet:

1. **Task 1: Trait + Types + Pure-fn alert_predicate** — `9cd68bb` (feat)
2. **Task 2: DAO-Erweiterung (find_pending_for_sales_person + list_all_pending + update_state_conditional) + Service-Wrapper** — `e575102` (feat)
3. **Task 3: RebookingReconciliationServiceImpl + Reporting-Filter + Test-Suite** — `e03f0e9` (feat)

## Files Created/Modified

**Created:**
- `service/src/rebooking_reconciliation.rs` — Trait mit 6 Methoden + `RebookingDirection` + `RebookingSuggestion` + 2 pure fns.
- `service_impl/src/rebooking_reconciliation.rs` — BL-Impl.
- `service_impl/src/test/rebooking_reconciliation.rs` — Testmodul (predicate_truth_table + proposed_hours + service + reporting_filter).
- `.sqlx/*.json` — drei neue Query-Fingerprints (find_pending, list_all_pending, update_state_conditional).

**Modified:**
- `service/src/lib.rs` — `pub mod rebooking_reconciliation;` + `ServiceError::BatchAlreadyResolved`.
- `service/src/rebooking_batch.rs` — Trait um 3 neue Methoden erweitert.
- `service_impl/src/lib.rs` — `pub mod rebooking_reconciliation;`.
- `service_impl/src/rebooking_batch.rs` — 3 neue Service-Impls.
- `service_impl/src/reporting.rs` — VOL-ACCT-03-Filter an 4 Fetch-Pfaden.
- `service_impl/src/test/mod.rs` — `pub mod rebooking_reconciliation;`.
- `dao/src/rebooking_batch.rs` — Trait um 3 neue Methoden erweitert.
- `dao_impl_sqlite/src/rebooking_batch.rs` — sqlx-Impls.
- `rest/src/lib.rs` — `ServiceError::BatchAlreadyResolved -> HTTP 409`.

## Decisions Made

- **Task 3 Soll-Snapshot-Trade-off:** `hydrate_pending_to_suggestion` und `suggest_for_week` setzen `voluntary_soll_before = 0.0` — der vollstaendige F2-Soll-Aggregator lebt in `VoluntaryStatsService` (BL). Diesen als Dep aufzunehmen wuerde Cross-BL-Coupling schaffen. Plan 55-02 wird `VoluntaryStatsService` **im REST-Handler-Layer** nach-aggregieren, sofern das FE Soll-Zahlen anzeigen soll. Die Fat-Backend-Regel D-55-03 bleibt gewahrt: `voluntary_delta_before/after` werden trotzdem Backend-berechnet (aus `voluntary_ist - voluntary_soll`) — nur die Basis-Soll-Zahl bleibt 0.0. Alternative (Soll direkt in `EmployeeReport` ergaenzen) wurde verworfen, weil `EmployeeReport` schon ein Aggregat-Schwergewicht ist.
- **`update_state_conditional` = "no error at 0 rows_affected":** Der Basic-Tier-Wrapper reicht `rows_affected` unveraendert durch; die BL entscheidet den Fehler-Modus (`BatchAlreadyResolved`). Grund: Race-Semantik ist BL-Business, nicht DAO-Verhalten.
- **Direction-Enum + Helper `build_pair_payloads`:** Statt Menge signiert zu uebergeben und im Impl zu verzweigen, akzeptiert der Trait `hours > 0` + `RebookingDirection`. Das eliminiert die Vorzeichen-Bug-Klasse und macht die Kategorien-Zuordnung an einer Stelle sichtbar.
- **Reporting-Filter zentral im ReportingService, nicht per Konsument:** VoluntaryStatsService + BookingInformationService konsumieren `ReportingService::get_report_for_employee*` (nicht direkt `extra_hours_service`), also greift der Filter fuer sie automatisch — Konsistenz-Garantie fuer Wave 1.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] rest/src/lib.rs exhaustive match brach nach neuer ServiceError-Variante**
- **Found during:** Task 2 (Trait-Erweiterung um `update_state_conditional`)
- **Issue:** `error_handler` in `rest/src/lib.rs` ist ein exhaustive match; nach Hinzufuegen von `ServiceError::BatchAlreadyResolved` (Plan-Task-3 Vorbereitung, in Task 1 in `service/src/lib.rs` eingefuehrt) wurde die REST-Kompilation blockiert.
- **Fix:** Neue Arm mit HTTP 409 Conflict + `err.to_string()`-Body ergaenzt (konsistent mit `EntityAlreadyExists`-Semantik). Damit ist die Race-Signal-Kette bis zum FE geschlossen.
- **Files modified:** `rest/src/lib.rs`
- **Verification:** `cargo build --workspace` gruen.
- **Committed in:** `e575102` (Task 2 commit)

**2. [Rule 1 - Bug/Fix] clippy `too_many_arguments` auf `rebook_manual`**
- **Found during:** Task 3 (Clippy Hard-Gate)
- **Issue:** `rebook_manual` hat 8 Argumente (sales_person_id, iso_year, iso_week, direction, hours, context, tx, &self) — clippy default limit ist 7.
- **Fix:** `#[allow(clippy::too_many_arguments)]` auf die Trait-Methode; Alternative (Payload-Struct) waere ein groesserer API-Bruch fuer den REST-Layer in Plan 55-02.
- **Files modified:** `service/src/rebooking_reconciliation.rs`
- **Verification:** `cargo clippy --workspace -- -D warnings` gruen.
- **Committed in:** `e03f0e9` (Task 3 commit)

**3. [Rule 1 - Bug/Fix] clippy `useless_vec` auf Filter-Testinput**
- **Found during:** Task 3 (Clippy Hard-Gate — `--tests`-Variante)
- **Issue:** `vec![...]` in Filter-Tests, wo Array reicht (`.iter().filter(...)` funktioniert auch auf `[T; N]`).
- **Fix:** `vec![...]` -> `[...]` in beiden reporting_filter-Tests.
- **Files modified:** `service_impl/src/test/rebooking_reconciliation.rs`
- **Verification:** `cargo clippy --workspace -- -D warnings` gruen; `cargo test -p service_impl rebooking_reconciliation` -> 23/23.
- **Committed in:** `e03f0e9` (Task 3 commit)

---

**Total deviations:** 3 auto-fixed (1 Rule 3 blocking, 2 Rule 1 lint bugs).
**Impact on plan:** Alle drei sind mechanische Compile-/Lint-Fixes. Kein Scope-Creep, keine Design-Aenderung.

## Issues Encountered

- **`cargo sqlx prepare` unter nix develop:** Das `~/.cargo/bin/cargo-sqlx`-Binary im System-PATH ist gegen eine alte `libssl.so.3` gelinkt und crasht sofort; das nix-develop-provided `sqlx-cli` (0.9.0) wird von `cargo sqlx prepare` NICHT aufgerufen, weil Cargo `~/.cargo/bin`-Subcommands mit Prioritaet auflost. Workaround: `cargo-sqlx` im User-Cargo-Bin waehrend des Prepare-Runs temporaer umbenennen — dann greift das nix-Binary. Fuer das lokale Dev-Setup ist das gutartig; CI nutzt `SQLX_OFFLINE=true` und ist davon nicht betroffen. Zukuenftig ein Alias in `.envrc` oder in `shell.nix`-Instruction dokumentieren.

## User Setup Required

None — kein externer Service, keine neuen ENV-Variablen, keine Migration (Datenmodell aus Phase 54 wiederverwendet).

## Next Phase Readiness

**Ready for Plan 55-02 (REST-Endpoints + rest-types + DTO-Erweiterung + Dioxus.toml-Proxy):**
- `RebookingReconciliationService` steht als BL-Service bereit; REST-Handler muss nur `check_permission(HR_PRIVILEGE)` durch den Service selber erledigen lassen (Handler bleibt duenn).
- `list_pending_for_sales_person(Option<Uuid>)` bedient sowohl den `has_pending_rebooking`-Predicate im `ShortEmployeeReportTO` (Some(id)) als auch `GET /rebooking-suggestions` (None).
- `BatchAlreadyResolved` bereits als HTTP 409 verdrahtet — FE kann auf Race-Signal reagieren.
- `.sqlx/` offline cache aktualisiert und committed; CI (`SQLX_OFFLINE=true`) bleibt gruen.

**Ready for Plan 55-03 (Property-Test Rebooking-Neutralitaet):**
- VOL-ACCT-03 Wave-1-Owner-Filter aktiv in `service_impl/src/reporting.rs`.
- Rebooking-Pair-Rows tragen zwingend `ExtraHoursSource::Rebooking` (im BL-Service als Contract).
- Unit-Tests des Filter-Predicates + der Pair-Semantik als Stufe 0 des Property-Tests bereits geleistet.

**Blocker fuer Plan 55-02:** keine.

---

## Self-Check: PASSED

- `service/src/rebooking_reconciliation.rs` — FOUND
- `service_impl/src/rebooking_reconciliation.rs` — FOUND
- `service_impl/src/test/rebooking_reconciliation.rs` — FOUND
- Commit `9cd68bb` (Task 1) — FOUND
- Commit `e575102` (Task 2) — FOUND
- Commit `e03f0e9` (Task 3) — FOUND

---

*Phase: 55-manual-rebooking-hr-alert*
*Completed: 2026-07-10*
