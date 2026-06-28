---
gsd_state_version: 1.0
milestone: v1.7
milestone_name: Automatische Feiertage & Freiwilligen-Abwesenheit
status: planning
last_updated: "2026-06-28T12:00:00.000Z"
last_activity: 2026-06-28
progress:
  total_phases: 2
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (Phasen 25–26 aktiv; v1.0–v1.6 archived)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v1.6-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions); Frontend in `shifty-dioxus/CLAUDE.md` + `.planning/codebase/frontend/`
- **Last shipped/closed**: v1.6 Paid-Capacity-Durchsetzung & Konfiguration (shipped + archiviert 2026-06-27/28, Phase 24, 5 Pläne, 7/7 must-haves, override_closeout)
- **Current milestone**: v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit
- **Current focus**: Phase 25 planen — `/gsd-plan-phase 25`

## Current Position

Phase: 25 of 26 (Feiertags-Auto-Anrechnung & Stichtag-Konfiguration)
Plan: —
Status: Context gathered — ready to plan
Last activity: 2026-06-28 — Phase 25 CONTEXT.md erstellt (discuss: 4 Entscheidungen — derive-on-read, manuell-gewinnt, toggle-value-Spalte, holiday_hours()-Helper)

Progress: [░░░░░░░░░░] 0%

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

## Session Continuity

**To resume work in a new session:**

1. Read `.planning/STATE.md` (this file)
2. Read `.planning/ROADMAP.md` (Phasen 25–26 aktiv)
3. Read `.planning/REQUIREMENTS.md` (v1.7-Scope, REQ-IDs, Coverage 10/10)
4. Read `.planning/PROJECT.md` (v1.7 Key Context + Referenz-Logik-Koordinaten)

**Next command**: `/gsd-plan-phase 25`

---

*State updated: 2026-06-28 — v1.7 Roadmap erstellt. Phase 25 (HOL-01/02/03 + HCFG-01/02/03 + HSNAP-01, BE+FE) und Phase 26 (VFA-01/02 + NAV-01, BE+FE) definiert. 10/10 Requirements gemappt. Nächster Schritt: `/gsd-plan-phase 25`.*
