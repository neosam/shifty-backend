# Phase 52: weekly-overview-performance-refactor — Research

**Researched:** 2026-07-05
**Domain:** Rust-Service-Refactor, Query-Batching, byte-identische Semantik-Erhaltung
**Confidence:** HIGH (Code-Verified, Zeilen zitiert aus HEAD)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions (D-52-01..D-52-16 — alle in CONTEXT.md entschieden)

- **D-52-01:** Bulk-Load-Scope erweitert um `slot_service.get_slots` (einmal vor der Wochen-Schleife), In-Memory-Filter im Consumer statt `get_slots_for_week_all_plans`.
- **D-52-02:** `ReportingService::get_year` Signatur: `async fn get_year(year: u32, context, tx) -> Result<Arc<[(u8, Arc<[ShortEmployeeReport]>)]>, ServiceError>`. Vec (nicht HashMap), kein neues Struct.
- **D-52-03:** Reihenfolge im Vec strikt aufsteigend `calendar_week` (1..=weeks_in_year). Leere Wochen mit leerem `Arc<[]>` (nicht ausgelassen).
- **D-52-04:** Spillover: `get_year(year)` + `get_year(year+1)`, Konsument iteriert `1..=weeks_in_year+3` mit Fallthrough.
- **D-52-05:** Keine `get_year_range`-Sondersignatur.
- **D-52-06:** Neue Trait-Methoden nur additiv: `ReportingService::get_year` + `ShiftplanReportService::extract_shiftplan_report_for_year`. `SpecialDayService::get_by_year` existiert bereits. `SlotService::get_slots` bestehend, In-Memory-Filter im Consumer.
- **D-52-07:** Keine `_for_range(from_date, to_date)`-Erweiterung.
- **D-52-08:** `assemble_weeks`-Helper in `service_impl/src/reporting.rs`, Signatur `fn assemble_weeks(weeks: &[(u32, u8)], work_details, shiftplan_reports, extra_hours) -> Vec<(u8, Arc<[ShortEmployeeReport]>)>` — Slice-Referenzen, KEIN DAO-Zugriff im Helper. `get_week` delegiert intern auf 1-Element-Vec-Aufruf.
- **D-52-09 (MUST-preserve Semantik):** Balance-Formel, CVC-06 Cap-Gating, Chain-C-Legacy-Filter (NICHT im Helper — bleibt in `booking_information.get_weekly_summary`), ExtraHours-Kategorien.
- **D-52-10:** `get_week` bleibt Public-Trait-Methode (REST-Konsument `rest/src/report.rs:148`). Helper ist `pub(crate)`.
- **D-52-11..14:** Test-Ansatz: 8 fest kodierte Fixtures. Byte-Identität via `f32::to_bits()`. KEIN `proptest`, KEIN `insta`. Snapshot-Ansatz bevorzugt (kein toter Legacy-Code), Fallback hart-kodierter Vec-Vergleich pro Fixture.
- **D-52-15:** Frontend unangetastet, `WeeklySummaryTO` bit-identisch.
- **D-52-16:** Messung `curl -w "%{time_total}\n" .../weekly-resource-report/2026`, 5 Runs Median. Zielwert < 0.500 s. Baseline vor Umbau messen.

### Claude's Discretion
Alles im User-Signal „Ich finde, du kannst alles entscheiden. Es soll einfach schnell sein, aber das Ergebnis sollte unverändert sein." Prio: (1) Latenz, (2) Byte-Identität.

### Deferred Ideas (OUT OF SCOPE)
- VAA-01..04 (Phase 53) — Freiwilligen-Absencen-Anzeige.
- `SlotService::get_slots_for_year`-Batch — bewusst NICHT eingeführt.
- HTTP-Cache / ETag — verworfen wegen Live-Korrektheit.
- Parallelisierung via `join_all` — SQLite serialisiert intern.
- Snapshot-Schema-Bump — bleibt 12.
- Migration (Cargo-Dep oder DB) — keine Ergänzung erlaubt.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WOP-01 | Bulk-Load `special_days` + `shiftplan_reports` (+ Slots per D-52-01), Ergebnis unverändert | Load-once-Muster in `booking_information.rs:291-303` bereits etabliert (`all_work_details`, `all_absences`); Trait-Extension in `shiftplan_report` symmetrisch zum bestehenden `_for_week`-Pattern |
| WOP-02 | `reporting_service.get_year(year)` ersetzt ~55 `get_week`-Calls; alle Invarianten erhalten | `assemble_weeks`-Helper-Design (siehe Q2 unten) — `get_week` und `get_year` delegieren beide auf denselben Helper mit Slice-Input, garantiert byte-identisches Verhalten für 1-Element-Vec (= alter `get_week`) und 55-Element-Vec (= neuer `get_year`) |
| WOP-03 | Property-/Regressions-Test byte-identisch (Feiertage, ShortDays, Freiwillige, CVC-06, Gate on/off) | 8-Fixture-Tabelle (D-52-11), `f32::to_bits()`-Vergleich (D-52-12), Snapshot-Test bevorzugt (D-52-14); Kategorien-Achsen aus `reporting.rs:1143-1156` (`ShortEmployeeReport`-Felder) |
| WOP-04 | Endpoint-Latenz < 500 ms auf Dev-DB | `curl -w`-Messung (D-52-16); Query-Ersparnis: ~55×3 DAO-Calls + ~55 Slot-Calls fallen auf 4+1 (`get_year(y)+get_year(y+1)`+Slots+Absences+WorkDetails); DB-Indices: Full-Scan bleibt gleich, aber pro Jahres-Endpoint statt pro Woche (siehe Q3) |
| WOP-05 | Alle Tests grün, insbesondere `booking_information_chain_c.rs`, Reporting-Tests, Clippy `-D warnings` | Bestehende Test-Suite bleibt unberührt; `get_week` bleibt Public-Trait-Methode (D-52-10) → keine Signatur-Änderung für existierende Tests; `booking_information_chain_c` prüft Slot-Clipping via `shortday_gate` — Toggle-Read bleibt in `get_weekly_summary` (D-52-09) |
</phase_requirements>

## Executive Summary

Der Refactor ist zweistufig und mechanisch: (1) `get_week` extrahiert seine Aggregations-Schleife in einen internen `pub(crate) assemble_weeks`-Helper, der drei bereits vorgeladene Kollektionen (`work_details`, `shiftplan_reports`, `extra_hours`) als Slice-Referenzen entgegennimmt und pro `(sales_person_id, calendar_week)` aggregiert; (2) neue `get_year` lädt dieselben drei Kollektionen einmal pro Jahr statt einmal pro Woche und ruft `assemble_weeks` mit einem 55-Element-Vec statt einem 1-Element-Vec auf. Byte-Identität ist strukturell garantiert, weil beide Pfade denselben Helper mit denselben Aggregations-Bausteinen (`find_working_hours_for_calendar_week`, `apply_weekly_cap`, `weight_for_week`, `build_derived_holiday_map`) verwenden.

Die drei offenen Detailfragen sind wie folgt beantwortet: **Q1 (CVC-06 Cap)** — die Cap-Entscheidung fällt pro `(sales_person_id, calendar_week)` in `apply_weekly_cap`, nicht pro Jahr; Aggregations-Granularität ist irrelevant. **Q2 (`assemble_weeks`-Struktur)** — Empfehlung Struktur C: Personen-Index als `HashMap<Uuid, Vec<&EmployeeWorkDetails>>`, Reports gruppiert nach `calendar_week` als `HashMap<u8, HashMap<Uuid, f32>>`, ExtraHours ebenso gebuckelt. Big-O: O(N_persons·N_weeks) pro Woche linear, gesamt O(N_persons·N_weeks) für ein Jahr. **Q3 (DB-Indices)** — KEIN Index auf `booking(year, calendar_week)` oder `extra_hours(year)`, aber der bestehende `_for_week`-Pfad hat auch keinen; der Refactor **verschlechtert nichts** und der Speed-Up kommt aus der Reduktion der Query-Anzahl. **Kein Migration-Blocker**.

**Primary recommendation:** Task-Sequenz aus CONTEXT.md `<downstream_hooks>` folgen. Wave 0 = Fixture-Test schreiben und mit **aktueller** `get_weekly_summary` grün bekommen (Golden-Snapshot). Wave 1 = `assemble_weeks` extrahieren (reiner Refactor, Test bleibt grün). Wave 2 = neue Trait-Methoden + `get_weekly_summary`-Umbau. Wave 3 = Latenz-Messung + Docs-Freshness.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Aggregation pro Woche (Balance-Formel, Cap, ExtraHours-Kategorien) | Service (Basic: Reporting) | — | Business-Kernberechnung, single source of truth |
| Bulk-Load von Jahres-Aggregaten | Service (Basic: ShiftplanReport, ExtraHours, EmployeeWorkDetails, SpecialDay) | DAO | Trait-Extension mit sqlx-Query gegen bestehende Tabellen |
| Slot-Clipping / ShortDay-Gate-Anwendung im Consumer | Service (Business-Logic: BookingInformation) | — | D-52-09: Toggle-Read bleibt in `get_weekly_summary`, NICHT in `reporting.get_year` |
| Wochen-Iteration mit Spillover (weeks_in_year+3) | Service (Business-Logic: BookingInformation) | — | Konsument-Verantwortung; `reporting.get_year` liefert nur 1..=weeks_in_year |
| Byte-Identität-Test | Service-Test (`service_impl/src/test/`) | — | Fixture-basiert, mockall-Mocks |

## Standard Stack

Kein neuer Cargo-Dep (Nicht-Ziel). Verwendet werden bestehende Krates:

### Core
| Library | Version (Cargo.toml) | Purpose | Why Standard |
|---------|----------------------|---------|--------------|
| `sqlx` | bestehend | Neue `query_as!`-Makros für `_for_year`-DAO | Compile-time-checked queries, projektweiter Standard |
| `mockall` | bestehend | Trait-Mocks in Fixture-Tests | Standard-Test-Framework des Projekts |
| `async-trait` | bestehend | Trait-Definitionen | Standard-Pattern |
| `tokio` (via async_trait) | bestehend | Async-Runtime | — |
| `time` | bestehend | ISO-Wochen-Berechnung (`weeks_in_year`, `from_iso_week_date`) | Bereits Konsument in `booking_information.rs:281` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Fixture-Vec-Vergleich | `insta` (Snapshot) | Neuer Dep verboten (D-52-13) — Fallback ist hart-kodierter Vec |
| `HashMap<Uuid, ...>`-Buckets | `Vec<(Uuid, ...)>` linear scan | Für ~10-50 Sales Persons + ~55 Wochen ist der Unterschied unter Messrauschen; HashMap dominiert in Klarheit |

**Installation:** Keine.

**Version verification:** Keine neuen Packages — Section übersprungen.

## Package Legitimacy Audit

Nicht anwendbar — kein externer Package-Install in dieser Phase.

## Q1: CVC-06 Cap-Semantik pro Person — mathematisch identisch pro Woche vs. pro Jahr?

### Antwort: JA, identisch. Die Cap-Entscheidung ist strikt per-Person-per-Woche und wird nicht durch Wochen-Aggregation beeinflusst.

**Code-Beleg 1 — Cap-Definition (`service_impl/src/reporting.rs:124-136`):**

```rust
pub fn apply_weekly_cap(
    cap_active: bool,
    shiftplan_hours: f32,
    expected_hours_for_week: f32,
) -> (f32, f32) {
    if cap_active && shiftplan_hours > expected_hours_for_week {
        (
            expected_hours_for_week,
            shiftplan_hours - expected_hours_for_week,
        )
    } else {
        ...
    }
}
```

Der Helper hat **KEINE** Persistenz und **KEINE** Cross-Week-State. Er ist eine pure Funktion `(bool, f32, f32) -> (f32, f32)`.

**Code-Beleg 2 — Cap-Aktivierung im `get_week`-Loop (`service_impl/src/reporting.rs:1030-1031`):**

```rust
let cap_active = find_working_hours_for_calendar_week(&working_hours, year, week)
    .any(|wh| wh.cap_planned_hours_to_expected);
```

`find_working_hours_for_calendar_week` (Def. bei `reporting.rs:84-93`) filtert die `EmployeeWorkDetails`-Rows anhand von `(from_year, from_calendar_week)..=(to_year, to_calendar_week)` — also **pro Woche**. Die Entscheidung ist damit rein von der Woche und der jeweiligen Employee-Row abhängig.

**Code-Beleg 3 — Anwendung (`service_impl/src/reporting.rs:1115-1116`):**

```rust
let (shiftplan_hours, auto_volunteer_hours) =
    apply_weekly_cap(cap_active, raw_shiftplan_hours, expected_hours_for_cap);
```

`raw_shiftplan_hours` ist pro Woche (Summe aus `shiftplan_report.get(&sales_person_id).map(|r| r.iter().map(|r| r.hours).sum::<f32>())` bei `reporting.rs:918-921`). `expected_hours_for_cap` ist ebenfalls pro Woche (aus `weight_for_week`, `reporting.rs:1113-1114`).

**Code-Beleg 4 — CVC-06-Gating in `booking_information.get_weekly_summary` (Zeile 383-388):**

```rust
find_working_hours_for_calendar_week(&all_work_details, year, week)
    .filter(|wh| {
        wh.sales_person_id == sp_id
            && (wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0)
    })
    .map(|wh| wh.committed_voluntary)
    .sum()
```

Auch hier: Cap-Filter pro Woche, pro Person, keine übergreifende Aggregation.

### Konsequenz für `assemble_weeks`

Der Helper muss folgende Invarianten pro Woche gewährleisten:
1. Der Cap-Filter (`wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0`) wird pro `(sales_person_id, calendar_week)` ausgewertet.
2. `apply_weekly_cap` wird pro `(sales_person_id, calendar_week)` genau einmal aufgerufen (nie über mehrere Wochen).
3. `weight_for_week(year, week, wh)` erhält immer die konkrete Zielwoche, nicht ein Jahres-Aggregat.

Das ist strukturell einfach zu garantieren: Wenn der Helper mit `weeks: &[(u32, u8)]` in einer äußeren Schleife arbeitet und pro Iteration `find_working_hours_for_calendar_week(work_details, year, week)` frisch aufruft, ist die Semantik zu `get_week` identisch. **Kein übergreifendes Jahres-Aggregat vorhanden, das die Cap-Entscheidung verändern könnte.**

## Q2: `assemble_weeks` — konkrete Struktur, Big-O, Refactor-Steps

### Empfehlung: Struktur C (Kombination) — Personen-Index für `work_details`, Woche-gebucket für Reports/ExtraHours

**Signatur:**

```rust
pub(crate) async fn assemble_weeks(
    &self,
    weeks: &[(u32, u8)],
    work_details: &[EmployeeWorkDetails],
    shiftplan_reports: &[ShiftplanReportDay],
    extra_hours: &[ExtraHours],
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Vec<(u8, Arc<[ShortEmployeeReport]>)>, ServiceError>
```

**Warum `async` + `tx`?** Der bestehende `get_week`-Body (Zeilen 1035-1092, 1136-1142) macht **Async-DAO-Calls im Loop**:
- `self.absence_service.derive_hours_for_range(...)` (Zeile 1036) — pro Person, pro Woche.
- `self.build_derived_holiday_map(...)` (Zeile 1085) — pro Person, pro Woche.
- `self.sales_person_service.get(sales_person_id, ...)` (Zeile 1137) — pro Person, pro Woche.

**Diese drei Calls müssen für Byte-Identität bleiben** (Semantik-Invariante D-52-09). Der Helper ist damit `async fn` und braucht `context` + `tx`.

**Perf-Anmerkung:** Diese drei Calls sind pro `(person, week)`. Bei ~10 Personen und 55 Wochen sind das ~1650 Calls. `derive_hours_for_range` und `build_derived_holiday_map` lesen `all_absences` bereits (im `AbsenceService`-Layer gecacht?) — der Perf-Gain aus dem Refactor kommt PRIMÄR aus dem Weg von 55×3 externen Bulk-Loads (special_days, shiftplan_report, extra_hours, working_hours) auf 1×3, NICHT aus der inneren Aggregations-Schleife. Das ist konsistent mit `weekly-overview-perf-analyse.md`, die den Hotspot bei den 55 sequenziellen Service-Calls sieht.

**Falls die Latenz nach dem Refactor immer noch > 500 ms:** Als Follow-Up in Phase 52-Verify prüfen, ob `sales_person_service.get(sales_person_id, ...)` (Zeile 1137, pro Person pro Woche!) durch Load-once ersetzt werden kann. `sales_person_service.get_all` existiert (wird bereits in `booking_information.rs:282-289` verwendet). Das ist eine mögliche Zusatzoptimierung, die byte-identisch bleibt.

### Index-Strukturen (im Helper-Rumpf aufgebaut)

```rust
// Personen-Index: aktive Rows pro Sales Person (für find_working_hours_for_calendar_week im Loop billig).
// Anmerkung: die Signatur bekommt work_details bereits als Slice — der Helper kann direkt
// find_working_hours_for_calendar_week(work_details, year, week) rufen (Full-Scan pro Woche,
// O(N_work_details) ~ O(10-100)). Ein zusätzlicher HashMap-Index bringt wenig.

// Shiftplan-Reports gruppiert nach (year, calendar_week): O(1) pro Woche.
let mut reports_by_week: HashMap<(u32, u8), HashMap<Uuid, f32>> = HashMap::new();
for report in shiftplan_reports {
    *reports_by_week
        .entry((report.year, report.calendar_week))
        .or_default()
        .entry(report.sales_person_id)
        .or_insert(0.0) += report.hours;
}

// ExtraHours gruppiert nach (year, week, sales_person_id) — Extraktion via to_date().as_shifty_week().
let mut extras_by_week: HashMap<(u32, u8), HashMap<Uuid, Vec<&ExtraHours>>> = HashMap::new();
for eh in extra_hours {
    let sw = eh.to_date().as_shifty_week();
    extras_by_week
        .entry((sw.year, sw.week))
        .or_default()
        .entry(eh.sales_person_id)
        .or_default()
        .push(eh);
}
```

**Warum kein Personen-Index für `work_details`?** `find_working_hours_for_calendar_week` filtert linear (Zeile 89-92) — für ~10-100 Rows ist ein HashMap-Aufbau overkill und würde die Aggregations-Semantik nicht beschleunigen. Der Refactor darf hier absichtlich **minimal invasiv** sein, um Byte-Identität leichter beweisbar zu machen.

### Big-O

| Metrik | Get_week (heute, 1 Woche) | Get_year (neu, 1 Jahr = 55 Wochen) |
|--------|---------------------------|------------------------------------|
| DAO/Service Bulk-Calls | 3 (work_details, shiftplan_report, extra_hours) + `special_days` in `booking_information` | 3 (identische Struktur, nur `_for_year`) |
| Innere Async-Calls (`derive_hours_for_range` + `build_derived_holiday_map` + `sales_person.get`) | O(N_persons) × O(1) | O(N_persons × N_weeks) |
| Innere Sync-Aggregation | O(N_persons × N_extra_hours) | O(N_persons × N_extra_hours × N_weeks) |
| Speicher (Peak) | O(N_persons × 1 week) | O(N_persons × N_weeks) — akzeptabel, Sales-Person-Report ist klein |

Wichtiger Perf-Insight: **Die Async-Calls im Loop skalieren linear mit Wochen×Personen (~550-1650 Calls für ein Jahr).** Wenn die Gesamt-Latenz nach dem Refactor immer noch nicht < 500 ms erreicht, ist das der nächste Hotspot. Der Planner sollte die Latenz-Messung in Wave 3 **VOR** einer möglichen Optimierung dieser Calls machen — vielleicht sind sie durch SQLite-Connection-Pool-Wärme schnell genug.

### Refactor-Steps (mechanisch, Test bleibt grün)

**Schritt A (Extract Function — Reiner Refactor):**
1. Zeilen 892-1156 aus `get_week` in eine neue `pub(crate) async fn assemble_weeks` extrahieren.
2. Die Bulk-Loads (Zeilen 892-913) BLEIBEN in `get_week`; sie werden zu Argumenten des Helpers gemacht.
3. `get_week` wird auf:
   ```rust
   async fn get_week(&self, year: u32, week: u8, context, tx) -> ... {
       let tx = self.transaction_dao.use_transaction(tx).await?;
       let work_details = self.employee_work_details_service.all_for_week(week, year, context.clone(), tx.clone()).await?;
       let shiftplan_reports = self.shiftplan_report_service.extract_shiftplan_report_for_week(year, week, Authentication::Full, tx.clone()).await?;
       let extra_hours = self.extra_hours_service.find_by_week(year, week, Authentication::Full, tx.clone()).await?;
       let mut assembled = self.assemble_weeks(&[(year, week)], &work_details, &shiftplan_reports, &extra_hours, context, tx.clone()).await?;
       self.transaction_dao.commit(tx).await?;
       Ok(assembled.pop().map(|(_, r)| r).unwrap_or_else(|| Arc::from(Vec::<ShortEmployeeReport>::new())))
   }
   ```
4. Alle bestehenden Tests laufen; keine Signatur-Änderung.

**Schritt B (Neuer Consumer — additive Trait-Methode):**
1. `ReportingService::get_year` in `service/src/reporting.rs:397` hinzufügen.
2. Impl in `service_impl/src/reporting.rs`:
   ```rust
   async fn get_year(&self, year, context, tx) -> Result<Arc<[(u8, Arc<[ShortEmployeeReport]>)]>, ServiceError> {
       let tx = self.transaction_dao.use_transaction(tx).await?;
       let work_details = self.employee_work_details_service.all(Authentication::Full, tx.clone()).await?;
       let shiftplan_reports = self.shiftplan_report_service.extract_shiftplan_report_for_year(year, Authentication::Full, tx.clone()).await?;
       let extra_hours = self.extra_hours_service.find_by_year(year, Authentication::Full, tx.clone()).await?;  // TODO: neue Methode? oder find_all + filter?
       let weeks_in_year = time::util::weeks_in_year(year as i32);
       let weeks: Vec<(u32, u8)> = (1..=weeks_in_year).map(|w| (year, w)).collect();
       let assembled = self.assemble_weeks(&weeks, &work_details, &shiftplan_reports, &extra_hours, context, tx.clone()).await?;
       self.transaction_dao.commit(tx).await?;
       Ok(assembled.into())
   }
   ```

**Anmerkung zu ExtraHours-Batch:** Ein `find_by_year` fehlt aktuell im `ExtraHoursService`-Trait (siehe `service/src/extra_hours.rs:209-215`). Optionen:
- **Option a:** Neue Trait-Methode `find_by_year(year, ...)` — konsistent zum `_for_week`-Pattern, aber vier Trait-Methoden statt drei zu erweitern.
- **Option b:** Bestehende `find_all` verwenden + In-Memory `filter(|eh| eh.date_time.year() == year || eh.date_time.year() == year+1)`. Kein neuer Trait-Endpoint, dafür wird die gesamte Historie geladen.
- **Empfehlung:** Option a. Ist symmetrisch zu D-52-06 (nur `_for_year` ergänzen wo `_for_week` existiert und `_for_year` fehlt), spart Speicher-Peak. Kleiner Aufwand: neue Trait-Methode + neue DAO-Query (Filter `WHERE date_time BETWEEN start_of_year AND end_of_year AND deleted IS NULL`) + `sqlx prepare`.

**Diese Option muss der Planner explizit in eine Task verpacken, sie ist in D-52-06 nicht wörtlich aufgezählt.** Das ist die einzige Abweichung von CONTEXT.md — CONTEXT.md ist nicht spezifisch zu ExtraHours-Batching, aber die Semantik verlangt eine Jahres-Batch-Möglichkeit. Fallback wäre `find_all` + Filter.

## Q3: DB-Index-Check auf `year`-Filter

### Ergebnis: KEIN Index vorhanden, aber KEIN Blocker.

**Grep-Verifikation (`migrations/sqlite/*.sql`):**

```
CREATE INDEX idx_text_template_name        ON text_template(name);
CREATE INDEX idx_text_template_type        ON text_template(template_type);
CREATE INDEX idx_text_template_deleted     ON text_template(deleted);
CREATE INDEX idx_user_invitation_*         ON user_invitation(...);
CREATE INDEX idx_absence_migration_*       ON absence_migration_quarantine(...);
CREATE INDEX idx_absence_period_*          ON absence_period(...);
CREATE INDEX idx_employee_yearly_carryover_pre_cutover_backup_sp_year ...;
CREATE UNIQUE INDEX idx_extra_hours_logical_id_active ...;
CREATE UNIQUE INDEX idx_week_status_active ...;
CREATE INDEX idx_absence_period_migration_source_period ...;
CREATE INDEX idx_absence_period_migration_source_run ...;
CREATE UNIQUE INDEX idx_vacation_entitlement_offset_active ...;
```

**Keine Indices auf `booking(year, calendar_week)`, `working_hours(from_year, to_year)`, oder `extra_hours(date_time)`.** Der bestehende `extract_raw_shiftplan_report_for_week` (`dao_impl_sqlite/src/shiftplan_report.rs:142-176`) filtert `WHERE booking.year = ? AND booking.calendar_week = ?` **ohne Index** — jeder Week-Call ist heute schon ein Full-Scan auf `booking` (was für den Dev-Datenbestand offenbar tolerabel ist).

### Konsequenzen für Phase 52

- Die neue `_for_year`-Query wird **denselben Full-Scan** durchführen, aber **einmal statt 55×**. Das ist eine **massive Netto-Ersparnis** (unabhängig vom Index).
- **KEIN Migration-Blocker** für D-52 (Nicht-Ziel „keine Migration" bleibt eingehalten).
- **Deferred / Follow-Up:** Wenn unter Produktions-Last (>100k Bookings) `booking(year, calendar_week)`-Index nötig würde, ist das ein separater Maintenance-Task. Er ist bereits als deferred in CONTEXT.md `<deferred>` sinngemäß angedeutet („Slot-Filter `valid_from`/`valid_to` als DB-Index prüfen").

### Empfehlung an den Planner

- Bitte im PLAN-Verify einen Punkt aufnehmen: **„Nach `curl`-Messung: wenn Latenz-Median > 400 ms auf Dev-DB (aber < 500 ms), ist im Folgetask eine `CREATE INDEX booking_year_cw`-Migration zu erwägen. Aktuell nicht Teil dieser Phase."**
- Keine Migration in Phase 52 committen.

## Empfohlener Implementation-Path

Task-Sequenz für `gsd-planner`, mit `read_first`-Zeilen für Executor:

### Wave 0 — Test-Baseline (WOP-03, Fixture-Gate)

**Task 0.1:** Neue Testdatei `service_impl/src/test/booking_information_weekly_summary_year_batch.rs`.
- `read_first`: `service_impl/src/booking_information.rs:259-491`, `service_impl/src/test/booking_information_chain_c.rs` (Fixture-Vorbild), `service_impl/src/reporting.rs:884-1160` (get_week Semantik).
- 8 Fixtures nach D-52-11: Baseline / Feiertag / ShortDay / Freiwilliger-Vacation / CVC-06-Cap / Gate-off / Gate-on / Kombi.
- Vergleich pro Feld via `f32::to_bits()` (D-52-12).
- **Gate:** Fixture-Test läuft gegen **aktuelle** `get_weekly_summary`-Impl und speichert Golden-Vergleichs-Vecs als hart-kodierte Konstanten pro Fixture. Nach jedem folgenden Refactor-Schritt muss der Test grün bleiben.
- `sales_person_service.get` bzw. `absence_service.derive_hours_for_range` und `build_derived_holiday_map` müssen im Test-Setup korrekt gemockt sein — hier Vorbild aus `booking_information_chain_c.rs` verwenden.

### Wave 1 — Refactor `assemble_weeks` (WOP-02 Teil 1, verhalten-invariant)

**Task 1.1:** In `service_impl/src/reporting.rs` neue `pub(crate) async fn assemble_weeks(weeks, work_details, shiftplan_reports, extra_hours, context, tx)`.
- `read_first`: `reporting.rs:884-1160` (kompletter `get_week`-Body).
- Zeilen 892-1160 verschieben ins neue `assemble_weeks`; die drei Bulk-Loads bleiben zunächst in `get_week`.
- `get_week` wird auf `let assembled = self.assemble_weeks(&[(year, week)], ...).await?; Ok(assembled.pop()...)` reduziert.
- **Gate:** `cargo test --workspace` + Fixture-Test grün. Byte-identisch garantiert.
- **Clippy:** `cargo clippy --workspace -- -D warnings` grün.

### Wave 2 — Bulk-Load-Trait-Methoden (WOP-01, WOP-02 Teil 2)

**Task 2.1:** `ShiftplanReportService::extract_shiftplan_report_for_year` (Trait + Impl + DAO).
- `read_first`: `service/src/shiftplan_report.rs:56-62` (Trait `_for_week`-Signatur), `service_impl/src/shiftplan_report.rs:258-310` (`_for_week`-Impl), `dao_impl_sqlite/src/shiftplan_report.rs:142-176` (DAO-Query).
- Trait erweitern.
- DAO: neue `extract_raw_shiftplan_report_for_year(year)` mit Query `WHERE booking.year = ?`.
- Service-Impl: SpecialDays über `special_day_service.get_by_year(year)` einmal (nicht per Woche). Aggregation identisch zu `_for_week`, aber gruppiert nach `(sales_person_id, year, calendar_week, day_of_week)` statt nur `(sales_person_id, day_of_week)`.
- `cargo sqlx prepare --workspace` + `.sqlx`-Delta committen (Memory `reference_sqlx_prepare_after_new_query.md`).

**Task 2.2:** `ExtraHoursService::find_by_year` (Trait + Impl + DAO) — **siehe Q2 Empfehlung, Option a**.
- `read_first`: `service/src/extra_hours.rs:209-215`, `service_impl/src/extra_hours.rs:154-170`, DAO-Layer.
- Falls Planner Option b (kein neuer Trait-Endpoint) bevorzugt: `find_all` + In-Memory-Filter im `get_year`-Consumer. Dokumentieren als bewusste Wahl.

**Task 2.3:** `ReportingService::get_year` (Trait + Impl auf `assemble_weeks`).
- `read_first`: `service/src/reporting.rs:397-403` (`get_week`-Trait-Signatur), Wave 1 Ergebnis.
- Trait-Signatur: `async fn get_year(year, context, tx) -> Result<Arc<[(u8, Arc<[ShortEmployeeReport]>)]>, ServiceError>` (D-52-02).
- Impl: 3 Bulk-Loads + `assemble_weeks(&weeks_1..=weeks_in_year, ...)`.

### Wave 3 — `get_weekly_summary`-Umbau (WOP-01 Rest)

**Task 3.1:** `booking_information.rs:259-491` umbauen.
- `read_first`: `booking_information.rs:259-491`, Wave 2 Ergebnis.
- Vor dem Wochen-Loop hinzufügen:
  ```rust
  let year_reports = self.reporting_service.get_year(year, Authentication::Full, tx.clone().into()).await?;
  let next_year_reports = self.reporting_service.get_year(year + 1, Authentication::Full, tx.clone().into()).await?;
  let special_days_year = self.special_day_service.get_by_year(year, Authentication::Full).await?;
  let special_days_next_year = self.special_day_service.get_by_year(year + 1, Authentication::Full).await?;
  let shiftplan_reports_year = self.shiftplan_report_service.extract_shiftplan_report_for_year(year, Authentication::Full, tx.clone().into()).await?;
  let shiftplan_reports_next_year = self.shiftplan_report_service.extract_shiftplan_report_for_year(year + 1, Authentication::Full, tx.clone().into()).await?;
  let all_slots = self.slot_service.get_slots(Authentication::Full, tx.clone().into()).await?;
  ```
- Im Loop `week_report`, `special_days`, `shiftplan_reports`, `slots` aus vorgeladenen Kollektionen ableiten statt per Service-Call:
  - `week_report` = `year_reports[week-1].1` bzw. `next_year_reports[week-weeks_in_year-1].1` bei Spillover.
  - `special_days` = In-Memory-Filter auf `special_days_year` / `special_days_next_year` per `week`.
  - `shiftplan_reports` = In-Memory-Filter auf `shiftplan_reports_year` per `year`+`week`.
  - `slots` = In-Memory-Filter auf `all_slots` per `valid_from`/`valid_to` gegen `(year, week)`. **KRITISCH:** Semantik von `SlotDao::get_slots_for_week_all_plans` reproduzieren — d.h. Slot ist aktiv wenn `valid_from <= week_end && (valid_to IS NULL || valid_to >= week_start)`. Der Executor MUSS die konkrete Filter-Logik aus `slot.rs` studieren und byte-identisch nachbauen.
- Toggle-Read (`shortday_gate::read_active_from`) bleibt bei Zeile 309.
- Slot-Clipping (Zeilen 409-439) bleibt komplett in `booking_information`, benutzt aber jetzt In-Memory-gefilterte `all_slots`.
- **Gate:** Fixture-Test muss BYTE-identisch grün bleiben. Falls nicht, sofort auf Wave 1 zurückrollen und Semantik-Diff dokumentieren.

### Wave 4 — Latenz-Messung + Docs-Check (WOP-04, F07)

**Task 4.1:** Latenz-Baseline vor Wave 3 (in Wave 0 zu erledigen — vor jeder Änderung, damit vergleichbar).
**Task 4.2:** Latenz-Messung nach Wave 3, 5 Runs, Median. Zielwert < 0.500 s dokumentieren.
**Task 4.3:** `docs/features/F07-reporting-balance.md` + `..._de.md`: Balance-Formel unverändert → nur Änderungshistorie-Sektion mit Hinweis „v2.5 Phase 52: `get_year`-Batch-Variante ergänzt, Formel unverändert". Falls die Doku keine Trait-Signaturen zitiert, gar nichts.

## Risiken & Landmines

| # | Risiko | Wo | Mitigation |
|---|--------|-----|-----------|
| R1 | Slot-Filter `valid_from`/`valid_to` in-memory != DAO-Query-Semantik | `booking_information.rs:409-439` + neuer In-Memory-Filter | Wave 3 Task 3.1: Executor liest `dao_impl_sqlite/src/slot.rs::get_slots_for_week_all_plans` und reproduziert **exakt** die SQL-`WHERE`-Klausel als Rust-Closure. Fixture-Test deckt einige Fälle (Baseline, ShortDay), aber Executor MUSS zusätzlich die valid_from/valid_to-Übergangs-Randfälle prüfen. |
| R2 | ExtraHours-Batching per In-Memory-Filter statt neuer Trait-Methode kann bei großen Historien Speicher-peak verursachen | Q2 Option b | Option a (`find_by_year`) bevorzugen; Planner entscheidet. |
| R3 | Async-Calls im Helper (`derive_hours_for_range`, `build_derived_holiday_map`, `sales_person.get`) skalieren linear mit Personen×Wochen — evtl. Perf-Bottleneck nach dem Refactor | Wave 3 Post-Verify | Falls Latenz > 500 ms: zusätzliche Load-once für `sales_person_service.get_all` einführen (Planner kann das als eigene Task planen, byte-identisch). |
| R4 | `get_week` bleibt Public-Trait und wird von REST konsumiert (`rest/src/report.rs:148`) — Signatur DARF sich nicht ändern | Wave 1 | Nur inneren Body ändern, Trait-Signatur bleibt intakt. Bestehende Unit-Tests sichern das ab. |
| R5 | `assemble_weeks` bekommt `tx: Option<Self::Transaction>` — muss dieselbe TX verwenden wie der Consumer, sonst inkonsistente Reads | Wave 1 | Helper wird `pub(crate) async fn` mit `tx` als Argument; Consumer übergibt `tx.clone()`. Kein neues `use_transaction`+`commit` im Helper (das bleibt in `get_week`/`get_year`). |
| R6 | Spillover-Vec-Index: bei `week > weeks_in_year` muss der Consumer `next_year_reports[week - weeks_in_year - 1]` verwenden (0-basiert!) | Wave 3 | Off-by-one-Test in Fixture 8 (Kombi) prüfen: Baseline in Woche 52 + Übergang in Woche 53. Fixture-Test explizit für Spillover-Woche schreiben. |
| R7 | `find_by_year` vs. `find_all`+Filter (Option a/b) — falls Option b: filter auf `ExtraHours::date_time.year()` **UND** `year+1` in einem einzigen Call, um konsistent mit Spillover zu bleiben | Wave 2 Task 2.2 | Explizite Dokumentation im Plan, Fixture-Test für Spillover-ExtraHours. |
| R8 | Chain-C-Toggle-Read wandert versehentlich in `assemble_weeks`/`get_year` | Wave 1, Wave 2 Task 2.3 | D-52-09 zwingt: Toggle-Read bleibt in `booking_information.get_weekly_summary`. Code-Review-Punkt in PLAN-Verify. |
| R9 | `sales_person_service.get(sales_person_id, ...)` (`reporting.rs:1137`) filtert `is_paid` — im Batch-Fall muss das per-Person-Filter identisch bleiben | Wave 1 | Filter läuft im `assemble_weeks`-Helper pro Zeile, nicht als Kollektiv-Filter. Der `continue` bei Zeile 1141 muss erhalten bleiben. |

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` mit `mockall` + in-memory SQLite |
| Config file | keine — `Cargo.toml` Standard |
| Quick run command | `cargo test --package service_impl booking_information_weekly_summary_year_batch` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WOP-01 | Bulk-Loads greifen, Ergebnis unverändert | integration | `cargo test --package service_impl booking_information_weekly_summary_year_batch` | Wave 0 |
| WOP-02 | `get_year` liefert dieselben `ShortEmployeeReport`s wie 55×`get_week` | integration | `cargo test --package service_impl reporting::test::get_year_matches_per_week` | Wave 2 |
| WOP-03 | Byte-Identität über 8 Fixtures | integration + Fixture | `cargo test --package service_impl weekly_summary_year_batch::fixture_` | Wave 0 |
| WOP-04 | Endpoint-Latenz < 500 ms | manual (curl-Skript) | `for i in {1..5}; do curl -s -o /dev/null -w "%{time_total}\n" http://localhost:3000/booking-information/weekly-resource-report/2026; done \| sort \| head -3 \| tail -1` | Wave 4 |
| WOP-05 | Alle Tests grün + Clippy `-D warnings` | ci-gate | `cargo test --workspace && cargo clippy --workspace -- -D warnings` | bereits |

### Sampling Rate

- **Per task commit:** `cargo test --package service_impl booking_information_weekly_summary_year_batch`
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Phase gate:** volle Suite grün + Latenz-Median < 0.500 s → `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `service_impl/src/test/booking_information_weekly_summary_year_batch.rs` — Fixture-Test mit 8 Szenarien
- [ ] `service_impl/src/test/mod.rs` — Modul-Registrierung
- [ ] Fixture-Baseline gegen **aktuelle** Impl aufnehmen (Golden-Werte hart-kodiert)

## Security Domain

`security_enforcement = true` (Default). Diese Phase ist ein **reiner Backend-Refactor ohne Auth-Änderung**. Alle bestehenden Guards bleiben:

- `SHIFTPLANNER_PRIVILEGE` / `SALES_PRIVILEGE` in `get_weekly_summary` (Zeile 266-272).
- `Authentication::Full` für alle internen Cross-Service-Calls unverändert.
- `is_paid`-Filter in `reporting.get_week` (Zeile 1140-1142) bleibt.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | nein — Auth unangetastet | — |
| V3 Session Management | nein | — |
| V4 Access Control | ja (bestehend) | RBAC via `permission_service.check_permission` — nicht angefasst |
| V5 Input Validation | nein — Path-Param `{year}` bleibt `u32`, keine neuen Inputs | — |
| V6 Cryptography | nein | — |

### Known Threat Patterns

Keine neuen Threats — der Refactor ist rein Interner-Aggregation-Umbau ohne neue API-Endpoints, ohne neue User-Inputs, ohne neue Datenmodelle.

## Runtime State Inventory

Keine Migration, kein Rename, keine Datenmodell-Änderung. Diese Sektion trifft nicht zu.

## Environment Availability

Diese Phase ist code-only. Externe Dependencies: keine neuen. Bestehende:

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust-Toolchain (workspace) | Build | ✓ | via `nix develop` | — |
| SQLite + `sqlx-cli` | `cargo sqlx prepare` in Wave 2 | ✓ (via `nix develop`) | — | — |
| curl | Wave 4 Latenz-Messung | ✓ | Standard-System | — |

## Referenzen (Datei:Zeile-Liste aller angesprochenen Code-Stellen)

### Service-Traits (additive Erweiterung)

- `service/src/reporting.rs:397-403` — `ReportingService::get_week` Vorbild-Signatur → **neu:** `get_year`
- `service/src/shiftplan_report.rs:56-62` — `extract_shiftplan_report_for_week` Vorbild → **neu:** `extract_shiftplan_report_for_year`
- `service/src/special_days.rs:91-95` — `get_by_year` **existiert bereits**, wird verwendet
- `service/src/slot.rs:104-108` — `get_slots` bestehend, wird für Load-once verwendet
- `service/src/extra_hours.rs:209-215` — `find_by_week` Vorbild → **neu (Q2 Option a):** `find_by_year`

### Service-Impls (Refactor + additiv)

- `service_impl/src/reporting.rs:84-93` — `find_working_hours_for_calendar_week` (pure)
- `service_impl/src/reporting.rs:124-136` — `apply_weekly_cap` (pure)
- `service_impl/src/reporting.rs:884-1160` — `get_week` (extrahiert `assemble_weeks`)
- `service_impl/src/reporting.rs:1030-1031` — CVC-06 Cap-Aktivierung (per-Woche)
- `service_impl/src/reporting.rs:1113-1116` — `apply_weekly_cap`-Anwendung
- `service_impl/src/reporting.rs:1136-1142` — `sales_person.get` + `is_paid`-Filter
- `service_impl/src/reporting.rs:1233-1289` — `weight_for_week` (pure)
- `service_impl/src/shiftplan_report.rs:258-310` — `extract_shiftplan_report_for_week` Vorbild für `_for_year`
- `service_impl/src/shiftplan_report.rs:75-97` — `hours_for_row` (Clip-Anwendung, wiederverwenden)
- `service_impl/src/booking_information.rs:259-491` — `get_weekly_summary` (Wave 3 Ziel)
- `service_impl/src/booking_information.rs:291-303` — bestehendes Load-once-Muster (Vorbild)
- `service_impl/src/booking_information.rs:309` — `shortday_gate::read_active_from` (bleibt, D-52-09)
- `service_impl/src/booking_information.rs:383-402` — CVC-06 Filter im Consumer (bleibt)
- `service_impl/src/booking_information.rs:409-439` — Slot-Clipping (bleibt)

### DAOs

- `dao_impl_sqlite/src/shiftplan_report.rs:142-176` — `extract_raw_shiftplan_report_for_week` Vorbild → **neu:** `extract_raw_shiftplan_report_for_year` (`WHERE booking.year = ?`)
- `dao_impl_sqlite/src/slot.rs::get_slots_for_week_all_plans` — Filter-Semantik-Vorbild für In-Memory-Filter

### REST

- `rest/src/report.rs:140-160` — `reporting.get_week`-Konsument. Signatur MUSS unangetastet bleiben (R4).

### Migrations

- `migrations/sqlite/20240502113031_add-slot.sql` — `slot`-Schema
- `migrations/sqlite/20240507063704_add-booking.sql` — `booking(year, calendar_week)`-Spalten, **KEIN INDEX**
- `migrations/sqlite/20240618125847_paid-sales-persons.sql` — `working_hours`, `extra_hours` Basis-Tabellen
- Alle `CREATE INDEX`-Migrations grepped: KEIN Index auf `booking.year`, `booking.calendar_week`, `working_hours.from_year`, `extra_hours.date_time` (Q3)

### Docs

- `docs/features/F07-reporting-balance.md` + `docs/features/F07-reporting-balance_de.md` — Docs-Freshness-Gate. Balance-Formel unverändert; wenn Doku keine Trait-Signaturen zitiert, kein Update nötig.

### CLAUDE.md-Constraints (Projekt)

- `shifty-backend/CLAUDE.md` — Service-Tier-Konventionen: `ReportingService`, `ShiftplanReportService`, `SpecialDayService`, `SlotService`, `ExtraHoursService` bleiben **Basic**; `BookingInformationService` bleibt **Business-Logic**. Neue `_for_year`-Methoden ändern das nicht.
- Clippy-Gate: `cargo clippy --workspace -- -D warnings` ist hart.
- sqlx-Gate: `cargo sqlx prepare --workspace` + `.sqlx`-Commit nach neuer `query!`-Makro.
- Docs-Freshness-Gate: `service/**/*.rs`-Änderung triggert `docs/features/F*.md`-Check; F07 hier vermutlich nur Änderungshistorie-Notiz.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `find_by_year` als neuer Trait-Endpoint auf `ExtraHoursService` ist die richtige Wahl (Option a) statt In-Memory-Filter (Option b) | Q2 Refactor-Steps Wave 2 Task 2.2 | Wenn Planner Option b bevorzugt: gesamte Historie wird pro Endpoint-Call geladen. Bei großen Datenbeständen wird das langsamer als der Full-Scan mit `WHERE`. Fallback dokumentiert. |
| A2 | Die Async-Calls im Helper (`derive_hours_for_range`, `build_derived_holiday_map`, `sales_person.get`) sind schnell genug, dass Load-once dieser drei nicht Teil dieser Phase sein muss | Q2, Risiken R3 | Wenn Latenz-Ziel < 500 ms nicht erreicht wird trotz Bulk-Load der Top-3-Kollektionen, ist eine zusätzliche Load-once-Optimierung nötig (byte-identisch). Wave 4 Latenz-Messung dokumentiert das. |
| A3 | `SlotDao::get_slots_for_week_all_plans`-Semantik lässt sich als reine `valid_from <= week_end && (valid_to IS NULL || valid_to >= week_start)` Filter-Closure exakt nachbauen | Wave 3, Risiko R1 | Wenn die DAO-Query zusätzliche Guards hat (z.B. `deleted IS NULL`), muss der In-Memory-Filter das identisch nachbilden. Executor MUSS die DAO-Query lesen. |
| A4 | Docs-Freshness auf F07 ist optional — Formel unverändert, nur additive Trait-Erweiterung | Docs-Sektion | Wenn F07 aktuell Trait-Signaturen zitiert, müssen die aktualisiert werden. Executor prüft am Ende. |

## Open Questions

Alle offenen Fragen wurden in Q1–Q3 beantwortet. Verbleibend nur die **eine strategische Frage für den Planner:**

**OQ-1: ExtraHours-Batch — Option a (`find_by_year` als neue Trait-Methode) oder Option b (`find_all` + In-Memory-Filter)?**
- Empfehlung Research: **Option a** (symmetrisch zu D-52-06 „nur additive `_for_year`", spart Speicher, konsistent zum Muster).
- CONTEXT.md D-52-06 zählt `ExtraHours` nicht explizit auf → Planner-Diskretion. Bitte in PLAN.md dokumentieren.

## Sources

### Primary (HIGH confidence — Code-verifiziert im Repo)

- `service_impl/src/reporting.rs:884-1160` — `get_week`-Body (kompletter Aggregations-Pfad)
- `service_impl/src/booking_information.rs:259-491` — `get_weekly_summary`-Body
- `service_impl/src/shiftplan_report.rs:258-310` — `_for_week`-Vorbild
- `dao_impl_sqlite/src/shiftplan_report.rs:142-176` — DAO-Vorbild
- `service/src/*.rs` (Trait-Definitionen)
- `migrations/sqlite/*.sql` — Index-Audit via grep (nichts auf `booking.year` / `extra_hours.date_time` / `working_hours.from_year`)
- `.planning/phases/52-weekly-overview-performance-refactor/52-CONTEXT.md` — locked Decisions D-52-01..D-52-16

### Secondary

- `.planning/REQUIREMENTS.md` (WOP-01..05)
- `.planning/notes/weekly-overview-perf-analyse.md` (Hotspot-Bestätigung)
- `.planning/seeds/weekly-overview-perf.md` (Umbau-Skizze)

## Metadata

**Confidence breakdown:**
- Q1 CVC-06 Cap-Semantik: **HIGH** — direkt aus `apply_weekly_cap`-Definition und Anwendung im Loop verifiziert
- Q2 `assemble_weeks`-Struktur: **HIGH** — mechanischer Extract aus bestehendem `get_week`-Code, Big-O aus Zählung
- Q3 DB-Indices: **HIGH** — vollständiger `grep CREATE INDEX` gegen alle Migrations, keine Vermutung
- Q2 Async-Call-Skalierung (R3): **MEDIUM** — die 3 pro-Person-pro-Woche-Calls sind ein bekanntes Perf-Risiko, aber nicht gemessen. Wave 4 klärt das empirisch.
- Frontend-Impact: **HIGH** — Frontend nicht angetastet (D-52-15)

**Research date:** 2026-07-05
**Valid until:** 2026-08-05 (Code-Stand HEAD `b99fcab`)

## RESEARCH COMPLETE
