# Roadmap: Shifty Backend

## Milestones

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

### 🚧 v2.4 — Kurzer-Tag-Slot-Kürzung (Phase 51) — IN PLANNING

**Charakter:** Fokus-Milestone auf einer einzelnen Semantik-Ergänzung. An Kurzen Tagen
(`special_day.ShortDay` mit Cutoff-Uhrzeit) werden Slots, die den Cutoff überlappen,
dynamisch auf `[start, cutoff]` gekürzt (Rendering + Ist-Stunden). Slots komplett hinter
dem Cutoff verschwinden. Soll-Stunden bleiben unverändert. Kein Snapshot-Bump, keine
Migration, keine neue Cargo-Dep.

**Requirements:** SHC-01 (Clip-Funktion), SHC-02 (Reporting/Ist-Stunden),
SHC-03 (WeekView-Rendering), SHC-04 (PDF-Renderer-Konsistenz),
SHC-05 (dynamische Wirkung auf existierende zukünftige Bookings ohne Rewrite).

**Semantik-Anker:** [`notes/shortday-slot-clipping-semantics.md`](notes/shortday-slot-clipping-semantics.md)
(D-01 bis D-06, User-bestätigt im Explore 2026-07-04).

**Consumed Seed:** [`seeds/shortday-slot-clipping.md`](seeds/shortday-slot-clipping.md).

**Open Research:** Q-01 in [`research/questions.md`](research/questions.md) — kanonischer
Ort der Clip-Funktion + Call-Sites, im discuss-phase-Vorfeld oder inline zu klären.

- [ ] Phase 51: Kurzer-Tag-Slot-Kürzung (Vertical MVP: BE-Helper + Reporting-Konsum + FE-WeekView + PDF-Renderer) — SHC-01..05

#### Phase 51: Kurzer-Tag-Slot-Kürzung

**Goal:** An Kurzen Tagen werden Slots, die den Cutoff überlappen, dynamisch gekürzt —
in Schichtplan-Rendering (WeekView + PDF) und Ist-Stunden-Berechnung (Reporting +
Booking-Information). Slots komplett hinter dem Cutoff verschwinden. Soll-Stunden bleiben
unverändert. Kürzung ist view-layer / dynamisch — keine DB-Änderung, kein Snapshot-Bump.

**Depends on:** Nichts Neues. Basiert auf existierendem `special_day.ShortDay`-Modell und
dem v2.3-PDF-Renderer.

**Requirements:** SHC-01, SHC-02, SHC-03, SHC-04, SHC-05
**Plans:** TBD (in `/gsd-plan-phase 51`)

**Vermutliche Wave-Struktur** (final in discuss-phase / plan-phase):

- **Wave 1:** Kanonische Clip-Funktion mit vollständigen Grenzfall-Tests (SHC-01).
  Ort TBD via Q-01 — Kandidaten: `shifty-utils`, Method auf `Slot`, Helper in
  `service_impl`.
- **Wave 2:** Backend-Konsum in Reporting + Booking-Information (SHC-02, SHC-05).
  Live-Balance zeigt gekürzte Ist-Stunden für zukünftige ShortDay-Buchungen.
- **Wave 3:** Frontend-Rendering (WeekView) + PDF-Renderer-Konsum (SHC-03, SHC-04).
  Beide nutzen dieselbe Clip-Semantik.

**Cross-cutting Constraints:**

- Dynamisch / view-layer — keine DB-Änderung
- Snapshot-Schema-Version bleibt 12 (rein additive Live-Berechnung)
- Keine Migration, keine neue Cargo-Dep
- Soll-Stunden unberührt
- Nur zukünftig — kein historischer Rewrite

**Success Criteria:**

1. Backend-Test: Slot 14:00–15:00 auf Datum mit ShortDay-Cutoff 14:30 zählt in
   Reporting-Ist-Stunden exakt 0,5 h; ohne ShortDay 1 h. Slot 15:00–16:00 auf
   demselben Tag zählt 0 h und erscheint nicht in Booking-Information-Aggregaten.
2. Frontend-Test: WeekView-Rendering für einen Tag mit ShortDay zeigt Slot
   14:00–15:00 mit Länge 30 min statt 60 min; Slot 15:00–16:00 wird nicht gerendert.
3. PDF-Konsistenz: PDF-Renderer produziert dieselbe geclippte Wochendarstellung —
   Slot 14:00–15:00 als Zelle 14:00–14:30, Post-Cutoff-Slot fehlt.
4. Alle Backend-Tests grün, `cargo clippy --workspace -- -D warnings` grün,
   `cargo build --target wasm32-unknown-unknown` grün, FE-Clippy `-D warnings` grün.

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

*Last updated: 2026-07-04 — **v2.4 gestartet** (Phase 51, 5 Requirements SHC-01…SHC-05). Kurzer-Tag-Slot-Kürzung: an ShortDays werden Slots dynamisch am Cutoff geclippt — Rendering (WeekView + PDF) und Ist-Stunden (Reporting + Booking-Info). Soll bleibt unverändert. Kein Snapshot-Bump, keine Migration. Seed `shortday-slot-clipping` konsumiert; Semantik-Anker `notes/shortday-slot-clipping-semantics.md`; offene Codebase-Mapping-Frage Q-01 in `research/questions.md`.*
