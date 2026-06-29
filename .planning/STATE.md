---
gsd_state_version: 1.0
milestone: none
milestone_name: "(kein aktiver Milestone — v1.7 + v1.8 geschlossen 2026-06-29)"
current_phase: null
current_phase_name: "(Planung des nächsten Milestones offen)"
status: milestone_complete
last_updated: "2026-06-29T12:30:00.000Z"
last_activity: 2026-06-29
last_activity_desc: "v1.7 + v1.8 Milestone-Close (manuell, jj-nativ). Beide Milestones archiviert: milestones/v1.7-{ROADMAP,REQUIREMENTS}.md + milestones/v1.8-{ROADMAP,REQUIREMENTS,MILESTONE-AUDIT}.md; MILESTONES.md + ROADMAP.md (collapsed) + PROJECT.md + RETROSPECTIVE.md nachgezogen; REQUIREMENTS.md entfernt (Inhalt war v1.7 → archiviert). override_closeout: Carry-over Deferred Items acknowledged. Snapshot-Schema-Version jetzt 12. Offen: User committet via jj + setzt Tags v1.7/v1.8; danach /gsd-new-milestone."
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (kein aktiver Milestone; v1.0–v1.8 alle archiviert/collapsed)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v1.8-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions); Frontend in `shifty-dioxus/CLAUDE.md` + `.planning/codebase/frontend/`
- **Last shipped/closed**: v1.8 Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX) (shipped + archiviert 2026-06-29, Phasen 27–28, 5 Pläne, 2/2 Requirements, Audit `passed`, override_closeout); zeitgleich v1.7 (Phasen 25–26, 10/10 Requirements) nachgeholt
- **Current milestone**: keiner — Planung des nächsten offen
- **Current focus**: `/gsd-new-milestone` (oder off-theme Backlog-Phase 999.1 via `/gsd-plan-phase 999.1`)
- **Snapshot-Schema-Version**: 12 (v1.7 Bump 10→11; v1.8 Bump 11→12)

## Current Position

Phase: keiner aktiv. v1.7 + v1.8 **geschlossen** am 2026-06-29 (manueller Close, jj-nativ — `milestone.complete`-CLI bewusst umgangen wg. Junk-Einträgen).
Status: Beide Milestones archiviert. v1.8 (Phasen 27–28, 5 Pläne) — Phase 27 (FE): gruppierter Freiwilligen-Selector (`grouped_selectable`/`PersonGroup`/`grouped_person_options`, Modal+FilterBar), `is_selectable_employee` NICHT gelockert (D-27-02). Phase 28 (BE+FE): signed Urlaubsanspruch-Offset (Delta, Tabelle `vacation_entitlement_offset` + Basic-Service + HR-gated REST CRUD + DI; `entitled_effective=round(base)+offset` mit API-level Hiding; FE-Inline-Editor HR-only) + Off-by-one-Proration-Fix + Snapshot-Bump 11→12. v1.7 (Phasen 25–26): Feiertags-Auto-Anrechnung derive-on-read + Stichtag-Toggle + VFA-Reduktion + Cross-Navigation, Snapshot-Bump 10→11. Alle Automatik-Gates grün (cargo test/clippy -D warnings, WASM-Build, 678 FE-Tests); v1.8 zusätzlich Live-HR-Browser-Smokes (`behavior_unverified: 0`).
Closeout: **override_closeout** für beide — Carry-over Deferred Items acknowledged (siehe unten). v1.8 hatte zusätzlich formalen Milestone-Audit (`passed`).
Last activity: 2026-06-29 — Milestone-Close v1.7 + v1.8: Archive geschrieben, ROADMAP collapsed, MILESTONES/PROJECT/RETROSPECTIVE nachgezogen, REQUIREMENTS.md entfernt. **Offen: User committet via jj + setzt Git-Tags v1.7/v1.8.**

Progress: [          ] kein aktiver Milestone — nächster via `/gsd-new-milestone`

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

## Shipped Milestones

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
- **Snapshot Versioning**: `CURRENT_SNAPSHOT_SCHEMA_VERSION` (aktuell 10) MUSS gebumpt werden, sobald sich `value_type`-Berechnung oder -Input-Set ändert. v1.7 erwartet Bump 10→11 (Holiday-Computation/Input-Set ändert sich durch HOL-01).
- **Clippy-Gate**: `cargo clippy --workspace -- -D warnings` ist Pflicht-Gate bei jedem Commit — `cargo test` allein reicht nicht.
- **Service-Tier-Konvention**: Basic Services konsumieren nur DAO/Permission/Transaction; Business-Logic Services kombinieren Aggregate. HOL-Logik gehört in `ReportingService` / Business-Logic-Tier.
- **Multi-Sprache (i18n)**: Alle benutzersichtbaren Texte in en/de/cs. HCFG-02 (Settings-UI) + NAV-01 (Cross-Links) betreffen neue Texte.
- **Offene Design-Fragen (v1.7, vor discuss-phase zu klären)**: materialize vs. derive-on-read für Feiertags-Anrechnung; VFA-Absence-Kategorien (nur Vacation oder auch SickLeave/UnpaidLeave); ob Stichtag auch VFA steuert; Konfliktregel HCFG-03; NAV-01 Deep-Link-Parameter.

### Roadmap Evolution

- Backlog-Phase 999.1 „Breaking/Major Dependency-Migration" angelegt (2026-06-28): off-theme zu v1.6, eskaliert aus Quick-Task 260627-vgo (stable cargo 1.95.0 kann kein `--breaking`; kein cargo-edit/nightly). Erster Task = Toolchain-Enabler. Seed: `.planning/phases/999.1-breaking-dependency-migration/SEED.md`.
- Milestone v1.7 + Phasen 25–26 angelegt (2026-06-28): 10 Requirements (HOL-01..03, VFA-01/02, HCFG-01..03, HSNAP-01, NAV-01) → 2 Phasen. Phase 25 = Axis-A (Reporting) + Settings-UI. Phase 26 = Axis-B (VFA) + Cross-Navigation.
- Milestone v1.8 + Phase 27 angelegt (2026-06-29): VOL-SEL-01 — Freiwillige (`is_paid=false`) in Abwesenheits-Selektoren auswählbar, gruppiert (optgroup Angestellte/Freiwillige) in AbsenceModal + AbsenceFilterBar. Reines Frontend; `is_selectable_employee` von `is_paid && !inactive` → `!inactive` lockern, `is_paid` wird Gruppierung. 2 neue i18n-Keys (de/en/cs). Offen für Planung: Kategorie-Set für Freiwillige. Dir: `.planning/phases/27-freiwillige-abwesenheitsliste-selector/`.
- Phase 28 angelegt (2026-06-29): VAC-OFFSET-01 — HR-Korrektur des Jahres-Urlaubsanspruchs via signed **Offset** (Delta, KEIN absoluter Override → überlebt Vertragsänderungen), pro Person+Jahr. `entitled_effective = round(berechnet) + offset`, wirkt auf `remaining_days` durch. HR-gekennzeichnet+editierbar in der Urlaubsübersicht, für User unsichtbar. Neue Tabelle `vacation_entitlement_offset` + HR-gated CRUD + Edit/Marker (`absences.rs:602–727`). Offen: User-Hiding UI-only vs API-level; Off-by-one-Begleit-Fix (`employee_work_details.rs:173`); Snapshot-Bump-Check (vermutlich nein). Dir: `.planning/phases/28-urlaubsanspruch-korrektur-offset/`.

## Session Continuity

**To resume work in a new session:**

1. Read `.planning/STATE.md` (this file)
2. Read `.planning/ROADMAP.md` (kein aktiver Milestone; v1.0–v1.8 archiviert; Backlog 999.1)
3. Read `.planning/PROJECT.md` (Charter + Current State; **keine aktive REQUIREMENTS.md** — wird beim nächsten Milestone via `/gsd-new-milestone` neu erstellt)
4. Read `.planning/MILESTONES.md` (Index inkl. v1.7 + v1.8)

**Offene Aktion (User):** Working-Copy via `jj` committen + Git-Tags `v1.7` / `v1.8` setzen (siehe Close-Zusammenfassung). Danach:

**Next command**: `/gsd-new-milestone` (nächsten Milestone-Charter aufsetzen) — oder off-theme Backlog-Phase 999.1 via `/gsd-plan-phase 999.1`.

---

*State updated: 2026-06-29 — **v1.7 + v1.8 Milestone-Close** (manuell, jj-nativ). Beide Milestones archiviert (milestones/v1.7-* + milestones/v1.8-*), ROADMAP collapsed, MILESTONES/PROJECT/RETROSPECTIVE nachgezogen, REQUIREMENTS.md entfernt. Snapshot-Schema-Version 12. Kein aktiver Milestone.*
