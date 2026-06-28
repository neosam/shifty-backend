---
gsd_state_version: 1.0
milestone: v1.8
milestone_name: Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX)
current_phase: 27
current_phase_name: Freiwillige in Abwesenheitsliste auswählbar (FE)
status: planning
last_updated: "2026-06-29T00:00:00.000Z"
last_activity: 2026-06-29
last_activity_desc: "Milestone v1.8: Phase 27 (VOL-SEL-01, gruppierter Selector, FE) + Phase 28 (VAC-OFFSET-01, signed Urlaubsanspruch-Offset pro Person+Jahr, HR-gekennzeichnet/User-unsichtbar, BE+FE) angelegt inkl. Konzept + SEED.md. Nächster Schritt: /gsd-plan-phase 27 bzw. 28."
progress:
  total_phases: 2
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (Phase 27 aktiv / v1.8; Phasen 25–26 v1.7 complete; v1.0–v1.6 archived)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v1.6-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions); Frontend in `shifty-dioxus/CLAUDE.md` + `.planning/codebase/frontend/`
- **Last shipped/closed**: v1.6 Paid-Capacity-Durchsetzung & Konfiguration (shipped + archiviert 2026-06-27/28, Phase 24, 5 Pläne, 7/7 must-haves, override_closeout)
- **Current milestone**: v1.8 Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX)
- **Current focus**: Phase 27 — Freiwillige in Abwesenheitsliste auswählbar (FE) (`/gsd-plan-phase 27`); danach Phase 28 (`/gsd-plan-phase 28`)

## Current Position

Phase: 27 + 28 angelegt, beide noch nicht geplant
Plan: 0/0 — `/gsd-plan-phase 27` bzw. `/gsd-plan-phase 28`
Status: v1.8 (2 Phasen) — Konzepte + UX-Entscheidungen + SEED.md erfasst. Phase 27 = gruppierter Freiwilligen-Selector (FE). Phase 28 = signed Urlaubsanspruch-Offset (Delta, HR-gekennzeichnet/User-unsichtbar, BE+FE). v1.7 (Phasen 25–26) bleibt complete & verified — Milestone-Close noch offen, falls gewünscht.
Last activity: 2026-06-29 — Phase 28 (VAC-OFFSET-01) zur Roadmap hinzugefügt; Offset-Mechanismus statt Override (überlebt Vertragsänderungen), Backend rundet bei `vacation_balance.rs:191`.

Progress: [──────────] 0% (Phasen 27 + 28 not planned)

## Deferred Items

Erneut acknowledged + deferred beim **v1.6-Milestone-Close am 2026-06-28** (User-Entscheidung override_closeout). Carry-over aus v1.5/v1.4:

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

## Shipped Milestones

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
2. Read `.planning/ROADMAP.md` (Phasen 25–26 aktiv)
3. Read `.planning/REQUIREMENTS.md` (v1.7-Scope, REQ-IDs, Coverage 10/10)
4. Read `.planning/PROJECT.md` (v1.7 Key Context + Referenz-Logik-Koordinaten)

**Next command**: `/gsd-plan-phase 27` (FE, kleiner) bzw. `/gsd-plan-phase 28` (BE+FE, größer) — optional vorher `/gsd-new-milestone` für v1.8-Charter, und/oder v1.7-Milestone-Close (audit → complete → cleanup).

---

*State updated: 2026-06-29 — Milestone v1.8 mit Phase 27 (VOL-SEL-01, FE) + Phase 28 (VAC-OFFSET-01, BE+FE) angelegt. Konzepte + UX-Entscheidungen in ROADMAP.md + SEED.md erfasst. Phase 28: Offset-Mechanismus (Delta) für HR-Urlaubsanspruch-Korrektur, HR-gekennzeichnet/User-unsichtbar.*
