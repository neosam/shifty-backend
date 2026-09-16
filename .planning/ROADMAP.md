# Roadmap: Shifty Backend

## Milestones

- ✅ **v2.6** — Phasen 54–57 (shipped 2026-07-31, deployed) — Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter ([`milestones/v2.6-ROADMAP.md`](milestones/v2.6-ROADMAP.md))
- ✅ **v2.5** — Phasen 52–53 (shipped 2026-07-06) — Weekly-Overview Performance & Freiwilligen-Abwesenheiten ([`milestones/v2.5-ROADMAP.md`](milestones/v2.5-ROADMAP.md))
- ✅ **v2.4** — Phase 51 (shipped 2026-07-05) — Kurzer-Tag-Slot-Kürzung ([`milestones/v2.4-ROADMAP.md`](milestones/v2.4-ROADMAP.md))
- ✅ **v2.3** — Phasen 49–50 (shipped 2026-07-04) — PDF-Export: Browser-Look & Download-Button ([`milestones/v2.3-ROADMAP.md`](milestones/v2.3-ROADMAP.md))
- ✅ **v2.2** — Phasen 43–48 (shipped 2026-07-03) — Aufräumen, WebDAV-Export & Wochentag-Muster ([`milestones/v2.2-ROADMAP.md`](milestones/v2.2-ROADMAP.md))
- ✅ **v2.1** — Phasen 39–42 (shipped 2026-07-02) — Schichtplan- & Reporting-Erweiterungen ([`milestones/v2.1-ROADMAP.md`](milestones/v2.1-ROADMAP.md))
- ✅ **v1.11** — Phasen 36–38 (shipped 2026-07-01) — Stabilisierung & UX-Politur ([`milestones/v1.11-ROADMAP.md`](milestones/v1.11-ROADMAP.md))
- ✅ **v1.10** — Phasen 33–35 (shipped 2026-06-30) — Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz ([`milestones/v1.10-ROADMAP.md`](milestones/v1.10-ROADMAP.md))
- ✅ **v1.9** — Phasen 29–32 (shipped 2026-06-29) — Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation ([`milestones/v1.9-ROADMAP.md`](milestones/v1.9-ROADMAP.md))
- ✅ **v1.8** — Phasen 27–28 (shipped 2026-06-29) — Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX) ([`milestones/v1.8-ROADMAP.md`](milestones/v1.8-ROADMAP.md))
- ✅ **v1.7** — Phasen 25–26 (shipped 2026-06-29) — Automatische Feiertage & Freiwilligen-Abwesenheit ([`milestones/v1.7-ROADMAP.md`](milestones/v1.7-ROADMAP.md))
- ✅ **v1.6** — Phase 24 (shipped 2026-06-27) — Paid-Capacity-Durchsetzung & Konfiguration ([`milestones/v1.6-ROADMAP.md`](milestones/v1.6-ROADMAP.md))
- ✅ **v1.5** — Phasen 18–23 (shipped 2026-06-27) — Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen ([`milestones/v1.5-ROADMAP.md`](milestones/v1.5-ROADMAP.md))
- ✅ **v1.4** — Phasen 14–17 (shipped 2026-06-25) — Committed Voluntary Capacity ([`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md))
- ✅ **v1.3** — Phasen 8–13 (closed 2026-06-22) — Frontend Abwesenheiten + UI-Closure-Restanten ([`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md))
- ✅ **v1.2** — Phasen 6–7 (shipped 2026-05-07) — Frontend rest-types Konsolidierung ([`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md))
- ✅ **v1.1** — Phase 5 (shipped 2026-05-04) — Slot Capacity & Constraints ([`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md))
- ✅ **v1.0** — Phasen 1–4 (shipped 2026-05-03) — Range-Based Absence Management ([`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md))

Vollständiger historischer Index: [`MILESTONES.md`](MILESTONES.md).

**Kein aktiver Milestone.** Nächste Iteration via `/gsd-new-milestone`.

**Kern-Scope für den nächsten Milestone** (User-Festlegung 2026-09-15):
Benutzerverwaltung und Sales-Person-Verwaltung in der UI zusammenlegen — heute
zerfällt dieselbe Person in drei Detailseiten (`/user_details`,
`/sales_person_details`, `/employees/:id`), was Nutzer verwirrt. Dafür bekommt
der Eintrag ein **eigenes Fachobjekt `Person`** als Aggregat über User und
SalesPerson (Entscheidung 2026-09-15) — also eine Data-Model-first-Phase mit
Migration und Backfill, nicht nur UI. Ist-Zustand, Blast Radius und offene
Modellierungsfragen:
[`seeds/personenverwaltung-zusammenlegen.md`](seeds/personenverwaltung-zusammenlegen.md).

**Zweiter Scope-Punkt** (User-Festlegung 2026-09-15): **Sollstunden bei
dynamischen Verträgen** fehlen in der Übersicht unterhalb des Schichtplans.
`weight_for_week` setzt `expected_hours` bei `is_dynamic` hart auf 0, damit das
Soll die Balance nicht beeinflusst — für die Planung soll es aber angezeigt und
vom Backend geliefert werden. Lösungsrichtung und die Falle beim naheliegenden
Fix: [`todos/pending/2026-09-15-sollstunden-fehlen-bei-dynamischen-vertraegen-unter-schichtplan.md`](todos/pending/2026-09-15-sollstunden-fehlen-bei-dynamischen-vertraegen-unter-schichtplan.md).

## Phasen

<details>
<summary>✅ v2.6 Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter (Phasen 54–57) — SHIPPED 2026-07-31, deployed</summary>

- [x] Phase 54: Data-Model + Voluntary Statistics (F1 + F2) — 9/9 Pläne
- [x] Phase 54.5: Voluntary-Soll Absence-Aware-Angleichung an VFA-01 (Hotfix v2.6.1, INSERTED) — 2/2 Pläne
- [x] Phase 55: Manuelle Umbuchung + HR-Alert-Modal (F3 + F5) — 6/6 Pläne
- [x] Phase 56: Wochen-Cron + Rollout-Backfill (F4) — 6/6 Pläne
- [x] Phase 57: F14-Rebooking-Wirksamkeit — Requirements-Korrektur + Balance-Ausgleich (INSERTED) — 6/6 Pläne

Detail-Archiv: [`milestones/v2.6-ROADMAP.md`](milestones/v2.6-ROADMAP.md), [`milestones/v2.6-phases/`](milestones/v2.6-phases/), Audit [`milestones/v2.6-MILESTONE-AUDIT.md`](milestones/v2.6-MILESTONE-AUDIT.md).

</details>

Ältere Milestones: siehe die jeweiligen `milestones/v*-ROADMAP.md` (oben verlinkt).

## Vorbereitete Phasen (noch keinem Milestone zugeordnet)

Diese Phasen sind ausgearbeitet, gehören aber zu keinem aktiven Milestone. Beim
nächsten `/gsd-new-milestone` entweder promoten oder bewusst liegen lassen.

- [ ] **Phase 58: Optionale Stundenverteilung pro Werktag** (Feature, Backend + Frontend) — sieben optionale `REAL`-Spalten auf der bereits versionierten `employee_work_details`-Vertragszeile; der bestehende Vertragswechsel-Mechanismus liefert „gültig ab KW" gratis. Betrifft beide Ableitungen der Tagesstunde (`hours_per_day`, `holiday_hours`) und damit Holiday-Derivation und Absence-Zählung. **Zieht Snapshot-Bump 13→14 nach sich.** Enabler für die Vertragszeit-Warnung. Entwurf: `.hermes/plans/2026-08-24_feature-phase-optionale-stundenverteilung-pro-werktag-snapshot-bump-13-14.md`.

- [ ] **Phase 59: E2E-Browser-Test-Infrastruktur (Playwright)** (Maintenance, off-theme) — kein Milestone-Goal, kein REQUIREMENTS-Eintrag; Abnahmemessstab ist der Scope aus dem Origin-Todo. Deckt genau die Invarianten-Klasse ab, die in v2.6 zweimal als `human_needed` stehenblieb (DOM-Rendering), plus die wiederkehrende Dev-Proxy-Bug-Klasse (fehlender `[[web.proxy]]`-Eintrag → 405). Plan liegt fertig: `.planning/phases/59-e2e-browser-testinfra/59-PLAN.md`.

## Backlog

Ungeplante / off-theme Arbeit, die NICHT zum aktiven Milestone gehört. Vor Ausführung
in einen Milestone promoten oder per `/gsd-plan-phase 999.1` direkt planen.

- [ ] **Phase 999.1: Breaking/Major Dependency-Migration** (Backend + Frontend, Maintenance) — Alle direkten Deps mit verfügbaren Major-Releases über beide Cargo-Workspaces (Backend-Root + `shifty-dioxus/`, 9 Member-Crates) auf den neuen Major heben (Cargo.toml-Constraint-Edits + Code-/API-Migration).

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
  **Plans:** 1/1 plans complete

*Last updated: 2026-09-15 — v2.6 geschlossen und archiviert (19/19 Requirements, Audit passed, override_closeout). Kein aktiver Milestone.*
