---
gsd_state_version: 1.0
milestone: —
milestone_name: (zwischen Milestones)
current_phase: —
status: Zwischen Milestones — v2.5 shipped 2026-07-06 (verified_closeout, Audit passed)
stopped_at: "v2.5-Close abgeschlossen 2026-07-06 (9/9 Requirements, Phasen 52+53 archiviert unter milestones/v2.5-phases/). Nächste Iteration via /gsd-new-milestone."
last_updated: "2026-07-06T16:00:00.000Z"
last_activity: 2026-07-06
last_activity_desc: Milestone v2.5 complete + archived
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
current_phase_name: —
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (zwischen Milestones; Backlog 999.1 erhalten)
- **Requirements**: keine aktive `.planning/REQUIREMENTS.md` — wird bei nächstem `/gsd-new-milestone` frisch erzeugt
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v2.5-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions); Frontend in `shifty-dioxus/CLAUDE.md` + `.planning/codebase/frontend/`
- **Last shipped/closed**: **v2.5 Weekly-Overview Performance & Freiwilligen-Abwesenheiten** (shipped + archiviert 2026-07-06, Phasen 52–53, 8 Pläne + 3 Follow-Ups, 9/9 Requirements WOP-01..05 + VAA-01..04, verified_closeout, Audit `passed`)
- **Current milestone**: — (zwischen Milestones)
- **Current focus**: Kein aktiver Milestone. Nächste Iteration via `/gsd-new-milestone`.
- **Snapshot-Schema-Version**: **12** (v1.7 Bump 10→11; v1.8 Bump 11→12; v1.9–v2.5 kein Bump — v2.4 rein additive View-Layer-Kürzung; v2.5 Performance-Refactor + additive DTO-Erweiterung, kein `BillingPeriodValueType` angefasst).

## Current Position

**Zwischen Milestones.** v2.5 abgeschlossen 2026-07-06 (verified_closeout, Milestone-Audit `passed`, 9/9 Requirements, 2/2 Phasen archiviert). Follow-up-Kandidaten für v2.6+ Backlog:

- SDF-03 Semantik-Cleanup (`SpecialDayService::get_by_iso_year`)
- DB-Indices (`booking(year,cw)`, `extra_hours(date_time)`, `working_hours(from_year,to_year)`)
- F07-Doku für neue Pure-Helper aus Follow-Up #1+#2

Nächste Iteration via `/gsd-new-milestone`.

Last activity: 2026-07-06 — Milestone v2.5 complete + archived

## Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260702-jql | Special-Day Duplikat-Hinweis nach Create ausblenden, erst bei Feld-Änderung wieder | 2026-07-02 | b9d270b | [260702-jql-special-day-duplikat-hinweis-nach-create](./quick/260702-jql-special-day-duplikat-hinweis-nach-create/) |

## Deferred Items

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

## Shipped Milestones

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
- **Snapshot Versioning**: `CURRENT_SNAPSHOT_SCHEMA_VERSION` (aktuell **12**) MUSS gebumpt werden, sobald sich `value_type`-Berechnung oder -Input-Set ändert. **v1.10 erwartet KEINEN Bump** (Feature HSP speist sich aus `get_week`/`booking_information`, nicht aus dem `reporting.rs`-Snapshot-Pfad) — in Phase 34 verifizieren.
- **Clippy-Gate**: `cargo clippy --workspace -- -D warnings` ist Pflicht-Gate bei jedem Commit — `cargo test` allein reicht nicht.
- **Service-Tier-Konvention**: Basic Services konsumieren nur DAO/Permission/Transaction; Business-Logic Services kombinieren Aggregate. HSP-Logik gehört in `ReportingService` / Business-Logic-Tier.
- **Multi-Sprache (i18n)**: Alle benutzersichtbaren Texte in en/de/cs. SPD-01..04 (Special-Days-UI) betreffen neue Texte.
- **D-25-08-Grenze (für HSP zentral)**: Die Feiertags-Automatik darf in `get_week`/`booking_information` **nur** `expected_hours`/`holiday_hours`/`available_hours` reduzieren — `dynamic_hours`/`paid_hours`/`committed_voluntary_hours`/`volunteer_hours` bleiben unangetastet. HOL-03-Regressionstest `test_holiday_auto_credit_no_year_view_impact` wird in Phase 34 bewusst neu formuliert (Bänder unverändert, aber expected/available reduziert) — Decision in discuss-phase festhalten.
- **WASM-Datepicker-Caveat (D-25-06)**: Programmatisches Setzen von `<input type=date>` triggert Dioxus-Signale nicht zuverlässig → Persistenz-/Anzeige-Loop von SPD-01 im echten Browser verifizieren.

### Roadmap Evolution

- Backlog-Phase 999.1 „Breaking/Major Dependency-Migration" angelegt (2026-06-28): off-theme, eskaliert aus Quick-Task 260627-vgo. Bleibt separat verfügbar via `/gsd-plan-phase 999.1`.
- Milestone v1.9 + Phasen 29–32 (2026-06-29): 7 Requirements → 4 Phasen. Geshipt + archiviert.
- **Milestone v1.10 + Phasen 33–34 angelegt (2026-06-30):** 8 Requirements (SPD-01..04, HSP-01..04) → 2 Phasen, abgeleitet aus den zwei sauber trennbaren Features.
  - **Phase 33 = Special-Days-UI** (Backend-CRUD `POST/DELETE /special-days` + `for-week`-Read existiert seit v1.7). **Korrektur in discuss-phase (D-33-01..05):** **shiftplanner**-gated (nicht admin), **zwei Flächen** voll-CRUD (Schichtplan-Wochenraster Per-Tag-Dropdown + Settings-Sektion Kalenderdatum-Picker), **neuer Range/Jahr-Read-Endpoint** (Multi-Wochen-Item aus „deferred" gezogen). Kalenderdatum → `(year, iso_week, weekday)`-Mapping, Liste mit abgeleitetem Kontext (`15.08.2026 (Samstag, KW 33, 2026)`). Frontend-API `create_special_day`/`delete_special_day`/`get_special_days_for_year`. i18n de/en/cs. WASM-Datepicker-Caveat (D-25-06) auf der Settings-Fläche. Dir: `.planning/phases/33-special-days-ui-einstellungen/`.
  - **Phase 34 = Feiertags-Soll im Schichtplan** (Backend-zentriert; Frontend-Tabelle rendert `report.expected_hours`/`holiday_hours` bereits — **keine neue API**). `get_week` (`reporting.rs:884`) bekommt einen vierten Injektionspunkt für den derived-Holiday via `build_derived_holiday_map`, reduziert nur `expected_hours`/`holiday_hours`/`available_hours`; Kapazitätsbänder per Regressions-Guard geschützt; HOL-03-Test neu formulieren; Snapshot-Bump in der Phase verifizieren (Default: kein Bump). Dir: `.planning/phases/34-feiertags-soll-schichtplan/`.
  - Beide Features sind fachlich unabhängig; sequenzielle Reihenfolge 33→34 ist sinnvoll (SPD erzeugt die Einträge, die HSP in der Tabelle sichtbar macht), aber nicht hart erzwungen.
- **Milestone v2.1 + Phasen 39–42 angelegt (2026-07-01):** 9 Requirements → 4 Phasen, abgeleitet aus 3 sauber trennbaren Features + der WST-Zweiteilung (Grundlage vs. Sperr-Durchsetzung).
  - **Phase 39 = KW-Status Grundlage** (BE+FE, WST-01/02/05): neue `week_status`-Tabelle + Migration (TEXT-Enum analog `special_day`, ISO-(year,week) analog `week_message`, partial UNIQUE), `WeekStatusService` (Basic-Tier), Status-CRUD-REST, DI-Wiring, FE Badge + Set-Button (nur Schichtplaner). Discuss: wer setzt Status + erlaubte Übergänge; None-Variante ≠ `None` (Clippy).
  - **Phase 40 = Wochen-Sperre durchsetzen** (BE+FE, WST-03/04): `assert_week_not_locked`-Helper in allen 6 Schreibpfaden (`ShiftplanEditService`), NEU `delete_booking` + Re-Routing `DELETE /booking/{id}` (schließt Basic-Tier-Bypass), `ServiceError::WeekLocked`→HTTP 423 (Default), Check in-Transaktion (TOCTOU), FE read-only + 423-Banner. Discuss: HTTP 423 vs 409.
  - **Phase 41 = Ø-Anwesenheit flexible Stunden** (BE+FE, AVG-01/02/03): neue Read-Aggregat-Methode in `ReportingService`, KEIN Snapshot-Bump, kein neuer `BillingPeriodValueType`, HR-gated REST + FE-Report. Discuss: D-AVG-01..08 (Bezugsgröße, Zähler, Exclusion-Set — A-22-1 ist NICHT identisch, `is_dynamic`-Scope, Anzeige-Ort, No-Persist).
  - **Phase 42 = Special-Days-„Anlegen"-Button-Bugfix** (FE-only, SDF-01): Reset-Block `settings.rs:458-459` entfernen (Option 2, nach Create nichts zurücksetzen) + SSR-/Komponenten-Test. Isoliert, zuletzt.
  - Reihenfolge: WST (39→40) vor AVG (41, unabhängig, geringeres Risiko), SDF (42) zuletzt. Querschnitts-Gates pro Phase: sqlx prepare + `.sqlx` committen nach neuer Query, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, FE WASM-Build + `cargo test -p shifty-dioxus`.
- **Phase 35 hinzugefügt (2026-06-30):** „Slot-Werte nur für eine Woche ändern" (SWO-01..04, aus Todo `2026-06-26-einzelnen-slot-nur-fuer-eine-kw-aendern`). Bewusst in v1.10 aufgenommen (User-Entscheidung, statt Backlog), obwohl thematisch Schichtplan-**Struktur** (leicht off-theme zum Feiertags-Fokus). Mechanik **diskutiert + gewählt**: Ansatz **B (Split+Re-Merge)** — `modify_slot` um drittes Restore-Segment (3 Slot-Versionen) erweitern + UI-Wahl „nur diese Woche"/„ab dieser Woche"; Ansatz A (Override-Datenmodell) verworfen. Harte Constraints: **eine Transaktion/Rollback** + **harte Re-Point-Tests gegen Doppelzählung**. CONTEXT.md geschrieben (D-35-01..06). Gate `shiftplan.edit`. v1.10 jetzt 12/12 Requirements, Phasen 33–35.

## Session Continuity

**Last session:** 2026-07-06
**Stopped at:** Phase 52 verified + committed. 5 Pläne + 2 Follow-ups landed. Verification report: `.planning/phases/52-weekly-overview-performance-refactor/52-VERIFICATION.md`.
**Resume file:** None

**Phase 52 Follow-Ups (nicht blockierend, für Milestone-Close-Audit):**

1. ✅ **Erledigt via Follow-up #3 (2026-07-06):** `SpecialDayService::get_by_iso_year` + `ExtraHoursService::find_by_iso_year` + `ShiftplanReportService::extract_shiftplan_report_for_iso_year` als ISO-Wochenjahr-Bulk-Varianten eingeführt; alte kalender-jahr-Methoden bei ExtraHours+ShiftplanReport gelöscht (grep-verifiziert keine externen Konsumenten); SpecialDay's `get_by_year` bleibt für REST-Endpoint `/special-days/year/{y}`. SC#4-Override formal aufgelöst.
2. DB-Indices aus RESEARCH.md Q3 (`booking(year, calendar_week)`, `extra_hours(date_time)`, `working_hours(from_year, to_year)`) — separater Task, benötigt Migration.
3. F07-Doku für neue crate-private In-Memory-Helper (`derive_hours_for_week_pure`, `build_derived_holiday_map_for_week_pure`) nachziehen — Balance-Formel selbst unverändert.

**To resume work in a new session:**

1. Read `.planning/STATE.md` (this file)
2. Read `.planning/ROADMAP.md` (kompaktes Milestone-Grouping-Format; Backlog 999.1 erhalten)
3. Read `.planning/PROJECT.md` (Charter + Current State: zwischen Milestones)
4. Read `.planning/MILESTONES.md` (Index inkl. v1.0–v2.4)

**Aktueller Stand:** Milestone v2.5 in Progress. Phase 52 (Weekly-Overview Performance-Refactor) am 2026-07-06 komplett + verifiziert. Phase 53 (Freiwilligen-Abwesenheiten in Jahresansicht) steht als nächstes offen — baut auf der neuen Assembly (`assemble_weeks` + `get_year`) aus Phase 52 auf.

**Next command**: `/gsd-execute-phase 53` (Phase 53 PLANNED 2026-07-06, 3 Pläne Wave 1→2→3, VERIFICATION PASSED 0 Blocker/0 Warnings).

---

*State updated: 2026-07-06 — **Phase 52 VERIFIED PASS + Follow-up #3 Jahresübergangs-Fix** (5 Pläne + 3 Follow-ups; 5/5 Success-Criteria erfüllt, SC#4-Override formal aufgelöst durch Follow-up #3; 9/9 Cross-Cutting-Must-haves grün; kumulativer Speedup ~19× 2.33s→0.09s; Byte-Identity 8/8 Wave-1-Fixtures + 16 Year-Boundary-Regression-Gates grün; Snapshot-Version bleibt 12). Follow-up #3 nach User-Report am 2026-07-06: 3 kalender-jahr-scharfe Bulk-Methoden (extra_hours + special_day + shiftplan_report) verursachten paid_hours/required_hours-Drift an KW 1 / KW 53 — gefixt via neue `_iso_year`-Varianten. VERIFICATION.md aus Verify + 2 DIAGNOSIS.md + 3 Follow-up-SUMMARY.md archiviert. Milestone v2.5 zu 50% (1/2 Phasen, 5/5 Pläne). Phase 53 offen.*

## Operator Next Steps

- Phase 53 ist geplant (3 Pläne Wave 1→2→3, alle Gates grün). Ausführen mit `/gsd-execute-phase 53`.
- Optional: Phase-52-Follow-ups (DB-Indices, F07-Doku für Pure-Helper) als Todos in `.planning/todos/pending/` erfassen.

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
