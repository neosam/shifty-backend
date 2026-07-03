# Roadmap: Shifty Backend

## Milestones

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

### 🚧 v2.3 — PDF-Export: Browser-Look & Download-Button (Phasen 49–50) — IN ARBEIT

**Charakter:** Kleiner Fix-Milestone auf dem v2.2-PDF-Export. Der v2.2-Renderer produziert
praktisch unlesbare PDFs; v2.3 macht daraus eine browser-ähnliche Wochenansicht und legt
einen On-Demand-Download-Button auf die Schichtplan-Seite. Kein Snapshot-Bump (bleibt 12),
keine Migration, keine neue Cargo-Dep. WebDAV-Scheduler aus Phase 48 nutzt den neuen
Renderer nach Phase 50 automatisch.

- [x] Phase 49: On-Demand-Download-Button (BE + FE) — PDF-03, PDF-04, PDF-05 (completed 2026-07-03)
- [ ] Phase 50: PDF-Renderer neu — Browser-Look + Timestamp — PDF-01, PDF-02

**Reihenfolge-Rationale:** Erst der Button (mit dem alten, unlesbaren Renderer), dann der
Renderer-Rewrite. So kann das Rendering in Phase 50 direkt per Button-Klick verifiziert
werden.

#### Phase 49: On-Demand-Download-Button (BE + FE)

**Goal:** Auf der Schichtplan-Seite gibt es einen PDF-Download-Button, der die aktuell
im UI selektierte Kalenderwoche des ausgewählten Shiftplans für jeden authentifizierten
User ausliefert — aber nur sichtbar, wenn `week_status ∈ {Planned, Locked}`.

**Depends on:** Nichts Neues (nutzt den v2.2-Renderer 1:1).
**Requirements:** PDF-03, PDF-04, PDF-05
**Plans:** 5/5 plans complete

Plans:
**Wave 1**

- [x] 49-01-PLAN.md — Backend Service Trait + Impl + 8+ Unit-Tests (PDF-03, PDF-04) [TDD, Wave 1]

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 49-02-PLAN.md — REST Handler + PdfShiftplanApiDoc + Router-Wiring + Router-Tests (PDF-03, PDF-04, PDF-05) [TDD, Wave 2]
- [x] 49-03-PLAN.md — Scheduler-Refactor (DRY) + DI-Wiring in main.rs (PDF-03, PDF-04) [Wave 2]

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 49-04-PLAN.md — FE PDF-Anchor neben iCal + i18n `PdfDownload` de/en/cs + 8-case Predicate-Tests (PDF-03, PDF-04) [Wave 3]
- [x] 49-05-PLAN.md — REQUIREMENTS.md + ROADMAP.md D-49-15/16 Doku-Deviation (PDF-03, PDF-04) [Wave 3]

**Success Criteria:**

1. Neuer REST-Endpoint `GET /shiftplan/{shiftplan_id}/{year}/{week}/pdf` liefert das PDF
   mit Dateiname `schichtplan-{JJJJ}-KW{NN}.pdf` (Content-Disposition-Header);
   auth-required, aber KEIN Admin-Gate (Employee-Auth liefert 200).

2. Backend gibt HTTP 409 zurück, wenn `week_status ∈ {Unset, InPlanning}`
   (Defense-in-Depth für Race-Case).

3. Frontend-Button in `shifty-dioxus/src/page/shiftplan.rs` sitzt neben dem
   iCal-Button und wird nur sichtbar gerendert, wenn `week_status ∈ {Planned, Locked}`
   (kein disabled-Zustand, kein Tooltip, kein Fehler-Toast). Button lädt die
   aktuell im UI selektierte KW des ausgewählten Shiftplans. i18n-Label in de/en/cs.

4. DRY: neuer Business-Logic-Service `PdfShiftplanService` kapselt
   `ShiftplanViewService` + `SalesPersonService` + `WeekStatusService` + `pdf_render`.
   `PdfExportScheduler` wird refactored, damit er diesen Service konsumiert statt
   inline zu orchestrieren (einheitlicher Assemble-Pfad für On-Demand + Cron).

#### Phase 50: PDF-Renderer neu — Browser-Look + Timestamp

**Goal:** `service_impl/src/pdf_render.rs` produziert PDFs, die visuell der
Browser-Wochenansicht (`shifty-dioxus/src/page/shiftplan.rs`) entsprechen — Slots als
Zellen mit Uhrzeiten, Bookings mit Sales-Person-Namen, Wochentage als Spalten — und
tragen den Renderzeitpunkt sichtbar auf jeder Seite.

**Depends on:** Nichts (Phase 49 unabhängig; Phase 50 verifiziert per Klick auf den
Phase-49-Button gegen ein reales Wochen-Fixture).
**Requirements:** PDF-01, PDF-02
**Plans:** 3/3 plans complete (executed 2026-07-03; verifier PASSED 14/14, human UAT D-50-17 pending)

Plans:
**Wave 1**

- [x] 50-01-PLAN.md — TDD-Wave-0-Vorbereitung: `local-offset`-Cargo-Feature, `FIXED_RENDER_TIMESTAMP`-Konstante, `make_sales_person`-Fixture-Extension (`is_paid`-Param), 6 neue D-50-16-Test-Skelette (`#[ignore]`), 3 obsolete Tests entfernt (D-50-13/D-50-15) [TDD, Wave 1]

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 50-02-PLAN.md — Renderer-Rewrite: neue 5-Parameter-Signatur mit `render_timestamp` (D-50-11), Hybrid-Stack-Layout (D-50-01/02), Slot-Boxen mit `add_rect`+`PaintMode::Stroke` (D-50-10), dynamische Sonntag-Spalte (D-50-08), Timestamp im Header (D-50-09), alphabetische Namen mit `(freiwillig)`-Suffix (D-50-06/07), `+ N weitere`-Overflow (D-50-03/04), pdf_shiftplan.rs Übergangs-Bridge [TDD, Wave 2]

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 50-03-PLAN.md — Aufrufer-Finalisierung: `resolve_render_timestamp()`-Fn mit `now_local()` + UTC-Fallback + `warn!`-Log (D-50-12), Service-Test `now_local_fallback_to_utc_on_indeterminate_offset` (D-50-16) [TDD, Wave 3]

**Cross-cutting constraints (aus PLAN.md must_haves.truths, in ≥2 Plänen):**
- D-50-11: Renderer nimmt `render_timestamp: OffsetDateTime` als 5. Parameter (definiert in 50-02, konsumiert in 50-03)
- D-50-12: `now_local()` mit UTC-Fallback + `warn!`-Log (Übergangs-Bridge in 50-02, echte Impl in 50-03)
- D-50-14: `FIXED_RENDER_TIMESTAMP`-Test-Konstante (definiert in 50-01, konsumiert in 50-02 Test-Aktivierung)
- D-50-16: 7 neue Tests (Skelette in 50-01, aktiviert in 50-02, Service-Test in 50-03)
- Clippy-Gate `cargo clippy --workspace -- -D warnings` grün nach jedem Wave

**Success Criteria:**

1. Rendering entspricht sichtbar der Browser-Wochenansicht: Slots als Zellen mit
   Uhrzeit-Label pro Zelle, Sales-Person-Namen in der Zelle, sieben Wochentag-Spalten,
   Landscape A4, „Schichtplan KW {NN} ({JJJJ})"-Kopfzeile.

2. Renderzeitpunkt „Erstellt am DD.MM.YYYY HH:MM Uhr" auf jeder Seite sichtbar; Renderer
   nimmt Timestamp als Argument (pure Funktion bleibt testbar).

3. Byte-Determinismus-Vertrag aus v2.2 wird bewusst aufgehoben (Timestamp bricht ihn
   ohnehin). WebDAV-Scheduler nutzt den neuen Renderer transparent — kein Scheduler-Code-
   Change nötig. Alle Backend-Tests + `cargo clippy --workspace -- -D warnings` grün.

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

*Last updated: 2026-07-03 — **v2.3 gestartet** (Phasen 49–50, 5 Requirements PDF-01…PDF-05). Kleiner Fix-Milestone auf dem v2.2-PDF-Export: erst Download-Button (Phase 49, unlesbarer alter Renderer), dann Renderer-Rewrite (Phase 50, Browser-Look + Timestamp). Kein Snapshot-Bump, keine Migration, keine neue Dep.*
