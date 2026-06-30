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
- ✅ **v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation** — Phasen 29–32 (shipped 2026-06-29) — siehe [`milestones/v1.9-ROADMAP.md`](milestones/v1.9-ROADMAP.md)
- 🚧 **v1.10 Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz** — Phasen 33–35 (**aktiv**, gestartet 2026-06-30)

## Phases

**🚧 Aktiver Milestone: v1.10 Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz (Phasen 33–35)**

- [x] **Phase 33: Special-Days-UI in den Einstellungen** - Shiftplanner pflegt Special Days (Holiday/ShortDay) auf zwei Flächen (Schichtplan-Wochenraster + Settings-Sektion) gegen die bestehende REST-CRUD + neuen Range/Jahr-Read — SPD-01..04 ✅ (e2e-verifiziert; create-Pfad-Bug gefixt; visuelle Smokes deferred)
- [x] **Phase 34: Feiertags-Soll im Schichtplan** - Automatisch angerechneter Feiertag reduziert das angezeigte Soll in der Wochentabelle (`get_week` derive-on-read); Kapazitätsbänder unangetastet — HSP-01..04 ✅ 2026-06-30
- [ ] **Phase 35: Slot-Werte nur für eine Woche ändern** - Einmalige Slot-Ausnahme für genau eine KW via Split+Re-Merge (3 Versionen, atomar, Buchungs-Re-Point ohne Doppelzählung) + UI-Wahl „nur diese Woche" vs „ab dieser Woche" — SWO-01..04

<details>
<summary>✅ v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation (Phasen 29–32) — SHIPPED 2026-06-29</summary>

- [x] Phase 29: Urlaubs-Balken-Konsistenz (FE) (1/1 plan) — VAC-01
- [x] Phase 30: Stale-Daten-Race Guard (FE) (1/1 plan) — SHP-02
- [x] Phase 31: Abwesenheit → Nicht-Verfügbar-Markierung im Schichtplan (FE) (1/1 plan) — SHP-01
- [x] Phase 32: Admin-Impersonation Frontend + Audit-Schicht (FE+BE) (3/3 plans) — IMP-01..04

Drei Schichtplan-/Urlaubs-UX-Fixes + vollwertige Admin-Impersonation mit Audit der echten
Admin-Identität. Urlaubsbalken `(used+planned)/total` (Überzug als Farb-Signal); geteilter
`(year,week)`-Staleness-Guard über alle Summary-Loader; proaktive „Nicht Verfügbar"-Markierung
eigener/ausgewählter Absence-Tage (kategorie-treu zur `BookingOnAbsenceDay`-Warnung, null Drift);
Impersonation-FE (nicht-schließbarer Banner, reload-persistent, Users-Tab-Einstieg) + zentrale
Audit-Middleware (`RealUser`) + Store-Teardown — ohne `Authentication<Context>`-Signatur-Change,
ohne Snapshot-Bump, ohne Migration. Audit `passed` (7/7 Requirements, 4/4 Integration + E2E).

Vollständige Phasen-Details, Decisions und Closeout:
[`milestones/v1.9-ROADMAP.md`](milestones/v1.9-ROADMAP.md) · [`milestones/v1.9-REQUIREMENTS.md`](milestones/v1.9-REQUIREMENTS.md) · [`milestones/v1.9-MILESTONE-AUDIT.md`](milestones/v1.9-MILESTONE-AUDIT.md)

</details>

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

## Phase Details (v1.10 — aktiv)

### Phase 33: Special-Days-UI in den Einstellungen

**Goal**: Ein Shiftplanner kann Special Days (Feiertage / Kurztage) auf **zwei Flächen** voll-CRUD pflegen — interaktiv im Schichtplan-Wochenraster (Per-Tag-Dropdown) **und** über eine Sektion in den Einstellungen (Kalenderdatum-Picker + Jahres-Liste) — verdrahtet gegen die bestehende REST-CRUD (`POST/DELETE /special-days`, `for-week`-Read) plus einen **neuen Range/Jahr-Read-Endpoint**.
**Depends on**: Nothing (Backend-CRUD existiert seit v1.7; fachlich unabhängig von Phase 34 — kann sequenziell zuerst laufen, weil es die Einträge erzeugt, die Phase 34 sichtbar macht)
**Requirements**: SPD-01, SPD-02, SPD-03, SPD-04
**Success Criteria** (what must be TRUE):

  1. Als Shiftplanner kann ich einen Special Day anlegen — auf der Settings-Seite per Kalenderdatum **und** im Schichtplan-Wochenraster per Per-Tag-Dropdown (Typ `Holiday` oder `ShortDay`; bei `ShortDay` mit Pflicht-Uhrzeit; Duplikat am selben Tag wird geblockt); nach dem Speichern erscheint er in Liste/Raster. (SPD-01)
  2. Die Settings-Liste zeigt jedes Datum im locale-üblichen Format plus abgeleitetem Kontext in Klammern, z. B. `15.08.2026 (Samstag, KW 33, 2026)` — chronologisch nach Jahr gruppiert (Jahr-Picker), gespeist aus dem neuen Range/Jahr-Read-Endpoint. (SPD-02)
  3. Als Shiftplanner kann ich einen vorhandenen Special Day löschen (Settings-Liste oder Wochenraster-Dropdown „Nichts"); die Ansicht aktualisiert sich sofort. (SPD-03)
  4. Die Special-Days-Pflege ist **shiftplanner-gated** (deckungsgleich zur bestehenden Special-Day-CRUD und Slot-Struktur-CRUD; FE-Gate `has_privilege("shiftplanner")`, kein 403-Mismatch). (SPD-04)
  5. Alle neuen benutzersichtbaren Texte sind in de/en/cs vorhanden. (SPD-04)

**Plans**: 4/4 plans complete
**Wave 1**

- [x] 33-01-PLAN.md — Backend Range/Jahr-Read-Endpoint (`GET /special-days/for-year/{year}`) + Service-Test-Modul (TDD; D-33-05/01) [Wave 1]
- [x] 33-02-PLAN.md — FE-Foundation: api.rs (create/delete/for-year) + 18 i18n-Keys de/en/cs (SPD-04) [Wave 1]

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 33-03-PLAN.md — Settings Card-3 (shiftplanner-gated): Datepicker-Create + Jahres-Liste + Delete (D-33-02/04/06/07/08) [Wave 2]
- [x] 33-04-PLAN.md — Schichtplan Per-Tag-Dropdown (Feiertag/Kurzer Tag/Nichts) + ShortDay-Inline-Prompt (D-33-01/03/06/07) [Wave 2]

**UI hint**: yes

**Phase-Note (Gate-Korrektur & Endpoint):** SPD-04 sprach ursprünglich von „admin-gated (Muster Phase 24/25)". In der discuss-phase code-verifiziert korrigiert auf **`shiftplanner`** (Special Days = Schichtplan-Struktur; Slot-CRUD + Special-Day-CRUD gaten bereits auf `shiftplanner`). Der zuvor als „deferred" geführte **Multi-Wochen-Read-Endpoint** wird per D-33-05 bewusst **in diese Phase** gehoben (Settings-Übersicht). Details: `phases/33-special-days-ui-einstellungen/33-CONTEXT.md`.

### Phase 34: Feiertags-Soll im Schichtplan

**Goal**: Ein automatisch angerechneter Feiertag reduziert das angezeigte Soll in der Wochentabelle unter dem Schichtplan — konsistent zum Stundenkonto —, während die Kapazitätsbänder (`paid_hours`/`dynamic_hours`/`committed_voluntary`/`volunteer`) unverändert bleiben (D-25-08-Grenze).
**Depends on**: Nothing (fachlich unabhängig von Phase 33; wiederverwendet den bestehenden `build_derived_holiday_map`-Pfad aus Phase 25)
**Requirements**: HSP-01, HSP-02, HSP-03, HSP-04
**Success Criteria** (what must be TRUE):

  1. In der Wochentabelle unter dem Schichtplan reduziert ein automatisch angerechneter Feiertag das angezeigte Soll (`available_hours`/`expected_hours`) eines Mitarbeiters; der Wert stimmt mit dem Stundenkonto überein. (HSP-01)
  2. Die abgeleiteten Feiertags-Stunden (`holiday_hours`) erscheinen pro Mitarbeiter in der Schichtplan-Tabelle. (HSP-02)
  3. Die Kapazitätsbänder (`paid_hours`/`dynamic_hours`/`committed_voluntary`/`volunteer`) sind in derselben Woche vor und nach der Änderung identisch (Regressions-Guard). (HSP-03)
  4. Ein Feiertag vor dem konfigurierten Stichtag bleibt wirkungslos, und ein manueller `ExtraHours(Holiday)` wird nicht doppelt gezählt — identisch zum Stundenkonto (Wiederverwendung von `build_derived_holiday_map`). (HSP-04)

**Plans**: 1 plan

**Wave 1**

- [x] 34-01-PLAN.md — `get_week` 4. Injektionspunkt (`holiday_derived_gated` reduziert nur `expected_hours`/`holiday_hours`, Bänder unangetastet) + HOL-03-Test-Rebuild + 2 HSP-04-Subtests + Snapshot-Verifikation (TDD; HSP-01..04, D-34-01..04) [Wave 1] ✅ 2026-06-30

**Phase-Note (Snapshot & HOL-03):** Snapshot-Schema-Version voraussichtlich **kein Bump** — `billing_period`-Snapshots speisen sich aus dem `reporting.rs`-`holiday_hours`-Pfad, nicht aus `get_week`/`booking_information`. In der Phase verifizieren (Default: kein Bump). Der HOL-03-Regressionstest `test_holiday_auto_credit_no_year_view_impact` wird bewusst neu formuliert: Kapazitätsbänder bleiben unverändert, aber `expected_hours`/`available_hours` werden um den derived-Holiday reduziert. *(Offene Decision für discuss-phase, D-NN.)*

### Phase 35: Slot-Werte nur für eine Woche ändern

**Goal**: Ein Shiftplanner kann die Werte eines Slots (Kapazität/Zeiten) für **genau eine Kalenderwoche** als einmalige Ausnahme ändern, ohne die wiederkehrende Struktur ab dieser KW dauerhaft zu verändern — **atomar** (alles in einer Transaktion, Rollback bei Fehler) und **ohne Doppelzählung** in Reports/Balance.
**Depends on**: Nothing (baut auf der bestehenden `ShiftplanEditService::modify_slot`-Mechanik auf; fachlich unabhängig von Phase 33/34)
**Requirements**: SWO-01, SWO-02, SWO-03, SWO-04
**Success Criteria** (what must be TRUE):

  1. Im Slot-Editor kann ein Shiftplanner explizit zwischen **„nur diese Woche"** und **„ab dieser Woche"** wählen; „nur diese Woche" wirkt ausschließlich in der gewählten KW. (SWO-01)
  2. Mechanik = **Split+Re-Merge**: 3 Slot-Versionen (alt bis KW-1 / nur diese KW mit neuen Werten / Original-Werte ab KW+1); Buchungen der KW → Segment 2, Buchungen ab KW+1 → Segment 3. (SWO-02)
  3. Der gesamte Vorgang (alle Segment-Schnitte + alle Booking-Re-Points) läuft in **einer Transaktion**; bei jedem Fehler ist der Zustand exakt wie vorher. (SWO-03)
  4. Die Booking-Neuzuweisungen sind durch harte Tests abgesichert — **nichts** doppelt oder verwaist in Reports/Balance. Gate = `shiftplan.edit` (konsistent zu `modify_slot`). (SWO-04)

**Plans**: 2/3 plans executed

- [x] 35-01-PLAN.md — Backend-Mechanik `modify_slot_single_week` (TDD: 3-Segment-Split + Booking-Partition + Atomarität + Gate) + REST-Route (SWO-02/03/04)
- [x] 35-02-PLAN.md — Frontend-Plumbing: single_week-State, api/loader single-week-Pfad, SetSingleWeek-Routing, 4 i18n-Keys de/en/cs (SWO-01/04)
- [ ] 35-03-PLAN.md — Frontend-Komponente: Modus-Radiogruppe + Hinweis im Slot-Editor + SSR-Tests (SWO-01)

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
| 24 — Paid-Limit konfigurierbar & rollenbasiert (BE+FE) | v1.6 | 5/5 | Complete | 2026-06-27 |
| 25 — Feiertags-Auto-Anrechnung & Stichtag-Konfiguration (BE+FE) | v1.7 | 4/4 | Complete | 2026-06-28 |
| 26 — Freiwilligen-Abwesenheit & Cross-Navigation (BE+FE) | v1.7 | 3/3 | Complete | 2026-06-28 |
| 27 — Freiwillige in Abwesenheitsliste auswählbar (FE) | v1.8 | 1/1 | Complete | 2026-06-29 |
| 28 — Urlaubsanspruch-Korrektur via Offset (BE+FE) | v1.8 | 4/4 | Complete | 2026-06-29 |
| 29 — Urlaubs-Balken-Konsistenz (FE) | v1.9 | 1/1 | Complete    | 2026-06-29 |
| 30 — Stale-Daten-Race Guard (FE) | v1.9 | 1/1 | Complete    | 2026-06-29 |
| 31 — Abwesenheit → Nicht-Verfügbar-Markierung im Schichtplan (FE) | v1.9 | 1/1 | Complete    | 2026-06-29 |
| 32 — Admin-Impersonation Frontend + Audit-Schicht (FE+BE) | v1.9 | 3/3 | Complete    | 2026-06-29 |
| 33 — Special-Days-UI in den Einstellungen (FE) | v1.10 | 4/4 | Complete   | 2026-06-30 |
| 34 — Feiertags-Soll im Schichtplan (BE) | v1.10 | 1/1 | Complete    | 2026-06-30 |
| 35 — Slot-Werte nur für eine Woche ändern (BE+FE) | v1.10 | 2/3 | In Progress|  |

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
  **Plans:** 1/1 plans complete

*Last updated: 2026-06-30 — **Phase 35 hinzugefügt** (Slot-Werte nur für eine Woche ändern, SWO-01..04; Mechanik B Split+Re-Merge, atomar, diskutiert). v1.10 jetzt Phasen 33–35, 12/12 Requirements gemappt. Phase 35 ist bewusst Schichtplan-Struktur (leicht off-theme zum Feiertags-Fokus, User-Entscheidung in v1.10 statt Backlog).*
