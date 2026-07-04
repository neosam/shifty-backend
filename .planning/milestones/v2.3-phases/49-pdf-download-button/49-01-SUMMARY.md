---
phase: 49-pdf-download-button
plan: 01
subsystem: service/pdf-shiftplan
status: complete
tags: [pdf, service-trait, business-logic-tier, tdd]
requires: [service/pdf_render, service/week_status, service/shiftplan, service/sales_person]
provides:
  - service::pdf_shiftplan::PdfShiftplanService
  - service::pdf_shiftplan::MockPdfShiftplanService
  - service_impl::pdf_shiftplan::PdfShiftplanServiceImpl
  - service_impl::pdf_shiftplan::PdfShiftplanServiceDeps
  - service_impl::pdf_shiftplan::filter_active
affects: []
tech-stack:
  added: []
  patterns: [gen_service_impl!, mockall::automock, business-logic-tier assembler]
key-files:
  created:
    - service/src/pdf_shiftplan.rs
    - service_impl/src/pdf_shiftplan.rs
    - service_impl/src/test/pdf_shiftplan.rs
  modified:
    - service/src/lib.rs
    - service_impl/src/lib.rs
    - service_impl/src/test/mod.rs
decisions:
  - "D-49-05: PdfShiftplanService als Business-Logic-Tier (consumes ShiftplanViewService + SalesPersonService + WeekStatusService + PermissionService + TransactionDao)"
  - "D-49-06: Trait-Signatur render_week_pdf(shiftplan_id: Uuid, year: u32, calendar_week: u8, context, tx) -> Result<Vec<u8>, ServiceError>; internal WeekStatus gate returns ValidationError für Unset/InPlanning"
  - "D-49-07: Aufrufer-context wird an alle konsumierten Services weitergereicht; kein Authentication::Full-Rewrite im Service"
metrics:
  duration_min: 5
  tasks_completed: 2
  tests_added: 10
  tests_passing: 10
  commits: [9ba632b, 6b75c7e]
requirements: [PDF-03, PDF-04]
---

# Phase 49 Plan 01: Backend Service Trait + Impl + Unit Tests Summary

Neuer Business-Logic-Service `PdfShiftplanService` (Trait + Impl + 10 Unit-Tests) als DRY-Kern für den Wochen-PDF-Download — assembliert WeekStatus-Gate, ShiftplanView, aktive SalesPerson-Filterung und pure `pdf_render`.

## Was passierte

Wave 1 der Phase 49 lieferte den zentralen Assembler-Service, den sowohl der REST-Handler (Plan 02) als auch der Scheduler-Refactor (Plan 03) konsumieren werden. TDD-first: erst 10 Tests (RED), dann die Impl (GREEN).

**RED-Commit `9ba632b`:** Trait `PdfShiftplanService` in `service/src/pdf_shiftplan.rs` (mit `#[automock]`, `type Context`, `type Transaction`, `render_week_pdf`-Signatur), Impl-Skelett `PdfShiftplanServiceImpl` in `service_impl/src/pdf_shiftplan.rs` (5 Deps via `gen_service_impl!`, Body returns `InternalError`), 10 `#[tokio::test]` in `service_impl/src/test/pdf_shiftplan.rs`. 9 Tests fail wie erwartet, 1 pure-fn-Test grün.

**GREEN-Commit `6b75c7e`:** `render_week_pdf`-Body implementiert exakt in der geforderten Reihenfolge:
1. `week_status_service.get_week_status(...)` — Gate. Nicht in `{Planned, Locked}` ⇒ `ServiceError::ValidationError(InvalidValue)` mit menschenlesbarer Message; kein weiterer Aufruf.
2. `shiftplan_view_service.get_shiftplan_week(...)`.
3. `sales_person_service.get_all(...)` + `filter_active` (pure fn, `deleted.is_none()`).
4. `pdf_render::render_shiftplan_week_pdf(...)`.

Der `context` wird per `.clone()` an die ersten beiden Aufrufe durchgereicht, per move an `get_all` — kein `Authentication::Full` im Code.

## Was gebaut wurde

### Neue Dateien
- **`service/src/pdf_shiftplan.rs`** (58 LOC) — Trait-Definition + Doc-Comment (Business-Logic-Tier-Note, Defense-in-Depth-Erklärung).
- **`service_impl/src/pdf_shiftplan.rs`** (140 LOC) — `PdfShiftplanServiceImpl` + `gen_service_impl!` mit 5 Deps, `new()`-Konstruktor, `Debug`-Impl, pure `filter_active`-Helper, `PdfShiftplanService`-Impl mit Assemble-Path.
- **`service_impl/src/test/pdf_shiftplan.rs`** (376 LOC) — 10 Tests + `TestDeps` + Helpers.

### Modul-Registrations
- `pub mod pdf_shiftplan;` in `service/src/lib.rs` (Zeile 27)
- `pub mod pdf_shiftplan;` in `service_impl/src/lib.rs` (Zeile 26)
- `#[cfg(test)] pub mod pdf_shiftplan;` in `service_impl/src/test/mod.rs` (Zeile 30-31)

### Neue Symbols
- `service::pdf_shiftplan::PdfShiftplanService` (trait, `#[automock]`)
- `service::pdf_shiftplan::MockPdfShiftplanService` (via automock)
- `service_impl::pdf_shiftplan::PdfShiftplanServiceImpl<Deps>`
- `service_impl::pdf_shiftplan::PdfShiftplanServiceDeps`
- `service_impl::pdf_shiftplan::filter_active(&[SalesPerson]) -> Vec<SalesPerson>` (pure helper, `pub(crate)`)

## Test-Ergebnisse

`cargo test -p service_impl pdf_shiftplan -- --nocapture` — **10 passed, 0 failed**:

| # | Test | Deckt ab |
|---|------|----------|
| 1 | `happy_path_returns_bytes` | PDF-03 happy-path (WeekStatus=Planned) |
| 2 | `week_status_locked_returns_bytes` | PDF-03 happy-path (WeekStatus=Locked) |
| 3 | `week_status_unset_returns_validation_error` | PDF-04 Defense-in-Depth |
| 4 | `week_status_in_planning_returns_validation_error` | PDF-04 Defense-in-Depth |
| 5 | `filters_deleted_sales_persons` | PDF-05 (pure `filter_active`) |
| 6 | `service_render_does_not_leak_deleted_sales_persons` | PDF-05 (Service-E2E) |
| 7 | `service_forwards_caller_context_to_dependencies` | D-49-07 Context-Weitergabe |
| 8 | `view_error_bubbles_up` | Fehler-Propagation |
| 9 | `sales_person_error_bubbles_up` | Fehler-Propagation |
| 10 | `week_status_error_bubbles_up` | Fehler-Propagation |

Full `cargo test -p service_impl`: **626 passed, 0 failed** — keine Regressionen.

## Clippy-Status

`cargo clippy --workspace -- -D warnings` — grün (0 Warnungen).

## Grep-Assertions (Plan-Acceptance-Criteria)

- `matches!(status, WeekStatus::Planned | WeekStatus::Locked)` — Treffer in `service_impl/src/pdf_shiftplan.rs:114` ✓
- `sp.deleted.is_none()` — Treffer in `service_impl/src/pdf_shiftplan.rs:90` (filter_active) ✓
- `pdf_render::render_shiftplan_week_pdf` — 1 aktiver Aufruf in `service_impl/src/pdf_shiftplan.rs:136` ✓
- `Authentication::Full` (non-comment) — 0 Treffer ✓

## Abweichungen vom Plan

**1. [Rule 1 - Bug] Falscher `ValidationFailureItem`-Konstruktor im Plan-Text**
- **Gefunden bei:** Task 2 Implementation.
- **Issue:** Plan §Task 2 action §1 verwendete `ValidationFailureItem { field, reason }`-Konstruktor mit Feldern — der reale Enum in `service/src/lib.rs:57` hat aber `ModificationNotAllowed(Arc<str>)`, `InvalidValue(Arc<str>)`, `IdDoesNotExist(...)`, `Duplicate`, `OverlappingPeriod(Uuid)` — keine `field`/`reason`-Struktur.
- **Fix:** `InvalidValue(Arc<str>)` gewählt, weil semantisch am nächsten am Plan-Intent ("Woche im Status X — kein Download") und `ValidationError(Arc<[...]>)` als äußerer Konstruktor korrekt erhalten bleibt. Test-Assertions matchen `ServiceError::ValidationError(_)` und sind daher unabhängig vom inneren Variant.
- **Files modified:** `service_impl/src/pdf_shiftplan.rs:116-118`.
- **Commit:** `6b75c7e`.

**2. [Rule 2 - Test-Ergonomie] `filters_deleted_sales_persons` prüft die pure fn statt Byte-Grep**
- **Gefunden bei:** Task 1 Test-Design.
- **Issue:** Plan §Task 2 action beschrieb "Text-Suche im PDF-Byte-Stream via `String::from_utf8_lossy`" als Fallback für den Filter-Test. Das ist nicht robust: `pdf_render` verwendet `printpdf` mit FlateDecode-komprimierten Content-Streams — Klartext-Namen erscheinen NICHT im Byte-Stream.
- **Fix:** Filter als separate pure fn `filter_active` in `service_impl/src/pdf_shiftplan.rs:88` extrahiert. Test `filters_deleted_sales_persons` prüft die pure fn direkt (deterministisch). Zusätzlicher E2E-Test `service_render_does_not_leak_deleted_sales_persons` sichert die Verwendung im `render_week_pdf`-Body. Diese Alternative wird vom Plan explizit als "schlanker + robuster" empfohlen (Task 2 action letzter Absatz).
- **Files modified:** `service_impl/src/pdf_shiftplan.rs:86-92`, `service_impl/src/test/pdf_shiftplan.rs:196-233`.
- **Commit:** `6b75c7e` (fn), `9ba632b` (Tests).

**3. [Rule 3 - Blocking] `-D warnings`-Guards für RED-Skelett-Impl**
- **Gefunden bei:** Task 1 Build.
- **Issue:** RED-Skelett muss kompilieren, verwendet aber `pdf_render` und `filter_active` noch nicht — `unused_imports` und `dead_code` sind im Workspace als Errors konfiguriert.
- **Fix:** `#[allow(unused_imports)]` auf `use crate::pdf_render;` und `#[allow(dead_code)]` auf `filter_active` in der RED-Phase. GREEN-Commit entfernt beide `#[allow]`s automatisch, weil Symbole dann genutzt werden.
- **Files modified:** `service_impl/src/pdf_shiftplan.rs` (nur im RED-Snapshot).
- **Commit:** `9ba632b` (temporär).

## Commits

- `9ba632b` — `test(49-01): add failing PdfShiftplanService trait + impl skeleton + unit tests (RED)` — 6 files, RED-Phase.
- `6b75c7e` — `feat(49-01): implement PdfShiftplanService with WeekStatus gate + active filter` — 1 file, GREEN-Phase.

## Locked Decisions manifest im Code

- **D-49-05** — `PdfShiftplanServiceImpl` in `service_impl/src/pdf_shiftplan.rs:39-58` konsumiert genau die vorgesehenen 5 Deps; keine Domain-Service-Zyklen.
- **D-49-06** — Trait-Signatur `render_week_pdf` in `service/src/pdf_shiftplan.rs:47-55` exakt wie spezifiziert; internes Gate in `service_impl/src/pdf_shiftplan.rs:107-120`.
- **D-49-07** — `context.clone()` an `get_week_status` (Zeile 111), `get_shiftplan_week` (Zeile 125); `context` per move an `get_all` (Zeile 130). Kein `Authentication::Full`.

## Nächste Schritte

- **Plan 49-02** — REST-Handler `GET /shiftplan/{id}/week/{y}/{w}/pdf` konsumiert `PdfShiftplanService` via `MockPdfShiftplanService`-Tests.
- **Plan 49-03** — Scheduler-Refactor ersetzt duplizierte Assemble-Logik in `pdf_export_scheduler.rs` durch Aufruf an `PdfShiftplanService::render_week_pdf` (mit `Authentication::Full`).

## Self-Check: PASSED

Verifiziert (`git log --oneline`):
- FOUND: `9ba632b`
- FOUND: `6b75c7e`

Verifiziert (Files exist):
- FOUND: `service/src/pdf_shiftplan.rs`
- FOUND: `service_impl/src/pdf_shiftplan.rs`
- FOUND: `service_impl/src/test/pdf_shiftplan.rs`

Verifiziert (Modul-Registrations):
- FOUND: `pub mod pdf_shiftplan;` in `service/src/lib.rs`
- FOUND: `pub mod pdf_shiftplan;` in `service_impl/src/lib.rs`
- FOUND: `pub mod pdf_shiftplan;` in `service_impl/src/test/mod.rs`
