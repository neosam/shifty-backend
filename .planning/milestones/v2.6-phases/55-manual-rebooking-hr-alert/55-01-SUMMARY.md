---
phase: 55-manual-rebooking-hr-alert
plan: 01
subsystem: database
tags: [rust, sqlx, sqlite, rebooking, state-machine, optimistic-locking, unique-claim, hr-alert]

# Dependency graph
requires:
  - phase: 54-vol-acct
    provides: "RebookingBatchDao/-Service Basic-Tier (find_by_id / find_by_sales_person_year_week / create), rebooking_batch-Tabelle + UNIQUE-Partial-Index"
provides:
  - "RebookingBatchDao::update_state_conditional — race-sicheres state-conditional UPDATE (pending → approved|rejected) mit rows_affected()==1-Guard"
  - "RebookingBatchDao::find_pending_for_sales_person — Reader fuer den pending-Batch im globalen UNIQUE-Slot"
  - "RebookingBatchDao::create_pending_suggestion — F5-Modell-B pending-Batch-Producer (Claim-INSERT, UNIQUE-Slot)"
  - "RebookingBatchService::{update_state, find_pending_for_sales_person, create_pending_suggestion} — HR-gated Basic-Service-Methoden"
  - "DaoError::EntityAlreadyExists — distinktes Conflict-Signal fuer Slot-Kollision + rows_affected==0"
affects: [55-02-reconciliation, 55-03-short-employee-report, 56-auto-cron]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Optimistic-State-Transition via state-conditional UPDATE (WHERE state='pending') + rows_affected()==1-Guard — kein version-Guard, kein Read-then-write (TOCTOU-frei)"
    - "UNIQUE-Slot-Claim via INSERT + is_unique_violation()-Detection statt Read-then-INSERT (Index ist die Autoritaet, Pitfall 6)"

key-files:
  created:
    - "service_impl/src/test/rebooking_batch_state.rs"
  modified:
    - "dao/src/rebooking_batch.rs"
    - "dao/src/lib.rs"
    - "dao_impl_sqlite/src/rebooking_batch.rs"
    - "service/src/rebooking_batch.rs"
    - "service_impl/src/rebooking_batch.rs"
    - "service_impl/src/test/mod.rs"
    - "docs/features/F14-rebooking.md"
    - "docs/features/F14-rebooking_de.md"
    - ".sqlx/"

key-decisions:
  - "Conflict-Variante = ServiceError::EntityAlreadyExists fuer BEIDE 409-Faelle (Slot-Kollision + rows_affected==0), da keine dedizierte ServiceError::Conflict-Variante existiert"
  - "Neue DaoError::EntityAlreadyExists-Variante als distinktes Conflict-Signal (UNIQUE-Violation via is_unique_violation() erkannt), im Service auf ServiceError::EntityAlreadyExists gemappt"
  - "approved_by als Option<Arc<str>> (statt Option<&str>) im DAO-Trait, damit automock die Signatur ohne HRTB-Lifetime-Fehler ableiten kann"

patterns-established:
  - "state-conditional UPDATE (WHERE state='pending') + rows_affected()!=1 -> EntityAlreadyExists: das genuin neue race-sichere Transition-Muster der Phase"
  - "Claim-INSERT mit UNIQUE-Partial-Index als Autoritaet + is_unique_violation()-Detection statt Read-then-INSERT"

requirements-completed: [HR-ALERT-03, HR-ALERT-04]

coverage:
  - id: D1
    description: "create_pending_suggestion claimt den leeren UNIQUE-Slot (F5-Modell-B Producer, HR-ALERT-04); belegter Slot -> EntityAlreadyExists"
    requirement: "HR-ALERT-04"
    verification:
      - kind: integration
        ref: "service_impl/src/test/rebooking_batch_state.rs#create_pending_suggestion_claims_slot"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/rebooking_batch_state.rs#create_pending_suggestion_on_taken_slot_is_conflict"
        status: pass
    human_judgment: false
  - id: D2
    description: "find_pending_for_sales_person liefert nur state=Pending im globalen UNIQUE-Slot"
    requirement: "HR-ALERT-04"
    verification:
      - kind: integration
        ref: "service_impl/src/test/rebooking_batch_state.rs#find_pending_returns_only_pending"
        status: pass
    human_judgment: false
  - id: D3
    description: "update_state hebt pending atomar auf approved/rejected; Double-Approve + already-decided -> Conflict (rows_affected==0)"
    requirement: "HR-ALERT-03"
    verification:
      - kind: integration
        ref: "service_impl/src/test/rebooking_batch_state.rs#approve_transitions_pending_to_approved"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/rebooking_batch_state.rs#reject_transitions_pending_to_rejected"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/rebooking_batch_state.rs#double_approve_second_is_conflict"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/rebooking_batch_state.rs#update_state_on_already_decided_is_conflict"
        status: pass
    human_judgment: false
  - id: D4
    description: "create_pending_suggestion + update_state + find_pending sind HR-gated (Non-HR -> Forbidden, Gate vor Datenabruf)"
    requirement: "HR-ALERT-03"
    verification:
      - kind: integration
        ref: "service_impl/src/test/rebooking_batch_state.rs#create_and_update_state_are_hr_gated"
        status: pass
    human_judgment: false

# Metrics
duration: 40min
completed: 2026-07-23
status: complete
---

# Phase 55 Plan 01: Rebooking-Batch state-conditional UPDATE + Claim-on-Suggest Summary

**Race-sichere `pending → approved|rejected` State-Transition (WHERE state='pending' + rows_affected()==1-Guard) plus F5-Modell-B pending-Batch-Producer via UNIQUE-Slot-Claim-INSERT, HR-gated auf DAO- und Basic-Service-Ebene.**

## Performance

- **Duration:** ~40 min
- **Completed:** 2026-07-23
- **Tasks:** 3 (TDD: RED → 2× GREEN) + 1 Docs-Gate-Deviation
- **Files modified:** 8 (+ .sqlx-Cache, +1 neues Test-Modul)

## Accomplishments
- **HR-ALERT-04-Fundament (F5-Modell-B Producer):** `create_pending_suggestion` INSERTet einen `rebooking_batch(kind=HrSuggestion, state=Pending)` und claimt den globalen UNIQUE-Slot `(sales_person_id, iso_year, iso_week)`. Slot-Kollision → `EntityAlreadyExists` via UNIQUE-Partial-Index (kein Read-then-INSERT).
- **HR-ALERT-03-Fundament (race-sichere Decide-Transition):** `update_state_conditional` mit `UPDATE ... WHERE state='pending' AND deleted IS NULL` + `rows_affected()!=1 → EntityAlreadyExists`. Double-Approve und already-decided-Wechsel liefern deterministisch Conflict — kein version-Guard, kein Read-then-write (TOCTOU-frei).
- **Reader fuer `pending_rebooking_id` (55-03):** `find_pending_for_sales_person` liefert den Slot-Batch nur, wenn `state=Pending`.
- **HR-Gate an erster Stelle** jeder neuen Service-Methode (vor jedem Datenabruf, T-55-02).
- 8 neue Integration-Tests gegen echte SQLite-in-memory DB — alle gruen; `cargo test --workspace` (768 service_impl + alle uebrigen) ohne Regression; `cargo clippy --workspace -- -D warnings` gruen.

## Task Commits

1. **Task 1: RED — Integration-Tests** - `c0520e1b` (test)
2. **Task 2: GREEN — DAO state-conditional UPDATE + find_pending + Claim-INSERT (SQLite) + .sqlx** - `8fe115f9` (feat)
3. **Task 3: GREEN — Basic-Service update_state + find_pending + create_pending_suggestion (HR-gated)** - `4a44293f` (feat)
4. **Deviation: Docs-Freshness-Gate F14** - `da4ef4d1` (docs)

_TDD: RED-Test-Commit vor den beiden GREEN-Commits._

## Files Created/Modified
- `service_impl/src/test/rebooking_batch_state.rs` - Neues Integration-Test-Modul (8 Tests, echte SQLite-in-memory DB)
- `service_impl/src/test/mod.rs` - Modul-Registrierung (`pub mod rebooking_batch_state;`)
- `dao/src/rebooking_batch.rs` - Drei neue Trait-Methoden (update_state_conditional, find_pending_for_sales_person, create_pending_suggestion)
- `dao/src/lib.rs` - Neue `DaoError::EntityAlreadyExists`-Variante
- `dao_impl_sqlite/src/rebooking_batch.rs` - SQLite-Impl der drei Methoden + `is_unique_violation()`-Helper
- `service/src/rebooking_batch.rs` - Drei neue HR-gated Trait-Methoden (automock erweitert Mock automatisch)
- `service_impl/src/rebooking_batch.rs` - Impl mit HR-Gate-First + DaoError→ServiceError-Conflict-Mapping
- `docs/features/F14-rebooking.md` + `_de.md` - Service-Methoden-Tabelle aktualisiert (Docs-Gate)
- `.sqlx/` - Drei neue compile-time-checked Query-Cache-Eintraege

## Decisions Made
- **Conflict-Variante = `ServiceError::EntityAlreadyExists`** fuer beide 409-Faelle (belegter Slot + rows_affected==0). Der Plan erlaubte `Conflict` oder `EntityAlreadyExists`; da keine `ServiceError::Conflict`-Variante existiert, ist `EntityAlreadyExists` die kanonische 409-Variante. Im RED-Test gepinnt und begruendet.
- **Neue `DaoError::EntityAlreadyExists`-Variante** als distinktes Conflict-Signal: notwendig, weil `map_db_error` UNIQUE-Violations sonst nur als generisches `DatabaseQueryError` boxt und der Service den Kollisions-Pfad (ohne Read-then-INSERT) nicht unterscheiden koennte.
- **`approved_by: Option<Arc<str>>`** statt `Option<&str>` im DAO-Trait: `mockall::automock` scheiterte an der HRTB-Lifetime-Elision eines Borrow-Params (E0637/E0106).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `approved_by`-Signatur von `Option<&str>` auf `Option<Arc<str>>` geaendert**
- **Found during:** Task 2 (DAO-Trait `update_state_conditional`)
- **Issue:** `#[automock]` konnte den Borrow-Param `Option<&str>` nicht ableiten — `cargo sqlx prepare` brach mit E0637/E0106 (higher-ranked lifetime) ab.
- **Fix:** Owned `Option<Arc<str>>` (konsistent mit `RebookingBatchEntity::approved_by`), Impl mit `.as_ref().map(|s| s.to_string())`.
- **Verification:** `cargo sqlx prepare --workspace` + `cargo clippy --workspace -- -D warnings` gruen.
- **Committed in:** `8fe115f9`

**2. [Rule 2 - Missing Critical / CLAUDE.md Docs-Freshness-Gate] F14-Doku aktualisiert**
- **Found during:** Nach Task 3 (dao/service-Trait-Signatur geaendert)
- **Issue:** CLAUDE.md Docs-Freshness-Gate: `dao/**/*.rs`- und `service/**/*.rs`-Trait-Signatur-Aenderungen verlangen ein synchrones Update von `docs/features/F14`.
- **Fix:** Service-Methoden-Tabelle in `F14-rebooking.md` UND `_de.md` (synchron) um die drei neuen Methoden ergaenzt.
- **Verification:** Beide Sprachdateien synchron.
- **Committed in:** `da4ef4d1`

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 CLAUDE.md-Docs-Gate)
**Impact on plan:** Beide notwendig (Compile-Blocker bzw. hartes CLAUDE.md-Gate). Kein Scope-Creep. Alle Plan-Prohibitions eingehalten (kein version-Guard, kein Read-then-write, kein Read-then-INSERT, keine Migration, kein Snapshot-Bump).

## Issues Encountered
- Kein lokaler `localdb.sqlite3` fuer `cargo sqlx prepare` vorhanden — via `sqlx database create` + `sqlx migrate run` frisch angelegt (Datei ist gitignored, kein Repo-Artefakt).

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- 55-02 (`RebookingReconciliationService`) kann jetzt `create_pending_suggestion` (Suggest/Claim) + `update_state` (Approve/Reject) konsumieren.
- 55-03 (`short_employee_report`) kann `find_pending_for_sales_person` fuer den `pending_rebooking_id`-Flag nutzen.
- Phase-56-Cron liest denselben `state=pending`-Batch zum Slot-Skippen.

---
*Phase: 55-manual-rebooking-hr-alert*
*Completed: 2026-07-23*

## Self-Check: PASSED
- Alle 6 Kern-Dateien + SUMMARY.md auf Disk vorhanden.
- Alle 4 Commits (c0520e1b, 8fe115f9, 4a44293f, da4ef4d1) in git-history.
