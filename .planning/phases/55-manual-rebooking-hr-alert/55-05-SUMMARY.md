---
phase: 55-manual-rebooking-hr-alert
plan: 05
subsystem: ui
tags: [dioxus, wasm, rebooking, hr-alert, page-integration, ssr-tests]

requires:
  - phase: 55-04
    provides: "3 Dioxus-Komponenten (RebookingAlertBanner, ManualRebookingModal, RebookingSuggestionModal) prop-driven + state/rebooking.rs Mapper + loader-Fns (submit_manual_rebooking, open_rebooking_suggestion POST-Claim, approve_/reject_rebooking_suggestion) + 27 i18n-Keys"
provides:
  - "page/employees.rs: RebookingAlertBanner pro ShortEmployeeReportTO mit has_pending_rebooking==true (HR-Path) + RebookingSuggestionModal-Mount; Banner-Klick -> open_rebooking_suggestion (POST-Claim, Modell B) -> Modal; Approve/Reject -> Loader -> Modal schliessen + Overview-Refetch (Banner verschwindet, D-55-07)"
  - "page/employee_details.rs: HR-gated F3-Trigger (Btn Secondary, RebookingManualTrigger) in der Action-Zone (D-55-06, NICHT in der Voluntary-Stats-Zeile) -> ManualRebookingModal -> submit_manual_rebooking -> Report-Refetch (EmployeeAction::Refresh); 409 haelt Modal offen (slot_taken_error inline)"
  - "state/employee.rs: Employee traegt has_pending_rebooking + pending_rebooking_id (Durchreichung aus ShortEmployeeReportTO) + volunteer_hours (Banner-Anzeigewert)"
affects: []

tech-stack:
  added: []
  patterns:
    - "Page besitzt den Async-Dispatch + 409/Refetch-Loop; die Komponenten bleiben prop-driven (Callbacks) und SSR-testbar (Praezedenz absence_convert_modal)"
    - "Backend-getriebene Banner-Sichtbarkeit ueber DTO-Flag has_pending_rebooking (kein FE-Rollen-Check) — Guard-Test T-55-16"
    - "Refetch nach Decide/Submit ueber reports.restart + Sidebar-Refresh-Token bzw. EmployeeAction::Refresh, sodass der entschiedene Batch/die Umbuchung sichtbar wird"
    - "Temporaere 55-04-#[allow]s entfernt, wo die Loader/Komponenten/API-Clients jetzt live sind — clippy -D warnings organisch gruen ohne Suppression"

key-files:
  created: []
  modified:
    - "shifty-dioxus/src/page/employees.rs"
    - "shifty-dioxus/src/page/employee_details.rs"
    - "shifty-dioxus/src/state/employee.rs"
    - "shifty-dioxus/src/service/employee.rs"
    - "shifty-dioxus/src/state/rebooking.rs"
    - "shifty-dioxus/src/component/employee_short.rs"
    - "shifty-dioxus/src/component/employees_list.rs"
    - "shifty-dioxus/src/component/mod.rs"
    - "shifty-dioxus/src/component/rebooking_suggestion_modal.rs"
    - "shifty-dioxus/src/api.rs"
    - "shifty-dioxus/src/loader.rs"
    - "shifty-dioxus/src/tests/volunteer_work_tests.rs"

key-decisions:
  - "F3-Trigger sitzt in der Action-Zone/TopBar der Employee-Details-Seite, NICHT in der Voluntary-Stats-Zeile (D-55-06). Banner-Sichtbarkeit haengt ausschliesslich am backend-gelieferten has_pending_rebooking-Flag (kein FE-Rollen-Check, T-55-16-Guard)."
  - "Der Async-Dispatch (open/approve/reject/submit) + der 409/Refetch-Loop liegen in der Page, nicht in den Komponenten — die Komponenten bleiben prop-driven und SSR-testbar. Employee carriet die zwei Flag-Felder + volunteer_hours aus dem ShortEmployeeReportTO durch (Fat Backend, Thin Client; keine FE-Arithmetik)."
  - "Recovery-Hinweis: Der Executor-Agent stuerzte nach den zwei feat-Commits (52a21c0f, b21c27d7) mit einem API-Connection-Fehler ab, BEVOR er diese SUMMARY.md + STATE/ROADMAP schreiben konnte. Beide Implementierungs-Commits sind vollstaendig in HEAD; der Orchestrator hat den finalen Integrations-Gate selbst nachgefahren (siehe Self-Check) und diese SUMMARY nachtraeglich erstellt."

commits:
  - "52a21c0f feat(55-05): HR-Alert-Banner + Suggestion-Modal in Employee-Overview verdrahten"
  - "b21c27d7 feat(55-05): F3-Manual-Rebooking-Trigger in Employee-Details-TopBar/Action-Zone"

requirements: [REB-MANUAL-03, HR-ALERT-01]
---

# Plan 55-05 — Seiten-Integration (F3/F5 Vertical-Slice-Anschluss)

## Was gebaut wurde

Die drei Phase-55-Komponenten wurden in die zwei realen Seiten integriert und schliessen
damit die vertikalen Slices F3 (manuelle Umbuchung) und F5 (HR-Alert + Vorschlag) ab:

1. **Employee-Overview (`page/employees.rs`, HR-Path):** Pro `ShortEmployeeReportTO` mit
   backend-gesetztem `has_pending_rebooking == true` rendert ein klickbarer
   `RebookingAlertBanner` (natuerliche Report-Reihenfolge). Klick loest
   `open_rebooking_suggestion` (POST-Claim, Modell B) aus und mountet das
   `RebookingSuggestionModal` mit der gelieferten Vorschau. Approve/Reject dispatchen die
   55-04-Loader, schliessen das Modal und refetchen die Overview (`reports.restart` +
   Sidebar-Refresh-Token), sodass der Banner verschwindet (Predicate false / entschiedener
   Batch, D-55-07). Bei 409 kein Modal-Open bzw. generischer inline-Fehler + Refetch.

2. **Employee-Details (`page/employee_details.rs`):** Ein HR-gated F3-Trigger (Btn Secondary,
   Label `RebookingManualTrigger`) in der Action-Zone (D-55-06 — NICHT in der
   Voluntary-Stats-Zeile) oeffnet das `ManualRebookingModal` fuer die aktuelle Person
   (SalesPerson-Id via `current_employee_id`-Signal, Default-KW = aktuelle KW). Submit ruft
   `submit_manual_rebooking`; bei Erfolg Modal schliessen + Report-Refetch
   (`EmployeeAction::Refresh`), sodass die Umbuchung als Batch sichtbar wird. Bei 409
   (belegter Slot, D-55-05) bleibt das Modal offen (`slot_taken_error` inline). Submit
   waehrend Request disabled (`in_flight`).

3. **State-Durchreichung (`state/employee.rs`):** `Employee` traegt jetzt
   `has_pending_rebooking` + `pending_rebooking_id` (aus `ShortEmployeeReportTO`) und
   carriet `volunteer_hours` durch (Banner-Anzeigewert). Kein FE-Rollen-Check, keine
   FE-Arithmetik.

4. **Aufraeumen:** Die in 55-04 gesetzten temporaeren `#[allow(dead_code)]` /
   `#[allow(unused_imports)]` wurden entfernt, wo die Loader/Komponenten/API-Clients jetzt
   live sind (open/approve/reject/submit-Loader, `RebookingSuggestion`-Struct,
   `dann_balance_class`, die vier `post_rebooking_*`-api-Clients, Banner-/Modal-Re-Exports).

## Prohibitions eingehalten

- Kein F3-Button in der Voluntary-Stats-Zeile — Action-Zone/TopBar (D-55-06). ✔
- Kein FE-Rollen-Check fuer den Banner — Sichtbarkeit haengt am backend-gelieferten
  `has_pending_rebooking`-Flag (Guard-Test T-55-16). ✔
- Keine FE-Arithmetik — alle Werte kommen backend-fertig. ✔

## Tests

- `short-report`-Mapper-Test auf carry-through umgestellt + Flag-Durchreichung getestet
  (`employee_carries_pending_rebooking_flag_from_short_report`).
- Guard-Test gegen FE-Rollen-Check im Banner-Pfad (T-55-16).
- Frontend-Suite gesamt: **831 passed / 0 failed** (inkl. i18n-Presence-Gate,
  Button-Type-Gate, Token-Class-Gate).

## Self-Check: PASSED

- [x] Beide feat-Commits in HEAD verifiziert (`52a21c0f`, `b21c27d7`).
- [x] `page/employees.rs` + `page/employee_details.rs` integriert (Banner, Suggestion-Modal,
      Manual-Modal, Loader-Aufrufe grep-verifiziert).
- [x] Frontend `cargo clippy --target wasm32-unknown-unknown -- -D warnings`: **green**.
- [x] Frontend `cargo test`: **831 passed, 0 failed**.
- [x] Working Tree sauber (nur pre-existing `.planning/config.json`).

**Recovery-Note:** Der Executor-Agent stuerzte nach den beiden feat-Commits mit einem
API-Connection-Fehler ab, bevor er SUMMARY/STATE/ROADMAP schreiben konnte. Der Orchestrator
hat den Integrations-Gate (clippy + test) selbst nachgefahren (beide gruen) und diese
SUMMARY + das Tracking nachgetragen. Keine Code-Nacharbeit noetig — die Implementierung war
vollstaendig und gruen.
