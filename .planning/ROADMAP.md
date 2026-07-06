# Roadmap: Shifty Backend

## Milestones

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

## Next Milestone

### v2.5 — Weekly-Overview Performance & Freiwilligen-Abwesenheiten

**Goal:** Die Jahresübersicht (`/weekly_overview/`) reagiert sub-Sekunde und
listet die Abwesenheiten der Freiwilligen sichtbar mit auf.

**Requirements:** `.planning/REQUIREMENTS.md` (9 Requirements: WOP-01..05, VAA-01..04).

**Phases:**

- [x] **Phase 52: Weekly-Overview Performance-Refactor** — Requirements WOP-01, WOP-02, WOP-03, WOP-04, WOP-05 — **verified 2026-07-06** (5 Pläne + 2 Follow-ups; Speedup **19.4×** 2.33s→0.12s, WOP-04 <0.5s um 4× übertroffen; Byte-Identity 8/8 Wave-1-Fixtures grün; 1 dokumentierter Override zu SC#4 — `special_day.get_by_week` ~55×/(year,week) statt „1 pro Endpoint", weil `get_by_year`-Kalenderjahr-Semantik nicht byte-identisch zu ISO-Wochen-gebundenem `get_by_week` ist; die eigentliche N_persons×N_weeks-Multiplikation ist eliminiert; VERIFICATION.md `passed`)

  **Goal:** `BookingInformationServiceImpl::get_weekly_summary` konsumiert
  Jahres-Aggregate: `special_days` und `shiftplan_reports` einmalig
  vorgeladen (analog zum existierenden `all_work_details`/`all_absences`-
  Muster), plus eine neue `reporting_service.get_year`-Aggregation, die die
  ~55 sequenziellen `get_week`-Calls ersetzt. Ergebnis byte-identisch zur
  alten Wochen-Iteration (Property-Test als hartes Gate). End-to-End-Latenz
  von `GET /booking-information/weekly-resource-report/{year}` <500ms auf
  Dev-DB. Alle bestehenden Tests grün (insb.
  `booking_information_chain_c.rs`, Reporting-Tests). Snapshot bleibt 12,
  keine Migration, keine neue Cargo-Dep. `reporting_service.get_week`
  bleibt für andere Call-Sites bestehen.

  **Success criteria:**
  1. Jahresansicht in Dev-DB antwortet <500ms (heute mehrere Sekunden), gemessen im PLAN. ✅ (0.12s Median, 4× unter Ziel)
  2. Property-Test grün: neue vs. alte Implementierung bit-exakt identisch über generierte Szenarien (Feiertage, ShortDays, Freiwilligen-Absencen, CVC-06-Cap, `shortday_gate.active_from` on/off). ✅ (8/8 Wave-1-Fixtures)
  3. `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` grün. ✅
  4. `special_days`- und `shiftplan_reports`-Calls sind pro Endpoint-Abruf 1 (statt ~55). ⚠️ Override: `get_weekly_summary` erreicht 2× (year + year+1 Spillover); `assemble_weeks` behält bewusst ~55× `get_by_week` per (year,week) statt `get_by_year` — Semantik-Divergenz ISO vs Kalenderjahr. Der eigentliche Skalierungs-Bug (N_persons-Multiplikation, ~26 000 Queries) ist eliminiert.
  5. Kein Snapshot-Schema-Bump; `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12. ✅

  **Plans:** 5 plans (planned 2026-07-05; 5 Waves strikt sequenziell — kein Wave läuft parallel, weil `reporting.rs` und `booking_information.rs` mehrfach angefasst werden)
  - [x] 52-01-PLAN.md — **Wave 1**: Fixture-Golden-Snapshots + Latenz-Baseline (WOP-03/05) — 8 Fixtures + 2.33s Median
  - [x] 52-02-PLAN.md — **Wave 2**: `assemble_weeks`-Helper aus `get_week` extrahiert (WOP-02/05) — reiner Refactor, byte-identisch
  - [x] 52-03-PLAN.md — **Wave 3**: `extract_shiftplan_report_for_year` + `find_by_year` Trait+DAO+`sqlx prepare` (WOP-01/05)
  - [x] 52-04-PLAN.md — **Wave 4**: `ReportingService::get_year` Trait+Impl (WOP-02/05) — delegiert auf `assemble_weeks`
  - [x] 52-05-PLAN.md — **Wave 5**: `get_weekly_summary`-Umbau (7 Bulk-Loads + In-Memory-Loop) — Median 1.13s (2.07×)
  - [x] **Follow-up #1**: `sales_person` load-once + `working_hours` HashMap-Index in `assemble_weeks` (WOP-04) — Median 0.97s (2.40× kumulativ)
  - [x] **Follow-up #2**: `build_derived_holiday_map` + `derive_hours_for_range` Year-Batch (WOP-04) — Median **0.12s** (19.4× kumulativ) ✅

  **Follow-Ups für Milestone-Close-Audit (nicht blockierend):** `SpecialDayService::get_by_iso_year` (ISO-Jahr statt Kalender-Jahr) für saubere SC#4-Semantik; DB-Indices aus RESEARCH.md Q3 (`booking(year,cw)`, `extra_hours(date_time)`, `working_hours(from_year,to_year)`); F07-Doku für neue Pure-Helper.

  **Cross-cutting constraints (in ≥2 Plänen):**
  - D-52-04 (Spillover via 2× `get_year(year)` + `get_year(year+1)` — Plans 04, 05)
  - D-52-06 (Neue Trait-Methoden: `get_year` + `extract_shiftplan_report_for_year` — Plans 03, 04)
  - D-52-08 (`assemble_weeks`-Helper — Plans 02, 04; RESEARCH Q2 überschreibt CONTEXT-Vereinfachung: `async fn` mit `tx`, NICHT sync)
  - D-52-09 (MUST-preserve: Balance-Formel, CVC-06-Cap, Chain-C-Toggle-Read bleibt in `booking_information`, NICHT im Helper — Plans 02, 04, 05)
  - D-52-10 (`get_week` bleibt public trait method, Signatur unverändert — Plans 02, 04)

- [ ] **Phase 53: Freiwilligen-Abwesenheiten in Jahresansicht** — Requirements VAA-01, VAA-02, VAA-03, VAA-04

  **Goal:** In `sales_person_absences` der Jahresansicht erscheinen zusätzlich
  zu bezahlten Mitarbeitern auch Freiwillige mit aktiver
  Vacation/SickLeave/UnpaidLeave-Period in der jeweiligen Kalenderwoche.
  Backend liefert Name + Stunden-Wert (Kandidat: `committed_voluntary` der
  Person; exakte Semantik in discuss-phase fixiert) fertig im DTO. Frontend
  rendert mit bestehender Zeile in `page/weekly_overview.rs`. Fat Backend,
  Thin Client — kein Merge im FE. Baut auf der neuen Assembly aus Phase 52
  auf.

  **Success criteria:**
  1. Freiwilliger mit `Vacation`-Period, die Kalenderwoche N überlappt → erscheint in `sales_person_absences` von Woche N (Backend-Test verifiziert).
  2. Freiwilliger ohne aktive Period → nicht in der Liste.
  3. Bezahlter Mitarbeiter bleibt unverändert (keine Regression).
  4. Angezeigter Stunden-Wert ist in discuss-phase entschieden und dokumentiert.
  5. Frontend rendert Freiwilligen-Zeilen visuell konsistent mit bezahlten (kein Redesign).

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

---

*Last updated: 2026-07-06 — **Phase 52 verified 2026-07-06** (5 Pläne + 2 Follow-ups; kumulativer Speedup 19.4× 2.33s→0.12s, WOP-04 <0.5s um 4× übertroffen; Byte-Identity 8/8 Wave-1-Fixtures grün; 1 dokumentierter Override zu SC#4 — special_day.get_by_week ~55×/(year,week) statt "1 pro Endpoint" wegen ISO-vs-Kalenderjahr-Semantik; die eigentliche N_persons×N_weeks-Multiplikation ist eliminiert). Milestone v2.5 zu 50 % (1/2 Phasen, 5/5 Pläne). Phase 53 (Freiwilligen-Abwesenheiten in Jahresansicht) als nächstes offen — baut auf `assemble_weeks` + `get_year` aus Phase 52 auf. Zuvor: 2026-07-05 v2.5 gestartet.*
