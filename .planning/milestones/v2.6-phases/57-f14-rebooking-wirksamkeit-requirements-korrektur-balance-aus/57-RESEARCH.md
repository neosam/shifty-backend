# Phase 57: F14-Rebooking-Wirksamkeit — Research

**Researched:** 2026-07-31
**Domain:** Interne Rust-Business-Logik (Reporting-Aggregation, Snapshot-Versionierung, Requirements-Pflege). Keine neuen externen Pakete, keine neue Infrastruktur.
**Confidence:** HIGH — alle Kernbehauptungen sind gegen den aktuellen Code verifiziert (`grep`/`Read`), nicht aus Trainingsdaten übernommen.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-57-01 — Zwei Untergrenzen, zwei Zuständigkeiten.** Auto-Cron gleicht die
  bezahlte Balance aus, aber nur soweit die Freiwilligen-Zusage noch erfüllt
  ist: `min( max(ist − committed, 0), max(−balance, 0) )`, Balance steigt nie
  über 0. HR-Vorschlag (`min(|balance|, ist)`) bleibt **unverändert** — darf
  die Zusage unterschreiten, Untergrenze ist Jahres-Ist ≥ 0.
- **D-57-02 — Referenz-Szenario.** Vertrag 5h bezahlt + 5h freiwillig/Woche,
  2 Wochen. Woche 1: 3h, Woche 2: 17h → 20h/20h. Heute: 8h bezahlt/12h
  freiwillig, Balance −2. Nach Übertrag von 2h: 10h/10h, Balance 0.
  Gegenprobe Übererfüllung: Woche 2 = 20h → 23h gesamt, Übertrag bleibt 2
  (Deckel), Balance 0.
- **D-57-03 — Rückwirkung.** Leseänderung wirkt sofort + rückwirkend, kein
  Stichtag-Guard. Unkritisch, da alle Rebooking-Rows aus 2026 stammen (kein
  Jahreswechsel dazwischen, `carryover_hours` erst zum Jahreswechsel
  geschrieben). Falls Verifikation Rows aus ≤2025 findet: Blocker, eskalieren.
- **D-57-04 — HR-Alert-Banner (D-55-07) bleibt UNVERÄNDERT.** Banner erlischt
  bei Approve UND Reject sowie beim belegten UNIQUE-Slot. Scope-Punkt 2 der
  ROADMAP ("D-55-07 entkoppeln") ist **gestrichen**. Die Rolle als Wächter
  gegen erneutes Versagen übernehmen die invertierten Tests (D-57-08).
- **D-57-05 — Produktionsstand.** Auto-Cron war nie aktiv
  (`voluntary_rebooking_auto_active_from` nie gesetzt, Backfill nie mit
  `dry_run=false` gelaufen). Keine `auto_cron`/`auto_cron_backfill`-Rows im
  Bestand. **Keine Bestandsdaten-Korrektur, keine Repair-Migration.** Der
  Planner MUSS das vor Umsetzung gegen die Ziel-DB verifizieren (siehe
  Verification-Query unten), nicht blind annehmen.
- **D-57-06 — Filter-Neufassung, site-spezifisch.** Drei Sites (`:1192`,
  `:1728`, `:2161`) den `source == Manual`-Filter ersatzlos streichen. Eine
  Site (`:859`, `assemble_weeks`) gezielt behandeln: nur Rebooking-getaggte
  `VolunteerWork`-Rows aus der `extra_working_hours`-Summe (`:876`) ausnehmen;
  in `manual_volunteer_hours` (`:932`) müssen sie drin bleiben.
  `ExtraHoursSource` bleibt als Provenienz-Marker erhalten.
- **RISK-01.** Alt-Inkonsistenz zwischen `assemble_weeks` (`availability()`)
  und `hours_per_week`/All-Employees (`as_report_type()`): gewöhnliche
  (Nicht-Rebooking-)VolunteerWork-Stunden zählen in der Wochenübersicht in die
  Balance, im Jahresreport nicht. Vorbestehend, **nicht Scope dieser Phase**.
  Der Eingriff an `:876` MUSS auf `source == Rebooking` beschränkt bleiben —
  nicht verschlimmern, nicht mitfixen.
- **D-57-07 — Balance-Deckel im Auto-Cron.** `run_for_week` liest neu die
  Balance **year-to-date bis einschließlich der verarbeiteten Woche,
  inklusive Vorjahres-Carryover** (nicht Laufzeitpunkt). `week_excess` wird um
  den Balance-Parameter erweitert:
  `min( max(ist − (soll + committed), 0), max(−balance, 0) )`, NaN/Inf-Guard
  bleibt. Optimierung (Balance einmal lesen + inkrementell fortschreiben)
  erlaubt, nicht gefordert.
- **D-57-08 — Tests invertieren, nicht löschen.** Neue Zusagen:
  `balance_after == balance_before + h`,
  `voluntary_ist_after == voluntary_ist_before − h`. `voluntary_soll` bleibt
  unverändert (bleibt inhaltlich Neutralitäts-Test). `assert_bit_eq` wird für
  die zwei invertierten Fälle durch Delta-Assertion mit f32-Toleranz ersetzt.
  Zusätzlich: Test für Balance-Deckel (Gegenprobe D-57-02, 23h → Übertrag
  bleibt 2).
- **D-57-09 — Drei REQ-Änderungen.** VOL-ACCT-03 (Neufassung: schützt nur
  noch die Freiwilligen-Statistik, VOL-ACCT-01-Soll unverändert durch
  Rebooking). VOL-ACCT-04 (neu: Ausgleichswirkung, Akzeptanz = D-57-02).
  REB-AUTO-06 (neu: Balance-Deckel im Auto-Cron). Traceability-Tabelle
  erweitern, Coverage danach 19/19.
- **D-57-10 — Snapshot-Bump 12 → 13** (`billing_period_report.rs:117`). D-01
  aus Phase 56 stützte sich auf Balance-Neutralität — entfällt.
- **D-57-11 — Straddling-Golden-Fixture invertieren, nicht löschen.** Muss
  jetzt beweisen: Periode MIT Rebooking-Woche verschiebt sich um `±h`.
  Perioden OHNE Rebooking-Woche bleiben byte-identisch (Regressions-
  Absicherung, unverändert).
- **D-57-12 — Bestehende v12-Snapshot-Rows bleiben unangetastet.** Kein
  Recompute, keine Migration. Validatoren vergleichen nur innerhalb
  derselben `snapshot_schema_version`.

### Claude's Discretion

Delegiert und in den Decisions oben bereits festgelegt: D-57-06 (Filter-
Sites), D-57-07 (Balance-Quelle im Cron), D-57-08 (Test-Zuschnitt), D-57-09
(REQ-IDs), D-57-10/11/12 (Snapshot). **Der Planner darf diese revidieren,
wenn die Recherche sie widerlegt — dann aber explizit mit Begründung im
PLAN.md.** Diese Recherche hat D-57-06/07/08 durch Code-Verifikation
bestätigt und um Implementierungsdetails ergänzt (siehe unten); keine
Widerlegung gefunden.

### Deferred Ideas (OUT OF SCOPE)

- Alt-Inkonsistenz `assemble_weeks` vs. `hours_per_week` (RISK-01) — eigener
  Fix, eigene Phase.
- Undo/Rollback von Rebooking-Batches (REB-UNDO-01, v2.7+).
- Repair-Werkzeug für zu große Auto-Cron-Paare — durch D-57-05 gegenstandslos.
- HR-Vorschlagsformel `min(|balance|, ist)` — bleibt unverändert.
- Entkopplung von D-55-07 — vom User explizit verworfen (D-57-04).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Beschreibung | Research-Unterstützung |
|----|--------------|------------------------|
| VOL-ACCT-03 (Neufassung) | Rebooking-Paare verändern VOL-ACCT-01-**Soll** nicht (Schutz der Freiwilligen-Statistik gegen Doppelzählung); Ist/Balance-Wirkung ist jetzt explizit **gewollt**, nicht mehr ausgeschlossen. | `committed_voluntary_target_in_range` (F2-Soll) liest **keine** ExtraHours — per Konstruktion rebooking-blind, unabhängig vom Filter-Fix (Sektion "F2-Soll ist strukturell blind" unten). Test `test_neutrality_f2_soll` bleibt unverändert grün. |
| VOL-ACCT-04 (neu) | Nach einer Umbuchung über `h`: Balance +`h`, Ehrenamt-Ist −`h`. Akzeptanz = D-57-02. | Sektion "Filter-Sites im Detail" (4 Sites) + "Code-Sketch: Filter-Fix" + Referenz-Szenario-Nachrechnung unten. |
| REB-AUTO-06 (neu) | Balance-Deckel im Auto-Cron: Balance steigt durch die Automatik nie über 0. | Sektion "Balance-Deckel: Datenfluss + Code-Sketch" + "Kritischer Fund: `backfill_range` braucht denselben Deckel". |
</phase_requirements>

## Summary

Der Fehler sitzt an genau vier Stellen in `service_impl/src/reporting.rs`
(Zeilen 859, 1192, 1728, 2161 — alle vier Zeilennummern gegen den aktuellen
Code verifiziert, keine Drift), die ein Rebooking-Paar `(−h VolunteerWork,
+h ExtraWork, source=Rebooking)` per `source == Manual`-Filter komplett aus
den Read-Aggregaten werfen. Der Fix ist **kein Full-Rewrite**, sondern das
Streichen dieses einen Filter-Prädikats an drei von vier Stellen — die
darunterliegende Kategorie-Logik (`ExtraHoursCategory::as_report_type()` /
`::availability()`) trennt "zählt in Balance" von "zählt als Ehrenamt-Ist"
bereits korrekt. Nur an der vierten Stelle (`assemble_weeks`, Zeile 859) ist
eine gezielte, kategorie- UND source-spezifische Ausnahme nötig, weil dort
eine strukturelle Alt-Inkonsistenz (RISK-01) sonst mitgefixt würde, was
außerhalb des Scopes liegt.

Der Balance-Deckel im Auto-Cron braucht **keine neue Service-Dependency** —
`RebookingReconciliationServiceImpl` hat `ReportingService` bereits als Dep
(genutzt von `suggest`/`approve`), und die DI-Reihenfolge in
`shifty_bin/src/main.rs` konstruiert `reporting_service` bereits vor
`rebooking_reconciliation_service`. Der Cron-Fix ist eine Erweiterung von
`week_excess` um einen vierten Parameter plus ein zusätzlicher
`get_report_for_employee_range`-Call in `run_for_week` — **UND** (kritischer,
in der Roadmap nicht genannter Fund) im `backfill_range`-Preview-Pfad, sonst
zeigt der Dry-Run-Vorschau-Wert eine falsche (ungedeckelte) Zahl an, die vom
tatsächlich gebuchten Wert abweicht.

Die Test-Landschaft ist günstiger als die Roadmap suggeriert: von den "fünf
grünen Tests" sind nur **drei** echte Bug-Zementierer, die eine
Verhaltens-Inversion brauchen (`test_neutrality_balance`,
`test_neutrality_f1_ist`, `rebooking_neutrality_single_manual`) — die
verbleibenden zwei (`test_neutrality_f2_soll`, `rebooking_neutrality_
roundtrip`) sind unter der korrigierten Logik weiterhin korrekt und bleiben
unverändert grün, weil Soll strukturell rebooking-blind ist bzw. ein
Roundtrip sich unabhängig von der Filter-Mechanik zu netto Null aufhebt.
Zusätzlich existiert in `run_for_week`s Testdatei bereits ein voll verdrahteter
`MockReportingService`-Mock (`reporting_service(balance: f32)`), der aktuell
ungenutzt mitläuft — die sieben bestehenden `run_for_week`-Testfälle bleiben
nach dem Fix ohne Änderung grün (nachgerechnet, Sektion unten), weil ihre
gemockte Balance (−8.0) in keinem Fall den Deckel bindet.

**Primäre Empfehlung:** Wave 1 = REQUIREMENTS.md-Neufassung (blockierend,
Spec vor Code). Wave 2 = Filter-Fix (`reporting.rs`, 4 Sites) + Balance-Deckel
(`rebooking_reconciliation.rs`, 2 Call-Sites) parallel, da sie unabhängige
Dateien berühren. Wave 3 = Tests invertieren + neue Tests (hängt an Wave 2).
Wave 4 = Snapshot-Bump + Straddling-Fixture + Docs-Freshness-Gate (hängt an
Wave 2, da die Bump-Begründung den fertigen Filter-Fix voraussetzt).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Requirements-Neufassung (VOL-ACCT-03/04, REB-AUTO-06) | Planning-Doc (`.planning/REQUIREMENTS.md`) | — | Kein Code, Spec-Änderung; blockiert alle Code-Waves. |
| Read-Aggregation-Filter (4 Sites) | API/Backend — `service_impl::reporting` (Business-Logic-Tier) | — | Reine Lese-Pfad-Korrektur in bestehenden pure/impure Aggregations-Funktionen, keine neue Schicht. |
| Balance-Deckel Auto-Cron | API/Backend — `service_impl::rebooking_reconciliation` (Business-Logic-Tier) | API/Backend — `service_impl::reporting` (liefert Balance) | `RebookingReconciliationService` bleibt der einzige Schreib-Pfad (Pitfall 15); `ReportingService` bleibt reiner Lese-Lieferant, keine neue Richtung im Service-Graph. |
| Snapshot-Versionierung | API/Backend — `service_impl::billing_period_report` | — | Konstante + Doc-Historie, kein neuer Konsument. |
| Tests | API/Backend — `service_impl::test::*` | — | Unit/Property-Tests auf Service-Ebene, kein E2E/Browser nötig (reine Zahlen-Invarianten). |
| Docs-Freshness (F14/F07/F08 + domain) | Dokumentation (`docs/`) | — | Kein Laufzeit-Tier; Pflicht-Gate laut CLAUDE.md, gleicher Commit wie Code-Diff. |
| Frontend (HR-Alert-Modal, Employee-Report-Zeilen) | — | — | **Kein FE-Code-Diff nötig.** Alle betroffenen DTO-Felder (`balance_hours`, `voluntary_ist`, `has_pending_rebooking`) werden bereits backend-seitig geliefert; sie werden nach dem Fix nur numerisch korrekt statt konstant. FE bleibt unverändert (Fat-Backend-Prinzip bestätigt sich hier positiv). |

## Standard Stack

Keine neuen Libraries. Diese Phase ändert ausschließlich vorhandenen
Rust-Code in bereits vorhandenen Crates (`service_impl`, `service`,
`.planning/REQUIREMENTS.md`, `docs/`). `mockall`, `tokio::test`, `time`,
`sqlx` (bestehend) reichen aus.

**Installation:** keine (`Cargo.toml` unverändert).

## Package Legitimacy Audit

**N/A — diese Phase installiert keine externen Pakete.** Keine
`Cargo.toml`-Änderung in Scope. Der Package-Legitimacy-Gate entfällt.

## Architecture Patterns

### Datenfluss-Diagramm (Read-Side-Fix)

```
Schreibseite (unverändert, Phasen 54-56):
  F3 rebook_manual / F4 run_for_week / F5 approve
        │
        ▼
  rebook() [einziger Pair-Write-Path, rebooking_reconciliation.rs:107]
        │  schreibt ATOMAR:
        │  ExtraHours{amount=-h, category=VolunteerWork, source=Rebooking}
        │  ExtraHours{amount=+h, category=ExtraWork,     source=Rebooking}
        ▼
  extra_hours (SQLite)

Leseseite (DIESE PHASE — 4 Sites in reporting.rs):
  extra_hours (alle Rows, Manual + Rebooking gemischt)
        │
        ├─► Site 1 assemble_weeks (:859/:876/:932)
        │     BUCKET-Filter (heute: source==Manual, betrifft ALLE Felder
        │     dieser Woche) → GEZIELT ändern: nur Rebooking-VolunteerWork
        │     aus extra_working_hours (:876) ausnehmen, in
        │     manual_volunteer_hours (:932) BLEIBEN lassen.
        │     → Weekly-Summary / HR-ALERT-Banner-Predicate
        │
        ├─► Site 2 get_reports_for_all_employees (:1192)
        │     BUCKET-Filter STREICHEN. Downstream nutzt as_report_type()
        │     (:1404/:1406) — trennt ExtraWork(WorkingHours) von
        │     VolunteerWork(Documented) bereits korrekt.
        │     → All-Employees-Report + HR-ALERT-Banner-Predicate
        │
        ├─► Site 3 overall_extra_work_hours (:1728)
        │     Filter STREICHEN (as_report_type()==WorkingHours bleibt).
        │     → EmployeeReport.balance_hours (Range-Report)
        │
        └─► Site 4 hours_per_week (:2161)
              Filter STREICHEN (as_report_type()/category-Filter
              downstream bleiben). Einzige Quelle für extra_work_hours
              UND volunteer_hours in GroupedReportHours (per-Woche).
              → EmployeeReport.by_week, .volunteer_hours, .balance_hours
                → VoluntaryStatsService.ist_total
                → BillingPeriodReportService (alle 4 Value-Types)
                → RebookingReconciliation.suggest()/.approve()/.run_for_week()
```

### Recommended Project Structure

Keine neuen Dateien. Geänderte/betroffene Pfade (alle bereits verifiziert):

```
.planning/REQUIREMENTS.md                              # VOL-ACCT-03 Neufassung + 2 neue REQs
service_impl/src/reporting.rs                           # 4 Filter-Sites (:859/:876/:932, :1192, :1728, :2161)
service_impl/src/rebooking_reconciliation.rs             # week_excess (4. Param), run_for_week, backfill_range
service_impl/src/billing_period_report.rs                # CURRENT_SNAPSHOT_SCHEMA_VERSION 12→13 + History-Kommentar
service_impl/src/test/rebooking_neutrality.rs             # 2 von 3 Tests invertieren
service_impl/src/test/rebooking_reconciliation.rs          # 1 von 2 Property-Tests invertieren
service_impl/src/test/voluntary_rebooking.rs              # neuer Deckel-Test (Gegenprobe D-57-02)
service_impl/src/test/rebooking_backfill.rs                # neuer Deckel-Test fuer backfill_range-Preview
service_impl/src/test/billing_period_straddling.rs          # Fixture erweitern: with-vs-without-Rebooking-Vergleich
docs/features/F14-rebooking.{md,_de.md}                   # §3/§7/§9.4 (Filter-Regel, Bump-Entscheidung)
docs/features/F07-reporting-balance.{md,_de.md}             # §2.6/§3 (source==Manual-Referenz veraltet)
docs/features/F08-billing-period.{md,_de.md}                # §7.4 Bump-History-Tabelle + v13-Zeile
docs/domain/time-accounting.md, billing-period.md           # Kategorie-Semantik / Snapshot-Rationale
```

### Pattern 1: Kategorie-Semantik trägt die Fachlogik, nicht der Source-Marker

**Was:** `ExtraHoursCategory::as_report_type()` (`ExtraWork→WorkingHours`,
`VolunteerWork→Documented`) und `::availability()`
(`ExtraWork→Available`, `VolunteerWork→Available`) entscheiden bereits, ob
eine Zeile in Balance (`WorkingHours`) bzw. Anwesenheit (`Available`) zählt.
Der `source`-Marker ist nur ein Filter, der VOR dieser Logik ansetzt und sie
für Rebooking-Rows komplett blockiert hatte.

**When to use:** Beim Entfernen der drei "einfachen" Filter-Sites (:1192,
:1728, :2161) reicht das Streichen der einen Zeile — keine Umstrukturierung
nötig, die Kategorie-Logik liefert das gewünschte Verhalten automatisch.

**Beleg (Site 2, `get_reports_for_all_employees`):**
```rust
// service_impl/src/reporting.rs:1404-1408 (unverändert, bereits korrekt)
let extra_working_hours = week_extra_hours
    .iter()
    .filter(|eh| eh.category.as_report_type() == ReportType::WorkingHours)
    .map(|eh| eh.amount)
    .sum::<f32>();
```
`ExtraWork.as_report_type() == WorkingHours` (zählt), `VolunteerWork
.as_report_type() == Documented` (zählt nicht) — die +h-Zeile zählt in die
Balance, die −h-Zeile nicht, OHNE dass der `source`-Filter das erzwingen muss.

### Pattern 2: Site 1 (`assemble_weeks`) ist ein Bucket-Filter, kein Feld-Filter

**Was:** Anders als an den drei anderen Sites filtert `:859` NICHT nur die
für die Balance relevante Teilmenge, sondern den **gesamten**
`extra_hours_bucket`, der danach für ALLE Felder dieser Person/Woche
wiederverwendet wird: `extra_working_hours` (:876), `abense_hours` (:884),
`vacation_hours`, `sick_leave_hours`, `holiday_hours`, `unavailable_hours`,
`unpaid_leave_hours`, `manual_volunteer_hours` (:932), `custom_absence_hours`.

**Warum das trotzdem sicher ist, den Filter NICHT komplett zu streichen:**
Rebooking-Rows tragen laut `rebook()` (`rebooking_reconciliation.rs:78-89`)
IMMER nur `VolunteerWork` oder `ExtraWork` als Kategorie — nie Vacation,
SickLeave, Holiday, Unavailable, UnpaidLeave oder CustomExtraHours. Ein
kompletter Wegfall des Filters an `:859` würde also NUR `extra_working_hours`
(via `availability()`, Zeile 879) und `manual_volunteer_hours` (via
`category==VolunteerWork`, Zeile 935) beeinflussen — alle anderen Felder sind
strukturell unberührt.

**Der Sonderfall:** `extra_working_hours` filtert hier über `availability()`
(`Available`), NICHT über `as_report_type()`. `VolunteerWork.availability()
== Available` (`extra_hours.rs:122`) — d.h. OHNE gezielte Ausnahme würde die
−h-VolunteerWork-Rebooking-Zeile HIER (anders als an den Sites 2-4) fälschlich
auch in `extra_working_hours` mitgezählt und die Balance-Anhebung wieder
neutralisieren. Deshalb: **gezielte Ausnahme nur für Rebooking-getaggte
VolunteerWork-Rows** bei `:876`, NICHT für gewöhnliche (Nicht-Rebooking)
VolunteerWork-Rows — sonst würde RISK-01 (die vorbestehende
Alt-Inkonsistenz) mitgefixt, was explizit außerhalb des Scopes liegt.

**Code-Sketch (Site 1):**
```rust
// :854-864 — Bucket-Load bleibt UNGEFILTERT nach source (nur Wochen-Filter):
let extra_hours_bucket = extra_hours
    .iter()
    .filter(|eh| {
        let sw = eh.to_date().as_shifty_week();
        sw.year == year && sw.week == week
    })
    .collect_to_hash_map_by(|eh| eh.sales_person_id);

// :876 — gezielte Ausnahme NUR fuer Rebooking-VolunteerWork:
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

// :932 — manual_volunteer_hours bleibt UNVERAENDERT (kein source-Filter
// noetig — Rebooking-VolunteerWork MUSS hier drin bleiben, damit das
// Ehrenamt-Ist sinkt):
let manual_volunteer_hours = employee_extra_hours
    .map(|eh| {
        eh.iter()
            .filter(|eh| eh.category == ExtraHoursCategory::VolunteerWork)
            .map(|eh| eh.amount)
            .sum::<f32>()
    })
    .unwrap_or(0.0);
```

### Pattern 3: `week_excess` als reine, vier-Parameter-Funktion

**Was:** `week_excess` (`rebooking_reconciliation.rs:63-73`) ist heute eine
reine 3-Parameter-Funktion (`ist, soll, committed_voluntary`). D-57-07
erweitert sie um `balance`.

**Code-Sketch:**
```rust
// service_impl/src/rebooking_reconciliation.rs:63
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

**Beide Call-Sites müssen angepasst werden** (siehe "Kritischer Fund" unten):
`run_for_week` (:618) UND `backfill_range` (:718).

### Pattern 4: Balance-Quelle — year-to-date, NICHT die Wochen-Range

**Kritische Unterscheidung, die im Code bereits als PRÄZEDENT für die FALSCHE
Variante existiert:** `suggest()` (:355-366) und `approve()` (:434-445) lesen
`report.balance_hours` bereits heute via `get_report_for_employee_range` —
aber mit `from_date=Montag(iso_week)`, `to_date=Sonntag(iso_week)`,
`include_carryover=false`. Das ist die Balance **isoliert für diese eine
Woche**, NICHT die kumulierte Jahres-Balance.

D-57-07 verlangt explizit **year-to-date bis einschließlich der
verarbeiteten Woche, inklusive Vorjahres-Carryover**. Das ist ein
STRUKTURELL ANDERER Range als der von `suggest`/`approve` — **der Planner
darf das `suggest`/`approve`-Muster NICHT kopieren**, sonst bindet der
Deckel an der falschen (zu kleinen/isolierten) Balance.

**Beleg für die korrekte Range-Semantik** (`get_report_for_employee_range`,
`reporting.rs:1804-1805`):
```rust
balance_hours: shiftplan_hours_by_week + overall_extra_work_hours - planned_hours
    + previous_year_carryover,   // NUR wenn include_carryover == true
```
Mit `from_date = ShiftyDate::first_day_in_year(iso_year)`,
`to_date = Sonntag(iso_week)`, `include_carryover = true` ergibt das exakt
"kumulierte Balance zum Ende der verarbeiteten Woche, inkl. Vorjahres-Carryover".

**Code-Sketch (`run_for_week`, nach dem `week_status`-Check, vor dem
`week_voluntary_figures`-Call, `:610` ff.):**
```rust
let year_start = ShiftyDate::first_day_in_year(iso_year);
let week_end = ShiftyWeek::new(iso_year, iso_week).as_date(DayOfWeek::Sunday);
let report = self
    .reporting_service
    .get_report_for_employee_range(
        &sales_person_id,
        year_start,
        week_end,
        true, // include_carryover — D-57-07
        Authentication::Full,
        Some(tx.clone()),
    )
    .await?;
let balance = report.balance_hours;

let (ist, _soll, committed) = self
    .week_voluntary_figures(sales_person_id, iso_year, iso_week, tx.clone())
    .await?;
let excess = week_excess(ist, 0.0, committed, balance);
```
`ShiftyDate` und `ShiftyWeek` sind in `rebooking_reconciliation.rs` bereits
importiert (`use shifty_utils::{DayOfWeek, ShiftyDate, ShiftyWeek};`) — keine
neue Dependency.

**Multi-Wochen-Backfill-Konsistenz (bereits gegeben, kein Zusatzaufwand):**
`backfill_range` öffnet EINE `read_tx` für den gesamten Wochen-Loop
(`:697`) und reicht sie unverändert an jeden `run_for_week`-Aufruf weiter
(`:763`). SQLite sieht innerhalb derselben Transaktion die zuvor in derselben
Transaktion geschriebenen Rows — die Balance-Abfrage für Woche N sieht damit
automatisch die in Woche N-1 bereits gebuchten Rebooking-Zeilen, ohne dass
der Planner selbst Buch führen muss. Der "einfache Weg" (D-57-07,
Optimierung erlaubt-nicht-gefordert) ist damit bereits korrekt.

## Kritischer Fund: `backfill_range` braucht denselben Deckel (nicht in ROADMAP genannt)

`week_excess` wird an **zwei** Stellen aufgerufen, nicht nur in
`run_for_week`:

1. `run_for_week` (`:618`) — der eigentliche Schreib-Pfad.
2. `backfill_range` (`:715-718`) — berechnet **selbstständig** `excess` für
   das `BackfillPreviewItem`, das im `dry_run`-Fall NICHT über
   `run_for_week` läuft, sondern direkt in die REST-Antwort geht.

Der Doc-Kommentar auf `BackfillPreviewItem.excess`
(`service/src/rebooking_reconciliation.rs:115`) sagt wörtlich: *"der
Ueberschuss, der gebucht wuerde"* — das ist ein Korrektheits-Versprechen an
die REST-Antwort/HR-UI. Wenn `backfill_range` nach dem Fix weiter die
alte 3-Parameter-`week_excess`-Formel nutzt (oder die 4-Parameter-Variante
mit `balance=0.0`/ungeprüft aufruft), zeigt der **Dry-Run-Vorschau-Wert**
einen zu hohen, ungedeckelten Betrag — der beim tatsächlichen `apply`-Lauf
(der über `run_for_week` läuft und den Deckel korrekt anwendet) NICHT gebucht
wird. Das wäre ein neuer, in dieser Phase eingeführter Bug (falsche
HR-Preview), obwohl der eigentliche Schreib-Pfad korrekt ist.

**Konsequenz für den Plan:** `backfill_range` braucht VOR der
`week_excess`-Berechnung (`:718`) denselben Balance-Read wie `run_for_week`
(year-to-date bis Ende der jeweiligen Wochen-Iteration, `read_tx.clone()`).
Das ist derselbe Code-Sketch wie oben, eingefügt in die
`for wd in contracts.iter()`-Schleife vor Zeile 718. Zusätzlicher
`get_report_for_employee_range`-Call pro (SalesPerson, Woche) im Preview-Pfad
— Performance-Impact ist vertretbar, da Backfill ein seltener Admin-Trigger
ist (kein Hot-Path) und D-57-05 belegt, dass es noch keine
Bestandsdaten/Nutzungsdruck gibt.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Kumulierte Jahres-Balance zu einem Stichtag | Eigene Summierung über `extra_hours`/`shiftplan_report` | `ReportingService::get_report_for_employee_range(sales_person_id, first_day_in_year, week_end, true, …)` | Bereits die Single-Source-of-Truth für Balance inkl. Carryover, Cap-Logik, Absence-Derivation; jede Neuimplementierung würde eine der ~6 Sonderregeln (Cap, dynamischer Vertrag, No-Contract-Ehrenamt, Absence-Derivation, Holiday-Auto-Credit) vergessen. |
| Balance-Deckel-Formel | Neue eigenständige Cap-Funktion außerhalb von `week_excess` | Erweiterung der bestehenden reinen `week_excess`-fn um einen 4. Parameter | `week_excess` ist bereits pure + getestet (`week_excess_matrix`); ein Parallel-Pfad würde zwei Wahrheiten über "was ist der buchbare Überschuss" schaffen. |
| Bug-Sichtbarkeit / Regressions-Schutz | Neue UI-Warnung oder zusätzlichen manuellen Check | Invertierte Property-Tests (D-57-08) als CI-Gate | Der ursprüngliche Fehler hat ein ganzes Milestone überlebt, WEIL ein grüner Test (Bit-Identität) ihn zementiert hat — die Lehre aus dieser Phase ist Test-als-Wächter, nicht UI-als-Wächter (User-Entscheidung D-57-04 explizit bestätigt). |

**Key insight:** Die gesamte Fachlogik (Kategorie→Report-Type-Mapping,
Cap-Overflow, Carryover, Absence-Derivation) existiert bereits korrekt
implementiert und getestet — dieser Bug ist ein reiner **Filter-Fehler auf
Lese-Seite**, kein fehlendes Feature. Jede Versuchung, "während wir eh
dabei sind" Teile der Balance-Berechnung neu zu schreiben, vergrößert den
Diff unnötig und erhöht das Risiko, RISK-01 versehentlich mitzufixen.

## Common Pitfalls

### Pitfall 1: Site 1 (`assemble_weeks`) pauschal wie die anderen drei Sites behandeln

**Was schiefgeht:** Der `source == Manual`-Filter komplett aus `:859`
streichen (wie an :1192/:1728/:2161).
**Warum es passiert:** Die vier Sites sehen syntaktisch identisch aus
(`.filter(|eh| eh.source == ExtraHoursSource::Manual)`), man kopiert den
gleichen Fix viermal.
**Wie man es vermeidet:** `:876` filtert über `availability()`, nicht
`as_report_type()` — `VolunteerWork.availability() == Available` heißt: ohne
gezielte Ausnahme zieht die -h-Zeile die Balance wieder herunter, netto 0
Effekt, der Bug bleibt live (nur an dieser einen Stelle). Siehe "Pattern 2"
oben für den exakten Code-Sketch.
**Warning signs:** Der neue Test `test_weekly_summary_rebooking_effective`
(oder analog) zeigt Balance-Delta 0 statt +h in der Wochenansicht, während
der Jahresreport bereits korrekt +h zeigt.

### Pitfall 2: `backfill_range`s Preview-Excess nicht mit dem Deckel synchronisieren

Siehe "Kritischer Fund" oben — eigener Abschnitt, weil es kein
Kopier-Fehler, sondern ein vom Roadmap-Text nicht genannter zweiter
Call-Site ist, den man beim reinen "week_excess bekommt einen 4. Parameter"-
Refactor leicht übersieht (Compiler zwingt zur Anpassung der Signatur an
beiden Stellen, aber NICHT dazu, an beiden Stellen auch die Balance korrekt
zu beschaffen — ein `balance: 0.0`-Platzhalter kompiliert klaglos und deckelt
fälschlich JEDEN Überschuss auf 0).

### Pitfall 3: Balance-Range von `suggest`/`approve` kopieren

Siehe "Pattern 4" oben. `suggest`/`approve` nutzen eine isolierte
Wochen-Range OHNE Carryover — für den HR-Vorschlag korrekt (der bewusst
gedeckelt auf `min(|balance_isolated_week|, ist)` reagiert), für den
Auto-Cron-Deckel (D-57-07: "year-to-date … inklusive Vorjahres-Carryover")
aber falsch. Copy-Paste des bestehenden `suggest`-Patterns wäre der
naheliegendste, aber falsche Reflex.

### Pitfall 4: Alle "fünf grünen Tests" gleich behandeln

**Was schiefgeht:** Man invertiert reflexhaft alle fünf in D-57-08 genannten
Tests inklusive `test_neutrality_f2_soll` und `rebooking_neutrality_roundtrip`.
**Warum es falsch wäre:** `test_neutrality_f2_soll` testet
`VoluntaryStatsService::soll_total`, das ausschließlich über
`committed_voluntary_target_in_range` (pure fn, liest NUR
`EmployeeWorkDetails` + `AbsencePeriod`, KEINE `ExtraHours`) berechnet wird —
strukturell rebooking-blind, unabhängig vom Filter-Fix. `rebooking_
neutrality_roundtrip` testet zwei Pairs (hin + zurück, netto Null in beiden
Kategorien) — das bleibt unter JEDER Filter-Logik neutral, weil sich die
Beträge algebraisch aufheben, nicht weil ein Filter sie versteckt.
**Wie man es vermeidet:** Nur `test_neutrality_balance`,
`test_neutrality_f1_ist` (`rebooking_neutrality.rs`) und
`rebooking_neutrality_single_manual` (`rebooking_reconciliation.rs`)
invertieren. Details + exakte Zeilen siehe "Validation Architecture" unten.

### Pitfall 5: `EmployeeReport.volunteer_hours == 0.0` Edge-Case im Snapshot

**Was schiefgeht:** Der `BillingPeriodValueType::Volunteer`-Eintrag wird nur
geschrieben, wenn `report_delta.volunteer_hours != 0.0`
(`billing_period_report.rs:283`). Wenn ein Test-Fixture das Rebooking so
wählt, dass sich Baseline-VolunteerWork und Rebooking-Delta exakt aufheben
(`volunteer_hours == 0.0`), verschwindet der `Volunteer`-Key komplett aus der
Snapshot-Map — `assert_snapshot_bit_eq`s Key-Set-Vergleich
(`billing_period_straddling.rs:84-87`) schlägt dann mit einer irreführenden
"key set differs"-Meldung fehl statt mit einer Werte-Abweichung.
**Wie man es vermeidet:** Für die D-57-11-Fixture einen `h`-Wert wählen, der
die Baseline-VolunteerWork-Summe NICHT exakt neutralisiert (die bestehende
Fixture mit Baseline=3.0 und Rebooking-Delta=−2.0 → Ergebnis 1.0 ≠ 0 ist
bereits sicher — beibehalten, nicht durch einen anderen Wert ersetzen).

## Balance-Deckel: Nachrechnung des Referenz-Szenarios (D-57-02) gegen die Formel

Damit der Planner die exakten erwarteten Testwerte hat (die Roadmap-Prosa
"Überschuss 5" bezieht sich NICHT auf denselben Wert wie `week_excess`s
Eingabe — siehe unten):

**Basis-Szenario:** Vertrag 5h bezahlt + 5h freiwillig/Woche (2 ISO-Wochen,
`cap_planned_hours_to_expected=true`). Woche 1: Shiftplan 3h gearbeitet.
Woche 2: Shiftplan 17h gearbeitet.

- Woche 1: `raw_shiftplan=3 < expected=5` → kein Cap-Überlauf →
  `shiftplan_paid=3`, `auto_volunteer=0`.
- Woche 2: `raw_shiftplan=17 > expected=5` → Cap greift →
  `shiftplan_paid=5`, `auto_volunteer=12`.
- Kumuliert: bezahlt 3+5=8h, Ehrenamt(auto) 0+12=12h. **Balance (2 Wochen) =
  8 − 10 = −2.** (Deckt sich exakt mit D-57-02s "8h bezahlt/12h
  freiwillig, Balance −2".)

**`run_for_week` für Woche 2 (Auto-Cron), NACH dem Fix:**
- `ist` = `VoluntaryStats::ist_total` für die Wochen-Range **Woche 2 allein**
  = 12h (nur der Cap-Überlauf dieser Woche; Woche 1 hatte 0 Überlauf).
- `committed` = committed_voluntary DIESER Woche = 5h.
- `week_excess` unconstrained = `12 − (0 + 5)` = 7h.
- `balance` = year-to-date-Balance BIS EINSCHLIESSLICH Woche 2, VOR dem
  Rebooking = **−2h** (siehe oben, kumuliert über beide Wochen).
- `headroom = max(-(-2), 0) = 2`.
- **Gebucht: `min(7, 2) = 2h`.** → Balance danach = −2+2 = **0**. ✅ deckt
  sich exakt mit D-57-02 ("Nach korrektem Übertrag von 2h: 10h/10h, Balance 0").

**Gegenprobe Übererfüllung (Woche 2 = 20h statt 17h):**
- Woche 2: `raw_shiftplan=20 > expected=5` → `shiftplan_paid=5`
  (unverändert, Cap greift bei 5h), `auto_volunteer=15`.
- Kumuliert bezahlt bleibt 3+5=8h (Cap ändert sich nicht) → **Balance
  bleibt −2h**, unabhängig vom höheren Überlauf.
- `ist`(Woche 2 allein)=15h, `committed`=5h → unconstrained excess = 10h.
- `headroom = max(2, 0) = 2` (Balance unverändert bei −2).
- **Gebucht: `min(10, 2) = 2h`.** Balance danach = 0. ✅ deckt sich mit
  "Übertrag trotzdem nur 2 (Deckel), Balance 0".

**Wichtiger Hinweis zur Roadmap-Formulierung "Überschuss 5":** Dieser Wert
ist NICHT `week_excess`s unconstrained Eingabewert (der ist 10, nicht 5) —
er entspricht der PERIODEN-Delta-Ansicht (`VOL-ACCT-01-Delta = Ist − Soll`
über beide Wochen: `ist_total(15) − soll_total(10) = 5`). Beide Zahlen sind
im Fachmodell konsistent, beziehen sich aber auf unterschiedliche Ranges
(kumulierte 2-Wochen-Periode vs. Auto-Crons Einzelwochen-Input). Der Planner
sollte in Tests/PLAN.md explizit dokumentieren, welche der beiden Zahlen an
welcher Stelle gemeint ist, um Verwechslung beim Review zu vermeiden.

## Runtime State Inventory

> Trigger: Diese Phase korrigiert einen Lese-Filter rückwirkend (D-57-03) —
> kein Rename/Refactor im klassischen Sinn, aber mit Bestandsdaten-Wirkung.
> Deshalb vollständig beantwortet.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | `extra_hours`-Rows mit `source='rebooking'` aus Phase 55 (manuelle F3-Umbuchungen seit 2026-07-24) und ggf. approvte F5-HR-Vorschläge. **Verification-Query vor Umsetzung:** `SELECT kind, COUNT(*) FROM rebooking_batch WHERE deleted IS NULL GROUP BY kind;` — erwartet laut D-57-05 NUR `manual` und optional `hr_suggestion`, NIE `auto_cron`/`auto_cron_backfill`. | Kein Daten-Rewrite nötig (D-57-03: Leseänderung wirkt automatisch rückwirkend korrekt, sobald der Filter fällt). Falls die Query dennoch `auto_cron`/`auto_cron_backfill`-Rows findet: **Blocker, eskalieren** (D-57-05 widerlegt) — NICHT stillschweigend weiterplanen. |
| Live service config | `voluntary_rebooking_auto_active_from`-Toggle (DB-Wert, kein Git-Artefakt). Laut D-57-05 User-bestätigt `None` (nie gesetzt). | Kein Handlungsbedarf in dieser Phase — Toggle-Semantik unverändert (steuert nur die Schreib-Seite F4, nicht die jetzt korrigierte Lese-Seite). |
| OS-registered state | Keine — der Cron läuft als in-process `tokio-cron-scheduler`-Task (`VoluntaryRebookingScheduler`), kein OS-Task/systemd-Unit mit Rebooking-spezifischem Namen. | Keine. |
| Secrets/env vars | Keine Rebooking-spezifischen Secrets/Env-Vars. | Keine. |
| Build artifacts | Keine — reiner Rust-Workspace-Code, kein separates Paket/Egg-Info betroffen. | `cargo build`/`cargo test` reicht; kein `reinstall`-Schritt nötig. |

**Bestätigt "Nichts gefunden":** OS-registered state, Secrets/Env, Build
Artifacts — alle drei explizit verifiziert (grep über Migrationen/Main/CI,
keine Treffer außerhalb der bereits gelisteten Code-Pfade).

## Code Examples

### Bereits vorhandene, wiederverwendbare Test-Harness (kein neues Muster nötig)

`service_impl/src/test/rebooking_neutrality.rs` UND
`service_impl/src/test/billing_period_straddling.rs` bauen bereits ein
**reales** `ReportingServiceImpl` mit gemocktem `ExtraHoursService` auf
(`build_reporting_service(extra_hours: Vec<ExtraHours>)`), inkl. Fixture
`work_details()` (5h/Woche, `cap_planned_hours_to_expected=true`,
`committed_voluntary=5.0`). Für das Referenz-Szenario (D-57-02, 2-Wochen
shiftplan-basiert statt direkt-per-ExtraHours) muss zusätzlich
`MockShiftplanReportService::expect_extract_shiftplan_report()` echte
`ShiftplanReportDay`-Einträge liefern (eine Row mit `hours=3.0` für KW1, eine
mit `hours=17.0`/`20.0` für KW2 reicht — `hours` wird pro Woche einfach
summiert, keine Tages-Granularität nötig):

```rust
// Source: service_impl/src/test/rebooking_neutrality.rs:176-244 (bestehendes Muster)
// + service/src/shiftplan_report.rs:12-18 (ShiftplanReportDay-Shape)
let mut shiftplan_report_service = MockShiftplanReportService::new();
shiftplan_report_service
    .expect_extract_shiftplan_report()
    .returning(|_, _, _, _, _| Ok(Arc::from(vec![
        ShiftplanReportDay { sales_person_id: sp_id(), hours: 3.0, year: 2026, calendar_week: 10, day_of_week: DayOfWeek::Monday },
        ShiftplanReportDay { sales_person_id: sp_id(), hours: 17.0, year: 2026, calendar_week: 11, day_of_week: DayOfWeek::Monday },
    ])));
```

### `run_for_week`-Test-Harness bereits Balance-fähig

`service_impl/src/test/voluntary_rebooking.rs:162-167` hat bereits einen
`reporting_service(balance: f32) -> MockReportingService`-Helper, den ALLE
sieben bestehenden `run_for_week`-Testfälle unverändert übergeben
(`reporting_service(-8.0)`). Da `mockall`s `.returning()` ohne `.times()`
beliebig oft (auch 0×) aufgerufen werden darf, brauchen die 7 Bestandstests
**keine Änderung** — der Fix fügt lediglich einen tatsächlichen Aufruf
hinzu, den der Mock bereits beantwortet. Nachgerechnet (siehe Sektion
"Validation Architecture" für die vollständige Tabelle): in allen 7 Fällen
bindet der Deckel nicht (Balance −8 ist weit unter dem jeweiligen
unconstrained Excess), Ergebnisse bleiben identisch.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|---------------|--------|
| `source == Manual`-Filter an 4 Read-Aggregations-Sites, macht Rebooking-Paare unsichtbar | Kategorie-basierte Filterung (`as_report_type()`/`availability()`), `source` nur noch Provenienz-Marker | Diese Phase (57) | Balance/Ehrenamt-Ist reagieren erstmals wie im Milestone-Goal vorgesehen; HR-ALERT-Banner-Predicate + `suggest()`s `balance_after`/`voluntary_ist_after`-Versprechen werden erstmals real erfüllt (kein Zusatz-Code nötig — automatischer Nebeneffekt des Filter-Fixes). |
| `week_excess(ist, soll, committed)` — Balance-blind | `week_excess(ist, soll, committed, balance)` — Balance-Deckel | Diese Phase (57) | Auto-Cron kann Balance nicht mehr über 0 heben (REB-AUTO-06); HR-Pfad bleibt bewusst unverändert (kann Zusage unterschreiten). |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12` (Phase 56 D-01: "Rebooking leakt nicht in Snapshots") | `= 13` | Diese Phase (57) | Validatoren müssen v12-Snapshots als "älteres Schema" behandeln und Balance/ExtraWork/Volunteer NICHT gegen die neue Computation revalidieren (Muster identisch zu v8-v12 Vorgängern in derselben History-Kommentar-Kette). |

**Deprecated/veraltet:** Die drei Neutralitäts-Tests, die Bit-Identität als
Erfolgskriterium für ein EINZELNES Rebooking-Pair fordern, sind das
Gegenteil des jetzt korrekten Verhaltens — sie werden invertiert, nicht als
"alte Variante" archiviert (D-57-08: invertieren, nicht löschen — Git-History
bleibt die Archivierung).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|----------------|
| — | *(keine)* | — | — |

**Diese Tabelle ist leer.** Jede Tatsachenbehauptung in diesem Dokument
wurde per `Read`/`grep` gegen den aktuellen Code, gegen
`.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `57-CONTEXT.md` und
`57-DISCUSSION-LOG.md` verifiziert — keine WebSearch-/Trainingsdaten-Claims,
da diese Phase keine externen Libraries oder Framework-Fragen berührt. Wo
Zeilennummern zitiert werden, wurden sie in dieser Session frisch gelesen
(nicht aus der Roadmap übernommen) und stimmen mit dem aktuellen
Arbeitsstand überein.

## Open Questions

1. **Exakter Deckel-Test-Ort für `backfill_range`-Preview**
   - Was wir wissen: `backfill_range` braucht denselben Balance-Read wie
     `run_for_week` vor der `week_excess`-Berechnung (Sektion "Kritischer
     Fund").
   - Was unklar ist: Ob der Planner dafür eine eigene Test-Datei
     (`rebooking_backfill.rs`, bereits vorhanden mit anderen
     Backfill-Tests) erweitert oder einen neuen Abschnitt anlegt.
   - Empfehlung: In `service_impl/src/test/rebooking_backfill.rs`
     erweitern (Datei existiert bereits, siehe "Recommended Project
     Structure"), Pattern von `voluntary_rebooking.rs`s `reporting_service()`
     -Helper spiegeln.

2. **Exakte Formulierung der REQUIREMENTS.md-Bump-Historie für VOL-ACCT-03**
   - Was wir wissen: D-57-09 verlangt "kurzer Historien-Hinweis statt
     ⚠️-Fehlervermerk, nicht gelöscht — die Fehlerspur bleibt auffindbar".
   - Was unklar ist: Ob der Planner den bestehenden Text als Git-History
     (Commit-Referenz) verlinkt oder inline eine Kurzfassung belässt.
   - Empfehlung: Claude's Discretion beim Planner — kein Blocker, reine
     Formulierungsfrage ohne Code-Auswirkung.

## Environment Availability

> Übersprungen — diese Phase hat keine externen Laufzeit-Dependencies
> (keine neuen Services, Datenbanken, CLIs). Bestehende Umgebung
> (`cargo`, `sqlx-cli` via `nix develop`, SQLite) ist durch alle
> Vorgänger-Phasen (54-56) bereits als funktionsfähig verifiziert.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in Test-Harness + `tokio::test` (async) + `mockall` (Service-Mocks) |
| Config file | keins (Cargo-Workspace-Standard) |
| Quick run command | `cargo test -p service_impl rebooking` (alle Rebooking-relevanten Modul-Tests) |
| Full suite command | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |

### Bestandsaufnahme: die "fünf grünen Tests" im Detail

| Test | Datei | Heutige Assertion | Nach Fix | Aktion |
|------|-------|---------------------|----------|--------|
| `test_neutrality_balance` | `test/rebooking_neutrality.rs:333` | `assert_bit_eq(bal_before, bal_after)` | `balance_after == balance_before + h` (h=2.0 im Fixture) | **INVERTIEREN** (Delta-Assertion, `(a-b).abs() < 1e-3`) |
| `test_neutrality_f1_ist` | `test/rebooking_neutrality.rs:343` | `assert_bit_eq(ist_before, ist_after)` | `ist_after == ist_before - h` | **INVERTIEREN** |
| `test_neutrality_f2_soll` | `test/rebooking_neutrality.rs:354` | `assert_bit_eq(soll_before, soll_after)` | unverändert (Soll ist rebooking-blind, siehe Pitfall 4) | **UNVERÄNDERT** |
| `rebooking_neutrality_single_manual` | `test/rebooking_reconciliation.rs:245` | `(bal_before-bal_after).abs()<1e-3`, `(vol_before-vol_after).abs()<1e-3` | Balance +h, Volunteer −h | **INVERTIEREN** (Epsilon-Pattern bereits vorhanden, nur Zielwerte ändern) |
| `rebooking_neutrality_roundtrip` | `test/rebooking_reconciliation.rs:262` | beide Aggregate neutral (2 Pairs, netto 0) | unverändert — netto-Null-Eigenschaft ist filter-unabhängig | **UNVERÄNDERT** (siehe Pitfall 4) |

**Neue Tests (D-57-08 zusätzlich + REB-AUTO-06):**

| Neuer Test | Ort | Zweck |
|------------|-----|-------|
| Balance-Deckel bindet (Gegenprobe D-57-02: ist=15, committed=5, balance=−2 → gebucht 2, nicht 10) | `test/voluntary_rebooking.rs` (erweitert `week_excess_matrix` + neuer `run_for_week`-Case) | REB-AUTO-06 Kern-Invariante |
| Balance-Deckel bindet NICHT, wenn Balance bereits ≥ 0 (kein Übertrag trotz Excess) | `test/voluntary_rebooking.rs` | Randfall aus Research-Schwerpunkt F |
| `backfill_range`-Preview zeigt gedeckelten Wert, nicht den unconstrained Wert | `test/rebooking_backfill.rs` | Kritischer Fund (siehe oben) |
| Weekly-Summary (`assemble_weeks`/Site 1) zeigt Balance-Wirkung — nicht nur der Jahresreport | `service_impl/src/test/` (neuer oder erweiterter Reporting-Test, Fixture-Pattern aus `rebooking_neutrality.rs`) | Pitfall 1 — stellt sicher, dass Site 1 NICHT übersehen wurde |
| Straddling-Fixture: with-Rebooking vs. without-Rebooking zeigt `±h`-Delta auf Balance/ExtraWork/Volunteer, alle anderen `value_type`s unverändert | `test/billing_period_straddling.rs` (erweitert) | D-57-11 |
| Straddling-Fixture: Toggle-Unabhängigkeit bleibt erhalten (pre/post/straddling weiterhin gegenseitig byte-identisch, sowohl mit als auch ohne Rebooking-Pair) | `test/billing_period_straddling.rs` (bestehende 3 Tests, unverändert lassen) | Regressions-Absicherung (D-57-11 "Perioden ohne Rebooking-Woche bleiben byte-identisch") |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|--------------------|--------------|
| VOL-ACCT-03 | Rebooking ändert VOL-ACCT-01-Soll nicht | unit | `cargo test -p service_impl test_neutrality_f2_soll` | ✅ (unverändert) |
| VOL-ACCT-04 | Balance +h, Ehrenamt-Ist −h nach Umbuchung | unit/property | `cargo test -p service_impl test_neutrality_balance test_neutrality_f1_ist rebooking_neutrality_single_manual` | ✅ (invertiert) |
| VOL-ACCT-04 | Referenz-Szenario D-57-02 end-to-end (2-Wochen shiftplan-basiert) | unit | neuer Test in `test/rebooking_neutrality.rs` oder eigenes Modul | ❌ Wave 3 |
| REB-AUTO-06 | Balance-Deckel bindet, Balance steigt nie über 0 | unit | `cargo test -p service_impl run_for_week` (neuer Case) | ❌ Wave 2/3 |
| REB-AUTO-06 | `backfill_range`-Preview konsistent mit Deckel | unit | `cargo test -p service_impl -- rebooking_backfill` (erweitert) | ❌ Wave 2/3 |
| — (Snapshot-Konsistenz, CLAUDE.md-Gate) | Bump 12→13 korrekt begründet + Fixture beweist `±h` | unit | `cargo test -p service_impl billing_period_straddling` | ⚠️ Datei existiert, Fixture muss erweitert werden — Wave 4 |

### Sampling Rate

- **Per task commit:** `cargo test -p service_impl rebooking` + `cargo clippy --workspace -- -D warnings` (Clippy ist hartes Gate laut CLAUDE.md, wird von `cargo test` NICHT abgedeckt).
- **Per wave merge:** `cargo test --workspace`.
- **Phase gate:** `cargo test --workspace && cargo clippy --workspace -- -D warnings` grün vor `/gsd-verify-work`.

### Wave 0 Gaps

Keine — alle benötigten Test-Dateien existieren bereits
(`rebooking_neutrality.rs`, `rebooking_reconciliation.rs`,
`voluntary_rebooking.rs`, `rebooking_backfill.rs`,
`billing_period_straddling.rs`), alle benötigten Mock-Harnesses
(`MockReportingService`, `MockExtraHoursService`,
`MockShiftplanReportService`) sind bereits im Workspace verfügbar. Kein
neues Test-Framework, keine neue Fixture-Infrastruktur nötig.

## Security Domain

> `security_enforcement` nicht explizit `false` in `.planning/config.json` →
> Sektion Pflicht, aber diese Phase hat keinen neuen Angriffs-Vektor.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|----------------|---------|-------------------|
| V2 Authentication | nein | Unverändert — kein neuer Auth-Pfad. |
| V3 Session Management | nein | Unverändert. |
| V4 Access Control | ja (unverändert) | `HR_PRIVILEGE`-Gate bleibt an allen berührten Service-Methoden ZUERST geprüft (`check_permission` vor jedem Datenzugriff) — bestehendes Muster, keine Änderung an der Gate-Reihenfolge in dieser Phase. |
| V5 Input Validation | nein | Keine neuen Eingabefelder/Endpoints. |
| V6 Cryptography | nein | Unberührt. |

### Known Threat Patterns for {stack}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|-----------------------|
| Balance-Deckel-Formel liefert negative/NaN-Werte bei korrupten Eingabedaten | Tampering (indirekt, Datenintegrität) | `week_excess`s bestehender NaN/Inf-Guard bleibt erhalten und wird auf den 4. Parameter (`balance`) erweitert — verhindert, dass ein korrupter Report (z.B. NaN-Balance durch einen Edge-Case in `derive_hours_for_range`) zu einer Panik oder einer falschen (negativen) Buchung führt. |
| Info-Disclosure über invertierte Test-Assertions in Logs | — | N/A — reine Test-Code-Änderung, keine Produktions-Log-Änderung. |

Kein neuer Endpoint, kein neues DTO-Feld, keine neue Persistenzschicht in
dieser Phase — die Angriffsfläche ist identisch zu Phase 54-56.

## Sources

### Primary (HIGH confidence — Code-verifiziert in dieser Session)
- `service_impl/src/reporting.rs` (2860+ Zeilen gelesen: Zeilen 1-50, 180-420,
  830-980, 1134-1560, 1610-1842, 2139-2410) — alle 4 Filter-Sites, `EmployeeReport`-
  Assembly, `hours_per_week`, `alert_predicate`/`proposed_hours`.
- `service/src/extra_hours.rs` (vollständig) — `ExtraHoursCategory::as_report_type()`/
  `::availability()`, `ExtraHoursSource`.
- `service_impl/src/rebooking_reconciliation.rs` (vollständig) — `week_excess`,
  `rebook()`, `run_for_week`, `backfill_range`, bestehende `ReportingService`-Dependency.
- `service_impl/src/billing_period_report.rs` (Zeilen 1-330) — Snapshot-Version-
  Historie-Kommentar-Muster, Value-Type-Assembly.
- `service_impl/src/shiftplan_edit.rs:360-399` — Carryover-Schreibpfad (D-57-03-Beleg).
- `service_impl/src/test/rebooking_neutrality.rs`,
  `service_impl/src/test/rebooking_reconciliation.rs`,
  `service_impl/src/test/voluntary_rebooking.rs`,
  `service_impl/src/test/billing_period_straddling.rs` (vollständig gelesen) —
  exakte Test-Assertions, Mock-Harnesses, bereits vorhandene `reporting_service()`-Helper.
- `dao_impl_sqlite/src/rebooking_batch.rs` — `RebookingBatchKind`-String-Mapping
  (`auto_cron`/`auto_cron_backfill`) für die D-57-05-Verification-Query.
- `shifty_bin/src/main.rs` (grep auf DI-Reihenfolge) — bestätigt `reporting_service`
  vor `rebooking_reconciliation_service` konstruiert.
- `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md` (§Phase 57), `57-CONTEXT.md`,
  `57-DISCUSSION-LOG.md`, `.planning/STATE.md`.
- `CLAUDE.md` (Projekt-Root) — Snapshot-Versionierungs-Klausel, Service-Tier-
  Konventionen, Docs-Freshness-Gate-Trigger-Tabelle, Clippy-Hard-Gate.

### Secondary / Tertiary
- Keine — diese Phase erforderte keine externe Web-/Doku-Recherche (kein neues
  Framework, keine neue Library, keine externe API).

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — keine neuen Dependencies, alles bestehender Workspace-Code.
- Architecture: HIGH — Datenfluss + alle 4 Filter-Sites + Balance-Deckel-Call-Sites
  gegen aktuellen Code gelesen und nachgerechnet (Referenz-Szenario stimmt exakt).
- Pitfalls: HIGH — Pitfall 1/2/3 sind aus konkreten Code-Stellen abgeleitet (nicht
  spekulativ), Pitfall 4/5 aus exakter Test-/Struct-Analyse.

**Research date:** 2026-07-31
**Valid until:** Bis zum Merge dieser Phase (Code-Zeilennummern können durch
parallele Arbeit an `reporting.rs`/`rebooking_reconciliation.rs` driften — vor
Planungs-Nutzung mit `grep -n` re-verifizieren, falls seit diesem Datum andere
Phasen an denselben Dateien gearbeitet haben).
