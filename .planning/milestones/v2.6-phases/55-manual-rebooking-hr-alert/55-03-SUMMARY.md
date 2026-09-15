---
phase: 55-manual-rebooking-hr-alert
plan: 03
subsystem: api
tags: [rebooking, hr-alert, reporting, rest, openapi, axum, dioxus-proxy, serde-default, tdd]

requires:
  - phase: 55-01
    provides: "RebookingBatchService.find_pending_for_sales_person / find_by_sales_person_year_week / create_pending_suggestion / update_state"
  - phase: 55-02
    provides: "RebookingReconciliationService (rebook_manual/suggest/approve/reject) + pure fns alert_predicate/proposed_hours + RebookingSuggestion/RebookingDirection"
provides:
  - "REST-Routes F3 (POST /rebooking/manual) + F5-Modell-B (POST /rebooking-suggestions, POST /rebooking-suggestions/{id}/approve, POST /rebooking-suggestions/{id}/reject) — HR-gated, OpenAPI-dokumentiert, zentrales 409/403/422-Mapping"
  - "rest-types::rebooking-Modul: ManualRebookingRequestTO, RebookingSuggestRequestTO, RebookingApproveRequestTO, RebookingSuggestionTO (IST/DANN backend-computed), RebookingBatchTO/-StateTO, RebookingDirectionTO"
  - "ShortEmployeeReportTO + ShortEmployeeReport: has_pending_rebooking + pending_rebooking_id (backend-berechnet, #[serde(default)], D-55-B/D-55-07-Banner-Regel)"
  - "RebookingReconciliationService in der DI verdrahtet (BL-Tier, nach Basic-Services) + Dioxus-Dev-Proxies"
affects: [55-04, 55-05, 55-06]

tech-stack:
  added: []
  patterns:
    - "Split-Nest-Routing: EIN Handler-Modul liefert zwei generate_route-Funktionen (/rebooking + /rebooking-suggestions) mit gemeinsamem RebookingApiDoc"
    - "DTO-additive Wire-Kompat via #[serde(default)] (Praezedenz VAA-04) fuer neue ShortEmployeeReportTO-Felder"
    - "Alert-Lookup nur fuer Praedikat-Kandidaten (Performance): find_by_sales_person_year_week/find_pending_for_sales_person laufen nur, wenn alert_predicate true ist"

key-files:
  created:
    - "rest-types/src/rebooking.rs"
    - "rest/src/rebooking.rs"
    - "service_impl/src/test/reporting_pending_rebooking.rs"
  modified:
    - "rest-types/src/lib.rs"
    - "rest-types/Cargo.toml"
    - "rest/src/lib.rs"
    - "service/src/reporting.rs"
    - "service_impl/src/reporting.rs"
    - "shifty_bin/src/main.rs"
    - "shifty-dioxus/Dioxus.toml"

key-decisions:
  - "RebookingSuggestionTO auf die REALEN Felder der 55-02-Domain-Struktur RebookingSuggestion modelliert (batch_id, sales_person_id, proposed_hours, balance/-_after, voluntary_ist/-_after) — die im Plan-Artefakt zusaetzlich gelisteten voluntary_soll/-delta-Felder existieren in der 55-02-Domain NICHT und wurden weggelassen, um die Prohibition 'kein FE-Arithmetik / alles backend-computed' nicht durch Fake-Werte zu verletzen."
  - "Optionale GET /rebooking-suggestions-Liste ausgelassen: es existiert kein list-pending-Service/DAO in 55-01/02 (waere Rule-4-Architektur, nicht Teil dieses Plans)."
  - "HR-Alert-Banner wird ausschliesslich ueber den short-report get_reports_for_all_employees (die /report-Liste) getrieben; der per-Woche-Pfad assemble_weeks/get_week bleibt neutral (false/None) — nur der Employee-Aggregat-Pfad braucht den Rebooking-Lookup."
  - "approve-Body traegt RebookingApproveRequestTO { hours }: der Client gibt die im Suggest-Response gelieferten proposed_hours zurueck (die Trait-Signatur approve(id, hours, ...) fordert den Betrag explizit)."

patterns-established:
  - "cap_active pro Woche in WeeklyHours getrackt + im fold OR-reduziert → Employee-Ebene-Input fuer alert_predicate"
  - "ReportingService (BL) konsumiert RebookingBatchService (Basic) ohne Zyklus (Basic kennt keinen Domain-Service); DI-Konstruktion rebooking_batch vor reporting"

requirements-completed: [REB-MANUAL-01, REB-MANUAL-03, HR-ALERT-01, HR-ALERT-02, HR-ALERT-03, HR-ALERT-04]

coverage:
  - id: D1
    description: "F3 POST /rebooking/manual + F5 POST /rebooking-suggestions + approve/reject registriert, HR-gated (Service), OpenAPI-dokumentiert; 409/403/422 zentral im error_handler"
    requirement: "REB-MANUAL-01"
    verification:
      - kind: integration
        ref: "cargo test -p shifty_bin (66 pass — Server assembliert inkl. neuer Nests, OpenAPI ohne Panic)"
        status: pass
      - kind: unit
        ref: "rest-types/src/rebooking.rs#rebooking_to_serde_roundtrip"
        status: pass
    human_judgment: false
  - id: D2
    description: "ShortEmployeeReportTO.has_pending_rebooking = alert_predicate ∧ kein entschiedener Batch; pending_rebooking_id = Some(pending.id); DTO-additiv wire-kompat"
    requirement: "HR-ALERT-01"
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_pending_rebooking.rs#short_report_predicate_true_no_batch"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/reporting_pending_rebooking.rs#short_report_predicate_true_pending_batch"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/reporting_pending_rebooking.rs#short_report_decided_batch_suppresses"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/reporting_pending_rebooking.rs#short_report_predicate_false"
        status: pass
      - kind: unit
        ref: "rest-types/src/lib.rs#test_short_report_to_serde_default::short_report_to_default_flags"
        status: pass
    human_judgment: false
  - id: D3
    description: "IST/DANN/proposed_hours im RebookingSuggestionTO backend-computed (from_domain-Mapping, kein FE-Arithmetik)"
    requirement: "HR-ALERT-02"
    verification:
      - kind: unit
        ref: "rest-types/src/rebooking.rs#rebooking_to_serde_roundtrip (RebookingSuggestionTO roundtrip)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Dioxus-Dev-Proxies /rebooking + /rebooking-suggestions"
    verification:
      - kind: other
        ref: "grep -c 'rebooking-suggestions' shifty-dioxus/Dioxus.toml == 1 (+ /rebooking-Block)"
        status: pass
    human_judgment: false

metrics:
  duration: "~55 min"
  completed: 2026-07-23
  tasks: 3
  files: 10

status: complete
---

# Phase 55 Plan 03: F3/F5 REST-Exposition + HR-Alert-Flag Summary

HR-gated REST-Routes fuer die Phase-55-Geschaeftslogik (F3 manuelle Umbuchung + F5-Modell-B
Suggest/Claim/Approve/Reject), plus der backend-berechnete `has_pending_rebooking`-Alert-Flag im
short-report — die DI-Verdrahtung des `RebookingReconciliationService` (BL-Tier) und die
Dioxus-Dev-Proxies runden die Frontend-Konsumierbarkeit ab.

## Accomplishments

- **rest-types::rebooking (neues Modul, D-55-Pin):** `ManualRebookingRequestTO`,
  `RebookingSuggestRequestTO`, `RebookingApproveRequestTO`, `RebookingSuggestionTO`
  (alle IST/DANN-Felder + `proposed_hours` backend-computed via `from_domain`),
  `RebookingBatchTO`/`RebookingBatchStateTO`, `RebookingDirectionTO` — je `ToSchema`
  + From-Mappings (unter `service-impl`). `dao` als optionale rest-types-Dependency ergaenzt.
- **4 REST-Routes + OpenAPI:** `create_manual_rebooking` (POST /rebooking/manual),
  `create_rebooking_suggestion` (POST /rebooking-suggestions — bewusste Mutation/Claim, Modell B),
  `approve_rebooking_suggestion`, `reject_rebooking_suggestion` — je `#[utoipa::path]` +
  `RebookingApiDoc`, in die zentrale `ApiDoc` eingehaengt. Status-Mapping (409/403/422)
  ausschliesslich zentral im `error_handler`.
- **ShortEmployeeReportTO + Domain-Struct** um `has_pending_rebooking` + `pending_rebooking_id`
  erweitert (`#[serde(default)]`, wire-additiv). Backend-Berechnung im short-report nach der
  F5-Modell-B-Banner-Regel (D-55-B + D-55-07).
- **DI-Wiring:** `RebookingReconciliationService` (BL-Tier) NACH allen Basic-Services konstruiert
  (kein OnceLock); `ReportingService` konsumiert jetzt `RebookingBatchService` (Basic, kein Zyklus).
- **Dioxus.toml:** zwei neue `[[web.proxy]]`-Bloecke.

## Task Commits

1. **Task 1: rest-types-DTOs + ShortEmployeeReportTO-Erweiterung** — `07687043` (feat)
2. **Task 2: 4 REST-Routes + OpenAPI + DI-Wiring + Dioxus-Proxy** — `8eb35d4a` (feat)
3. **Task 3: has_pending_rebooking im short-report berechnen** — `5216354f` (feat)

## Files Created

- `rest-types/src/rebooking.rs` — F3/F5-TOs + From-Mappings + serde-Roundtrip-Test
- `rest/src/rebooking.rs` — 4 Handler + `#[utoipa::path]` + `RebookingApiDoc` + zwei `generate_route`-Fns
- `service_impl/src/test/reporting_pending_rebooking.rs` — 4 Behavior-Tests der Banner-Regel

## Files Modified

- `rest-types/src/lib.rs` — `pub mod rebooking;` + ShortEmployeeReportTO +2 Felder + From-Mapping + serde-default-Test
- `rest-types/Cargo.toml` — `dao` optionale Dependency (unter `service-impl`)
- `rest/src/lib.rs` — Modul + RebookingApiDoc-Nest + RestStateDef-Accessor + zwei Route-Nests
- `service/src/reporting.rs` — ShortEmployeeReport +2 Felder
- `service_impl/src/reporting.rs` — RebookingBatchService-Dep + cap_active-Tracking + Alert-Berechnung
- `shifty_bin/src/main.rs` — RebookingReconciliationService-DI + ReportingService-Dep-Erweiterung
- `shifty-dioxus/Dioxus.toml` — zwei Proxy-Bloecke
- 7 bestehende Reporting-Test-Dateien — RebookingBatchService-Mock-Dep ergaenzt

## Deviations from Plan

### 1. [Rule 3 - blocking] RebookingSuggestionTO an die reale 55-02-Domain angepasst

- **Gefunden bei:** Task 1
- **Issue:** Das Plan-Artefakt listete fuer `RebookingSuggestionTO` zusaetzliche Felder
  (`voluntary_soll`, `voluntary_soll_after`, `voluntary_delta`, `voluntary_delta_after`), die die
  55-02-Domain-Struktur `RebookingSuggestion` NICHT liefert.
- **Fix:** TO auf die real vorhandenen backend-computed Felder modelliert (`batch_id`,
  `sales_person_id`, `proposed_hours`, `balance`, `balance_after`, `voluntary_ist`,
  `voluntary_ist_after`). Fake-Soll/Delta-Felder haetten die Prohibition "alles backend-computed,
  kein FE-Arithmetik" verletzt.
- **Commit:** `07687043`

### 2. [Rule 4-vermieden] Optionale GET-Liste ausgelassen

- **Gefunden bei:** Task 2
- **Issue:** Fuer `GET /rebooking-suggestions` (Liste offener Vorschlaege) existiert kein
  list-pending-Service/DAO in 55-01/02.
- **Entscheidung:** Ausgelassen (im Plan als optional markiert). Ein neuer DAO/Service-Lookup waere
  eine Rule-4-Architektur-Aenderung und nicht Teil dieses Plans. Die 4 Kern-Routes decken
  REB-MANUAL + HR-ALERT vollstaendig ab.

## Deferred Issues

- **`cargo clippy --workspace --all-targets`** meldet `clippy::cloned-ref-to-slice-refs` in
  `service_impl/src/test/voluntary_stats.rs` (pre-existing, nicht von 55-03 beruehrt). Der
  dokumentierte Projekt-Gate `cargo clippy --workspace -- -D warnings` ist gruen. Siehe
  `deferred-items.md`.

## Gate Results

- `cargo build --workspace`: gruen
- `cargo test --workspace`: gruen (785 service_impl inkl. 4 neue + 66 Integration; alle Crates 0 failed)
- `cargo clippy --workspace -- -D warnings`: gruen (dokumentierter Projekt-Gate)
- OpenAPI: alle 4 Handler tragen `#[utoipa::path]` + sind in `RebookingApiDoc` → zentrale `ApiDoc` registriert; alle neuen DTOs `ToSchema`
- SQLx: `.sqlx/` unveraendert (keine neuen compile-time-checked Queries)

## TDD Gate Compliance

Die vier Behavior-Tests (`reporting_pending_rebooking.rs`) + die zwei rest-types-Tests wurden vor
der Implementierung geschrieben (RED) und nach der Berechnung/Handler-Verdrahtung gruen (GREEN).
Da RED/GREEN im selben Task-Commit landen (Handler+DTO+Berechnung sind eng gekoppelt), tragen die
Task-Commits `feat(...)`; separate `test(...)`-Gate-Commits wurden nicht angelegt.

## Self-Check: PASSED

- Created files verified on disk (rest-types/src/rebooking.rs, rest/src/rebooking.rs,
  reporting_pending_rebooking.rs, 55-03-SUMMARY.md).
- Task commits verified in git log (07687043, 8eb35d4a, 5216354f).
