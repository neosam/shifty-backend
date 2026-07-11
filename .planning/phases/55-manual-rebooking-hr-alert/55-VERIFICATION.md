---
phase: 55-manual-rebooking-hr-alert
verified: 2026-07-10T12:00:00Z
status: human_needed
score: 18/18 must-haves verified
behavior_unverified: 2
overrides_applied: 0
behavior_unverified_items:
  - truth: "D-55-07 Approve/Reject in SuggestionModal triggers page refresh -> alert row disappears from list"
    test: "Als HR-User: /employees aufrufen, Banner einer Person mit negativer Balance+voluntary>0 anklicken, Approve klicken -> Banner verschwindet ohne manuellen Reload."
    expected: "Banner ist nach Approve nicht mehr sichtbar; Backend liefert has_pending_rebooking=false."
    why_human: "Statetransition Banner-verschwindet-nach-Approve ist ein Full-UI-Roundtrip (Backend-Refresh, Dioxus-Signal-Propagation, WASM-Renderer). Grep/Build-Check beweisen nur Symbol-Praesenz + Wiring des on_after_action-Handlers."
  - truth: "REB-MANUAL-03 modal shows preview (Menge/Richtung/Woche) before submit"
    test: "ManualRebookingModal oeffnen, hours/direction/week einstellen -> Vorschau-Text erscheint vor Submit-Klick."
    expected: "Preview-Text zeigt 'Umbuchung: X h — Richtung — KW W/Jahr' live beim Eingeben."
    why_human: "Visuelles Rendering-Verhalten im WASM-Browser; SSR-Test prueft Strukturerstellung, nicht die Live-Aktualisierung beim Tippen."
human_verification:
  - test: "F5-Roundtrip: Banner erscheint bei negativer Balance + Voluntary>0 + cap_active, verschwindet nach Approve"
    expected: "HR-User sieht Banner in /employees fuer betroffene Person; nach Approve-Klick im Modal ist Banner weg (on_after_action triggert Refresh)."
    why_human: "D-55-07 State-Transition erfordert Full-UI-Roundtrip: Backend-Approve + Reporting-Reload + Dioxus-Signal-Propagation + WASM-Render."
  - test: "F3-Preview-Sektion aktualisiert sich live beim Eingeben"
    expected: "ManualRebookingModal zeigt 'Umbuchung: {hours} h — {direction} — KW {week}/{year}' unverzoeglich beim Aendern der Inputs."
    why_human: "Reaktivitaet der Preview-Signals in WASM kann nicht per Grep oder SSR verifiziert werden."
---

# Phase 55: Manuelle Umbuchung + HR-Alert-Modal (F3 + F5) — Verifikationsbericht

**Phase-Ziel:** REB-MANUAL-01/02/03 (F3 Manuelle Umbuchung) + HR-ALERT-01/02/03/04 (F5 HR-Alert + Vorschlags-Modal) + VOL-ACCT-03 (Property-Test Roundtrip-Neutralitaet)
**Verifiziert:** 2026-07-10
**Status:** human_needed
**Re-Verifikation:** Nein — initiale Verifikation

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidenz |
|----|-------|--------|---------|
| 1  | D-55-01 alert_predicate pure fn mit korrekten Grenzwerten | VERIFIED | `service/src/rebooking_reconciliation.rs:alert_predicate()`; Truth-Table-Test 7 Cases gruen (balance=-0.49 → false, -0.5 → true, -0.51 → true) |
| 2  | D-55-02 ShortEmployeeReportTO bekommt has_pending_rebooking:bool + pending_rebooking_id:Option<Uuid> | VERIFIED | `rest-types/src/lib.rs` Z.387+393; `#[serde(default)]` vorhanden; From-Impl Z.406+407 |
| 3  | D-55-03 proposed_hours + voluntary_delta_before/after Backend-computed via RebookingSuggestionTO | VERIFIED | `service/src/rebooking_reconciliation.rs:proposed_rebooking_hours()`; `RebookingSuggestionTO` in rest-types hat `voluntary_delta_before/after`; Fat-Backend-Guard per include_str-Test in suggestion_modal |
| 4  | D-55-04 Kein Undo-State auf RebookingBatchState | VERIFIED | `dao/src/rebooking_batch.rs`: Enum hat nur Pending/Approved/Rejected/SkippedLocked; kein Undone |
| 5  | D-55-05 iso_year/iso_week sind HR-gewaehlt (nicht server-seitig abgeleitet) | VERIFIED | `ManualRebookingRequestTO.iso_year/iso_week` als Pflichtfelder; Modal-Props `current_iso_year/current_iso_week` als Default; Caller-supplied in `rebook_manual`-Signatur |
| 6  | D-55-06 Direction (VolunteerWork<->ExtraWork) ist Caller-supplied | VERIFIED | `RebookingDirection`-Enum; Radio-Group in `manual_rebooking_modal.rs` mit onChange-Handlern; Direction als Body-Feld in `ManualRebookingRequestTO` |
| 7  | D-55-07 Approve+Reject beide beenden Pending-State | VERIFIED | `approve_suggestion` + `reject_suggestion` schreiben state-conditional UPDATE; `BatchAlreadyResolved` bei Race → 409; `on_after_action` triggert EMPLOYEES_LIST_REFRESH |
| 8  | D-55-07 Banner verschwindet nach Approve/Reject (Refresh-Roundtrip) | PRESENT_BEHAVIOR_UNVERIFIED | Code vorhanden + verdrahtet (on_after_action→Refresh→Backend liefert false); Transition ist UI-Roundtrip |
| 9  | REB-MANUAL-01 rebook_manual schreibt 2 ExtraHours + 1 Batch + 1 Entry in EINER Tx | VERIFIED | `service_impl/src/rebooking_reconciliation.rs` Z.149-221: use_transaction → create(out_row) → create(in_row) → create(batch+entry) → commit; Test `rebook_manual_writes_two_extra_hours_batch_and_entry_in_one_tx` gruen |
| 10 | REB-MANUAL-02 Beide Richtungen VolunteerToExtra + ExtraToVolunteer | VERIFIED | `RebookingDirection`-Enum mit beiden Varianten; `build_pair_payloads` implementiert beide; Test `rebook_manual_supports_reverse_direction` gruen |
| 11 | REB-MANUAL-03 Modal zeigt Preview vor Submit | PRESENT_BEHAVIOR_UNVERIFIED | Preview-Sektion in `manual_rebooking_modal.rs` implementiert (Z.125-135); SSR-Test prueft Struktur; Live-Reaktivitaet ist Browser-Behavior |
| 12 | HR-ALERT-01 Alert-Flag lebt in ShortEmployeeReportTO (Fat Backend) | VERIFIED | `ShortEmployeeReportTO.has_pending_rebooking` mit `#[serde(default)]`; Reporting-Chain befuellt es Predicate+HR-gated; FE-Banner nur wenn `has_pending_rebooking=true` |
| 13 | HR-ALERT-02 GET /rebooking-suggestions liefert volle IST+DANN-Payload inkl. voluntary_delta_before/after | VERIFIED | `rest/src/rebooking.rs:get_pending()` liefert `Vec<RebookingSuggestionTO>` mit allen 13 Feldern; `voluntary_delta_before/after` Backend-computed |
| 14 | HR-ALERT-03 approve+reject sind state-conditional (WHERE state='pending', affected-rows check) | VERIFIED | `service_impl/src/rebooking_reconciliation.rs` Z.344+395+427+445: `update_state_conditional` + `BatchAlreadyResolved` wenn affected==0; Tests `approve_suggestion_double_approve_race_yields_error` + `reject_after_approve_yields_error` gruen |
| 15 | HR-ALERT-04 Pending-Suggestion belegt UNIQUE-Slot via existing partial index | VERIFIED | Claim-on-Suggest: `suggest_for_week` schreibt `state=Pending` in `rebooking_batch` mit `(sales_person_id, iso_year, iso_week)` UNIQUE-Index (Phase 54); EntityAlreadyExists → 409 korrekt gemappt |
| 16 | VOL-ACCT-03 beide ExtraHours mit ExtraHoursSource::Rebooking + Filter aktiv in ReportingService | VERIFIED | `rebooking_reconciliation.rs` Z.101+113: source=Rebooking auf beiden Rows; `reporting.rs` Z.1214/1228/1676/1869/1970: Filter `.filter(|eh| eh.source != ExtraHoursSource::Rebooking)` an 4 Stellen; Static-Guard-Test `reporting_rs_still_filters_rebooking_marker_rows` gruen |
| 17 | Property-Test test_rebooking_neutrality_reporting (128 cases) praesent + gruen | VERIFIED | `service_impl/src/test/rebooking_roundtrip_neutrality.rs`: 4 Tests (2x proptest 128 cases + 1 Integration-Test + 1 Static-Guard), alle gruen |
| 18 | CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 12 | VERIFIED | `service_impl/src/billing_period_report.rs` Z.117: `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;` |

**Score:** 18/18 truths verified (2 present-but-behavior-unverified)

### Required Artifacts

| Artifact | Erwartet | Status | Details |
|----------|----------|--------|---------|
| `service/src/rebooking_reconciliation.rs` | Trait + alert_predicate + proposed_rebooking_hours + RebookingSuggestion | VERIFIED | Vollstaendig; alle Methoden + pure fns + structs vorhanden |
| `service_impl/src/rebooking_reconciliation.rs` | BL-Impl; rebook_manual/suggest/approve/reject in einer Tx | VERIFIED | 563 Zeilen; alle 5 Trait-Methoden implementiert; Single-Tx-Pattern durchgaengig |
| `rest-types/src/lib.rs` | 6 neue TOs inkl. voluntary_delta; ShortEmployeeReportTO-Erweiterung | VERIFIED | ManualRebookingRequestTO, RebookingBatchTO, RebookingSuggestionTO, RebookingBatchKindTO, RebookingBatchStateTO, RebookingDirectionTO alle vorhanden; has_pending_rebooking + pending_rebooking_id ergaenzt |
| `rest/src/rebooking.rs` | 4 Routes + 409-Mapping + ApiDoc | VERIFIED | 276 Zeilen; generate_manual_route() + generate_suggestions_route(); conflict_body() ohne SQL-Leak |
| `shifty_bin/src/main.rs` | DI-Wiring RebookingReconciliationService + Route-Registrierung | VERIFIED | Typ-Alias + RebookingReconciliationServiceDependencies + RestStateImpl-Feld + Konstruktion nach rebooking_batch_service |
| `shifty-dioxus/Dioxus.toml` | web.proxy fuer /rebooking + /rebooking-suggestions | VERIFIED | Z.113: `backend = "http://localhost:3000/rebooking"`, Z.116: `backend = "http://localhost:3000/rebooking-suggestions"` |
| `service_impl/src/test/rebooking_roundtrip_neutrality.rs` | Proptest-basierte Suite (128 cases) | VERIFIED | 405 Zeilen; proptest_config cases=128; 4 Tests alle gruen |
| `shifty-dioxus/src/component/rebooking_alert_banner.rs` | Inline-Banner (KEIN Dialog) | VERIFIED | 98 Zeilen; button-Element, kein dialog; SSR-Test `banner_renders_as_button_not_dialog` vorhanden |
| `shifty-dioxus/src/component/rebooking_suggestion_modal.rs` | IST/DANN-Tabelle + Approve/Reject + kein FE-Arithmetic | VERIFIED | 277 Zeilen; Backend-Felder direkt gerendert; Anti-Arithmetic-Guard per include_str-Test |
| `shifty-dioxus/src/component/manual_rebooking_modal.rs` | Year+Week number inputs + Direction Radio + Preview + Submit | VERIFIED | 328 Zeilen; iso_year/iso_week als number inputs; direction als Radio-Group; Preview-Sektion; Submit-Button HR-MANUAL-03 |
| `shifty-dioxus/src/loader.rs` | 4 Loader-Fns fuer 4 REST-Endpoints | VERIFIED | submit_manual_rebooking Z.1060, load_rebooking_suggestions_pending Z.1096, approve_rebooking_suggestion Z.1114, reject_rebooking_suggestion Z.1146 |
| `shifty-dioxus/src/state/rebooking.rs` | Thin From<&TO>-Mapper ohne Arithmetik | VERIFIED | 180 Zeilen; From-Impls 1:1 Feldkopie; kein Arithmetik-Operator |
| `shifty-dioxus/src/page/employees.rs` | HR-Path rendert Banner + verdrahtet SuggestionModal | VERIFIED | on_banner_click, open_suggestion_batch_id-Signal, suggestion_resource, RebookingSuggestionModal-Overlay; on_after_action triggert EMPLOYEES_LIST_REFRESH |
| `shifty-dioxus/src/page/employee_details.rs` | F3-Trigger HR-only in Header-Row, oeffnet ManualRebookingModal | VERIFIED | show_manual_rebooking_dialog-Signal; is_hr-Gate; ManualRebookingModal-Renderer |
| `docs/features/F14-rebooking.md` | F3+F5-Sektionen (EN) | VERIFIED | Sektionen 8 "Manual Rebooking (F3)" Z.328 + 9 "HR-Alert + Suggestion Modal (F5)" Z.414 vorhanden |
| `docs/features/F14-rebooking_de.md` | F3+F5-Sektionen (DE) | VERIFIED | Abschnitt 8 "Manuelle Umbuchung (F3)" Z.342 + 9 "HR-Alert + Vorschlags-Modal (F5)" Z.433 vorhanden |
| `docs/architecture/02-service-tiers.md` | RebookingReconciliationService in BL-Tabelle | VERIFIED | Z.58: Eintrag mit 7 Deps + Phase-55-Referenz; auch in `02-service-tiers_de.md` |
| `shifty-dioxus/src/i18n/mod.rs` | >= 8 neue i18n-Keys | VERIFIED | 24 Rebooking-Key-Varianten in mod.rs; in de.rs, en.rs, cs.rs alle uebersetzt |

### Key Link Verification

| Von | Nach | Via | Status | Details |
|-----|------|-----|--------|---------|
| `rest/src/rebooking.rs` | `service_impl/src/rebooking_reconciliation.rs` | `rest_state.rebooking_reconciliation_service()` calls rebook_manual/approve/reject | WIRED | RestStateDef.rebooking_reconciliation_service() in rest/src/lib.rs definiert; verwendet in allen 4 Handlern |
| `service_impl/src/reporting.rs` | `service_impl/src/rebooking_batch.rs` | predicate-gated find_pending_for_sales_person → has_pending_rebooking | WIRED | Z.620: alert_predicate-Check; Z.623-630: find_pending_for_sales_person + Feld-Zuweisung |
| `shifty-dioxus/Dioxus.toml` | `rest/src/rebooking.rs` | web.proxy /rebooking + /rebooking-suggestions → localhost:3000 | WIRED | Z.113+116 in Dioxus.toml; Z.695-698 in rest/src/lib.rs Router-Nesting |
| `shifty-dioxus/src/page/employees.rs` | `shifty-dioxus/src/component/rebooking_alert_banner.rs` | employees_list iteriert reports; rendert RebookingAlertBanner bei has_pending_rebooking=true | WIRED | `employees_list.rs` Z.154: `let banner_batch_id = if employee.has_pending_rebooking {...}`, Z.174: `RebookingAlertBanner {` |
| `shifty-dioxus/src/page/employee_details.rs` | `shifty-dioxus/src/component/manual_rebooking_modal.rs` | TopBar-Action oeffnet ManualRebookingModal mit current ISO year/week | WIRED | Z.60: `show_manual_rebooking_dialog`; Z.128: set(true); Z.213: `ManualRebookingModal {` |
| `shifty-dioxus/src/state/rebooking.rs` | `rest-types/src/lib.rs` | From<&RebookingSuggestionTO> Mapper 1:1 ohne Arithmetik | WIRED | `state/rebooking.rs` implementiert alle From-Traits; keine Minus-Operatoren auf IST-Feldern |

### Behavioral Spot-Checks

| Verhalten | Command | Ergebnis | Status |
|-----------|---------|----------|--------|
| alert_predicate Grenzfall-Tests | `cargo test -p service_impl predicate_truth_table` | 7 passed | PASS |
| rebook_manual atomare Transaktion | `cargo test -p service_impl rebook_manual_writes_two_extra_hours_batch_and_entry_in_one_tx` | ok | PASS |
| Approve Race → 409 | `cargo test -p service_impl approve_suggestion_double_approve_race_yields_error` | ok | PASS |
| Reject nach Approve → Fehler | `cargo test -p service_impl reject_after_approve_yields_error` | ok | PASS |
| Proptest Roundtrip-Neutralitaet (128 cases) | `cargo test -p service_impl rebooking_roundtrip_neutrality` | 4 tests ok (proptest 128 cases + integration) | PASS |
| Static Guard: Filter an >= 4 Stellen in reporting.rs | `cargo test -p service_impl reporting_rs_still_filters_rebooking_marker_rows` | ok (4 Treffer) | PASS |
| Workspace-Tests gesamt | `cargo test --workspace` | 787 + 66 + … = alle ok, 0 failed | PASS |
| Clippy -D warnings | `cargo clippy --workspace -- -D warnings` | Finished dev profile, keine Warnings | PASS |
| WASM-Build | `cargo build --target wasm32-unknown-unknown` (manifest: shifty-dioxus) | Finished, 0 errors | PASS |

### Requirements Coverage

| Requirement | Plan | Beschreibung | Status | Evidenz |
|-------------|------|-------------|--------|---------|
| REB-MANUAL-01 | 55-01, 55-02 | HR bucht manuell um; 2×ExtraHours + Batch + Entry in einer Tx | SATISFIED | rebook_manual implementiert; Test gruen |
| REB-MANUAL-02 | 55-01, 55-04 | Beide Richtungen VolunteerToExtra + ExtraToVolunteer | SATISFIED | RebookingDirection-Enum; Direction-Radio im Modal |
| REB-MANUAL-03 | 55-04 | Modal mit Vorschau vor Submit | SATISFIED (Verhalten human-gated) | Preview-Sektion implementiert; SSR-Test prueft Struktur; Reaktivitaet human |
| HR-ALERT-01 | 55-02, 55-05 | Warnzeile in /employees per has_pending_rebooking-Flag | SATISFIED | employees_list.rs + RebookingAlertBanner |
| HR-ALERT-02 | 55-02, 55-04 | Suggestion Modal mit IST+DANN inkl. voluntary_delta | SATISFIED | RebookingSuggestionTO alle Felder; Modal rendert Backend-computed Deltas |
| HR-ALERT-03 | 55-01, 55-02 | Approve + Reject; state-conditional UPDATE; Race → 409 | SATISFIED | update_state_conditional; BatchAlreadyResolved; Tests gruen |
| HR-ALERT-04 | 55-01 | Pending-Suggestion belegt UNIQUE-Slot (Claim-on-Suggest) | SATISFIED | suggest_for_week schreibt state=Pending; UNIQUE-Index aus Phase 54 |
| VOL-ACCT-03 | 55-01, 55-03 | Rebooking-Pair beeinflusst Read-Aggregate NICHT | SATISFIED | ExtraHoursSource::Rebooking-Filter an 4 Stellen in reporting.rs; proptest 128 cases + integration test gruen |

### Anti-Patterns

| Datei | Zeile | Muster | Schwere | Impact |
|-------|-------|--------|---------|--------|
| Keine gefunden | — | Keine TBD/FIXME/XXX/Placeholder in phasen-relevanten Dateien | — | — |

Spezifisch geprueft:
- `service/src/rebooking_reconciliation.rs`: 0 Treffer
- `service_impl/src/rebooking_reconciliation.rs`: 0 Treffer
- `rest/src/rebooking.rs`: 0 Treffer
- `shifty-dioxus/src/component/rebooking_alert_banner.rs`: 0 Treffer
- `shifty-dioxus/src/component/rebooking_suggestion_modal.rs`: 0 Treffer
- `shifty-dioxus/src/component/manual_rebooking_modal.rs`: 0 Treffer

### Prohibition-Check

| Prohibition | Status | Evidenz |
|-------------|--------|---------|
| MUST NOT expose undo/delete endpoint | VERIFIED — kein Verstos | `grep -n "undo\|delete\|revert" rest/src/rebooking.rs` → 0 Treffer |
| MUST NOT compute DANN/Delta im FE | VERIFIED — kein Verstos | Anti-Arithmetic-Guard per include_str-Test in rebooking_suggestion_modal; state/rebooking.rs 1:1 Feldkopie |
| MUST NOT create blocking confirmation dialog | VERIFIED — kein Verstos | rebooking_alert_banner.rs ist button-Element; Modals haben keinen zweiten Confirm-Dialog |
| MUST NOT leak SQL/internal error strings on UNIQUE collision | VERIFIED — kein Verstos | conflict_body() liefert nur i18n-Key-String, kein Error-Detail |
| MUST NOT gate banner visibility via FE logic | VERIFIED — kein Verstos | employees_list.rs Z.154: `if employee.has_pending_rebooking` — kein FE-Predicate-Recompute |

### Human Verification Required

#### 1. F5-Roundtrip: Banner verschwindet nach Approve

**Test:** Als HR-User mit einer Test-Person, die `balance <= -0.5h`, `voluntary_ist > 0` und `cap_active=true` hat: `/employees` aufrufen, Banner der Person anklicken, Suggestion-Modal oeffnen, Approve klicken.

**Expected:** Das Suggestion-Modal schliesst sich, die Employees-Liste aktualisiert sich automatisch, und der Banner der Person ist nicht mehr sichtbar (Backend liefert `has_pending_rebooking=false` nach Approve).

**Warum human:** D-55-07 State-Transition ist ein Full-UI-Roundtrip: Backend-Approve schreibt Pair-ExtraHours → Reporting-Chain berechnet Balance neu → `has_pending_rebooking=false` → EMPLOYEES_LIST_REFRESH-Signal → Dioxus-WASM-Neurender. Grep und Build-Checks beweisen nur Code-Praesenz und Handler-Wiring, nicht die korrekte Signal-Propagation in der laufenden WASM-App.

#### 2. F3-Preview-Sektion: Live-Aktualisierung beim Tippen

**Test:** ManualRebookingModal oeffnen (HR-User auf `/employees/{id}`), iso_week auf eine andere Woche stellen, direction auf "Bezahlt → Freiwillig" aendern, hours eingeben.

**Expected:** Preview-Text aktualisiert sich sofort nach jeder Eingabe und zeigt "Umbuchung: {hours} h — {direction_label} — KW {week}/{year}".

**Warum human:** Die Reaktivitaet der Dioxus-Signals (`year_signal`, `week_signal`, `direction`, `hours_signal`) und deren Einfluss auf den Preview-String ist nur im laufenden WASM-Browser testbar. Der SSR-Test prueft die Struktur, aber nicht die Echtzeit-Aktualisierung bei User-Input.

---

## Zusammenfassung

**Phase-Ziel erreicht?** JA — alle 18 Must-Haves sind im Codebase verankert und funktional verdrahtet. Die zwei als PRESENT_BEHAVIOR_UNVERIFIED eingestuften Truths (Banner-Refresh-Roundtrip + Preview-Reaktivitaet) sind Code-vollstaendig und korrekt verdrahtet, erfordern aber eine menschliche E2E-Pruefung im Browser.

**Kernergebnisse:**
- `RebookingReconciliationService` (BL-Tier) implementiert: `rebook_manual` schreibt atomar 2×ExtraHours + Batch + Entry; `approve_suggestion`/`reject_suggestion` mit state-conditional UPDATE + Race-Schutz; `list_pending_for_sales_person` hydriert Suggestions.
- REST-Layer: 4 Routen, 409-Mapping mit i18n-Keys ohne SQL-Leak, keine Undo/Delete-Endpoints.
- Reporting-Chain: `has_pending_rebooking` Predicate+HR-gated befuellt; `source!=Rebooking`-Filter an 4 Stellen aktiv.
- Frontend: 3 neue Komponenten (Banner/Suggestion-Modal/Manual-Modal), 4 Loader-Fns, 24 i18n-Keys in 3 Sprachen, Integration in employees.rs + employee_details.rs.
- Property-Test (`proptest` 128 cases + Integration-Test + Static-Guard) beweist VOL-ACCT-03-Neutralitaet.
- `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12` unveraendert.
- Keine Debt-Marker (TBD/FIXME/XXX) in phasen-relevanten Dateien.
- `cargo test --workspace`: alle Tests gruen. `cargo clippy --workspace -- -D warnings`: clean. WASM-Build: clean.

---

_Verifiziert: 2026-07-10_
_Verifier: Claude (gsd-verifier, claude-sonnet-4-6)_
