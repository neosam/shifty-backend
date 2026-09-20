---
gsd_state_version: 1.0
milestone: v2.7
milestone_name: — Personenverwaltung & Planungs-Soll
current_phase: 63
current_phase_name: personen-detailansicht-auspr-gungs-aktionen
status: executing
stopped_at: Completed 63-06-PLAN.md
last_updated: "2026-09-20T21:18:46.795Z"
progress:
  total_phases: 5
  completed_phases: 3
  total_plans: 31
  completed_plans: 27
  percent: 87
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (aktiver Milestone **v2.7**, Phasen 60–64; Phasen 58 + 59 weiter reserviert/vorbereitet, Backlog 999.1 erhalten)
- **Requirements**: `.planning/REQUIREMENTS.md` (18 REQ-IDs: PERS-01..06, PADM-01..08, PSOLL-01..04; 18/18 auf Phasen gemappt). Letzte abgeschlossene: `.planning/milestones/v2.6-REQUIREMENTS.md` (19/19 complete)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v2.6-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions); Frontend in `shifty-dioxus/CLAUDE.md` + `.planning/codebase/frontend/`
- **Last shipped/closed**: **v2.6 Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter** (shipped 2026-07-31, deployed; Planungs-Archiv 2026-09-15; Phasen 54, 54.5, 55, 56, 57 — 29 Pläne, 19/19 Requirements, Audit `passed`, `override_closeout`)
- **Current milestone**: **v2.7 Personenverwaltung & Planungs-Soll** (in Arbeit; Roadmap erstellt 2026-09-16, Phasen 60–64; 1/5 Phasen abgeschlossen).
- **Current focus**: **Phase 62 abgeschlossen (7/7 Plaene, Verifikation `passed`, 9/9 must-haves) — Phase 63 ist die naechste, noch ungeplant.** `/user_management/` zeigt eine durchsuchbare Liste statt zweier Tabs, gespeist von `GET /person-directory` mit Admin-Gate im Service. WR-02 ist mit Phase 62 gefixt (D-62-05). **Zwei Punkte fuer Phase 63:** (1) D-63-03 in der ROADMAP — der neue Anlege-Dialog (SC-2) muss die beiden heutigen Knoepfe „Neuen Verkaeufer erstellen“ / „Neuen Benutzer hinzufuegen“ **ersetzen**, nicht ergaenzen (UAT-Befund Phase 62: seit dem Tab-Merge stehen sie kontextlos nebeneinander). (2) Das Auspraegungs-Badge wurde im UAT von 62-07 auf Produktentscheidung **entfernt** (Farbkollision mit dem Bezahlt/Freiwillig-Chip); `PersonKindTO` bleibt und steuert Aktions-Sichtbarkeit und Bearbeiten-Link — die drei i18n-Keys stehen fuer die Detailansicht bereit. Offen bis zum Deploy: CR-02 Pre-Flight-Query (siehe **v2.7 Deploy-Gates**). Die Entwuerfe **Phase 58** (Werktags-Stundenverteilung, Snapshot-Bump 13→14) und **Phase 59** (E2E-Playwright) bleiben ausserhalb von v2.7; ihre Nummern sind reserviert.
- **Snapshot-Schema-Version**: aktuell **13** (Bump 12→13 in Phase 57, D-57-10). **v2.7 bumpt nicht** — `person` ist rein administrativ, das Planungs-Soll ein reines Anzeigefeld. Phase 58 (nicht in v2.7) würde auf 14 heben.

## Current Position

Phase: 63 (personen-detailansicht-auspr-gungs-aktionen) — EXECUTING
Plan: 7 of 11 (63-01, 63-02, 63-03, 63-04, 63-05, 63-06 abgeschlossen)
Status: Welle 1 und der erste Plan von Welle 2 abgeschlossen. 63-01 hat `PermissionDao::create_user`/`delete_user` auf die
geteilte Transaktion gehoben — SC-2-Atomaritaet und der D-63-01-Orphan-Cleanup sind
damit ueberhaupt erst konstruierbar. `service::PermissionService` bleibt eingefroren.
63-02 hat die DAO-Primitive nachgezogen (`PersonDao::delete`/`has_live_facets`/
`find_by_sales_person`/`find_by_user`, `PersonDirectoryDao::find_directory_entry`) und
D-63-01 als Lesart (b) festgeschrieben.
63-03 hat die i18n-Schicht vorgezogen: 32 Keys fuer Detailansicht, Anlege-Dialog,
Kandidaten-Picker und die drei Bestaetigungen stehen in de/en/cs bereit, abgesichert
durch einen Key-Mengen- und einen `{name}`-Platzhalter-Test. Die zwei D-63-08-Saetze
stehen zeichengleich in `de.rs`; es gibt genau einen Anlege-Einstieg
(`Key::CreatePersonEntry`, D-63-03) und keinen Sammel-Loesch-Text (D-63-05).
63-04 hat den Lesepfad der Detailansicht gebaut: vier Transport-Objekte
(`CreatePersonEntryTO`, `CreatedPersonEntryTO`, `PersonFacetTO`, `AddPersonFacetTO`),
das Business-Logic-Trait `PersonDetailService` mit drei admin-gegateten Lesemethoden und
`PersonDetailServiceImpl`. Der Zuschnitt des Aggregats ist entschieden (D-63-04-A:
Kern-Aggregat plus Nachladepfade), und `entry_from_row` ist `pub(crate)`, damit
Auspraegung und Anzeigename genau eine Definition behalten. Der Service ist noch **nicht**
in `shifty_bin/src/main.rs` verdrahtet und hat noch keinen REST-Handler — beides ist
Sache von 63-06.
63-05 hat die drei Auspraegungs-Mutationen nachgezogen: `create_combined` (PADM-02/SC-2,
eine Transaktion ueber Benutzer UND Verkaeufer, leerer Auswahl-Satz serverseitig
abgewiesen), `add_facet` (PADM-03/SC-3, gibt die RESULTIERENDE `person_id` zurueck — beim
Zuschalten einer Sales Person gewinnt sie nach D-61-03 und der Eintrag wechselt seine
Identitaet), `remove_facet` (PADM-07/SC-5, Hard Delete beim Login vs. Soft Delete beim
Verkaeufer, plus D-63-01-Orphan-Cleanup in derselben Transaktion) und
`get_detail_by_sales_person` (SC-4). Alle drei Testsuiten sind per Mutation gegen
Vakuitaet abgesichert. **Fuer 63-06 wichtig:** `PersonDetailServiceDeps` hat jetzt ACHT
Dependencies — `SalesPersonDao` ist dazugekommen.

Was Phase 62 geliefert hat:

- 62-01 `PersonDirectoryDao` mit Drei-Wege-Outer-Join; WR-02 (Soft-Delete in der
  `ON`-Klausel) geschlossen.

- 62-02 `PersonEntryTO`/`PersonKindTO` in `rest-types`, sechs Beschriftungen in de/en/cs.
- 62-03 `PersonDirectoryService` — Admin-Gate als erste Anweisung (D-62-08),
  Ausprägungs-Klassifikation, `display_name`-Ableitung, case-insensitive Sortierung.

- 62-04 Frontend: eine Liste statt Tabs, Aggregat-Ladepfad statt N+1, Suche über
  Anzeige- und Benutzername, Component-Level-Admin-Guard.

- 62-05 `GET /person-directory` mit OpenAPI, DI nach dem Basic-Block,
  `Dioxus.toml`-Proxy im selben Commit; Integrationstests 200/403.

- 62-06 Docs-Freshness-Gate: F01 und 02-service-tiers je EN+DE, Runtime-Graph.
- 62-07 Laufzeit-Verifikation + UAT mit dem Nutzer; zwei Änderungen ausgelöst
  (Badge entfernt, Spaltenüberschrift „Verkäufer" → „Name") und D-63-03 gepinnt.

## Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260710-r7v | /absences: Neue-Abwesenheit-Modal preselectet die gefilterte Person statt des eingeloggten Users | 2026-07-10 | 275ecd1 | _(inline, kein Verzeichnis — kleiner atomarer Commit)_ |
| 260710-p8k | Voluntary Erfüllungsgrad (`ist_per_soll_pct = ist/soll*100`) im Employee-Detail-Report; FE-Zeile bleibt bei `soll≈0` unsichtbar | 2026-07-10 | e2aee9b | _(inline, kein Verzeichnis — kleiner atomarer Commit)_ |
| 260702-jql | Special-Day Duplikat-Hinweis nach Create ausblenden, erst bei Feld-Änderung wieder | 2026-07-02 | b9d270b | [260702-jql-special-day-duplikat-hinweis-nach-create](./quick/260702-jql-special-day-duplikat-hinweis-nach-create/) |

## Deferred Items

### v2.7 Deploy-Gates (vor dem Milestone-Deploy abarbeiten)

| Item | Herkunft | Status |
|------|----------|--------|
| **CR-02 — Pre-Flight-Query gegen die Produktions-DB** vor dem v2.7-Deploy: `SELECT COUNT(*) FROM sales_person_user spu WHERE NOT EXISTS (SELECT 1 FROM sales_person sp WHERE sp.id = spu.sales_person_id) OR NOT EXISTS (SELECT 1 FROM user u WHERE u.name = spu.user_id);` — bei Treffer > 0 vorher bereinigen oder Datei-Backup als Rollback bereithalten. | Phase 60, `60-VERIFICATION.md` (human_verification) + `60-UAT.md` Test 1 (`blocked`) | offen — bewusst auf den Milestone-Check vertagt (User, 2026-09-17) |

Hintergrund: Die in 60-04 verschaerfte Selbst-Assertion bricht die Migration bei
verwaisten `sales_person_user`-Zeilen mit einer nicht-diagnostischen
`CHECK constraint failed`-Meldung ab und blockiert damit den Serverstart. Ueber die
laufende Anwendung sind solche Zeilen nicht erzeugbar (`PRAGMA foreign_keys = ON`,
`sales_person` wird nur soft-geloescht); Restrisiko sind Altbestaende aus manuellen
Eingriffen, Restores oder aelteren Tools. Kein Test kann die reale Datenlage pruefen.

### Doku-Drift-Befunde aus dem Phase-61-Sweep (2026-09-17, Plan 61-10)

Alle drei wurden beim Sweep fuer WR-06 gefunden, liegen aber ausserhalb des
Gap-Scopes dieses Plans (`files_modified: [AUTHENTICATION.md]`). Sie sind
bewusst **nicht** stillschweigend mitrepariert worden.

| Item | Fundstelle | Status |
|------|------------|--------|
| **arc42 D3, zweiter Drift-Befund zu `AUTHENTICATION.md`**: die Datei beschreibt weiter einen `MockContext`-Context-Typ hinter `#[cfg(feature = "mock_auth")]`. Im Code existiert `MockContext` nicht mehr (`grep -rn "MockContext" --include="*.rs" .` → 0 Treffer); `rest/src/session.rs:12` fuehrt `pub type Context = Option<Arc<str>>` ohne `cfg`, und `service_impl/src/lib.rs:64` verwendet denselben Typ. | `docs/arc42/11-risks-and-technical-debt.md` Zeile D3 | offen — ausserhalb WR-06 (dort geht es um die Verknuepfungstabelle, nicht um den Context-Typ) |
| **`openspec/specs/dev-seed/spec.md:79`** behauptet, der Dev-Seed-Reset loesche Zeilen aus der in Phase 61 aufgeloesten 1:1-Verknuepfungstabelle. `BasicDaoImpl::clear_all` tut das seit Phase 61 nicht mehr (die Tabelle existiert nicht). | `openspec/specs/dev-seed/spec.md` | offen — der `openspec/`-Baum ist laut arc42 D3 ohnehin als stillgelegtes Artefakt zum Loeschen/Markieren vorgemerkt |
| **ROADMAP-Progress-Tabelle**: die Zeile fuer Phase 60 steht weiter auf `0/? | Not started`, obwohl die Phase abgeschlossen ist (STATE: `completed_phases: 1`). Plan 61-10 hat nur die Zeile fuer Phase 61 angefasst. | `.planning/ROADMAP.md` §Progress Table | offen — vorbestehende Drift, nicht durch diese Welle verursacht |

### Beim v2.6-Close acknowledged (2026-09-15)

19 offene Artefakte wurden beim Milestone-Close gescannt und als deferred
dokumentiert (`closeout_type: override_closeout`, 0 aus einem früheren Close
fortgeschrieben).

> **Technischer Hinweis:** Die installierte `gsd-tools`-Version kennt den
> `audit-open acknowledge`-Writer noch nicht (`Unknown audit-open subcommand`).
> Die maschinelle Suppression konnte daher nicht gesetzt werden — diese Tabelle
> ist die Disclosure. Praktische Folge: dieselben Items erscheinen beim nächsten
> `audit-open`-Scan erneut, wie schon bei den Closes v2.2 und v2.5.

| Kategorie | Item | Status | Deferred At | Milestone |
|-----------|------|--------|-------------|-----------|
| debug_sessions | carryover-absence-vs-report | awaiting_human_verify | 2026-09-15 | v2.6 |
| verification_gaps | 54-data-model-voluntary-stats / 54-VERIFICATION.md | human_needed — FE-Rendering-Invariante (HR-Zeile sichtbar, Non-HR nicht) nur im Browser prüfbar | 2026-09-15 | v2.6 |
| verification_gaps | 56-wochen-cron-rollout-backfill-f4 / 56-VERIFICATION.md | human_needed — Cron-Feuern Mo 03:15 nur zur Laufzeit beobachtbar | 2026-09-15 | v2.6 |
| uat_gaps | 54-data-model-voluntary-stats / 54-UAT.md | diagnosed — 0 pending scenarios | 2026-09-15 | v2.6 |
| uat_gaps | 56-wochen-cron-rollout-backfill-f4 / 56-UAT.md | partial — 0 pending scenarios | 2026-09-15 | v2.6 |
| context_questions | 54.5-voluntary-soll-absence-fix / 54.5-CONTEXT.md | 3 offene Planungsfragen (beantwortet durch die Ausführung, nie zurückgepflegt) | 2026-09-15 | v2.6 |
| context_questions | 55-manual-rebooking-hr-alert / 55-CONTEXT.md | 3 offene Planungsfragen (dito) | 2026-09-15 | v2.6 |
| quick_tasks | 260516-g63-stunden-eingaben-muessen-auf-zwei-nachkommastellen | unknown | 2026-09-15 | v2.6 |
| quick_tasks | 260516-gv8-delete-button-fuer-arbeitsvertraege-employee | unknown | 2026-09-15 | v2.6 |
| quick_tasks | 260612-nlv-absence-page-stundenbasierte-marker | unknown | 2026-09-15 | v2.6 |
| quick_tasks | 260612-o7t-absencespage-sales-person-filter | unknown | 2026-09-15 | v2.6 |
| quick_tasks | 260613-jxe-abwesenheitsseite-inaktive-sales-persons | unknown | 2026-09-15 | v2.6 |
| quick_tasks | 260624-qgk-stunden-ohne-arbeitsvertrag-als-ehrenamt | unknown | 2026-09-15 | v2.6 |
| quick_tasks | 260624-ujk-mitarbeiter-report-geleistete-stunden | unknown | 2026-09-15 | v2.6 |
| todos | 2026-05-05-warnung-eintrag-ausserhalb-vertragszeiten.md | (presence-only) | 2026-09-15 | v2.6 |
| todos | 2026-05-07-review-frontend-list-user-invitations-silent-empty-fallback.md | (presence-only) | 2026-09-15 | v2.6 |
| todos | 2026-05-07-review-silent-renew-iframe-in-index-html.md | (presence-only) | 2026-09-15 | v2.6 |
| todos | 2026-05-08-admin-rolle-bekommt-alle-privilegien.md | (presence-only) | 2026-09-15 | v2.6 |
| todos | 2026-05-08-cutover-ui-admin-feature.md | (presence-only) | 2026-09-15 | v2.6 |
| todos | 2026-06-22-committed-voluntary-capacity-jahresansicht.md | (presence-only) | 2026-09-15 | v2.6 |
| todos | 2026-06-29-end-to-end-integrationstests-mit-ferngesteuertem-browser.md | (presence-only) — Origin-Todo für die vorbereitete Phase 59 | 2026-09-15 | v2.6 |
| todos | 2026-07-01-optionale-stundenverteilung-pro-werktag-fuer-salesperson.md | (presence-only) — Origin-Todo für die vorbereitete Phase 58 | 2026-09-15 | v2.6 |
| todos | 2026-07-01-urlaub-beantragen-mit-genehmigung.md | (presence-only) | 2026-09-15 | v2.6 |
| todos | 2026-07-03-schmale-admin-rolle-mit-user-impersonation.md | (presence-only) | 2026-09-15 | v2.6 |

**Milestone-eigene Items** sind nur die vier Phasen-Artefakte (2 verification_gaps,
2 uat_gaps) plus die zwei CONTEXT-Fragen. Debug-Session, Quick-Tasks und Todos
sind historischer Ballast aus v1.4–v2.1, beim v2.2- und v2.5-Close bereits
acknowledged.

### Historisch (v1.4–v2.5)

Erneut acknowledged + deferred beim **v2.2-Milestone-Close am 2026-07-03**
(User-Entscheidung override_closeout — Pre-Close-Audit meldete 13 offene Items: 1 Debug-
Session `awaiting_human_verify`, 7 Quick-Tasks, 5+ Pending Todos aus Mai/Juni 2026 —
alles historischer Ballast aus v1.4–v2.1, keines v2.2-spezifisch). Davor beim v1.7+v1.8-
Close am 2026-06-29; Ursprung v1.5/v1.4:

| Kategorie | Item | Status | Notiz |
|-----------|------|--------|-------|
| debug | carryover-absence-vs-report | code-fixed, awaiting_human_verify | v1.5: Code-Fix drin (`vacation_balance.rs:225` → `year-1`, Tests grün) + Phase-18-Mock-Lock; nur Browser-Bestätigung offen, kein offener Code |
| debug | working-hours-wrong-employee | resolved (obsolet) | gefixt: Signal-Mirror `current_employee_id` + Regressionstest `FROZEN_CAPTURE` in `employee_details.rs` |
| human_uat | Phase 16: visuelle Drei-Farben-Chart-Stapelung (v1.4) | pending | nicht test-automatisierbar (SSR pinnt keine Pixel) |
| human_uat | Phase 16: Czech-Übersetzungsqualität (v1.4) | pending | A3 MEDIUM-confidence, manuelle Sprachprüfung |
| quick_task | 7 Quick-Tasks (Mai/Juni, Status „missing"/unknown) | deferred | historischer Absence-Ballast, vor v1.4 |
| todo | 5+ pending Todos (ab Mai 2026) | deferred | historisch (booking-log 500er, admin-rolle-privilegien u.a.) |
| tech_debt | Nyquist-VALIDATION Phasen 14/15/17 + v1.5-FE-Phasen unvollständig | deferred | Discovery-only, optional `/gsd-validate-phase` |
| human_uat | Phase 24 #1: Inline-Block-Platzierung (v1.6) | deferred | acknowledged beim v1.6-Close 2026-06-28; 409-Meldung rendert global unter WeekView statt an Slot-Zelle; nicht-blockierend, Backend-409-Logik durch 4 Unit-Tests abgedeckt |
| human_uat | Phase 27: Browser-Smoke Freiwilligen-Selector (v1.8) | ✅ resolved 2026-06-29 | Live-HR-Smoke bestätigt: Modal + FilterBar splitten Angestellte (Anna/Max M/Max S/Sarah) vs Freiwillige (Tom Bauer); inaktive ausgeblendet; „All people" erhalten. (Hinweis: is_paid ist für Nicht-HR backend-redacted — by-design, Selektoren sind HR-gated.) |
| human_uat | Phase 28: Browser-Smoke HR-Offset-Roundtrip (v1.8) | ✅ resolved 2026-06-29 | Live-HR-Smoke bestätigt: HR-StatBox „calculated 15 + Offset [n]"; Offset 3 gesetzt → effektiv 18, remaining 33, persistiert (Backend offset_days=3). Smoke fand+fixte Dev-Proxy-Gap (Dioxus.toml fehlte /vacation-entitlement-offset → FE-Save 405) via fix(28). |
| human_uat | Phase 30: Browser-Smoke schnelles Wochen-Klicken (v1.9) | deferred (optional) | acknowledged beim v1.9-Close 2026-06-29; nicht pixel-/timing-automatisierbar; strukturelle Korrektheit voll verifiziert (pure Prädikat-Tests + Source-verifizierte Synchron-vor-Dispatch-Ordering + alle 4 Summary-Loader gegated). |
| human_uat | Phase 32: Browser-Smoke Impersonation-Roundtrip (v1.9) | deferred (optional) | acknowledged beim v1.9-Close 2026-06-29; Live-Roundtrip (Start→Banner→Reload-Persistenz→auditierter Write→Stop→Teardown); strukturell voll verifiziert (17/17 must-haves, 3 BE-Integration-Tests SC3/SC5/P10, Banner non-closable per SSR-Test). |
| human_uat | Phase 33: visuelle Special-Days-UI-Smokes (v1.10) | deferred (optional) | 5 rein-visuelle Items (WASM-Datepicker-Signal D-25-06, Add-Button-Disabled-Rendering, Jahres-Liste-Badges, Dropdown-onclick-Roundtrip, ShortDay-Inline-Prompt). Browser-e2e 2026-06-30 durchgeführt: Backend-CRUD voll verifiziert (create 201/dup 422/shortday 422/for-year 200/delete 204), shiftplanner-Gating bestätigt, **create-Pfad-Bug gefunden+gefixt** (FE POSTete /special-days/ → Axum-0.8-404 → fix /special-days). Visuelle Dioxus-Interaktion nicht zuverlässig automatisierbar → manueller Smoke via /gsd-verify-work 33. |
| tech_debt | Phase 35: WR-02 FE-Borrow `save_slot_edit` über `.await` (v1.10) | deferred | Code-Review WARNING, **pre-existing** (nicht von Phase 35 eingeführt): `save_slot_edit` hält den `SLOT_EDIT_STORE`-Write-Borrow über `.await` → already-borrowed-Panic-Risiko. Als Todo erfasst (`.planning/todos/pending/2026-06-30-fe-save-slot-edit-borrow-across-await.md`). Harte Phase-35-Constraints (Atomarität, keine Doppelzählung) unberührt. |
| human_uat | Phase 36: SDF-02 Browser-Smoke Anlegen-Button-Re-Enable (v1.11) | deferred (optional) | User akzeptierte 2026-07-01 strukturelle Verifikation (SSR/Component-Tests) als ausreichend; das *live* WASM-Button-Re-Enable nach Create in Settings-Card-3 (controlled `<select>` → DOM → `sd_form_valid`) ist D-25-06-Klasse (programmatisches Signal-Setzen unzuverlässig) → optionaler Browser-Smoke. Auch SDF-01-UI (Schichtplan-Dropdown Feiertag↔Kurzer-Tag Roundtrip) live nur browser-verifizierbar; Backend voll test-abgedeckt (HTTP 422→201). |
| tech_debt | Phase 36: WR-01 Special-Day replace nicht transaktional / kein UNIQUE-Index (v1.11) | deferred | Code-Review WARNING (teils pre-existing): create-vs-replace ist read-then-write ohne Transaktion, und es gibt keinen UNIQUE-Index auf `(year, calendar_week, day_of_week)` → bei Nebenläufigkeit/vorhandenen Dup-Aktivzeilen partielles Replace. Sauberer Fix bräuchte **Migration** → out-of-scope für v1.11 („keine Migration"). Realrisiko gering (Single-Shiftplanner). |
| tech_debt | Phase 36: WR-02 stale „already exists"-Hinweis in Settings-Special-Days-Card (v1.11) | deferred | Code-Review WARNING: nach SDF-01 überschreibt der Anlegen-Button jetzt statt zu blockieren, aber der Hinweis-Text (`settings.rs:691`) sagt weiter „already exists" (wie blockierend). Fix bräuchte i18n-Copy-Änderung → out-of-scope für v1.11 (Success-Kriterium „i18n unberührt"). Kandidat für kleinen Folge-Fix. |
| human_uat | Phase 37: MOD-01 Browser-Drag-Smoke (v1.11) | deferred (optional) | Live-Browser-Drag (mousedown im Panel → mouseup auf Backdrop lässt Modal offen) — D-25-06-Klasse, per D-10 bewusst strukturell statt browser-automatisiert verifiziert (5 `BackdropPress`-Unit-Tests grün). User-Präzedenz Phase 36 (2026-07-01): strukturelle Verifikation als ausreichend akzeptiert, Browser-Smoke optional. |
| tech_debt | Phase 37: WR-02 toter `cancel_label`-Conditional `contract_modal.rs:77-81` (v1.11) | deferred | Code-Review WARNING, **pre-existing** (nicht im Phase-37-Diff): `if read_only {…} else {…}` mit identischen Branches (beide `Key::Cancel`) — vermutlich verlorenes „Close"-Label im Read-Only-Modal. Unabhängig von MOD-01/MOD-02. Kandidat für kleinen Folge-Fix (braucht `Close`-i18n-Key-Entscheidung). |
| tech_debt | Phase 37: WR-01 Panel-`stop_propagation` blockt document-level Outside-Click (v1.11) | deferred | Code-Review WARNING: das für den MOD-01-Fix nötige Panel-`onmousedown`-`stop_propagation` verhindert, dass künftige generische Dialog-Kinder document-level Outside-Click-Detection nutzen. Inhärent zum gelockten D-01-Ansatz, aktuell kein Opfer. |

## v2.5 Tech-Debt Backlog (bei v2.6-Milestone-Close-Audit prüfen)

Aus `milestones/v2.5-MILESTONE-AUDIT.md`, nicht-blockierend für v2.5, Kandidaten für Backlog / spätere Milestones:

- SDF-03-Semantik-Cleanup: `SpecialDayService::get_by_iso_year` als Follow-up, um die 55 `special_day.get_by_week`-Calls in `assemble_weeks` auf konstant 2 zu reduzieren. Nicht latenz-relevant.
- DB-Indices: `booking(year, calendar_week)`, `extra_hours(date_time)`, `working_hours(from_year, to_year)` — RESEARCH-Q3 v2.5. Kein Bottleneck mehr nach Follow-up #2.
- F07-Doku-Nachschärfung: neue Pure-Helper `derive_hours_for_week_pure` + `build_derived_holiday_map_for_week_pure` (aus Follow-Ups #1 + #2) in `docs/features/F07-reporting-balance.{md,_de.md}` dokumentieren.

## Shipped Milestones

### v2.5 — Weekly-Overview Performance & Freiwilligen-Abwesenheiten (shipped 2026-07-06)

- **Geliefert:** Phasen 52–53 (8 Pläne + 3 Follow-Ups). Performance: `get_weekly_summary` konsumiert Jahres-Aggregate; drei Chain-Preloads auf Year-Scope gehoben; End-to-End-Median 2.33s → 0.12s (19.4×). Follow-up #3 Jahresübergangs-Fix (3 neue `_iso_year`-Bulk-Methoden, 16 Regressions-Gates). Freiwilligen-Sichtbarkeit (VAA): `sales_person_absences` als Union bezahlt + Freiwillig; Backend liefert Name + cap-gated `committed_voluntary` fertig im DTO; FE reiner Union-Merge.
- **Snapshot-Schema-Version:** bleibt **12**.
- **Verifikation:** Phase 52 VERIFIED PASS (5/5 SC, 1 Override in Follow-up #3 formal aufgelöst); Phase 53 VERIFIED PASS (10/10 must-haves inkl. INT-Sightcheck); Milestone-Audit `passed` (9/9 Requirements, 2/2 E2E-Flows, 4 non-blocking Tech-Debt-Items).
- **Closeout:** verified_closeout.
- **Archiv:** `milestones/v2.5-ROADMAP.md`, `milestones/v2.5-REQUIREMENTS.md`, `milestones/v2.5-MILESTONE-AUDIT.md`, `milestones/v2.5-phases/`.

### v2.4 — Kurzer-Tag-Slot-Kürzung (shipped 2026-07-05)

- **Geliefert:** Phase 51 (8 Pläne). Dynamische View-Layer-Kürzung an ShortDays. Kanonische pure Value-Methode `Slot::clip_to` auf `service::slot::Slot` (D-51-01). Vier BE-Aggregat-Ketten (Chain A' BlockService / Chain B ShiftplanWeek+PDF / Chain C BookingInformation / Chain D ShiftplanReport Rust-Layer-Refactor) gaten am `shortday_gate::should_clip`. Admin-Stichtag `shortday_slot_clipping_active_from` (ISO-Datum via `ToggleService`, Präzedenz HCFG-02) schützt historische Balance-Views. DTO `ShiftplanSlotTO.effective_to` (Wrapper, D-51-09) trägt geclippten Wert ans FE + PDF; `SlotTO` bleibt bidirektional roh. FE-Loader collapst `effective_to` in `state::Slot.to`, damit WeekView + PDF-Renderer automatisch geclippte Werte sehen. Fat Backend, Thin Client (D-51-02): grep-verifiziert kein `clip_to`-Call im FE. Admin-Settings Card 2b + 6 neue i18n-Keys de/en/cs.
- **Snapshot-Schema-Version:** bleibt **12** (kein Bump — rein additive Live-Berechnung, grep-verifiziert). Migration: additiver Toggle-Seed `20260704000001_seed-shortday-slot-clipping-toggle.sql`. Keine neue Cargo-Dep.
- **Verifikation:** Phase 51 VERIFIED PASS (6/6 must-haves, `behavior_unverified: 0`); Milestone-Audit `passed` (6/6 Requirements, 6/6 Cross-Phase Wirings, 6/6 E2E-Flows, 2 non-blocking Warnings W1+W2). Gates: Backend `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` grün; FE `cargo build --target wasm32-unknown-unknown` + FE-Clippy `-D warnings` grün.
- **Bonus-Bugfixes (pre-existing, nicht in Requirements):** Filter-statt-Clip in `shiftplan.rs` + `booking_information.rs` (ShortDay-Slots wurden ganz ausgefiltert statt am Cutoff gekürzt); `/60.0`-SQL-Bug in alten Chain-D-SUM-Queries via Delete-Branch beim Rust-Layer-Refactor; `ToggleService`-Full-Context-Bypass für internal-Aggregate-Konsumenten als Gap-Closure gefixt (`f654613`, `7f21bd4`, `1b863e8`, `5aee47e`, `9cbe151`).
- **Closeout:** override_closeout (Audit `passed`; historische Deferred-Items acknowledged; W1 P07-SUMMARY-Doc-Drift + W2 latent `From<&SlotTO> for Slot` ohne `effective_to`-Awareness als non-blocking Warnings deferred). Kein git tag hier — SemVer-Tag via `/release-version`, `git.create_tag=false`.
- **Archiv:** `milestones/v2.4-ROADMAP.md`, `milestones/v2.4-REQUIREMENTS.md`, `milestones/v2.4-MILESTONE-AUDIT.md`, `milestones/v2.4-phases/`.

**Deferred (acknowledged at close):**

- **W1 (cosmetic):** P07-SUMMARY-Doc-Drift — SUMMARY nennt pdf_render-Fns die nicht existieren; Runtime korrekt via loader-Collapse.
- **W2 (latent):** `shifty-dioxus/src/state/shiftplan.rs:199-214` `From<&SlotTO> for Slot` kopiert raw `slot.to` ohne `effective_to`-Awareness; heute nur im Slot-Edit-Form-Pfad (raw korrekt); Empfehlung Doc-Warnkommentar oder Rename zu `Slot::from_edit_to(SlotTO)`.
- Pre-existing 4 `dbg!`-Makros in `service_impl/src/block.rs:71-91` (kein Clippy-Verstoß, in VERIFICATION.md dokumentiert).

### v2.2 — Aufräumen, WebDAV-Export & Wochentag-Muster (shipped 2026-07-03)

- **Geliefert:** Phasen 43–48 (16 Pläne, 14 Tasks). SDF-03/04/05 Special-Days-Feintuning (Kalenderjahr-Loader-Fix, Duplikat-Hinweis Replace-Copy in de/en/cs, Feiertag↔Kurzer-Tag atomarer Roundtrip-Test). BUG-01/02/03 Frontend-Korrektheit (`save_slot_edit` Snapshot-vor-`.await` + 6 Regressionstests; `ShiftyError::InvitationParse` + Inline-Banner für sichtbaren Parse-Fehler; durable Grep-Invariant für `BackdropPress`-Adoption). HYG-03 shifty-dioxus Warnings (177→0, FE-Clippy-Gate `-D warnings` erstmals scharfgestellt). HYG-04/HYG-05/IMP-05 Backend-Hygiene (Edit-structure-i18n, OpenAPI-Reflection-Content-Type-Drift-Guard über 120 Operationen, i18n-Impersonation-Test grün). RPT-01/02/03 Wochentag-Anwesenheits-Muster (pure fn `weekday_attendance_distribution` ersetzt v2.1-AVG, DTO-Shape-Umbau, Formatter „Mo: 8 (80 %) · …", 9 neue i18n-Keys de/en/cs). EXP-01/02/03 Nextcloud-PDF-Export via WebDAV (`printpdf` deterministisches Layout, `reqwest_dav` mit 3× Backoff 2s/4s/8s, `tokio-cron-scheduler`, admin-gated Settings-Card mit „Jetzt exportieren", Single-Row-`pdf_export_config`-Tabelle).
- **Snapshot-Schema-Version:** bleibt **12** (kein Bump — RPT = reines Read-Aggregat, EXP berührt keinen `BillingPeriodValueType`; grep-verifiziert). Migration in Phase 48 (`pdf_export_config` Single-Row + Seed). Neue Deps: `printpdf`, `reqwest_dav`, `tokio-cron-scheduler`.
- **Verifikation:** alle 6 Phasen VERIFIED passed (43: 8/8; 44: 3/3; 45: 5/5; 46: 3/3; 47: 11/11; 48: 30/30 inkl. E2E `boot_trigger_reload_flow` gegen wiremock-WebDAV); Milestone-Audit `passed` (16/16 Requirements, Integration clean, 3/3 E2E-Flows). Gates: Backend `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` grün; FE `cargo build --target wasm32-unknown-unknown` warnungsfrei + `cargo test -p shifty-dioxus` **787 grün** (inkl. der zuvor gebrochenen `i18n_impersonation_keys_match_german_reference` via IMP-05) + FE-`cargo clippy -p shifty-dioxus -- -D warnings` erstmals grün.
- **Closeout:** override_closeout (Audit `passed`; Pre-Close-Audit-13-Carry-over-Items acknowledged, keines v2.2-spezifisch). Kein git tag hier — SemVer-Tag via `/release-version`, `git.create_tag=false`.
- **Archiv:** `milestones/v2.2-ROADMAP.md`, `milestones/v2.2-REQUIREMENTS.md`, `milestones/v2.2-MILESTONE-AUDIT.md`.

**Deferred (acknowledged at close):**

- Pre-existing `doc_lazy_continuation`-Clippy-Warning in `service_impl/src/test/shiftplan_edit_lock.rs:6` (Origin Phase 40 / v2.1): fires nur mit `--all-targets`, nicht Teil des Standard-Clippy-Gates. Non-blocking.
- Phase 45 Scope-Caveat: 13 pre-existing `#[allow(dead_code)]`/`#[allow(non_snake_case)]` ohne reason-Kommentar — Clippy `-D warnings` trotzdem grün.
- Phase 48 Deviation: PDF-Determinismus-Test normalisiert `/ID`-Array-Bytes test-seitig (dokumentiert im SUMMARY) — Payload/Layout byte-gleich, nur printpdf-internal `/ID`-Fingerprint runspezifisch.

### v2.1 — Schichtplan- & Reporting-Erweiterungen (shipped 2026-07-02)

- **Geliefert:** Phasen 39–42 (14 Pläne). WST-01/02/05 KW-Status Grundlage (neue `week_status`-Tabelle, `WeekStatusService` Basic-Tier, shiftplanner-gated CRUD, farbkodiertes Badge + Dropdown für alle Rollen). WST-03/04 Wochen-Sperre (TOCTOU-sicheres `assert_week_not_locked` in allen 6 Schreibpfaden in-Transaktion, HTTP 423, Shiftplanner-Bypass, neue `delete_booking`-Methode + REST-Re-Routing, FE read-only + 423-Banner). AVG-01/02/03 Ø-Anwesenheit flexibler Mitarbeiter (pure fn `average_hours_per_attendance_day`, HR-gated Endpoint, FE-Sektion mit Leerzustand, i18n de/en/cs). SDF-01 Special-Days-„Anlegen"-Button-Fix (Option-2-Reset-Removal, reine Validitäts-Fns unit-getestet).
- **Snapshot-Schema-Version:** bleibt 12 (kein Bump — AVG = reines Read-Aggregat, kein neuer `BillingPeriodValueType`; grep-verifiziert). Migration nur in Phase 39 (`week_status`-Tabelle, partial UNIQUE). Keine neuen Dependencies.
- **Verifikation:** alle 4 Phasen passed (39: 20/20; 40: 11/11, inkl. **CRITICAL CR-01-Bugfix (Privileg-Mismatch) via Code-Review gefunden + gefixt** + Regressionstest; 41: 13/13; 42: 7/7); Milestone-Audit `passed` (9/9 Requirements, Integration clean, 3/3 Flows, Nyquist compliant). Gates: Backend `cargo test --workspace` (569+64+weitere, 0 Failures) + `cargo clippy --workspace -- -D warnings` clean; FE `cargo build --target wasm32-unknown-unknown` + `cargo test -p shifty-dioxus` (752) grün.
- **Closeout:** override_closeout (Audit `passed`; 3 optionale D-25-06-Browser-Smokes + WR-03 (akzeptiert) + PRÄ-v2.1 i18n-Impersonation-Carry-over acknowledged). Kein git tag.
- **Archiv:** `milestones/v2.1-ROADMAP.md`, `milestones/v2.1-REQUIREMENTS.md`, `milestones/v2.1-MILESTONE-AUDIT.md`.

**Deferred (acknowledged at close):**

- 3 optionale D-25-06-Browser-Smokes (Phase 40: +/- Buttons weg in Locked-Woche; Phase 41: Ø-Anwesenheits-Zahl im HR-Report; Phase 42: Button bleibt aktiv nach Create) — strukturell verifiziert.
- WR-03 (akzeptiert): `is_dynamic`-Filter ohne Perioden-Bezug — konsistent mit bestehendem Muster.
- PRÄ-v2.1 Carry-over: `i18n_impersonation_keys_match_german_reference` (Phase 37-02/v1.11, '🥸 Agieren' vs Referenz 'Als diese Person agieren') — Produkt-Copy-Entscheidung offen; Todo: `.planning/todos/pending/2026-07-02-i18n-impersonation-key-test-mismatch.md`.

### v1.11 — Stabilisierung & UX-Politur (shipped 2026-07-01)

- **Geliefert:** Phasen 36–38 (6 Pläne). SDF-01/02 Special-Days-Bugfixes (atomarer in-place Special-Day-Replace Feiertag↔Kurzer-Tag statt HTTP-422-Duplicate; controlled `SelectInput`-`value`-Prop + Settings-Card-3-Bindung → „Anlegen"-Button re-enabled). MOD-01/02 Modal-UX (zentrale drag-sichere `BackdropPress`-Backdrop-Logik in `dialog.rs` + `absence_convert_modal.rs`; pro-Feld-Help-Texte im Arbeitsvertrag-Modal, 6 neue `*Help`-Keys de/en/cs). HYG-01/02 Frontend-Build-Hygiene (`shifty-dioxus` `cargo build` warnungsfrei — 14 auto-fix, 2 deprecated→`parse_borrowed`, ~34 Dead-Code gelöscht / 11 begründete `#[allow]`; Backend-Clippy-Gate grün).
- **Kein Snapshot-Bump** (bleibt 12), **keine** Migration, **keine** neuen Deps.
- **Verifikation:** alle 3 Phasen VERIFIED passed (36: 14/15, 1 optionaler Browser-Smoke deferred; 37: 14/14; 38: 9/9, 4 Gates live grün); Milestone-Audit `passed` (6/6 Requirements, Integration clean, 4/4 Flows). Pro Phase Code-Review (0 Blocker). Gates: Backend `cargo test --workspace` (528+64) + clippy grün; FE `cargo build` 0 Warnings + `cargo test -p shifty-dioxus` (727) + WASM grün.
- **Closeout:** override_closeout (Audit `passed`; 13 historische Carry-over-Items acknowledged). Deferred: SDF-02/MOD-01 Browser-Smokes (D-25-06, strukturell verifiziert), WR-01/36 (Migration out-of-scope), WR-02/36 + WR-02/37 (i18n-Copy out-of-scope), pre-existing Impersonation-Test + dioxus-Clippy.
- **Archiv:** `milestones/v1.11-ROADMAP.md`, `milestones/v1.11-REQUIREMENTS.md`, `milestones/v1.11-MILESTONE-AUDIT.md`.

### v1.10 — Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz (shipped 2026-06-30)

- **Geliefert:** Phasen 33–35 (8 Pläne). SPD-01..04 Special-Days-UI shiftplanner-gated auf zwei Flächen (Schichtplan-Wochenraster Per-Tag-Dropdown + Settings-Kalenderdatum-Picker + Jahres-Liste mit abgeleitetem Kontext) gegen bestehende REST-CRUD + neuen `GET /special-days/for-year/{year}`-Read; HSP-01..04 `get_week` 4. Injektionspunkt (`holiday_derived_gated` reduziert nur `expected_hours`/`holiday_hours`, Kapazitätsbänder per Regressions-Guard geschützt, derive-on-read via `build_derived_holiday_map`, identisch zum Stundenkonto); SWO-01..04 Slot-Einzelwochen-Ausnahme via 3-Segment-Split+Re-Merge (`modify_slot_single_week`, atomar/Rollback, Booking-Re-Point ohne Doppelzählung, Gate `shiftplan.edit`) + UI-Wahl „nur diese Woche"/„ab dieser Woche".
- **Kein Snapshot-Bump** (bleibt 12, grep-verifiziert in Phase 34), **keine** Migration, **keine** neuen Deps.
- **Verifikation:** alle 3 Phasen VERIFIED (33 passed inkl. Backend-CRUD-Browser-Smoke + create-Pfad-Bugfix; 34 6/6 must-haves, re-verified nach CR-01-Gap-Closure; 35 4/4 must-haves, **SWO-01 live-browser-bestätigt** 3-Segment 4/5/4); Milestone-Audit `passed` (12/12 Requirements, Integration clean, 2/2 E2E-Flows). Gates: `cargo test --workspace` (526 Tests) + `cargo clippy --workspace -- -D warnings` grün; FE WASM-Build + `cargo test -p shifty-dioxus` grün. **Regression-Gate während Phase 34 fand+fixte den committeten Cross-Phase-Blocker aus Phase 33** (`find_by_year`-Query fehlte im `.sqlx`-Offline-Cache → CI wäre rot gewesen).
- **Closeout:** override_closeout (Audit `passed`; Carry-over Deferred Items acknowledged). Deferred: **5 rein-visuelle Phase-33-Smokes** (WASM-Datepicker-Signal D-25-06, Add-Button-Disabled-Rendering, Jahres-Liste-Badges, Dropdown-onclick-Roundtrip, ShortDay-Inline-Prompt) + **WR-02** (pre-existing FE-Borrow-Todo: `save_slot_edit` hält Write-Borrow über `.await`, nicht von Phase 35 eingeführt).
- **Archiv:** `milestones/v1.10-ROADMAP.md`, `milestones/v1.10-REQUIREMENTS.md`, `v1.10-MILESTONE-AUDIT.md`.

### v1.9 — Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation (shipped 2026-06-29)

- **Geliefert:** Phasen 29–32 (6 Pläne). VAC-01 Urlaubsbalken `(used+planned)/total` (Überzug = Farb-Signal); SHP-02 geteilter `(year,week)`-Staleness-Guard (`week_guard.rs`) über alle 4 Summary-Loader; SHP-01 proaktive „Nicht Verfügbar"-Markierung eigener/ausgewählter Absence-Tage (`absence_marker.rs`, kategorie-treu zur Buchungs-Warnung, reused Phase-30-Guard); IMP-01..04 Admin-Impersonation-FE (nicht-schließbarer Banner, reload-persistent, Users-Tab-Einstieg, Store-Teardown) + zentrale Audit-Middleware (`RealUser`, kein Privilege-Leak).
- **Kein Snapshot-Bump** (bleibt 12), **keine** Migration, **keine** neuen Deps, **keine** `Authentication<Context>`-Signatur-Änderung.
- **Verifikation:** alle 4 Phasen VERIFIED (29: 3/3, 30: 5/5 struktur., 31: 7/7, 32: 17/17); Milestone-Audit `passed` (7/7 Requirements, 4/4 Integration + E2E). Code-Review pro Phase, alle Findings adressiert (u.a. P30 WR-01 4. Loader `working_hours_mini`, P32 3 Audit-WR). Gates: `cargo clippy --workspace -- -D warnings` + `cargo test --workspace` grün; FE WASM-Build + 705 FE-Tests grün.
- **Closeout:** override_closeout (Audit `passed`; Carry-over Deferred Items acknowledged; 2 optionale Browser-Smokes deferred). **Code uncommitted** (jj manueller Commit durch User).
- **Archiv:** `milestones/v1.9-ROADMAP.md`, `milestones/v1.9-REQUIREMENTS.md`, `milestones/v1.9-MILESTONE-AUDIT.md`.

### v1.8 — Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX) (shipped 2026-06-29)

- **Geliefert:** Phasen 27–28 (5 Pläne). Freiwillige in Abwesenheits-Selektoren auswählbar (gruppiert Modal+FilterBar, gemeinsamer Helfer, VOL-SEL-01); signed Urlaubsanspruch-Offset pro Person+Jahr (Delta, HR-gated CRUD, API-level Hiding, FE-Inline-Editor, VAC-OFFSET-01) + Off-by-one-Fix.
- **Snapshot-Bump 11→12** (`BillingPeriodValueType::VacationEntitlement`-Computation geändert).
- **Verifikation:** beide Phasen VERIFIED inkl. Live-HR-Browser-Smokes (`behavior_unverified: 0`); Audit `passed` (2/2 Requirements, 100% Integration, 2/2 Flows). 2 Bonus-Bugfixes im Smoke (Dioxus.toml-Proxy, AbsenceModal-Close).
- **Closeout:** override_closeout (formaler Audit `passed`; Carry-over Deferred Items acknowledged).
- **Archiv:** `milestones/v1.8-ROADMAP.md`, `milestones/v1.8-REQUIREMENTS.md`, `milestones/v1.8-MILESTONE-AUDIT.md`.

### v1.7 — Automatische Feiertage & Freiwilligen-Abwesenheit (shipped 2026-06-29; verified 2026-06-28)

- **Geliefert:** Phasen 25–26 (7 Pläne). Feiertags-Auto-Anrechnung derive-on-read (identisch zu manuellem ExtraHours(Holiday)) ab konfigurierbarem Stichtag; Freiwilligen-Abwesenheit reduziert committed-Zusage in der Jahresansicht (Feiertage nicht — Asymmetrie); bidirektionale Deep-Links /absences ↔ Report. 10/10 Requirements (HOL/VFA/HCFG/HSNAP/NAV).
- **Snapshot-Bump 10→11** (Holiday-Computation/Input-Set geändert).
- **Verifikation:** beide Phasen complete & verified (Automatik-Gates grün); Browser-Verifikation der NAV-Links als Carry-over deferred.
- **Closeout:** override_closeout — Close war nach „verified 2026-06-28" liegengeblieben, am 2026-06-29 gemeinsam mit v1.8 nachgeholt.
- **Archiv:** `milestones/v1.7-ROADMAP.md`, `milestones/v1.7-REQUIREMENTS.md`.

### v1.6 — Paid-Capacity-Durchsetzung & Konfiguration (shipped 2026-06-27)

- **Geliefert:** Phase 24 (5 Pläne) — Paid-Capacity-Limit von Soft-Hinweis zu global konfigurierbarem Hard/Soft-Limit. Admin-Toggle `paid_limit_hard_enforcement` (Default weich, keine Regression); pre-persist Hard-Block in `ShiftplanEditService` (Shiftplanner-Bypass, nur bezahlte zählen, `prospective > max`); `ServiceError::PaidLimitExceeded` → HTTP 409 + lokalisierte Inline-Meldung; admin-gated `/settings/`-Seite; persistente Overage-Warn-Sektion für alle Rollen; Permission-Gate-Fix `HR ∨ self` → `Shiftplanner ∨ self` (D-24-04). i18n De/En/Cs.
- **Kein Snapshot-Bump** (Baseline bleibt 10): keine persistierte `BillingPeriodValueType`-Computation berührt.
- **Verifikation:** 7/7 must-haves verified; Human-UAT 3/4 PASS; 2 Bugs während UAT gefunden+gefixt
- **Closeout:** override_closeout — kein formaler Milestone-Audit
- **Archiv:** `milestones/v1.6-ROADMAP.md`

## Accumulated Context (carry forward)

### Constraints In Force

- **VCS**: Repository wird mit `jj` (co-located mit git) verwaltet. GSD-Auto-Commit ist **aktiv** (`commit_docs: true`, yolo-Mode) — GSD committet Phasen-Arbeit automatisch (Executor/Docs via git, von jj im co-located Repo automatisch importiert). Verifiziert 2026-06-30: Phasen 33+34 wurden so committet, Arbeitskopie sauber.
- **NixOS**: Tools wie `sqlx-cli` via `nix develop` (NICHT `nix-shell`, shell.nix kaputt). DB-Befehle: `sqlx database reset` ist DESTRUCTIVE → für additive Migrationen `sqlx migrate run`.
- **Snapshot Versioning**: `CURRENT_SNAPSHOT_SCHEMA_VERSION` (aktuell **12**) MUSS gebumpt werden, sobald sich `value_type`-Berechnung oder -Input-Set ändert. **v2.6 Phase 56 discuss-phase pinnt die Entscheidung** (Divergenz-2 in Research SUMMARY): Ist die neue F4-Cron-Rebooking-ExtraHours-Quelle ein „Input-Set-Change" im Sinne CLAUDE.md-Klausel? Beweislast beim „Nein"-Zweig = Straddling-Golden-Snapshot, byte-identisch.
- **Clippy-Gate**: `cargo clippy --workspace -- -D warnings` ist Pflicht-Gate bei jedem Commit — `cargo test` allein reicht nicht.
- **Service-Tier-Konvention**: Basic Services konsumieren nur DAO/Permission/Transaction; Business-Logic Services kombinieren Aggregate. `RebookingBatchService` = Basic; `RebookingReconciliationService`, `VoluntaryStatsService`, `VoluntaryRebookingScheduler` = BL.
- **Fat Backend, Thin Client**: alle Berechnungen (F1-Ist, F2-Soll, F4-Excess, F5-DANN-Werte) im Backend; FE zeigt vorbereitete DTOs.
- **Multi-Sprache (i18n)**: Alle benutzersichtbaren Texte in en/de/cs. F1..F5-neue-Labels betreffen neue Texte.
- **Docs-Freshness-Gate (v2.6)**: Neu `docs/features/F14-rebooking.md` + `_de.md`; Update F07 + F08 + `02-service-tiers.md` + `03-data-model.md` (beide Sprachen synchron, gleicher Commit wie Code-Diff — MEMORY `feedback_docs_always_current_no_followup.md`).

### Roadmap Evolution

- Backlog-Phase 999.1 „Breaking/Major Dependency-Migration" angelegt (2026-06-28): off-theme, eskaliert aus Quick-Task 260627-vgo. Bleibt separat verfügbar via `/gsd-plan-phase 999.1`.
- **Milestone v2.6 + Phasen 54–56 angelegt (2026-07-06):** 17 Requirements → 3 Phasen (ARCHITECTURE-C Vertikal-Slice-Baseline aus Research SUMMARY-Divergenz-1 gewählt; Rationale in ROADMAP.md dokumentiert). Phase 54 = Data-Model + F1+F2 (5 Requirements: VOL-STAT/VOL-ACCT). Phase 55 = F3+F5 (7 Requirements: REB-MANUAL, HR-ALERT). Phase 56 = F4+Backfill (5 Requirements: REB-AUTO). Zwei offene Discuss-Decisions bleiben in den Phasen: Snapshot-Bump 12→13 in P56 (REB-AUTO-05), Denominator-Definition D-F1-01 + Mid-Week-Vertragswechsel D-F2-01 in P54.
- **Milestone v2.7 + Phasen 60–64 angelegt (2026-09-16):** 18 Requirements → 5 Phasen.
  Research schlug vier Phasen vor (60–63); die Roadmap teilt den UI-Block zusätzlich in
  **Liste (62)** und **Detail + Ausprägungs-Aktionen (63)**, weil er sonst 10+ Pläne in
  einer Phase gebündelt hätte. Phase 60 = Datenmodell + Backfill (PERS-01/02/03/04/06,
  Backend-only). Phase 61 = `PersonService` + Auflösung `sales_person_user` inkl.
  Umschreibung der Selbst-Zugriffs-Prüfung `find_sales_person_by_user_id` (PERS-05,
  Backend-only; bewusst eigene Phase trotz nur eines Requirements — einzige
  sicherheitsrelevante Umschreibung des Milestones). Phase 62 = Aggregat-Service +
  merged Liste (PADM-01/06/08). Phase 63 = Detailansicht + Ausprägungs-Mutationen
  (PADM-02/03/04/05/07). Phase 64 = Planungs-Soll (PSOLL-01..04), architektonisch
  unabhängig, bewusst ans Ende sequenziert (irreversibles Migrationsrisiko zuerst;
  eigene Test-Matrix für die zwei `ShortEmployeeReport`-Konstruktionsstellen).
  **Nummern 58 und 59 bleiben reserviert.** Gesperrt in der Roadmap: kein
  Snapshot-Bump (bleibt 13), keine Umschlüsselung von `sales_person.id`/`user.name`,
  abschnittsweises Rollen-Gating ist PADM-F-02 (Future) — die Seite ist als Ganzes
  admin-gated.

- **Phase 57 angelegt (2026-07-31): F14-Rebooking-Wirksamkeit — Requirements-Korrektur + Balance-Ausgleich.** Analyse auf User-Nachfrage ergab: der in 54–56 gebaute Übertrag freiwillig→bezahlt ist auf der Leseseite ein vollständiger No-Op — der `source == Manual`-Filter verwirft beide Beine des `(−h, +h)`-Paars, die Balance bleibt unverändert (empirisch: `−53 → −53`), das Minus wandert per Carryover ins Folgejahr. **Ursache liegt in der Planung, nicht in der Implementierung:** kein v2.6-Requirement operationalisiert das Milestone-Goal „Ausgleich"; VOL-ACCT-03 fordert wörtlich das Gegenteil („balance-neutral"). Der Widerspruch stand sichtbar in `55-CONTEXT.md` (D-55-07 „balance wird neu berechnet" vs. Constraint „NICHT verschiebt") und wurde in `55-RESEARCH.md:443` als bestätigungsbedürftig markiert, dann planintern zugunsten von VOL-ACCT-03 aufgelöst. Die Gates prüften gegen die Spec, nicht gegen den Zweck: das 55-UAT prüft „Banner verschwindet" — genau den Mechanismus (belegter UNIQUE-Slot, D-55-07), der das Versagen kaschiert. P57 korrigiert **Spec vor Code**: VOL-ACCT-03 neu fassen + neues Ausgleichs-REQ → D-55-07 entkoppeln → Filter kategorie-spezifisch → Balance-Deckel im Auto-Cron → Neutralitäts-Tests invertieren → Snapshot-Bump 12→13 (D-01 fällt mit dem Fix).

## Session Continuity

**Last session:** 2026-09-20T21:18:46.787Z
**Stopped at:** Completed 63-06-PLAN.md
**Resume file:** None

**To resume work in a new session:**

1. Read `.planning/STATE.md` (this file)
2. Read `.planning/ROADMAP.md` (Sektion „Milestone v2.7 … (Active)", Phasen 60–64 mit Details)
3. Read `.planning/REQUIREMENTS.md` (18 REQ-IDs + gefüllte Traceability)
4. Read `.planning/research/SUMMARY.md` + `ARCHITECTURE.md` + `PITFALLS.md`
5. Read `.planning/seeds/personenverwaltung-zusammenlegen.md` (gesperrte Entscheidungen)
6. Read `.planning/PROJECT.md` (Current Milestone: v2.7-Sektion)

**Aktueller Stand:** Milestone v2.7 in Planning. ROADMAP.md fertig (Phasen 60–64,
18/18 Requirements gemappt). Snapshot-Version bleibt **13** — kein Bump in v2.7.

**Next command**: `/gsd-discuss-phase 60` (Person-Datenmodell + Backfill). Offene
Discuss-Punkte pro Phase stehen in der ROADMAP-Phase-Detail-Sektion (D-62-01/02,
D-63-01/02, D-64-01/02/03).

---

*State updated: 2026-09-16 — Roadmap v2.7 created (Phasen 60–64, 18/18 Requirements gemappt; UI-Block gegen den Research-Vorschlag in Liste + Detail geteilt; PSOLL als unabhängige Schluss-Phase; Nummern 58/59 reserviert; kein Snapshot-Bump, bleibt 13; Docs-Freshness-Check nach jeder Phase statt erst beim Close).*

## Operator Next Steps

- Review `.planning/ROADMAP.md` (Sektion „Milestone v2.7 … (Active)")
- Dann `/gsd-discuss-phase 60` — Person-Datenmodell + Backfill

## Decisions

- [Phase ?]: D-33-02 enforced: Card-3 shiftplanner-gated with inner has_privilege guard, page admin gate unchanged
- [Phase ?]: Rule 3 deviation: reqwest switched from native-tls to rustls-tls for host test compilation without openssl
- [Phase ?]: weekday_sub_headers Vec on WeekView; spawn wrapper for Signal::set in DropdownEntry Fn closures
- [Phase ?]: D-35-01: modify_slot_single_week als separate Methode (Option B), 3-Segment-Split, atomar, Permission-Gate shiftplan.edit
- [Phase ?]: [D-35-02] single_week: bool Default false — 100% Backward-Compat; bestehender save_slot-Pfad bleibt unverändert erreichbar
- [Phase ?]: [D-35-02] borrow-sicheres Auslesen von store-Feldern vor await in save_slot_edit verhindert Borrow-Konflikte
- [Phase ?]: Controlled SelectInput value prop fix
- [Phase ?]: BackdropPress Copy struct with press_backdrop/press_panel/release encodes MOD-01 drag-safe close rule as a pure unit-tested state machine
- [Phase ?]: 37-02: Help sibling-span pattern (text-small font-normal text-ink-muted); From/To excluded; CommittedVoluntaryHelp scoped to if show_committed block
- [Phase ?]: WeekStatusService is Basic-tier: DAO+Permission+Clock+Uuid+Transaction only, no domain-service dep (D-39-12)
- [Phase ?]: Unset == row absence, mapped as 4th WeekStatus variant, never persisted (D-39-04)
- [Phase ?]: 39-04: FE fresh-fetch KW-Status store (D-39-06) — Set->PUT->GET, kein optimistisches Signal (T-39-05)
- [Phase ?]: 39-04: WeekStatus FE-Enum 4 Varianten Default=Unset + i18n de/en/cs 4x3 (WST-05)
- [Phase ?]: 39-05: should_show_badge pure-fn is the tested source of truth for the Unset->hidden badge rule (D-39-05); WeekStatusDropdown on DropdownTrigger, no controlled select (D-39-06), with an Unset reset entry (D-39-07)
- [Phase ?]: 40-01: WeekStatusService dep uses full <Context, Transaction> bound — Open Question 1 resolved, no reduction needed
- [Phase ?]: 40-01: assert_week_not_locked is pass-through scaffold (reads status, always Ok); enforcement+bypass deferred to 40-03, delete_booking handler re-route to 40-04
- [Phase 41]: 41-01: average_hours_per_attendance_day is a SEPARATE pure fn from A-22-1 (own struct EmployeeAttendanceStatistics, input &[WorkingHoursDay]); A-22-1 byte-for-byte unchanged
- [Phase 41]: 41-01: attendance day = DISTINCT date (BTreeSet<time::Date>) with ≥1 work-category entry (Shiftplan|ExtraWork|VolunteerWork, hours>0); Absence+Custom excluded by filter; <2 days → None (D-AVG-06); no snapshot bump (stays 12, D-AVG-08)
- [Phase 41]: 41-02: ReportingService::get_employee_attendance_statistics — HR_PRIVILEGE ist die ERSTE await-Operation (D-AVG-05, kein Datenabruf vor Auth); is_dynamic-Filter server-seitig (nicht-flexibler MA → Ok(None)); aggregiert via get_report_for_employee über by_week[*].days mit until_week-Clamp (D-AVG-04); reines Read-Aggregat, Snapshot bleibt 12 (D-AVG-08)
- [Phase 42]: 42-01: SDF-01 FE-only — Option 2 (D-42-01): 3 Post-Create-Feld-Resets entfernt, Felder bleiben gefüllt → Anlegen-Button bleibt aktiv; sd_year.set + sd_resource.restart bleiben (D-42-02). Validitäts-Prädikat + Retention-Policy in reine Fns extrahiert + unit-getestet (is_special_day_form_valid, SpecialDayForm/special_day_form_after_create; D-42-05). Duplikat-Hinweis nicht an disabled gekoppelt (D-42-03), sd_save_result unverändert (D-42-04). SSR-Mount-Test begründet übersprungen (D-42-06 Fall B — SettingsPage ohne Live-Harness nicht mountbar). WASM-Build warnungsfrei; kein Backend, kein Snapshot-Bump, keine Migration, keine Deps, i18n unverändert.
- [Phase ?]: Phase 45 D-45-01: FE-Clippy-Gate -D warnings scharfgestellt (177 zu 0 warnings).
- [Phase ?]: D-49-15/D-49-16 doc audit passed (grep-verified) for REQUIREMENTS.md PDF-03 and ROADMAP.md Phase 49 Goal+SC 3 — no-op regelfall
- [Phase ?]: Phase 49 Plan 04: Pure-fn visibility gate + <a>-anchor download-attribute pattern for Cookie-Auth file download in Dioxus WASM (sidesteps signal-mocking via cargo-test-only unit-test matrix).
- [Phase 54]: D-54-DM-01 enforced: partial UNIQUE index on rebooking_batch (sales_person_id, iso_year, iso_week) WHERE deleted IS NULL — global across all kinds (Claim-on-Suggest, blocks second active row per (sp, y, w) verified via sqlite3 constraint test).
- [Phase 54]: D-54-DM-02 enforced: extra_hours.source TEXT NOT NULL DEFAULT 'manual' + ExtraHoursSource enum in service::extra_hours; F1/F2 aggregators (Plan 03) filter source==Manual.
- [Phase 54]: Basic-tier RebookingBatchDao consumes only Transaction (no Domain-Service deps); mirrors WeekStatusDao precedent for str<->enum roundtrip.
- [Phase ?]: [54-02] Basic-Tier RebookingBatchService via gen_service_impl! — HR-gated, deps strikt DAO+Permission+Clock+Uuid+Transaction; D-54-DM-01 UNIQUE-Slot via Same-Transaction Pre-Check → EntityAlreadyExists.
- [Phase ?]: Plan 54-04: HR-Only-Redaktion verbleibt im Service (VoluntaryStatsService), Handler leitet 1:1 durch — VAC-OFFSET-01-Praezedenz konsistent
- [Phase ?]: 55-01: Conflict-Variante = ServiceError::EntityAlreadyExists fuer beide 409-Faelle (Slot-Kollision + rows_affected==0)
- [Phase ?]: 55-01: Neue DaoError::EntityAlreadyExists-Variante als distinktes UNIQUE-Violation-Signal (via is_unique_violation), im Service auf EntityAlreadyExists gemappt
- [Phase ?]: D-55-A1: source==Manual-Filter am Report-Aggregations-Eingang (nicht Korrektur-Row) → VOL-ACCT-03-Neutralitaet property-getestet
- [Phase ?]: Single Pair-write-path rebook() geteilt von rebook_manual (F3) + approve (F5); approve transitioniert bestehenden pending-Batch statt neuen anzulegen
- [Phase 55]: 55-03: RebookingSuggestionTO auf reale 55-02-Domain-Felder modelliert (keine Fake-Soll/Delta) — Prohibition backend-computed gewahrt
- [Phase 55]: 55-03: HR-Alert-Banner nur ueber short-report get_reports_for_all_employees getrieben; assemble_weeks/get_week bleibt neutral (false/None)
- [Phase ?]: 55-04: RebookingSuggestionModal rendert nur backend-gelieferte IST/DANN-Zeilen (Balance + Freiwillig-Ist + proposed_hours); voluntary_soll/delta existieren im Backend nicht (55-03 Deviation) und werden NICHT im FE gerechnet.
- [Phase ?]: 55-04: Rebooking-Modals sind prop-driven (Callbacks); Async-Dispatch + 409/Refetch gehoert der Page (55-05).
- [Phase ?]: [Phase 56]: 56-01 D-01 outcome STAYS AT snapshot version 12 — straddling golden-snapshot byte-identical across pre/post/straddling/None cutoff scenarios; no chain leaks rebooking rows. Plan 06 skips the 12->13 bump branch.
- [Phase ?]: [Phase 56]: 56-01 voluntary_rebooking_gate::should_run_for_week is the single ISO-week cutoff predicate (Monday via Date::from_iso_week_date, inclusive lower boundary) reused by the Plan 02/03 scheduler write-guard.
- [Phase ?]: 56-02: F4 excess = ist - committed (VoluntaryStats::soll_total is the committed-voluntary target; avoids double-subtraction)
- [Phase ?]: 56-02: run_for_week is the single trigger-agnostic F4 engine; backfill_range delegates apply-writes to it (single write-path)
- [Phase ?]: 56-03: VoluntaryRebookingScheduler consumes the minimal dep set (RebookingReconciliation + EmployeeWorkDetails + Permission + Clock + TransactionDao); toggle-gate/week-lock stay inside run_for_week.
- [Phase ?]: 56-03: Scheduler has no RestStateDef accessor (no REST surface until Plan 04); plain RestStateImpl field booted after pdf_export_scheduler.
- [Phase ?]: 56-03: Weekly cron hardcoded 6-field 0 15 3 * * 1 (Monday 03:15), conflict-free vs pdf_export; no reload_from_db (toggle read fresh per run).
- [Phase ?]: 56-04: F4 backfill REST endpoint POST /admin/rebooking/backfill is a thin delegate to backfill_range — from-week parsed/validated to 400 before the service; HR-Admin gate + dry-run/apply write-path stay in the service; REST coverage via in-crate parse unit tests + OpenAPI-surface contract test.
- [Phase ?]: 56-05: F4 FE toggle name mirrored as a single const referencing voluntary_rebooking_gate::TOGGLE_NAME (rest-types does not re-export it)
- [Phase ?]: 56-05: FE is_valid_iso_week_input is syntactic-only; authoritative range/53-week check stays server-side (Fat Backend, Thin Client)
- [Phase ?]: [Phase 56]: 56-06 Docs-Freshness-Gate synced (EN+DE) for F4 cron+backfill; snapshot version STAYS AT 12 (D-01, no bump) — stale F14 12→13 corrected, F08 non-bump confirmed via straddling fixture.
- [Phase 57]: 57-01 D-57-09 umgesetzt — VOL-ACCT-03 schuetzt nur noch VOL-ACCT-01-Soll; Balance/Ist-Neutralitaet ersatzlos gestrichen, Fehlerspur D-55-A1 als Historien-Blockquote erhalten
- [Phase 57]: 57-01 VOL-ACCT-04 (Ausgleichswirkung Balance +h / Ehrenamt-Ist -h, Akzeptanz Referenz-Szenario D-57-02) und REB-AUTO-06 (Balance-Deckel im Auto-Cron inkl. BackfillPreviewItem.excess, HR-Formel min(|balance|, ist) ausgenommen) angelegt; Coverage 19/19, VOL-ACCT-03 auf Phase 57 umgehaengt
- [Phase 57]: 57-01 REB-AUTO-05 erhaelt Nachtragsnotiz (D-57-10/D-57-11) statt Ummapping — Phase-56-Ausgang 'bleibt bei 12' auf 13 revidiert, Owner bleibt Phase 56
- [Phase ?]: D-57-06 umgesetzt (57-02): Sites 2/3/4 verlieren das source-Prädikat ersatzlos, Site 1 bekommt stattdessen eine konjunktive VolunteerWork-UND-Rebooking-Ausnahme an der availability()-basierten extra_working_hours-Summe
- [Phase ?]: ExtraHoursSource bleibt reines Provenienz-Feld, für die Aggregation nur noch an einer Stelle wirksam (assemble_weeks); RISK-01 bleibt bewusst unverändert (57-02)
- [Phase ?]: D-57-07 umgesetzt: Balance-Deckel liest year-to-date ab first_day_in_year(iso_year) bis Sonntag der ISO-Woche mit include_carryover=true — nicht die isolierte Wochen-Range von suggest()/approve()
- [Phase ?]: Balance-Deckel gilt an beiden Call-Sites: run_for_week (Schreib-Pfad) und backfill_range (Dry-Run-Preview) — der Preview zeigt exakt den Betrag, den der Apply-Lauf bucht (I-7)
- [Phase ?]: REB-AUTO-06 bleibt offen bis Plan 04: die im Requirement geforderte Gegenprobe (Woche 2 mit 20h) ist ein Ende-zu-Ende-Szenario
- [Phase ?]: Plan 04: VOL-ACCT-04 und REB-AUTO-06 sind mit dem Ende-zu-Ende-Referenz-Szenario D-57-02 belegt (8/12/-2 vor, 10/10/0 nach dem Uebertrag); die Ehrenamt-Stunden entstehen ausschliesslich aus dem Shiftplan-Cap-Ueberlauf
- [Phase ?]: D-57-10 umgesetzt: CURRENT_SNAPSHOT_SCHEMA_VERSION 12 -> 13 (Filter-Fix ist ein 'Change the input set the computation reads from')
- [Phase ?]: Der v13-Bump betrifft VIER value_types (Overall, Balance, ExtraWork, Volunteer) - Overall war im Plan uebersehen und wurde per Messung ergaenzt
- [Phase ?]: Fuenf statt drei Snapshot-Locking-Gates nachgezogen; pdf_export_config und shiftplan_report pinnen die Konstante ebenfalls absolut
- [Phase ?]: D-57-06 dokumentiert: F14 §3 / F07 §2.6 beschreiben die Kategorie-Semantik statt des Marker-Filters, mit der einen assemble_weeks-Ausnahme
- [Phase ?]: Widerrufene Entscheidungen werden als HISTORISCH markiert und mit Umschlagpunkt versehen statt geloescht (F08 §7.4, T-57-18)
- [Phase ?]: Die F08-Bump-Historie nennt VIER betroffene value_types (Overall, Balance, ExtraWork, Volunteer) — Messbefund aus Plan 05
- [Phase 60]: D-60-01 bestaetigt (60-01 Checkpoint, auto-mode): UNIQUE-Index auf sales_person.person_id bleibt vollstaendig statt partiell — soft-geloeschte Sales Persons behalten ihre Person-Identitaet dauerhaft (one-way).
- [Phase 60]: Apply-twice-Nachweis via Marker-Split (BACKFILL-START/END-Kommentare + sqlx::raw_sql) statt vollem Datei-Replay — SQLite kennt fuer ALTER TABLE ADD COLUMN keine Existenz-Guard-Klausel.
- [Phase 60]: ~~[To verify] in der Foreign-Keys-Konvention aufgeloest — PRAGMA foreign_keys wird nirgends gesetzt, FK-Deklarationen sind dekorativ (D-60-06)~~ — **WIDERLEGT in Plan 60-04**, siehe den Korrektur-Eintrag weiter unten
- [Phase 60]: one-way-Vermerk zu den vollstaendigen Unique-Indizes steht in beiden Sprachfassungen von 03-data-model UND im Migrations-Kopfkommentar — Vertragsbestandteil gegen kuenftige Aufraeum-Passes
- [Phase 60]: SQLite-Zeitstempel in Migrationen als strftime('%Y-%m-%dT%H:%M:%S.000000000Z','now') schreiben — datetime('now') erzeugt ein Leerzeichen, das Iso8601::DATE_TIME in beiden Parser-Zweigen ablehnt
- [Phase 60]: Korrektur zu D-60-06: FK-Durchsetzung IST in diesem Repo aktiv — sqlx setzt PRAGMA foreign_keys = ON per Verbindung (sqlx-sqlite-0.8.6/src/options/mod.rs:185). Ein Quelltext-Grep ist hier kein Beleg.
- [Phase 61-01]: PersonService ist Basic-Tier mit genau fuenf Deps (PersonDao, Permission, Clock, Uuid, Transaction) — kein Domain-Service; Konstruktion im Basic-Block vor der Business-Logic-Schicht (SC5, T-61-07)
- [Phase 61-01]: Beide Kandidaten-Lesepfade sind LEFT-JOIN-Anti-Joins ueber person_id, keine IS-NULL-Abfragen (D-61-02); Regressionsschutz: kandidat_trotz_gesetzter_person_id
- [Phase 61-01]: Alle vier PersonService-Methoden HR-gated (T-61-05); Phase 62/63 darf auf admin verschaerfen, nicht lockern
- [Phase 61]: 61-02: person_id-Vergabe bei Neuanlage — ID in Rust erzeugt (kein Rowid-Readback), created per strftime im Backfill-Format; DAO-Ebene bleibt ClockService-frei
- [Phase 61]: 61-02: create_user oeffnet eine methoden-interne Transaktion — Trait-Signatur und beide Aufrufer bleiben unveraendert, Atomaritaet per PK-Konflikt-Test belegt (Pitfall 4)
- [Phase 61]: D-61-08 umgesetzt: Self-Gate entscheidet ausschliesslich im SQL-JOIN (u.person_id = sp.person_id); kein Rust-seitiger Option-Vergleich, weil None == None fail-open waere
- [Phase 61]: Kein Soft-Delete-Filter in den vier Lesepfaden nachgeruestet — verify_user_is_sales_person haengt an get_assigned_user; festgezurrt durch soft_geloeschte_sales_person_behaelt_selbstzugriff
- [Phase 61]: LEGACY-VERGLEICH-Markerblock in person_self_gate.rs ist der Abbau-Vertrag fuer Plan 61-05 (D-61-09-Matrix verschwindet mit der DROP-Migration)
- [Phase 61]: 61-04: Verknuepfen ist ein Merge zweier Personen; die absorbierte person-Zeile wird geloescht (D-61-03)
- [Phase 61]: 61-04: 409 nur bei echter Mehrdeutigkeit — komplementaeres Verschmelzen zweier Bestandspersonen geht durch (D-61-04)
- [Phase 61]: 61-04: Loesen laesst die Sales Person identitaetsstabil und gibt dem Benutzer eine frische Person (D-61-05)
- [Phase 61]: 61-04: Re-Link A->B laeuft als Split(A)+Merge(B) in einer Transaktion; der 409 rollt den Split mit zurueck (P-61-B)
- [Phase ?]: Doku-Nachzug prueft zuerst, was fruehere Plaene derselben Phase bereits korrigiert haben — 02-service-tiers und arc42/05 blieben deshalb bewusst unveraendert (61-06)
- [Phase ?]: Die Regel 'Identitaetsvergleich gehoert ins SQL, nicht nach Rust' ist als dauerhafte Konvention in 04-auth{,_de}.md verankert, nicht als Phasen-Notiz (T-61-21)
- [Phase 61]: D-61-07-A: Der Nachzug der DROP-Migration wendet die Gewinner-Regel des Laufzeitpfads en bloc an — Entkopplung des verdraengten Benutzers (D-61-05) VOR der Verschmelzung auf die Person der Sales Person (D-61-03), danach Aufraeumung traegerloser person-Zeilen. Der im Review vorgeschlagene COALESCE-Fix wurde verworfen: er tauscht CHECK constraint failed gegen UNIQUE constraint failed.
- [Phase 61]: D-61-07-B: Die Migrationsdatei 20260917000000 wird IN PLACE korrigiert statt per Folgemigration — ein Abbruch innerhalb dieser Datei laesst jede spaetere Migration nie laufen. Preis ist die lokale sqlx-Ledger-Checksumme; die Reparaturanleitung steht in 61-07-SUMMARY.md.
- [Phase 61]: clear_all loescht nur traegerlose person-Zeilen — Loeschbedingung wortgleich zu Anweisung A7 der DROP-Migration — Review-Befund CR-02: ein pauschales DELETE FROM person liess jede ueberlebende user-Zeile (inkl. DEVUSER/admin) auf eine nicht mehr existierende Person zeigen; user.person_id traegt keine FK-Deklaration, die DB faengt das nicht ab.
- [Phase 61]: D-61-09a: Split-Konflikt in set_user wird wie der Merge-Konflikt nach ServiceError::EntityAlreadyExists (409) uebersetzt; die Variante bleibt bewusst EntityAlreadyExists, weil 409 fuer einen Race-Verlierer der richtige Status ist und eine eigene Variante eine Vertragserweiterung ohne Bedarf waere (WR-04)
- [Phase 61]: D-61-09b: Der rows_affected() != 1-Zweig eines DAO wird auf Service-Ebene gemockt statt integriert getestet — Vorab-Lesung und UPDATE laufen in derselben Transaktion, ein Nebenlaeufer kommt nicht dazwischen; die Begruendung steht im Doc-Kommentar der Tests
- [Phase 61]: D-61-10a: AUTHENTICATION.md bleibt bestehen und wird NICHT durch docs/architecture/04-auth.md ersetzt (die zweite im Review angebotene Option) — die Datei traegt Setup-, OIDC- und Beispielstoff, den die kanonischen Dokumente nicht fuehren. Stattdessen legt ein Verweis-Banner am Dateianfang die Vorrangregel fest: bei Widerspruch gewinnen die zweisprachig gepflegten Dokumente unter docs/architecture/.
- [Phase 61]: D-61-10b: Es wird KEIN AUTHENTICATION_de.md angelegt. Die EN/DE-Regel aus CLAUDE.md gilt fuer Paare, die es gibt; eine neue 600-Zeilen-Uebersetzung waere neuer Doku-Scope statt Gap-Closure.
- [Phase 61]: D-61-10c: Der Repo-Sweep nach dem alten Tabellennamen kann nicht 0 liefern und soll es auch nicht — migrations/sqlite/*.sql bleiben byte-identisch (sqlx-Checksumme je Datei), shifty_bin/src/person_link_migration_test.rs baut den Vor-Phase-61-Zustand absichtlich nach, und die Treffer in rest/ und shifty-dioxus/ sind Bezeichner (get_sales_person_user, sales_person_user_links), keine Tabellenverweise. Die pruefbare Aussage ist 'kein Dokument beschreibt die Tabelle als aktuelles Schema', nicht 'kein Vorkommen des Strings'.
- [Phase 62]: D-62-05/WR-02 geschlossen: Soft-Delete-Filter in der ON-Klausel des LEFT JOIN, nicht in WHERE — In WHERE degeneriert der LEFT JOIN zum INNER JOIN. Ein Benutzer, dessen einzige Sales Person soft-geloescht ist, fiel vorher aus beiden Kandidatenlisten und war nie wieder verknuepfbar.
- [Phase 62]: Das Personen-Verzeichnis bekommt ein eigenes DAO-Trait (PersonDirectoryDao) statt einer fuenften Methode auf PersonDao — PersonDao ist im Modulkopf auf genau vier Methoden festgezurrt; das Verzeichnis ist eine andere Lesart derselben Tabellen (Aggregat statt Identitaet/Anti-Join). Zwei Zustaendigkeiten, zwei Traits.
- [Phase 62]: 62-02 — PersonEntryTO landet ohne From-Impl in Welle 1 (Domain-Typ folgt in 62-03), damit Frontend- und Business-Logic-Plan parallel starten koennen
- [Phase 62]: 62-02 — Neuer i18n-Key ColumnUsername statt Wiederverwendung von ColumnLinkedUser — seit D-62-03 zeigt jede Zeile einen Benutzernamen
- [Phase 62]: 62-02 — Akzeptanzbefehle mit 'cargo test -- --exact' muessen den Modulpfad tragen, sonst laufen sie gruen mit 0 Tests
- [Phase 62]: D-62-08 umgesetzt — PersonDirectoryService gated auf das Literal "admin" (Vorbild PermissionServiceImpl::get_all_users), bewusst NICHT HR_PRIVILEGE des Basic-Tier PersonService
- [Phase 62]: Fehler statt Redaktion — bei fehlendem Admin-Privileg propagiert list() ServiceError::Forbidden; von VoluntaryStatsService wird nur Tier-/DI-Form uebernommen, nicht die None-Redaktion
- [Phase 62]: Gate-Wahl als Test-Invariante — der Permission-Mock akzeptiert nur das Literal "admin", ein falsches Gate faellt in allen acht Tests um statt nur im Gate-Test
- [Phase 62]: Auspraegung kommt als Backend-Enum ins Frontend; keine clientseitige Namens-Fallback-Regel (grep-verifiziert)
- [Phase 62]: load_all_users/get_all_users/state::User als begruendetes dead_code fuer Phase 63 konserviert statt geloescht
- [Phase 62]: Das Admin-Gate des Personen-Verzeichnisses bleibt allein im Service; der REST-Handler prueft bewusst nicht doppelt (T-62-13).
- [Phase 62]: Routen-Praefix /person-directory als Vier-Stellen-Vertrag (Router-Nest, OpenAPI-Nest, Frontend-URL, Dev-Proxy); der Proxy-Eintrag ist Pflichtinhalt desselben Commits.
- [Phase 62]: 62-06: F01 §5.4 als eigener REST-Abschnitt statt nur eines Absatzes in §4.6 — §5 ist die kanonische Endpoint-Sektion; EN/DE-Ueberschriftenparitaet bleibt 33/33
- [Phase 62]: 62-06: Tier-Faustregel in 02-service-tiers §Rules Nr. 3 praezisiert — Dependency-Zaehlung ist Indikator, die Aggregat-Frage entscheidet; PersonDirectoryService ist Business-Logic trotz Basic-aussehender Dep-Liste
- [Phase 62]: 62-06: PersonDirectory-Knoten im Runtime-Graph bewusst ohne Kante — Deps sind per Kopf-Konvention ausgeblendet; Tier folgt der Aggregat-Eigenschaft, nicht der Kantenzahl
- [Phase 62]: 62-06: vorgefundene Phase-56-Drift geschlossen (VoluntaryRebookingScheduler fehlte in 02-service-tiers_de.md) statt das Paritaets-Kriterium durch Weglassen in der EN-Fassung zu erfuellen
- [Phase ?]: D-63-01-A: Transaktions-Assoziationstyp sitzt auf dao::PermissionDao, nicht auf service::PermissionService — das Service-Trait bleibt eingefroren (rund 20 Konsumenten).
- [Phase ?]: D-63-01-B: PermissionDao bleibt ein bewusst gemischtes Trait — nur create_user und delete_user nehmen die geteilte Transaktion, keine Vorratshaltung.
- [Phase 63]: D-63-01 als Lesart (b): Person ist traegerlos, wenn keine user-Zeile und keine sales_person-Zeile mit deleted IS NULL mehr auf sie zeigt (63-02)
- [Phase 63]: Haengender person_id auf soft-geloeschten sales_person-Zeilen ist akzeptierte Konsequenz von D-63-01, kein Bug (63-02)
- [Phase 63]: 63-03: Die zwei D-63-08-Saetze stehen zeichengleich in de.rs; die Zusaetze sind eigene Keys und verlaengern den gesperrten Satz nicht.
- [Phase 63]: 63-03: Neben dem geforderten Key-Mengen-Test gibt es einen Platzhalter-Test, der {name} in UnlinkConfirmBody und DeleteSalesPersonConfirmHint ueber alle drei Sprachen garantiert (T-63-03-A).
- [Phase 63]: D-63-04-A: Zuschnitt des Detail-Aggregats = Kern-Aggregat plus gezielte Nachladepfade — GET /person/{id} liefert genau die Verzeichniszeile, die vier Abschnitte laden ihre bestehenden Endpoints
- [Phase 63]: D-63-04-B: entry_from_row wird pub(crate) statt einer zweiten Ableitung — kind und display_name behalten genau eine Definition (PADM-06)
- [Phase 63]: D-63-04-C: Kandidatenlisten rufen den HR-gegateten PersonService mit Authentication::Full, erst nach dem eigenen Admin-Gate
- [Phase 63]: 63-05: add_facet gibt die RESULTIERENDE person_id zurueck, nicht die uebergebene — beim Zuschalten einer Sales Person gewinnt sie nach D-61-03 und der Eintrag wechselt seine Identitaet.
- [Phase 63]: 63-05: Beide Anlege-Pfade validieren Benutzername UND Anzeigename serverseitig vor jedem Datenbankzugriff (D-63-07, ASVS V5).
- [Phase 63]: 63-05: session.user_id ohne ON DELETE CASCADE bleibt ungefixt (T-63-05-E, accept) und ist im Modulkopf als bekannte Kante fuer Plan 63-11 dokumentiert.
- [Phase 63]: 63-06: add_person_facet antwortet mit 200 + CreatedPersonEntryTO statt 204 — die RESULTIERENDE person_id muss den Aufrufer erreichen (D-61-03)
- [Phase 63]: 63-06: rest::sales_person wird pub-exportiert (Datei unveraendert), damit der 409-Regressionstest den unveraenderten Phase-61-Endpoint via tower::oneshot befaehrt
- [Phase 63]: 63-06: der [[web.proxy]]-Eintrag /person steht HINTER /person-directory — der Dev-Server-Proxy matcht praefix-literal

## Performance Metrics

| Phase | Plan | Duration | Notes |
|-------|------|----------|-------|
| Phase 35 P01 | 25 | - tasks | - files |
| Phase 35 P02 | 14 | 3 tasks | 8 files |
| Phase 36 P02 | 10 | 3 tasks | 2 files |
| Phase 37 P01 | 7m | 2 tasks | 2 files |
| Phase 37-modal-ux-politur P02 | 35 | 2 tasks | 5 files |
| Phase 39 P02 | 7min | 2 tasks | 6 files |
| Phase 39 P04 | 12min | 2 tasks | 11 files |
| Phase 39 P05 | ~18min | 2 tasks | 5 files |
| Phase 40 P01 | 18min | 2 tasks | 6 files |
| Phase 40 P02 | 12min | 2 tasks | 5 files |
| Phase 40 P03 | 11min | 2 tasks | 3 files |
| Phase 41 P01 | ~10min | 2 tasks | 3 files |
| Phase 41 P04 | ~12min | 3 tasks | 7 files |
| Phase 42 P01 | ~13min | 3 tasks | 2 files |
| Phase 45 P01 | 20 | 3 tasks | 22 files |
| Phase 49 P01 | 5 | - tasks | - files |
| Phase 49 P05 | 1min | 2 tasks | 1 files |
| Phase 49 P04 | 14min | 3 tasks | 5 files |
| Phase 54 P01 | 38min | 6 tasks | 27 files |
| Phase 54 P02 | 22m | 6 tasks | 9 files |
| Phase 54 P04 | 7min | 4 tasks | 5 files |
| Phase 54 P05 | 10min | 5 tasks | 10 files |
| Phase 54 P06 | 11m | 5 tasks | 14 files |
| Phase 54 P08 | 6 | 4 tasks | 8 files |
| Phase 55 P01 | 40min | 3 tasks | 8 files |
| Phase 55 P02 | 40min | 3 tasks | 7 files |
| Phase 55 P03 | 55 min | 3 tasks | 10 files |
| Phase 55 P04 | 45min | 3 tasks | 13 files |
| Phase 55 P06 | 15min | 2 tasks | 7 files |
| Phase 56 P01 | 9min | 3 tasks | 5 files |
| Phase 56 P02 | 15min | 2 tasks | 8 files |
| Phase 56 P03 | 12min | 3 tasks | 7 files |
| Phase 56 P04 | 11min | 2 tasks | 4 files |
| Phase 56 P05 | 8min | 3 tasks | 9 files |
| Phase 56 P06 | 7min | 3 tasks | 8 files |
| Phase 57 P01 | 25min | 2 tasks | 2 files |
| Phase 57 P02 | ~35min | 3 tasks | 3 files |
| Phase 57 P03 | 20min | 2 tasks | 4 files |
| Phase 57 P04 | 15min | 2 tasks | 2 files |
| Phase 57 P05 | 25min | 2 tasks | 7 files |
| Phase 57 P06 | ~35min | 3 tasks | 10 files |
| Phase 60 P01 | 12min | 3 tasks | 3 files |
| Phase 60 P02 | 7min | 2 tasks | 1 files |
| Phase 60 P03 | 6min | 2 tasks | 7 files |
| Phase 61 P01 | 25min | 3 tasks | 17 files |
| Phase 61 P02 | 20min | 2 tasks | 8 files |
| Phase 61 P03 | 15min | 2 tasks | 9 files |
| Phase 61 P04 | 40min | 3 tasks | 9 files |
| Phase 61 P05 | 25min | 3 tasks | 15 files |
| Phase 61 P06 | 30min | 2 tasks | 10 files |
| Phase 61 P07 | 35min | 3 tasks | 4 files |
| Phase 61 P08 | 20 min | 2 tasks | 6 files |
| Phase 61 P09 | ~25min | 2 tasks | 5 files |
| Phase 61 P10 | ~30min | 2 tasks | 1 files |
| Phase 62 P01 | 32min | 3 tasks | 12 files |
| Phase 62 P02 | 13 min | 2 tasks | 5 files |
| Phase 62 P03 | 10min | 2 tasks | 9 files |
| Phase 62 P04 | 22min | 3 tasks | 9 files |
| Phase 62 P05 | 7min | 2 tasks | 6 files |
| Phase 62 P06 | 18min | 2 tasks | 5 files |
| Phase 63 P01 | 38min | 3 tasks | 9 files |
| Phase 63 P02 | 42min | 3 tasks | 6 files |
| Phase 63 P03 | 18min | 2 tasks | 4 files |
| Phase 63 P04 | 44min | 3 tasks | 8 files |
| Phase 63 P05 | 21min | 3 tasks | 3 files |
| Phase 63 P06 | 27min | 3 tasks | 6 files |
