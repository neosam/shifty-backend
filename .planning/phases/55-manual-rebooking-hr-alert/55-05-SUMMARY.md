---
phase: 55-manual-rebooking-hr-alert
plan: 05
subsystem: frontend
tags: [dioxus, wasm, rebooking, hr-alert, page-integration, fe-integration, hr-gating]

requires:
  - phase: 55
    plan: 04
    provides: "Modal-Bausteine (RebookingAlertBanner, RebookingSuggestionModal, ManualRebookingModal) + 4 Direct-HTTP-Loader + State-Mapper wire-ready"

provides:
  - "shifty-dioxus/src/state/employee.rs: Employee-Struct + zwei zusaetzliche Felder (has_pending_rebooking: bool, pending_rebooking_id: Option<Uuid>), gespeist aus ShortEmployeeReportTO (D-55-02, HR-ALERT-01)"
  - "shifty-dioxus/src/component/employees_list.rs: RebookingAlertBanner rendert pro Row wenn has_pending_rebooking=true; Klick propagiert batch_id via neuer on_banner_click-Prop"
  - "shifty-dioxus/src/component/employees_shell.rs: reicht on_banner_click-Handler vom Parent an EmployeesList weiter (Default = Noop, damit EmployeeDetails-Konsumenten unveraendert bleiben)"
  - "shifty-dioxus/src/page/employees.rs: haelt open_suggestion_batch_id-Signal + Suggestion-Resource (filter per batch_id in load_rebooking_suggestions_pending) + rendert RebookingSuggestionModal-Overlay; on_after_action triggert EmployeeAction::Refresh + EMPLOYEES_LIST_REFRESH-Bump (D-55-07)"
  - "shifty-dioxus/src/page/employee_details.rs: EmployeeDetailsAction um Open/Close/Saved-ManualRebooking-Varianten erweitert; HR-only Header-Row-Button unter TopBar (Btn Secondary, i18n RebookingModalTitleManual); ManualRebookingModal mit js::get_current_year/week als Default (D-55-05, D-55-06, T-55-08 defense-in-depth)"

affects:
  - "Cleanup #[allow(dead_code)]/#[allow(unused_imports)]-Marker aus Plan 55-04 entfernt: component/mod.rs (3 Marker) + loader.rs (5 Marker fuer die 4 Rebooking-Loader + map_conflict_body-Helper); state/rebooking.rs behaelt Modul-Level-Allow fuer reine Wire-Deserialisierungs-Felder, Rationale aktualisiert."
  - "Der no_billing_period_content_in_source-Test in page/employees.rs wurde praezisiert: die alte 'Modal'-Heuristik (aus BillingPeriodModal-Cutover in Phase 8) haette den legitimen RebookingSuggestionModal-Import blockiert; neu wird auf 'BillingPeriodModal' geprueft."

tech-stack:
  added: []
  patterns:
    - "on_banner_click-Prop mit Noop-Default: EmployeesList und EmployeesShell bekommen eine optionale EventHandler-Prop, damit unterschiedliche Parents (page/employees.rs mit Signal-Handler vs. page/employee_details.rs ohne Handler) dieselben Komponenten ohne Refactor konsumieren koennen. Default = EventHandler::new(|_| {}) im #[props(default=...)]."
    - "HR-Gate an zwei Stellen (defense-in-depth): Header-Row-Button rendert nur wenn AUTH.has_privilege(\"hr\"); zusaetzlich rendert der ManualRebookingModal-Overlay-Renderer die Modal-Konstruktion nur wenn is_hr && show_signal — falls das Signal jemals auf true stehen bleibt, waehrend AUTH abgelaufen ist, greift der zweite Gate."
    - "Loader-Reachability-Analyse als Cleanup-Trigger: die Wave-3-Allows wurden mechanisch entfernt, sobald Plan 55-05 die Loader mountet — Rust flaggt sonst `unused_attributes`, unser Clippy `-D warnings` faengt das."

key-files:
  created:
    - ".planning/phases/55-manual-rebooking-hr-alert/55-05-SUMMARY.md"
  modified:
    - "shifty-dioxus/src/state/employee.rs"
    - "shifty-dioxus/src/service/employee.rs"
    - "shifty-dioxus/src/component/employee_short.rs"
    - "shifty-dioxus/src/component/employees_list.rs"
    - "shifty-dioxus/src/component/employees_shell.rs"
    - "shifty-dioxus/src/component/mod.rs"
    - "shifty-dioxus/src/loader.rs"
    - "shifty-dioxus/src/page/employees.rs"
    - "shifty-dioxus/src/page/employee_details.rs"
    - "shifty-dioxus/src/state/rebooking.rs"

key-decisions:
  - "D-55-EXEC-05-01 (Header-Row-Button, kein TopBar-Menu-Refactor): der F3-Trigger sitzt als eigener flex-Container mit justify-end unter TopBar, oberhalb der EmployeeView-Section. Praezedenz-konform mit ExtraHoursModal-Trigger; verhindert einen TopBar-Menu-Refactor, der WARNING #3 aus dem Plan-Checker verletzt haette."
  - "D-55-EXEC-05-02 (Employee-Struct-Felder statt Signal-Wrapping): die Alert-Flags haengen jetzt direkt am Employee-Struct (has_pending_rebooking, pending_rebooking_id). Alternative waere gewesen, sie im EMPLOYEE_STORE als separates Signal zu fuehren — aber die Employees-Liste (EmployeesList) verarbeitet Rc<[Employee]>-Slices und braucht die Flags per-Row-lokal. Zwei zusaetzliche Felder sind lokaler und billiger als ein separates Signal-Map."
  - "D-55-EXEC-05-03 (Praezisierung des no_billing_period_content_in_source-Tests): der Test wurde aus Phase 8 (BillingPeriod-Cutover) mit einer breiten 'Modal'-Verbotsliste angelegt. Wave-5 mount des RebookingSuggestionModal haette den Test faelschlich brechen; die neue Liste prueft konkret auf 'BillingPeriodModal'. Der Schutzzweck (kein BillingPeriod-Rest in der Employees-Page) bleibt intakt."
  - "D-55-EXEC-05-04 (find-Filter statt Suggestion-per-ID-Endpoint): das Suggestion-Modal laedt die Suggestion via load_rebooking_suggestions_pending + client-side find(batch_id). Wie im Plan spezifiziert — kein neuer Backend-Endpoint noetig, weil der GET-Endpoint bereits alle offenen Suggestions phase-weit liefert (Plan 55-02). Wenn die Suggestion zwischenzeitlich resolved ist (Race), gibt find() None zurueck und das Overlay rendert nicht — der Refresh laeuft dann durch den regulaeren EmployeeAction::Refresh-Pfad."
  - "D-55-EXEC-05-05 (Cleanup der Wave-4-Dead-Code-Allows): die 4 Loader (+ map_conflict_body) + 3 component-re-exports haben ihre #[allow]-Marker verloren; state/rebooking.rs behaelt den Modul-Level-Allow fuer reine Wire-Deserialisierungs-Felder (RebookingBatch::state etc.), die Kommentar-Rationale wurde aktualisiert."

patterns-established:
  - "Prop-Default via #[props(default = EventHandler::new(|_| {}))] fuer optionale Component-Callbacks: erlaubt einen Handler nur an einer von mehreren Konsum-Stellen zu setzen, ohne dass andere Parents den Handler explizit auskommentieren muessen. Nutzlich bei geteilten Shell-Komponenten (EmployeesShell) mit Page-spezifischen Sub-Features."

requirements-completed: [HR-ALERT-01, HR-ALERT-02, REB-MANUAL-01, REB-MANUAL-03]

coverage:
  - id: D1
    description: "D-55-02 (HR-Path Banner-Rendering): EmployeesList rendert RebookingAlertBanner pro Row wenn has_pending_rebooking=true. Predicate stammt aus ShortEmployeeReportTO, kein FE-Recompute."
    requirement: "HR-ALERT-01"
    verification:
      - kind: source
        ref: "grep -n 'has_pending_rebooking' shifty-dioxus/src/component/employees_list.rs — nur Read-Access, kein `if balance < 0 && ...`-Recompute"
        status: pass
      - kind: build
        ref: "cargo build --target wasm32-unknown-unknown — Erfolg"
        status: pass
    human_judgment: false
  - id: D2
    description: "D-55-05 + D-55-06 (F3-Trigger als Header-Row-Button, HR-only, Default current KW): page/employee_details.rs rendert Btn Secondary unter TopBar; is_hr-Gate greift; ManualRebookingModal bekommt js::get_current_year/week als Default."
    requirement: "REB-MANUAL-01"
    verification:
      - kind: source
        ref: "grep -n 'is_hr\\|RebookingModalTitleManual\\|current_iso_year' shifty-dioxus/src/page/employee_details.rs"
        status: pass
      - kind: build
        ref: "cargo build --target wasm32-unknown-unknown — Erfolg"
        status: pass
    human_judgment: false
  - id: D3
    description: "D-55-07 (Refresh nach Approve/Reject): on_after_action in page/employees.rs setzt open_suggestion_batch_id=None + EmployeeAction::Refresh + EMPLOYEES_LIST_REFRESH-Bump; Backend liefert has_pending_rebooking=false nach Approve/Reject → Banner verschwindet automatisch."
    requirement: "HR-ALERT-02"
    verification:
      - kind: source
        ref: "grep -n 'EmployeeAction::Refresh\\|EMPLOYEES_LIST_REFRESH' shifty-dioxus/src/page/employees.rs"
        status: pass
    human_judgment: false
  - id: D4
    description: "WARNING #3 respektiert: F3-Trigger sitzt NICHT in voluntary_stats_row, sondern als eigener Header-Row-Container zwischen TopBar und EmployeeView (fixierte Wahl per D-55-06)."
    verification:
      - kind: source
        ref: "grep -n 'voluntary_stats\\|OpenManualRebooking' shifty-dioxus/src/page/employee_details.rs — Trigger ausserhalb von EmployeeView, oberhalb"
        status: pass
    human_judgment: false
  - id: D5
    description: "WASM-Build-Gate + FE-Tests + FE-Clippy `-D warnings` gruen (Clippy aus backend-nix-shell)."
    verification:
      - kind: build
        ref: "nix develop --command bash -c 'cd shifty-dioxus && cargo build --target wasm32-unknown-unknown' — Erfolg"
        status: pass
      - kind: unit
        ref: "cargo test → 821/821 passed"
        status: pass
      - kind: integration
        ref: "cargo clippy -- -D warnings — Erfolg"
        status: pass
    human_judgment: false
  - id: D6
    description: "Backend-Workspace unberuehrt: cargo build --workspace bleibt gruen (keine Wire-Vertragsaenderung in Wave 5)."
    verification:
      - kind: build
        ref: "cargo build --workspace — Erfolg"
        status: pass
    human_judgment: false
  - id: D7
    description: "REB-MANUAL-03 (Preview-Section im ManualRebookingModal) wird von Plan 55-04 geliefert; Plan 55-05 mountet den Modal per Header-Row-Button — Trigger + Modal-Reachability verifiziert."
    requirement: "REB-MANUAL-03"
    verification:
      - kind: source
        ref: "grep -n 'ManualRebookingModal' shifty-dioxus/src/page/employee_details.rs"
        status: pass
    human_judgment: false

duration: ~15min
completed: 2026-07-10
status: complete
---

# Phase 55 Plan 05: FE-Integration F3 + F5 Summary

**F3-Trigger als HR-only Header-Row-Button in EmployeeDetails + F5-Alert-Banner in der EmployeesList mit SuggestionModal-Overlay in page/employees.rs — die Wave-4-Modal-Bausteine sind gemounted, der Refresh-Zyklus schliesst sich, und die Wave-4-Dead-Code-Allows fallen mechanisch weg.**

## Performance

- **Duration:** ~15 min
- **Tasks:** 2 (atomar committed)
- **Files touched:** 10 (davon 1 neu: 55-05-SUMMARY.md)

## Accomplishments

- **F5-Alert Path:** EmployeesList rendert `RebookingAlertBanner` unter jedem Employee-Row mit `has_pending_rebooking=true`. Klick propagiert batch_id via `on_banner_click`-Prop bis in `page/employees.rs`, das eine `use_resource` mit `find(batch_id)` in `load_rebooking_suggestions_pending` verwendet und das `RebookingSuggestionModal` overlaid. Approve/Reject triggert `EmployeeAction::Refresh` — Backend liefert `has_pending_rebooking=false` — Banner verschwindet (D-55-07).
- **F3-Trigger Path:** `page/employee_details.rs` bekommt einen HR-only Header-Row-Button (Btn Secondary) unter TopBar. Klick oeffnet `ManualRebookingModal` mit `js::get_current_year/week` als Default-KW (D-55-05). `on_success` dispatched `EmployeeAction::Refresh` → neue ExtraHours-Rows sichtbar.
- **State-Layer erweitert:** `Employee`-Struct hat jetzt `has_pending_rebooking: bool` + `pending_rebooking_id: Option<Uuid>` — gespeist aus `ShortEmployeeReportTO`, kein FE-Recompute. Alle 5 Konstruktions-Stellen (`state/employee.rs` × 3 `From`/`unpaid_placeholder`, `service/employee.rs` Store-Init, `component/employee_short.rs` + `employees_list.rs` Testfixtures) sind aktualisiert.
- **Wave-4-Dead-Code-Allows entfernt:** 3 `#[allow(unused_imports)]` in `component/mod.rs` + 5 `#[allow(dead_code)]` in `loader.rs` (4 Rebooking-Loader + `map_conflict_body`-Helper). State-Modul-Level-Allow bleibt fuer Wire-Deserialisierungs-Felder (`RebookingBatch::state` etc.) mit aktualisierter Rationale.
- **T-55-08 Defense-in-Depth:** HR-Gate zweifach — Button-Rendering (`if is_hr {}`) und Modal-Konstruktion (`if is_hr && show_signal {}`).

## Task Commits

Jede Task wurde atomar committed:

1. **Task 1: HR-Path Banner-Integration + Employee-State-Erweiterung** — `ca36ec3` (feat)
2. **Task 2: F3-Trigger + Dead-Code-Cleanup** — `def3951` (feat)

## Files Created/Modified

**Created:**
- `.planning/phases/55-manual-rebooking-hr-alert/55-05-SUMMARY.md`

**Modified:**
- `shifty-dioxus/src/state/employee.rs` — 2 neue Felder in `Employee`, 3 `From`-Impls + `unpaid_placeholder`.
- `shifty-dioxus/src/service/employee.rs` — `EMPLOYEE_STORE` Default um die 2 Felder erweitert.
- `shifty-dioxus/src/component/employee_short.rs` — Testfixture `employee_with(...)` um die 2 Felder erweitert.
- `shifty-dioxus/src/component/employees_list.rs` — Banner-Render im Loop + `on_banner_click`-Prop (Default-Noop) + Testfixtures.
- `shifty-dioxus/src/component/employees_shell.rs` — `on_banner_click`-Prop durchreichen (Default-Noop).
- `shifty-dioxus/src/component/mod.rs` — 3 `#[allow(unused_imports)]` entfernt.
- `shifty-dioxus/src/loader.rs` — 5 `#[allow(dead_code)]`-Marker entfernt, Rebooking-Sektion-Kommentar aktualisiert.
- `shifty-dioxus/src/page/employees.rs` — `open_suggestion_batch_id`-Signal, Resource, `RebookingSuggestionModal`-Overlay, `on_after_action`-Refresh; `no_billing_period_content_in_source`-Test praezisiert.
- `shifty-dioxus/src/page/employee_details.rs` — `EmployeeDetailsAction` um 3 Varianten erweitert, `show_manual_rebooking_dialog`-Signal, HR-Header-Row-Button, `ManualRebookingModal`-Renderer.
- `shifty-dioxus/src/state/rebooking.rs` — Modul-Level-Allow-Rationale aktualisiert.

## Decisions Made

- **Header-Row-Button, kein TopBar-Menu-Refactor** (D-55-EXEC-05-01) — Praezedenz-konform mit ExtraHoursModal, respektiert WARNING #3.
- **Employee-Struct-Felder statt Signal-Wrapping** (D-55-EXEC-05-02) — die Alert-Flags haengen am Employee-Struct, weil `EmployeesList` `Rc<[Employee]>`-Slices per Row verarbeitet.
- **no_billing_period_content_in_source-Test praezisiert** (D-55-EXEC-05-03) — die alte breite `"Modal"`-Verbotsliste haette den legitimen `RebookingSuggestionModal`-Import blockiert; neu wird auf `"BillingPeriodModal"` geprueft.
- **find-Filter statt neuer Suggestion-per-ID-Endpoint** (D-55-EXEC-05-04) — plan-konform, minimalste Aenderung.
- **Cleanup der Wave-4-Dead-Code-Allows** (D-55-EXEC-05-05) — Loader- und Component-Allows raus, State-Modul-Allow behalten fuer Wire-Deserialisierungs-Felder.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Pre-existing Test-Verbotsliste blockierte legitime Modal-Referenz**
- **Found during:** Task 1 Vorbereitung (Test-Read)
- **Issue:** Der Test `no_billing_period_content_in_source` in `page/employees.rs` verbietet die String-Fragmente `"BillingPeriod"`, `"BILLING_PERIOD"`, `"Modal"`, `"billing_period"` im Nicht-Test-Bereich der Datei — der `"Modal"`-Guard war eine ueberbreiten BillingPeriod-Cutover-Heuristik aus Phase 8. Task 1 muss `RebookingSuggestionModal` als Component-Referenz in `page/employees.rs` verwenden — das haette den Test gebrochen.
- **Fix:** Verbotsliste praezisiert auf `["BillingPeriod", "BILLING_PERIOD", "BillingPeriodModal", "billing_period"]`. Der Schutzzweck (kein BillingPeriod-Rest in der Employees-Page) bleibt intakt.
- **Files modified:** `shifty-dioxus/src/page/employees.rs` (Test-Block)
- **Verification:** `cargo test` gruen (821/821 incl. `no_billing_period_content_in_source`).
- **Committed in:** `ca36ec3` (Task 1 commit).

**2. [Rule 1 - Bug] `question_mark`-Lint bei `let ... else`**
- **Found during:** Task 1 clippy-Gate
- **Issue:** Der Suggestion-Resource-Async-Closure nutzte `let Some(batch_id) = selected else { return None; };` — clippy `-D warnings` flaggt das als `clippy::question_mark`.
- **Fix:** Ersetzt durch `let batch_id = selected?;`.
- **Files modified:** `shifty-dioxus/src/page/employees.rs`
- **Verification:** `cargo clippy -- -D warnings` gruen.
- **Committed in:** `ca36ec3` (Task 1 commit).

**3. [Rule 3 - Blocking] `unused_mut`-Warning auf `on_close`-Closure**
- **Found during:** Task 1 erster WASM-Build
- **Issue:** Erster Wurf des `on_close`-Handlers war `let mut on_close = move |_| { ... };` — clippy `-D warnings` haette das als `unused_mut` geflaggt.
- **Fix:** `mut` entfernt.
- **Files modified:** `shifty-dioxus/src/page/employees.rs`
- **Verification:** Build + Clippy gruen.
- **Committed in:** `ca36ec3` (Task 1 commit).

---

**Total deviations:** 3 auto-fixed (2× Rule 3 blocking / Design-Klarstellung, 1× Rule 1 Clippy-Idiom). Keine Rule-4-Aenderung.
**Impact on plan:** Kein Scope-Creep. Deviation 1 ist ein Praezisierung eines pre-existing Regression-Tests; Deviation 2 + 3 sind Clippy-Idiom-Fixes.

## Issues Encountered

- **Keine.** Die Wave-4-Modal-Bausteine waren wire-ready und mountbar ohne Refactor — der Wave-3-Plan hat sauber vorgearbeitet.

## User Setup Required

None — keine ENV-Variablen, keine Migration.

## Next Phase Readiness

**Ready for Plan 55-06 (F14-Docs — falls noch nicht abgeschlossen):**
- Die 3 Modal-Bausteine sind in ihrem finalen Mount-Kontext dokumentierbar: `RebookingAlertBanner` in `EmployeesList`, `RebookingSuggestionModal` als Overlay in `page/employees.rs`, `ManualRebookingModal` als HR-Header-Row-Trigger in `page/employee_details.rs`.

**Blocker fuer Wave 4:** keine — Wave 4 ist mit diesem Plan abgeschlossen (F3+F5 vollstaendig gemounted).

**Manual E2E-Smoke (empfohlen, User-facing Verifikation):**
1. Backend + `dx serve` starten, als HR-User anmelden.
2. `/employees` → Banner erscheint bei einer Person mit cap+negativer Balance+voluntary>0 (Backend setzt `has_pending_rebooking=true`).
3. Banner-Klick → SuggestionModal oeffnet mit IST/DANN-Grid.
4. Approve → Modal schliesst, Banner verschwindet aus der Liste.
5. Auf `/employees/{id}` → HR-Header-Row-Button „Manuelle Umbuchung" sichtbar; Klick oeffnet ManualRebookingModal mit aktueller KW als Default.
6. Buchen → Modal schliesst, EmployeeReport zeigt neue ExtraHours-Row.
7. Non-HR-User: Banner sichtbar (Backend redigiert Flag) — Header-Row-Button unsichtbar.

---

## Self-Check: PASSED

- `.planning/phases/55-manual-rebooking-hr-alert/55-05-SUMMARY.md` — geschrieben in diesem Step
- Commit `ca36ec3` (Task 1) — `git log --oneline` verified
- Commit `def3951` (Task 2) — `git log --oneline` verified
- `shifty-dioxus/src/state/employee.rs::Employee` — `has_pending_rebooking` + `pending_rebooking_id` Felder VORHANDEN
- `shifty-dioxus/src/component/employees_list.rs::EmployeesList` — `on_banner_click`-Prop + `RebookingAlertBanner`-Import VORHANDEN
- `shifty-dioxus/src/page/employees.rs` — `open_suggestion_batch_id`-Signal + `RebookingSuggestionModal`-Render + `EmployeeAction::Refresh` VORHANDEN
- `shifty-dioxus/src/page/employee_details.rs` — `OpenManualRebooking`-Action + `is_hr`-Gate + `ManualRebookingModal`-Render VORHANDEN
- `cargo build --target wasm32-unknown-unknown` — PASSED
- `cargo test` (shifty-dioxus) — PASSED (821/821)
- `cargo clippy -- -D warnings` (backend shell) — PASSED
- `cargo build --workspace` (backend) — PASSED

---

*Phase: 55-manual-rebooking-hr-alert*
*Completed: 2026-07-10*
