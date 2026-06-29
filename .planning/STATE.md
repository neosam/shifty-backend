---
gsd_state_version: 1.0
milestone: v1.8
milestone_name: Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX)
current_phase: 28
current_phase_name: Urlaubsanspruch-Korrektur via Offset (BE+FE)
status: executed
last_updated: "2026-06-29T08:25:00.000Z"
last_activity: 2026-06-29
last_activity_desc: "Phase 28 (VAC-OFFSET-01) EXECUTED — 4 Pläne (data layer, off-by-one + Snapshot-Bump 11→12, integration + API-hiding + REST CRUD + DI, FE inline editor). Integrated-Gates grün: cargo test --workspace + clippy --workspace -D warnings (0 Fehler/0 Warnungen), WASM-Build + 678 FE-Tests. Browser-Smoke (HR-Offset-Roundtrip) als Human-UAT offen. Beide v1.8-Phasen executed → Milestone-Lifecycle (audit/complete/cleanup) ausstehend."
progress:
  total_phases: 2
  completed_phases: 2
  total_plans: 5
  completed_plans: 5
  percent: 100
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

Phase: 27 + 28 BEIDE EXECUTED (je Browser-Smoke offen). v1.8-Milestone-Lifecycle ausstehend.
Plan: 27 → 1/1 complete; 28 → 4/4 complete
Status: v1.8 (2 Phasen, autonom) — beide ausgeführt, alle Automatik-Gates grün, jj-nativ pro Task committet. Phase 27 (FE): gruppierter Freiwilligen-Selector (`grouped_selectable`/`PersonGroup`/`grouped_person_options`, Modal+FilterBar), `is_selectable_employee` NICHT gelockert (D-27-02). Phase 28 (BE+FE): signed Urlaubsanspruch-Offset (Delta, neue Tabelle `vacation_entitlement_offset` + Basic-Service + HR-gated REST CRUD + DI; `entitled_effective=round(base)+offset` mit API-level Hiding; FE-Inline-Editor HR-only) + Off-by-one-Proration-Fix + Snapshot-Bump 11→12 (VacationEntitlement). Integrated-Gates: cargo test --workspace ✓, clippy --workspace -D warnings ✓, WASM-Build ✓, 678 FE-Tests ✓. 2 Browser-Smokes deferred (Human-UAT). v1.7 (25–26) bleibt complete & verified — auch dort Milestone-Close offen.
Last activity: 2026-06-29 — Phase 28 (4 Pläne) ausgeführt + jj-nativ committet; integrierter Backend+FE-Gate grün.

Progress: [██████████] 100% (beide v1.8-Phasen executed; Lifecycle + 2 Smokes offen)

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
| human_uat | Phase 27: Browser-Smoke Freiwilligen-Selector (v1.8) | pending | 2026-06-29 deferred (User: weiter zu Phase 28). Logik voll unit-getestet (5 Tests) + WASM-Build grün; offen nur visueller Roundtrip: Freiwilliger gruppiert in Modal+FilterBar, Anlege-Pfad, de/cs-Labels. Resume: `/gsd-verify-work 27`. Braucht aktive is_paid=false-Person in Dev-DB |
| human_uat | Phase 28: Browser-Smoke HR-Offset-Roundtrip (v1.8) | pending | 2026-06-29 deferred. Backend+FE voll automatik-verifiziert (offset_calc/delta/api_hiding + 678 FE-Tests). Offen: HR-Detail zeigt „berechnet {n} + Offset [x]" → Box=Effektiv; +1 setzen, blur/Enter persistiert nach Reload; User-Self-View zeigt NUR Effektivwert (kein offset_days in roher API-Antwort). Resume: `/gsd-verify-work 28` |

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
