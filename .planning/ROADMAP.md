# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (shipped 2026-05-07) — siehe [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)
- ✅ **v1.3 Frontend Abwesenheiten + UI-Closure-Restanten** — Phasen 8–13 (closed 2026-06-22; geliefert: 8, 8.2, 8.4, 8.5, 8.6, 9; 8.1/11 ⊘ superseded; **8.3/10/12/13 bewusst aufgegeben**) — siehe [`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md)
- ✅ **v1.4 Committed Voluntary Capacity** — Phasen 14–17 (shipped 2026-06-25) — siehe [`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md)
- ◆ **v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen** — Phasen 18–22 (aktiv, gestartet 2026-06-25)

## Phases

### ◆ v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen (Phasen 18–22)

**Milestone-Goal:** Urlaubs-/Abwesenheitswerte sind überall konsistent, das Umwandeln
stundenbasierter Legacy-Einträge braucht für HR nur noch minimale Handarbeit, die
Mitarbeiter-Jahresansicht ist schnell les- und zuordenbar, und HR bekommt pro
Mitarbeiter eine Auswertung der durchschnittlichen Anwesenheit.

12 Requirements (UV-01..05, YV-01..03, STAT-01/02, UI-01/02) → 5 Phasen. Coverage 100 %.

---

#### Phase 18 — Report-/Balance-Korrektheit (Backend)

- [ ] **Goal:** Urlaubs-Carryover und Urlaubstage-Zählung im Reporting sind nach
  Absence-Konvertierung und gegenüber dem Report-Service konsistent.
- **Requirements:** UV-04, UV-05
- **Ebene:** Backend (`service_impl/src/reporting.rs`, `service_impl/src/vacation_balance.rs`, `service/src/reporting.rs`)
- **Success Criteria:**
  1. Carryover-Resturlaub in der Vacation-Balance entspricht für beliebige Mitarbeiter dem Wert des Report-Service (`year-1`-Quelle).
  2. Nach Konvertierung eines stundenbasierten Urlaubseintrags in eine Absence Period zeigt der Employee-Report `vacation_days` weiterhin korrekt (>0, deckungsgleich mit den Stunden) — nicht 0.
  3. `sick_leave_days` und `absence_days` zählen die derived Absences ebenfalls mit, ohne Doppelzählung.
  4. Regressionstests decken beide Pfade ab; `cargo test --workspace` grün.
- **Snapshot-Hinweis:** `CURRENT_SNAPSHOT_SCHEMA_VERSION` nur bumpen, falls sich eine persistierte `value_type`-Berechnung ändert (in discuss/plan prüfen — UV-05 berührt Display-/Tage-Ableitung, voraussichtlich kein Bump).

#### Phase 19 — Convert-Dialog UX (Frontend)

- [ ] **Goal:** Das „In Zeitraum umwandeln"-Modal schlägt sinnvolle Zeiträume vor, sodass HR kaum noch manuell rechnen muss.
- **Requirements:** UV-01, UV-02
- **Ebene:** Frontend (`shifty-dioxus/src/component/absence_convert_modal.rs`, `shifty-dioxus/src/page/absences.rs`)
- **Success Criteria:**
  1. Beim Öffnen des Convert-Modals ist „bis" arbeitstagbasiert vorbelegt (Wochenende + Feiertage übersprungen), sodass der Zeitraum den berechneten Urlaubstagen entspricht.
  2. Entsprechen die Stunden exakt dem Wochen-Soll, zeigt die Anzeige „1 Woche" und das Modal schlägt Mo–So der betroffenen Kalenderwoche vor.
  3. Bei allen anderen Werten gilt die Arbeitstage-/Tage-Logik (keine Vielfachen, keine Teilwochen).
  4. Frontend-Tests + `cargo build --target wasm32-unknown-unknown` grün.

#### Phase 20 — Absences-Indikator & Jahres-Histogramm (Frontend)

- [ ] **Goal:** Stundenbasierte Einträge sind als „noch nicht echter Urlaub" erkennbar, und die Mitarbeiter-Jahresansicht ist gut les-/zuordenbar inkl. Freiwilligen-Stunden.
- **Requirements:** UV-03, YV-01, YV-02, YV-03
- **Ebene:** Frontend (`shifty-dioxus/src/page/absences.rs` `HourlyMarkerRow`; `shifty-dioxus/src/component/employee_weekly_histogram.rs`; `shifty-dioxus/src/component/employee_view.rs`)
- **Success Criteria:**
  1. Stundenbasierte Marker auf `/absences` zeigen einen ⚠️-Indikator am Zeilenanfang.
  2. Histogramm-Balken (`EmployeeWeeklyHistogram`) zeigen im Hover KW + von–bis Datum.
  3. Wo bisher nur die KW-Nummer stand (X-Achse / aufgeklappte KW-Liste), steht jetzt zusätzlich das von–bis Datum.
  4. Freiwilligen-Stunden (`volunteer_hours`) erscheinen gestapelt im Histogramm + als separater Wert in der aufgeklappten KW-Liste / `WeekDetailPanel`.
  5. Frontend-Tests + WASM-Build grün.

#### Phase 21 — Tabellen-Lesbarkeit (Frontend)

- [ ] **Goal:** Breite Tabellen auf großen Bildschirmen sind zeilenweise gut lesbar.
- **Requirements:** UI-01, UI-02
- **Ebene:** Frontend (`shifty-dioxus/src/component/working_hours_mini_overview.rs`, `shifty-dioxus/src/page/absences.rs`)
- **Success Criteria:**
  1. Die Stunden-Tabelle unter dem Schichtplan (`WorkingHoursMiniOverview` TableLayout) hat eine maximale Breite + Zebra-Striping.
  2. In der `/absences`-Tabelle ist die Mitarbeiter-Spalte deutlich schmaler (weg von `1.5fr`).
  3. Frontend-Tests + WASM-Build grün.

#### Phase 22 — Mitarbeiter-Statistik HR (Backend + Frontend)

- [ ] **Goal:** HR sieht pro Mitarbeiter eine Statistik, primär die durchschnittlich gearbeiteten Stunden pro Woche (urlaubsbereinigt).
- **Requirements:** STAT-01, STAT-02
- **Ebene:** Backend (`ReportingService` / neue Berechnung + REST) + Frontend (neue HR-Ansicht)
- **⚠ Braucht eigene `discuss-phase`** — offene Definitionen: Bezugszeitraum, Definition „gearbeitet", welche Abwesenheiten aus dem Nenner fallen, nur flexible Verträge oder alle. Setzt Todo `AVG-01` um.
- **Success Criteria:**
  1. Eine pro-SalesPerson Statistik-Ansicht ist ausschließlich mit HR-Rolle zugänglich/sichtbar.
  2. Die Ansicht zeigt die durchschnittlich gearbeiteten Stunden pro Woche, mit aus dem Nenner herausgerechneten Abwesenheitszeiträumen (Definition aus discuss).
  3. Backend-Berechnung + REST + Frontend getestet; `cargo test --workspace` + WASM-Build grün.

**Abhängigkeiten:** Phase 18 (BE) ist unabhängig. 19/20/21 (FE) sind unabhängig voneinander. 22 baut konzeptionell auf der Reporting-Ecke (18) auf, ist aber separat planbar.

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
| 18 — Report-/Balance-Korrektheit (BE) | v1.5 | 0/? | Planned | — |
| 19 — Convert-Dialog UX (FE) | v1.5 | 0/? | Planned | — |
| 20 — Absences-Indikator & Jahres-Histogramm (FE) | v1.5 | 0/? | Planned | — |
| 21 — Tabellen-Lesbarkeit (FE) | v1.5 | 0/? | Planned | — |
| 22 — Mitarbeiter-Statistik HR (BE+FE) | v1.5 | 0/? | Planned | — |

---

*Last updated: 2026-06-25 — **v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung gestartet** (Phasen 18–22, 12 Requirements UV/YV/STAT/UI, Coverage 100 %). Nächster Schritt: gebündelte Discuss-Runde (Schwerpunkt STAT), dann autonomer plan+execute-Lauf.*
