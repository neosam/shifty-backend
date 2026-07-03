---
phase: 46-backend-hygiene-i18n
plan: 03
subsystem: testing
tags: [rest, utoipa, openapi, content-type, drift-guard, hygiene]

requires:
  - phase: 08-vacation-balance
    provides: "openapi_surface.rs Muster (Route-Enumeration via ApiDoc::openapi().paths)"
provides:
  - "REST-Content-Type-Drift-Guard: iteriert 120 registrierte Operationen, prüft jede 2xx-Response gegen Whitelist (application/json + text/plain)"
  - "Grandfather-Liste KNOWN_NO_BODY_2XX für 13 pre-existing Handler die 200/201 ohne Body deklarieren — dokumentiert für spätere Cleanup-Phase"
  - "Coverage-Sanity-Test verhindert stille Regression falls utoipa-API bricht und Iterator plötzlich leer liefert"
affects: [future-rest-phases, hyg-cleanup-phases]

tech-stack:
  added: []
  patterns:
    - "OpenAPI-Reflection statt Live-Roundtrip für Contract-Assertions (D-46-03) — deterministisch, ohne DB/Auth/Server-Fixture"
    - "Grandfather-Liste-Muster für pre-existing Drift: neue Verstöße failen sofort, existierende sind explizit gelistet mit TODO-Kommentar"

key-files:
  created:
    - rest/tests/content_type_surface.rs
  modified: []

key-decisions:
  - "D-46-03: OpenAPI-Reflection statt Live-Roundtrip (kein DB-/Auth-Setup nötig, deterministisch)"
  - "Whitelist bewusst minimal (2 Einträge) — jeder neue Content-Type erfordert expliziten Edit mit Handler-Referenz"
  - "204-Responses spec-conform vom Content-Deklarations-Zwang ausgenommen (leere Body-Semantik erlaubt)"
  - "13 pre-existing 200/201-ohne-Body-Handler grandfathered via KNOWN_NO_BODY_2XX-Liste — Cleanup out-of-scope für HYG-05, aber dokumentiert für Follow-up-Hygiene-Phase"
  - "Coverage-Sanity-Threshold: MIN_OPERATIONS_EXPECTED = 40 (aktuell 120) — fängt stille utoipa-API-Brüche ab"

patterns-established:
  - "OpenAPI-Reflection-Drift-Guard: `ApiDoc::openapi().paths.paths` + `PathItem` HTTP-Method-Fields iterieren, pro Operation `responses.responses` (BTreeMap<String, RefOr<Response>>) inspizieren"
  - "Strukturierter Fail-Report: Vec<String> sammeln, am Ende `assert!(offenders.is_empty(), ...)` mit vollständiger Aufzählung — Fix-Hinweis pro Offender"
  - "Grandfather-Liste als const &[(method, path, status)] — jedem Eintrag mit Kommentar warum + Cleanup-Pfad"

requirements-completed: [HYG-05]

coverage:
  - id: D1
    description: "REST-Content-Type-Drift-Guard: automatisierter Test iteriert alle utoipa/OpenAPI-registrierten Operationen und failt hart bei unbekanntem Content-Type oder fehlender Deklaration"
    requirement: HYG-05
    verification:
      - kind: unit
        ref: "rest/tests/content_type_surface.rs#every_response_declares_known_content_type"
        status: pass
      - kind: unit
        ref: "rest/tests/content_type_surface.rs#content_type_surface_covers_all_openapi_operations"
        status: pass
    human_judgment: false
  - id: D2
    description: "Manueller Mutation-Sanity-Check: temporär `content_type = \"application/xml\"` in `rest/src/report.rs:53` injiziert, Test failt mit strukturiertem Offender-Report, danach revertet"
    requirement: HYG-05
    verification:
      - kind: manual_procedural
        ref: "Mutation report.rs:53 → cargo test -p rest --test content_type_surface every_response_declares_known_content_type → FAILED erwartet + reverted"
        status: pass
    human_judgment: false

duration: 20min
completed: 2026-07-02
status: complete
---

# Phase 46 Plan 03: HYG-05 REST Content-Type Surface Drift-Guard Summary

**OpenAPI-Reflection-Test iteriert alle 120 utoipa-registrierten Operationen und failt hart bei unbekanntem Response-Content-Type oder fehlender Deklaration — verhindert schleichendes Content-Type-Drift analog zum etablierten `openapi_surface.rs`-Muster.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-07-02T21:00:00Z
- **Completed:** 2026-07-02T21:20:38Z
- **Tasks:** 2 (RED-Baseline + GREEN/REFACTOR)
- **Files modified:** 1 (created)

## Accomplishments

- **RED-Baseline erhoben:** Test-Datei mit dediziertem Dump-Test schrieb die tatsächliche Content-Type-Verteilung nach `/tmp/hyg05-baseline.log`. Ergebnis: **120 Operationen**, nur **2 distinct content-types** (`application/json`, `text/plain`), **38 Operationen mit 2xx-Response ohne deklarierten Content-Type**, davon 25× spec-conforme `204`, **13× pre-existing Drift (200/201 ohne Body)**.
- **GREEN-Test `every_response_declares_known_content_type`:** iteriert alle Operationen; failt hart wenn (a) ein deklarierter Content-Type nicht auf `ALLOWED_CONTENT_TYPES` steht oder (b) eine non-204-2xx-Response weder Content deklariert noch auf `KNOWN_NO_BODY_2XX` steht.
- **Coverage-Sanity-Test `content_type_surface_covers_all_openapi_operations`:** guarded gegen stille utoipa-API-Brüche (Iterator liefert plötzlich 0 → Test 1 würde trivial grün) mit Threshold `MIN_OPERATIONS_EXPECTED = 40`.
- **Mutation-Sanity-Check:** in `rest/src/report.rs:53` temporär `content_type = "application/xml"` injiziert → Test failt mit strukturiertem Offender-Report (`GET /report status=200: unknown content-type 'application/xml'...`) → revertet.
- **Determinismus verifiziert:** drei aufeinanderfolgende Runs grün.

## Task Commits

Beide Tasks werden per GSD-Auto-Commit gebündelt (RED-Baseline war reines Erhebungswerkzeug, im finalen Commit ersetzt).

1. **Task 1: HYG-05 RED-Baseline** — Erhebung der Content-Type-Verteilung, danach ersetzt.
2. **Task 2: HYG-05 GREEN + REFACTOR** — finale zwei Tests + Whitelist + Grandfather-Liste + Doc-Kommentar.

## Files Created/Modified

- `rest/tests/content_type_surface.rs` — neue Test-Datei mit zwei Tests, Whitelist-Konstanten, Grandfather-Liste, Helper `operations_of` für PathItem-HTTP-Method-Iteration.

## Decisions Made

- **D-46-03 bestätigt:** OpenAPI-Reflection statt Live-Roundtrip. Die utoipa-`#[utoipa::path(responses(...))]`-Deklaration IST der Contract, den Frontend + externe Konsumenten sehen. Runtime-Header-Mismatch zwischen Deklaration und tatsächlichem Handler-Output ist separate Concern — abgedeckt durch bestehende Domain-Tests je Handler.
- **204-Ausnahme:** Spec-conform (HTTP 204 hat definitionsgemäß keinen Body). Test überspringt 204 beim Content-Deklarations-Check.
- **Grandfather-Liste statt Test-Aufweichung:** Für 13 pre-existing Handler die 200/201 ohne Body deklarieren wurde `KNOWN_NO_BODY_2XX` als expliziter Const angelegt statt "irgendein Content-Type ist ok" zu erlauben. Cleanup dieser 13 Handler (entweder → 204 oder → echte Body-Deklaration) ist als Follow-up-Kandidat dokumentiert, out-of-scope für HYG-05 (nur Test-Layer).
- **Whitelist bewusst minimal:** nur `application/json` + `text/plain`. Neue Content-Types (z.B. `text/csv`, `application/pdf`) erfordern expliziten Whitelist-Edit mit Handler-Referenz — zwingt die Entscheidung sichtbar durch Code-Review.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Fehlende kritische Funktionalität] Grandfather-Liste für 13 pre-existing Non-204-2xx-Handler ohne Body**

- **Found during:** Task 1 (RED-Baseline-Dump)
- **Issue:** Der Plan sagt "hartes Fail sobald ein Handler ohne deklarierten Content-Type … registriert wird" (2xx-Perspektive). Der Baseline-Dump zeigt aber 38 Operationen mit 2xx-Response ohne Content, davon 25× `204` (spec-conform!) und 13× `200`/`201` (pre-existing Drift). Ohne Sonderbehandlung würde der Test von Anfang an mit 38 Offendern failen und wäre nicht mergebar.
- **Fix:** (a) 204-Responses vom Content-Deklarations-Zwang ausgenommen (spec-conform). (b) 13 pre-existing 200/201-Handler explizit als `KNOWN_NO_BODY_2XX`-const-Liste grandfathered mit per-Endpoint-Kommentar warum + Cleanup-Pfad. Neue Verstöße failen sofort.
- **Files modified:** `rest/tests/content_type_surface.rs` (const `KNOWN_NO_BODY_2XX` + `is_grandfathered`-Helper).
- **Verification:** Test grün. Mutation-Test bestätigt: unbekannter Content-Type failt sofort mit sauberer Meldung.
- **Committed in:** wird per GSD-Auto-Commit erfasst.

---

**Total deviations:** 1 auto-fixed (Rule 2: pragmatische Sonderbehandlung damit Drift-Guard einführbar wird ohne 13 pre-existing Handler in derselben Phase mit fixen zu müssen).
**Impact on plan:** Keine Scope-Creep. Cleanup der 13 grandfathered Handler ist als Follow-up-Kandidat dokumentiert (out-of-scope HYG-05).

## Issues Encountered

- **Pre-existing Clippy-Warnung in `service_impl/src/test/shiftplan_edit_lock.rs:6`** (`doc-lazy-continuation`): fires nur mit `cargo clippy --workspace --all-targets -- -D warnings`, nicht mit dem Plan-mandated `cargo clippy --workspace -- -D warnings` (matches CI-Setup in `.github/workflows/rust.yml`). Datei stammt aus Phase 40, wurde in dieser Plan-Session nicht angefasst. Out-of-scope per SCOPE BOUNDARY-Regel — als deferred-Kandidat für spätere Hygiene notiert.

## Known Stubs

Keine Stubs. Reine Test-Datei, keine UI/Data-Wire-Ups.

## Threat Flags

Keine neuen Trust-Boundary-Surfaces eingeführt (Test-Datei ohne Runtime-Effekte).

## Next Phase Readiness

- **HYG-05 abgeschlossen** — REST-Content-Type-Surface hat Drift-Guard, läuft im normalen `cargo test --workspace`-Sweep mit, keine Zusatzflags.
- **Follow-up-Kandidat (out-of-scope v2.2):** die 13 grandfathered 200/201-ohne-Body-Handler cleanen (entweder → 204 oder → Body-DTO deklarieren). Kandidat für spätere HYG-Phase.
- **Pattern etabliert:** Weitere OpenAPI-Contract-Assertions (z.B. "jeder Endpoint hat mind. eine 4xx-Error-Response deklariert", "jeder mutation-Endpoint hat einen tag") können analog dem gleichen Reflection-Muster folgen.

## Self-Check: PASSED

- `rest/tests/content_type_surface.rs` existiert (verified via file state).
- Beide Tests grün (`cargo test -p rest --test content_type_surface` → 2 passed, 0 failed).
- Determinismus: 3 aufeinanderfolgende Runs grün.
- Backend-Full-Gate:
  - `cargo test --workspace` → alle grün (rest 11 passed inkl. neue 2, service_impl 570 passed, dao_impl_sqlite 64 passed, etc.).
  - `cargo clippy --workspace -- -D warnings` → grün (Plan-mandated Invocation, matches CI).
  - `cargo build --workspace` → grün.
- Frontend WASM-Sanity: `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` → grün (unverändert, wie erwartet).
- Mutation-Sanity: unbekannter Content-Type in `report.rs:53` löste erwarteten Offender-Report aus, danach revertet.

---
*Phase: 46-backend-hygiene-i18n*
*Completed: 2026-07-02*
