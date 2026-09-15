---
phase: 55-manual-rebooking-hr-alert
plan: 04
subsystem: ui
tags: [dioxus, wasm, i18n, rebooking, hr-alert, tailwind, dialog, ssr-tests]

requires:
  - phase: 55-03
    provides: "REST-Routes F3/F5 + rest-types::rebooking (RebookingSuggestionTO IST/DANN backend-computed, ManualRebookingRequestTO, RebookingSuggestRequestTO, RebookingApproveRequestTO, RebookingDirectionTO) + ShortEmployeeReportTO.has_pending_rebooking/pending_rebooking_id + Dioxus-Dev-Proxies"
provides:
  - "3 Dioxus-Komponenten: RebookingAlertBanner (Zero-State-Guard + A11y), ManualRebookingModal (F3, Submit-Defense + inline 409), RebookingSuggestionModal (F5, IST/DANN-Grid + Approve/Reject)"
  - "state/rebooking.rs: thin From<&RebookingSuggestionTO>/<&RebookingBatchTO>-Mapper + RebookingDirection (keine Arithmetik)"
  - "loader.rs: submit_manual_rebooking, open_rebooking_suggestion (POST-Claim, Modell B), approve_rebooking_suggestion, reject_rebooking_suggestion + api.rs post_rebooking_* (409 -> ShiftyError::Conflict)"
  - "27 i18n-Keys (Banner/Suggestion-Modal/Manual-Modal/Fehler) in de/en/cs (Presence-Gate gruen)"
affects: [55-05, 55-06]

tech-stack:
  added: []
  patterns:
    - "Prop-driven Modals (on_submit/on_approve/on_reject/on_cancel-Callbacks) — Async-Dispatch bleibt der Page (55-05) vorbehalten; SSR-testbar wie absence_convert_modal"
    - "Dialog-Shell-Reuse (kein eigenes fixed-inset-0-Backdrop) => BUG-03-Grep-Invariant nicht getriggert (neue Modal-Files muessen NICHT ins FILES-Set)"
    - "Backend-getriebene Sichtbarkeit ueber DTO-Flag (has_pending_rebooking) statt FE-Logik — SSR-Component-Tests statt Browser-Tests (Pitfall 11)"
    - "409 -> ShiftyError::Conflict-Mapping im api.rs-Client (Praezedenz update_absence_period), Modal bleibt offen bzw. Page refetcht"

key-files:
  created:
    - "shifty-dioxus/src/state/rebooking.rs"
    - "shifty-dioxus/src/component/rebooking_alert_banner.rs"
    - "shifty-dioxus/src/component/manual_rebooking_modal.rs"
    - "shifty-dioxus/src/component/rebooking_suggestion_modal.rs"
  modified:
    - "shifty-dioxus/src/i18n/mod.rs"
    - "shifty-dioxus/src/i18n/de.rs"
    - "shifty-dioxus/src/i18n/en.rs"
    - "shifty-dioxus/src/i18n/cs.rs"
    - "shifty-dioxus/src/state/mod.rs"
    - "shifty-dioxus/src/loader.rs"
    - "shifty-dioxus/src/api.rs"
    - "shifty-dioxus/src/component/mod.rs"
    - "shifty-dioxus/src/tests/volunteer_work_tests.rs"

key-decisions:
  - "RebookingSuggestionModal rendert nur die real vom Backend gelieferten IST/DANN-Zeilen (Balance + Freiwillig-Ist + proposed_hours). Die im UI-SPEC skizzierten voluntary_soll/voluntary_delta-Zeilen existieren in der 55-03-Domain/DTO NICHT (55-03 Deviation 1) — sie werden NICHT gerendert, um die Prohibition 'keine Fake-/FE-gerechneten Werte' zu wahren. Die zugehoerigen i18n-Keys bleiben im Contract erhalten (27 Keys)."
  - "Komponenten sind prop-driven (Callbacks) statt selbst-dispatchend — der Async-Loader-Aufruf + der 409/Refetch-Loop gehoert der Page (55-05). Das haelt die Komponenten SSR-testbar (Praezedenz absence_convert_modal) und die Loader-/API-Fns bleiben bis 55-05 ungenutzt (per #[allow(dead_code)]-mit-Grund markiert)."
  - "A11y-Keydown ueber ev.key().to_string()-Vergleich (Enter / ' ') statt dioxus::events::Key-Konstanten (Idiom aus page/absences.rs; Key::Character(\" \") serialisiert zu \" \")."
  - "open_rebooking_suggestion ist ein POST-Claim (Modell B), KEIN GET — es erzeugt/oeffnet den pending-Batch und liefert die RebookingSuggestion; bei 409 (entschiedener Slot) reicht der Fehler durch."

patterns-established:
  - "Zero-State-Guard: rebooking_alert_banner rendert bei has_pending_rebooking==false gar nichts (early rsx!{}); Sichtbarkeit rein backend-getrieben"
  - "IST/DANN-Anzeige-Grid grid-cols-[1fr_auto_auto] mit voluntary_stats_row-Typo/Farbe (font-mono tabular-nums, text-good>=0 / text-warn<0), reine Formatierung backend-computed Werte"

requirements-completed: [REB-MANUAL-03, HR-ALERT-01, HR-ALERT-02, HR-ALERT-03]

coverage:
  - id: D1
    description: "27 i18n-Keys (Banner/Suggestion-Modal/Manual-Modal/Fehler) in de/en/cs vollstaendig"
    requirement: "REB-MANUAL-03"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/i18n/mod.rs#i18n_rebooking_keys_present_in_all_locales"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/i18n/mod.rs#i18n_rebooking_templates_carry_placeholders"
        status: pass
    human_judgment: false
  - id: D2
    description: "RebookingAlertBanner: Zero-State-Guard (has_pending_rebooking) + klickbar (role=button/tabindex=0/Enter-Space)"
    requirement: "HR-ALERT-01"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/component/rebooking_alert_banner.rs#banner_hidden_when_no_pending_rebooking"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/component/rebooking_alert_banner.rs#banner_visible_with_pending_flag_and_a11y"
        status: pass
    human_judgment: false
  - id: D3
    description: "RebookingSuggestionModal: IST/DANN-Grid ausschliesslich aus RebookingSuggestionTO (kein FE-Rechnen), DANN good/warn, Approve/Reject in-flight disabled, Empty/Error-State"
    requirement: "HR-ALERT-02"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/component/rebooking_suggestion_modal.rs#grid_shows_backend_ist_dann_values_only"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/component/rebooking_suggestion_modal.rs#in_flight_disables_both_buttons"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/component/rebooking_suggestion_modal.rs#empty_state_shows_copy_and_only_close"
        status: pass
    human_judgment: false
  - id: D4
    description: "ManualRebookingModal (F3): Richtung/Menge/Woche + Vorschau + Submit-Defense (Menge>0) + inline 409 (Modal bleibt offen)"
    requirement: "HR-ALERT-03"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/component/manual_rebooking_modal.rs#open_modal_renders_fields_and_defaults"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/component/manual_rebooking_modal.rs#slot_taken_error_renders_inline"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/component/manual_rebooking_modal.rs#in_flight_disables_submit"
        status: pass
    human_judgment: false
  - id: D5
    description: "state/rebooking.rs thin From-Mapper ohne Arithmetik + Loader/API (submit/open/approve/reject, 409->Conflict)"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/state/rebooking.rs#suggestion_mapper_is_pure_field_copy"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/state/rebooking.rs#direction_roundtrips_through_to"
        status: pass
    human_judgment: false
  - id: D6
    description: "Visuelle UI-SPEC-Konformitaet (Klassenstrings, Typografie, Farb-Token) der 3 neuen Oberflaechen"
    verification: []
    human_judgment: true
    rationale: "Pixel-/Layout-/Copywriting-Feinschliff der neuen Surfaces braucht menschliche Sichtpruefung im laufenden Frontend (55-05 mountet Banner + Modals in die Pages); SSR-Tests decken Struktur/Klassen, nicht das gerenderte Erscheinungsbild."

duration: ~45min
completed: 2026-07-24
status: complete
---

# Phase 55 Plan 04: Rebooking-Frontend (HR-Alert-Banner + Manual-/Suggestion-Modal) Summary

**Drei neue Dioxus/WASM-Komponenten (klickbarer HR-Alert-Banner mit Zero-State-Guard, F3-Manual-Modal mit Submit-Defense + inline 409, F5-Suggestion-Modal mit backend-computed IST/DANN-Grid) plus thin State-Mapper, vier Loader/API-Funktionen und 27 i18n-Keys (de/en/cs) — Fat Backend, Thin Client durchgaengig.**

## Performance

- **Duration:** ~45 min
- **Completed:** 2026-07-24
- **Tasks:** 3
- **Files modified:** 13 (4 created, 9 modified)

## Accomplishments

- **27 i18n-Keys** (Banner 2, Suggestion-Modal 10, Manual-Modal 11, Fehler/Zustaende 4) in `enum Key` + de/en/cs — inkl. Presence-Gate `i18n_rebooking_keys_present_in_all_locales` (27 Keys) und Placeholder-Guard-Test.
- **state/rebooking.rs** — thin `From<&RebookingSuggestionTO>`/`<&RebookingBatchTO>`-Mapper + `RebookingDirection`-Enum (reines Feld-Kopieren, per Test `suggestion_mapper_is_pure_field_copy` verifiziert; kein arithmetischer Ausdruck).
- **4 Loader + 4 API-Clients** — `submit_manual_rebooking`, `open_rebooking_suggestion` (POST-Claim, Modell B), `approve_rebooking_suggestion`, `reject_rebooking_suggestion`; `post_rebooking_*` mit zentralem 409->`ShiftyError::Conflict`-Mapping.
- **RebookingAlertBanner** — Zero-State-Guard (nur bei `has_pending_rebooking==true`), klickbar (`role=button`, `tabindex=0`, Enter/Space), warn-Container 1:1 per UI-SPEC, Zahlen `font-mono tabular-nums`, `on_open(pending_rebooking_id)`.
- **ManualRebookingModal (F3)** — Dialog-Shell (Center/460), Richtung/Menge/Woche (getippte Jahr/KW, KEIN week-Picker), Text-Vorschau, Submit-Defense (Menge>0), inline 409 (`RebookingErrorSlotTaken`, Modal bleibt offen), in-flight-disabled Submit.
- **RebookingSuggestionModal (F5)** — Dialog-Shell + Drei-Spalten-IST/DANN-Grid (`grid-cols-[1fr_auto_auto]`), DANN-Balance `text-good`(>=0)/`text-warn`(<0), hervorgehobene `proposed_hours`-Zeile, Reject(Danger)+Approve(Primary) in-flight disabled, Empty-State + inline `RebookingErrorGeneric`.

## Task Commits

1. **Task 1: i18n-Keys (27, de/en/cs) + State-Mapper + Loader/API** — `20c1df6a` (feat)
2. **Task 2: rebooking_alert_banner + manual_rebooking_modal** — `eb3d50ed` (feat)
3. **Task 3: rebooking_suggestion_modal (IST/DANN-Grid + Approve/Reject)** — `c8a35cd6` (feat)

## Files Created/Modified

**Created:**
- `shifty-dioxus/src/state/rebooking.rs` — thin From-Mapper + RebookingDirection (keine Arithmetik) + Tests
- `shifty-dioxus/src/component/rebooking_alert_banner.rs` — Zero-State-Guard + A11y-klickbare Warn-Zeile
- `shifty-dioxus/src/component/manual_rebooking_modal.rs` — F3-Modal, Submit-Defense + inline 409
- `shifty-dioxus/src/component/rebooking_suggestion_modal.rs` — F5-IST/DANN-Grid + Approve/Reject

**Modified:**
- `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs` — 27 Keys + Presence/Placeholder-Tests
- `shifty-dioxus/src/state/mod.rs` — `pub mod rebooking;`
- `shifty-dioxus/src/loader.rs` — 4 Loader-Fns (+ `#[allow(dead_code)]`-mit-Grund bis 55-05)
- `shifty-dioxus/src/api.rs` — 4 `post_rebooking_*`-Clients (409->Conflict)
- `shifty-dioxus/src/component/mod.rs` — 3 Modul-Deklarationen + Re-Exports
- `shifty-dioxus/src/tests/volunteer_work_tests.rs` — Testliteral um 55-03-Felder ergaenzt (Rule 3)

## Decisions Made

- **IST/DANN-Grid nur mit real gelieferten Feldern:** `RebookingSuggestionTO` traegt (55-03 Deviation 1) nur `balance`/`balance_after` + `voluntary_ist`/`voluntary_ist_after` + `proposed_hours` — KEINE `voluntary_soll`/`voluntary_delta`. Das Modal rendert daher nur diese Zeilen; die im UI-SPEC skizzierten Soll/Delta-Zeilen werden ausgelassen, um "keine Fake-/FE-gerechneten Werte" zu wahren. Die i18n-Keys bleiben (27 Keys, Contract).
- **Prop-driven Modals:** Callbacks statt Selbst-Dispatch → SSR-testbar, Async-Loop gehoert der Page (55-05).
- **A11y-Keydown** via `ev.key().to_string()`-Vergleich (Idiom `page/absences.rs`).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] ShortEmployeeReportTO-Testliterale um 55-03-Felder ergaenzt**
- **Found during:** Task 1 (Baseline-Testlauf schlug fehl)
- **Issue:** 55-03 hatte `has_pending_rebooking` + `pending_rebooking_id` zu `ShortEmployeeReportTO` hinzugefuegt, aber zwei Struct-Literale im Frontend (`src/loader.rs` working-hours-Mini-Test-Fixture, `src/tests/volunteer_work_tests.rs`) wurden nicht mitgezogen → `cargo test` kompilierte nicht (E0063). Das blockierte das i18n-Gate.
- **Fix:** Beide Literale um `has_pending_rebooking: false, pending_rebooking_id: None` ergaenzt (Feld-Drift-Nachzug, kein Verhalten geaendert).
- **Files modified:** `shifty-dioxus/src/loader.rs`, `shifty-dioxus/src/tests/volunteer_work_tests.rs`
- **Verification:** `cargo test` kompiliert + 830 Tests gruen.
- **Committed in:** `20c1df6a` (Task 1 commit)

**2. [Rule 1/3 - Hygiene] `#[allow(dead_code)]`/`#[allow(unused_imports)]` mit Begruendung fuer noch-nicht-verdrahtete Surface**
- **Found during:** Task 1-3 (Clippy `-D warnings` ist harter Gate)
- **Issue:** Loader/API-Fns, `RebookingBatch`-Mapper und die Komponenten-Re-Exports werden erst in 55-05 (Page-Wiring) konsumiert → im bin-Build dead-code/unused-import → Clippy `-D warnings` haette gefailt.
- **Fix:** Gezielte `#[allow(...)]`-Attribute je mit `// reason:`-Kommentar (Praezedenz `load_feature_flag`), der auf den 55-05-Konsumenten verweist. `dann_balance_class` (aus RSX aufgerufen, Subtree bis 55-05 dead) analog.
- **Files modified:** `api.rs`, `loader.rs`, `state/rebooking.rs`, `component/mod.rs`, `component/rebooking_suggestion_modal.rs`
- **Verification:** `cargo clippy -p shifty-dioxus --target wasm32-unknown-unknown -- -D warnings` gruen.
- **Committed in:** `20c1df6a`, `eb3d50ed`, `c8a35cd6`

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 hygiene). **Impact:** Kein Scope-Creep — der Feld-Nachzug ist reine 55-03-Drift-Korrektur; die allow-Attribute halten die per-Task-Clippy-Gates gruen ohne die spaetere 55-05-Verdrahtung vorwegzunehmen.

## Issues Encountered

- **wasm32-Target-Verfuegbarkeit:** `rustup target list` war leer (Nix-Toolchain). Der `wasm32-unknown-unknown`-Target ist ueber die Nix-`rustc` vorhanden (`.../rustlib/wasm32-unknown-unknown`), sodass `cargo build/clippy --target wasm32-unknown-unknown` wie im Plan-Verify funktioniert.

## User Setup Required

None - keine externe Service-Konfiguration.

## Next Phase Readiness

- **55-05 (Page-Integration):** Banner in `page/employees.rs` (HR-Path, pro SalesPerson mit `has_pending_rebooking==true`) mounten; F3-Trigger in `page/employee_details.rs` TopBar; Async-Dispatch der 4 Loader + 409/Refetch-Loop verdrahten. Beim Verdrahten werden die `#[allow(dead_code)]`-Marker auf Loader/API/Komponenten-Exports lebendig (redundante allows sind clippy-neutral, koennen entfernt werden).
- **Offener Contract-Punkt fuer 55-05/55-06:** Falls die Soll/Delta-IST/DANN-Zeilen fachlich noch gewuenscht sind, muss zuerst das Backend (55-01/02) die Felder liefern (Rule-4-Architektur) — nicht im FE nachrechnen.

---
*Phase: 55-manual-rebooking-hr-alert*
*Completed: 2026-07-24*

## Self-Check: PASSED

- Created files verified on disk: `state/rebooking.rs`, `component/rebooking_alert_banner.rs`, `component/manual_rebooking_modal.rs`, `component/rebooking_suggestion_modal.rs`, `55-04-SUMMARY.md`.
- Task commits verified in git log: `20c1df6a`, `eb3d50ed`, `c8a35cd6`.
- Gate results: frontend `cargo test` 830 passed / 0 failed (inkl. i18n 27-Key-Presence-Gate); `cargo build --target wasm32-unknown-unknown` gruen; `cargo clippy -p shifty-dioxus --target wasm32-unknown-unknown -- -D warnings` gruen.
