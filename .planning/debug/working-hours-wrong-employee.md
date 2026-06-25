---
slug: working-hours-wrong-employee
status: resolved
trigger: "Wenn man in der Mitarbeiteransicht einen Mitarbeiter auswählt und einen Vertrag (Working Hours) anlegen will, wird der Vertrag dem falschen Mitarbeiter zugeordnet."
created: 2026-06-12
updated: 2026-06-25
resolution: "Gefixt im Frontend (employee_details.rs): employee_id wird per Signal `current_employee_id` gespiegelt und der Coroutine-Handler liest `*current_employee_id.peek()` statt eines eingefrorenen Closure-Captures. Regressionstest `FROZEN_CAPTURE` vorhanden. User-bestätigt 2026-06-25 beim v1.5-Milestone-Start."
---

# Debug Session: Working Hours dem falschen Mitarbeiter zugeordnet

## Symptoms

- **Expected behavior:** Wird in der Mitarbeiteransicht ein Mitarbeiter ausgewählt und ein Vertrag (Working Hours) angelegt, soll der Vertrag genau diesem ausgewählten Mitarbeiter zugeordnet werden.
- **Actual behavior:** Der angelegte Vertrag wird einem *falschen* Mitarbeiter zugeordnet.
- **Error messages:** (keine bekannt — Fehlverhalten ist eine falsche Zuordnung, kein Crash)
- **Timeline:** (unbekannt — vom User zu klären)
- **Reproduction:** Mitarbeiteransicht öffnen → Mitarbeiter auswählen → Vertrag/Working Hours anlegen → Vertrag landet beim falschen Mitarbeiter.

## Current Focus

reasoning_checkpoint:
  hypothesis: "In `EmployeeDetails` (page/employee_details.rs) capturt das `use_coroutine`-Closure den `employee_id: Uuid` per `move` EINMALIG beim ersten Mount. Im Master/Detail-Layout bleibt `EmployeeDetails` gemountet, wenn man via Sidebar-Link zu einem anderen Mitarbeiter wechselt (nur der Route-Prop `employee_id` ändert sich). Der Coroutine-Handler `NewEmployeeWorkDetails` sendet aber `NewWorkingHours(employee_id)` mit dem EINGEFRORENEN ersten employee_id — daher landet der neue Vertrag beim zuerst geöffneten Mitarbeiter, nicht beim aktuell angezeigten."
  confirming_evidence:
    - "EmployeeDetails bleibt gemountet bei Mitarbeiterwechsel: Kommentar Z.107-118 erklärt explizit, dass employee_id nur via `last_loaded_id`-Signal in den EmployeeService gesynct wird, WEIL die Komponente bei Prop-Wechsel gemountet bleibt."
    - "Das `use_coroutine`-Closure (Z.58-105) capturt `employee_id` per `async move` — der Wert wird beim ersten Aufbau der Coroutine festgehalten und NIE aktualisiert; nur reaktive Signale würden sich ändern."
    - "NewEmployeeWorkDetails-Arm (Z.71-76) sendet `NewWorkingHours(employee_id)` mit genau diesem eingefrorenen Wert; new_employee_work_details_for_sales_person setzt blank_standard(sales_person_id) → falsche sales_person_id im selected_employee_work_details → Save persistiert sie 1:1."
  falsification_test: "Wenn die Hypothese falsch wäre, müsste `employee_id` im Coroutine-Closure reaktiv mitwandern. Test: Sidebar Mitarbeiter A öffnen, dann zu B wechseln (ohne Reload), Vertrag anlegen → Vertrag landet bei A statt B = Hypothese bestätigt."
  fix_rationale: "employee_id muss zum Sende-Zeitpunkt aus der aktuellen Route/Signal gelesen werden statt aus dem eingefrorenen Closure-Capture. Korrektur: aktuellen employee_id in ein Signal spiegeln (last_loaded_id existiert bereits) bzw. das Closure den jeweils aktuellen Wert lesen lassen."
  blind_spots: "OpenEmployeeWorkDetails(Edit) ist nicht betroffen (nutzt id aus dem geklickten Contract). Muss prüfen: andere Closure-Captures von employee_id (ExtraHoursModal sales_person_id: employee_id Z.149) — die werden im rsx! gerendert und lesen props frisch, daher vermutlich ok."

## Evidence

- timestamp: 2026-06-12
  checked: "Backend create-Pfad (rest/src/employee_work_details.rs:34 create_working_hours, service_impl/src/employee_work_details.rs:182 create)"
  found: "Backend übernimmt sales_person_id 1:1 aus dem eingehenden EmployeeWorkDetailsTO; keine Ableitung aus Context/Auth. Persistiert exakt die vom Frontend gesendete sales_person_id."
  implication: "Root Cause liegt im Frontend — die falsche sales_person_id wird bereits im selected_employee_work_details gesetzt, bevor sie ans Backend geht."

- timestamp: 2026-06-12
  checked: "Save-Pfad Frontend (service/employee_work_details.rs Save-Arm + loader save_new_employee_work_details + TryFrom<&EmployeeWorkDetails> for EmployeeWorkDetailsTO)"
  found: "Save liest selected_employee_work_details und sendet dessen sales_person_id. Edits via UpdateWorkingHours überschreiben den ganzen Struct, behalten aber sales_person_id aus blank_standard. Quelle der sales_person_id ist also ausschließlich blank_standard(sales_person_id) aus NewWorkingHours(sales_person_id)."
  implication: "Die fehlerhafte sales_person_id stammt aus dem Argument von NewWorkingHours — und damit aus employee_id im EmployeeDetails-Coroutine-Closure."

- timestamp: 2026-06-12
  checked: "page/employee_details.rs Z.40-118 — Closure-Capture von employee_id im use_coroutine; Master/Detail-Mount-Verhalten"
  found: "employee_id wird per `async move` ins Coroutine-Closure gemoved und beim Mitarbeiterwechsel NICHT aktualisiert (Komponente bleibt gemountet, Kommentar Z.107-118 bestätigt dies). NewEmployeeWorkDetails sendet NewWorkingHours(employee_id) mit dem eingefrorenen Wert."
  implication: "Stale-Closure-Capture: Nach Wechsel A→B (ohne Remount) wird ein neuer Vertrag dem zuerst geladenen Mitarbeiter A zugeordnet statt B. = Root Cause."

## Eliminated

## Resolution

root_cause: "Stale closure capture im EmployeeDetails-use_coroutine: `employee_id` wird beim ersten Mount per `move` ins async-Closure gecaptured. Im Master/Detail-Layout bleibt EmployeeDetails bei Mitarbeiterwechsel gemountet, der Route-Prop ändert sich, aber der gecapturte `employee_id` bleibt eingefroren. NewEmployeeWorkDetails sendet `NewWorkingHours(employee_id)` mit dem alten Wert → neuer Vertrag wird dem zuerst geöffneten Mitarbeiter zugeordnet."
fix: "In page/employee_details.rs den Route-Prop employee_id in ein Signal `current_employee_id` gespiegelt (re-sync bei jedem Render) und im Coroutine-Handler NewEmployeeWorkDetails statt des eingefrorenen Captures `*current_employee_id.peek()` gelesen. Das bisherige separate `last_loaded_id`-Signal wurde mit diesem Signal + `loaded_once`-Flag konsolidiert (gleiche Load-Gate-Logik). So liest der Coroutine immer den AKTUELL angezeigten Mitarbeiter zum Dispatch-Zeitpunkt."
verification: "WASM-Build (cargo build --target wasm32-unknown-unknown) grün. Volle Frontend-Test-Suite grün (554 passed). Neuer Regressionstest `signal_mirror_tracks_current_employee_while_frozen_capture_is_stale` reproduziert den Master/Detail-Mount-Switch in einer VirtualDom und beweist: Signal-Read folgt dem aktuellen Mitarbeiter (zweite id), während ein frozen First-Mount-Capture auf der ersten id hängenbleibt (= der gefixte Bug)."
files_changed:
  - "shifty-dioxus/src/page/employee_details.rs"
