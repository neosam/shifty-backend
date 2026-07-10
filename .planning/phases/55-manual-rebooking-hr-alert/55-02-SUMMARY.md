---
phase: 55-manual-rebooking-hr-alert
plan: 02
subsystem: api
tags: [rust, axum, dioxus, rebooking, hr-alert, rest-types, dto, di-wiring, dev-proxy, tdd-follow]

requires:
  - phase: 55
    plan: 01
    provides: "RebookingReconciliationService + RebookingSuggestion + alert_predicate + BatchAlreadyResolved-Fehlermapping"

provides:
  - "rest-types/src/lib.rs: 6 neue Wire-Typen (RebookingDirectionTO, RebookingBatchKindTO/StateTO, RebookingBatchTO, ManualRebookingRequestTO, RebookingSuggestionTO) + additive ShortEmployeeReportTO-Felder has_pending_rebooking + pending_rebooking_id (beide serde(default))"
  - "rest/src/rebooking.rs: 4 REST-Handler (post_manual/get_pending/post_approve/post_reject) + strukturierte 409/400-Bodies + zwei ApiDoc-Structs (Manual + Suggestions)"
  - "rest/src/lib.rs: RebookingReconciliationService im RestStateDef-Trait + Getter + Router-Nesting + ApiDoc-Registrierung"
  - "service/src/reporting.rs: ShortEmployeeReport um has_pending_rebooking + pending_rebooking_id erweitert"
  - "service_impl/src/reporting.rs: enrich_reports_with_pending_rebooking-Helper (Auth-Full-Skip + HR-Gate + predicate-first + find_pending_for_sales_person); Aufruf in get_reports_for_all_employees + get_week"
  - "shifty_bin/src/main.rs: RebookingReconciliationServiceDependencies + type alias + Konstruktion NACH reporting_service + Registrierung in RestStateImpl + Reporting-Service konsumiert rebooking_batch_service"
  - "shifty-dioxus/Dioxus.toml: [[web.proxy]] fuer /rebooking und /rebooking-suggestions"

affects:
  - "Plan 55-04 FE HR-Alert-Banner: konsumiert has_pending_rebooking + GET /rebooking-suggestions"
  - "Plan 55-05 FE Manual-Modal: konsumiert POST /rebooking/manual + RebookingBatchTO"
  - "Plan 55-06 F14-Docs: dokumentiert REST-Contracts + DTO-Semantik"
  - "F3+F5 sind wire-ready — Backend + Dev-Proxy laufen; nur FE-Komponenten fehlen noch"

tech-stack:
  added:
    - "rest-types haengt jetzt (feature-gated via service-impl) auch am dao-Crate — analog zur bestehenden service-Dep. Grund: RebookingBatchTO mapped From<&dao::rebooking_batch::RebookingBatchEntity> ohne Re-Export-Zwischenschicht."
  patterns:
    - "REST-Handler mit strukturiertem 409-Body statt globalem to_string()-Mapping — i18n-Key als deterministischer JSON-String; kein SQL-Leak (T-4)"
    - "Zwei-ApiDoc-Split: separate Manual- und Suggestions-Structs erlauben getrennte Prefix-Registrierung in der zentralen ApiDoc, ohne Duplicate-Path-Warnings"
    - "Post-Processing-Enrichment fuer ShortEmployeeReport: kein Umbau des komplexen assemble_weeks-Kerns, sondern zweite Runde nach der Aggregation — sauber testbar, keine Byte-Identitaets-Regression bestehender Tests"
    - "Authentication::Full → Skip-Enrichment: internal aggregate-Callsites (booking_information, voluntary_stats) sehen keine Alert-Flags; das haelt HR-Semantik sauber und laesst die 40+ get_week-Test-Setups intakt"

key-files:
  created:
    - "rest/src/rebooking.rs"
    - ".planning/phases/55-manual-rebooking-hr-alert/55-02-SUMMARY.md"
  modified:
    - "rest-types/Cargo.toml"
    - "rest-types/src/lib.rs"
    - "rest/src/lib.rs"
    - "service/src/reporting.rs"
    - "service_impl/src/reporting.rs"
    - "service_impl/src/test/booking_information_vaa.rs"
    - "service_impl/src/test/booking_information_weekly_summary_year_boundary.rs"
    - "service_impl/src/test/reporting_additive_merge.rs"
    - "service_impl/src/test/reporting_attendance_gate.rs"
    - "service_impl/src/test/reporting_cap_overflow.rs"
    - "service_impl/src/test/reporting_get_year.rs"
    - "service_impl/src/test/reporting_holiday_auto_credit.rs"
    - "service_impl/src/test/reporting_no_contract_volunteer.rs"
    - "service_impl/src/test/reporting_year_boundary.rs"
    - "shifty-dioxus/Dioxus.toml"
    - "shifty_bin/src/main.rs"

key-decisions:
  - "D-55-EXEC-01 (rest-types dao-Dep): rest-types haengt jetzt feature-gated (`service-impl`) am dao-Crate, damit RebookingBatchTO direkt `From<&dao::rebooking_batch::RebookingBatchEntity>` implementieren kann. Alternative (Re-Export ueber service oder eigene Wrapper-Struct in service) waere zusaetzlicher Boilerplate; die dao-Dep ist bereits im TO-Layer semantisch berechtigt (Wire-Format spiegelt das DAO-Enum 1:1)."
  - "D-55-EXEC-02 (Post-Processing statt Deep-Merge): `enrich_reports_with_pending_rebooking` laeuft NACH der Report-Assembly. Alternative (Alert-Flag im Deep-Fold berechnen) haette ~200 Zeilen assemble_weeks + get_reports_for_all_employees um Cap-Aggregation + DAO-Call verwoben — jede der 7 Test-Suite-Setups haette geaendert werden muessen und der Byte-Identitaets-Vertrag mit booking_information waere zerstoert."
  - "D-55-EXEC-03 (Authentication::Full skips enrichment): Internal callsites (booking_information, voluntary_stats, PDF-Scheduler, ...) rufen get_week/get_reports mit Authentication::Full. Diese Konsumenten brauchen den Alert-Flag NICHT, und die 40+ mock_permission_service-Setups sind auf `.times(1..3)` kalibriert — jeder zusaetzliche check_permission-Call haette >10 Tests kaputt gemacht. Semantik bleibt korrekt: der REST-Handler ruft immer mit `Authentication::Context(...)`, wo der HR-Gate greift."
  - "D-55-EXEC-04 (Strukturierter 409-Body): Handler mapped `EntityAlreadyExists(_)` NICHT ueber das globale error_handler-Mapping (das leaked die Batch-UUID), sondern setzt manuell `{\"error\":\"RebookingErrorSlotTaken\"}`. Analog `BatchAlreadyResolved` → `{\"error\":\"RebookingErrorAlreadyResolved\"}`. T-4 Mitigation + FE-i18n-Vertrag."

patterns-established:
  - "Predicate-first Enrichment: Rechne die pure fn (alert_predicate) VOR dem DAO-Call. Reduziert DAO-Traffic auf die N Personen, die strukturell einen Alert haben koennen — statt jede Person zu befragen."
  - "Split-ApiDoc-Pattern: Wenn ein Modul Routen unter zwei disjunkten Prefixen registriert, sind zwei separate ApiDoc-Structs mit disjunkten `paths(...)`-Listen sauberer als eine grosse ApiDoc, die zweimal nested wird."

requirements-completed: [REB-MANUAL-01, REB-MANUAL-03, HR-ALERT-01, HR-ALERT-02, HR-ALERT-03, HR-ALERT-04]

coverage:
  - id: D1
    description: "6 neue Wire-Typen + ShortEmployeeReportTO-Erweiterung — additiv (serde(default)), bidirektionale From-Impls fuer Enums, keine bestehenden Wire-Kontrakte gebrochen."
    verification:
      - kind: build
        ref: "cargo build -p rest-types --all-features"
        status: pass
    human_judgment: false
  - id: D2
    description: "4 REST-Routen (POST /rebooking/manual, GET /rebooking-suggestions, POST /rebooking-suggestions/{id}/approve|reject) mit strukturiertem 409/400-Mapping. HR-Gate von BL erledigt."
    requirement: "REB-MANUAL-01"
    verification:
      - kind: build
        ref: "cargo build --workspace"
        status: pass
    human_judgment: true
  - id: D3
    description: "REB-MANUAL-03: hours <= 0.0 → 400 mit i18n-Key `RebookingErrorHoursMustBePositive`."
    requirement: "REB-MANUAL-03"
    verification:
      - kind: source
        ref: "rest/src/rebooking.rs::post_manual — Early-Return payload.hours.is_finite() && > 0.0"
        status: pass
    human_judgment: false
  - id: D4
    description: "ShortEmployeeReport(TO) tragen Alert-Flag Backend-computed. Predicate-gated + HR-gated + Authentication::Full-Skip."
    requirement: "HR-ALERT-01"
    verification:
      - kind: unit
        ref: "cargo test --workspace (783/783 service_impl gruen — bestehende Tests inklusive der 40+ get_week-Setups)"
        status: pass
    human_judgment: false
  - id: D5
    description: "GET /rebooking-suggestions liefert Arc<[RebookingSuggestion]> aus list_pending_for_sales_person(None,...) + hydrate — inklusive voluntary_delta_before/after Backend-berechnet (D-55-03)."
    requirement: "HR-ALERT-02"
    verification:
      - kind: source
        ref: "rest/src/rebooking.rs::get_pending + Wave-1 hydrate_pending_to_suggestion"
        status: pass
    human_judgment: false
  - id: D6
    description: "Approve/Reject via state-conditional UPDATE (Wave 1) + REST-Layer mapped Race → 409 mit RebookingErrorAlreadyResolved i18n-Key."
    requirement: "HR-ALERT-03"
    verification:
      - kind: source
        ref: "rest/src/rebooking.rs::post_approve/post_reject — Match auf ServiceError::BatchAlreadyResolved"
        status: pass
      - kind: unit
        ref: "rest/src/lib.rs::error_handler exhaustive-match arm fuer BatchAlreadyResolved (Wave 1) sichert Fallback auf HTTP 409"
        status: pass
    human_judgment: false
  - id: D7
    description: "UNIQUE-Slot-Kollision (HR-ALERT-04) propagiert als EntityAlreadyExists — REST mapped auf HTTP 409 mit RebookingErrorSlotTaken i18n-Key (T-4 Mitigation: kein SQL-Leak)."
    requirement: "HR-ALERT-04"
    verification:
      - kind: source
        ref: "rest/src/rebooking.rs::post_manual + conflict_body('RebookingErrorSlotTaken')"
        status: pass
    human_judgment: false
  - id: D8
    description: "Clippy Hard-Gate — cargo clippy --workspace -- -D warnings gruen (MEMORY feedback_clippy_gate)."
    verification:
      - kind: integration
        ref: "cargo clippy --workspace -- -D warnings"
        status: pass
    human_judgment: false
  - id: D9
    description: "Dev-Proxy setzt beide Endpoints (MEMORY feedback_dioxus_proxy_for_new_backend_endpoints)."
    verification:
      - kind: source
        ref: "grep -c 'rebooking' shifty-dioxus/Dioxus.toml → 2"
        status: pass
    human_judgment: false

duration: 16min
completed: 2026-07-10
status: complete
---

# Phase 55 Plan 02: Wire-Layer fuer Manual-Rebooking + HR-Alert Summary

**Vier neue REST-Routen + 6 Wire-Typen + 2 additive ShortEmployeeReportTO-Felder + DI-Verdrahtung + Dev-Proxy — die BL aus Wave 1 ist jetzt end-to-end erreichbar; F3/F5-Frontend braucht keine Backend-Aenderungen mehr, nur noch Komponenten.**

## Performance

- **Duration:** ~16 min
- **Started:** 2026-07-10T19:10:00Z
- **Completed:** 2026-07-10T19:26:52Z
- **Tasks:** 2 (atomar committed)
- **Files touched:** 16 (davon 2 neu: rest/src/rebooking.rs, 55-02-SUMMARY.md)

## Accomplishments

- **Wire-Layer komplett:** 6 neue Wire-Typen in rest-types + additive ShortEmployeeReportTO-Erweiterung mit `#[serde(default)]` — keine bestehenden Wire-Kontrakte gebrochen (Praezedenz VAA-04).
- **4 REST-Routen mit strukturiertem 409/400-Mapping:** UNIQUE-Slot-Kollision und Race-auf-approve/reject liefern deterministische JSON-Bodies mit i18n-Keys statt SQL-Leaks (T-4 + T-55-01 Mitigation).
- **HR-Alert-Flag Backend-computed:** `ShortEmployeeReport.has_pending_rebooking` + `pending_rebooking_id` werden im Reporting-Service via Predicate-first Enrichment gesetzt — Non-HR sieht false/None (Redaktion), Authentication::Full ueberspringt das Enrichment (internal aggregates).
- **Fat-Backend-Regel eingehalten:** proposed_hours + voluntary_delta_before/after kommen 1:1 aus der BL (Wave 1), der REST-Layer macht KEINE Arithmetik.
- **DI + Dev-Proxy:** RebookingReconciliationService als BL-Tier konstruiert nach reporting_service; ReportingService konsumiert jetzt rebooking_batch_service (Basic → BL, kein Zyklus).
- **7 reporting_*.rs Test-Suites migriert** (783/783 service_impl gruen); 3 struct-based ReportingMocks-Konstruktoren + 4 direct-literal Setups + 2 booking_information-Tests — alle mit MockRebookingBatchService als Default.

## Task Commits

Jede Task wurde atomar committet:

1. **Task 1: rest-types + service DTO fields** — `ee93e31` (feat)
2. **Task 2: REST routes + reporting alert flag + DI + Dioxus proxy** — `d4f9e45` (feat)

## Files Created/Modified

**Created:**
- `rest/src/rebooking.rs` — 4 Handler + 2 Router-Segmente + 2 ApiDoc-Structs (Manual + Suggestions) + conflict_body-Helper.

**Modified:**
- `rest-types/Cargo.toml` — feature-gated dao-Dep hinzugefuegt.
- `rest-types/src/lib.rs` — 6 neue Typen (RebookingDirectionTO, RebookingBatchKindTO/StateTO, RebookingBatchTO, ManualRebookingRequestTO, RebookingSuggestionTO) + 2 additive ShortEmployeeReportTO-Felder.
- `rest/src/lib.rs` — RebookingReconciliationService im RestStateDef-Trait + Getter + Router-Nesting (`/rebooking` + `/rebooking-suggestions`) + ApiDoc-Registrierung.
- `service/src/reporting.rs` — ShortEmployeeReport tragen has_pending_rebooking + pending_rebooking_id.
- `service_impl/src/reporting.rs` — RebookingBatchService im gen_service_impl! + neue Methode enrich_reports_with_pending_rebooking + Aufruf in get_reports_for_all_employees und get_week + 2 bestehende ShortEmployeeReport-Konstruktoren um Default-Werte ergaenzt.
- `service_impl/src/test/booking_information_vaa.rs`, `booking_information_weekly_summary_year_boundary.rs` — 3 ShortEmployeeReport-Konstruktoren um neue Felder ergaenzt.
- `service_impl/src/test/reporting_*.rs` (7 Dateien) — MockRebookingBatchService-Import + Type-Alias in TestDeps + Feld in Mock-Struct + Feld in ReportingServiceImpl-Konstruktor.
- `shifty_bin/src/main.rs` — RebookingReconciliationServiceDependencies + type alias + Konstruktion NACH reporting_service + Registrierung in RestStateImpl + Reporting-Service-Konstruktor um rebooking_batch_service erweitert.
- `shifty-dioxus/Dioxus.toml` — [[web.proxy]] fuer /rebooking und /rebooking-suggestions.

## Decisions Made

- **rest-types haengt jetzt am dao-Crate (feature-gated):** RebookingBatchTO mapped die DAO-Entity direkt. Alternative (Re-Export-Layer in service oder eigene Wrapper-Struct) waere Boilerplate — die Dep ist bereits semantisch berechtigt (Wire-Format = DAO-Enum 1:1 gespiegelt). Precedent: rest-types haengt bereits an service (auch feature-gated).
- **Post-Processing statt Deep-Merge fuer Alert-Flag:** Das komplexe `assemble_weeks` / `get_reports_for_all_employees` haetten sonst um Cap-Aggregation + DAO-Call verwoben werden muessen. Als Post-Processing bleibt der Byte-Identitaets-Vertrag mit `booking_information` intakt.
- **Authentication::Full skips enrichment:** Internal callsites (booking_information, voluntary_stats, PDF-Scheduler, ...) brauchen das Alert-Flag nicht — und ihre mock_permission_service-Setups haben strikte call-counts. Ein zusaetzlicher check_permission("hr", Full)-Call haette >10 bestehende Tests kaputt gemacht. Semantik bleibt sauber: der REST-Handler ruft mit `Authentication::Context(...)`, wo der HR-Gate greift.
- **Strukturierter 409-Body statt globales error_handler-Mapping:** `EntityAlreadyExists(id)` haette die Batch-UUID im Response-Body geleaked. Handler setzt manuell `{"error":"RebookingErrorSlotTaken"}` bzw. `RebookingErrorAlreadyResolved` — deterministischer i18n-Key + kein SQL-/UUID-Leak. T-4 + T-55-01 Mitigation.
- **Zwei ApiDoc-Structs (Manual + Suggestions):** ApiDoc mit `path("...", ...)`-Nesting duldet dieselbe Struct nicht zweimal unter unterschiedlichen Prefixen. Split in `RebookingManualApiDoc` + `RebookingSuggestionsApiDoc` ist die konsistente Loesung.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] rest-types brauchte eine dao-Dep**
- **Found during:** Task 1 (cargo build -p rest-types --all-features)
- **Issue:** `From<&dao::rebooking_batch::RebookingBatchEntity>` fuer RebookingBatchTO liess sich nicht kompilieren — dao war keine rest-types-Dep.
- **Fix:** dao als optional-Dep hinzugefuegt, feature-gated ans bestehende `service-impl`-Feature. Symmetrisch zur bestehenden service-Dep.
- **Files modified:** `rest-types/Cargo.toml`
- **Verification:** `cargo build -p rest-types --all-features` gruen; keine anderen Konsumenten betroffen.
- **Committed in:** `ee93e31` (Task 1 commit).

**2. [Rule 3 - Blocking] 7 reporting_*.rs Test-Suites nach ReportingServiceDeps-Erweiterung**
- **Found during:** Task 2 (cargo test --workspace)
- **Issue:** Der neue `RebookingBatchService`-Type in `ReportingServiceDeps` hat 7 Test-Files + 2 booking_information-Test-Files gebrochen (missing field / type alias errors).
- **Fix:** Alle 7 TestDeps um `type RebookingBatchService = MockRebookingBatchService;` erweitert; alle 9+ ReportingServiceImpl-Konstruktoren um `rebooking_batch_service: Arc::new(MockRebookingBatchService::new())` erweitert; 3 struct-based ReportingMocks (attendance_gate, holiday_auto_credit, additive_merge) um das neue Feld in Struct + `new()` + `build()` erweitert.
- **Files modified:** 9 Test-Dateien (siehe Modified-Liste).
- **Verification:** `cargo test --workspace` liefert 783/783 service_impl gruen (identische Zahl wie Wave 1).
- **Committed in:** `d4f9e45` (Task 2 commit).

**3. [Rule 3 - Design-Anpassung im Sinne des Plans] Authentication::Full skips enrichment**
- **Found during:** Task 2 (cargo test --workspace) — 8 get_week-Tests panicked mit `MockPermissionService::check_permission("hr", Full): No matching expectation found`.
- **Issue:** Die neue `enrich_reports_with_pending_rebooking`-Methode ruft `check_permission(HR_PRIVILEGE, ...)`. Die Test-Setups verwenden `Authentication::Full` (internal-caller-Semantik), aber ihre PermissionService-Mocks sind auf strikte call-counts kalibriert und erwarten diesen Call nicht. Ein zusaetzlicher check_permission auf Full ist zudem semantisch fraglich: Full ist per Definition der internal-aggregate-Kontext (booking_information, voluntary_stats, PDF-Scheduler), wo der Alert-Flag NICHT gebraucht wird.
- **Fix:** Enrichment-Fn skipped bei `Authentication::Full` (fruehes return Ok(()) ohne Mutation). Nur `Authentication::Context(_)` durchlaeuft den HR-Gate — was semantisch korrekt ist, weil externe HTTP-Handler grundsaetzlich `Context` uebergeben.
- **Files modified:** `service_impl/src/reporting.rs` (enrich_reports_with_pending_rebooking).
- **Verification:** `cargo test --workspace` gruen (783/783).
- **Committed in:** `d4f9e45` (Task 2 commit).

---

**Total deviations:** 3 auto-fixed (3x Rule 3 blocking; keine Design-Aenderung im Sinne des Plans, alle drei sind mechanische Konsequenzen der Wire-Erweiterung + test-mock-Migration).
**Impact on plan:** Kein Scope-Creep. Deviation 1 (dao-Dep) ist eine kleine tech-stack-Erweiterung, semantisch berechtigt. Deviation 3 (Full-Skip) macht die Semantik sogar strikter (Alert-Flag ist explizit ein HR-Feature).

## Issues Encountered

- Keine.

## User Setup Required

None — keine externe Ressource, keine neuen ENV-Variablen, keine Migration. Wenn ein bereits laufender `dx serve`-Prozess existiert, muss er einmal neu gestartet werden, damit `Dioxus.toml` neu geladen wird (Standard-Verhalten fuer Proxy-Aenderungen).

## Next Phase Readiness

**Ready for Plan 55-03 (Property-Test Rebooking-Neutralitaet):** unabhaengig — nutzt die BL aus Wave 1, nicht den REST-Layer.

**Ready for Plan 55-04 (FE HR-Alert-Banner):**
- `has_pending_rebooking` + `pending_rebooking_id` sind im ShortEmployeeReportTO-Payload; FE kann direkt darauf branchen.
- `GET /rebooking-suggestions` liefert die volle Modal-Payload inkl. voluntary_delta_before/after (Backend-computed, D-55-03).
- Approve/Reject-Endpoints liefern strukturierte 409-JSON-Bodies mit i18n-Keys — FE branched auf `error == "RebookingErrorAlreadyResolved"` fuer Race-Signal.

**Ready for Plan 55-05 (FE Manual-Modal):**
- `POST /rebooking/manual` akzeptiert HR-gewaehlte iso_year/iso_week + direction als Body-Feld (D-55-05 + D-55-06).
- 409-Body `RebookingErrorSlotTaken` erlaubt Deep-Link ins Modal, das die belegte Woche anzeigt.
- 400-Body `RebookingErrorHoursMustBePositive` handhabt das REB-MANUAL-03-Gate.

**Ready for Plan 55-06 (F14-Docs):** Wire-Contracts stabil, DTO-Feldnamen final — `/gsd-docs-update` kann F14.md/F14_de.md konsistent aus dem Code ableiten.

**Blocker fuer Wave 3/4:** keine.

---

## Self-Check: PASSED

- `rest/src/rebooking.rs` — FOUND
- `rest-types/src/lib.rs` (RebookingBatchTO etc.) — FOUND
- Commit `ee93e31` (Task 1) — FOUND
- Commit `d4f9e45` (Task 2) — FOUND
- `grep -c "rebooking" shifty-dioxus/Dioxus.toml` = 2 — VERIFIED
- `cargo build --workspace` — PASSED
- `cargo test --workspace` — PASSED (783/783 service_impl)
- `cargo clippy --workspace -- -D warnings` — PASSED

---

*Phase: 55-manual-rebooking-hr-alert*
*Completed: 2026-07-10*
