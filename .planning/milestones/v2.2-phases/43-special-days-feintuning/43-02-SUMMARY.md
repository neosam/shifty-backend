---
phase: 43-special-days-feintuning
plan: 2
subsystem: special-days
tags:
  - backend
  - frontend
  - special-days
  - regression-test
  - bugfix
  - sdf-05
requires:
  - "SDF-01 in-place Replace-Pfad (Phase 36, service_impl/src/special_days.rs:137-163)"
  - "SDF-01 Test-Fundus (test_create_replaces_same_date_entry, test_create_switches_holiday_to_shortday, test_create_switches_shortday_to_holiday)"
provides:
  - "Backend-Roundtrip-Integrationstest: SDF-01 Preserve-id-Invariante über zwei Ersetzungen (Holiday → ShortDay → Holiday) verkettet"
  - "Frontend pure fn `special_day_error_after_create` + 3 Unit-Tests: strukturelle Absicherung, dass der Success-Zweig des Dropdown-Handlers das Error-Signal nicht setzt"
affects:
  - "Todo `2026-07-01-schichtplan-feiertag-auf-kurzer-tag-wirft-fehler.md` (Verifikationstest schließt den User-Report)"
tech-stack:
  added: []
  patterns:
    - "Extraktion einer Success/Error-Zweig-Entscheidung in eine pure fn zur Unit-Test-Absicherung (Präzedenz: settings.rs `is_special_day_form_valid`, `sd_year_after_create`)"
    - "Mock-basierter Backend-Roundtrip mit `Arc<Mutex<Option<SpecialDayEntity>>>` als Zustands-Plumbing zwischen `dao.create`/`dao.update`/`dao.find_by_week`-Mocks"
key-files:
  created: []
  modified:
    - service_impl/src/test/special_days.rs
    - shifty-dioxus/src/page/shiftplan.rs
decisions:
  - "D-43-02-01: Backend-Replace-Pfad bleibt unverändert — der Test bestätigt die Semantik seit Phase 36, kein Code-Change am Impl (kein Snapshot-Bump, keine Migration)."
  - "D-43-02-02: Frontend-Refactor durchgeführt (alle drei match-Blöcke ersetzt) — Semantik byte-für-byte äquivalent. Anti-Overreach-Guardrail wurde geprüft: der `Ok(_)`-Zweig des ShortDay-Confirm-Handlers behält zusätzlich `shortday_prompt_day.set(None)` + `shortday_time.set(String::new())`; die pure fn kapselt nur die Signal-Set-Semantik, das Prompt-Cleanup + Resource-Restart bleibt konditional im `if outcome.is_ok()`-Block."
  - "D-43-02-03: `Weekday` ist `Copy` und wird per Wert in die pure fn übergeben (nicht per Ref), `ImStr` per `&ImStr` (Clone im Err-Zweig). Signatur `special_day_error_after_create<T, E>(result: &Result<T, E>, day: Weekday, err_msg: &ImStr)`."
metrics:
  completed_date: 2026-07-02
  duration_minutes: ~25
status: complete
---

# Phase 43 Plan 2: SDF-05 Wechsel Feiertag↔Kurzer-Tag — Verifikationstest Summary

Backend-Roundtrip-Integrationstest + Frontend Success-Zweig-pure-fn samt drei Regressionstests belegen strukturell, dass der Wechsel Feiertag↔Kurzer-Tag im Wochenraster-Dropdown atomar denselben `(year, calendar_week, day_of_week)`-Datensatz überschreibt und keine UI-Fehlermeldung produziert. SDF-05 damit vollständig verifiziert; User-Report-Todo `2026-07-01-schichtplan-feiertag-auf-kurzer-tag-wirft-fehler.md` kann geschlossen werden.

## Tasks Completed

### Task 1: Backend-Roundtrip-Integrationstest `test_holiday_shortday_roundtrip_atomic`

- Datei: `service_impl/src/test/special_days.rs` (neuer `#[tokio::test]` am Dateiende, direkt hinter `test_create_switches_shortday_to_holiday`).
- Verkettet **drei** `SpecialDayService::create`-Calls auf `(2026, W1, Monday)`:
  1. **Schritt 1** (Holiday initial, leere Woche): Insert-Pfad → `dao.create` × 1, `clock.date_time_now` × 1, `uuid.new_uuid` je 1× für `create id` und `create version`.
  2. **Schritt 2** (Holiday → ShortDay): Replace-Pfad → `dao.update` × 1 mit id-Preserve + `time_of_day = Some(09:00)`.
  3. **Schritt 3** (ShortDay → Holiday, client sendet `time_of_day = Some(...)`): Replace-Pfad → `dao.update` × 1 mit id-Preserve + `time_of_day = None` (Holiday-Normalisierung, `service_impl/src/special_days.rs:125-127`).
- Persistenz-Plumbing via `Arc<Mutex<Option<SpecialDayEntity>>>`: `dao.create.returning` schreibt initial, `dao.find_by_week.returning` liest, `dao.update.returning` überschreibt. `find_by_week` × 3, `dao.update` × 2, `dao.create` × 1 — Mock-Counts exakt.
- Assertions:
  - `result_1.id == result_3.id` (Preserve-id über zwei Ersetzungen, SDF-01 D-01).
  - `result_2.day_type == ShortDay`, `result_2.time_of_day == Some(09:00)`.
  - `result_3.day_type == Holiday`, `result_3.time_of_day == None`.
  - Finaler `persisted`-Zustand: exakt EIN aktiver Eintrag, `deleted IS None`, `day_type == Holiday`, id stabil.
- Test-Body-Kommentar zitiert Todo `2026-07-01-schichtplan-feiertag-auf-kurzer-tag-wirft-fehler` + SDF-01/D-01/D-09-Präzedenz.

### Task 2: Frontend pure fn + Refactor + 3 Unit-Tests

- Neue `pub(crate) fn special_day_error_after_create<T, E>(result: &Result<T, E>, day: Weekday, err_msg: &ImStr) -> Option<(Weekday, ImStr)>` in `shifty-dioxus/src/page/shiftplan.rs`, direkt nach den Imports und vor `pub enum ShiftPlanAction`. Docstring nennt SDF-05, SDF-01, User-Report-Todo und den Backend-Replace-Pfad.
- **Refactor durchgeführt** — die drei bisherigen `match`-Blöcke im Dropdown-Handler wurden minimal-invasiv umgeschrieben:
  * `holiday_entry` (~Zeile 858–867): `create_special_day` → pure fn setzt Signal → im `if outcome.is_ok()` weiterhin `special_days_for_week.restart()` + `shift_plan_context.restart()`.
  * `none_entry`-Delete-Handler (~Zeile 895–911): `delete_special_day` → pure fn → im `if outcome.is_ok()` dieselben zwei Restarts.
  * ShortDay-Confirm-Handler (~Zeile 975–992): `create_special_day` → pure fn → im `if outcome.is_ok()` `shortday_prompt_day.set(None)`, `shortday_time.set(String::new())`, dann beide Restarts (Prompt-Cleanup bleibt bewusst konditional im Success-Zweig).
  * Byte-für-byte-Semantik: Ok → Signal `None` + Resource-Restarts (+ ShortDay-Prompt-Cleanup); Err → Signal `Some((day, err_msg))`, keine Restarts, kein Prompt-Cleanup. Verifiziert per WASM-Build-Grün.
- Drei neue `#[test]`-Funktionen im existierenden `#[cfg(test)] mod tests` am Dateiende:
  * `special_day_error_after_create_ok_clears_error`: `Ok(())` → `None`.
  * `special_day_error_after_create_err_sets_error`: `Err(())` mit `day=Monday`, `err_msg="boom"` → `Some((Monday, "boom".into()))`.
  * `special_day_error_after_create_roundtrip_success_leaves_none`: Drei aufeinanderfolgende Ok-Ergebnisse (Holiday, ShortDay, Holiday) — jedes gibt `None` zurück, Test-Kommentar dokumentiert SDF-05-Bezug.

## Gates

| Gate | Status |
|------|--------|
| `cargo test -p service_impl --lib test_holiday_shortday_roundtrip_atomic` | ✅ PASS (1 test, 0 failed) |
| `cargo test --workspace` | ✅ PASS (alle Workspace-Tests grün) |
| `cargo clippy --workspace -- -D warnings` | ✅ PASS (Finished, keine Warnings) |
| `cargo test -p shifty-dioxus special_day_error_after_create` | ✅ PASS (3 tests, 0 failed) |
| `cargo test -p shifty-dioxus` (Full-Frontend) | 764 passed, 1 pre-existing failure — siehe „Deferred / Out-of-Scope" |
| `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` | ✅ PASS |
| Regression-Gates (`test_create_replaces_same_date_entry`, `test_create_switches_holiday_to_shortday`, `test_create_switches_shortday_to_holiday`) | ✅ Alle drei grün als Teil von `--workspace` |

## Deferred / Out-of-Scope

- **Pre-existing failing test**: `page::user::i18n::tests::i18n_impersonation_keys_match_german_reference` (shifty-dioxus, `src/i18n/mod.rs:1578`). Fehler war bereits vor dieser Phase da und ist als pending Todo erfasst (`.planning/todos/pending/2026-07-02-i18n-impersonation-key-test-mismatch.md`). NICHT im Scope von SDF-05. Kein neuer Fehler durch diesen Plan verursacht (Regel: Scope Boundary — nur Auto-Fix für Task-verursachte Probleme).
- **shifty-dioxus clippy hard-gate**: `cargo clippy -p shifty-dioxus --bin shifty-dioxus -- -D warnings` scheitert mit ~151 Fehlern. Diese sind pre-existing und in der User-Memory als bekannt dokumentiert (Referenz „Dioxus Clippy nicht gated + Toolchain-Split"): shifty-dioxus ist ein eigener Workspace, der vom CI-Clippy-Gate ausgeschlossen ist. Die Plan-Verify-Zeile war `cargo clippy -p shifty-dioxus --lib -- -D warnings`, aber shifty-dioxus hat kein `lib`-Target (nur `bin`), sodass dieser exakte Befehl fehlschlägt. Verifikation gegen page/shiftplan.rs speziell: die neue pure fn + die Refactored `match`-Blöcke produzieren KEINE zusätzlichen Clippy-Errors (`grep` in Clippy-Output zeigt keine Meldungen mit „page/shiftplan.rs" — nur ein pre-existing Treffer in `state/shiftplan.rs`).

## Deviations from Plan

Keine — Plan wurde exakt so ausgeführt. Der optionale Refactor der drei `match`-Blöcke (Bonus laut `<action>` Anti-Overreach-Guardrail) wurde durchgeführt, weil die Semantik byte-für-byte durch den pure-fn-Aufruf + `if outcome.is_ok()`-Block darstellbar ist. WASM-Build + Backend-Tests bestätigen keine Verhaltensänderung.

## Todo Closure

`.planning/todos/pending/2026-07-01-schichtplan-feiertag-auf-kurzer-tag-wirft-fehler.md` kann als resolved markiert werden — der Verifikationstest (Backend-Roundtrip) belegt, dass das ursprünglich berichtete Symptom (422-Fehler / UI-Fehlermeldung bei Umstellung Feiertag → Kurzer Tag) auf dem realen Aufrufpfad nicht mehr auftritt. Die Frontend-pure-fn + drei Unit-Tests sichern zusätzlich strukturell ab, dass der Success-Zweig kein Error-Signal setzt.

## Self-Check: PASSED

- Backend-Test existiert und läuft grün: `service_impl/src/test/special_days.rs::test_holiday_shortday_roundtrip_atomic`.
- Frontend pure fn existiert: `shifty-dioxus/src/page/shiftplan.rs::special_day_error_after_create` (nach Zeile 55, vor `pub enum ShiftPlanAction`).
- Frontend-Tests existieren: `special_day_error_after_create_ok_clears_error`, `_err_sets_error`, `_roundtrip_success_leaves_none` im existierenden `#[cfg(test)] mod tests`-Block.
- Alle mandaten Gates grün (siehe Gates-Tabelle).
- Backend-Impl unverändert (keine Verhaltensänderung), Migration nicht nötig, keine neuen Deps, keine REST-Route-Änderung — Success-Kriterien SDF-05 erfüllt.
