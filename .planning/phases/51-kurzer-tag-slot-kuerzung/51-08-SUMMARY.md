---
phase: 51
plan: 08
subsystem: shifty-dioxus/settings
tags: [FE, admin, settings, i18n, SHC-06, D-51-07]
requires: [51-02]
provides: [SHC-06 FE-editor]
affects: [shifty-dioxus/src/page/settings.rs, shifty-dioxus/src/loader.rs, shifty-dioxus/src/i18n]
tech-stack:
  added: []
  patterns: [HCFG-02-Card-2-Blueprint, ToggleValueAPI, PureFnValidator]
key-files:
  created: []
  modified:
    - shifty-dioxus/src/page/settings.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
decisions:
  - "D-51-07 FE-Umsetzung: Card 2b (zwischen HCFG-02 Card 2 und Card 3), strukturell identisch zum HCFG-02-Blueprint"
  - "Save/Clear-Button-Keys aus HCFG-02 re-used — spart 6 add_text-Aufrufe und hält den Wortlaut konsistent"
  - "is_within_shortday_gate mit #[cfg(test)] markiert — Kontraktspiegel des Backends, kein Runtime-Code"
metrics:
  duration_seconds: 611
  duration_minutes: 10
  tasks_completed: 4
  files_modified: 6
  commits: 5
  completed: 2026-07-05
status: complete
requirements: [SHC-06]
---

# Phase 51 Plan 08: Admin-Editor für ShortDay-Slot-Clipping-Stichtag — Summary

Admin-gated Settings-Card 2b für den Toggle `shortday_slot_clipping_active_from`,
strukturell identisch zum HCFG-02-Blueprint aus v1.7, mit drei neuen i18n-Keys
und einer pure-fn Validierung als D-25-06-Fallback.

## Was gebaut wurde

### Card 2b in Settings-Page

Neue admin-gated Settings-Card zwischen HCFG-02 Card 2 und der
Special-Days-Card 3 (`shifty-dioxus/src/page/settings.rs`, unmittelbar nach
dem Card-2-`div`-Block; nach Injection bei ~Zeile 1147):

- **Row A** — Label + Description (i18n).
- **Row B** — `<input type=date>`-Widget (max-width 200px, kongruent zu Card 2).
- **Row C** — Save + Clear + Inline-Feedback (`SettingsSaved` / `SettingsSaveError`
  re-used).
- **Row D** — UnsetHint (nur wenn nach dem initialen Load kein Datum gesetzt ist).

Der äußere `is_admin`-Gate der Settings-Seite (`settings.rs:471`) schützt die
gesamte Card — kein separater Inner-Gate nötig, weil die Page bei fehlendem
Admin-Privileg früh mit "Not authorized." zurückkehrt (WR-02).

### Loader-Wrapper

Zwei neue Funktionen in `shifty-dioxus/src/loader.rs`:

- `get_shortday_clipping_active_from(config) -> Result<Option<String>, _>`
- `set_shortday_clipping_active_from(config, Option<&str>) -> Result<(), _>`

Beide delegieren an die bestehende `/toggle`-API (Endpoint aus v1.7 HCFG-02).
Toggle-Name `"shortday_slot_clipping_active_from"` spiegelt die Backend-Konstante
`service_impl::shortday_gate::TOGGLE_NAME`. Die Toggle-Row wurde bereits durch
die P02-Migration mit `value = NULL` seed-inserted, deshalb ist der erste Save
ein `UPDATE`, nicht ein `INSERT`.

### i18n-Keys (drei neue, in allen drei Locales)

`shifty-dioxus/src/i18n/mod.rs`:

- `Key::SettingsShortdayClippingLabel`
- `Key::SettingsShortdayClippingDescription`
- `Key::SettingsShortdayClippingUnsetHint`

Übersetzungen:

| Key | DE | EN | CS |
|---|---|---|---|
| Label | "Kurzer-Tag-Slot-Kürzung aktiv ab" | "Short-day slot clipping active from" | "Zkracování slotů v krátkých dnech aktivní od" |
| Description | "Ab diesem Datum werden Slots an Kurzen Tagen am Cutoff gekürzt (Rendering und Ist-Stunden). Leer lassen = keine Kürzung." | "From this date, slots on short days are clipped at the cutoff time (rendering and actual hours). Leave empty to disable." | "Od tohoto data se sloty v krátkých dnech zkracují na čas cutoff (zobrazení i hodiny). Prázdné = zkracování vypnuto." |
| UnsetHint | "Nicht gesetzt — Kürzung inaktiv." | "Not set — clipping is off." | "Nenastaveno — zkracování vypnuto." |

Save/Clear-Button-Labels bleiben `SettingsHolidayAutoCreditSave` und
`SettingsHolidayAutoCreditClear` (Wortlaut generisch — spart Duplikate und
hält die Card-Familie konsistent).

Deckungs-Test `i18n_phase51_shortday_clipping_keys_present_in_all_locales`
in `i18n/mod.rs` (Nachbau des `phase25`-Tests, iteriert über alle drei
Locales und die drei neuen Keys).

### Pure-fn Validator + Kontraktspiegel

**`is_valid_shortday_date_input(&str) -> bool`** — akzeptiert Leerstring
(Legacy off) oder eine gültige ISO-8601-`YYYY-MM-DD`-Datumsangabe;
verwirft alles andere. Wird vom Save-Button als Disable-Guard verwendet
(`sc_save_disabled = is_sc_saving || sc_date_empty || !is_valid_shortday_date_input(...)`)
und im Handler als Defense-in-Depth vor dem PUT.

**`is_within_shortday_gate(booking_date, active_from) -> bool`** — spiegelt
`service_impl::shortday_gate::should_clip` (Backend P02) mit inklusiver
Grenze (`booking_date >= active_from`). Reiner Kontraktspiegel, deshalb
`#[cfg(test)]`. Der Test dokumentiert dem FE-Reviewer die SHC-06-Semantik
und failt sofort, wenn die Grenze im Backend jemals von `>=` auf `>`
flippt, ohne dass der FE-Code angepasst wird.

### Test-Namen (Task 3 TDD)

Alle in `shifty-dioxus/src/page/settings.rs::tests`:

- `test_empty_shortday_input_is_valid`
- `test_valid_iso_date_is_accepted` (inkl. Schaltjahr 2028-02-29)
- `test_malformed_date_is_rejected` (DE-Format, Natural Language, Range,
  Whitespace)
- `test_grenzfall_active_from_equals_booking_date` (SHC-06 Boundary
  Case, inkl. `None`-Fall)

Plus `i18n_phase51_shortday_clipping_keys_present_in_all_locales` in
`i18n/mod.rs::tests`.

**TDD-Zyklus:** RED-Commit (`b98c39f`, Funktions-Stubs geben `false`
zurück, 3/4 Tests failen) → GREEN-Commit (`1c0bd86`, echte Implementierung,
alle 4 Tests grün).

### Kein neuer Backend-Endpoint, kein Dioxus.toml-Change

Wie erwartet: `/toggle`-Proxy ist bereits in
`shifty-dioxus/Dioxus.toml:110` — Auto-Memory
`feedback_dioxus_proxy_for_new_backend_endpoints` bewusst gecheckt und
bestätigt. Snapshot-Version bleibt 12 (kein Backend-Snapshot berührt).

## Verifikation

- `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` — grün.
- `cd shifty-dioxus && cargo test` — **800 passed, 0 failed** (alle FE-Tests).
- `cd shifty-dioxus && cargo clippy -- -D warnings` — grün (keine Warnings).
- `cargo test --workspace` (Backend) — grün (Backend nicht angefasst).
- `cargo clippy --workspace -- -D warnings` (Backend) — grün.

Manuelle Sichtprüfung wurde in diesem autonomen Executor-Run nicht durchgeführt
(D-25-06-Fallback deckt genau diesen Fall ab: der pure-fn-Test belegt die
Save-Button-Logik ohne Browser-Interaktion). Falls der Verifier später die
Card visuell prüfen möchte: Backend hochfahren, `dx serve --hot-reload` in
`shifty-dioxus/` starten, als Admin einloggen, Settings-Seite öffnen — Card 2b
erscheint unter der HCFG-02-Card mit demselben Layout.

## Deviations from Plan

**None** — Plan wurde exakt wie geschrieben umgesetzt.

Kleine Präzisierungen ohne semantische Abweichung:

- Task 4 (Gate-only) hat keinen eigenen Commit, weil keine Code-Änderung
  nötig war — die Gates sind Konsequenz der Task-1/2/3-Commits. STATE.md
  wird davon nicht berührt.
- `is_within_shortday_gate` habe ich mit `#[cfg(test)]` markiert (nicht im
  Plan explizit gefordert). Alternative wäre `#[allow(dead_code)]`, aber
  `cfg(test)` ist expliziter — die Fn ist reiner Kontraktspiegel, nicht
  Runtime-Code, und soll auch nicht ausversehen als solcher missbraucht
  werden.

## Deferred Issues

Keine.

## Known Stubs

Keine.

## Commits

- `692665e` — feat(51-08): add i18n keys for short-day slot clipping settings card
- `b98c39f` — test(51-08): add failing tests for shortday clipping validator (RED)
- `1c0bd86` — feat(51-08): implement shortday clipping validator + gate mirror (GREEN)
- `592aa60` — feat(51-08): add short-day slot-clipping Settings Card 2b

## Self-Check

Verifiziert nach Write:

- FOUND: shifty-dioxus/src/page/settings.rs (Card 2b Block + pure fns + Tests)
- FOUND: shifty-dioxus/src/loader.rs (get/set_shortday_clipping_active_from)
- FOUND: shifty-dioxus/src/i18n/mod.rs (3 Key-Enum-Varianten + Coverage-Test)
- FOUND: shifty-dioxus/src/i18n/de.rs, en.rs, cs.rs (3× 3 add_text)
- FOUND: commits 692665e, b98c39f, 1c0bd86, 592aa60

## Self-Check: PASSED
