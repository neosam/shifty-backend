# Phase 55: Manuelle Umbuchung + HR-Alert-Modal (F3 + F5) — Context

**Gathered:** 2026-07-10
**Status:** Ready for planning
**Ships as:** Regulaerer v2.6-Release (nach v2.6.x-Patches auf `main`; endgueltiger Milestone-Close nach Phase 56).

<domain>
## Phase Boundary

Phase 55 implementiert die zwei User-facing-Rebooking-Trigger, die auf der in
Phase 54 gelegten Datenmodell-Basis (`rebooking_batch` DAO + Service,
`RebookingBatchKind::{Manual, HrSuggestion, AutoCron, AutoCronBackfill}`,
UNIQUE-Partial-Index `(sales_person_id, iso_year, iso_week)`) aufsetzen:

**F3 (Manuelle Umbuchung — REB-MANUAL-01..03):**
HR kann im Mitarbeiter-Jahresreport eine Umbuchung Freiwillig ↔ Bezahlt anlegen.
Konkret: neuer `RebookingReconciliationService` (BL) orchestriert in genau einer
Transaktion:
  1. zwei `ExtraHours`-Rows (`-N VolunteerWork`, `+N ExtraWork` — oder umgekehrt);
  2. ein `rebooking_batch` (`kind=Manual`, `state=Approved`) + ein
     `rebooking_batch_entry` mit `extra_hours_out_id` / `extra_hours_in_id` FKs;
  3. Marker (Phase-54-source-Spalte) auf beiden ExtraHours-Rows setzen, damit
     Rebooking-Neutralitaet in Read-Aggregaten (VOL-ACCT-03) haelt.

**F5 (HR-Alert + Vorschlags-Modal — HR-ALERT-01..04):**
Die Employee-Overview zeigt eine dauerhafte Warnzeile pro betroffenem
SalesPerson. Klick auf die Zeile oeffnet ein IST/DANN-Vorschlags-Modal;
HR approve → dieselbe Doppel-Eintrag-Semantik wie F3, aber
`kind=HrSuggestion, state=Approved`. HR reject → `state=Rejected` persistiert
(Audit; UNIQUE-Slot bleibt belegt bis naechste ISO-Woche).

**In-Scope:**
- `service/src/rebooking_reconciliation.rs` (neu; Trait).
- `service_impl/src/rebooking_reconciliation.rs` (neu; BL, orchestriert
  ExtraHours + RebookingBatchService + Marker in einer Tx; state-conditional
  UPDATE `WHERE state='pending' AND version=?` fuer Approve/Reject).
- `rest/src/rebooking.rs` (neu; `POST /rebooking/manual`,
  `GET /rebooking-suggestions`, `POST /rebooking-suggestions/{id}/approve`,
  `POST /rebooking-suggestions/{id}/reject`).
- `rest-types/src/rebooking.rs` (neu; `ManualRebookingRequestTO`,
  `RebookingSuggestionTO` mit IST/DANN-Feldern, `RebookingBatchTO`).
- `rest-types/src/employee_report.rs`: `ShortEmployeeReportTO` additiv um
  `has_pending_rebooking: bool` + `pending_rebooking_id: Option<Uuid>` erweitern
  (`#[serde(default)]`, Praezedenz VAA-04).
- `service_impl/src/reporting.rs` bzw. `short_employee_report.rs`: das neue
  DTO-Feld ausrechnen (Predicate mit `<=-0.5h`-Toleranz; siehe D-55-01).
- `shifty_bin/src/main.rs`: DI-Wiring `RebookingReconciliationServiceImpl` +
  Route-Registrierung.
- Frontend:
  - `shifty-dioxus/Dioxus.toml`: neue Proxy-Eintraege `/rebooking`,
    `/rebooking-suggestions` (MEMORY `feedback_dioxus_proxy_for_new_backend_endpoints`).
  - `shifty-dioxus/src/loader.rs`: 4 neue Loader-Funktionen.
  - `shifty-dioxus/src/state/rebooking.rs` (neu; thin `From<&…TO>`-Mapper).
  - `shifty-dioxus/src/component/rebooking_alert_banner.rs` (neu).
  - `shifty-dioxus/src/component/rebooking_suggestion_modal.rs` (neu; IST/DANN-Table).
  - `shifty-dioxus/src/component/manual_rebooking_modal.rs` (neu; Woche + Menge
    + Richtung + Vorschau).
  - `shifty-dioxus/src/page/employees.rs`: Banner-Einbindung im HR-Path.
  - `shifty-dioxus/src/page/employee_details.rs`: F3-Trigger in TopBar / Action-
    Menu (siehe D-55-05).
  - i18n en/de/cs: Modal-Labels, Approve/Reject, Banner-Text.
- Docs-Freshness (MEMORY `feedback_docs_always_current_no_followup`):
  `docs/features/F14-rebooking.md` + `_de.md` bekommen die F3/F5-Sektionen
  (Frontend-Trigger, State-Machine, IST/DANN-Semantik).
- Property-Test „Rebooking-Roundtrip-Neutralitaet" (VOL-ACCT-03-Guard):
  `balance_before == balance_after`, `voluntary_ist_before == _after`,
  `voluntary_soll_before == _after` — sichert dass die Doppel-Eintrag-Semantik
  Read-Aggregate nicht verschiebt.

**Out-of-Scope (bleibt Phase 56):**
- Kein Wochen-Cron (`VoluntaryRebookingScheduler`) — F4.
- Kein Stichtag-Toggle `voluntary_rebooking_auto_active_from` — F4.
- Kein Backfill-REST-Endpoint — F4.
- Kein Snapshot-Schema-Version-Bump (12 → 13) — Discuss in Phase-56.
- Kein Undo nach Approve/Reject (Anti-Feature REB-UNDO-01, defer v2.7+; siehe D-55-04).

</domain>

<decisions>
## Implementation Decisions

### D-55-01 (D-F5-01) — Alert-Threshold: `balance <= -0.5h`

**Entscheidung:** Der Predicate fuer die HR-Alert-Warnzeile ist:
`cap_planned_hours_to_expected = true` AND `balance <= -0.5h` AND
`voluntary_ist > 0`.

**Warum:** Float-Noise-Tolerance. Ein `balance < 0`-Predicate (strict) kann durch
Rundungsverzerrungen (0.0001-Defizite) fake-Alerts erzeugen. `<= -0.5h` deckt
sich mit der typischen UI-Granularitaet (Stunden mit einer Nachkommastelle) und
zeigt Alerts erst bei realen halben-Stunden-Luecken.

**Konsequenz:** Die Truth-Table fuer die pure-fn `alert_predicate(balance,
voluntary_ist, cap_active) -> bool` bekommt einen Grenzfall-Test:
`balance=-0.49 → false`, `balance=-0.5 → true`, `balance=-0.51 → true`.

### D-55-02 (D-F5-02) — Alert-Payload-Shape: DTO-additiv

**Entscheidung:** `ShortEmployeeReportTO` bekommt zwei zusaetzliche Felder:
- `has_pending_rebooking: bool` (mit `#[serde(default)]` fuer Wire-Kompat)
- `pending_rebooking_id: Option<Uuid>`

**Warum:** Weniger Round-Trips im FE (kein separater API-Call fuer eine
Sammel-Summary). Praezedenz VAA-04 (Phase 53) hat dasselbe Muster fuer
Absence-Marker verwendet. Non-HR-Redaktion bleibt bei API-Level (Felder sind
`Option` bzw. `false`-Default).

**Ripple:** `EmployeeReport`-Aggregations-Kette (bzw. `short_employee_report.rs`)
muss beim Aufbau des DTOs pruefen, ob ein `rebooking_batch(kind=HrSuggestion,
state=Pending, week=aktuelle_woche_der_person)` existiert. Das erfordert ein
neues DAO-Query in `rebooking_batch.rs` (Basic-Tier, `find_pending_for_sales_person`).

### D-55-03 (D-F5-03) — proposed_rebooking_hours = `min(|balance|, voluntary_ist)`

**Entscheidung:** Die vorgeschlagene Umbuchungs-Menge ist
`min(|balance|, voluntary_ist)`.

**Warum:** Praezedenz v2.5-VAA-cap-gated-Formel. Balance-Defizit ausgleichen,
aber hoechstens so viel wie tatsaechlich als Freiwilligenarbeit erfasst — sonst
wuerde man mehr rebooken als geleistet, was Neutralitaet bricht.

**Konsequenz:** Der `RebookingSuggestionTO.proposed_hours: f32` wird
Backend-berechnet; das FE macht keine Arithmetik (Fat Backend, Thin Client).
Test-Matrix: `balance=-10, voluntary_ist=5 → 5.0`,
`balance=-3, voluntary_ist=10 → 3.0`, `balance=0.4, voluntary_ist=5 → 0.0`
(kein Alert wegen D-55-01).

### D-55-04 (D-55-UNDO-01) — Kein Undo nach Approve/Reject (defer v2.7+)

**Entscheidung:** Weder Approve noch Reject kann rueckgaengig gemacht werden.
Approved-Batches bleiben persistent inkl. der Pair-ExtraHours-Rows.
Rejected-Batches bleiben persistent (Audit-Trail).

**Warum:** REQUIREMENTS.md v2.6 Anti-Feature REB-UNDO-01 explizit gepinnt. Undo
haette State-Machine-Komplexitaet (`undone`-State, Cascading-Delete auf
Pair-ExtraHours + Marker-Reset) und stellt eine eigene Fach-Diskussion dar.

**Explizit NICHT:** Approve → Reject-Uebergang oder umgekehrt; jeder Batch ist
ein one-shot.

### D-55-05 (D-55-F3-01) — F3 Woche-Wahl: HR waehlt im Modal

**Entscheidung:** Der F3-Manual-Rebooking-Modal hat ein Wochen-Feld
(`iso_year` + `iso_week`; Default: aktuelle KW). HR kann die Woche fuer die
Buchung frei waehlen — auch retrospektiv fuer alte Wochen.

**Warum:** F3 ist der HR-Korrektur-Pfad (im Gegensatz zu F4-Cron, der immer die
Vorwoche verarbeitet). HR braucht Flexibilitaet, Buchungen fuer die tatsaechlich
verursachende ISO-Woche anzulegen — nicht fuer die aktuelle. Beispiel: HR
bemerkt am 2026-08-15, dass in KW 20 ein Rebook noetig gewesen waere → kann
direkt KW 20 waehlen.

**Konsequenz:** UNIQUE-Slot-Kollisionen (`(sp, iso_year, iso_week)` schon
belegt) sind moeglich — Backend antwortet mit HTTP 409 + einer klaren
Fehlermeldung (i18n-Key `RebookingErrorSlotTaken`). Der FE-Modal zeigt die
Fehlermeldung inline; kein Auto-Overwrite.

### D-55-06 (D-55-F3-02) — F3 Button-Placement: TopBar / Action-Menu

**Entscheidung:** Der F3-Trigger sitzt in der TopBar bzw. dem Action-Menu der
`Employee-Details`-Seite (nicht direkt in der Voluntary-Stats-Zeile).

**Warum:** Explizite User-Wahl. Erhaelt die Voluntary-Stats-Zeile als reine
Lese-Anzeige (keine Buttons in der Row) und verhindert Overloading der neuen
Prozent-Zeile. TopBar-Menu-Item ist entdeckbar und stoert nicht den Report-Flow.

**Konsequenz:** Der Modal muss selbst die Richtungswahl anbieten
(`VolunteerWork → ExtraWork` bzw. `ExtraWork → VolunteerWork` als
Radio/Select), weil im TopBar-Trigger keine Delta-Vorzeichen-Ableitung
stattfindet.

### D-55-07 (D-55-F5-04) — Alert verschwindet nach Approve UND Reject

**Entscheidung:** Der Alert-Predicate erloescht in beiden Faellen:
- **Approve** schreibt die Pair-ExtraHours-Rows → `balance` wird neu berechnet
  → Predicate wird `false` → Alert weg.
- **Reject** persistiert `state=Rejected` → UNIQUE-Slot fuer
  `(sp, iso_year, iso_week)` bleibt belegt → das Backend liefert
  `has_pending_rebooking=false` fuer diese Woche (kein neues Suggestion in
  derselben Woche moeglich) → Alert weg. Rejected-Batch bleibt im Audit.

**Warum:** Konsistent mit REQUIREMENTS.md HR-ALERT-03 („sichtbar-vermerkt" =
im Audit, aber nicht im Alert-Banner). Vermeidet dass HR den gleichen Vorschlag
in derselben Woche erneut ablehnen muss.

**Konsequenz:** Das Backend-Predicate fuer `has_pending_rebooking` prueft
konkret nur `state=Pending` — sowohl `Approved` als auch `Rejected` beenden den
Alert. Die naechste Alert-Chance fuer dieselbe Person entsteht erst mit der
naechsten ISO-Woche.

</decisions>

<constraints>
## Constraints & Assumptions

- **Phase 54 hat die Datenmodell-Basis geliefert:** `rebooking_batch`,
  `rebooking_batch_entry`, `RebookingBatchKind::{Manual, HrSuggestion, …}`,
  `RebookingBatchState::{Pending, Approved, Rejected, SkippedLocked}`,
  UNIQUE-Partial-Index. Kein neues Datenmodell in Phase 55.
- **UNIQUE-Slot pro `(sales_person_id, iso_year, iso_week)` global ueber alle
  `kind`s.** F3 und F5 muessen bei Kollision (409) klar reagieren. F5 nutzt
  die Claim-on-Suggest-Semantik: der Alert erzeugt einen `pending`-Batch, der
  den Slot belegt bis Approve/Reject.
- **Rebooking-Neutralitaet in Read-Aggregaten (VOL-ACCT-03):** Property-Test
  muss beweisen, dass eine manuelle Umbuchung `balance`, `voluntary_ist`,
  `voluntary_soll` in den Read-Aggregaten NICHT verschiebt. Pair-ExtraHours
  sind negativ + positiv in denselben Betrag; Marker-Filter im Reporting
  neutralisiert sie fuer die Read-Sicht.
- **Fat Backend, Thin Client** (MEMORY `feedback_fat_backend_thin_client`):
  alle DANN-Werte im Modal + `proposed_hours` sind Backend-berechnet. FE
  spiegelt nur.
- **Kein Snapshot-Schema-Bump:** Phase 55 aendert keine `value_type`s auf
  `billing_period_sales_person`. `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12`
  bleibt unveraendert. (Der Bump ist Thema von Phase 56.)
- **jj+git-VCS + GSD-Auto-Commit** aktiv (`commit_docs: true`, yolo-mode).
- **Clippy hart** (MEMORY `feedback_clippy_gate`): `cargo clippy --workspace
  -- -D warnings` im jedem Test-Gate; nicht durch `cargo test` abgedeckt.
- **Dioxus.toml-Proxy fuer neue Backend-Endpoints** (MEMORY
  `feedback_dioxus_proxy_for_new_backend_endpoints`): `POST /rebooking/manual`,
  `GET /rebooking-suggestions`, `POST /rebooking-suggestions/{id}/approve`,
  `POST /rebooking-suggestions/{id}/reject` — alle brauchen Proxy-Eintraege.
- **State-conditional UPDATE fuer Race** (Pitfall 12): Approve/Reject-SQL nutzt
  `WHERE state='pending' AND version=?` mit affected-rows-Check; im
  Zwei-HR-Race gewinnt genau einer, der Zweite bekommt 409.

</constraints>

<open_questions>
## Open Questions for Planner

Keine grundsaetzlichen. Der Planner kalibriert:

1. **Sub-Plan-Anzahl** (Empfehlung 4–6 Plans):
   - 55-01: Backend-BL — `RebookingReconciliationService` + rebooking_batch-DAO-
     Query `find_pending_for_sales_person` + Predicate-pure-fn.
   - 55-02: REST-Endpoints (4 Routes) + rest-types-DTOs +
     `ShortEmployeeReportTO`-Erweiterung + Dioxus.toml-Proxy.
   - 55-03: Property-Test Rebooking-Neutralitaet (VOL-ACCT-03-Guard).
   - 55-04: FE-Komponenten (Banner, Suggestion-Modal, Manual-Modal) +
     Loader + State-Mapper + i18n.
   - 55-05: FE-Integration (page/employees Banner-Einbindung,
     page/employee_details TopBar-Trigger).
   - 55-06 (optional): Docs-Sync F14 EN+DE (koennte auch in 55-05 mit-wandern).

2. **F3-Modal-Vorschau-Inhalt:** Menge, Richtung, Woche, plus IST/DANN-Zeilen
   (analog F5-Suggestion-Modal) oder minimalistisch nur Menge+Richtung+Woche?
   Empfehlung: minimalistisch — der HR kennt den Kontext, keine
   Duplikat-DANN-Rechnung noetig.

3. **State-conditional UPDATE — Version-Uuid vs. State-only:** REB-Task-Text
   sagt „state-conditional UPDATE (`WHERE state='pending'`, affected-rows==1)".
   Reicht das? Oder braucht der UPDATE auch `AND version=?`
   (Optimistic-Locking)? Empfehlung: `state='pending'` reicht — der State ist
   der harte Diskriminator; Approve/Reject sind atomic.

</open_questions>

<requirements_touched>
## Requirements Impact

- **REB-MANUAL-01, REB-MANUAL-02, REB-MANUAL-03**: F3 vollstaendig geliefert.
- **HR-ALERT-01, HR-ALERT-02, HR-ALERT-03, HR-ALERT-04**: F5 vollstaendig
  geliefert (Alert-Banner, Modal, State-Machine, UNIQUE-Slot-Claim).
- **VOL-ACCT-03**: Property-Test-Guard `Rebooking-Roundtrip-Neutralitaet`.
- **REB-AUTO-*** (F4): NICHT beruehrt — bleibt Phase 56.

</requirements_touched>
