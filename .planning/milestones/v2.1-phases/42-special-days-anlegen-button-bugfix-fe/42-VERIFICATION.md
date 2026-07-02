---
phase: 42-special-days-anlegen-button-bugfix-fe
verified: 2026-07-02T12:00:00Z
status: passed
score: 7/7 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification: false
---

# Phase 42: Special-Days-„Anlegen"-Button-Bugfix (FE) — Verification Report

**Phase Goal:** Nach dem Anlegen eines Special-Day bleibt der „Anlegen"-Button aktiv; mehrfaches Anlegen hintereinander ist ohne Dropdown-Toggle möglich.
**Verified:** 2026-07-02
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| #  | Truth                                                                                                       | Status     | Evidence                                                                                                                                         |
|----|-------------------------------------------------------------------------------------------------------------|------------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| 1  | D-42-01: die 3 Feld-Resets (sd_date_str, sd_type, sd_time_str) sind ENTFERNT aus dem Create-Success-Arm   | ✓ VERIFIED | git-diff b77394e bestätigt Entfernung; grep findet keine reset-Aufrufe im Success-Arm; retention policy via `special_day_form_after_create` ersetzt sie (settings.rs:596-599) |
| 2  | D-42-02: sd_year.set(iso_year) (WR-04) und sd_resource.restart() (Listen-Reload) bleiben erhalten          | ✓ VERIFIED | settings.rs:590, 600 — beide Zeilen vorhanden und unverändert                                                                                    |
| 3  | D-42-03: sd_is_duplicate ist informativ, NICHT an Btn.disabled gekoppelt                                    | ✓ VERIFIED | settings.rs:823: `disabled: !sd_form_valid \|\| *sd_saving.read()` — kein sd_is_duplicate; Hinweis nur als `span` bei Zeile 830                 |
| 4  | D-42-04: sd_save_result bleibt unveraendert, kein Auto-Clear beim Feld-Edit eingefuehrt                    | ✓ VERIFIED | git-diff b77394e berührt sd_save_result in Change-Handlern nicht; Pre-existing clear-on-change unverändert                                       |
| 5  | D-42-05 (HARD gate): is_special_day_form_valid und special_day_form_after_create als reine Funktionen unit-getestet | ✓ VERIFIED | settings.rs:103-135; 7 neue Tests (prefixed `special_day`); `cargo test -p shifty-dioxus special_day` → **10 passed, 0 failed**            |
| 6  | D-42-06 (best-effort): SSR-Test oder begruendeter Skip in VALIDATION.md                                    | ✓ VERIFIED | Fall B dokumentiert in 42-VALIDATION.md:66-94; `nyquist_compliant: true` gesetzt; Pure-fn-Test deckt das Button-enabled-Invariant ab           |
| 7  | Controlled-Select D-06/D-08 intakt: Typ-Feld bleibt nach Create GEFUELLT (kein Signal-DOM-Desync)          | ✓ VERIFIED | settings.rs:596-599: `retained = special_day_form_after_create(&sd_form_before)` → `sd_type.set(retained.ty)` belegt den Wert, nicht None; SelectInput nutzt `sd_type_to_select_value(sd_type_val.clone())` als controlled value (Zeile 785) |

**Score:** 7/7 truths verified

---

### Requirements Coverage

| Requirement | Plan    | Beschreibung                                        | Status      | Evidenz                                                        |
|-------------|---------|-----------------------------------------------------|-------------|----------------------------------------------------------------|
| SDF-01      | 42-01   | Anlegen-Button bleibt aktiv, mehrfaches Anlegen ohne Dropdown-Toggle | ✓ SATISFIED | REQUIREMENTS.md markiert SDF-01 als `[x]` complete; drei Feld-Resets entfernt, Retention-Policy implementiert, Tests gruen |

---

### Required Artifacts

| Artifact                                                      | Erwartet                                           | Status     | Details                                                              |
|---------------------------------------------------------------|----------------------------------------------------|------------|----------------------------------------------------------------------|
| `settings.rs::is_special_day_form_valid`                      | Pure `pub(crate)` fn, extrahiertes Validitäts-Prädikat | ✓ VERIFIED | settings.rs:103-111; in Render-Body verdrahtet (Zeile 517)           |
| `settings.rs::SpecialDayForm + special_day_form_after_create` | Pure struct + fn, Retention-Policy modelliert      | ✓ VERIFIED | settings.rs:117-135; load-bearing verdrahtet in Success-Arm (Zeile 535-599) |
| `settings.rs::#[cfg(test)]` Unit-Tests                        | 7 Tests mit Präfix `special_day`                   | ✓ VERIFIED | settings.rs:222-307; 10 Tests laufen durch (inkl. pre-existing), 0 failed |

---

### Key Link Verification

| Von                          | Zu                                    | Via                                             | Status     | Details                                                                   |
|------------------------------|---------------------------------------|-------------------------------------------------|------------|---------------------------------------------------------------------------|
| settings.rs Render-Body:517  | `is_special_day_form_valid`           | direkter Aufruf ersetzt Inline-Prädikat          | ✓ WIRED    | `sd_form_valid = is_special_day_form_valid(...)` — gleiche Semantik       |
| settings.rs Success-Arm:596  | `special_day_form_after_create`       | `let retained = special_day_form_after_create(&sd_form_before)` | ✓ WIRED | load-bearing, kein Dead-Code                                              |
| Btn:823 disabled             | `sd_form_valid` (nicht `sd_is_duplicate`) | `!sd_form_valid \|\| *sd_saving.read()`          | ✓ WIRED    | D-42-03 bestätigt: sd_is_duplicate ist NOT gekoppelt an disabled          |

---

### Gate-Ergebnisse (Behavioral Spot-Checks)

| Verhalten                                             | Befehl                                                          | Ergebnis              | Status  |
|-------------------------------------------------------|-----------------------------------------------------------------|-----------------------|---------|
| special_day-Tests gruen                               | `cargo test -p shifty-dioxus special_day`                       | 10 passed, 0 failed   | ✓ PASS  |
| WASM-Build warnungsfrei (HYG-01)                      | `nix develop -c cargo build --target wasm32-unknown-unknown`    | Finished 54.74s, 0 warnings/errors | ✓ PASS |
| Backend-Clippy clean (Backend unberuehrt)             | `nix develop -c cargo clippy --workspace -- -D warnings`        | Finished, 0 output    | ✓ PASS  |
| TDD RED→GREEN Commits vorhanden                       | `git log --oneline b0fed7a e144847`                             | test(42-01) + feat(42-01) | ✓ PASS |

---

### Anti-Patterns

| Datei          | Zeile | Pattern               | Schwere    | Auswirkung                                                                                                |
|----------------|-------|-----------------------|------------|-----------------------------------------------------------------------------------------------------------|
| settings.rs    | 67-70 | Veralteter Doc-Kommentar (`sd_type.set(None)` referenziert altes Verhalten) | ⚠ Warning | Irreführend, aber kein funktionaler Defekt — `sd_type_to_select_value` arbeitet korrekt mit retained.ty; Phase 42 hat nur WR-02 im Success-Arm aktualisiert, nicht diesen Doc-Block |
| settings.rs    | 782-784 | Veralteter Inline-Kommentar (SelectInput, gleiche Referenz zu set(None)) | ⚠ Warning | Gleiche Einstufung wie oben; pre-existing aus Phase 33/36; kein TBD/FIXME/XXX → kein BLOCKER |

Kein `TBD`/`FIXME`/`XXX`-Marker in settings.rs gefunden. Keine Stub-Muster. Beide Stale-Kommentare sind pre-existing und wurden nicht von Phase 42 eingeführt.

---

### Scope-Check

| Bereich              | Erwartung      | Befund                                                   |
|----------------------|----------------|----------------------------------------------------------|
| Backend (service/dao/rest) | Unberuehrt | Keine Änderungen in diesen Crates; Clippy clean          |
| Snapshot-Bump        | Kein           | CURRENT_SNAPSHOT_SCHEMA_VERSION nicht geändert           |
| Migrationen          | Keine          | Kein neuer Eintrag in migrations/                        |
| Neue Dependencies    | Keine          | Cargo.toml unverändert                                   |
| i18n                 | Unveraendert   | Keine neuen Keys in en/de/cs.rs                          |
| Bekannter pre-existing Fehler | i18n_impersonation_keys_match_german_reference (Phase 37-02) | In vollem FE-Lauf vorhanden, nicht Phase 42 zuzuschreiben |

---

### Manuell (optional, nicht blockierend)

Live-Browser-Smoke: Tag A anlegen → Button bleibt aktiv, Felder gefüllt, „Gespeichert" sichtbar, Duplikat-Hinweis für A → Datum auf Tag B ändern → Hinweis verschwindet, Button aktiv → Tag B anlegen ohne Dropdown-Toggle. Nicht automatisiert testbar (D-25-06-Klasse: WASM Signal↔DOM).

---

_Verified: 2026-07-02_
_Verifier: Claude (gsd-verifier)_
