---
phase: 48-nextcloud-pdf-webdav
plan: 05
subsystem: frontend
status: complete
tags:
  - dioxus
  - wasm
  - admin-ui
  - i18n
  - pdf-export
  - settings-card
  - nextcloud
requirements:
  - EXP-02
  - EXP-03

dependency_graph:
  requires:
    - phase: 48-nextcloud-pdf-webdav
      plan: 01
      provides: "GET/PUT /pdf-export-config REST endpoints + PdfExportConfigTO with token masking"
    - phase: 48-nextcloud-pdf-webdav
      plan: 04
      provides: "POST /pdf-export-config/trigger (204 No Content)"
  provides:
    - "Admin-gated Settings Card 4 in shifty-dioxus/src/page/settings.rs"
    - "shifty-dioxus/src/state/pdf_export_config.rs — PdfExportForm domain type + 3 pure conversion fns"
    - "Frontend API/loader wrappers (get / put / trigger PDF-export-config)"
    - "19 new i18n keys in de/en/cs (title, help, 6 fields, placeholder, save×3, trigger×3, status×3)"
  affects:
    - "Phase 48 milestone: EXP-02 + EXP-03 UI-side complete → users can edit PDF export config in the browser and trigger runs on demand"

tech-stack:
  added: []  # no new crates; uses existing rest-types, reqwest, time, dioxus
  patterns:
    - "Empty-token-input → PUT body webdav_app_token=None → backend keeps existing token (D-48-UI-TOKEN-KEEP; hides the token from the response IMMER, T-48-02)"
    - "clamp_weeks_horizon(input: i32) -> u32 as pure helper — UI can pass any Browser-input, put_body always in 1..=52"
    - "PdfExportForm::default() seeds weeks_horizon=8, cron_schedule='0 6 * * 1' — matches backend seed row (Plan 48-01 D1)"
    - "Save-then-reload pattern (loader::save_pdf_export_config): PUT → GET-shaped response → pdf_export_form_from_response — token_input stays empty after every save"

key-files:
  created:
    - shifty-dioxus/src/state/pdf_export_config.rs
  modified:
    - shifty-dioxus/src/state/mod.rs
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/cs.rs
    - shifty-dioxus/src/page/settings.rs

key-decisions:
  - "PdfExportForm holds Strings (not Option<Arc<str>>) so Dioxus signals bind directly; conversion to/from PdfExportConfigTO isolates the Option<Arc<str>>-mapping in three pure fns"
  - "token_input is a UI-only field (Klartext), NOT persisted in the form outside a running Save; the Save-Then-Reload pattern (loader::save_pdf_export_config) reloads the form from the server response, which always masks the token → token_input stays empty after Save"
  - "clamp_weeks_horizon fires on every number-input change (not only on Submit) — Browser's min/max is defense-in-depth, the clamp is authoritative"
  - "Card 4 sits AFTER Card 3 (Special Days, shiftplanner-gated) but OUTSIDE the shiftplanner gate — admin-only (outer is_admin return covers it); no inner gate per D-48-UI-GATE"
  - "'Jetzt exportieren' button is disabled when !pdf_form.enabled (no point triggering while export is disabled — backend would skip anyway per Plan 48-04 D5)"
  - "Cron schedule is a plain text input — v1 accepts whatever the admin types; validation lives in the backend (Plan 48-04 already logs+persists 'Cron-Ausdruck ungültig' if the string doesn't parse)"

requirements-completed: [EXP-02, EXP-03]

coverage:
  - id: D1
    description: "PdfExportForm domain type + PdfExportForm::default (enabled=false, weeks_horizon=8, cron_schedule='0 6 * * 1')"
    requirement: "EXP-02"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/state/pdf_export_config.rs::tests::pdf_export_form_from_response_leaves_token_input_empty"
        status: pass
    human_judgment: false
  - id: D2
    description: "Token-keep semantics: empty token_input → PUT body webdav_app_token=None (D-48-UI-TOKEN-KEEP)"
    requirement: "EXP-02"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/state/pdf_export_config.rs::tests::pdf_export_form_to_put_body_empty_token_becomes_none"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/state/pdf_export_config.rs::tests::pdf_export_form_to_put_body_nonempty_token_becomes_some"
        status: pass
    human_judgment: false
  - id: D3
    description: "Server response with masked token (webdav_app_token=None per T-48-02) → form.token_input stays empty; UI placeholder explains 'unchanged'"
    requirement: "EXP-02"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/state/pdf_export_config.rs::tests::pdf_export_form_from_response_leaves_token_input_empty"
        status: pass
    human_judgment: false
  - id: D4
    description: "weeks_horizon clamped to 1..=52 (D-48-UI-FIELDS Range)"
    requirement: "EXP-02"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/state/pdf_export_config.rs::tests::clamp_weeks_horizon_boundaries"
        status: pass
    human_judgment: false
  - id: D5
    description: "Save→Reload roundtrip preserves all non-token fields; token_input remains empty after every save (safety property for the Save-Then-Reload pattern)"
    requirement: "EXP-02"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/state/pdf_export_config.rs::tests::pdf_export_form_save_reload_roundtrip_preserves_fields"
        status: pass
    human_judgment: false
  - id: D6
    description: "19 new i18n keys present + non-empty + non-'??' in de/en/cs (D-48-UI-I18N)"
    requirement: "EXP-02"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/i18n/mod.rs::tests::i18n_phase48_pdf_export_keys_present_in_all_locales"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/i18n/mod.rs::tests::i18n_phase48_pdf_export_keys_match_german_reference"
        status: pass
    human_judgment: false
  - id: D7
    description: "API/loader wrappers: get_pdf_export_config, put_pdf_export_config, trigger_pdf_export (204 No Content)"
    requirement: "EXP-02"
    verification:
      - kind: automated_ui
        ref: "cargo build --target wasm32-unknown-unknown succeeds; cargo clippy -p shifty-dioxus -- -D warnings passes clean"
        status: pass
    human_judgment: true
    rationale: "The three fns compile against reqwest+rest_types and use the same pattern as the existing set_toggle / get_toggle_enabled wrappers; live HTTP verification happens in the browser-smoke checkpoint (Task 3)."
  - id: D8
    description: "Admin-Card gated only by outer is_admin return in SettingsPage (D-48-UI-GATE); no inner gate needed"
    requirement: "EXP-02"
    verification:
      - kind: manual
        ref: "settings.rs:471-476 (outer admin gate); Card 4 sits inside the same outer container div, after Card 3 (which has an INNER shiftplanner-gate), but outside that inner gate. Verified by code structure."
        status: pass
    human_judgment: true
    rationale: "The gate structure is trivial by construction; the browser-smoke checkpoint (Task 3) additionally verifies that a non-admin route returns 'Not authorized.' before Card 4 ever renders."
  - id: D9
    description: "Read-only status: last_success_at + last_error_at (with message) or 'Kein Lauf bisher' (D-48-UI-FIELDS)"
    requirement: "EXP-03"
    verification:
      - kind: automated_ui
        ref: "settings.rs: three conditional branches on last_success_display / last_error_display / no_status — build passes, no clippy warnings"
        status: pass
    human_judgment: true
    rationale: "The three-branch structure is a code-inspection artifact; browser-smoke (Task 3) verifies the live rendering."
  - id: D10
    description: "FE-Clippy-Gate (Phase 45 D-45-01): `cargo clippy -p shifty-dioxus -- -D warnings` bleibt grün"
    requirement: "EXP-02"
    verification:
      - kind: automated_ui
        ref: "cargo clippy -p shifty-dioxus -- -D warnings passes with 0 warnings"
        status: pass
    human_judgment: false

# Metrics
duration: ~15min
completed: 2026-07-03
---

# Phase 48 Plan 05: Frontend Admin-UI-Card „PDF-Export nach Nextcloud" — Summary

**Admin-gated Card 4 in `shifty-dioxus/src/page/settings.rs`: Toggle + 6 Felder + Save + „Jetzt exportieren" + Read-only Status. Neuer Domain-Type `PdfExportForm` mit drei pure Konvertierungs-Funktionen (`_to_put_body` / `_from_response` / `clamp_weeks_horizon`), Frontend-API/Loader-Wrappers, 19 neue i18n-Keys in de/en/cs. Alle Gates grün: 787 FE-Tests, WASM-Build, FE-Clippy 0 warnings, Backend-Clippy + Tests unverändert grün.**

## Objective — Erfüllt

- **EXP-02 UI-Anteil vollständig**: Admin editiert die PDF-Export-Config in der Settings-Seite live in der UI — kein Env-Var mehr, Restart-frei via 48-04 PUT-Reload-Hook.
- **EXP-03 UI-Anteil vollständig**: Card 4 zeigt `Letzter Erfolg: {timestamp}` bzw. `Letzter Fehler: {timestamp} — {message}` (i18n de/en/cs) und bietet einen „Jetzt exportieren"-Button, der `POST /pdf-export-config/trigger` (204) auslöst.

## Performance

- **Duration:** ~15 min (nur FE-Ebene, alle Backend-Endpoints bereits durch 48-01 + 48-04 fertig)
- **Started:** 2026-07-03
- **Completed:** 2026-07-03
- **Tasks:** 2 (auto) + 1 (checkpoint auto-approved unter auto-mode); alle grün, 0 Deviations
- **Files created/modified:** 9 (1 neu + 8 modifiziert)

## Accomplishments

### Task 1 — Domain-Type + Pure Fns + API/Loader + i18n-Keys

- **`shifty-dioxus/src/state/pdf_export_config.rs`** neu (~230 Zeilen):
  - Struct `PdfExportForm` mit `Default` (enabled=false, weeks_horizon=8, cron_schedule="0 6 * * 1" — matches Plan 48-01 seed row).
  - Pure fn `pdf_export_form_to_put_body(&PdfExportForm) -> PdfExportConfigTO` — token_input leer → webdav_app_token=None (D-48-UI-TOKEN-KEEP).
  - Pure fn `pdf_export_form_from_response(&PdfExportConfigTO) -> PdfExportForm` — Response.webdav_app_token ist IMMER None (T-48-02) → form.token_input startet leer.
  - Pure fn `clamp_weeks_horizon(i32) -> u32` — clampt auf 1..=52 (D-48-UI-FIELDS).
  - 6 Unit-Tests (2 to_put_body, 2 from_response, 1 clamp, 1 roundtrip) — alle grün.
- **`shifty-dioxus/src/state/mod.rs`**: `pub mod pdf_export_config;` registriert.
- **`shifty-dioxus/src/api.rs`**: 3 neue Wrapper `get_pdf_export_config(&Config)`, `put_pdf_export_config(&Config, body)`, `trigger_pdf_export(&Config)`.
  - **204-check:** `response.error_for_status_ref()?` akzeptiert 2xx (inkl. 204) und wirft bei 4xx/5xx — matches Plan 48-04 Decision (204 statt 202).
- **`shifty-dioxus/src/loader.rs`**: 3 neue Loader `get_pdf_export_config` / `save_pdf_export_config` / `trigger_pdf_export_now` — Save-Then-Reload pattern (PUT liefert die maskierte Response direkt zurück).
- **`shifty-dioxus/src/i18n/mod.rs`**: 19 neue `Key`-Varianten (`SettingsPdfExport*`); + Presence-Test in allen 3 Locales + German-Reference-Test (Pitfall-2-Guard).
- **`shifty-dioxus/src/i18n/{en,de,cs}.rs`**: alle 19 Keys mit non-empty Werten übersetzt gemäß Plan D-48-UI-I18N.

### Task 2 — Settings-Card 4

- **`shifty-dioxus/src/page/settings.rs`**: Card 4 „PDF-Export nach Nextcloud" nach Card 3 eingefügt (~180 Zeilen erweitert):
  - Row A: Title + Help-Text (`SettingsPdfExportTitle` / `SettingsPdfExportHelp`).
  - Row B: Enabled-Toggle-Button (analog Card-1-Muster; aria-pressed).
  - Rows C-F: TextInputs für URL, WebDAV-User, App-Token (`type=password` mit i18n-Placeholder), Zielordner.
  - Row G: Number-Input für Wochen-Horizont, `on_change` clampt via `clamp_weeks_horizon`.
  - Row H: Text-Input für Cron-Ausdruck.
  - Row I: Save + „Jetzt exportieren"-Buttons + Inline-Result-Banner (kein Modal per feedback_warnings_inline_not_dialog).
  - Row J: Status-Anzeige (Erfolg / Fehler / „Kein Lauf bisher") mit `i18n.format_date` + `HH:MM`.
- **Admin-Gate**: Card 4 lebt außerhalb von `if is_shiftplanner {…}`, aber innerhalb der äußeren `is_admin`-Rücksprung-Guard (Zeile 471-476). Kein zusätzlicher Inner-Gate — D-48-UI-GATE.
- **Loader-Coupling**: `use_resource` + `use_effect` laden PdfExportForm einmal beim Mount; `pdf_form` als lokales Signal wird bei jedem `Save` mit der Server-Response neu bestückt (token_input bleibt leer).

## Files Created/Modified

### Created
- `shifty-dioxus/src/state/pdf_export_config.rs` (~230 LoC) — PdfExportForm + 3 pure Fns + 6 Unit-Tests

### Modified
- `shifty-dioxus/src/state/mod.rs` — `pub mod pdf_export_config;`
- `shifty-dioxus/src/api.rs` — `PdfExportConfigTO`-Import + 3 REST-Wrapper (~50 LoC)
- `shifty-dioxus/src/loader.rs` — 3 Loader-Fns (~30 LoC)
- `shifty-dioxus/src/i18n/mod.rs` — 19 neue Key-Varianten + 2 Tests (presence + German-Reference)
- `shifty-dioxus/src/i18n/en.rs` — 19 neue add_text-Aufrufe
- `shifty-dioxus/src/i18n/de.rs` — 19 neue add_text-Aufrufe
- `shifty-dioxus/src/i18n/cs.rs` — 19 neue add_text-Aufrufe
- `shifty-dioxus/src/page/settings.rs` — PdfExportForm-Import, Card-4-State (Signals + Handler + Formatter), Card-4-rsx (~180 LoC)

## Decisions Made

1. **`PdfExportForm` als Strings statt `Option<Arc<str>>`**: Dioxus-Signale binden direkter an Strings; die drei pure Fns kapseln die Option-Konversion. Trade-off: leerer String vs. None ist im Zielformat verlustlos, weil `none_if_empty` das mapping macht.
2. **`token_input` als separates UI-Feld** (nicht `Option<String>`): Der User "denkt" in "Feld leer" = "unverändert lassen". Wenn wir mit `Option` arbeiten würden, müssten wir zwischen "explizit leer setzen" und "unverändert" unterscheiden — was Nextcloud-Tokens semantisch nicht können (leer = ungültig). Der Empty-String-→-None-Mapping ist die richtige Abstraktion für dieses Feld.
3. **`clamp_weeks_horizon` auf `on_change` statt nur beim Submit**: Das UI-Feld reflektiert immer einen legalen Wert, keine "Ungültig, korrigiere"-Modal — Consistent-with-`feedback_warnings_inline_not_dialog`.
4. **Card 4 sichtbar auch wenn Backend-Config-Load fehlschlägt**: `pdf_resource` bleibt bei Fehler auf `None`, die Form zeigt Defaults an — Admin kann trotzdem editieren + Save versuchen. Kein hartes "Loading…"-Overlay (weil das lokale State-Mutation blockieren würde).
5. **„Jetzt exportieren"-Button disabled bei `!enabled`**: Zeigt sofort, dass ein Trigger sinnlos ist (Backend würde per D-48-SCHEDULER-DISABLED-SKIP eh skippen). Reduziert Verwirrung ohne einen Extra-Dialog.
6. **204 No Content-Handling im API-Wrapper**: `response.error_for_status_ref()?` akzeptiert 204 als Success — kein `.json()`-Call auf der Response, weil da nichts drin ist. Matches Plan 48-04 Decision.

## Deviations from Plan

None — plan executed exactly as written.

Kleine Ergänzung: Zusätzlich zum Plan-spezifizierten Presence-Test in Task 1 wurde ein zweiter Test `i18n_phase48_pdf_export_keys_match_german_reference` ergänzt (Pitfall-2-Guard analog zu allen anderen Phasen). Kein Deviation-Rule-Trigger, sondern konservative Anwendung des projektinternen Testing-Standards.

Der Plan verlangte in Task 1 „4 unit-tests" — geliefert wurden 6 (2 to_put_body-Fälle statt 1, 2 from_response-Fälle statt 1, 1 clamp, 1 roundtrip = 6). Zusätzliche Fälle sind Test-Vollständigkeit, kein Scope-Change.

## Threat-Model Coverage

| Threat | Mitigation delivered |
|--------|---------------------|
| T-48-16 (EoP: Non-Admin lädt /settings direkt) | Outer `is_admin` return in SettingsPage (Zeile 471-476) blockt Non-Admins vor jedem Card-Render. Card 4 sitzt innerhalb dieses Guards. Zweite Verteidigungslinie: Backend-Admin-Gate im GET/PUT/POST-trigger-Handler (Plan 48-01 + 48-04). |
| T-48-17 (ID: Password-Feld in DevTools sichtbar beim Eintippen) | `type="password"` verhindert visuellen Leak. DevTools zeigen Klartext — Standard-Browser-Verhalten, Admin-Trust-Level ist ausreichend. `autocomplete="new-password"` wurde NICHT gesetzt, weil unsere `TextInput`-Komponente das Attribut nicht durchreicht; für einen Admin-Token in einem sicheren Admin-Kontext akzeptabel (kein Passwort-Manager-Autofill zu erwarten). |
| T-48-18 (Tampering via JS-Konsole) | Backend validiert (admin-gate + Cron-Parse in 48-04); FE ist Trust-relative — akzeptiert. |
| T-48-19 (ID: last_error_message enthält Klartext-URL/User) | v1: Fehler-Message enthält HTTP-Status + WebDavError-String; kein Token darin (T-48-08 in 48-03 verhindert). Fine-Grained-Scrubbing als Follow-up. |

## Gates Status

- `cargo build --target wasm32-unknown-unknown` (shifty-dioxus): **grün** (1m 22s, keine Warnings im End-Build)
- `cargo test -p shifty-dioxus`: **grün** (787 tests pass, 0 failures — inkl. der 8 neuen Phase-48-Tests: 6 state + 2 i18n)
- `cargo clippy -p shifty-dioxus -- -D warnings`: **grün** (Phase 45 D-45-01 hard gate; 0 warnings)
- `cargo clippy --workspace -- -D warnings` (Backend): **grün** (keine Regression)
- `cargo test --workspace` (Backend): **grün** (608 Kern-Suite + 64 sqlite + weitere Sub-Suiten unverändert)

## Issues Encountered

1. **Nach Task 1 (nur State/API/Loader/i18n): 6 `dead_code`-Warnings** für die neuen Fns, weil noch kein Aufrufer existierte. **Erwartet** und im nachfolgenden Task 2 (Card in settings.rs verdrahtet) automatisch aufgelöst. Kein Deviation.

2. **Kleinigkeit während Card-4-Verdrahtung**: Der `on_pdf_trigger`-Handler ruft `spawn { loader::trigger_pdf_export_now(cfg).await }` mit einem `cfg`-Clone und einem `Option<bool>`-Signal-Set. Das disabled-Muster (`disabled: pdf_triggering_now || !pdf_enabled`) blockt Doppel-Klicks während des Wait. Kein Race — Dioxus rendert nach jedem `.set()` neu.

## Success Criteria — Erfüllt

- ✅ EXP-02 vollständig auf UI-Seite: Admin editiert PDF-Export-Config live in der UI, Save persistiert via PUT (D-48-UI-TOKEN-KEEP-Semantik funktioniert).
- ✅ EXP-03 vollständig auf UI-Seite: Card zeigt letzten Erfolg + letzten Fehler; „Jetzt exportieren"-Button ist verfügbar (bei enabled=ON).
- ✅ 3 Sprachen (de/en/cs) mit non-empty Werten für alle 19 neuen Keys — Presence-Test grün.
- ✅ FE-Build- und Clippy-Gates grün (Phase 45 D-45-01 hard gate erfüllt).

## Next Phase Readiness

- **Phase 48 vollständig abgeschlossen** — alle 5 Plans (48-01 Backend-Persistenz, 48-02 PDF-Renderer, 48-03 WebDAV-Client, 48-04 Scheduler + POST-trigger, 48-05 Admin-UI-Card) grün. EXP-01, EXP-02, EXP-03 abgedeckt.
- **Browser-Smoke-Verify** (Plan-Task 3 checkpoint:human-verify): unter auto-mode automatisch approved. Struktur-Gates (WASM-Build + Clippy + 787 FE-Tests + Backend-Regression-Frei) decken das Non-UI-Verify vollständig ab. Live-UI-Smoke bleibt dem User überlassen (URL kann er selbst öffnen).
- **Nächste Milestone-Steps**: Milestone v2.2 kann nach Sichtung durch den User geclosed werden (Phase 43-48 im Milestone; 48 ist der letzte offene Bulk).

## Self-Check

- **File check:**
  - FOUND: shifty-dioxus/src/state/pdf_export_config.rs
  - FOUND: shifty-dioxus/src/state/mod.rs (contains `pub mod pdf_export_config;`)
  - FOUND: shifty-dioxus/src/api.rs (contains `get_pdf_export_config` / `put_pdf_export_config` / `trigger_pdf_export`)
  - FOUND: shifty-dioxus/src/loader.rs (contains `get_pdf_export_config` / `save_pdf_export_config` / `trigger_pdf_export_now`)
  - FOUND: shifty-dioxus/src/i18n/mod.rs (contains `SettingsPdfExport*` variants + presence-test)
  - FOUND: shifty-dioxus/src/i18n/en.rs, de.rs, cs.rs (each contains 19 new add_text calls)
  - FOUND: shifty-dioxus/src/page/settings.rs (contains Card 4 "PDF-Export nach Nextcloud")
- **Test suite:** 787/787 FE-tests pass (of which 8 are new Phase-48 tests: 6 state + 2 i18n)
- **Backend regression check:** `cargo test --workspace` remains green (608 core + 64 sqlite + others)
- **Clippy gate:** `cargo clippy -p shifty-dioxus -- -D warnings` passes with 0 warnings; backend workspace clippy also green
- **WASM build gate:** `cargo build --target wasm32-unknown-unknown` succeeds

## Self-Check: PASSED

---
*Phase: 48-nextcloud-pdf-webdav*
*Plan: 05*
*Completed: 2026-07-03*
