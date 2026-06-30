---
gsd_state_version: 1.0
milestone: v1.10
milestone_name: — aktiv)
current_phase: 33
current_phase_name: Special-Days-UI in den Einstellungen
status: roadmap_ready
last_updated: "2026-06-30T11:47:02.164Z"
last_activity: 2026-06-30
last_activity_desc: "Phase 33 Plan 01 executed (TDD: DAO→Service→REST for-year chain + 5 mockall tests)"
progress:
  total_phases: 3
  completed_phases: 0
  total_plans: 4
  completed_plans: 3
  percent: 0
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (v1.10 **aktiv**, Phasen 33–35 expanded; v1.0–v1.9 archiviert/collapsed; Backlog 999.1 erhalten)
- **Requirements**: `.planning/REQUIREMENTS.md` (v1.10, 12/12 Requirements SPD-01..04 + HSP-01..04 + SWO-01..04, Traceability befüllt)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v1.9-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions); Frontend in `shifty-dioxus/CLAUDE.md` + `.planning/codebase/frontend/`
- **Last shipped/closed**: **v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation** (shipped + archiviert 2026-06-29, Phasen 29–32, 6 Pläne, 7/7 Requirements, Audit `passed`, override_closeout) — gesamter v1.9-Code **uncommitted** im Arbeitsbaum (jj manueller Commit durch User)
- **Current milestone**: **v1.10 Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz** (Roadmap erstellt 2026-06-30; interne Planungs-Labels, reale Release-Version datumsbasiert via `cli-update-version`)
- **Current focus**: Phasen 33, 34, 35 alle **diskutiert** (CONTEXT.md je vorhanden). **Phase 34 (Feiertags-Soll im Schichtplan)** zuletzt diskutiert — 4 Decisions D-34-01..04. Kern: gezielter Konsistenz-Fix der versehentlichen Auslassung in `get_week` (4. Injektionspunkt von `build_derived_holiday_map` analog 1a/1b/1c, D-34-01 nur per-Mitarbeiter expected_hours/holiday_hours, NICHT per-Tag-Zeile); **BE-only** (D-34-02, keine FE-Spalte); HOL-03-Test umbauen (Band-Guard dynamic_hours==40 + expected 32 + holiday 8, D-34-03); **kein Snapshot-Bump**, im Plan grep-verifizieren (D-34-04). Nächster Schritt: `/gsd-plan-phase 33` (oder 34/35) — alle drei sind plan-ready.
- **Snapshot-Schema-Version**: 12 (v1.7 Bump 10→11; v1.8 Bump 11→12); v1.9 **kein** Bump; v1.10 voraussichtlich **kein** Bump (in Phase 34 verifizieren)

## Current Position

Phase: 33 (Special-Days-UI in den Einstellungen) — **Plan 01 complete** (Backend for-year endpoint + tests)
Plan: 01 done — Plan 02 next (FE api.rs + Settings Card-3)
Status: Phase 33 in progress (1/4 plans complete); Plans 02-04 pending (FE wiring, Settings Card, Shiftplan dropdown, i18n)
Last activity: 2026-06-30 — Phase 33 Plan 01 executed (TDD: DAO→Service→REST for-year chain + 5 mockall tests)

## Deferred Items

Erneut acknowledged + deferred beim **v1.7 + v1.8 Milestone-Close am 2026-06-29**
(User-Entscheidung override_closeout — Pre-Close-Audit meldete 15 offene Items, alle
bereits vorab-deferred Carry-over, keines v1.7/v1.8-spezifisch). Davor beim v1.6-Close
am 2026-06-28; Ursprung v1.5/v1.4:

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

## Shipped Milestones

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

- **VCS**: Repository wird mit `jj` (co-located mit git) verwaltet — Commits manuell durch User. GSD-Auto-Commit ist deaktiviert (`commit_docs: false`). KEINE `git commit`/`git add` aus Agents heraus.
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
- **Phase 35 hinzugefügt (2026-06-30):** „Slot-Werte nur für eine Woche ändern" (SWO-01..04, aus Todo `2026-06-26-einzelnen-slot-nur-fuer-eine-kw-aendern`). Bewusst in v1.10 aufgenommen (User-Entscheidung, statt Backlog), obwohl thematisch Schichtplan-**Struktur** (leicht off-theme zum Feiertags-Fokus). Mechanik **diskutiert + gewählt**: Ansatz **B (Split+Re-Merge)** — `modify_slot` um drittes Restore-Segment (3 Slot-Versionen) erweitern + UI-Wahl „nur diese Woche"/„ab dieser Woche"; Ansatz A (Override-Datenmodell) verworfen. Harte Constraints: **eine Transaktion/Rollback** + **harte Re-Point-Tests gegen Doppelzählung**. CONTEXT.md geschrieben (D-35-01..06). Gate `shiftplan.edit`. v1.10 jetzt 12/12 Requirements, Phasen 33–35.

## Session Continuity

**To resume work in a new session:**

1. Read `.planning/STATE.md` (this file)
2. Read `.planning/ROADMAP.md` (v1.10 aktiv, Phasen 33–34 expanded; v1.0–v1.9 archiviert/collapsed; Backlog 999.1)
3. Read `.planning/REQUIREMENTS.md` (v1.10, 8/8 Requirements, Traceability befüllt)
4. Read `.planning/PROJECT.md` (Charter + Current State: v1.10 aktiv)
5. Read `.planning/MILESTONES.md` (Index inkl. v1.7–v1.9)

**Aktueller Stand:** v1.10-Roadmap fertig — 8/8 Requirements gemappt (SPD-01..04 → Phase 33, HSP-01..04 → Phase 34). Phasen-Verzeichnisse angelegt.

**Next command**: `/gsd-plan-phase 33` (Phase 33 Kontext erfasst — shiftplanner-Gate, zwei Flächen voll-CRUD, neuer Range/Jahr-Read-Endpoint). Danach `/gsd-discuss-phase 34` (HSP: HOL-03-Test-Neuformulierung, vierter Injektionspunkt, Snapshot-Bump-Check).

---

*State updated: 2026-06-30 — **v1.10 Roadmap created** (manuell). Phasen 33–34 angelegt, 8/8 Requirements gemappt, Progress-Counters auf total_phases=2 gesetzt.*

## Operator Next Steps

- `/gsd-discuss-phase 33` oder `/gsd-discuss-phase 34`, dann `/gsd-plan-phase`.
- Optional vorab: v1.9-Code mit jj committen (Carry-over aus dem vorigen Milestone).

## Decisions

- [Phase ?]: D-33-02 enforced: Card-3 shiftplanner-gated with inner has_privilege guard, page admin gate unchanged
- [Phase ?]: Rule 3 deviation: reqwest switched from native-tls to rustls-tls for host test compilation without openssl
