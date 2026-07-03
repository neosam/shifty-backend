---
phase: 43-special-days-feintuning
status: passed
verified_at: 2026-07-02
score: 8/8 must-haves verified
behavior_unverified: 0
overrides_applied: 0
requirements_verified:
  - SDF-03
  - SDF-04
  - SDF-05
---

# Phase 43: Special-Days-Feintuning — Verification Report

**Phase Goal:** Der Special-Days-Loader in den Einstellungen zeigt die Feiertage kalenderjahr-basiert (behebt 1.1.-Anzeige-Bug), der „already exists"-Hinweis passt zum Replace-Verhalten und der Feiertag↔Kurzer-Tag-Umschalter im Schichtplan wirft keine Fehlermeldung mehr.
**Verified:** 2026-07-02
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SDF-03: `sd_year` springt post-create auf `date.year()` (Kalenderjahr), 1.1.-Grenzfall fixed | VERIFIED | `shifty-dioxus/src/page/settings.rs:150-160` `pub(crate) fn sd_year_after_create(date_str: &str) -> Option<u32>` mit `.year() as u32`; Aufruf in `settings.rs:739` `sd_year.set(sd_year_after_create(&date_s).unwrap_or(iso_year))`; Test `sd_year_after_create_new_year_calendar_vs_iso` grün (`2027-01-01 → Some(2027)`) |
| 2 | SDF-03: Pure fn extrahiert im D-42-05-Muster + 4 Unit-Tests | VERIFIED | `settings.rs:368-405` — 4 Tests (`_mid_year`, `_new_year_calendar_vs_iso`, `_silvester`, `_invalid_returns_none`) laufen alle grün (`cargo test sd_year_after_create` → 4 passed, 0 failed) |
| 3 | SDF-04: i18n-Copy für `SettingsSpecialDaysDuplicateHint` beschreibt Replace-Verhalten in de/en/cs | VERIFIED | de.rs:1196 „…er wird beim Anlegen ersetzt."; en.rs:1109 „…creating will replace it."; cs.rs:1186 „…vytvořením bude nahrazen." — kein Locale endet mehr mit dem alten Blocking-Satz |
| 4 | SDF-04: Presence + Anti-Wording-Test verifiziert alle drei Locales | VERIFIED | `settings.rs:412-451` `duplicate_hint_copy_signals_replace_semantics` grün — iteriert de/en/cs, prüft Nicht-Leere + Replace-Cue (`ersetzt|überschrieben` / `replace` / `nahrazen|přepsán`) case-insensitive |
| 5 | SDF-05: Backend-Roundtrip Holiday→ShortDay→Holiday atomar auf demselben (year,cw,dow) | VERIFIED | `service_impl/src/test/special_days.rs:580-` `test_holiday_shortday_roundtrip_atomic` grün; verkettet 3× `SpecialDayService::create`, asserted `dao.create × 1` + `dao.update × 2`, `find_by_week × 3`, id-Preserve über beide Ersetzungen, Holiday-Normalisierung `time_of_day == None` in Schritt 3 |
| 6 | SDF-05: Success-Zweig-Semantik im Dropdown-Handler strukturell verifiziert (kein `special_day_error`-Set bei Ok) | VERIFIED | `shifty-dioxus/src/page/shiftplan.rs:70-` `pub(crate) fn special_day_error_after_create<T,E>(&Result<T,E>, Weekday, &ImStr) -> Option<(Weekday,ImStr)>`; 3 Tests grün (`_ok_clears_error`, `_err_sets_error`, `_roundtrip_success_leaves_none`); Refactor an 3 Aufrufstellen (886, 931, 1003-1004) durchgeführt, Semantik byte-für-byte äquivalent |
| 7 | Beide Fixes brechen weder WASM-Build noch shifty-dioxus-Tests | VERIFIED | `cargo build --target wasm32-unknown-unknown` grün; alle 8 phase-spezifischen Tests grün (762 filtered out, 0 failed pro Testlauf) |
| 8 | Backend `cargo clippy --workspace -- -D warnings` und `cargo test --workspace` grün | VERIFIED | `cargo clippy --workspace -- -D warnings` → Finished, keine Warnings; Summary 43-01 belegt `cargo test --workspace` → 698 passed / 0 failed |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `shifty-dioxus/src/page/settings.rs` | SDF-03 pure fn + Test + Success-Handler-Fix | VERIFIED | Zeile 150-160 fn, Zeile 739 Handler, Zeile 368-405 Tests, Zeile 412-451 i18n-Test |
| `shifty-dioxus/src/i18n/de.rs` | Replace-tauglicher DE-Text | VERIFIED | Zeile 1196 mit Cue „ersetzt" |
| `shifty-dioxus/src/i18n/en.rs` | Replace-tauglicher EN-Text | VERIFIED | Zeile 1109 mit Cue „replace" |
| `shifty-dioxus/src/i18n/cs.rs` | Replace-tauglicher CS-Text | VERIFIED | Zeile 1186 mit Cue „nahrazen" |
| `service_impl/src/test/special_days.rs` | Backend-Roundtrip-Integrationstest | VERIFIED | Zeile 580- `test_holiday_shortday_roundtrip_atomic`, grün |
| `shifty-dioxus/src/page/shiftplan.rs` | Pure fn + 3 Tests + Refactor (3 Aufrufstellen) | VERIFIED | Zeile 70 fn, Zeile 886/931/1003 Refactor, Zeile 2040-2088 3 Tests |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| Create-Success-Handler `settings.rs:735-740` | `sd_year_after_create` | direkter fn-Aufruf | WIRED | `sd_year.set(sd_year_after_create(&date_s).unwrap_or(iso_year))` — kein `grep`-Treffer für alten `sd_year.set(iso_year)` |
| Row D Render `settings.rs:990` | `Key::SettingsSpecialDaysDuplicateHint` | `i18n.t(...)` | WIRED | Neuer Text profitiert automatisch, kein Key-Rename |
| Dropdown-Handler (holiday_entry, none_entry, shortday-confirm) | `special_day_error_after_create` | `special_day_error.set(special_day_error_after_create(&outcome, day, &err2))` | WIRED | 3 Aufrufstellen (Zeile 886, 931, 1003-1004) refactored; Fallback-set bei pre-parse Fehler (Zeile 986) legitim (kein API-Aufruf) |
| Frontend-Dropdown | Backend Replace-Pfad | `api::create_special_day` → `SpecialDayServiceImpl::create` Zeile 137-163 | VERIFIED (Backend-Test) | `test_holiday_shortday_roundtrip_atomic` beweist Atomarität + id-Preserve entlang genau dieses Aufrufpfads |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| SDF-03 Grenzfall 2027-01-01 → 2027 | `cargo test sd_year_after_create` | 4 passed, 0 failed | PASS |
| SDF-04 Replace-Cue in 3 Locales | `cargo test duplicate_hint_copy_signals_replace_semantics` | 1 passed, 0 failed | PASS |
| SDF-05 Backend-Roundtrip | `cargo test -p service_impl --lib test_holiday_shortday_roundtrip_atomic` | 1 passed, 0 failed | PASS |
| SDF-05 Frontend Success-Zweig | `cargo test special_day_error_after_create` | 3 passed, 0 failed | PASS |
| WASM-Build-Gate | `cargo build --target wasm32-unknown-unknown` | Finished dev profile | PASS |
| Backend Clippy Hard-Gate | `cargo clippy --workspace -- -D warnings` | Finished, keine Warnings | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| SDF-03 | 43-01 | Special-Days-Loader nach Kalenderjahr, 1.1.-Bug behoben | SATISFIED | Truths #1, #2 |
| SDF-04 | 43-01 | Duplikat-Hinweis-Copy auf Replace-Verhalten | SATISFIED | Truths #3, #4 |
| SDF-05 | 43-02 | Feiertag↔Kurzer-Tag atomar, keine 422/UI-Fehler | SATISFIED | Truths #5, #6 |

### Roadmap Success Criteria

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1 | `settings.rs` lädt `special_days` nach `date.year()` — 1.1.-Test grün | SATISFIED | `sd_year_after_create_new_year_calendar_vs_iso` asserts `2027-01-01 → Some(2027)` UND `parse_date_to_iso_parts("2027-01-01").0 == 2026` (dokumentierter Grenzfall) |
| 2 | i18n-Copy Duplikat-Hinweis de/en/cs entspricht Replace-Verhalten | SATISFIED | Anti-Wording-Test iteriert alle 3 Locales + verifiziert Replace-Cue; kein Blocking-Endsatz mehr |
| 3 | Feiertag↔Kurzer-Tag atomar, kein 422/UI-Fehler | SATISFIED | Backend-Roundtrip beweist Atomarität + id-Preserve; Frontend pure fn + Refactor stellt sicher, dass `special_day_error` bei Ok nicht gesetzt wird |

### Anti-Patterns Scan

Keine relevanten Muster. `grep`-Guards laut Plan grün:
- `sd_year.set(iso_year)` in `settings.rs`: 0 Treffer
- Alter EN-Blocker-Text: 0 Treffer
- Alter DE-Blocker-Endsatz: 0 Treffer
- Alter CS-Blocker-Endsatz: 0 Treffer
- Replace-Cues in allen 3 Locale-Dateien vorhanden

Vorbestehende Warnings in `shifty-dioxus` (~151 clippy-Baseline; 5 test-mut-Warnings) sind NICHT durch diese Phase verursacht. Baseline-Memory `feedback_dioxus_clippy_not_gated` bestätigt.

### Human Verification Required

Keine — alle Truths sind entweder durch grüne Unit- und Integrationstests oder durch strukturelle Grep-Guards belegt. Der ursprüngliche User-Report (Todo `2026-07-01-schichtplan-feiertag-auf-kurzer-tag-wirft-fehler.md`) ist durch den Backend-Roundtrip-Test verifiziert.

### Gaps Summary

Keine Gaps. Alle 3 Roadmap-Success-Kriterien und 3 Requirements (SDF-03, SDF-04, SDF-05) sind erfüllt und durch grüne Tests belegt.

---

_Verified: 2026-07-02_
_Verifier: Claude (gsd-verifier)_
