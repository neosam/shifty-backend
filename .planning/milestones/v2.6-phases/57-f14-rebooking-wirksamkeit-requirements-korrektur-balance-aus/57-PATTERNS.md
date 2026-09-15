# Phase 57: F14-Rebooking-Wirksamkeit — Pattern Map

**Mapped:** 2026-07-31
**Files analyzed:** 14 (überwiegend Modifikation bestehender Dateien, keine Neuanlage)
**Analogs found:** 14 / 14 (jede zu ändernde Stelle hat eine unmittelbar benachbarte
Analog-Stelle in derselben Datei — diese Phase ist "wie sieht es an der
Nachbar-Site aus", nicht "welches andere Modul kopiere ich")

**Besonderheit:** Kein neues Datei-Skelett nötig. Für jede Stelle unten: wörtlicher
Ist-Stand-Auszug + die konkrete Nachbar-Site, die zeigt, wie es nach der Änderung
aussehen soll.

## File Classification

| Datei | Rolle | Data Flow | Art der Änderung |
|-------|-------|-----------|-------------------|
| `service_impl/src/reporting.rs` | service (Business-Logic-Tier, read-aggregation) | CRUD (read-heavy aggregation) | 4 Filter-Sites ändern (3× streichen, 1× gezielt einschränken) |
| `service_impl/src/rebooking_reconciliation.rs` | service (Business-Logic-Tier) | CRUD (write, mit Balance-Read) | `week_excess` Signatur +1 Parameter, 2 Call-Sites anpassen, 1 neuer Balance-Read pro Call-Site |
| `service_impl/src/billing_period_report.rs` | service (Business-Logic-Tier) | batch/snapshot | Konstante bumpen + History-Kommentar ergänzen |
| `service_impl/src/test/rebooking_neutrality.rs` | test | unit/property | 2 von 3 Assertions invertieren (Bit-Identität → Delta-Toleranz) |
| `service_impl/src/test/rebooking_reconciliation.rs` | test | unit/property | 1 von 2 Property-Tests invertieren, neue Deckel-Testfälle |
| `service_impl/src/test/voluntary_rebooking.rs` | test | unit (table-driven pure fn) | `week_excess_matrix` um 4. Parameter erweitern + neue Deckel-Zeilen |
| `service_impl/src/test/rebooking_backfill.rs` | test | unit | neuer Test: Preview-Excess zeigt gedeckelten Wert |
| `service_impl/src/test/billing_period_straddling.rs` | test | unit (golden fixture) | Fixture-Assertion invertieren (±h statt Byte-Identität) |
| `.planning/REQUIREMENTS.md` | config/doc | — | VOL-ACCT-03 neu fassen, 2 neue REQs (VOL-ACCT-04, REB-AUTO-06), Traceability-Tabelle erweitern |
| `docs/features/F14-rebooking.md` + `_de.md` | doc | — | §3 (Marker-Filter-Regel), §9.4 (Snapshot-Entscheidung) umschreiben |
| `docs/features/F07-reporting-balance.md` + `_de.md` | doc | — | §2.6 (`source == Manual`-Referenz veraltet) |
| `docs/features/F08-billing-period.md` + `_de.md` | doc | — | §7.4 Bump-History-Tabelle um v13 ergänzen |
| `docs/domain/time-accounting.md` (+ `_de.md` falls vorhanden) | doc | — | Kategorie-Semantik-Absatz |
| `docs/domain/billing-period.md` (+ `_de.md` falls vorhanden) | doc | — | Snapshot-Rationale-Absatz |

## Pattern Assignments

### `service_impl/src/reporting.rs` — vier Filter-Sites

Alle vier Sites folgen syntaktisch dem gleichen Muster
(`.filter(|eh| eh.source == ExtraHoursSource::Manual)`), aber **nur drei dürfen
identisch behandelt werden** (D-57-06). Zeilennummern frisch gegen den aktuellen
Code verifiziert (2026-07-31, Stand dieser Session).

#### Site 1 — `assemble_weeks`, Zeile 859 (Bucket-Filter, SONDERFALL)

**Ist-Stand** (Zeilen 849–864):
```rust
        for &(year, week) in weeks {
            let shiftplan_report = shiftplan_reports
                .iter()
                .filter(|r| r.year == year && r.calendar_week == week)
                .collect_to_hash_map_by(|r| r.sales_person_id);
            let extra_hours_bucket = extra_hours
                .iter()
                // Phase 55 (VOL-ACCT-03 / D-55-A1): Rebooking-Rows aus dem
                // Weekly-Summary-Aggregat filtern (VolunteerWork- UND
                // ExtraWork-Bein) — die Weekly-Sicht ist rebooking-neutral.
                .filter(|eh| eh.source == ExtraHoursSource::Manual)
                .filter(|eh| {
                    let sw = eh.to_date().as_shifty_week();
                    sw.year == year && sw.week == week
                })
                .collect_to_hash_map_by(|eh| eh.sales_person_id);
```

**Warum SONDERFALL:** Dieser Filter wirkt auf den gesamten `extra_hours_bucket`,
der danach für ALLE Felder der Woche wiederverwendet wird — nicht nur für
`extra_working_hours`. Zeile 876 filtert dort über `availability()`
(`VolunteerWork.availability() == Available`), NICHT über `as_report_type()`
wie die Sites 2–4. Würde man hier den `:859`-Filter komplett streichen, würde
das Rebooking-`-h VolunteerWork`-Bein wieder mitgezählt und die
Balance-Anhebung netto neutralisiert (Pitfall 1 im Research).

**Ziel-Muster:** Bucket-Load bleibt UNGEFILTERT nach `source` (nur der
Wochen-Filter bleibt an `:859`); stattdessen wird an Zeile 876
(`extra_working_hours`) gezielt nur die Rebooking-getaggte `VolunteerWork`
ausgenommen — `manual_volunteer_hours` (Zeile 932) bleibt unverändert (dort
MUSS die Rebooking-Zeile drin bleiben, damit das Ehrenamt-Ist sinkt).

**Nachbar-Site, die das Analog-Muster für "Kategorie-basierter Filter ohne
`source`" zeigt** — Zeile 876 selbst (heutiger Zustand, VOR der Änderung):
```rust
                let employee_extra_hours = extra_hours_bucket.get(&sales_person_id);
                let extra_working_hours = employee_extra_hours
                    .map(|eh| {
                        eh.iter()
                            .filter(|eh| eh.category.availability() == Availability::Available)
                            .map(|eh| eh.amount)
                            .sum::<f32>()
                    })
                    .unwrap_or(0.0);
```
und Zeile 932 (`manual_volunteer_hours`, bleibt exakt so):
```rust
                let manual_volunteer_hours = employee_extra_hours
                    .map(|eh| {
                        eh.iter()
                            .filter(|eh| eh.category == ExtraHoursCategory::VolunteerWork)
                            .map(|eh| eh.amount)
                            .sum::<f32>()
                    })
                    .unwrap_or(0.0);
```

**Code-Sketch für den Fix** (aus RESEARCH.md, Pattern 2 — bereits vom
Researcher gegen den Code verifiziert):
```rust
let extra_working_hours = employee_extra_hours
    .map(|eh| {
        eh.iter()
            .filter(|eh| eh.category.availability() == Availability::Available)
            // Phase 57 (VOL-ACCT-04): Rebooking-VolunteerWork zaehlt NICHT
            // in die Balance (das +h-ExtraWork-Bein traegt das bereits).
            // Gewoehnliche (Nicht-Rebooking) VolunteerWork bleibt laut
            // RISK-01 unveraendert Teil dieser Summe (Alt-Inkonsistenz,
            // nicht Scope dieser Phase).
            .filter(|eh| {
                !(eh.category == ExtraHoursCategory::VolunteerWork
                    && eh.source == ExtraHoursSource::Rebooking)
            })
            .map(|eh| eh.amount)
            .sum::<f32>()
    })
    .unwrap_or(0.0);
```

#### Site 2 — `get_reports_for_all_employees`, Zeile 1192 (Feld-Filter, STREICHEN)

**Ist-Stand** (Zeilen 1186–1194):
```rust
            // Phase 55 (VOL-ACCT-03 / D-55-A1): Rebooking-Rows aus dem
            // All-Employees-Report filtern (VolunteerWork- UND ExtraWork-Bein),
            // damit eine Umbuchung in dieser Sicht balance/volunteer-neutral ist.
            let extra_hours_array: Arc<[ExtraHours]> = extra_hours_array
                .iter()
                .filter(|eh| eh.source == ExtraHoursSource::Manual)
                .cloned()
                .collect();
```
**Fix:** Den `.filter(|eh| eh.source == ExtraHoursSource::Manual)`-Aufruf
inklusive Kommentar ersatzlos streichen. Downstream trennt
`as_report_type()` (Zeilen ~1404 ff.) `WorkingHours` von `Documented` bereits
korrekt — kein weiterer Eingriff an dieser Site nötig.

**Beleg, warum das ausreicht** (Downstream-Analog, `:1404-1408`, unverändert):
```rust
let extra_working_hours = week_extra_hours
    .iter()
    .filter(|eh| eh.category.as_report_type() == ReportType::WorkingHours)
    .map(|eh| eh.amount)
    .sum::<f32>();
```

#### Site 3 — `overall_extra_work_hours`, Zeile 1728 (Feld-Filter, STREICHEN)

**Ist-Stand** (Zeilen 1720–1735):
```rust
        // Hinweis: Das rohe, ungedeckelte shiftplan_hours wird bewusst NICHT mehr
        // fuer overall/balance/shiftplan_hours verwendet (Debug
        // `report-ehrenamt-gesamtstunden`). Der per-Woche gedeckelte Wert
        // `shiftplan_hours_by_week` (siehe unten) ist die einzige Quelle.
        let overall_extra_work_hours = extra_hours
            .iter()
            // Phase 55 (VOL-ACCT-03 / D-55-A1): Rebooking-Rows raus, damit das
            // +N-ExtraWork-Bein einer Umbuchung nicht in overall/balance zaehlt.
            .filter(|eh| eh.source == ExtraHoursSource::Manual)
            .filter(|eh| {
                eh.to_date() >= from_date
                    && eh.to_date() <= to_date
                    && eh.category.as_report_type() == ReportType::WorkingHours
            })
            .map(|eh| eh.amount)
            .sum::<f32>();
```
**Fix:** Zeile `.filter(|eh| eh.source == ExtraHoursSource::Manual)` (inkl.
Kommentar) streichen — die verbleibende `.filter(...)`-Kette
(`to_date`-Range + `as_report_type() == WorkingHours`) bleibt unverändert und
trägt bereits die korrekte Kategorie-Semantik.

#### Site 4 — `hours_per_week`, Zeile 2161 (Feld-Filter, STREICHEN)

**Ist-Stand** (Zeilen 2151–2163):
```rust
    let mut weeks: Vec<GroupedReportHours> = Vec::new();
    for week in from_week.iter_until(&to_week) {
        tracing::info!("Week: {}, Year: {}", week.week, week.year);
        let filtered_extra_hours_list = extra_hours_list
            .iter()
            // Phase 55 (VOL-ACCT-03 / D-55-A1): Rebooking-Rows (source != Manual)
            // werden aus ALLEN Read-Aggregaten dieser Woche gefiltert — sowohl
            // aus den VolunteerWork- als auch aus den ExtraWork/WorkingHours-
            // Sites, die in overall/balance einfliessen. Damit ist eine einzelne
            // manuelle Umbuchung (-N/+N-Pair) balance/ist/soll/delta-neutral.
            .filter(|eh| eh.source == ExtraHoursSource::Manual)
            .filter(|eh| eh.to_date().as_shifty_week() == week)
            .collect::<Vec<_>>();
```
**Fix:** `.filter(|eh| eh.source == ExtraHoursSource::Manual)` (inkl.
Kommentar) streichen. Downstream-Kategorie-Filter innerhalb dieser Funktion
(belegt durch Test `volunteer_extra_hours_excluded_from_balance_and_overall`,
Zeile ~2800) bleiben unverändert und liefern das gewünschte Verhalten.

#### `alert_predicate` / `proposed_hours` (Zeilen 395/405) — UNVERÄNDERT, Referenz für Planner

```rust
pub fn alert_predicate(balance: f32, voluntary_ist: f32, cap_active: bool) -> bool {
    cap_active && balance <= -0.5 && voluntary_ist > 0.0
}

pub fn proposed_hours(balance: f32, voluntary_ist: f32) -> f32 {
    balance.abs().min(voluntary_ist)
}
```
Beide bleiben **exakt so** (Out-of-Scope laut Context). Sie sind Beleg dafür,
dass der HR-Vorschlagspfad bereits die richtige Formel hat — nur der
Auto-Cron-Pfad (`week_excess`) braucht den neuen Deckel-Parameter.

---

### `service_impl/src/rebooking_reconciliation.rs` — `week_excess` + 2 Call-Sites

**Ist-Stand `week_excess`** (Zeilen 59–73):
```rust
/// Reine, testbare Ueberschuss-Formel (F4 core, REB-AUTO-01): der buchbare
/// Ausgleich einer Woche ist `Ist - (Soll + committed_voluntary)`, geklemmt auf
/// `>= 0`. Nicht-endliche Eingaben (NaN/Inf) ergeben `0.0` (defensiver Skip statt
/// Panik / Muell-Buchung).
pub(crate) fn week_excess(ist: f32, soll: f32, committed_voluntary: f32) -> f32 {
    if !ist.is_finite() || !soll.is_finite() || !committed_voluntary.is_finite() {
        return 0.0;
    }
    let excess = ist - (soll + committed_voluntary);
    if excess > 0.0 {
        excess
    } else {
        0.0
    }
}
```

**Ziel-Signatur** (Code-Sketch aus RESEARCH.md, Pattern 3):
```rust
pub(crate) fn week_excess(ist: f32, soll: f32, committed_voluntary: f32, balance: f32) -> f32 {
    if !ist.is_finite() || !soll.is_finite() || !committed_voluntary.is_finite() || !balance.is_finite() {
        return 0.0;
    }
    let unconstrained = ist - (soll + committed_voluntary);
    let unconstrained = if unconstrained > 0.0 { unconstrained } else { 0.0 };
    let headroom = if -balance > 0.0 { -balance } else { 0.0 };
    unconstrained.min(headroom)
}
```

**Call-Site 1 — `run_for_week`, Zeilen 610–618 (Ist-Stand):**
```rust
        // Ueberschuss = Ist - committed_voluntary (beide fuer die Wochen-Range
        // Mo..=So). `committed` == VoluntaryStats::soll_total (committed-voluntary-
        // target). Der Balance-Report wird nicht fuer die Excess-Formel benoetigt
        // (Ist/Soll kommen aus VoluntaryStats), aber wir lesen ihn NICHT, um den
        // Write-Pfad schlank zu halten.
        let (ist, _soll, committed) = self
            .week_voluntary_figures(sales_person_id, iso_year, iso_week, tx.clone())
            .await?;
        let excess = week_excess(ist, 0.0, committed);
```
Der Kommentar "Balance wird bewusst nicht gelesen" kippt (D-57-07) — dort
kommt der neue `get_report_for_employee_range`-Call rein (Code-Sketch siehe
RESEARCH.md Pattern 4, `year_start = ShiftyDate::first_day_in_year(iso_year)`,
`week_end` = Sonntag der ISO-Woche, `include_carryover = true`). **NICHT** das
Muster von `suggest`/`approve` (Zeilen 356–366/435–445) kopieren — das ist eine
isolierte Wochen-Range ohne Carryover (Pitfall 3, structurally falscher Range
für den Deckel).

**Call-Site 2 — `backfill_range`, Zeilen 715–718 (Ist-Stand):**
```rust
                let (ist, soll, committed) = self
                    .week_voluntary_figures(sales_person_id, iso_year, iso_week, read_tx.clone())
                    .await?;
                let excess = week_excess(ist, 0.0, committed);
```
**Kritischer Fund (Research):** Diese zweite Call-Site wird von der ROADMAP
nicht genannt, braucht aber denselben Balance-Read + denselben 4-Parameter-Aufruf
— sonst zeigt der Dry-Run-Preview-Wert (`BackfillPreviewItem.excess`) einen
falschen, ungedeckelten Betrag, der vom tatsächlichen `apply`-Lauf (über
`run_for_week`) abweicht (Pitfall 2). `read_tx` ist an dieser Stelle bereits
eine offene Transaktion (Zeile 697) — derselbe Balance-Read-Sketch, nur mit
`Some(read_tx.clone())` statt `Some(tx.clone())`.

**DI-Dependency bereits vorhanden** — `ReportingService` ist bereits Dep von
`RebookingReconciliationServiceImpl` (Zeile 48 im `gen_service_impl!`-Block):
```rust
gen_service_impl! {
    struct RebookingReconciliationServiceImpl: RebookingReconciliationService = RebookingReconciliationServiceDeps {
        ExtraHoursService: ExtraHoursService<Transaction = Self::Transaction, Context = Self::Context> = extra_hours_service,
        RebookingBatchService: RebookingBatchService<Transaction = Self::Transaction, Context = Self::Context> = rebooking_batch_service,
        ReportingService: ReportingService<Transaction = Self::Transaction, Context = Self::Context> = reporting_service,
        ...
```
Kein `main.rs`-DI-Change nötig — Reihenfolge (`reporting_service` vor
`rebooking_reconciliation_service`) bereits gegeben (Research, per `grep`
gegen `shifty_bin/src/main.rs` verifiziert).

---

### `service_impl/src/billing_period_report.rs` — Snapshot-Bump

**Ist-Stand** (Zeilen 108–117, letzter History-Eintrag + Konstante):
```rust
/// - v12: Phase 28 (VAC-OFFSET-01 / D-28-05) — off-by-one fix in
///   `EmployeeWorkDetails::vacation_days_for_year`. The year-START proration no longer
///   over-subtracts ~1/365 of the annual entitlement at a 1.1. contract start (it now
///   subtracts the days STRICTLY before the start, so a 1.1. start subtracts 0). Because
///   this value feeds the persisted `BillingPeriodValueType::VacationEntitlement`
///   (reporting.rs:853 <- :803), the computed entitlement changes for partial-year and
///   full-year contracts. Validators MUST treat v11 snapshots as "older schema" and skip
///   vacation-entitlement re-validation for those entries. NOTE: `VacationDays` (taken
///   vacation) is UNAFFECTED — only `VacationEntitlement` (contract aliquot) changes.
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;
```
**Fix:** Neuer `/// - v13: ...`-Absatz (analog obigem Muster: was ändert sich,
welcher `value_type`, warum Validatoren v12 als "älteres Schema" behandeln
müssen) VOR der Konstante einfügen, Konstante auf `13` setzen. Begründung
laut D-57-10: "Change the input set the computation reads from" (Rebooking-Rows
fließen jetzt in `Balance`/`ExtraWork`/`Volunteer` ein, wo sie vorher gefiltert
wurden).

**Wo gestempelt wird** (Zeile 390, unverändert, nur zur Orientierung):
```rust
            snapshot_schema_version: CURRENT_SNAPSHOT_SCHEMA_VERSION,
```

**Analog für "kein Bump nötig"-Gegenstück** (Zeile 369, zeigt das
Kommentar-Muster für den entgegengesetzten Fall, nützlich als Kontrast beim
Formulieren des v13-Eintrags):
```rust
            // KEIN value_type-Change -> KEIN CURRENT_SNAPSHOT_SCHEMA_VERSION-Bump
```

---

### `service_impl/src/test/rebooking_neutrality.rs` — Fixture + 3 Assertions

**Datei-Doku-Kopf, Ist-Stand (Zeilen 1–26)** — muss neu erklären, was
"Neutralität" nach der Inversion noch heißt (nur noch Soll-Schutz):
```rust
//! Phase 56 Plan 01 — D-06 Rebooking-Neutralitäts-Property-Tests (REB-AUTO-02).
//!
//! Diese drei Tests sind das **CI-Gate** für D-06: eine EINZELNE Umbuchung
//! (das -N/+N-ExtraHours-Pair mit `source=Rebooking`) lässt die drei
//! Read-Aggregate **bit-identisch**:
//! ...
//! Wenn eine Assertion FEHLSCHLÄGT, ist das eine echte Entdeckung (eine Kette
//! leakt Rebooking-Rows) — NICHT die Assertion anpassen, sondern melden.
```
Dieser Kopf-Kommentar (insb. der letzte Satz) ist der zu invertierende
Kernfehler — er verbietet explizit das, was jetzt korrekt ist. Muss auf D-57-08
umgeschrieben werden: Balance/F1-Ist sollen sich um `h` bewegen,
nur F2-Soll bleibt neutral.

**`assert_bit_eq`-Helper, Ist-Stand (Zeilen 58–71)** — bleibt für
`test_neutrality_f2_soll` erhalten, wird für die anderen zwei durch
Delta-Assertion ersetzt:
```rust
#[track_caller]
fn assert_bit_eq(before: f32, after: f32, aggregate: &str) {
    assert!(
        !before.is_nan() && !after.is_nan(),
        "{aggregate} is NaN — cannot compare rebooking-neutrality bit-exact (before={before}, after={after})"
    );
    assert_eq!(
        before.to_bits(),
        after.to_bits(),
        "{aggregate} must be rebooking-neutral: before={before} (0x{:08x}), after={after} (0x{:08x}) — a chain leaked the rebooking pair",
        before.to_bits(),
        after.to_bits(),
    );
}
```
**Epsilon-Muster, das für die zwei invertierten Fälle genutzt werden soll**
(bereits als Analog woanders in derselben Codebase vorhanden, siehe
`rebooking_reconciliation.rs`-Test unten): `(a - b).abs() < 1e-3`.

**Die 3 Tests, Ist-Stand (Zeilen 331–358):**
```rust
/// D-06 (1/3): `balance_hours` bleibt bit-identisch nach einem Rebooking-Pair.
#[tokio::test]
async fn test_neutrality_balance() {
    let (bal_before, _) = balance_and_ist(baseline_extra_hours()).await;
    let (bal_after, _) = balance_and_ist(with_rebooking_pair()).await;
    assert_bit_eq(bal_before, bal_after, "balance_hours");
}

/// D-06 (2/3): F1-Ist (`volunteer_hours` == `VoluntaryStats::ist_total`) bleibt
/// bit-identisch — die Rebooking-Rows (source=Rebooking) werden aus dem
/// Ist-Aggregat gefiltert.
#[tokio::test]
async fn test_neutrality_f1_ist() {
    let (_, ist_before) = balance_and_ist(baseline_extra_hours()).await;
    let (_, ist_after) = balance_and_ist(with_rebooking_pair()).await;
    assert_bit_eq(ist_before, ist_after, "f1_ist (volunteer_hours)");
}

/// D-06 (3/3): F2-Soll (`VoluntaryStats::soll_total`) bleibt bit-identisch —
/// Soll liest überhaupt keine ExtraHours (nur committed_voluntary + Absences),
/// ist also per Konstruktion rebooking-blind. Der Test exerziert den vollen
/// HR-gated VoluntaryStats-Pfad, um diese Unabhängigkeit empirisch zu belegen.
#[tokio::test]
async fn test_neutrality_f2_soll() {
    let soll_before = soll_total(baseline_extra_hours()).await;
    let soll_after = soll_total(with_rebooking_pair()).await;
    assert_bit_eq(soll_before, soll_after, "f2_soll (soll_total)");
}
```
`test_neutrality_f2_soll` bleibt **unverändert** (VOL-ACCT-03 neu, Soll ist
strukturell rebooking-blind). Die anderen zwei werden von `assert_bit_eq(...)`
auf eine Delta-Assertion mit dem `h`-Wert der Fixture (aus `with_rebooking_pair()`,
laut Research h=2.0) umgestellt: `balance_after == balance_before + h`,
`ist_after == ist_before - h`.

**Fixture-Setup (Referenz für `h`, Zeilen 75–90):**
```rust
fn sp_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_0001)
}

/// Gedeckelter Vertrag 5h/Woche Mo-Fr, KW 10..=20/2026, cap aktiv,
/// committed_voluntary=5.0.
fn work_details() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_0010),
        sales_person_id: sp_id(),
        expected_hours: 5.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 10,
        from_year: 2026,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 20,
        ...
```
`baseline_extra_hours()` / `with_rebooking_pair()` liegen weiter unten in
derselben Datei (nicht in dieser Session erneut gelesen — beim Planen per
`grep -n "fn baseline_extra_hours\|fn with_rebooking_pair"` lokalisieren, um
den exakten `h`-Wert zu bestätigen).

---

### `service_impl/src/test/rebooking_reconciliation.rs` — Property-Tests + Mock-Helper

**`rebooking_neutrality_single_manual`, Ist-Stand (Zeilen 241–257) — INVERTIEREN:**
```rust
/// VOL-ACCT-03 (D-55-A1) HART: eine EINZELNE VolunteerToExtra-Umbuchung
/// (-N VolunteerWork + +N ExtraWork, beide source=Rebooking) laesst balance
/// UND volunteer_hours (Ist-Aggregat) unveraendert. Property: baseline == pair.
#[tokio::test]
async fn rebooking_neutrality_single_manual() {
    let (bal_before, vol_before) = balance_and_volunteer(baseline_extra_hours()).await;
    let (bal_after, vol_after) = balance_and_volunteer(extra_hours_with_rebooking_pair()).await;

    assert!(
        (bal_before - bal_after).abs() < 1e-3,
        "balance must be rebooking-neutral: before={bal_before}, after={bal_after}"
    );
    assert!(
        (vol_before - vol_after).abs() < 1e-3,
        "volunteer_hours (Ist) must be rebooking-neutral: before={vol_before}, after={vol_after}"
    );
}
```
Das Epsilon-Muster (`(a-b).abs() < 1e-3`) ist bereits vorhanden — der Fix
ändert nur die Zielwerte: `(bal_after - (bal_before + h)).abs() < 1e-3` und
`(vol_after - (vol_before - h)).abs() < 1e-3`, wobei `h` aus
`extra_hours_with_rebooking_pair()` folgt (Fixture in derselben Datei
weiter oben; beim Planen den `-N`/`+N`-Wert dort per `grep` bestätigen).

**`rebooking_neutrality_roundtrip`, Ist-Stand (Zeilen 259–282) — UNVERÄNDERT lassen:**
```rust
/// VOL-ACCT-03 Roundtrip: hin (VolunteerToExtra) UND zurueck (ExtraToVolunteer)
/// — also zwei Pairs (4 Rebooking-Rows, netto 0) — ebenfalls neutral.
#[tokio::test]
async fn rebooking_neutrality_roundtrip() {
    let mut roundtrip = baseline_extra_hours();
    roundtrip.push(eh(-2.0, ExtraHoursCategory::VolunteerWork, ExtraHoursSource::Rebooking, 3));
    roundtrip.push(eh(2.0, ExtraHoursCategory::ExtraWork, ExtraHoursSource::Rebooking, 4));
    roundtrip.push(eh(-2.0, ExtraHoursCategory::ExtraWork, ExtraHoursSource::Rebooking, 5));
    roundtrip.push(eh(2.0, ExtraHoursCategory::VolunteerWork, ExtraHoursSource::Rebooking, 6));
    ...
```
Bleibt exakt so — zwei Pairs heben sich algebraisch auf, unabhängig vom
Filter-Fix (Pitfall 4).

**`reporting_service(balance: f32)`-Mock-Helper** — dieser existiert bereits
verdrahtet, wird bisher aber von keinem `run_for_week`-Testfall tatsächlich
zum Deckel-Bind gebracht (Research-Fund). Fundstelle liegt in
`test/voluntary_rebooking.rs` (siehe unten), nicht in dieser Datei — die
7 `run_for_week`-Testfälle in `rebooking_reconciliation.rs` selbst rufen
`build_service(...)` mit `reporting_service(-8.0)` auf; nach dem Fix bleiben
sie unverändert grün (Balance −8 bindet den Deckel in keinem der 7 Fälle).

---

### `service_impl/src/test/voluntary_rebooking.rs` — Mock-Helper + `week_excess_matrix`

**`reporting_service(balance: f32)`-Helper, Ist-Stand (Zeilen 162–167):**
```rust
fn reporting_service(balance: f32) -> MockReportingService {
    let mut r = MockReportingService::new();
    r.expect_get_report_for_employee_range()
        .returning(move |_, _, _, _, _, _| Ok(make_report(balance)));
    r
}
```
Signatur des gemockten `get_report_for_employee_range` hat 6 Positionsparameter
(`_, _, _, _, _, _`) — das Analog für die im Plan geforderte Anpassung von
`run_for_week`/`backfill_range`, die diesen Aufruf real nutzen sollen (siehe
Code-Sketch oben in `rebooking_reconciliation.rs`).

**`week_excess_matrix`, Ist-Stand (Zeilen 198–207) — TABELLEN-VORBILD für neue
Deckel-Fälle:**
```rust
/// REB-AUTO-01: week_excess == max(ist - (soll + committed), 0), NaN/Inf → 0.
#[test]
fn week_excess_matrix() {
    // ist - committed (soll=0 in run_for_week; committed==soll_total).
    assert!((week_excess(10.0, 0.0, 5.0) - 5.0).abs() < 1e-6, "10-5=5");
    assert!((week_excess(5.0, 0.0, 5.0) - 0.0).abs() < 1e-6, "5-5=0 clamp");
    assert!((week_excess(3.0, 0.0, 5.0) - 0.0).abs() < 1e-6, "negative clamps to 0");
    assert!((week_excess(f32::NAN, 0.0, 5.0) - 0.0).abs() < 1e-6, "NaN -> 0");
    assert!((week_excess(f32::INFINITY, 0.0, 5.0) - 0.0).abs() < 1e-6, "Inf -> 0");
}
```
Nach dem Signatur-Fix braucht jede Zeile einen 4. Parameter (`balance`).
**Neue Deckel-Zeilen (D-57-02-Gegenprobe, aus Research nachgerechnet):**
`week_excess(15.0, 0.0, 5.0, -2.0)` muss `2.0` liefern (nicht `10.0`);
`week_excess(10.0, 0.0, 5.0, 0.0)` muss `0.0` liefern (Deckel bindet bei
Balance ≥ 0, kein Übertrag trotz Excess).

`build_service`, Ist-Stand (Zeilen 169–194) — unverändertes DI-Wiring-Analog,
falls neue Testfälle einen eigenen `RebookingReconciliationServiceImpl`
brauchen:
```rust
#[allow(clippy::too_many_arguments)]
fn build_service(
    extra_hours_service: MockExtraHoursService,
    rebooking_batch_service: MockRebookingBatchService,
    reporting_service: MockReportingService,
    voluntary_stats_service: MockVoluntaryStatsService,
    week_status_service: MockWeekStatusService,
    employee_work_details_service: MockEmployeeWorkDetailsService,
    toggle_service: MockToggleService,
    clock_service: MockClockService,
    permission_service: MockPermissionService,
    transaction_dao: dao::MockTransactionDao,
) -> RebookingReconciliationServiceImpl<TestDeps> {
    RebookingReconciliationServiceImpl {
        extra_hours_service: Arc::new(extra_hours_service),
        rebooking_batch_service: Arc::new(rebooking_batch_service),
        reporting_service: Arc::new(reporting_service),
        voluntary_stats_service: Arc::new(voluntary_stats_service),
        week_status_service: Arc::new(week_status_service),
        employee_work_details_service: Arc::new(employee_work_details_service),
        toggle_service: Arc::new(toggle_service),
        clock_service: Arc::new(clock_service),
        permission_service: Arc::new(permission_service),
        transaction_dao: Arc::new(transaction_dao),
    }
}
```

`run_for_week_books_excess`-Testfall (Zeilen 213 ff.) zeigt das Standard-Muster
für einen neuen "Deckel bindet"-Testfall (Mock-Setup für
`ExtraHoursService::create`, `RebookingBatchService::create`,
Assertion auf `RebookingBatchKind::AutoCron` + `RebookingBatchState::Approved`)
— als Vorlage für den neuen Testfall aus REB-AUTO-06 kopierbar, nur mit
`reporting_service(-2.0)` statt `reporting_service(-8.0)` und Ist=15 statt
den bisherigen Werten.

---

### `service_impl/src/test/rebooking_backfill.rs` — Preview-Test-Erweiterung

Kein Ist-Stand-Auszug in dieser Session gelesen (Datei existiert laut Research
bereits mit Backfill-Preview-Tests). **Vorgehen für den Planner:** Struktur an
`voluntary_rebooking.rs`s `reporting_service()`-Helper spiegeln (s.o.) — das ist
auch die explizite Empfehlung aus RESEARCH.md Open-Question 1. Neuer Test
muss belegen: `BackfillPreviewItem.excess` zeigt den gedeckelten Wert
(`min(unconstrained, headroom)`), nicht den `unconstrained`-Rohwert — sonst
weicht der Dry-Run-Preview vom tatsächlichen `run_for_week`-Write ab
(Pitfall 2).

---

### `service_impl/src/test/billing_period_straddling.rs` — Fixture-Inversion

Kein Ist-Stand-Auszug in dieser Session gelesen. Laut Research (Pitfall 5)
ist die bestehende Fixture (Baseline-VolunteerWork=3.0, Rebooking-Delta=−2.0
→ Ergebnis 1.0 ≠ 0) bereits sicher gewählt — **diesen `h`-Wert beibehalten,
nicht ersetzen**, sonst verschwindet der `BillingPeriodValueType::Volunteer`-Key
komplett aus der Snapshot-Map (`report_delta.volunteer_hours != 0.0`-Gate in
`billing_period_report.rs:283`) und die Test-Fehlermeldung wird irreführend
("key set differs" statt Werte-Delta). Fix-Ziel laut D-57-11: Periode MIT
Rebooking-Woche zeigt `±h`-Delta auf Balance/ExtraWork/Volunteer; Perioden
OHNE Rebooking-Woche bleiben byte-identisch (3 bestehende Toggle-Tests
unverändert lassen).

---

### `.planning/REQUIREMENTS.md` — VOL-ACCT-03 Neufassung + 2 neue REQs

**Ist-Stand VOL-ACCT-03 (Zeilen 57–71):**
```
- [x] **VOL-ACCT-03**: Rebooking-Pair-ExtraHours (F3/F4/F5) beeinflussen weder
      VOL-STAT-01-Ist noch VOL-ACCT-01-Soll noch VOL-ACCT-01-Ist. Property-Test:
      Rebooking über eine Woche → Ist/Soll/Delta unverändert (balance-neutral).

      > ⚠️ **FEHLERHAFT — wird in Phase 57 neu gefasst. Nicht darauf weiterbauen.**
      > Der Zusatz „(balance-neutral)" widerspricht dem Milestone-Goal dieser
      > Datei (Zeile 10–15: Freiwilligen-Stunden „als Ausgleich gegen ein
      > negatives Stundenkonto einsetzen"). Wörtlich umgesetzt in Phase 55
      > (D-55-A1, `source == Manual`-Filter an 4 Aggregations-Sites) macht es den
      > gesamten Übertrag wirkungslos: die Balance bleibt nach einer Umbuchung
      > unverändert (empirisch `−53 → −53`), das Minus wandert per Carryover ins
      > Folgejahr. Gemeint war der Schutz der **Freiwilligen-Statistik** gegen
      > Doppelzählung, nicht das Einfrieren der Balance. Zudem fehlt in dieser
      > Liste ein Requirement, das den Ausgleich überhaupt operationalisiert —
      > beides liefert Phase 57. Analyse: ROADMAP.md §Phase 57.
```
Laut D-57-09: Neufassung schützt nur noch die Freiwilligen-Statistik
(Soll unverändert), ⚠️ wird durch kurzen Historien-Hinweis ersetzt (nicht
gelöscht).

**Format-Vorbild eines Nachbar-REQ** (`REB-MANUAL-01`, Zeilen 75–79 — als
Muster für die zwei neuen REQ-Blöcke VOL-ACCT-04 / REB-AUTO-06):
```
- [x] **REB-MANUAL-01**: HR kann im Mitarbeiter-Jahresreport eine Umbuchung
      Freiwillig ↔ Bezahlt anlegen (1-Klick-Aktion). Ergebnis: zwei atomar
      geschriebene ExtraHours-Rows (`−N` in Quell-Kategorie, `+N` in
      Ziel-Kategorie) plus ein `rebooking_batch(kind=manual)` mit einem
      `rebooking_batch_entry`. Alles in einer Transaktion; Rollback bei Fehler.
```
(Format: `- [x] **ID**: Beschreibung.` — Einrückung 6 Spaces bei Folgezeilen,
Backtick für Code-Identifier.) **Hinweis:** `[x]` bedeutet hier "als Requirement
erfasst", nicht "bereits implementiert" — Status-Spalte in der
Traceability-Tabelle trägt "Pending"/"Done" separat.

**Traceability-Tabellen-Kopf + Format (Zeilen 196–224):**
```
## Traceability

17 v1-Requirements → 3 Phasen (ARCHITECTURE-C 3-Phasen-Baseline aus Research
SUMMARY, dokumentiert in ROADMAP.md). Coverage: 17/17.

| Requirement    | Kategorie   | Phase    | Status  |
|----------------|-------------|----------|---------|
| VOL-STAT-01    | F1 Statistik | Phase 54 | Pending |
...
| VOL-ACCT-03    | F2 Konto    | Phase 54 | Pending |
...
```
**Wichtig:** VOL-ACCT-03 steht heute mit `Phase 54` in der Tabelle — laut
D-57-09 "VOL-ACCT-03 auf Phase 57 umhängen" muss diese Zeile umgeschrieben
werden, nicht nur ergänzt. Die zwei neuen Zeilen (`VOL-ACCT-04`, `REB-AUTO-06`)
kommen mit `Phase 57` hinzu; die einleitende Coverage-Zahl (`17/17`) muss auf
`19/19` aktualisiert werden (D-57-09).

---

### Docs-Freshness-Gate — Sektions-Gerüst

**`docs/features/F14-rebooking.md`** (Sektions-Nummerierung EN, identisch DE):
```
## 3. Marker-Filter Rule ([D-54-DM-02])       ← Filter-Regel umschreiben (D-57-06)
## 9.4 Snapshot-schema decision (REB-AUTO-05, D-01)  ← kippt (D-57-10), Bump-Text
```
`docs/features/F14-rebooking_de.md` spiegelt exakt (`## 3. Marker-Filter-Regel`,
`## 9.4 Snapshot-Schema-Entscheidung (REB-AUTO-05, D-01)`) — beide synchron
halten.

**`docs/features/F07-reporting-balance.md`** — betroffene Sektionen:
```
## 2.6 Further rules that are not obvious
  Zeile ~250: "with `source == ExtraHoursSource::Manual` and **excludes**..."
## 3. Data model
  Zeile ~336: "`service_impl/src/reporting.rs` filter `source == Manual` (keep
  `Manual`, exclude `Rebooking`) at their aggregation entry..."
```
Beide Referenzen sind nach dem Filter-Fix veraltet (3 von 4 Sites streichen
den Filter komplett) — DE-Pendant `F07-reporting-balance_de.md` analog prüfen.

**`docs/features/F08-billing-period.md`** — Ziel-Sektion:
```
### 7.4 History of bumps (verified in the code)
  Zeile 509: | **v12** | Phase 28 (VAC-OFFSET-01 / D-28-05) — off-by-one fix... |
```
Neue Tabellenzeile `**v13**` im selben Format anhängen (Phase 57-Referenz +
kurze Begründung + betroffener `value_type`).

**`docs/features/F08-billing-period_de.md`** — Ziel-Sektion:
```
### 7.4 Historie der Bumps (verifiziert im Code)
  Zeile 503: | **v12** | Phase 28 (VAC-OFFSET-01 / D-28-05) — off-by-one Fix... |
  Zeile 512: "**Milestone v2.6 (Phasen 54–56) — Non-Bump-Bestätigung."
  Zeile 528: "REB-AUTO-05-Bump-Frage — und pinnt den „Nein"-Zweig (kein Bump)."
```
**Wichtig für den Planner:** Die DE-Version hat einen zusätzlichen Absatz
("Milestone v2.6 Non-Bump-Bestätigung", Zeilen 512–541), der explizit
begründet, warum v12 NICHT gebumpt wurde — dieser Absatz widerspricht direkt
D-57-10 und muss inhaltlich umgeschrieben werden (nicht nur eine neue Zeile
anhängen wie in der EN-Version reicht hier nicht).

**`docs/domain/time-accounting.md`** und **`docs/domain/billing-period.md`** —
kein Sektions-Grep in dieser Session durchgeführt; laut Context/Research
betroffen ist die Kategorie-Semantik-Beschreibung (`time-accounting.md`) bzw.
die Snapshot-Rationale (`billing-period.md`). Planner sollte vor dem Schreiben
kurz per `grep -n "^#"` das Sektionsgerüst holen (analog zum obigen Muster für
F07/F08) — in dieser Session aus Budget-Gründen nicht mehr gezogen.

## Shared Patterns

### Kategorie-Semantik trägt die Fachlogik, nicht der `source`-Marker
**Quelle:** `service/src/extra_hours.rs:91` (`as_report_type()`),
`:114` (`availability()`), `:60` (`ExtraHoursSource`).
**Gilt für:** Alle 4 Filter-Sites in `reporting.rs`. Der `source`-Marker bleibt
als Provenienz-Feld (UI/Undo) erhalten, verliert aber seine Aggregations-Wirkung.

### Transaktionsklammer beim Aufrufer
**Quelle:** `rebook()` (`rebooking_reconciliation.rs:107`), committet nicht
selbst — der Aufrufer (`run_for_week`/`rebook_manual`/`approve`) besitzt die
Klammer.
**Gilt für:** Der neue Balance-Read in `run_for_week`/`backfill_range` MUSS
dieselbe (bereits offene) Transaktion via `tx.clone()`/`read_tx.clone()`
verwenden — keine neue Transaktion öffnen.

### Snapshot-Versionierung als Vergleichs-Gate, nicht Datenmigration
**Quelle:** `billing_period_report.rs:117` + Doc-Kommentar-Kette (v9–v12).
**Gilt für:** Der v13-Bump. Bestehende v12-Rows bleiben unangetastet (D-57-12);
Validatoren vergleichen nur innerhalb derselben `snapshot_schema_version`.

## No Analog Found

Keine — jede zu ändernde Datei existiert bereits mit einer direkt benachbarten
Analog-Stelle im selben Modul (siehe Klassifikationstabelle). Diese Phase legt
keine neuen Dateien an.

## Metadata

**Analog search scope:** `service_impl/src/reporting.rs`,
`service_impl/src/rebooking_reconciliation.rs`,
`service_impl/src/billing_period_report.rs`,
`service_impl/src/test/{rebooking_neutrality,rebooking_reconciliation,voluntary_rebooking}.rs`,
`.planning/REQUIREMENTS.md`, `docs/features/F14*`, `docs/features/F08*`,
`docs/features/F07*`.
**Files scanned:** 10 (alle vier Filter-Sites + beide `week_excess`-Call-Sites
+ Snapshot-Konstante + 3 Testdateien + REQUIREMENTS.md, per `Read`/`grep`
gegen den aktuellen Stand 2026-07-31 verifiziert — keine Zeilennummer aus
RESEARCH.md ungeprüft übernommen).
**Pattern extraction date:** 2026-07-31
</content>
