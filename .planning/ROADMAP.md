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

### ✅ v2.4 — Kurzer-Tag-Slot-Kürzung (Phase 51) — EXECUTED + VERIFIED (2026-07-05, bereit für Milestone-Close)

**Charakter:** Fokus-Milestone auf einer einzelnen Semantik-Ergänzung. An Kurzen Tagen
(`special_day.ShortDay` mit Cutoff-Uhrzeit) werden Slots, die den Cutoff überlappen,
dynamisch auf `[start, cutoff]` gekürzt (Rendering + Ist-Stunden). Slots komplett hinter
dem Cutoff verschwinden. Soll-Stunden bleiben unverändert. Kein Snapshot-Bump, keine
Migration, keine neue Cargo-Dep.

**Requirements:** SHC-01 (Clip-Funktion), SHC-02 (Reporting/Ist-Stunden),
SHC-03 (WeekView-Rendering), SHC-04 (PDF-Renderer-Konsistenz),
SHC-05 (dynamische Wirkung auf existierende zukünftige Bookings ohne Rewrite),
SHC-06 (admin-konfigurierbarer Stichtag `shortday_slot_clipping_active_from`).

**Semantik-Anker:** [`notes/shortday-slot-clipping-semantics.md`](notes/shortday-slot-clipping-semantics.md)
(D-01 bis D-06, User-bestätigt im Explore 2026-07-04).

**Consumed Seed:** [`seeds/shortday-slot-clipping.md`](seeds/shortday-slot-clipping.md).

**Open Research:** Q-01 in [`research/questions.md`](research/questions.md) — kanonischer
Ort der Clip-Funktion + Call-Sites, im discuss-phase-Vorfeld oder inline zu klären.

- [x] Phase 51: Kurzer-Tag-Slot-Kürzung (BE-Helper + Toggle-Stichtag + vier BE-Aggregat-Ketten + DTO/FE-Konsum + Admin-UI) — SHC-01..06 (completed 2026-07-04)

#### Phase 51: Kurzer-Tag-Slot-Kürzung

**Goal:** An Kurzen Tagen werden Slots, die den Cutoff überlappen, dynamisch gekürzt —
in Schichtplan-Rendering (WeekView + PDF) und Ist-Stunden-Berechnung (Reporting +
Booking-Information). Slots komplett hinter dem Cutoff verschwinden. Soll-Stunden bleiben
unverändert. Kürzung ist view-layer / dynamisch — keine DB-Änderung, kein Snapshot-Bump.

**Depends on:** Nichts Neues. Basiert auf existierendem `special_day.ShortDay`-Modell und
dem v2.3-PDF-Renderer.

**Requirements:** SHC-01, SHC-02, SHC-03, SHC-04, SHC-05, SHC-06
**Plans:** 8/8 plans complete

Plans:

- [x] 51-01-PLAN.md — `Slot::clip_to` kanonische Clip-Fn + 4 D-04-Grenzfall-Tests (SHC-01)
- [x] 51-02-PLAN.md — Toggle-Seed `shortday_slot_clipping_active_from` + `shortday_gate`-Helper (SHC-06 BE)
- [x] 51-03-PLAN.md — Chain B: `build_shiftplan_day` clippt via `effective_to`; Bug-Fix + Gate (SHC-02/03/05/06)
- [x] 51-04-PLAN.md — Chain A': BlockService clippt vor Merge (iCal + insufficient) (SHC-02/05/06)
- [x] 51-05-PLAN.md — Chain C: BookingInformation-Aggregate clippen (SHC-02/05/06 + D-51-03 Verify)
- [x] 51-06-PLAN.md — Chain D: ShiftplanReport DAO liefert raw rows, Service-Layer aggregiert+clippt (SHC-02/06, D-51-08)
- [x] 51-07-PLAN.md — DTO `effective_to` am `ShiftplanSlotTO`-Wrapper + FE-Loader + PDF-Renderer-Konsum (SHC-03/04, D-51-09)
- [x] 51-08-PLAN.md — Admin-Settings-UI für Stichtag + i18n de/en/cs (SHC-06 FE)

**Wave-Struktur** (final):

- **Wave 1** (parallel): 51-01, 51-02 — Foundation.
- **Wave 2** (parallel): 51-03, 51-04, 51-05, 51-06 — vier BE-Aggregat-Ketten.
- **Wave 3**: 51-07 — DTO/FE/PDF (hängt an 51-03).
- **Wave 4**: 51-08 — Admin-Settings-UI (hängt an 51-02).

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

*Last updated: 2026-07-04 — **v2.4 geplant** (Phase 51, 6 Requirements SHC-01…SHC-06, 8 Pläne). Kurzer-Tag-Slot-Kürzung: an ShortDays werden Slots dynamisch am Cutoff geclippt — Rendering (WeekView + PDF) und Ist-Stunden (Reporting + Booking-Info + Balance). Soll bleibt unverändert. Admin-konfigurierbarer Stichtag (D-51-07) schützt historische Balance-Views. Kein Snapshot-Bump (bleibt 12), nur additive Toggle-Seed-Migration. Vier BE-Aggregat-Ketten (Chain A' BlockService, Chain B ShiftplanWeek/PDF, Chain C BookingInformation, Chain D ShiftplanReport-Rust-Layer-Refactor). DTO `effective_to` am `ShiftplanSlotTO`-Wrapper (SlotTO bleibt roh). Seed `shortday-slot-clipping` konsumiert; Semantik-Anker `notes/shortday-slot-clipping-semantics.md`; Research Q-01 in `phases/51-.../51-RESEARCH.md` beantwortet.*
