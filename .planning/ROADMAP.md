# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (shipped 2026-05-07) — siehe [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)
- ✅ **v1.3 Frontend Abwesenheiten + UI-Closure-Restanten** — Phasen 8–13 (closed 2026-06-22; geliefert: 8, 8.2, 8.4, 8.5, 8.6, 9; 8.1/11 ⊘ superseded; **8.3/10/12/13 bewusst aufgegeben**) — siehe [`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md)
- ◆ **v1.4 Committed Voluntary Capacity** — Phasen 14–17 (active, started 2026-06-22)

## Phases

### v1.4 Committed Voluntary Capacity (active)

Zeit-versioniertes `committed_voluntary: f32` auf `EmployeeWorkDetails` (D-01 / Variante B: nur die freiwillige Zusage obendrauf, entkoppelt von `expected_hours`), in der Jahresansicht **ohne Doppelzählung** als separat ausgewiesene Kapazität. Build-Order strikt compile-dependency-geordnet (Backend-Foundation vor konsumierendem Frontend). Reporting-Formel ist Achse-B-only (Jahresansicht) — KEIN Snapshot-Bump (D-01 revidiert 2026-06-23). **Achse B** (`booking_information.rs::get_weekly_summary`) ist der Jahresansicht-Pfad, NICHT `reporting.rs` (Achse A).

- [x] **Phase 14: Data-model foundation (backend)** (Backend) — zeit-versioniertes `committed_voluntary: f32` durch alle Layer, Feld inert (nirgends gelesen)
  Migration (`REAL NOT NULL DEFAULT 0`) + `.sqlx`-Regen → DAO (Entity + 4 SELECT + INSERT + UPDATE + `TryFrom`) → Service (Struct + 2 Konversionen) → `EmployeeWorkDetailsTO` + `#[serde(default)]`. Kopiert die `cap_planned_hours_to_expected`-Präzedenz Zeile-für-Zeile. KEIN REST/OpenAPI-Change (Endpoint-Familie hat bewusst keine utoipa). Überlapp-Aggregation explizit definiert + per Test gepinnt.
  Requirements: CVC-01, CVC-02, CVC-03
  Success Criteria:
  1. Migration `… ADD COLUMN committed_voluntary REAL NOT NULL DEFAULT 0` läuft additiv; `.sqlx`-Offline-Cache regeneriert; `cargo check --workspace` + `cargo test --workspace` grün; Bestandsdaten driftfrei (Default 0, kein Reporting-Effekt da Feld nirgends gelesen)
  2. `committed_voluntary` ist auf DAO-Entity, Service-Struct und `EmployeeWorkDetailsTO` (mit `#[serde(default)]`) präsent; beide Konversionsrichtungen an jeder Boundary durchgezogen; ein erweiterter `employee_work_details_update`-Integrationstest verifiziert einen fraktionalen Open→Save→Reload-Round-Trip
  3. Beim Rotieren einer Vertrags-Version (neue Zeit-Periode) wird `committed_voluntary` korrekt mitgeführt und nicht still auf Default zurückgesetzt (Test gepinnt)
  4. Aggregation bei zwei überlappenden aktiven Vertrags-Rows in derselben ISO-Woche ist explizit definiert (sum/max/first via `find_working_hours_for_calendar_week`-Selektion) und durch einen Test gepinnt (D-OVERLAP-AGG — Boolean-`.any()`-Pattern des Cap-Flags generalisiert nicht auf einen numerischen Wert)

- [x] **Phase 15: Reporting no-double-count (Achse B only, KEIN Snapshot-Bump)** (Backend) — separater `committed_voluntary_hours`-Term (Band 1) + reduzierter `volunteer_hours`-Term (Band 2), per-Person-Surplus via FORMULA B
  Höchst-Risiko-Phase (D-FORMULA-PATH, D-SCOPE-GATE). Integration landet in **`booking_information.rs::get_weekly_summary` (Achse B)**, NICHT `reporting.rs` (Achse A) — sonst Doppelzählung. Zwei-Band-Dekomposition (D-05): Band 1 = cap-gated Σ_Person committed, Band 2 = Σ_Person max(actual_p − committed_p, 0) (FORMULA B). KEIN `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump — `committed_voluntary_hours` (Band 1) + reduzierter `volunteer_hours` (Band 2) sind Achse-B-only (Jahresansicht), berühren keinen persistierten `BillingPeriodValueType`; Version bleibt 7 (D-01).
  Requirements: CVC-04, CVC-05, CVC-06
  Success Criteria:
  1. `get_weekly_summary` exponiert einen **neuen separaten** `committed_voluntary_hours`-Term (nicht in `paid`/`volunteer` gefaltet), per ISO-Woche via `counted_volunteer = max(committed_voluntary, actual_volunteer)` summiert; ein Worked-Example-Fixture-Test (z.B. committed=5, actual=7 → 7; committed=5, actual=3 → 5) pinnt die per-Woche-vor-Summe-Reihenfolge, niemals `max(Σ, Σ)`
  2. Jeder Read von `committed_voluntary` ist auf `cap_planned_hours_to_expected = true` gegated; für `cap = false` trägt die Zusage `0.0` zu allen Berechnungen bei (Backward-Compat: `committed = 0` ⇒ Ergebnis identisch zu heute, durch Regressionstest verifiziert)
  3. `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **7** (KEIN Bump) — begründet im Audit-Trail (Phase 15 berührt keinen persistierten `value_type`; `WeeklySummary` wird von `billing_period_report.rs` nie gelesen). Ein Regressionstest `snapshot_schema_version_unchanged_at_7` bestätigt die unveränderte Version (CVC-05).
  4. `cargo test --workspace` grün; Billing-Period-Snapshot-Regression sauber; keine Doppelzählung gegen Achse B (durch den Fixture-Test in Kriterium 1 strukturell ausgeschlossen)

- [ ] **Phase 16: Jahresansicht display** (Backend + Frontend) — separater „zugesagt"-Token inkl. Überschuss; i18n de/en/cs
  `WeeklySummary` + `WeeklySummaryTO` + `From` tragen den `committed_voluntary_hours`-Term; Frontend `state/weekly_overview.rs` + `page/weekly_overview.rs` rendert einen **dritten** Token („zugesagt"), getrennt von `paid`/`volunteer`. Überschuss sichtbar (committed=5, actual=7 → `5 + 2`; committed=5, actual=3 → gedeckte `5`). committed=0 wird als `🎯0.00` gezeigt (keine blank/Strich-Sonderlogik, D-03).
  Requirements: CVC-07, CVC-08
  Success Criteria:
  1. `weekly_overview` zeigt die committed-Kapazität als eigenen Token („zugesagt") — nicht mit `paid`/`volunteer` vermischt; der `From<&WeeklySummaryTO>`-Mapping-Pfad trägt das neue Feld (keine Omission-Lücke)
  2. Überschuss wird sichtbar ausgewiesen (committed=5, actual=7 → `5 + 2`; committed=5, actual=3 → gedeckte `5`); committed=0 wird als `🎯0.00` gezeigt — keine blank/Strich-Sonderlogik (D-03; die blank/Strich-Idee gehört, falls überhaupt, in die Mitarbeiteransicht/Phase 17)
  3. Alle neuen benutzersichtbaren Strings sind in De / En / Cs vollständig gepflegt; Per-Locale-Reference-Matcher-Tests (analog v1.3) schließen den `Locale::En`-statt-`Locale::De`-Bug aus
  4. `cargo build --target wasm32-unknown-unknown` grün; `cargo test --workspace` grün
  **UI hint**: yes

- [ ] **Phase 17: Contract editor input + „alle"-Filter / unpaid-volunteer path** (Backend + Frontend) — numerisches Eingabefeld, „alle"-Toggle, unbezahlte Freiwillige
  Most design-open phase (D-UNPAID-RECORD). Numerisches `committed_voluntary`-Feld in `contract_modal.rs` neben dem Cap-Toggle (Open→Save-unverändert-Round-Trip bewahrt den Wert). Einblendbarer „alle"-Filter in der Mitarbeiteransicht; rein unbezahlte Freiwillige bekommen einen `EmployeeWorkDetails`-Record (`is_paid = false`, `expected_hours = 0`) und werden sichtbar/auswählbar. Jede enumerierte paid-only-Site ist explizit auf `sales_person.is_paid` gegated (nicht auf Record-Präsenz) — kein Leak in `paid_hours`/Billing/Year-Summary.
  Requirements: CVC-09, CVC-10
  Success Criteria:
  1. `committed_voluntary` ist im Vertrags-Editor (`contract_modal.rs`, neben dem Cap-Toggle) als numerisches Feld editierbar; ein Open→Save-unverändert-Round-Trip bewahrt den Backend-Wert (beide `TryFrom`-Richtungen in `state/employee_work_details.rs`)
  2. Mitarbeiteransicht bekommt einen einblendbaren „alle"-Filter; rein unbezahlte Freiwillige können einen `EmployeeWorkDetails`-Record halten (`is_paid = false`, `expected_hours = 0`) und werden sichtbar/auswählbar
  3. Jede work-details-iterierende paid-only-Site ist explizit auf `sales_person.is_paid` gegated (nicht auf Record-Präsenz); ein Integrationstest sichert den `get_week`-Seiteneffekt ab und verifiziert: kein Leak in `paid_hours` / Billing / Year-Summary, Personen-Set-Konsistenz über year-summary / all-employees-report / Billing
  4. i18n De / En / Cs für neue Labels/Filter vollständig (Per-Locale-Reference-Matcher); `cargo build --target wasm32-unknown-unknown` + `cargo test --workspace` grün
  **UI hint**: yes

<details>
<summary>✅ v1.3 Frontend Abwesenheiten + UI-Closure-Restanten (Phasen 8–13) — CLOSED 2026-06-22</summary>

Geliefert: Phasen 8 (Absence-CRUD-Page), 8.2 (Manual-Range-Convert), 8.3 (Halbtag-Support), 8.4 (Reporting-Additiv-Merge + Koexistenz-Modell), 8.5 (Read-Projektion + HR-Inline-Convert), 8.6 (Cutover-Abriss), 9 (Booking-Flow Reverse-Warnings; Copy-Week descoped).
⊘ Superseded: 8.1 (Batch-Cutover-Wizard) + 11 (Deprecation-Handling) — abgelöst durch das per-row Koexistenz-Modell (8.4–8.6).
Bewusst aufgegeben: 8.3-Restscope-Folgen, 10 (Shiftplan-View Unavailability-Marker), 12 (UI-Closure v1.1/v1.2-Restanten — FUI-01..04), 13 (i18n-Audit + Smoke-Closure) — nicht ausgeliefert, kein Rest-Scope in v1.4 gezogen.

**Full milestone archive (vollständige Phasen-Detail + Plan-Listen + Progress):** [`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md), Phasen-Artefakte unter [`milestones/v1.3-phases/`](milestones/v1.3-phases/).

</details>

<details>
<summary>✅ v1.0 Range-Based Absence Management (Phasen 1–4) — SHIPPED 2026-05-03</summary>

- [x] **Phase 1: Absence Domain Foundation** (5/5 plans) — completed 2026-05-01
  Neue parallele `absence` Domain (DAO + Service + REST + Permission), additiv, ohne Reporting-Wirkung
- [x] **Phase 2: Reporting Integration & Snapshot Versioning** (4/4 plans) — completed 2026-05-02
  `derive_hours_for_range` + Reporting-Switch hinter Feature-Flag, `CURRENT_SNAPSHOT_SCHEMA_VERSION` 2 → 3 im selben Commit
- [x] **Phase 3: Booking & Shift-Plan Konflikt-Integration** (6/6 plans) — completed 2026-05-02
  Forward/Reverse Booking-Warnings + Shift-Plan-Anzeige aus AbsencePeriod ohne Doppel-Eintragung
- [x] **Phase 4: Migration & Cutover** (8/8 plans) — completed 2026-05-03
  Heuristik-Migration, Validierungs-Gate (< 0.01h Drift-Toleranz), atomarer Feature-Flag-Flip mit Carryover-Refresh, REST-Deprecation. Plus Bonus-Recovery von `extra_hours.update` mit logical_id-Versionierung.

**Full milestone archive:** [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)

</details>

<details>
<summary>✅ v1.1 Slot Capacity & Constraints (Phase 5) — SHIPPED 2026-05-04</summary>

- [x] **Phase 5: Slot Paid Capacity Warning** (6/6 plans) — completed 2026-05-04
  Slots erhalten ein optionales `max_paid_employees: Option<u8>` Capacity-Limit. Backend emittiert nicht-blockierende `Warning::PaidEmployeeLimitExceeded` (a) im `BookingCreateResult.warnings` im Conflict-Aware-Booking-Flow und (b) als `current_paid_count` per Slot im Shiftplan-Week-View. 461 Tests grün; 16/16 D-decisions verified. Frontend (shifty-dioxus) out of scope.

**Full milestone archive:** [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)

</details>

<details>
<summary>✅ v1.2 Frontend rest-types Konsolidierung (Phasen 6–7) — SHIPPED 2026-05-07</summary>

- [x] **Phase 6: rest-types Unification & Frontend Compile-Through** (5/5 plans) — completed 2026-05-07
  Backend-`rest-types` als single source of truth verdrahtet, Frontend-Fork gelöscht, 17 fehlende TOs/Enum-Varianten + 4 fehlende Felder + Match-Arme adressiert; `cargo build --target wasm32-unknown-unknown` grün; 466 Backend-Tests ohne Regression. 8/8 V-Truths verified.
- [x] **Phase 7: Runtime Smoke & Regression Safety** (1/1 plan) — completed 2026-05-07
  Frontend-Boot, Login und Shiftplan-Navigation auf Integrationsumgebung verifiziert (User-UAT 2026-05-07); Backend `cargo check --workspace` + `cargo test --workspace` re-verifiziert (Subsumption von Phase-6 V-Truth #6 + #7 plus lokaler Re-Run). 4/4 Success Criteria verified.

**Full milestone archive:** [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)

</details>

## Phase Details

> v1.0–v1.3 Phasen-Details sind in den Milestone-Archiven unter [`milestones/`](milestones/) abgelegt (siehe `<details>`-Blöcke oben). Die folgenden Detail-Sektionen decken die aktive v1.4 ab.

### Phase 14: Data-model foundation (backend)

**Goal:** Das zeit-versionierte Feld `committed_voluntary: f32` (D-01 / Variante B — nur die freiwillige Zusage obendrauf, entkoppelt von `expected_hours`) existiert durchgängig auf `EmployeeWorkDetails` über alle Layer (SQLite-Migration → DAO → Service → `rest-types`). Das Feld ist **inert** (nirgends gelesen): es transportiert und persistiert, hat aber noch keine Reporting-/Display-Wirkung. Damit ist die Foundation gelegt, auf der Phase 15 (Reporting) lesen und Phase 16/17 (Frontend) konsumieren/editieren kann.

**Depends on:** Nothing (erste v1.4-Phase). Baut auf der existierenden v1.3-`EmployeeWorkDetails`-Infrastruktur auf; kopiert die `cap_planned_hours_to_expected`-Präzedenz (v1.3) und `is_dynamic` Zeile-für-Zeile.

**Requirements:** CVC-01, CVC-02, CVC-03

**Success Criteria** (was muss WAHR sein, nachdem die Phase abgeschlossen ist):

1. Additive Migration `… ADD COLUMN committed_voluntary REAL NOT NULL DEFAULT 0` läuft via `sqlx migrate run` (NICHT `reset`); `.sqlx`-Offline-Cache via `cargo sqlx prepare` regeneriert; `cargo check --workspace` + `cargo test --workspace` grün; Bestandsdaten driftfrei (Default 0, kein Reporting-Effekt da das Feld nirgends gelesen wird) (CVC-01).
2. `committed_voluntary` ist auf DAO-Entity/Row, Service-Struct und `EmployeeWorkDetailsTO` (mit `#[serde(default)]` für Wire-Backward-Compat) präsent; beide Konversionsrichtungen an jeder Boundary (DAO `TryFrom`, Service↔DAO, TO↔Service) durchgezogen; ein erweiterter `employee_work_details_update`-Integrationstest verifiziert einen fraktionalen Open→Save→Reload-Round-Trip. KEIN REST/OpenAPI-Change (diese Endpoint-Familie hat bewusst keine `#[utoipa::path]`/`ToSchema` — serde-transparent, kein Phantom-Task) (CVC-01).
3. Beim Rotieren einer Vertrags-Version (neue Zeit-Periode) wird `committed_voluntary` korrekt mitgeführt und nicht still auf Default zurückgesetzt; ein Test über die Update/Rotate-Pfad-Struct-Update-Spread verifiziert die Carry-Forward-Semantik (CVC-02).
4. Die Aggregation bei zwei überlappenden aktiven `EmployeeWorkDetails`-Rows in derselben ISO-Woche ist explizit definiert (sum/max/first, gelesen über dieselbe `find_working_hours_for_calendar_week`-Selektion wie `expected_hours`) und durch einen Test gepinnt (D-OVERLAP-AGG — das Boolean-`.any()`-Pattern des Cap-Flags generalisiert nicht auf einen numerischen Wert) (CVC-03).

**Plans:** 2/2 plans complete

Plans:
**Wave 1**
- [x] 14-01-PLAN.md — committed_voluntary durch alle Backend-Layer (Migration -> DAO -> Service -> rest-types + Carry-Forward)

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 14-02-PLAN.md — Tests: Round-Trip (CVC-01), Carry-Forward (CVC-02), SUM-Aggregation (CVC-03 / D-OVERLAP-AGG)

**Notes for plan-phase:** Reine Backend-Phase, **kein Frontend-Anteil** (Feld inert; Frontend folgt in Phase 16/17 — begründeter Skip im Sinne der GSD-Scope-Regel). Zwei exakte Präzedenzfälle: `cap_planned_hours_to_expected` (v1.3) und `is_dynamic`. `EmployeeWorkDetailsService` bleibt **Basic Service** (Feld ist Daten, keine Dependency — keine neue DI-Verdrahtung). `.sqlx`-Regen ist der erste harte Compile-Gate. Auf NixOS: `nix develop` + `sqlx migrate run` (additiv); **niemals** `sqlx database reset` (destruktiv, braucht User-Confirmation). D-OVERLAP-AGG ist in der CONTEXT-Decision-Liste zu fixieren. Skip research-phase (Standard-Pattern, line-für-line-Präzedenz).

---

### Phase 15: Reporting no-double-count (Achse B only, KEIN Snapshot-Bump)

**Goal:** Die Jahresansicht-Verfügbarkeit rechnet die Zusage **ohne Doppelzählung** ein — als **separater** `committed_voluntary_hours`-Term in `booking_information.rs::get_weekly_summary` (**Achse B**, NICHT `reporting.rs`/Achse A), per ISO-Woche via Zwei-Band-Dekomposition (D-05 FORMULA B): Band 1 = cap-gated Σ_Person committed, Band 2 = Σ_Person max(actual_p − committed_p, 0), summiert über das Jahr (nie `max(Σ, Σ)`); gegated auf `cap_planned_hours_to_expected = true`. KEIN Snapshot-Bump: die Zwei-Band-Dekomposition (D-05) fließt ausschließlich in Achse B (`get_weekly_summary`/Jahresansicht) und NICHT in den persistierten `BillingPeriodValueType::Volunteer` (der speist sich aus `reporting.rs`/Achse A). Version bleibt 7 (D-01; CLAUDE.md "purely additive changes that do not touch the snapshot's value_types"). Dies ist die **höchst-Risiko-Phase** des Milestones (D-FORMULA-PATH, D-SCOPE-GATE).

**Depends on:** Phase 14 (das `committed_voluntary`-Feld muss auf dem Service-Struct existieren, bevor Reporting es lesen kann). KEIN Snapshot-Bump (D-01 revidiert): die Zwei-Band-Dekomposition ist Achse-B-only und berührt keinen persistierten `BillingPeriodValueType` — `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 7.

**Requirements:** CVC-04, CVC-05, CVC-06

**Success Criteria** (was muss WAHR sein, nachdem die Phase abgeschlossen ist):

1. `get_weekly_summary` exponiert einen **neuen separaten** `committed_voluntary_hours`-Term (nicht in den `paid`- oder `volunteer`-Term gefaltet), berechnet per ISO-Woche via `counted_volunteer = max(committed_voluntary, actual_volunteer)` und **dann** über das Jahr summiert (niemals `max(Σ, Σ)`); ein Worked-Example-Fixture-Test (z.B. committed=5/actual=7 → 7; committed=5/actual=3 → 5) pinnt die per-Woche-vor-Summe-Reihenfolge und schließt Doppelzählung gegen Achse B strukturell aus (CVC-04).
2. Jeder Read von `committed_voluntary` ist auf `cap_planned_hours_to_expected = true` gegated; für nicht-gedeckelte Personen (`cap = false`) trägt die Zusage `0.0` zu allen Berechnungen bei. Ein Backward-Compat-Regressionstest verifiziert: `committed = 0` ⇒ Ergebnis bit-identisch zu vor v1.4 (CVC-06).
3. `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **7** (KEIN Bump) — begründet im Audit-Trail (Phase 15 berührt keinen persistierten `value_type`; `WeeklySummary` wird von `billing_period_report.rs` nie gelesen). Ein Regressionstest `snapshot_schema_version_unchanged_at_7` bestätigt die unveränderte Version (CVC-05).
4. `cargo test --workspace` grün inkl. Fixture- + Snapshot-Validator-Tests; Billing-Period-Snapshot-Regression sauber; die Integration landet ausschließlich in Achse B (`booking_information.rs`), `reporting.rs`/Achse A bleibt für den Jahresansicht-Pfad unangetastet (verifiziert, dass kein zweiter Einrechnungspfad existiert) (CVC-04).

**Plans:** 2/2 plans complete

Plans:
**Wave 1**
- [x] 15-01-PLAN.md — committed_voluntary_hours-Feld auf WeeklySummary + pure FORMULA-A-Reduktions-Helper + cap-gegateter Einbau in get_weekly_summary (CVC-04, CVC-06)

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 15-02-PLAN.md — D-02-Maximal-Fixture-Suite (9 Fixtures, FORMULA B: multi_person=committed 5.0/volunteer 3.0/total 8.0) + no-bump-Regressionstest (Version bleibt 7) + ROADMAP/REQUIREMENTS-Reconciliation (CVC-04, CVC-05, CVC-06)

**Notes for plan-phase:** **Höchst-Risiko-Phase — Design-Note vor dem Plan empfohlen.** D-FORMULA-PATH (Achse B / `booking_information.rs`, NICHT `reporting.rs`) und D-SCOPE-GATE (neuer separater Term, gegated auf `cap = true`; der „5h paid + 5h pledged"-Fall) sind zuerst in der CONTEXT-Decision-Liste zu fixieren. Business-Logic-Tier, **keine neue DI-Dependency**. **KEIN Snapshot-Bump** (D-01 revidiert 2026-06-23): die Zwei-Band-Dekomposition ist Achse-B-only (`get_weekly_summary`/Jahresansicht) und berührt keinen persistierten `BillingPeriodValueType` — `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 7 (siehe `CLAUDE.md` § Billing Period Snapshot Schema Versioning: "purely additive changes that do not touch the snapshot's value_types"). Anti-Pattern: NICHT den `reporting.rs`-Cap-Overflow-Pfad (Achse A) für die Jahresansicht-Zahl wiederverwenden (doppelt gegen Achse B). D-PARTIAL-WEEK: flat per aktiver Woche, kein Pro-rating. D-ABSENCE-DISPLAY: keine Kopplung an Absence/Urlaub/Feiertag. Reine Backend-Phase (der Frontend-Display-Teil folgt in Phase 16 — begründeter Skip im Sinne der GSD-Scope-Regel: hier wird nur der Service-/TO-Term produziert, gerendert wird in 16).

---

### Phase 16: Jahresansicht display

**Goal:** Die Jahresansicht (`weekly_overview`) zeigt die committed-Kapazität als **separaten** Token („zugesagt") — sichtbar getrennt von `paid` und `volunteer` —, inklusive sichtbar ausgewiesenem Überschuss (committed=5, actual=7 → `5 + 2`; committed=5, actual=3 → gedeckte `5`). Der in Phase 15 produzierte `committed_voluntary_hours`-Term wird durch `WeeklySummary` → `WeeklySummaryTO` → Frontend gefädelt und in `state/weekly_overview.rs` + `page/weekly_overview.rs` als dritter Token gerendert. In der aggregierten Jahresansicht zeigt der committed-Token bei `committed_voluntary_hours == 0` schlicht `0.00` (konsistent mit paid/volunteer) — **keine** blank/Strich-Sonderlogik (revidiert per Phase-16-CONTEXT D-03; die blank/Strich-Idee gehört, falls überhaupt, in die Mitarbeiteransicht / Phase 17). Zusätzlich bekommt der `WeeklyOverviewChart` ein drittes gestapeltes Farb-Segment (paid/committed/surplus) — CVC-F-02 wird per D-04 in Phase 16 vorgezogen. Alle neuen Strings sind in De/En/Cs vollständig.

**Depends on:** Phase 15 (`committed_voluntary_hours` auf den Report-/Summary-Strukturen). Read-/Display-Pfad — niedrigeres Risiko und self-contained, daher vor dem design-offenen Editier-Pfad (Phase 17).

**Requirements:** CVC-07, CVC-08

**Success Criteria** (was muss WAHR sein, nachdem die Phase abgeschlossen ist):

1. `weekly_overview` zeigt die committed-Kapazität als eigenen Token („zugesagt") — nicht mit `paid`/`volunteer` vermischt; der `From<&WeeklySummaryTO>`-Mapping-Pfad trägt das neue Feld vollständig (keine Mapping-Omission-Lücke) (CVC-07).
2. Der Überschuss wird sichtbar ausgewiesen: committed=5, actual=7 → `🎯5.00 | 🤝2.00`; committed=5, actual=3 → gedeckte `🎯5.00 | 🤝0.00`. Bei `committed_voluntary_hours == 0` zeigt der committed-Token `🎯0.00` (plain zero, zwei Dezimalstellen), **keine** blank/Strich-Sonderlogik (revidiert per Phase-16-CONTEXT D-03) (CVC-07).
3. Alle neuen benutzersichtbaren Strings sind in De / En / Cs vollständig gepflegt; Per-Locale-Reference-Matcher-Tests (analog v1.3) schließen den `Locale::En`-statt-`Locale::De`-Bug aus (CVC-08).
4. `cargo build --target wasm32-unknown-unknown` im `shifty-dioxus/`-Subordner liefert Exit-Code 0; `cargo test --workspace` im Backend-Root grün (Backend-TO-Erweiterung ohne Regression).

**Plans:** 3/3 plans complete

Plans:
**Wave 1**
- [x] 16-01-PLAN.md — Backend: overall_available_hours (D-01) + WeeklySummaryTO-Feld/From-Mapping (CVC-07b)

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 16-02-PLAN.md — Frontend-State: WeeklySummary committed-Feld + From-Mapping + WASM-Gate (CVC-07c)

**Wave 3** *(blocked on Wave 2 completion)*
- [x] 16-03-PLAN.md — Frontend-Render: dritter Token (D-02/D-03) + drittes Chart-Segment (D-04) + i18n De/En/Cs + cs.rs-Lücken (CVC-08)

**UI hint**: yes

**Notes for plan-phase:** Backend (`WeeklySummary` + `WeeklySummaryTO` + `From`) **und** Frontend (`state/weekly_overview.rs` + `page/weekly_overview.rs`) im selben Monorepo — Frontend trails das Backend-TO in derselben Phase (die einzige `rest-types`-Crate bricht den WASM-Compile, falls ein Feld unmirrored bleibt — erzwungener Sync). Überschuss-Display kann `diff_color_and_sign`-Token-Reuse nutzen. Inline-Banner „Zusage nicht erfüllt" (committed > actual) ist **v1.5** (CVC-F-01), NICHT hier. Eigenes committed-Band im `WeeklyOverviewChart` ist **v1.5** (CVC-F-02). D-ABSENCE-DISPLAY: jeglicher „Hide-on-full-holiday-week"-Filter ist display-only, bleibt aus der Kern-Formel raus.

---

### Phase 17: Contract editor input + „alle"-Filter / unpaid-volunteer path

**Goal:** `committed_voluntary` wird im Vertrags-Editor (`contract_modal.rs`, neben dem Cap-Toggle) als numerisches Feld editierbar (Open→Save-unverändert-Round-Trip bewahrt den Backend-Wert). Die Mitarbeiteransicht bekommt einen einblendbaren „alle"-Filter; rein unbezahlte Freiwillige können einen `EmployeeWorkDetails`-Record halten (`is_paid = false`, `expected_hours = 0`) und werden sichtbar/auswählbar. Kritisch: Jede work-details-iterierende paid-only-Site ist explizit auf `sales_person.is_paid` gegated (nicht auf Record-Präsenz) — kein Leak in `paid_hours`/Billing/Year-Summary. Dies ist die **design-offenste Phase** (D-UNPAID-RECORD).

**Depends on:** Phase 14 (das `EmployeeWorkDetailsTO`-Feld muss existieren, damit der Editor es schreiben kann). Aufgeteilt nach Phase 16, damit die offene paid-only-Assumption-Design-Frage den niedrig-riskanten Display-Pfad nicht blockiert.

**Requirements:** CVC-09, CVC-10

**Success Criteria** (was muss WAHR sein, nachdem die Phase abgeschlossen ist):

1. `committed_voluntary` ist im Vertrags-Editor (`contract_modal.rs`, neben dem Cap-Toggle) als numerisches Feld editierbar; ein Open→Save-unverändert-Round-Trip bewahrt den Backend-Wert (beide `TryFrom`-Richtungen in `state/employee_work_details.rs`; numerisches Input — der `<input type=date>`-Signal-Caveat gilt hier NICHT) (CVC-09).
2. Die Mitarbeiteransicht bekommt einen einblendbaren „alle"-Filter; rein unbezahlte Freiwillige können einen `EmployeeWorkDetails`-Record halten (`is_paid = false`, `expected_hours = 0`) und werden über den Filter sichtbar/auswählbar (CVC-10).
3. Jede enumerierte work-details-iterierende paid-only-Site ist explizit auf `sales_person.is_paid` gegated (nicht auf Record-Präsenz); ein `get_week`-Seiteneffekt-Integrationstest verifiziert: kein Leak in `paid_hours` / Billing / Year-Summary und Personen-Set-Konsistenz über year-summary / all-employees-report / Billing (CVC-10).
4. i18n De / En / Cs für neue Labels/Filter vollständig (Per-Locale-Reference-Matcher); `cargo build --target wasm32-unknown-unknown` + `cargo test --workspace` grün.

**Plans:** TBD

**UI hint**: yes

**Notes for plan-phase:** **Design-offenste Phase — D-UNPAID-RECORD vor dem Plan in CONTEXT fixieren.** `is_paid` lebt auf `SalesPerson`, NICHT auf `EmployeeWorkDetails` — die historische „has work-details ⇒ paid employee"-Annahme bricht. Die enumerierten at-risk-Sites (aus SUMMARY.md/PITFALLS.md) MÜSSEN jede einzeln auf `is_paid` gegated werden: `reporting::get_week` (`all_for_week`, nicht paid-gefiltert — die Haupt-Überraschung), `booking_information` `paid_hours`-Akkumulation, `reporting::get_reports_for_all_employees` (is_paid-gefiltert → Personen-Set-Inkonsistenz), `booking_information` day-level loop, `billing_period_report::build_new_billing_period` (`get_all`, kein paid-Filter), `vacation_balance` (`get_all_paid` — verifizieren, dass nichts direkt work-details liest). Anti-Pattern: unpaid Volunteers KEINEN paid-style-Record geben (`expected_hours > 0` würde sie in paid-Loops flippen). Backend (Filter-Relaxierung in `loader.rs`/`reporting.rs`/`booking_information.rs`) + Frontend (`contract_modal.rs`, `state/employee_work_details.rs`) in derselben Phase.

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
| 14 — Data-model foundation (backend) | v1.4 | 2/2 | Complete    | 2026-06-23 |
| 15 — Reporting no-double-count (KEIN Snapshot-Bump) | v1.4 | 2/2 | Complete    | 2026-06-24 |
| 16 — Jahresansicht display | v1.4 | 3/3 | Complete   | 2026-06-24 |
| 17 — Contract editor input + „alle"-Filter / unpaid-volunteer | v1.4 | 0/? | Not started | — |

---

*Last updated: 2026-06-22 — **v1.4-Roadmap erstellt.** v1.3 closed (Phasen 8–13; 8.1/11 superseded, 8.3/10/12/13 bewusst aufgegeben; vollständiges Detail in `milestones/v1.3-ROADMAP.md`). Neue aktive Phasen 14–17 (Committed Voluntary Capacity): 14 Data-model foundation (CVC-01..03), 15 Reporting no-double-count Achse-B-only KEIN Snapshot-Bump (CVC-04..06; D-01 revidiert 2026-06-23), 16 Jahresansicht-Display (CVC-07/08), 17 Contract-Editor + „alle"-Filter/unpaid-volunteer (CVC-09/10). 10/10 CVC-Requirements gemappt, 0 Orphans. Build-Order strikt compile-dependency-geordnet (Achse B / `booking_information.rs`, NICHT `reporting.rs`).*
