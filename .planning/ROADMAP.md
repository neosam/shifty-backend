# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (shipped 2026-05-07) — siehe [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)
- ✅ **v1.3 Frontend Abwesenheiten + UI-Closure-Restanten** — Phasen 8–13 (closed 2026-06-22) — siehe [`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md)
- ✅ **v1.4 Committed Voluntary Capacity** — Phasen 14–17 (shipped 2026-06-25) — siehe [`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md)
- ✅ **v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen** — Phasen 18–23 (shipped 2026-06-27) — siehe [`milestones/v1.5-ROADMAP.md`](milestones/v1.5-ROADMAP.md)
- ✅ **v1.6 Paid-Capacity-Durchsetzung & Konfiguration** — Phase 24 (shipped 2026-06-27) — siehe [`milestones/v1.6-ROADMAP.md`](milestones/v1.6-ROADMAP.md)
- ✅ **v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit** — Phasen 25–26 (shipped 2026-06-29) — siehe [`milestones/v1.7-ROADMAP.md`](milestones/v1.7-ROADMAP.md)
- ✅ **v1.8 Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX)** — Phasen 27–28 (shipped 2026-06-29) — siehe [`milestones/v1.8-ROADMAP.md`](milestones/v1.8-ROADMAP.md)

Aktiv: **keiner** — nächster Milestone via `/gsd-new-milestone` (oder off-theme Backlog-Phase 999.1 direkt via `/gsd-plan-phase 999.1`).

## Phases

<details>
<summary>✅ v1.8 Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX) (Phasen 27–28) — SHIPPED 2026-06-29</summary>

- [x] Phase 27: Freiwillige in Abwesenheitsliste auswählbar (FE) (1/1 plan) — VOL-SEL-01
- [x] Phase 28: Urlaubsanspruch-Korrektur via Offset (BE+FE) (4/4 plans) — VAC-OFFSET-01

Gruppierter Personen-Selector (optgroup Angestellte/Freiwillige) in AbsenceModal +
AbsenceFilterBar via gemeinsamem Helfer; signed Urlaubsanspruch-Offset pro Person+Jahr
(Delta, kein Override), HR-gekennzeichnet+editierbar, für User unsichtbar (API-level
Hiding) + Off-by-one-Proration-Fix + Snapshot-Bump 11→12. Audit `passed`.

Vollständige Phasen-Details, Decisions und Closeout:
[`milestones/v1.8-ROADMAP.md`](milestones/v1.8-ROADMAP.md) · [`milestones/v1.8-REQUIREMENTS.md`](milestones/v1.8-REQUIREMENTS.md) · [`milestones/v1.8-MILESTONE-AUDIT.md`](milestones/v1.8-MILESTONE-AUDIT.md)

</details>

<details>
<summary>✅ v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit (Phasen 25–26) — SHIPPED 2026-06-29</summary>

- [x] Phase 25: Feiertags-Auto-Anrechnung & Stichtag-Konfiguration (BE+FE) (4/4 plans) — HOL-01..03, HCFG-01..03, HSNAP-01
- [x] Phase 26: Freiwilligen-Abwesenheit & Cross-Navigation (BE+FE) (3/3 plans) — VFA-01/02, NAV-01

Feiertage werden automatisch (derive-on-read, identisch zu manuellem ExtraHours(Holiday))
ab konfigurierbarem Stichtag angerechnet; Urlaub von Freiwilligen reduziert die committed-
Zusage in der Jahresansicht (Feiertage bewusst nicht — Asymmetrie); bidirektionale
Deep-Links /absences ↔ Report. Snapshot-Bump 10→11.

Vollständige Phasen-Details, Success-Criteria und Requirements:
[`milestones/v1.7-ROADMAP.md`](milestones/v1.7-ROADMAP.md) · [`milestones/v1.7-REQUIREMENTS.md`](milestones/v1.7-REQUIREMENTS.md)

</details>

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
| 24 — Paid-Limit konfigurierbar & rollenbasiert (BE+FE) | v1.6 | 5/5 | Complete | 2026-06-27 |
| 25 — Feiertags-Auto-Anrechnung & Stichtag-Konfiguration (BE+FE) | v1.7 | 4/4 | Complete | 2026-06-28 |
| 26 — Freiwilligen-Abwesenheit & Cross-Navigation (BE+FE) | v1.7 | 3/3 | Complete | 2026-06-28 |
| 27 — Freiwillige in Abwesenheitsliste auswählbar (FE) | v1.8 | 1/1 | Complete | 2026-06-29 |
| 28 — Urlaubsanspruch-Korrektur via Offset (BE+FE) | v1.8 | 4/4 | Complete | 2026-06-29 |

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

*Last updated: 2026-06-29 — **v1.7 + v1.8 Milestone-Close**: beide Milestones archiviert (Phasen 25–28 → collapsed `<details>` + `milestones/v1.7-*` / `milestones/v1.8-*`). Kein aktiver Milestone — nächster Schritt `/gsd-new-milestone`.*
