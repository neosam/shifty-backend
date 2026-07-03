---
phase: 45-shifty-dioxus-warnings-aufraeumen
plan: 01
subsystem: shifty-dioxus (frontend)
tags: [hygiene, clippy, warnings, dioxus, HYG-03]
status: complete
requires:
  - phase 38 (HYG-01/02) — backend clippy gate green + FE build warning-free baseline
provides:
  - FE clippy `-D warnings` gate scharfgestellt (HYG-03 complete)
  - shifty-dioxus warning-baseline auf 0 gesenkt
affects:
  - shifty-dioxus/src (mechanische Aufräumung, keine Logik-Änderung)
tech-stack:
  added: []
  patterns: []
key-files:
  created: []
  modified:
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/component/add_extra_hours_form.rs
    - shifty-dioxus/src/component/contract_modal.rs
    - shifty-dioxus/src/component/dialog.rs
    - shifty-dioxus/src/component/dropdown_base.rs
    - shifty-dioxus/src/component/employee_view.rs
    - shifty-dioxus/src/component/tooltip.rs
    - shifty-dioxus/src/component/top_bar.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/page/absences.rs
    - shifty-dioxus/src/page/custom_extra_hours_management.rs
    - shifty-dioxus/src/page/shiftplan.rs
    - shifty-dioxus/src/page/user_details.rs
    - shifty-dioxus/src/page/user_management.rs
    - shifty-dioxus/src/service/feature_flag.rs
    - shifty-dioxus/src/service/user_management.rs
    - shifty-dioxus/src/state/employee_work_details.rs
    - shifty-dioxus/src/state/shiftplan.rs
    - shifty-dioxus/src/tests/error_tests.rs
    - shifty-dioxus/src/tests/integration_tests.rs
    - shifty-dioxus/src/tests/week_tests.rs
    - "+ ~40 weitere Files auto-fix-berührt (Import-/Deref-/Unit-/Clone-Cleanup)"
decisions:
  - "D-45-01: Zwei-Wellen-Vorgehen — cargo fix + clippy --fix (auto) zuerst, dann manuelle Kategorien. Wie Phase 38 (HYG-01/02)."
  - "D-45-02: `#[allow]`s bei API-brechenden oder lokalitäts-erhaltenden Fällen (type_complexity, wrong_self_convention, enum_variant_names, module_inception in Tests) — jeder mit `// reason:`-Kommentar auf der Zeile darüber."
  - "D-45-03: `assertions_on_constants` in error_tests.rs durch Entfernen des `assert!(true)`-No-Ops behoben, nicht durch `#[allow]` — der Match-Arm allein beweist die Case-Erreichbarkeit."
metrics:
  duration: "20min"
  completed: "2026-07-02"
---

# Phase 45 Plan 01: shifty-dioxus Warnings-Aufräumen Summary

**One-liner:** shifty-dioxus Clippy-Baseline von 177 → 0 Warnings gebracht (auto-fix erste Welle, dann manuelle Kategorien mit begründeten `#[allow]`s bei API-brechenden Fällen); FE-Clippy-Gate `-D warnings` erstmals scharfgestellt.

## Baseline & Progress

| Zeitpunkt | Warning-Anzahl | Verbleibende Kategorien |
| --------- | -------------- | ----------------------- |
| Baseline (vor Task 1) | **177** | 30+ Kategorien, siehe Kategorie-Verteilung unten |
| Nach Auto-fix (Task 1) | **29** | 12 Kategorien (siehe unten) |
| Nach Manual Fixes (Task 2) | **0** | — |

## Baseline-Kategorie-Verteilung (Top 15)

| Anzahl | Lint | Fix-Modus |
| ------ | ---- | --------- |
| 37 | `clippy::clone_on_copy` | auto-fix |
| 30 | `clippy::useless_conversion` | auto-fix |
| 25 | `clippy::redundant_closure` | auto-fix |
| 17 | `clippy::unused_unit` | auto-fix |
| 5  | `clippy::explicit_auto_deref` | auto-fix |
| 4  | `clippy::type_complexity` | `#[allow]` mit reason |
| 4  | `clippy::needless_borrow` | auto-fix |
| 4  | `clippy::doc_lazy_continuation` | manual (Doc-Indent) |
| 4  | `clippy::collapsible_match` | manual |
| 3  | `clippy::redundant_pattern_matching` | manual (`is_err`/`is_ok`) |
| 3  | `clippy::module_inception` | `#[allow]` mit reason |
| 3  | `clippy::manual_range_contains` | auto-fix |
| 3  | `clippy::expect_fun_call` | auto-fix |
| 3  | `clippy::assertions_on_constants` | manual (No-op entfernt) |
| 2  | `unused_mut`, `unused_imports`, `clippy::wrong_self_convention` u.a. | mix |

## Task 2: Manuelle Kategorien

| Kategorie | Anzahl | Behandlung |
| --------- | ------ | ---------- |
| `clippy::redundant_pattern_matching` | 3 | Umformung zu `.is_err()` / `.is_ok()` |
| `redundant_redefinition` (`let x = x;`) | 4 | Entfernt (top_bar.rs x2, user_management.rs x2) |
| `clippy::manual_strip` | 1 | `.starts_with()` + Slice → `.strip_prefix()` |
| `clippy::if_same_then_else` | 1 | Duplicate branches kollabiert (contract_modal cancel_label) |
| `clippy::cloned_ref_to_slice_refs` | 1 | `&[x.clone()]` → `std::slice::from_ref(&x)` |
| `clippy::collapsible_match` | 1 | Nested `if let` in `if let Ok(Some(..))` fusioniert |
| `clippy::assertions_on_constants` | 3 | `assert!(true)` No-Ops entfernt |
| `clippy::doc_lazy_continuation` | 4 | Doc-Continuation-Lines eingerückt (dialog.rs backdrop_invariant) |
| `unused_imports` | 1 | `use super::*;` entfernt (feature_flag tests) |
| `clippy::type_complexity` | 4 | `#[allow]` + `// reason:` |
| `clippy::wrong_self_convention` | 2 | `#[allow]` + `// reason:` (from_hour, from_as_calendar_week) |
| `clippy::enum_variant_names` | 1 | `#[allow]` + `// reason:` (CustomExtraHoursManagementAction) |
| `clippy::module_inception` | 3 | `#[allow]` + `// reason:` (tests + i18n) |

## `#[allow]`-Registry (Phase-45 additions)

| File:Line | Lint | Reason |
| --------- | ---- | ------ |
| `src/component/contract_modal.rs:231` | `clippy::type_complexity` | 4-tuple encodes (checked, label, disabled, setter) for each weekday pill; factoring into a type alias would obscure the local-only shape |
| `src/component/employee_view.rs:849` | `clippy::type_complexity` | 3-tuple encodes (label, hint, predicate) per extra-hours category; type alias would obscure the local-only structure |
| `src/component/top_bar.rs:162` | `clippy::type_complexity` | return partitions the same tuple shape used in nav-item routing; extracting a type alias would obscure the D-10 contract locality |
| `src/service/user_management.rs:669` | `clippy::type_complexity` | test-local fixture typed for readability; extracting alias would spread noise |
| `src/state/employee_work_details.rs:106` | `clippy::wrong_self_convention` | name mirrors the `from`/`to` field naming; rename would break UI callsites without semantic benefit |
| `src/state/shiftplan.rs:190` | `clippy::wrong_self_convention` | name mirrors the `from`/`to` field naming; rename would ripple through all shiftplan callsites without semantic benefit |
| `src/page/custom_extra_hours_management.rs:18` | `clippy::enum_variant_names` | variants use the CustomExtraHours domain name; renaming would lose the domain qualifier used in the coroutine handler match |
| `src/tests/integration_tests.rs:2` | `clippy::module_inception` | intentional test-module organization — grouping tag matches file name for readability |
| `src/tests/week_tests.rs:2` | `clippy::module_inception` | intentional test-module organization — grouping tag matches file name for readability |
| `src/i18n/mod.rs:5` | `clippy::module_inception` | i18n module holds the I18n type; historic naming re-exported below via pub use |

Alle Phase-45-Allows tragen einen `// reason:`-Kommentar auf der Zeile direkt darüber (Plan-Spec: "in derselben oder darüberliegenden Zeile").

**Nicht-modifiziert:** pre-existing `#[allow(dead_code)]`/`#[allow(non_snake_case)]` ohne reason-Kommentar sind Bestandsaufnahme aus früheren Phasen und out-of-scope für HYG-03 (das Gate zielt auf neue Regressions, nicht historische Allows).

## Gate-Ergebnisse (alle 5 grün)

| Gate | Kommando | Ergebnis |
| ---- | -------- | -------- |
| 1 — FE build 0 warnings | `cargo build -p shifty-dioxus` | ✅ 0 warnings (aus Backend-Root-Shell) |
| 2 — FE clippy `-D warnings` | `cargo clippy -p shifty-dioxus --workspace --tests -- -D warnings` | ✅ exit 0 |
| 3 — WASM build | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` | ✅ exit 0 |
| 4 — Backend clippy | `cargo clippy --workspace -- -D warnings` | ✅ exit 0 (unverändert grün) |
| 5 — Backend tests | `cargo test --workspace` | ✅ alle Doc-Tests + Unit-Tests grün |

**Zusatz — shifty-dioxus tests:** 777 passed, 1 failed. Der eine Failure ist `i18n::tests::i18n_impersonation_keys_match_german_reference` — pre-existing Mismatch zwischen deutschem Reference-String "🥸 Agieren" und dem Test-Erwartungswert "Als diese Person agieren". Ausdrücklich als "out-of-scope für Phase 45" markiert im Executor-Prompt und in `.planning/todos/pending/2026-07-02-i18n-impersonation-key-test-mismatch.md` getrackt.

## Deviations from Plan

**None** — der Plan wurde exakt wie geschrieben ausgeführt:
- Task 1 (Auto-fix + WASM-Sanity): 177 → 29 Warnings, WASM grün.
- Task 2 (Manuelle Kategorien): 29 → 0 Warnings, WASM grün.
- Task 3 (Alle 5 Gates + `#[allow]`-Registry): alle 5 Gates grün, alle neuen Allows reason-getaggt.

**Nicht verändert (per Plan-Contract):**
- `shifty-dioxus/Cargo.toml` — kein Diff.
- `rest-types/` — kein Diff.
- REST-Handler / DAOs / Migrations — kein Diff.
- i18n-Keys / Übersetzungen — kein Diff.
- Business-Logik in irgendeinem File — kein Diff.

Kommit-Struktur folgt der GSD-Auto-Commit-Konvention: Executor bereitet Fixes vor, GSD-Orchestrator committet die Änderungen samt SUMMARY.md.

## Self-Check: PASSED

- SUMMARY.md exists at `.planning/phases/45-shifty-dioxus-warnings-aufraeumen/45-01-SUMMARY.md`.
- All 5 phase gates green (Gate 1: 0 warnings; Gate 2: exit 0; Gate 3: exit 0; Gate 4: exit 0; Gate 5: exit 0).
- shifty-dioxus test-suite 777/778 grün (der eine bekannte Failure ist out-of-scope, in Todos getrackt).
- No Cargo.toml / rest-types / REST-handler / migration diffs.
- Every new `#[allow]` in shifty-dioxus/src carries a `// reason:` comment on the preceding line.
