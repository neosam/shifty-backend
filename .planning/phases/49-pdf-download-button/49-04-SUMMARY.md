---
phase: 49-pdf-download-button
plan: 04
subsystem: ui
tags: [dioxus, rsx, i18n, pdf, wasm, shiftplan, download, anchor-tag]

# Dependency graph
requires:
  - phase: 49-pdf-download-button/02
    provides: "REST endpoint GET /shiftplan/{id}/{y}/{w}/pdf with Cookie auth + 200/404/409 semantics + Content-Disposition"
provides:
  - "Frontend `<a>` download anchor next to the iCal button in the Shiftplan toolbar (D-49-10, D-49-11)"
  - "Pure visibility predicate `should_show_pdf_button(status, shiftplan_id) -> bool` (D-49-13)"
  - "8-case unit-test matrix covering the 4 WeekStatus × 2 Option<Uuid> truth table"
  - "New i18n `Key::PdfDownload` in all three locales (de/en/cs), value `\"PDF\"` (D-49-14)"
affects:
  - "49-05 (Wave 4: verification / manual smoke)"
  - "Future PDF-export features that reuse the pure-predicate visibility pattern"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pure-function visibility gate for RSX conditional rendering — tested via `cargo test` without a Dioxus VirtualDom"
    - "Signal-owned WeekStatus clone at file-scope binding, passed by value into pure predicate"
    - "HTML5 `download` attribute for browser-side filename hint, symmetric to Content-Disposition backend header"

key-files:
  created: []
  modified:
    - "shifty-dioxus/src/page/shiftplan.rs — pure fn + 8 tests + RSX anchor block"
    - "shifty-dioxus/src/i18n/mod.rs — Key::PdfDownload variant"
    - "shifty-dioxus/src/i18n/de.rs — DE translation"
    - "shifty-dioxus/src/i18n/en.rs — EN translation"
    - "shifty-dioxus/src/i18n/cs.rs — CS translation"

key-decisions:
  - "Ran RED → GREEN as separate atomic commits (Task 2 → Task 3): pure fn + test matrix land first, RSX block wires the callsite in the second commit. `#[allow(dead_code)]` bridged the interim clippy gate and was removed in Task 3 — visible as a two-line delta in git log."
  - "Nested `if should_show_pdf_button(...) { if let Some(sp_id) = sp_id_opt { ... } }` chosen over `sp_id_opt.unwrap()` on the plan's recommendation for cleaner RSX-level unwrap-avoidance."

patterns-established:
  - "Pattern: Pure predicate function co-located with the RSX callsite, exercised via `cargo test` — sidesteps Dioxus signal-mocking limitations documented in memory-anchor 'Dioxus Browser-Test: Datepicker'."

requirements-completed: [PDF-03, PDF-04]

coverage:
  - id: D1
    description: "New i18n key `Key::PdfDownload` present in de/en/cs with value \"PDF\""
    requirement: "PDF-03"
    verification:
      - kind: unit
        ref: "cd shifty-dioxus && cargo build --target wasm32-unknown-unknown (enum-exhaustiveness gate)"
        status: pass
      - kind: unit
        ref: "grep -c 'PdfDownload' shifty-dioxus/src/i18n/mod.rs shifty-dioxus/src/i18n/{de,en,cs}.rs → 1/1/1/1"
        status: pass
    human_judgment: false
  - id: D2
    description: "Pure fn `should_show_pdf_button(status, shiftplan_id) -> bool` returns true iff shiftplan_id is Some AND status ∈ {Planned, Locked}, exercised over the full 4×2 truth table"
    requirement: "PDF-04"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/page/shiftplan.rs::tests::pdf_button_tests::{some_id_planned,some_id_locked,some_id_unset,some_id_in_planning,none_id_planned,none_id_locked,none_id_unset,none_id_in_planning} (8/8 pass)"
        status: pass
    human_judgment: false
  - id: D3
    description: "PDF-Download `<a>` anchor rendered next to the iCal button in the Shiftplan toolbar, gated on `should_show_pdf_button`, with `download` attribute + no `target=\"_blank\"`"
    requirement: "PDF-03"
    verification:
      - kind: unit
        ref: "grep -c 'should_show_pdf_button' shifty-dioxus/src/page/shiftplan.rs = 12 (def + call + 10 test-mentions, ≥2 required)"
        status: pass
      - kind: unit
        ref: "grep -c 'schichtplan-{y}-KW{w:02}.pdf' shifty-dioxus/src/page/shiftplan.rs = 1"
        status: pass
      - kind: unit
        ref: "grep -c '/shiftplan/{}/{}/{}/pdf' shifty-dioxus/src/page/shiftplan.rs = 1"
        status: pass
      - kind: unit
        ref: "grep -A 5 'download: format!(\"schichtplan' shifty-dioxus/src/page/shiftplan.rs | grep -c 'target' = 0 (no target=\"_blank\")"
        status: pass
      - kind: unit
        ref: "cd shifty-dioxus && cargo build --target wasm32-unknown-unknown (WASM-Build-Gate)"
        status: pass
      - kind: unit
        ref: "cd shifty-dioxus && cargo clippy -p shifty-dioxus -- -D warnings"
        status: pass
    human_judgment: false
  - id: D4
    description: "End-to-end browser smoke: user on Planned/Locked week clicks button, browser downloads `schichtplan-{yyyy}-KW{ww}.pdf`; navigating to InPlanning hides the button"
    requirement: "PDF-03"
    verification:
      - kind: manual_procedural
        ref: "Manual UAT deferred to Wave-4 Plan 49-05 verification (per plan `<action>` — 'nicht Blocker fuer Commit, aber im Summary vermerken')"
        status: unknown
    human_judgment: true
    rationale: "Full browser download + filename-check + navigate-to-InPlanning behavior requires a real Dioxus runtime + backend + logged-in Employee session; pure-fn unit tests cover the visibility contract, but click-to-file behavior is judgment-only. Nyquist manual-only per VALIDATION.md."

# Metrics
duration: 14min
completed: 2026-07-03
status: complete
---

# Phase 49 Plan 04: Frontend Button + i18n + Predicate Test Summary

**PDF-Download `<a>`-Anchor neben iCal-Button in der Shiftplan-Toolbar, gegated durch pure fn `should_show_pdf_button` mit 8-case Wahrheits-Tabelle und i18n-Label `PDF` in de/en/cs.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-03T14:36:57Z
- **Completed:** 2026-07-03T14:50:38Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Neuer i18n-Key `Key::PdfDownload` in allen drei Locales (de/en/cs), Wert konsistent `"PDF"` (symmetrisch zum iCal-Kürzel)
- Pure Sichtbarkeits-Prädikat `should_show_pdf_button(status, shiftplan_id) -> bool` — true genau dann, wenn `Some(shiftplan_id)` UND `WeekStatus ∈ {Planned, Locked}`
- 8/8 Unit-Tests decken die vollständige 4×2-Matrix ab (`tests::pdf_button_tests::*`)
- `<a>`-Download-Anchor in der Shiftplan-Toolbar direkt neben dem iCal-Button; Styling 1:1 vom iCal-Precedent; `download="schichtplan-{yyyy}-KW{ww}.pdf"`; bewusst KEIN `target="_blank"`
- Cookie-Auth flows automatisch — kein WASM-Fetch/Blob-Umweg

## Task Commits

Jeder Task wurde atomar committed:

1. **Task 1: i18n-Key + 3 Übersetzungen** — `5de95e2` (feat)
2. **Task 2: Pure Predikat + 8 Unit-Tests** — `1167cd2` (test)
3. **Task 3: RSX-Block für PDF-Anchor** — `f9b2e9d` (feat)

**Plan metadata:** [pending final `docs(49-04):` commit]

_Task 2 is committed as `test(...)` because it lands the pure fn + full unit-test matrix atomically before Task 3 wires the RSX callsite — this preserves the RED → GREEN sequence in git log even though the fn itself is trivially implemented in the same commit._

## Files Created/Modified
- `shifty-dioxus/src/i18n/mod.rs` — Neuer `Key::PdfDownload` Enum-Variant nach `PersonalCalendarExport`, mit Doc-Kommentar zur Phase-49-Semantik
- `shifty-dioxus/src/i18n/de.rs` — `i18n.add_text(Locale::De, Key::PdfDownload, "PDF")` nach dem PersonalCalendarExport-Block
- `shifty-dioxus/src/i18n/en.rs` — analog `Locale::En`
- `shifty-dioxus/src/i18n/cs.rs` — analog `Locale::Cs`
- `shifty-dioxus/src/page/shiftplan.rs` — Pure fn `should_show_pdf_button` file-scope (Task 2); Test-Sub-Modul `pdf_button_tests` mit 8 Test-Funktionen; RSX-Anchor-Block direkt nach dem iCal-Anchor in der Toolbar-Row (Task 3)

## Decisions Made
- **Atomicity trade-off Task 2 → Task 3:** Um die RED → GREEN-Sequenz in git-log sichtbar zu halten, wurde die pure fn in Task 2 mit `#[allow(dead_code)]` versehen (fn ist definiert + getestet, aber im Task-2-Commit noch nicht konsumiert). Task 3 entfernt das Attribut und wired die RSX-Nutzung — 2-Zeilen-Delta plus Anchor-Block. Alternative wäre gewesen, Task 2 und 3 in einen einzigen Commit zusammenzuführen; die separaten Commits sind aber wertvoller für das git-log-Storytelling und die spätere Nachvollziehbarkeit (unit tests landen zuerst, dann feature).
- **Nested `if let Some(sp_id)` statt `.unwrap()`:** Der RSX-Block nutzt die vom Plan empfohlene cleanere Form `if should_show_pdf_button(ws, sp_id_opt) { if let Some(sp_id) = sp_id_opt { ... } }` statt `sp_id_opt.unwrap()`. Struktur-Duplikat mit dem Guard-Prädikat ist bewusst — Rust's borrow-checker akzeptiert das anstandslos und die Rendering-Logik bleibt panic-frei.
- **Test-UUID stabil:** Für die 4 `Some(...)`-Test-Fälle wird eine deterministische UUID via `Uuid::from_u128(0x4949_0404_...)` erzeugt — kein Random, damit Test-Ausgabe reproduzierbar bleibt und die UUID-Bytes den Kontext (Phase 49 Plan 04) selbst-dokumentieren.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `#[allow(dead_code)]` bridge for Task 2 clippy gate**
- **Found during:** Task 2 (Pure predicate + tests commit)
- **Issue:** `cargo clippy -p shifty-dioxus -- -D warnings` failt in Task 2 auf `function 'should_show_pdf_button' is never used` — die fn wird erst in Task 3 vom RSX-Block konsumiert. Plan-Task-2-Acceptance-Criterion `cargo clippy -- -D warnings gruen` konnte ohne Bridge nicht atomar erfüllt werden.
- **Fix:** Task-2-Commit annotiert die fn mit `#[allow(dead_code)]` plus Kommentar, dass Task 3 das Attribut entfernt. Task-3-Commit dropt das Attribut und wired den Callsite in der gleichen Änderung.
- **Files modified:** `shifty-dioxus/src/page/shiftplan.rs`
- **Verification:** `cargo clippy -p shifty-dioxus -- -D warnings` gruen nach beiden Commits.
- **Committed in:** `1167cd2` (Task 2) → `f9b2e9d` (Task 3, drop attribute + add call)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Minimaler struktureller Bridge, um die vom Plan geforderte atomare Task-Trennung zu bewahren. Kein Scope-Creep, kein Verhaltens-Change.

## Issues Encountered
- `cargo test should_show_pdf_button` matcht keine Tests, weil die Test-Namen `some_id_planned` / `none_id_locked` / ... heißen und der Filter ein Substring auf Test-Pfaden ist. Mit `cargo test pdf_button` (Substring auf Modul-Namen) laufen alle 8/8 grün. Plan-Verify-Kommando entsprechend interpretiert.

## Known Stubs
Keine — der Endpoint aus Wave 2 ist voll funktional, der Frontend-Button ist komplett wired.

## Threat Flags
Keine neuen Surfaces — der Anchor ruft den bereits per Cookie-Auth gemitigateten Endpoint aus Plan 49-02 auf. STRIDE-Register aus PLAN.md unverändert:
- T-49-01 (Spoofing/Auth): mitigated durch Backend-Middleware `forbid_unauthenticated`
- T-49-INFO (Info Disclosure/Race): accepted; Backend 409-Response ist die einzige Race-Response-Surface

## User Setup Required
Keine.

## Next Phase Readiness
- Bereit für Wave 4 / Plan 49-05 (Verifikation): manueller Browser-Smoke-Test, Nachweis dass Download in Firefox+Chrome funktioniert, Dateiname wie erwartet, KW-Navigation Sichtbarkeits-Verhalten korrekt
- Kein Blocker; alle Automated-Gates (cargo build native + WASM, cargo test 795/795, cargo clippy -D warnings) grün

---
*Phase: 49-pdf-download-button*
*Plan: 04 — Frontend Button + i18n + Predicate Test*
*Completed: 2026-07-03*

## Self-Check: PASSED

Verifications:
- `shifty-dioxus/src/i18n/mod.rs` exists with `PdfDownload` variant (grep = 1)
- `shifty-dioxus/src/i18n/{de,en,cs}.rs` each contain exactly 1 `PdfDownload` reference
- `shifty-dioxus/src/page/shiftplan.rs` contains `pub fn should_show_pdf_button` definition + RSX callsite + 8 test functions (grep = 12 occurrences)
- Commits present in git log: `5de95e2`, `1167cd2`, `f9b2e9d`
- All build gates pass: `cargo build`, `cargo build --target wasm32-unknown-unknown`, `cargo test` (795/795), `cargo clippy -p shifty-dioxus -- -D warnings`
