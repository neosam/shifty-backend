# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (shipped 2026-05-07) — siehe [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)
- ✅ **v1.3 Frontend Abwesenheiten + UI-Closure-Restanten** — Phasen 8–13 (closed 2026-06-22; geliefert: 8, 8.2, 8.4, 8.5, 8.6, 9; 8.1/11 ⊘ superseded; **8.3/10/12/13 bewusst aufgegeben**) — siehe [`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md)
- ✅ **v1.4 Committed Voluntary Capacity** — Phasen 14–17 (shipped 2026-06-25) — siehe [`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md)
- ◆ **v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen** — Phasen 18–23 (aktiv, gestartet 2026-06-25)

## Phases

### v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen (active)

**Milestone-Goal:** Urlaubs-/Abwesenheitswerte sind überall konsistent, das Umwandeln
stundenbasierter Legacy-Einträge braucht für HR nur noch minimale Handarbeit, die
Mitarbeiter-Jahresansicht ist schnell les- und zuordenbar, und HR bekommt pro
Mitarbeiter eine Auswertung der durchschnittlichen Anwesenheit.

12 Requirements (UV-01..05, YV-01..03, STAT-01/02, UI-01/02) → 5 Phasen. Coverage 100 %.

- [x] **Phase 18: Report-/Balance-Korrektheit** (Backend) — Carryover-Quelle (`year-1`) angleichen + Urlaubstage-Zählung nach Absence-Konvertierung korrigieren (derived Absences in per-Woche-Kategorien mergen, ohne Doppelzählung). ✅ 2026-06-26
  Code: `service_impl/src/reporting.rs`, `service_impl/src/vacation_balance.rs`, `service/src/reporting.rs`. Snapshot-Hinweis: `CURRENT_SNAPSHOT_SCHEMA_VERSION` Bump 9→10 nötig, weil die persistierte `VacationDays`-Computation sich ändert (konvertierte Einträge: 0 → >0).
  Requirements: UV-04, UV-05
  Success Criteria:
  1. Carryover-Resturlaub in der Vacation-Balance entspricht für beliebige Mitarbeiter dem Wert des Report-Service (`year-1`-Quelle).
  2. Nach Konvertierung eines stundenbasierten Urlaubseintrags in eine Absence Period zeigt der Employee-Report `vacation_days` weiterhin korrekt (>0, deckungsgleich mit den Stunden) — nicht 0.
  3. `sick_leave_days` und `absence_days` zählen die derived Absences ebenfalls mit, ohne Doppelzählung.
  4. Regressionstests decken beide Pfade ab; `cargo test --workspace` grün.
  Plans: 2 plans
  - [x] 18-01-PLAN.md — UV-04: Carryover auf `year-1`-Quelle pinnen (Fix bereits präsent) + Regressionstest, der den `year-1`-Read im Mock-Matcher festnagelt.
  - [x] 18-02-PLAN.md — UV-05: derived Absences in per-Woche-Kategorien mergen (ungated), Jahreslumpen-Doppelzählung in `get_report_for_employee` entfernen (Single Source = `by_week`), Snapshot-Bump 9→10, Regressionstests (days>0 stabil über Konvertierung, kein Double-Count, sick_leave-Fall).

- [x] **Phase 19: Convert-Dialog UX** (Frontend+Backend) — Smart bis-Datum (arbeitstagbasiert) + exakter Wochen-Fall („1 Woche" + Mo–So-Vorschlag) im „In Zeitraum umwandeln"-Modal. ✅ 2026-06-26
  Code: `shifty-dioxus/src/component/absence_convert_modal.rs`, `shifty-dioxus/src/page/absences.rs`.
  Requirements: UV-01, UV-02
  Success Criteria:
  1. Beim Öffnen des Convert-Modals ist „bis" arbeitstagbasiert vorbelegt (Wochenende + Feiertage übersprungen), sodass der Zeitraum den berechneten Urlaubstagen entspricht.
  2. Entsprechen die Stunden exakt dem Wochen-Soll, zeigt die Anzeige „1 Woche" und das Modal schlägt Mo–So der betroffenen Kalenderwoche vor.
  3. Bei allen anderen Werten gilt die Arbeitstage-/Tage-Logik (keine Vielfachen, keine Teilwochen).
  4. Frontend-Tests + `cargo build --target wasm32-unknown-unknown` grün.
  Plans: 2 plans (Arch-Entscheidung: FE+BE — Backend rechnet die Vorschlagswerte vor; FE nur Wiring/Anzeige)
  - [x] 19-01-PLAN.md — Backend: `suggested_end` + `is_full_week` auf `ExtraHoursMarkerTO`, `AbsenceService::suggest_convert_ranges_for_markers` (Arbeitstag/Feiertag/Wochen-Cap + Exakt-Wochen-Soll), Wiring in beide List-Handler + Tests.
  - [x] 19-02-PLAN.md — Frontend: Felder durch `ExtraHoursMarker`-State + Modal-Props threaden, bis aus `suggested_end` vorbelegen, „1 Woche"/„N Tage" in `HourlyMarkerRow`, i18n (De/En/Cs), SSR-Tests + WASM-Gate.

- [x] **Phase 20: Absences-Indikator & Jahres-Histogramm** (Frontend) — ⚠️-Indikator bei stundenbasierten Einträgen; Histogramm-Hover (KW+Datum), KW+Datum-Beschriftung und gestapelte Freiwilligen-Stunden. ✅ 2026-06-26
  Code: `shifty-dioxus/src/page/absences.rs` `HourlyMarkerRow`; `shifty-dioxus/src/component/employee_weekly_histogram.rs`; `shifty-dioxus/src/component/employee_view.rs`.
  Requirements: UV-03, YV-01, YV-02, YV-03
  Success Criteria:
  1. Stundenbasierte Marker auf `/absences` zeigen einen ⚠️-Indikator am Zeilenanfang.
  2. Histogramm-Balken (`EmployeeWeeklyHistogram`) zeigen im Hover KW + von–bis Datum.
  3. Wo bisher nur die KW-Nummer stand (X-Achse / aufgeklappte KW-Liste), steht jetzt zusätzlich das von–bis Datum.
  4. Freiwilligen-Stunden (`volunteer_hours`) erscheinen gestapelt im Histogramm + als separater Wert in der aufgeklappten KW-Liste / `WeekDetailPanel`.
  5. Frontend-Tests + WASM-Build grün.
  Plans: 2 plans (Wave 1, parallel — keine Datei-Überschneidung außer den additiv-erweiterten i18n-Dateien; UV-03 disjunkt vom Histogramm)
  - [x] 20-01-PLAN.md — UV-03: ⚠️-Indikator führend in `HourlyMarkerRow` Spalte 1 (statisches Tailwind, i18n title+aria in De/En/Cs), Badge bleibt; SSR-Test.
  - [x] 20-02-PLAN.md — YV-01/02/03: gestapelte Balken (regulär + volunteer dezent) + `<title>`-Hover (KW+Datum), `compute_max_y` auf Stapel-Summe, KW+Datum + separater volunteer-Wert in `WeekListExpanded`/`WeekDetailPanel`, i18n; SSR-Tests + WASM-Gate.

- [x] **Phase 21: Tabellen-Lesbarkeit** (Frontend) — max-width + Zebra für die Schichtplan-Tabelle; schmalere Mitarbeiter-Spalte in der `/absences`-Tabelle. ✅ 2026-06-26
  Code: `shifty-dioxus/src/component/working_hours_mini_overview.rs`, `shifty-dioxus/src/page/absences.rs`.
  Requirements: UI-01, UI-02
  Success Criteria:
  1. Die Stunden-Tabelle unter dem Schichtplan (`WorkingHoursMiniOverview` TableLayout) hat eine maximale Breite + Zebra-Striping.
  2. In der `/absences`-Tabelle ist die Mitarbeiter-Spalte deutlich schmaler (weg von `1.5fr`).
  3. Frontend-Tests + WASM-Build grün.
  Plans: 1 plan (Wave 1 — beide UI-Polish-Änderungen sind kleine Tailwind-Edits, zusammen ~15 % Kontext)
  - [x] 21-01-PLAN.md — UI-01: max-width (`max-w-5xl`) + Zebra-Striping (Design-Tokens, Selected/Hover gewinnt) im `TableLayout`; UI-02: schmalere Mitarbeiter-Spalte (`1.5fr` → `200px`) konsistent an allen drei `/absences`-grid-cols; SSR-Tests + WASM-Gate.

- [x] **Phase 22: Mitarbeiter-Statistik HR** (Backend + Frontend) — HR-only pro-SalesPerson Statistik in `/employees/:id`; Kennzahl Ø gearbeitete Stunden/Woche (urlaubsbereinigt). Setzt Todo `AVG-01` um. ✅ 2026-06-26
  Code: `ReportingService` (neue Methode + REST) + `shifty-dioxus/src/component/employee_view.rs`. Berechnungsregel A-22-1 in `22-CONTEXT.md` gepinnt (Jahr bis heute; worked = shiftplan+extrawork+volunteer; voll-abwesende Wochen raus; alle vier Abwesenheitskategorien).
  Requirements: STAT-01, STAT-02
  Success Criteria:
  1. Eine pro-SalesPerson Statistik-Ansicht ist ausschließlich mit HR-Rolle zugänglich/sichtbar.
  2. Die Ansicht zeigt die durchschnittlich gearbeiteten Stunden pro Woche, mit aus dem Nenner herausgerechneten Abwesenheitszeiträumen (Definition gemäß A-22-1).
  3. Backend-Berechnung + REST + Frontend getestet; `cargo test --workspace` + WASM-Build grün.
  Plans: 2 plans (Wave 1 BE → Wave 2 FE)
  - [x] 22-01-PLAN.md — Backend: `EmployeeWeeklyStatistics` + reine A-22-1-Formel (`average_worked_hours_per_week` über `by_week`), neue HR-gated `ReportingService`-Methode (Jahr bis heute via ClockService, baut auf `get_report_for_employee`), `EmployeeWeeklyStatisticsTO` (ToSchema) + HR-gated REST-Endpoint `GET /report/{id}/weekly-statistics` (+ ReportApiDoc), Unit-Tests (voll-abwesend raus / Teilwoche drin / flexibler Vertrag / Ehrenamt zählt).
  - [x] 22-02-PLAN.md — Frontend: `get_employee_weekly_statistics`-Fetch, `EmployeeStore.weekly_statistics`-Wiring (Err/403 → None), HR-only Block in `EmployeeView` (is_hr-Gating via `has_privilege("hr")` + `should_show_hr_stats`), i18n De/En/Cs, SSR-Tests (sichtbar mit HR / unsichtbar ohne) + WASM-Gate.

- [ ] **Phase 23: Frontend: Slot Paid-Capacity UI** (Frontend) — Capacity-Editor in den Slot-Settings (`max_paid_employees` setzen, NULL = kein Limit) + Warn-Farbe im Schichtplan-Week-View, wenn `current_paid_count > max_paid_employees`. Reines Frontend-Feature in `shifty-dioxus`; das Backend liefert die Felder bereits.

  **Goal:** [To be planned]
  **Requirements:** TBD
  **Depends on:** —  (Backend bereits fertig: v1.1 / Phase 5 „Slot Paid Capacity Warning")
  **Backend-Vorbedingungen (bereits vorhanden):** `slot.max_paid_employees: Option<u8>` + `current_paid_count` werden über REST/DTO geliefert (`SlotTO`, `ShiftplanSlotTO`, `WarningTO`); nicht-blockierende `Warning::PaidEmployeeLimitExceeded`. Buchen bleibt erlaubt, NULL = kein Limit.
  **Plans:** 0 plans
  Plans:
  - [ ] TBD (run /gsd-plan-phase 23 to break down)

**Abhängigkeiten:** Phase 18 (BE) ist unabhängig. 19/20/21 (FE) sind unabhängig voneinander. 22 baut konzeptionell auf der Reporting-Ecke (18) auf, ist aber separat planbar. 23 (FE) ist unabhängig — die nötigen Backend-Felder existieren seit v1.1.

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
| 23 — Frontend: Slot Paid-Capacity UI (FE) | v1.5 | 0/0 | Not planned | — |

---

*Last updated: 2026-06-26 — **Phase 23 hinzugefügt** (Frontend: Slot Paid-Capacity UI — Capacity-Editor + Warn-Farbe; Backend aus v1.1/Phase 5 bereits vorhanden). Nächster Schritt: `/gsd-discuss-phase 23` (UI-Phase) oder `/gsd-ui-phase 23`.*
