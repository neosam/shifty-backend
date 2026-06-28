# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (shipped 2026-05-07) — siehe [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)
- ✅ **v1.3 Frontend Abwesenheiten + UI-Closure-Restanten** — Phasen 8–13 (closed 2026-06-22; geliefert: 8, 8.2, 8.4, 8.5, 8.6, 9; 8.1/11 ⊘ superseded; **8.3/10/12/13 bewusst aufgegeben**) — siehe [`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md)
- ✅ **v1.4 Committed Voluntary Capacity** — Phasen 14–17 (shipped 2026-06-25) — siehe [`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md)
- ✅ **v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen** — Phasen 18–23 (shipped 2026-06-27) — siehe [`milestones/v1.5-ROADMAP.md`](milestones/v1.5-ROADMAP.md)
- ◆ **v1.6 Paid-Capacity-Durchsetzung & Konfiguration** — Phase 24 (aktiv, gestartet 2026-06-27)

## Phases

### v1.6 Paid-Capacity-Durchsetzung & Konfiguration (active)

**Milestone-Goal:** Die Paid-Capacity-Grenze (`max_paid_employees` pro Slot/Woche) wird
von einem rein visuellen Soft-Hinweis (v1.1/Phase 5, Phase 23) zu einem global
konfigurierbar durchsetzbaren Limit — admin-schaltbar hart/weich, im harten Modus
rollenbasiert (nur Shiftplanner darf überziehen), mit deutlicherer Overage-Anzeige.

- [ ] **Phase 24: Paid-Limit konfigurierbar & rollenbasiert durchsetzen** (Backend+Frontend) — Globaler System-Toggle „Paid-Limit hart/weich", rollenbasierte Überschreitung (nur Shiftplanner-Rolle darf über das Limit buchen) und eine deutlichere Overage-Anzeige im Wochenplan. Baut auf der Phase-23-UI + dem v1.1-Backend (`Warning::PaidEmployeeLimitExceeded`) auf.

  **Goal:** Ein global konfigurierbarer Modus bestimmt, ob das Buchen über `max_paid_employees` hinaus (a) hart blockiert wird (außer für die Shiftplanner-Rolle) oder (b) wie heute nur eine nicht-blockierende Warnung erzeugt; zusätzlich ist die Overage-Situation im Wochenplan deutlicher (persistente Warn-Sektion über dem Plan) sichtbar — alles für En/De/Cs lokalisiert und getestet. Mitgefixt: das Buchungs-Permission-Gate (Shiftplanner ∨ self statt HR ∨ self).
  **Scope (aus Diskussion 2026-06-27, D-24-01..08):**
  - D-24-01/01a/07: Globaler Toggle „Paid-Limit-Modus = hart | weich" über den bestehenden `ToggleService` (Key `paid_limit_hard_enforcement`, neue Seed-Migration, `enabled=0`=weich); Default = weich (keine Regression). Bewusst NICHT `feature_flag`.
  - D-24-02 + Grenzregel: Im harten Modus blockiert das Backend (`book_slot_with_conflict_check`) das Buchen, das den bezahlten Count strikt über das Limit brächte — außer der Akteur hat `SHIFTPLANNER_PRIVILEGE`; nur bezahlte zählen; keine Bestandsbuchungen werden rückwirkend angefasst.
  - D-24-08: Pre-Persist-Check — `ShiftplanEditService` bekommt `ToggleService`-Dep; die Zählung/Prüfung wird vor `booking_service.create` gezogen.
  - D-24-04: Buchungs-Gate `HR ∨ self` → `Shiftplanner ∨ self` (Permission-Bugfix).
  - D-24-05: Neuer, unterscheidbarer ServiceError (`PaidLimitExceeded`, → HTTP 409, NICHT 403) + lokalisierte Inline-Meldung am Slot.
  - D-24-06: Neue admin-gated `/settings/`-Seite mit nur diesem einen Schalter.
  - D-24-03: Persistente Overage-Warn-Sektion über dem Schichtplan, alle Rollen, rein clientseitig.
  **Requirements:** Contract = D-24-01, D-24-01a, D-24-02, D-24-03, D-24-04, D-24-05, D-24-06, D-24-07, D-24-08 + strikt-größer-Grenzregel (keine formalen REQ-IDs; in 24-CONTEXT.md definiert, je Plan als `requirements`-Tags referenziert).
  **Depends on:** Phase 23 (Slot-Paid-Capacity-UI) ✅ + v1.1/Phase 5 (`Warning::PaidEmployeeLimitExceeded`, Backend-Buchungspfad).
  **Plans:** 5 plans (2 Waves; Wave 1 parallel: 24-01 Backend-Contract + 24-03 i18n; Wave 2 parallel: 24-02 Backend-Enforcement, 24-04 Settings-Seite, 24-05 Shiftplan-UI — keine Datei-Überschneidung je Wave)
  Plans:
  - [x] 24-01-PLAN.md — ServiceError::PaidLimitExceeded (→409) + Toggle-Seed-Migration (D-24-01/01a/05/07)
  - [x] 24-02-PLAN.md — ToggleService-DI + Pre-Persist-Hard-Block + Gate-Fix HR→Shiftplanner + Tests (D-24-02/04/08 + Grenzregel)
  - [x] 24-03-PLAN.md — 9 neue i18n-Keys En/De/Cs + Present-in-all-locales-Test
  - [x] 24-04-PLAN.md — admin-gated /settings/-Seite mit einem Paid-Limit-Toggle + Toggle-REST-Client (D-24-06)
  - [x] 24-05-PLAN.md — Inline-Hard-Block-Meldung am Slot (D-24-05) + persistente Overage-Sektion (D-24-03)

<details>
<summary>✅ v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen (Phasen 18–23) — SHIPPED 2026-06-27</summary>

- [x] Phase 18: Report-/Balance-Korrektheit (BE) (2/2 plans) — UV-04, UV-05
- [x] Phase 19: Convert-Dialog UX (FE+BE) (2/2 plans) — UV-01, UV-02
- [x] Phase 20: Absences-Indikator & Jahres-Histogramm (FE) (2/2 plans) — UV-03, YV-01/02/03
- [x] Phase 21: Tabellen-Lesbarkeit (FE) (1/1 plan) — UI-01, UI-02
- [x] Phase 22: Mitarbeiter-Statistik HR (BE+FE) (2/2 plans) — STAT-01, STAT-02
- [x] Phase 23: Frontend Slot Paid-Capacity UI (FE) (2/2 plans)

Vollständige Phasen-Details, Success-Criteria und Requirements:
[`milestones/v1.5-ROADMAP.md`](milestones/v1.5-ROADMAP.md) · [`milestones/v1.5-REQUIREMENTS.md`](milestones/v1.5-REQUIREMENTS.md)

</details>

<details>
<summary>✅ v1.4 Committed Voluntary Capacity (Phasen 14–17) — SHIPPED 2026-06-25</summary>

- [x] Phase 14: Data-model foundation (backend) (2/2 plans) — CVC-01/02/03
- [x] Phase 15: Reporting no-double-count (Achse B only, kein Snapshot-Bump) (2/2 plans) — CVC-04/05/06
- [x] Phase 16: Jahresansicht display (3/3 plans) — CVC-07/08
- [x] Phase 17: Contract editor input + „alle"-Filter / unpaid-volunteer path (4/4 plans) — CVC-09/10

Vollständige Phasen-Details, Success-Criteria und Audit:
[`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md) · [`milestones/v1.4-MILESTONE-AUDIT.md`](milestones/v1.4-MILESTONE-AUDIT.md)

</details>

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1 — Absence Domain Foundation | v1.0 | 5/5 | Complete | 2026-05-01 |
| 2 — Reporting Integration & Snapshot Versioning | v1.0 | 4/4 | Complete | 2026-05-02 |
| 3 — Booking & Shift-Plan Konflikt-Integration | v1.0 | 6/6 | Complete | 2026-05-02 |
| 4 — Migration & Cutover | v1.0 | 8/8 | Complete | 2026-05-03 |
| 5 — Slot Paid Capacity Warning | v1.1 | 6/6 | Complete | 2026-05-04 |
| 6 — rest-types Unification & Frontend Compile-Through | v1.2 | 5/5 | Complete | 2026-05-07 |
| 7 — Runtime Smoke & Regression Safety | v1.2 | 1/1 | Complete | 2026-05-07 |
| 8–13 — v1.3 (siehe milestones/v1.3-ROADMAP.md) | v1.3 | — | Closed | 2026-06-22 |
| 14 — Data-model foundation (backend) | v1.4 | 2/2 | Complete | 2026-06-23 |
| 15 — Reporting no-double-count (KEIN Snapshot-Bump) | v1.4 | 2/2 | Complete | 2026-06-24 |
| 16 — Jahresansicht display | v1.4 | 3/3 | Complete | 2026-06-24 |
| 17 — Contract editor input + „alle"-Filter / unpaid-volunteer | v1.4 | 4/4 | Complete | 2026-06-24 |
| 18 — Report-/Balance-Korrektheit (BE) | v1.5 | 2/2 | Complete | 2026-06-26 |
| 19 — Convert-Dialog UX (FE+BE) | v1.5 | 2/2 | Complete | 2026-06-26 |
| 20 — Absences-Indikator & Jahres-Histogramm (FE) | v1.5 | 2/2 | Complete | 2026-06-26 |
| 21 — Tabellen-Lesbarkeit (FE) | v1.5 | 1/1 | Complete | 2026-06-26 |
| 22 — Mitarbeiter-Statistik HR (BE+FE) | v1.5 | 2/2 | Complete | 2026-06-26 |
| 23 — Frontend: Slot Paid-Capacity UI (FE) | v1.5 | 2/2 | Complete | 2026-06-27 |
| 24 — Paid-Limit konfigurierbar & rollenbasiert (BE+FE) | v1.6 | 5/5 | Complete   | 2026-06-27 |

## Backlog

Ungeplante / off-theme Arbeit, die NICHT zum aktiven Milestone gehört. Vor Ausführung
in einen Milestone promoten oder per `/gsd-plan-phase 999.1` direkt planen.

- [ ] **Phase 999.1: Breaking/Major Dependency-Migration** (Backend + Frontend, Maintenance) — Alle direkten Deps mit verfügbaren Major-Releases über beide Cargo-Workspaces (Backend-Root + `shifty-dioxus/`, 9 Member-Crates) auf den neuen Major heben (Cargo.toml-Constraint-Edits + Code-/API-Migration). **Off-theme zu v1.6** (Paid-Capacity) → bewusst Backlog.

  **Goal:** Reproduzierbares Breaking-Update-Tooling etabliert und alle tragbaren Major-Bumps migriert, mit grünen Gates über beide Workspaces — ohne die heiklen Pins (dioxus 0.6.x) ungefragt anzufassen.

  **Context:** Quick-Task `260627-vgo` hat die **semver-kompatible** Baseline bereits geliefert (nur Cargo.lock, alle Gates grün). Offen ist NUR der Breaking/Major-Teil, der dort eskaliert wurde, weil die gepinnte **stable cargo 1.95.0** kein `cargo update --breaking` kann (nightly-only) und weder `cargo-edit` (`cargo upgrade`) noch `cargo-outdated` noch `+nightly` verfügbar sind.

  **Scope / grobe Wave-Struktur:**
  - Task 1 — Toolchain-Enabler: nightly-Toolchain bzw. `cargo-edit`/`cargo-outdated` ins `flake.nix` aufnehmen, sodass `cargo update --breaking` oder `cargo upgrade --incompatible` reproduzierbar laufen.
  - Task 2 — Major-Bump-Inventar: welche direkten Deps, welcher Sprung, Changelog-/Breaking-Risiko (beide Workspaces).
  - Task 3 — iterativ pro Major migrieren mit Gates: Backend `cargo build` + `cargo clippy --workspace -- -D warnings` + `cargo test --workspace`; Frontend `cargo build --target wasm32-unknown-unknown` (nix-shell -p openssl pkg-config lld) + `cargo test`.

  **Constraints:**
  - **dioxus-Major** (0.6.x-Pin) NUR mit expliziter User-Freigabe — dx-CLI-0.7-Inkompatibilität dokumentiert (App startet nicht + Design gestrippt).
  - `flake.lock` Nix-Inputs sind NICHT Teil dieser Phase (separater Maintenance-Job).
  - jj-Repo: User committet manuell, keine git-Fallbacks.

  **Depends on:** Quick-Task `260627-vgo` (compatible baseline) ✅
  **Plans:** noch nicht geplant — `/gsd-plan-phase 999.1`

---

*Last updated: 2026-06-28 — **Backlog-Phase 999.1 (Breaking/Major Dependency-Migration)** angelegt (off-theme zu v1.6, eskaliert aus Quick-Task 260627-vgo). v1.5 archiviert; v1.6/Phase 24 ausgeführt + verifiziert, Milestone-Close ausstehend.*
