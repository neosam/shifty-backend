# Phase 2: Reporting Integration & Snapshot Versioning — Research

**Researched:** 2026-05-01
**Domain:** Rust Backend — Reporting-Pipeline-Erweiterung, Feature-Flag-Infrastruktur, Snapshot-Schema-Versionierung
**Confidence:** HIGH (alle kritischen Pfade direkt im Quellcode verifiziert)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Cross-Category-Aufloesung im Reporting (BUrlG §9)**
- D-Phase2-01: Sick gewinnt bei Vacation∩SickLeave (BUrlG §9-konform). An jedem Tag mit aktiver Vacation- UND SickLeave-AbsencePeriod: SickLeave gewinnt, Vacation produziert 0h — Urlaub bleibt unverbraucht.
- D-Phase2-02: Cross-Category-Resolver lebt in `AbsenceService::derive_hours_for_range`. Single source of truth fuer Phase 4 Migration-Gate.
- D-Phase2-03: Deterministische Prioritaets-Reihenfolge: `SickLeave > Vacation > UnpaidLeave`.

**UnpaidLeave Snapshot-Mapping**
- D-Phase2-04: Neuer `BillingPeriodValueType::UnpaidLeave`-Variante in `service/src/billing_period.rs`.
- D-Phase2-05: Snapshot-Read v2 — fehlende `unpaid_leave`-Zeile wird als 0.0 interpretiert (fail-safe).

**Feature-Flag-Mechanik**
- D-Phase2-06: Eigene generische Tabelle `feature_flag(key TEXT PK, enabled INTEGER NOT NULL DEFAULT 0, description TEXT, update_timestamp TEXT, update_process TEXT NOT NULL)`. NICHT Reuse von `toggle`-Tabelle. Migration seedet `('absence_range_source_active', 0, ...)`.
- D-Phase2-07: Eigener `FeatureFlagService` (Trait + Impl + Mock + DI) mit Privileg `feature_flag_admin`. API: `is_enabled(key, ctx, tx)`, `set(key, value, ctx, tx)`. REST out of scope fuer Phase 2.

**Reporting-Switch**
- D-Phase2-08-A: Switch-Mechanismus lebt in `ReportingService`. Einmaliges `is_enabled("absence_range_source_active", ...)` pro Report-Range. Wenn false: bestehender ExtraHours-Pfad (unveraendert). Wenn true: `derive_hours_for_range(...)` fuer Vacation/Sick/UnpaidLeave; ExtraHours-Quelle fuer diese 3 Kategorien komplett ignoriert. ExtraWork/Volunteer/Holiday/CustomExtraHours bleiben immer ExtraHours-Quelle.

**Locking-Test (SNAP-02)**
- D-Phase2-08-B: Hybrid Locking-Test in `service_impl/src/test/billing_period_report.rs`: (1) Pin-Map-Test ueber alle 12 BillingPeriodValueType-Varianten, (2) Compiler-Exhaustive-Match-Test.
- D-Phase2-09: Pin-Map-Scope: alle 12 Varianten (inkl. UnpaidLeave neu).

**Snapshot-Bump**
- D-Phase2-10: `CURRENT_SNAPSHOT_SCHEMA_VERSION` 2 → 3 in `service_impl/src/billing_period_report.rs:37`. Im selben Commit wie D-Phase2-04, D-Phase2-08-A, D-Phase2-08-B, und die Feature-Flag-Migration.

### Claude's Discretion

- C-Phase2-01: Return-Type von `derive_hours_for_range` — Vorschlag `Result<BTreeMap<time::Date, ResolvedAbsence>, ServiceError>` mit `struct ResolvedAbsence { category: AbsenceCategory, hours: f32 }`. Plan-Phase darf alternativ `Vec<(Date, AbsenceCategory, f32)>` waehlen. Constraint: Output muss bereits konflikt-aufgeloest sein.
- C-Phase2-02: Feiertags-0-Aufloesung — Vertragsstunden des Tages = 0 ergibt Absence-Stunden = 0 (natuerlich aus Per-Tag-Vertrag-Lookup). Wochenend-Logik analog.
- C-Phase2-03: FeatureFlagService-DAO-Surface — schmaales DAO mit nur `get(key)` / `set(key, value)` reicht fuer Phase 2.
- C-Phase2-04: Pin-Map-Fixture-Werte entscheidet Plan-Phase.
- C-Phase2-05: DI-Reihenfolge fuer FeatureFlagService in `main.rs`.
- C-Phase2-06: Naming des Toggle-Keys: `absence_range_source_active` (snake_case ohne Punkt).

### Deferred Ideas (OUT OF SCOPE)

- REST-Endpoints fuer FeatureFlagService (Phase 5+)
- Feature-Flag-Audit-Trail
- Phase-4-Cutover-Gate (MIG-02/MIG-03)
- Atomares Feature-Flag-Flippen in derselben Tx wie MIG-01/MIG-04 (Phase 4)
- Carryover-Refresh nach Flag-Flip (Phase 4)
- REST-Endpoints fuer feature_flag mit OpenAPI
- Booking-Konflikt-Detection / Forward-/Reverse-Warnings (Phase 3)
- Migration aus ExtraHours (Phase 4)
- Frontend

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Beschreibung | Research Support |
|----|-------------|------------------|
| REP-01 | `AbsenceService::derive_hours_for_range(from, to, sales_person_id)` liefert pro Tag Vertragsstunden des am Tag gueltigen Vertrags; Feiertage = 0; Vertragswechsel mid-range korrekt | `AbsenceDao::find_by_sales_person` (verifiziert) + `EmployeeWorkDetailsService::find_by_sales_person_id` (verifiziert) + `DateRange::iter_days()` (Phase 1 fertig) |
| REP-02 | Solange Flag aus, bit-identische Ergebnisse (Werte-Map, nicht `snapshot_schema_version`) | `get_report_for_employee_range` Switch-Pfad verifiziert; Mock-Pattern fuer FeatureFlagService besteht |
| REP-03 | Flag an → ReportingService wechselt atomar zur neuen Quelle fuer Vacation/Sick/UnpaidLeave | Switch-Einstiegspunkte in `reporting.rs:558-583` verifiziert |
| REP-04 | `UnpaidLeave` korrekt in Report-Feldern und im Snapshot persistiert | `EmployeeReport.unpaid_leave_hours` existiert; Snapshot-Builder-Patch in `billing_period_report.rs:155-163` identifiziert |
| SNAP-01 | `CURRENT_SNAPSHOT_SCHEMA_VERSION = 3`; neue Snapshots tragen Version 3; v2-Snapshots lesbar (fehlende Zeile = 0.0) | `BillingPeriodSalesPerson::from_entities` liest via `BTreeMap`-Insert — fehlende Zeile ergibt einfach keinen Eintrag (semantisch 0.0) |
| SNAP-02 | Locking-Test schlaegt fehl bei Berechnungs-Drift ohne Versions-Bump | Exhaustive-Match-Pattern und Pin-Map-Pattern aus bestehendem Test-Infrastruktur abgeleitet |

</phase_requirements>

---

## Summary

Phase 2 koppelt drei unabhaengige Teilsysteme in einer Delivery-Einheit: (1) die neue `AbsenceService::derive_hours_for_range`-Methode als konflikt-aufgeloesten Tages-Iterator, (2) einen Feature-Flag-gesteuerten Switch im `ReportingService` der diese Methode als alternative Datenquelle fuer Vacation/Sick/UnpaidLeave einsetzt, und (3) die Snapshot-Schema-Bump-Pflicht (2 → 3) die CLAUDE.md verlangt, sobald sich Reporting-Inputs aendern. Alle drei muessen im selben Commit landen.

Die technische Basis aus Phase 1 ist vollstaendig verwendbar: `AbsenceService` existiert mit DI-Verdrahtung, `DateRange::iter_days()` ist fertig, das `gen_service_impl!`-Macro laesst sich direkt auf `FeatureFlagServiceImpl` anwenden, und die `ToggleService`/`toggle`-Architektur dient als 1:1-Strukturvorlage fuer `FeatureFlagService`. Der Reporting-Switch benoetigt keine neuen DAO-Schichten — er konsumiert bestehende Service-Interfaces hinter einem einzigen `if`-Zweig in `get_report_for_employee_range`.

Das groesste Risiko ist die Bit-Identitaets-Garantie bei Flag=off: `snapshot_schema_version` steigt unweigerlich von 2 auf 3 (da Phase 2 UnpaidLeave als neuen value_type einfuehrt), aber die `values`-Map eines Reports bei deaktiviertem Flag muss bit-identisch zu einem Pre-Phase-2-Report sein. Tests muessen explizit nur die `values`-Map vergleichen, nicht das Versionsfeld.

**Primaere Empfehlung:** Plan-Phase folgt der Wellen-Reihenfolge: Wave 0 (Feature-Flag-Infrastruktur + Migration), Wave 1 (`derive_hours_for_range` im AbsenceService), Wave 2 (Reporting-Switch + UnpaidLeave Snapshot-Variante + Version-Bump + Locking-Tests) — alles in Wave 2 als ein atomarer Commit.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `derive_hours_for_range` Cross-Category-Resolver | Service (AbsenceService) | DAO (AbsenceDao read) | Konflikt-Aufloesung ist Geschaeftslogik; DAO liefert Rohdaten |
| Feature-Flag-Zustand lesen | Service (FeatureFlagService) | DAO (FeatureFlagDao) | Thin DAO; Service handhabt Permission-Gate fuer `set` |
| Feature-Flag-Zustand schreiben | Service (FeatureFlagService) | — | `feature_flag_admin`-Privileg-Check im Service |
| Reporting-Switch (ExtraHours vs. AbsencePeriod) | Service (ReportingService) | — | Switch ist Business-Logik, nicht Datenzugriff |
| Snapshot-Builder UnpaidLeave-Insert | Service (BillingPeriodReportService) | — | Bestehende Funktion `build_billing_period_report_for_sales_person` |
| Per-Tag-Vertrags-Lookup | Service (EmployeeWorkDetailsService) | DAO | `find_by_sales_person_id` liefert alle Vertraege; Service iteriert |
| Feiertags-Lookup | Service (AbsenceService intern) | SpecialDayService | Einfachste Loesung: `find_by_sales_person` gibt Perioden; `SpecialDayService::get_by_week` fuer Feiertags-0-Aufloesung |

---

## Existing Code Inventory

### 1. Reporting-Pipeline-Einstiegspunkte

**`service_impl/src/reporting.rs:418-591`** — `get_report_for_employee_range`

Die zentrale Funktion des Reporting-Service. Ruft:
1. Permission-Check (HR oder self)
2. `employee_work_details_service.find_by_sales_person_id(...)` — holt alle Vertraege
3. `shiftplan_report_service.extract_shiftplan_report(...)` — Schichtplan
4. `extra_hours_service.find_by_sales_person_id_and_year_range(...)` — **hier ist der Switch-Punkt**
5. `hours_per_week(...)` — berechnet gruppierte Stunden-Map
6. Aggregiert alles zu `EmployeeReport`

**Switch-Stellen fuer den `absence_range_source_active`-Flag:**

```
service_impl/src/reporting.rs:456-465    extra_hours_service.find_by_sales_person_id_and_year_range(...)
service_impl/src/reporting.rs:558-583    Aggregation von vacation_hours/sick_leave_hours/unpaid_leave_hours
```

Wenn Flag an: `extra_hours_service`-Call faellt weg (oder wird auf ExtraWork/Holiday/Volunteer reduziert), `absence_service.derive_hours_for_range(...)` ersetzt die drei Felder. Die `hours_per_week`-Aggregation muss dann ebenfalls auf Absence-derived-Stunden umgestellt werden — das betrifft die wochenweise Gruppierung in `GroupedReportHours`.

**[VERIFIED: Quellcode service_impl/src/reporting.rs]**

### 2. ExtraHours → Absence-Aggregations-Pfade (alle Switch-Stellen)

| Zeile(n) | Was wird aggregiert | Switch-Notwendigkeit |
|----------|--------------------|--------------------|
| `558-562` | `vacation_hours` aus ExtraHours-Filter | Ja — durch `derive_hours_for_range` ersetzen |
| `563-567` | `sick_leave_hours` aus ExtraHours-Filter | Ja — durch `derive_hours_for_range` ersetzen |
| `568-572` | `holiday_hours` aus ExtraHours-Filter | Nein — Holiday bleibt ExtraHours-Quelle |
| `573-577` | `volunteer_hours` aus `by_week` | Nein — bleibt ExtraHours-Quelle |
| `579-583` | `unpaid_leave_hours` aus ExtraHours-Filter | Ja — durch `derive_hours_for_range` ersetzen |
| `875-980` in `hours_per_week()` | per-Woche-Aggregation (vacation, sick, unpaid) | Ja — wenn Flag an, muessen wochenweise Werte aus `derive_hours_for_range`-Map extrapoliert werden |

**[VERIFIED: Quellcode service_impl/src/reporting.rs]**

### 3. Per-Tag-Vertragsstunden-Lookup

`EmployeeWorkDetails` in `service/src/employee_work_details.rs`:
- Felder: `expected_hours`, `workdays_per_week`, `from_calendar_week`/`from_year`/`to_calendar_week`/`to_year`, Wochentags-Flags (`monday`..`sunday`)
- `hours_per_day() -> f32 = expected_hours / workdays_per_week`
- `has_day_of_week(weekday: Weekday) -> bool`
- `from_date() -> Result<ShiftyDate, ...>` und `to_date() -> Result<ShiftyDate, ...>`

**Kein `find_active_for(date)` existiert** — `find_by_sales_person_id` liefert alle Vertraege fuer einen Mitarbeiter. Die Per-Tag-Aufloesung erfordert manuelles Filtern: `wh.from_date() <= day <= wh.to_date()`. Das Pattern ist bereits in `find_working_hours_for_calendar_week(working_hours, year, week)` vorhanden (`service_impl/src/reporting.rs:72-81`). Der Plan muss dies in `derive_hours_for_range` analog implementieren — batch-Fetch einmal, dann per Tag filtern.

**[VERIFIED: Quellcode service/src/employee_work_details.rs]**

### 4. Feiertags-Handling

`SpecialDayService` in `service/src/special_days.rs`:
- `get_by_week(year, calendar_week, ctx) -> Result<Arc<[SpecialDay]>, ServiceError>`
- Kein `find_by_date`-Methode — nur nach Woche
- `SpecialDayType::Holiday` vs. `SpecialDayType::ShortDay`

**Befund:** `ReportingService` nutzt `SpecialDayService` NICHT direkt. Holidays fliessen als `ExtraHoursCategory::Holiday`-Eintraege in die Berechnung ein — sie sind manuell als `ExtraHours`-Eintraege angelegt, nicht aus `special_day`-Tabelle deriviert.

**Konsequenz fuer `derive_hours_for_range`:** Fuer die Feiertags-0-Aufloesung gibt es zwei Optionen:
- Option A: `AbsenceService` bekommt `SpecialDayService` als neue Dependency und ruft `get_by_week(...)` auf fuer jeden betroffenen Kalenderwochen-Bereich.
- Option B: Das "Vertragsstunden des Tages = 0 ⇒ Absence-Stunden = 0"-Pragma aus C-Phase2-02 — an Feiertagen hat `EmployeeWorkDetails.has_day_of_week(Weekday::Sunday)` keine Relevanz, ABER der Feiertag ist ein Werktag mit Null-Stunden-Override durch ExtraHours (Holiday-Kategorie), nicht durch den Vertrag. Der Vertrag liefert 8h auch am Feiertag.

**Korrekter Befund:** Feiertage in Shifty sind ExtraHours-Eintraege, keine SpecialDay-basierte Vertragsnullierung. Fuer `derive_hours_for_range` bedeutet das: Ein Feiertag innerhalb einer Absence-Periode soll 0 Urlaubsstunden verbrauchen, weil der Feiertag bereits als Holiday-ExtraHours-Eintrag das Stunden-Konto faellt. `derive_hours_for_range` braucht also `SpecialDayService` oder alternativ `ExtraHours`-Lookup um Feiertage auszuschliessen. Die sauberste Loesung ist: `AbsenceService` bekommt `SpecialDayService` als Dependency (Erweiterung des DI-Blocks in `main.rs`).

**[VERIFIED: Quellcode service/src/special_days.rs, service_impl/src/reporting.rs]**
**[ASSUMED: Semantik "Feiertag = 0 Urlaubsstunden in derive_hours_for_range via SpecialDayService"]**

### 5. Snapshot-Writer

**`service_impl/src/billing_period_report.rs:54-239`** — `build_billing_period_report_for_sales_person`

Ruft `reporting_service.get_report_for_employee_range(...)` vier mal (report_start, report_end, report_end_of_year, report_delta) und schreibt die Werte als `BillingPeriodValue`-Eintraege in die BTreeMap. Aktuelle Varianten:

| Zeilen | BillingPeriodValueType | Quelle |
|--------|------------------------|--------|
| 110-118 | Overall | `report_delta.overall_hours` |
| 119-127 | Balance | `report_delta.balance_hours` |
| 128-136 | ExpectedHours | `report_delta.expected_hours` |
| 137-145 | ExtraWork | `report_delta.extra_work_hours` |
| 146-154 | VacationHours | `report_delta.vacation_hours` |
| 155-163 | SickLeave | `report_delta.sick_leave_hours` |
| 164-172 | Holiday | `report_delta.holiday_hours` |
| 173-181 | VacationDays | `report_delta.vacation_days` |
| 182-190 | VacationEntitlement | `report_delta.vacation_entitlement` |
| 191-201 | Volunteer (optional) | `report_delta.volunteer_hours` (nur wenn != 0.0) |
| 202-228 | CustomExtraHours(name) | via `report_delta.custom_extra_hours` |

**Fehlender Eintrag: UnpaidLeave** — `report_delta.unpaid_leave_hours` existiert in `EmployeeReport`, wird aber bisher nicht persistiert. Insert nach Zeile 163 analog zum SickLeave-Insert.

**[VERIFIED: Quellcode service_impl/src/billing_period_report.rs:109-228]**

### 6. CURRENT_SNAPSHOT_SCHEMA_VERSION

```
service_impl/src/billing_period_report.rs:37
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 2;
```

**[VERIFIED: Quellcode service_impl/src/billing_period_report.rs:37]**

### 7. Bestehende BillingPeriodValueType-Varianten

Aktuell 11 Varianten in `service/src/billing_period.rs:34-46`:
- Balance, Overall, ExpectedHours, ExtraWork, VacationHours, SickLeave, Holiday, Volunteer, CustomExtraHours(Arc<str>), VacationDays, VacationEntitlement

Phase 2 fuegt hinzu: `UnpaidLeave` (12. Variante).

`FromStr::from_str` und `as_str()` muessen erweitert werden. Read-Path (`from_entities`) liest via `BTreeMap::insert` — fehlende Zeile fuer alte v2-Snapshots ergibt keinen Eintrag in der Map (semantisch 0.0 aus Sicht des Callers). Kein neuer versions-aware Lesepfad noetig.

**[VERIFIED: Quellcode service/src/billing_period.rs:36-90]**

### 8. AbsenceService und AbsenceDao — existierende Read-Methoden

`dao::absence::AbsenceDao` (verifiziert in `dao/src/absence.rs`):
- `find_by_sales_person(sales_person_id, tx)` — alle aktiven Eintraege eines Mitarbeiters
- `find_overlapping(sales_person_id, category, range, exclude_logical_id, tx)` — Overlap-Query

**`find_by_sales_person` reicht fuer `derive_hours_for_range`**: Alle 3 Kategorien in einem Call laden, dann per Tag / per Kategorie filtern. Kein neuer DAO-Call noetig fuer Phase 2. [VERIFIED: Quellcode dao/src/absence.rs]

`service::absence::AbsenceService` (verifiziert in `service/src/absence.rs`):
- `find_by_sales_person(sales_person_id, ctx, tx)` — bestehend
- `derive_hours_for_range(from, to, sales_person_id, ctx, tx)` — **neu in Phase 2**

---

## Integration Points

### Dateien, die modifiziert werden

| Datei | Art der Aenderung | Betroffene Stellen |
|-------|-------------------|-------------------|
| `service/src/billing_period.rs` | `UnpaidLeave`-Variante zu Enum + `as_str()` + `FromStr` | Zeilen 34-90 |
| `service/src/absence.rs` | Neue Trait-Methode `derive_hours_for_range(...)` | nach Zeile 162 |
| `service_impl/src/absence.rs` | Implementation von `derive_hours_for_range` | neue Methode + neue Deps |
| `service_impl/src/billing_period_report.rs` | `CURRENT_SNAPSHOT_SCHEMA_VERSION` 2→3; UnpaidLeave-Insert | Zeile 37; nach Zeile 163 |
| `service_impl/src/reporting.rs` | Feature-Flag-Switch; neue Dep `FeatureFlagService` und `AbsenceService` | gen_service_impl Block; `get_report_for_employee_range`-Body |
| `service_impl/src/test/billing_period_report.rs` | 2 neue Locking-Tests | nach Zeile 1149 |
| `shifty_bin/src/main.rs` | `FeatureFlagServiceDependencies`-Block; ReportingServiceDeps erweitern | analog Phase-1-Pattern |

### Dateien, die neu erstellt werden

| Datei | Inhalt |
|-------|--------|
| `service/src/feature_flag.rs` | `FeatureFlagService`-Trait + Domain-Modell + Konstante `FEATURE_FLAG_ADMIN_PRIVILEGE` |
| `service_impl/src/feature_flag.rs` | `FeatureFlagServiceImpl` via `gen_service_impl!` |
| `service_impl/src/test/feature_flag.rs` | Service-Tests (is_enabled, set, forbidden) |
| `dao/src/feature_flag.rs` | `FeatureFlagDao`-Trait |
| `dao_impl_sqlite/src/feature_flag.rs` | `FeatureFlagDaoImpl` mit SQLx |
| `migrations/sqlite/<timestamp>_add-feature-flag-table.sql` | CREATE TABLE + Seed + Privileg-INSERT |

### Lib.rs-Patches (pub mod / re-exports)

- `service/src/lib.rs`: `pub mod feature_flag;`
- `service_impl/src/lib.rs`: `pub mod feature_flag;`
- `dao/src/lib.rs`: `pub mod feature_flag;`
- `dao_impl_sqlite/src/lib.rs`: `pub mod feature_flag;`

---

## Feature Flag Design

### Strukturvorbild: ToggleService

`ToggleService` in `service/src/toggle.rs` und `service_impl/src/toggle.rs` ist das 1:1-Template. Die Parallele ist exakt:

```
service/src/toggle.rs          →  service/src/feature_flag.rs
service_impl/src/toggle.rs     →  service_impl/src/feature_flag.rs
dao/src/toggle.rs              →  dao/src/feature_flag.rs
dao_impl_sqlite/src/toggle.rs  →  dao_impl_sqlite/src/feature_flag.rs
```

**Unterschiede zu ToggleService:**
- Schmaales DAO (nur `get(key)` und `set(key, value)`) — kein Group-Management
- Privileg: `feature_flag_admin` (nicht `toggle_admin`)
- Tabellen-Schema: `feature_flag(key TEXT PK, enabled INTEGER NOT NULL DEFAULT 0, description TEXT, update_timestamp TEXT, update_process TEXT NOT NULL)`
- `is_enabled` fuer einen nicht-existenten Key: safe default = false (analog `ToggleDaoImpl::is_enabled` Zeile 168: `.unwrap_or(false)`)

### Schema-Template (aus Toggles)

```sql
-- migrations/sqlite/<timestamp>_add-feature-flag-table.sql
CREATE TABLE feature_flag (
    key TEXT NOT NULL PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,
    description TEXT,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);

INSERT INTO feature_flag (key, enabled, description, update_process)
VALUES (
    'absence_range_source_active',
    0,
    'When ON, range-based AbsencePeriods are the source of truth for Vacation/Sick/UnpaidLeave hours. Flip atomically with Phase-4 migration.',
    'phase-2-migration'
);

INSERT INTO privilege (name, update_process)
VALUES ('feature_flag_admin', 'initial');
```

**[VERIFIED: Strukturvorlage aus migrations/sqlite/20260105000000_app-toggles.sql und dao_impl_sqlite/src/toggle.rs]**

### Wie der Switch in `get_report_for_employee_range` eingebaut wird

Der `gen_service_impl!`-Block von `ReportingServiceImpl` bekommt zwei neue Deps:
- `FeatureFlagService: FeatureFlagService<...> = feature_flag_service`
- `AbsenceService: AbsenceService<...> = absence_service`

In `get_report_for_employee_range` (nach dem Permission-Check, vor der ExtraHours-Aggregation):

```rust
let use_absence_range_source = self.feature_flag_service
    .is_enabled("absence_range_source_active", Authentication::Full, tx.clone().into())
    .await?;

// if use_absence_range_source:
//   derived = absence_service.derive_hours_for_range(from_date, to_date, sales_person_id, ...)
//   vacation_hours = derived.iter().filter(|r| r.category == Vacation).map(|r| r.hours).sum()
//   sick_leave_hours = ...
//   unpaid_leave_hours = ...
// else:
//   vacation_hours = extra_hours.iter().filter(Vacation).map(amount).sum()
//   sick_leave_hours = ...
//   unpaid_leave_hours = ...
```

**ACHTUNG:** Das Flag-Lesen geschieht mit `Authentication::Full` (intern, kein User-Context-Check fuer interne Service-zu-Service-Calls).

---

## Snapshot Versioning Mechanic

### Aktueller Stand

```
service_impl/src/billing_period_report.rs:37
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 2;
```

### Was sich in Version 3 aendert

| Trigger (CLAUDE.md) | Konkret |
|--------------------|---------|
| Neuer `value_type` (additiv) | `BillingPeriodValueType::UnpaidLeave` — D-Phase2-04 |
| Input-Set-Aenderung | Wenn Flag an: VacationHours/SickLeave/UnpaidLeave werden aus AbsencePeriod-derived Quelle geschrieben statt aus ExtraHours |

### v2-Snapshots bleiben lesbar

`BillingPeriodSalesPerson::from_entities` (`service/src/billing_period.rs:132-162`) baut eine `BTreeMap` aus den DB-Zeilen. Zeilen mit unbekanntem `value_type` werden still ignoriert (kein `?`-Propagation, sondern `if let Ok(...)`). Fehlende `unpaid_leave`-Zeile in v2-Snapshots ergibt keinen Eintrag in der Map. Der Consumer (`billing_period_values.get(&UnpaidLeave)`) erhaelt `None` → 0.0. **Kein expliziter v2/v3-Branch-Code noetig.**

**[VERIFIED: service/src/billing_period.rs:132-162]**

### Commit-Atomaritaet (CLAUDE.md-Pflicht)

Alle folgenden Aenderungen MUESSEN im selben Commit landen:
1. `service/src/billing_period.rs`: `UnpaidLeave`-Variante
2. `service_impl/src/billing_period_report.rs:37`: Version 2 → 3
3. `service_impl/src/billing_period_report.rs:155-163`: UnpaidLeave-Insert nach SickLeave
4. `service_impl/src/reporting.rs`: Feature-Flag-Switch
5. `service_impl/src/test/billing_period_report.rs`: Locking-Tests
6. Die neue Feature-Flag-Migration

---

## Locking-Test Pattern

### Test 1: Pin-Map (`test_snapshot_v3_pinned_values`)

**Datei:** `service_impl/src/test/billing_period_report.rs` (ergaenzt bestehende Tests ab Zeile 1149)

```rust
/// LOCKING TEST — DO NOT NAIVELY UPDATE.
///
/// If this test fails after a code change:
///   - Did you intentionally change the snapshot computation?
///   - If yes, you MUST also bump CURRENT_SNAPSHOT_SCHEMA_VERSION
///     in service_impl/src/billing_period_report.rs.
///   - See CLAUDE.md § "Billing Period Snapshot Schema Versioning"
///     for the bump-trigger rules.
#[tokio::test]
async fn test_snapshot_v3_pinned_values() {
    // Fixture: 1 Sales-Person, 8h/Tag 5 Werktage Mo-Fr
    // Range: 2024-06-03 (Mo) bis 2024-06-09 (So)
    // - Vacation AbsencePeriod: 2024-06-03..2024-06-05 (Mo-Mi)
    // - SickLeave AbsencePeriod: 2024-06-04..2024-06-04 (Di — ueberlappt mit Vacation)
    // - ExtraHours ExtraWork: 2024-06-06 +2h
    //
    // Erwartete Werte (Flag = an):
    //   VacationHours.delta = 16.0  (Mo: 8h + Mi: 8h; Di faellt an SickLeave)
    //   SickLeave.delta = 8.0       (Di: 8h)
    //   UnpaidLeave.delta = 0.0
    //   ExtraWork.delta = 2.0
    //   Holiday.delta = 0.0
    //   ExpectedHours.delta = 40.0  (5 Werktage * 8h)
    //   Balance.delta = -14.0       (2h ExtraWork - 40h Expected + 16+8h Absence-Kredit)
    //   ...

    let result = build_billing_period_report_for_sales_person(...).await;
    let values = &result.values;

    assert_eq!(values[&BillingPeriodValueType::VacationHours].value_delta, 16.0);
    assert_eq!(values[&BillingPeriodValueType::SickLeave].value_delta, 8.0);
    assert_eq!(values[&BillingPeriodValueType::UnpaidLeave].value_delta, 0.0);
    assert_eq!(values[&BillingPeriodValueType::ExtraWork].value_delta, 2.0);
    // ... alle 12 Varianten
}
```

**Anmerkung:** Die exakten Fixture-Werte legt Plan-Phase fest (C-Phase2-04). Der obige Sketch basiert auf dem Vorschlag aus CONTEXT.md `<specifics>`.

### Test 2: Compiler-Exhaustive-Match (`test_billing_period_value_type_surface_locked`)

```rust
/// LOCKING TEST — DO NOT NAIVELY UPDATE.
///
/// Wenn der Compiler hier eine fehlende Variante meldet: bist du sicher,
/// dass du nicht CURRENT_SNAPSHOT_SCHEMA_VERSION bumpen wolltest?
/// Siehe CLAUDE.md § "Billing Period Snapshot Schema Versioning".
#[test]
fn test_billing_period_value_type_surface_locked(value_type: &BillingPeriodValueType) {
    // Diese Funktion wird NIE direkt aufgerufen — sie existiert nur damit
    // der Compiler bei einem neuen Enum-Arm einen Compile-Error wirft.
    fn ensure_locked(value_type: &BillingPeriodValueType) {
        match value_type {
            BillingPeriodValueType::Overall => {}
            BillingPeriodValueType::Balance => {}
            BillingPeriodValueType::ExpectedHours => {}
            BillingPeriodValueType::ExtraWork => {}
            BillingPeriodValueType::VacationHours => {}
            BillingPeriodValueType::SickLeave => {}
            BillingPeriodValueType::UnpaidLeave => {}   // NEU in v3
            BillingPeriodValueType::Holiday => {}
            BillingPeriodValueType::Volunteer => {}
            BillingPeriodValueType::VacationDays => {}
            BillingPeriodValueType::VacationEntitlement => {}
            BillingPeriodValueType::CustomExtraHours(_) => {}
        }
    }
}
```

**[VERIFIED: Pattern aus CONTEXT.md `<specifics>` und bestehenden Tests in billing_period_report.rs:1089-1149]**

---

## Bit-Identity Test Strategy

### Problem

Wenn `absence_range_source_active = false`, MUSS `get_report_for_employee_range` identische `values`-Werte wie vor Phase 2 produzieren. ABER: `snapshot_schema_version` steigt von 2 auf 3 (unvermeidbar, weil `UnpaidLeave` als neuer value_type hinzukommt). Bit-Identitaet gilt nur fuer die `values`-Map.

### Test-Skelett

```rust
#[tokio::test]
async fn test_flag_off_produces_identical_values_to_pre_phase2() {
    // Fixture: bestehende ExtraHours-Eintraege (Vacation 8h, SickLeave 4h, UnpaidLeave 2h)
    // MockFeatureFlagService: expect_is_enabled().returning(|_, _, _| Ok(false))
    // MockAbsenceService: expect_derive_hours_for_range() MUSS 0x mal aufgerufen werden
    
    let mut feature_flag_mock = MockFeatureFlagService::new();
    feature_flag_mock
        .expect_is_enabled()
        .returning(|_, _, _| Ok(false));
    
    // ... mock rest of deps identisch zu pre-phase2 setup ...
    
    let report = service.get_report_for_employee_range(...).await.unwrap();
    
    // Nur values-Map vergleichen, NICHT snapshot_schema_version
    assert_eq!(report.vacation_hours, 8.0);
    assert_eq!(report.sick_leave_hours, 4.0);
    assert_eq!(report.unpaid_leave_hours, 2.0);
    // (derive_hours_for_range wurde nicht aufgerufen — mock haette panic wenn aufgerufen)
}
```

**Platzierung:** `service_impl/src/test/` — entweder in `billing_period_report.rs` oder in einer neuen `reporting.rs`-Testdatei fuer den Switch-Pfad.

---

## Cross-Category Overlap Policy

**Entscheidung D-Phase2-01/02/03 (aus CONTEXT.md, verbatim):**

An jedem Tag, an dem sowohl eine `Vacation`- als auch eine `SickLeave`-AbsencePeriod aktiv ist, werden die Vertragsstunden **ausschliesslich** der `SickLeave`-Kategorie zugerechnet. `Vacation` produziert fuer diesen Tag 0 Stunden — Urlaub bleibt unverbraucht (BUrlG §9).

**Deterministische Prioritaet: `SickLeave > Vacation > UnpaidLeave`**

Ueber alle 3 Pair-Kombinationen:
- SickLeave ∩ Vacation → SickLeave gewinnt, Vacation = 0h
- SickLeave ∩ UnpaidLeave → SickLeave gewinnt, UnpaidLeave = 0h
- Vacation ∩ UnpaidLeave → Vacation gewinnt, UnpaidLeave = 0h

**Implementierung in `derive_hours_for_range`:** Per-Tag-Iteration via `DateRange::iter_days()`. Fuer jeden Tag:
1. Vertragsstunden ermitteln (aus `EmployeeWorkDetails.hours_per_day()` fuer am Tag gueltigen Vertrag)
2. Wenn Vertragsstunden = 0 (Wochenende oder Feiertag) → alle Kategorien = 0
3. Sonst: unter allen AbsencePeriods die mit hoechester Prioritaet auswaehlen → diese bekommt die Vertragsstunden, alle anderen 0

**Resolver ist conflict-resolved:** `derive_hours_for_range` gibt pro Tag maximal eine Kategorie mit Stunden zurueck.

---

## Open Risks / Pitfalls

### Pitfall 1: Snapshot-Bump-Commit-Atomaritaet (CLAUDE.md, HOCH)

**Was schief geht:** Version-Bump und Berechnungs-Aenderung landen in verschiedenen Commits. Dann zeigt ein v3-Snapshot entweder alte Werte (Bump zuerst) oder ein v2-Snapshot zeigt neue Werte (Aenderung zuerst).

**Praevention:** Plan-Phase designt Wave 2 explizit als Single-Commit-Wave. Executor bekommt in Task-Beschreibung: "ALLE folgenden Dateien in einem Commit zusammenfuehren: billing_period_report.rs (Version + UnpaidLeave-Insert), billing_period.rs (UnpaidLeave-Variante), reporting.rs (Switch), test/billing_period_report.rs (Locking-Tests), Migration."

**[VERIFIED: CLAUDE.md Snapshot-Versioning-Abschnitt]**

### Pitfall 2: Bit-Identitaet bei Flag=off (HOCH)

**Was schief geht:** `snapshot_schema_version` wird irrtuemlicherweise in den Bit-Identitaets-Test einbezogen — wird immer 3 sein, auch bei Flag=off. Test schlaegt faelschlicherweise fehl.

**Praevention:** Test vergleicht explizit nur `report.vacation_hours`, `report.sick_leave_hours`, `report.unpaid_leave_hours` — nicht das Version-Feld.

### Pitfall 3: Vertragswechsel mid-range Performance

**Was schief geht:** `derive_hours_for_range` ruft `find_by_sales_person_id` einmal am Anfang, dann iteriert per Tag. Das ist korrekt. ABER: Wenn jemand mehrere Vertraege hat, muss fuer jeden Tag der richtige Vertrag gefunden werden.

**Praevention:** `find_by_sales_person_id` liefert alle Vertraege als `Arc<[EmployeeWorkDetails]>`. Fuer jeden Tag: `working_hours.iter().find(|wh| wh.from_date() <= day && day <= wh.to_date())`. Analoges Pattern zu `find_working_hours_for_calendar_week`.

### Pitfall 4: FeatureFlag-Race in Report

**Was schief geht:** Flag wird waehrend eines laufenden Reports von `set()` geaendert. Einmal-Lesen am Anfang des Reports (`is_enabled` vor der Haupt-Logik) verhindert inkonsistenten Wechsel mid-Report. SQLite-Tx-Isolation haelt, da der gesamte Report in einer Transaktion laeuft.

**Praevention:** Switch-Wert wird einmal am Anfang von `get_report_for_employee_range` gelesen und danach als lokale `bool`-Variable verwendet. **Nicht** pro Tag/Woche erneut abfragen.

### Pitfall 5: `hours_per_week()` muss ebenfalls umgestellt werden

**Was schief geht:** `get_report_for_employee_range` ruft `hours_per_week(shiftplan, extra_hours, ...)` auf. Diese Funktion aggregiert `vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours` intern aus `extra_hours` — auch bei Flag=an. Das wuerde doppelte Quelle erzeugen.

**Praevention:** Wenn Flag an: `hours_per_week` wird entweder (a) mit leerer `extra_hours`-Liste fuer die 3 Absence-Kategorien aufgerufen (ExtraHours gefiltert), oder (b) die `GroupedReportHours`-Felder werden nach dem `hours_per_week`-Aufruf mit den Absence-derived-Werten ueberschrieben. Option (a) ist sauberer. Plan-Phase entscheidet.

**[ASSUMED: Konsequenz aus der Analyse der hours_per_week-Funktion]**

### Pitfall 6: OpenAPI-Regen bei neuer Enum-Variante

**Was schief geht:** `BillingPeriodValueType::UnpaidLeave` erscheint in `rest-types`-DTOs. Wenn die ToSchema-Ableitung dort unveraendert bleibt, keine automatische Regen noetig. Aber: falls der REST-Endpunkt fuer BillingPeriod-Daten `UnpaidLeave` in der Response-JSON exponiert, muss `ApiDoc` nicht explizit angepasst werden (Enum-Werte werden automatisch abgeleitet).

**Verifizierung:** `BillingPeriodValueType` ist in `service/src/billing_period.rs` definiert und hat kein `ToSchema` — die REST-Response-DTOs in `rest-types` enthalten keine direkte `BillingPeriodValueType`-Serialisierung (sie serialisieren `value_type` als String). Kein OpenAPI-Manualpatch noetig.

### Pitfall 7: jj-Commit-Reihenfolge

**Was schief geht:** User committet Waves in falscher Reihenfolge mit jj. Feature-Flag-Infrastruktur muss vor dem Reporting-Switch committed sein (Compiler-Abhaengigkeit).

**Praevention:** Plan-Phase definiert klare Wave-Grenzen. Wave 0 (Feature-Flag-Infra) muss gruener `cargo build` sein vor Wave 1-Start. Wave 2 ist ein einziger Commit.

### Pitfall 8: AbsenceService bekommt neue Deps (SpecialDayService)

**Was schief geht:** `AbsenceServiceImpl` hat aktuell 6 Deps (Phase 1: `AbsenceDao`, `PermissionService`, `SalesPersonService`, `ClockService`, `UuidService`, `TransactionDao`). Wenn `SpecialDayService` fuer Feiertags-0-Aufloesung benoetigt wird, muss der `gen_service_impl!`-Block in `service_impl/src/absence.rs` und der `AbsenceServiceDependencies`-Block in `main.rs` erweitert werden.

**Praevention:** Alternativ: AbsenceService-Caller (ReportingService) reicht den Kontext mit, oder `EmployeeWorkDetailsService.has_day_of_week` wird als Proxy genutzt. Plan-Phase entscheidet welche Dep-Erweiterung sauberer ist.

**[ASSUMED: Benoetigung von SpecialDayService in AbsenceService fuer Feiertags-Orthogonalitaet]**

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in tests + `mockall` + `tokio::test` |
| Config file | keine separate Konfig — `[dev-dependencies]` in `service_impl/Cargo.toml` |
| Quick run command | `cargo test -p service_impl test::billing_period_report` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automatisierter Befehl | Datei |
|--------|----------|-----------|----------------------|-------|
| REP-01 | `derive_hours_for_range` pro Tag korrekte Vertragsstunden | unit | `cargo test -p service_impl test::absence::test_derive_hours_for_range` | `service_impl/src/test/absence.rs` ❌ Wave 1 |
| REP-01 | Feiertage → 0 Urlaubsstunden | unit | `cargo test -p service_impl test::absence::test_derive_hours_holiday_is_zero` | `service_impl/src/test/absence.rs` ❌ Wave 1 |
| REP-01 | Vertragswechsel mid-range | unit | `cargo test -p service_impl test::absence::test_derive_hours_contract_change` | `service_impl/src/test/absence.rs` ❌ Wave 1 |
| REP-02 | Flag=off → bit-identische values-Map | unit | `cargo test -p service_impl test::reporting::test_flag_off_values_identical` | `service_impl/src/test/reporting.rs` ❌ Wave 2 |
| REP-03 | Flag=on → Absence-derived Stunden im Report | unit | `cargo test -p service_impl test::reporting::test_flag_on_uses_absence_source` | `service_impl/src/test/reporting.rs` ❌ Wave 2 |
| REP-04 | `unpaid_leave_hours` korrekt | unit (Teil von REP-01/03) | (abgedeckt durch REP-01/03 Tests) | — |
| SNAP-01 | `snapshot_schema_version = 3` nach Bump | unit | `cargo test -p service_impl test::billing_period_report::test_build_and_persist_writes_current_snapshot_schema_version` | ✅ bestehend (prueft CURRENT_SNAPSHOT_SCHEMA_VERSION-Konstante) |
| SNAP-01 | UnpaidLeave-Zeile persistiert | unit | `cargo test -p service_impl test::billing_period_report::test_snapshot_v3_pinned_values` | ❌ Wave 2 |
| SNAP-01 | v2-Snapshot lesbar (fehlende unpaid_leave-Zeile) | unit | `cargo test -p service test::billing_period::test_from_entities_missing_unpaid_leave_is_zero` | ❌ Wave 2 |
| SNAP-02 | Locking-Test schlaegt bei Drift fehl | compile-time + unit | `cargo test -p service_impl test::billing_period_report::test_billing_period_value_type_surface_locked` | ❌ Wave 2 |
| SNAP-02 | Pin-Map alle 12 Varianten | unit | `cargo test -p service_impl test::billing_period_report::test_snapshot_v3_pinned_values` | ❌ Wave 2 |

### Sampling Rate

- **Pro Task-Commit:** `cargo test -p service_impl` (schnell, ~10s)
- **Pro Wave-Merge:** `cargo test --workspace`
- **Phase-Gate:** `cargo test --workspace` vollstaendig gruen vor Abschluss

### Wave 0 Gaps

- [ ] `service/src/feature_flag.rs` — FeatureFlagService-Trait (Wave 0)
- [ ] `service_impl/src/feature_flag.rs` — FeatureFlagServiceImpl (Wave 0)
- [ ] `service_impl/src/test/feature_flag.rs` — Service-Tests (Wave 0)
- [ ] `dao/src/feature_flag.rs` — FeatureFlagDao-Trait (Wave 0)
- [ ] `dao_impl_sqlite/src/feature_flag.rs` — FeatureFlagDaoImpl (Wave 0)
- [ ] `migrations/sqlite/<timestamp>_add-feature-flag-table.sql` — Schema + Seed (Wave 0)
- [ ] `service_impl/src/test/absence.rs` — derive_hours_for_range Tests (Wave 1)
- [ ] `service_impl/src/test/reporting.rs` — Flag-Switch-Tests (Wave 2)

---

## Standard Stack

### Core (verifiziert, keine Aenderungen noetig)

| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| `sqlx` | 0.8 | SQLite-Queries fuer FeatureFlagDaoImpl | Bereits in `Cargo.toml` |
| `mockall` | 0.13 | `#[automock]` fuer FeatureFlagService-Trait | Bereits in `dev-dependencies` |
| `async-trait` | 0.1 | Async-Trait-Pattern | Bereits da |
| `tokio` | 1 | `#[tokio::test]` fuer async Tests | Bereits da |
| `time` | 0.3 | `time::Date` fuer `derive_hours_for_range` | Bereits da |
| `shifty_utils::DateRange` | Phase 1 | `iter_days()` fuer Per-Tag-Iteration | Phase 1 fertig |

**[VERIFIED: Cargo.toml-Abhaengigkeiten implizit aus Bestandscode]**

---

## Don't Hand-Roll

| Problem | Nicht bauen | Stattdessen | Warum |
|---------|-------------|-------------|-------|
| Feature-Flag-DAO | Direktes SQLite-Query in Service | `FeatureFlagDaoImpl` mit `#[automock]` | Testbarkeit; Konsistenz mit Architektur |
| Per-Tag-Vertrags-Lookup | Neues DAO-Query | `find_by_sales_person_id` + lokale Iteration | Bestehend; analog `find_working_hours_for_calendar_week` |
| Konflikt-Aufloesung | Ad-hoc if/else pro Callsite | `derive_hours_for_range` zentralisiert | Phase-4-Gate benoetigt dieselbe Logik |
| Snapshot-Read-Versionierung | Versions-aware branching | `BTreeMap`-Semantik (`from_entities`) | Fehlende Zeile = 0.0 ist bereits korrekt |

---

## Assumptions Log

| # | Claim | Abschnitt | Risiko wenn falsch |
|---|-------|-----------|-------------------|
| A1 | Feiertage in Shifty sind ExtraHours-Eintraege (Holiday-Kategorie), nicht SpecialDay-basierte Vertragsnullierung | Existing Code Inventory §4, Pitfall 8 | SpecialDayService wuerde dann nicht benoetigt fuer Feiertags-0-Aufloesung; aber die Frage "was passiert an einem Feiertag der in einer AbsencePeriod liegt" bleibt offen |
| A2 | `hours_per_week()` muss bei Flag=an auf gefilterte ExtraHours-Liste (ohne Vacation/Sick/Unpaid) oder auf Override der wochenweisen Felder umgestellt werden | Open Risks §5 | Wenn nicht behoben: doppelte Quellen-Mischung im wochenweisen Report |
| A3 | AbsenceService benoetigt SpecialDayService als neue Dep fuer Feiertags-Orthogonalitaet | Feature Flag Design, Pitfall 8 | Alternative: Plan-Phase nutzt ExtraHours-basierte Holiday-Detection; kein neuer Dep |

---

## Project Constraints (from CLAUDE.md)

- **Tests sind Pflicht** fuer jede Aenderung (`~/.claude/CLAUDE.md`)
- **Snapshot-Bump-Pflicht:** `CURRENT_SNAPSHOT_SCHEMA_VERSION` muss im selben Commit gebumpt werden wie Berechnungs-Aenderung (`CLAUDE.md § Billing Period Snapshot Schema Versioning`)
- **OpenAPI-Pflicht:** Wenn REST-Endpoints fuer `FeatureFlagService` erwoeitert werden (nicht in Phase 2), `#[utoipa::path]`-Annotation erforderlich. Phase 2 ist Service-only, kein REST.
- **`cargo build`, `cargo test`, `cargo run`** nach neuen Features ausgefuehrt werden
- **VCS:** jj-only, kein `git commit`. `commit_docs: false` — User commitet manuell.
- **NixOS:** `sqlx-cli` via `nix-shell` falls benoetigt fuer Migration-Setup.
- **WHERE deleted IS NULL** in jeder DAO-Read-Query (gilt nicht fuer `feature_flag` — keine Soft-Delete).

---

## Sources

### Primary (HIGH confidence)
- `service_impl/src/billing_period_report.rs` — CURRENT_SNAPSHOT_SCHEMA_VERSION, Snapshot-Builder
- `service_impl/src/reporting.rs` — vollstaendige Reporting-Pipeline, Switch-Stellen
- `service/src/billing_period.rs` — BillingPeriodValueType-Enum (11 Varianten verifiziert)
- `service/src/reporting.rs` — EmployeeReport-Struktur-Felder
- `service/src/absence.rs` — AbsenceService-Trait (Phase 1 Output)
- `service/src/employee_work_details.rs` — EmployeeWorkDetails-Felder, find_by_sales_person_id
- `service/src/special_days.rs` — SpecialDayService-API
- `service/src/toggle.rs` — ToggleService-Strukturvorlage
- `service_impl/src/toggle.rs` — ToggleServiceImpl-Strukturvorlage
- `dao/src/absence.rs` — AbsenceDao-Trait (find_by_sales_person, find_overlapping)
- `dao_impl_sqlite/src/toggle.rs` — is_enabled-Implementierung (fail-safe false)
- `migrations/sqlite/20260105000000_app-toggles.sql` — Schema-Template
- `service_impl/src/test/billing_period_report.rs:1089-1149` — bestehende Schema-Version-Tests
- `.planning/phases/02-reporting-integration-snapshot-versioning/02-CONTEXT.md` — alle D-Phase2-*-Entscheidungen

### Secondary (MEDIUM confidence)
- `.planning/phases/01-absence-domain-foundation/01-04-SUMMARY.md` — Phase 1 Ergebnis-Bestaetigung
- `.planning/ROADMAP.md` — Phase-2-Success-Criteria

---

## Metadata

**Confidence breakdown:**
- Existing Code Inventory: HIGH — alle Dateipfade und Zeilennummern direkt verifiziert
- Integration Points: HIGH — aus Quellcode abgeleitet
- Feature Flag Design: HIGH — ToggleService als Template verifiziert
- Snapshot Versioning: HIGH — Konstante und Builder direkt verifiziert
- Locking-Test Pattern: HIGH — aus CONTEXT.md-Spezifikation und bestehenden Tests abgeleitet
- Feiertags-Orthogonalitaet: MEDIUM — Annahme A1 besteht; Plan-Phase soll SpecialDayService-vs-ExtraHours-Holiday-Ansatz entscheiden

**Research date:** 2026-05-01
**Valid until:** 2026-06-01 (stabile Architektur; Rust-Versionierungen aendern sich nicht schnell)
