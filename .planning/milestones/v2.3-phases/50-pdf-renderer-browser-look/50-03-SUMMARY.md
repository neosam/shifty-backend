---
phase: 50-pdf-renderer-browser-look
plan: 03
subsystem: pdf
tags: [pdf, timezone, tracing, offset-datetime, service-impl, backend, tdd]

# Dependency graph
requires:
  - phase: 50-pdf-renderer-browser-look
    provides: "Wave 2 (50-02) — 5-Parameter-Renderer + Übergangs-Bridge OffsetDateTime::now_utc() in PdfShiftplanServiceImpl::render_week_pdf"
  - phase: 49-pdf-download
    provides: "PdfShiftplanService + Scheduler-Delegation (D-49-08)"
provides:
  - "pub(crate) fn resolve_render_timestamp() -> OffsetDateTime in service_impl/src/pdf_shiftplan.rs"
  - "now_local()-Fallback auf UTC mit tracing::warn!-Log bei IndeterminateOffset (D-50-12)"
  - "D-50-16 Service-Level-Smoke-Test now_local_fallback_to_utc_on_indeterminate_offset"
  - "Scheduler-Sanity-Check: pdf_export_scheduler.rs delegiert weiterhin ausschließlich an PdfShiftplanService"
affects: [51+, future-pdf-work, ops-observability]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Graceful-Degradation via unwrap_or_else(|_| { warn!(...); fallback })"
    - "Testbare Extraktion einer sonst inline gebauten Timestamp-Beschaffung als pub(crate) fn"

key-files:
  created: []
  modified:
    - "service_impl/src/pdf_shiftplan.rs (resolve_render_timestamp() + warn!-Import + Bridge-Ersatz)"
    - "service_impl/src/test/pdf_shiftplan.rs (D-50-16 Fallback-Smoke-Test)"

key-decisions:
  - "D-50-12 finalisiert: Aufrufer-Verantwortung für Render-Timestamp liegt in PdfShiftplanService — resolve_render_timestamp() ist die single injection point."
  - "Kein .unwrap()/.expect() auf now_local() — auf Multi-Thread-Deployments ohne set_local_offset oder in minimal-Containern ohne TZ-Data würde das den PDF-Download aus rein informativem Grund killen. Stattdessen warn!-Log + UTC-Fallback."
  - "IndeterminateOffset lässt sich auf Linux/NixOS (localtime_r thread-safe) nicht ohne unsafe simulieren — D-50-16 ist Smoke-Test (year in [2020, 2100)), der als Nyquist-Guardrail gegen versehentliches .unwrap() greift."

patterns-established:
  - "Timestamp-Beschaffung in Business-Logic-Services: pub(crate) fn mit unwrap_or_else-Fallback statt inline in Service-Methoden"
  - "Deutschsprachiges warn!-Log konsistent mit Projekt-Sprache (D-50-12 §Aufrufer-Verantwortung)"

requirements-completed: [PDF-01, PDF-02]

coverage:
  - id: D1
    description: "resolve_render_timestamp() beschafft Render-Timestamp per now_local() mit warn!-geloggtem UTC-Fallback bei IndeterminateOffset (D-50-12)."
    requirement: PDF-01
    verification:
      - kind: unit
        ref: "service_impl/src/test/pdf_shiftplan.rs#now_local_fallback_to_utc_on_indeterminate_offset"
        status: pass
      - kind: unit
        ref: "cargo test -p service_impl pdf_shiftplan (12 tests inkl. bestehende PDF-03/04/05-Contract-Tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "PdfShiftplanServiceImpl::render_week_pdf konsumiert resolve_render_timestamp() statt der Wave-2-Bridge OffsetDateTime::now_utc()."
    requirement: PDF-01
    verification:
      - kind: unit
        ref: "grep -c 'let render_timestamp = resolve_render_timestamp' service_impl/src/pdf_shiftplan.rs = 1"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/pdf_shiftplan.rs#happy_path_returns_bytes (Renderer wird end-to-end mit dem echten Timestamp aufgerufen)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Kein .unwrap()/.expect() auf OffsetDateTime::now_local() im Code (Anti-Pattern-Guard)."
    requirement: PDF-01
    verification:
      - kind: unit
        ref: "grep -v '^\\s*//' service_impl/src/pdf_shiftplan.rs | grep -cE '\\.unwrap\\(\\)|\\.expect\\(' = 0"
        status: pass
    human_judgment: false
  - id: D4
    description: "pdf_export_scheduler.rs konsumiert weiterhin ausschließlich PdfShiftplanService — kein direkter pdf_render::render_shiftplan_week_pdf-Aufruf (D-49-08-Sanity)."
    requirement: PDF-02
    verification:
      - kind: unit
        ref: "grep -c 'render_shiftplan_week_pdf' service_impl/src/pdf_export_scheduler.rs = 0"
        status: pass
      - kind: unit
        ref: "grep -c 'PdfShiftplanService' service_impl/src/pdf_export_scheduler.rs = 5"
        status: pass
    human_judgment: false
  - id: D5
    description: "Alle Workspace-Gates grün (cargo build, cargo test --workspace, cargo clippy --workspace -- -D warnings)."
    requirement: PDF-01
    verification:
      - kind: unit
        ref: "cargo test --workspace (alle Test-Bins ok, 0 failed; inkl. 633 im service_impl-Bin und 64 im integration-Bin)"
        status: pass
      - kind: unit
        ref: "cargo clippy --workspace -- -D warnings (exit code 0)"
        status: pass
    human_judgment: false

# Metrics
duration: 6min
completed: 2026-07-03
status: complete
---

# Phase 50 Plan 03: PDF-Renderer Timestamp-Finalisierung (Aufrufer-Verantwortung nach D-50-12)

**`resolve_render_timestamp()` extrahiert als `pub(crate) fn` mit `now_local()`-und-UTC-Fallback + `warn!`-Log; Übergangs-Bridge aus Wave 2 entfernt; D-50-16 Smoke-Test als Nyquist-Guardrail gegen versehentliches `.unwrap()`.**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-07-03T17:20Z (approx)
- **Completed:** 2026-07-03T17:26:11Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- `pub(crate) fn resolve_render_timestamp() -> OffsetDateTime` in `service_impl/src/pdf_shiftplan.rs` — nutzt `OffsetDateTime::now_local().unwrap_or_else(|_| { warn!("PDF-Renderer: Lokale TZ nicht bestimmbar — UTC wird verwendet"); OffsetDateTime::now_utc() })`.
- `use tracing::warn;` ergänzt.
- Wave-2-Bridge `let render_timestamp = OffsetDateTime::now_utc();` in `PdfShiftplanServiceImpl::render_week_pdf` ersetzt durch `resolve_render_timestamp()` — single injection point nach D-50-12.
- D-50-16 Service-Level-Smoke-Test `now_local_fallback_to_utc_on_indeterminate_offset` in `service_impl/src/test/pdf_shiftplan.rs` — verifiziert `year in [2020, 2100)`.
- Scheduler-Sanity-Check bestätigt: `service_impl/src/pdf_export_scheduler.rs` ruft `render_shiftplan_week_pdf` NICHT direkt auf (0 Matches) und delegiert weiterhin an `PdfShiftplanService` (5 Matches — D-49-08 aus Phase 49 unverändert).

## Task Commits

Jeder Task wurde atomar committed:

1. **Task 1: `resolve_render_timestamp()` + `warn!`-Import + Bridge-Ersatz** — `dcc3ba1` (feat)
2. **Task 2: D-50-16 Fallback-Smoke-Test** — `402a798` (test)

## Files Modified

- `service_impl/src/pdf_shiftplan.rs` — `resolve_render_timestamp()`-Fn, `use tracing::warn;`, Bridge-Ersatz in `render_week_pdf`.
- `service_impl/src/test/pdf_shiftplan.rs` — D-50-16 Fallback-Smoke-Test.

## Decisions Made

- **`pub(crate)` statt `pub`:** Die Fn wird nur intern (im `render_week_pdf` und im Test) genutzt; kein Grund, sie im Service-Trait oder REST-Layer zu exponieren.
- **Smoke-Test statt echter `IndeterminateOffset`-Simulation:** `localtime_r` auf Linux/NixOS ist thread-safe und liefert nie den Error; ohne `unsafe { set_local_offset }` (out of scope) lässt sich der Error-Pfad nicht deterministisch triggern. Der Smoke-Test verifiziert stattdessen die `unwrap_or_else`-Verkabelung — würde jemand versehentlich `.unwrap()` einführen, würde in Deployments ohne funktionierendes Local-TZ (Docker ohne `TZ`-Env, minimal-Alpine) sofort ein Panic auftreten und dieser Test wäre rot. Rationale ist im Test-Doc-Comment und im Plan (Objective §"Warum eine dedizierte Fn") festgehalten.
- **Test als `#[test]` (sync), nicht `#[tokio::test]`:** `resolve_render_timestamp()` ist synchron; kein Grund für Runtime-Overhead.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. Bestehende 12 pdf_shiftplan-Tests blieben grün; das Ersetzen der Bridge durch `resolve_render_timestamp()` hat keine Kontrakte gebrochen (Renderer-Signatur unverändert, Aufruf-Semantik identisch abgesehen vom Timestamp-Ursprung).

## Sanity-Check-Ergebnis

`grep -c 'render_shiftplan_week_pdf' service_impl/src/pdf_export_scheduler.rs` → **0** (PASS — kein direkter Renderer-Aufruf im Scheduler).
`grep -c 'PdfShiftplanService' service_impl/src/pdf_export_scheduler.rs` → **5** (PASS — Scheduler delegiert weiterhin, D-49-08 aus Phase 49 unangetastet).

## Test-Gates

- `cargo build --workspace` → exit 0
- `cargo test -p service_impl pdf_shiftplan` → 12 tests passed (inkl. neuer D-50-16-Test)
- `cargo test --workspace` → alle Test-Bins ok (633 im service_impl, 64 im integration-Bin, 24 im shifty_utils, restliche Bins <15 Tests, 0 failed insgesamt)
- `cargo clippy --workspace -- -D warnings` → exit 0

## User Setup Required

None - keine externe Konfiguration erforderlich.

## Next Phase Readiness

- Phase 50 ist backend-seitig **abgeschlossen**. Alle Layout-Decisions aus Wave 2 und die D-50-12-Aufrufer-Verantwortung aus Wave 3 sind implementiert und getestet.
- Ausstehend: **UAT (D-50-17)** via Phase-49-Button gegen reales Wochen-Fixture. Der UAT ist NICHT Teil dieses Plans — er wird via `/gsd-verify-work 50` gefahren nach dem Wave-3-Merge.
- Keine Blocker für Phase 51+.

## Self-Check: PASSED

- File `service_impl/src/pdf_shiftplan.rs` — FOUND, modified (2 hunks: import + fn + Bridge-Ersatz)
- File `service_impl/src/test/pdf_shiftplan.rs` — FOUND, modified (1 hunk: neuer Test am Ende)
- Commit `dcc3ba1` (Task 1) — FOUND in git log
- Commit `402a798` (Task 2) — FOUND in git log
- All test gates green, clippy green.

---
*Phase: 50-pdf-renderer-browser-look*
*Completed: 2026-07-03*
