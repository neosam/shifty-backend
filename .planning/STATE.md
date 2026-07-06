---
gsd_state_version: 1.0
milestone: v2.6
milestone_name: Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter
current_phase: 54
status: planning
stopped_at: ""
last_updated: "2026-07-06T18:30:00.000Z"
last_activity: 2026-07-06
last_activity_desc: Roadmap v2.6 created (Phasen 54-56, 17/17 Requirements mapped)
progress:
  total_phases: 3
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
current_phase_name: Data-Model + Voluntary Statistics (F1 + F2)
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (v2.6 aktiv: Phasen 54–56; Backlog 999.1 erhalten)
- **Requirements**: `.planning/REQUIREMENTS.md` (17 REQ-IDs in 5 Kategorien: VOL-STAT, VOL-ACCT, REB-MANUAL, REB-AUTO, HR-ALERT)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v2.5-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions); Frontend in `shifty-dioxus/CLAUDE.md` + `.planning/codebase/frontend/`
- **Last shipped/closed**: **v2.5 Weekly-Overview Performance & Freiwilligen-Abwesenheiten** (shipped + archiviert 2026-07-06, Phasen 52–53, 8 Pläne + 3 Follow-Ups, 9/9 Requirements WOP-01..05 + VAA-01..04, verified_closeout, Audit `passed`)
- **Current milestone**: **v2.6 Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter** (planning; Roadmap 2026-07-06 geschrieben)
- **Current focus**: Phase 54 discuss-phase — Data-Model + F1 + F2 (Migrations, RebookingBatchService Basic, VoluntaryStatsService BL, FE-Row im Employee-Detail-Report).
- **Snapshot-Schema-Version**: aktuell **12**. Potenzieller Bump 12→13 in **Phase 56** discuss-phase zu pinnen (REB-AUTO-05; Beweislast beim „Nein"-Zweig = Straddling-Golden-Snapshot).

## Current Position

Phase: 54 (planning)
Plan: —
Status: Roadmap fertig, `/gsd-discuss-phase 54` als nächster Schritt
Last activity: 2026-07-06 — Roadmap v2.6 created (Phasen 54-56, 17/17 Requirements mapped)

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

## Session Continuity

**Last session:** 2026-07-06
**Stopped at:** Roadmap v2.6 written (Phasen 54–56, 17/17 Requirements mapped).
**Resume file:** None

**To resume work in a new session:**

1. Read `.planning/STATE.md` (this file)
2. Read `.planning/ROADMAP.md` (v2.6 Milestone-Sektion + Phasen 54–56 mit Details)
3. Read `.planning/REQUIREMENTS.md` (17 REQ-IDs + Traceability)
4. Read `.planning/research/SUMMARY.md` (2 offene Discuss-Decisions)
5. Read `.planning/PROJECT.md` (Current Milestone: v2.6-Sektion)

**Aktueller Stand:** Milestone v2.6 in Planning. ROADMAP.md fertig (Phasen 54–56).
Snapshot-Version bleibt aktuell 12; Bump-Entscheidung in Phase 56 discuss-phase.

**Next command**: `/gsd-discuss-phase 54` (Data-Model + F1 + F2; Discuss-Points: D-F1-01
Denominator, D-F2-01 Mid-Week-Contract, D-54-DM-01 UNIQUE-Shape, D-54-DM-02 Marker-Approach).

---

*State updated: 2026-07-06 — Roadmap v2.6 created (Phasen 54–56, 17/17 Requirements mapped; ARCHITECTURE-C 3-Phasen-Baseline gewählt; Snapshot-Bump-Entscheidung + Stichtag-Toggle-Seed in korrekten Phasen zugeordnet; Docs-Freshness-Gate für F14 + F07 + F08 + Architektur-Docs vorgemerkt).*

## Operator Next Steps

- `/gsd-discuss-phase 54` starten (Discuss-Points aus ROADMAP.md Phase-54-Sektion).
- Optional: v2.5 Tech-Debt (SDF-03-Cleanup, DB-Indices, F07-Doku) als Todos in `.planning/todos/pending/` erfassen.

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
