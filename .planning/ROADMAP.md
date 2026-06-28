# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (shipped 2026-05-07) — siehe [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)
- ✅ **v1.3 Frontend Abwesenheiten + UI-Closure-Restanten** — Phasen 8–13 (closed 2026-06-22; geliefert: 8, 8.2, 8.4, 8.5, 8.6, 9; 8.1/11 ⊘ superseded; **8.3/10/12/13 bewusst aufgegeben**) — siehe [`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md)
- ✅ **v1.4 Committed Voluntary Capacity** — Phasen 14–17 (shipped 2026-06-25) — siehe [`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md)
- ✅ **v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen** — Phasen 18–23 (shipped 2026-06-27) — siehe [`milestones/v1.5-ROADMAP.md`](milestones/v1.5-ROADMAP.md)
- ✅ **v1.6 Paid-Capacity-Durchsetzung & Konfiguration** — Phase 24 (shipped 2026-06-27) — siehe [`milestones/v1.6-ROADMAP.md`](milestones/v1.6-ROADMAP.md)
- 🚧 **v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit** — Phasen 25–26 (active 2026-06-28)

## Phases

### v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit (Phasen 25–26) — ACTIVE 2026-06-28

**Milestone Goal:** Feiertage werden automatisch (statt manuell pro Mitarbeiter) im Report angerechnet, und Urlaub von Freiwilligen verzerrt die Jahresansicht nicht mehr.

- [ ] **Phase 25: Feiertags-Auto-Anrechnung & Stichtag-Konfiguration** — Vollständige Feiertags-Automatik mit konfigurierbarem Aktivierungsstichtag: auto-Anrechnung in `reporting.rs` (Wirkung identisch zu manuellem ExtraHours(Holiday)), Konflikt-/Doppelzähl-Schutz, Admin-Settings-UI für den Stichtag und ggf. Snapshot-Schema-Bump (BE+FE).
- [ ] **Phase 26: Freiwilligen-Abwesenheit & Cross-Navigation** — Urlaub von Freiwilligen reduziert ihre committed-Zusage 🎯 in der Jahresansicht (`booking_information.rs::get_weekly_summary`); bidirektionale Deep-Links zwischen `/absences` und Mitarbeiterreport/Jahresansicht (BE+FE).

<details>
<summary>✅ v1.6 Paid-Capacity-Durchsetzung & Konfiguration (Phase 24) — SHIPPED 2026-06-27</summary>

- [x] Phase 24: Paid-Limit konfigurierbar & rollenbasiert durchsetzen (BE+FE) (5/5 plans) — D-24-01..08 + strikt-größer-Grenzregel

Globaler hart/weich-Toggle (`paid_limit_hard_enforcement`, Default weich), pre-persist
Hard-Block (Shiftplanner-Bypass, HTTP 409), admin-gated `/settings/`-Seite, persistente
Overage-Sektion für alle Rollen, Permission-Gate-Fix HR→Shiftplanner.

Vollständige Phasen-Details, Decisions und Closeout:
[`milestones/v1.6-ROADMAP.md`](milestones/v1.6-ROADMAP.md)

</details>

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

## Phase Details

### Phase 25: Feiertags-Auto-Anrechnung & Stichtag-Konfiguration

**Goal**: Feiertage werden automatisch und korrekt im Mitarbeiterreport angerechnet — mit identischer Wirkung zu einem manuellen ExtraHours(Holiday)-Eintrag — und ein Admin kann den Aktivierungsstichtag über eine Settings-UI setzen.
**Depends on**: Phase 24 (Settings-UI-Pattern aus v1.6; Toggle-/Konfig-Infrastruktur vorhanden)
**Requirements**: HOL-01, HOL-02, HOL-03, HCFG-01, HCFG-02, HCFG-03, HSNAP-01
**Success Criteria** (what must be TRUE):

  1. Ein Mitarbeiter mit laut Vertrag am betreffenden Wochentag arbeitendem Feiertag hat im Report denselben `holiday_hours`-Wert wie bei einem äquivalenten manuellen ExtraHours(Holiday)-Eintrag — verifiziert per Vergleichstest (HOL-01, HOL-02).
  2. Feiertage vor dem konfigurierten "aktiv ab"-Datum werden von der Automatik nicht angerechnet; bestehende manuelle Einträge und historische Snapshots bleiben davon unberührt (HCFG-01).
  3. Ein Admin kann das "aktiv ab"-Datum in der admin-gated Settings-UI setzen, ändern und nach Seitenreload wiederfinden; alle Texte sind in de/en/cs übersetzt (HCFG-02).
  4. Hat ein Feiertag bereits einen manuellen ExtraHours(Holiday)-Eintrag, erscheint er nicht doppelt im Report — Konfliktregel greift (HCFG-03).
  5. `paid_hours`, `committed_voluntary_hours` und `volunteer_hours` in der Jahresansicht sind durch die Feiertags-Automatik unverändert (HOL-03 Regressions-Guard); `CURRENT_SNAPSHOT_SCHEMA_VERSION` ist bei Bedarf auf 11 gebumpt (HSNAP-01).

**Plans**: 4/4 plans complete
**Wave 1**

- [x] 25-01-PLAN.md — Toggle `value`-Spalte Infrastruktur (Migration + DAO + Service + REST, HCFG-01/02 Backend)

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 25-02-PLAN.md — Reporting derive-on-read (3 Injektionspunkte) + main.rs-DI-Fix + Snapshot-Bump 10→11 (HOL-01/02/03, HCFG-01/03, HSNAP-01)
- [x] 25-03-PLAN.md — Frontend Settings-Datumsfeld „aktiv ab" + i18n de/en/cs (HCFG-02)

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 25-04-PLAN.md — Feiertags-Auto-Anrechnung Tests (HOL-01/02/03, HCFG-01/03)

**UI hint**: yes

### Phase 26: Freiwilligen-Abwesenheit & Cross-Navigation

**Goal**: Urlaub von Freiwilligen verzerrt die Jahresansicht nicht mehr (committed-Zusage wird für Abwesenheitswochen korrekt reduziert), und Benutzer können per Deep-Link direkt zwischen Abwesenheitsansicht und Mitarbeiterreport navigieren.
**Depends on**: Phase 25 (Feiertags-Guard HOL-03 muss stabil sein, damit VFA-02-Asymmetrie sauber verifizierbar ist)
**Requirements**: VFA-01, VFA-02, NAV-01
**Success Criteria** (what must be TRUE):

  1. In der Jahresansicht zeigt ein Freiwilliger (`is_paid=false`, `committed_voluntary>0`) für Wochen mit Urlaub/Abwesenheit eine niedrigere committed-Zusage 🎯 als für Wochen ohne Abwesenheit (VFA-01).
  2. Feiertage senken die committed-Zusage eines Freiwilligen nicht — die Asymmetrie ist per Regressionstest abgesichert (VFA-02).
  3. Von der Jahresansicht/Mitarbeiterreport führt ein Link direkt zur Abwesenheitsansicht des jeweiligen Mitarbeiters (Sales: eigene Ansicht; HR: Mitarbeiter-Filter vorbelegt); alle Beschriftungen in de/en/cs (NAV-01a).
  4. Von der Abwesenheitsansicht führt ein Link pro Mitarbeiter direkt zur Jahresansicht/Mitarbeiterreport desselben Mitarbeiters; alle Beschriftungen in de/en/cs (NAV-01b).

**Plans**: TBD
**UI hint**: yes

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
| 25 — Feiertags-Auto-Anrechnung & Stichtag-Konfiguration (BE+FE) | v1.7 | 4/4 | Complete   | 2026-06-28 |
| 26 — Freiwilligen-Abwesenheit & Cross-Navigation (BE+FE) | v1.7 | 0/? | Not started | - |

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

*Last updated: 2026-06-28 — **v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit** Roadmap erstellt (Phasen 25–26, 10/10 Requirements gemappt). Phase 25: HOL-01/02/03 + HCFG-01/02/03 + HSNAP-01 (BE+FE). Phase 26: VFA-01/02 + NAV-01 (BE+FE). Nächster Schritt: `/gsd-plan-phase 25`.*
