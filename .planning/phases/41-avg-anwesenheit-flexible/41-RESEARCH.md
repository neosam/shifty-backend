# Phase 41: Ø-Anwesenheit bei flexiblen Stunden (BE+FE) – Research

**Researched:** 2026-07-02
**Domain:** Rust/Axum Backend (pure Aggregat-Funktion + HR-gated Endpoint) + Dioxus WASM Frontend (Kennzahl-Rendering im HR-Stats-Block)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (aus CONTEXT.md)

### Locked Decisions

- **D-AVG-01 (Kennzahl):** Ø Stunden pro Anwesenheitstag = `Σ geleistete Stunden / Anzahl Anwesenheitstage` über den Report-Zeitraum. Tagebasiert, bewusst verschieden von A-22-1 (Wochen-Durchschnitt).
- **D-AVG-02 (Anwesenheitstag):** Mindestens ein Tages-Eintrag der Kategorie `Shiftplan`, `ExtraWork` oder `VolunteerWork` mit `hours > 0`. Wochentag irrelevant.
- **D-AVG-03 (Exclusion by construction):** `Vacation`, `SickLeave`, `Holiday`, `UnpaidLeave`, `Unavailable` sind keine Anwesenheitstage und daher nicht im Nenner.
- **D-AVG-04 (Zeitraum):** Aggregation über den angezeigten Report-Zeitraum (`year`/`until_week`). Kein separater Picker.
- **D-AVG-05 (Scope):** Nur `is_dynamic == true` Mitarbeiter, server-seitig gefiltert. HR_PRIVILEGE erforderlich.
- **D-AVG-06 (Schwelle):** < 2 Anwesenheitstage → kein Wert (Leerzustand). Ebenso bei 0 Arbeitstagen.
- **D-AVG-07 (Ort):** Im bestehenden HR-Statistik-Block (`employee_view.rs`), direkt nach „Ø Std/Woche", vor „Einbezogene Wochen".
- **D-AVG-08 (No-Persist):** Reines Read-Aggregat. `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **12**. Kein `BillingPeriodValueType`, keine Migration.
- **D-AVG-09 (i18n):** Label, Tooltip und Leerzustand in de/en/cs.

### Claude's Discretion

- Ob die Zahl über den bestehenden `/{id}/weekly-statistics`-Endpoint (erweitert, range-aware) oder einen neuen Endpoint geliefert wird — Planner entscheidet. (Empfehlung: eigener Endpoint, Begründung unten.)
- Exakte Wiederverwendung von Report-Daten: direkt auf `EmployeeReport.by_week.days` arbeiten oder über neue Struktur.
- Rundung/Formatierung (bestehendes `format_hours(..)` nutzen).

### Deferred Ideas (OUT OF SCOPE)

- AVG-04: Trend über mehrere Abrechnungsperioden
- AVG-05: konfigurierbare Absence-Exclusion-Kategorien
- Multi-MA-Übersichtsliste
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Beschreibung | Research-Support |
|----|-------------|------------------|
| AVG-01 | HR sieht pro flexiblem MA (is_dynamic) Ø geleistete Stunden pro Anwesenheitstag über den Report-Zeitraum. Urlaub und Absences aus dem Nenner. | Pure function `average_hours_per_attendance_day` auf `WorkingHoursDay`-Slice; bestehende `ExtraHoursReportCategory`-Varianten klassifizieren Arbeit vs. Absence. |
| AVG-02 | FE zeigt Auswertung im Report. Reines Read-Aggregat, kein Snapshot-Bump, keine Persistenz. | Neuer HR-gated Endpoint (analog weekly-statistics); FE-Loader + Rendering im HR-Stats-Block. |
| AVG-03 | i18n de/en/cs für Labels, Tooltip, Leerzustand. | 3 neue i18n-Keys definiert; Übersetzungen aus UI-SPEC.md bestätigt. |
</phase_requirements>

---

## Summary

Phase 41 ergänzt eine **tagebasierte Anwesenheits-Kennzahl** für flexible Mitarbeiter (`is_dynamic == true`) auf einem reinen Read-Aggregat-Pfad. Alle strukturellen Muster existieren bereits im Codebase: die A-22-1 pure Funktion in `service/src/reporting.rs` dient als Vorlage, der `/{id}/weekly-statistics`-Endpoint ist Kopiervorlage für REST/utoipa/ApiDoc, und der HR-Stats-Block in `employee_view.rs` liefert das genaue FE-Einfügekonzept.

Die neue pure Funktion `average_hours_per_attendance_day` nimmt einen `&[WorkingHoursDay]`-Slice (aus `EmployeeReport.by_week[*].days` flattened), zählt DISTINCT Datum-Einträge der Kategorien `Shiftplan | ExtraWork | VolunteerWork` mit `hours > 0` als Anwesenheitstage, summiert die zugehörigen Stunden als Zähler und gibt `Option<f32>` (None wenn < 2 Anwesenheitstage) zurück. A-22-1 (`average_worked_hours_per_week`) wird NICHT verändert und NICHT wiederverwendet.

Der Endpoint wird als **neuer** range-aware `GET /report/{id}/attendance-statistics?year=Y&until_week=W` realisiert, da der bestehende `/weekly-statistics`-Endpoint keine Date-Params hat (er ist „year-to-date") und eine Erweiterung eine Breaking Change wäre. Der `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12 — es gibt keinen neuen `BillingPeriodValueType` und keinen neuen Snapshot-Schreibpfad.

**Primäre Empfehlung:** Neuer Endpoint `/report/{id}/attendance-statistics?year=Y&until_week=W` (nicht Erweiterung von `/weekly-statistics`). Testmodul analog zu `service_impl/src/test/reporting_avg_weekly.rs`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Anwesenheitstag-Klassifikation | Service / Pure Function | — | Reine Datenverarbeitung ohne I/O; testbar ohne Mocks |
| HR-Privilege-Gate | Service (Business-Logic) | — | Auth-Check vor Data-Fetch (STAT-01-Pattern bestätigt) |
| is_dynamic-Filter | Service (Business-Logic) | — | Server-seitig per D-AVG-05; FE bekommt None für nicht-flexible MA |
| Report-Datenabruf | Service (Business-Logic via ReportingService) | DAO | Bestehender `get_report_for_employee`-Pfad; keine neue DB-Query |
| REST-Serialisierung | REST Layer (Axum) | rest-types | utoipa-Annotation + ToSchema TO |
| FE-Rendering | Browser / WASM (Dioxus) | — | `employee_view.rs` HR-Stats-Block |
| i18n | Browser / WASM | — | i18n mod.rs + de/en/cs-Dateien |

---

## Verifikation: Enum-Varianten `ExtraHoursReportCategory` [VERIFIED: service/src/reporting.rs:14]

```
Arbeit (Attendance-Kategorien):
  Shiftplan
  ExtraWork
  VolunteerWork

Absence (NICHT im Nenner, per D-AVG-03):
  Vacation
  SickLeave
  Holiday
  Unavailable
  UnpaidLeave

Sonstiges (nicht als Attendance gewertet, da nicht explizit D-AVG-02):
  Custom(LazyLoad<Uuid, CustomExtraHours>)
```

`Custom(_)` ist KEIN Anwesenheitstag per D-AVG-02 (nur `Shiftplan | ExtraWork | VolunteerWork` zählen). Custom-Stunden erscheinen nicht im Zähler und nicht im Nenner.

---

## Standard Stack

### Core (alle bereits im Projekt vorhanden – keine neuen Dependencies)

| Library / Crate | Version | Zweck | Warum Standard |
|----------------|---------|-------|----------------|
| `service` (Workspace-Crate) | workspace | Trait `ReportingService` + neue pure Funktion | Bestehende Schicht; neue Methode wird hier deklariert |
| `service_impl` (Workspace-Crate) | workspace | Impl `get_employee_attendance_statistics` | Bestehende Business-Logic-Tier |
| `rest-types` (Workspace-Crate) | workspace | Neues TO `EmployeeAttendanceStatisticsTO` | Single source of truth für DTOs (BE+FE) |
| `rest` (Workspace-Crate) | workspace | Neuer Axum-Handler + ApiDoc-Eintrag | Bestehende REST-Schicht |
| `utoipa` | workspace | `#[utoipa::path]` + `ToSchema` | Pflicht laut CLAUDE.md; bereits für alle Endpoints |
| Dioxus + rest-types | workspace | FE-Rendering + Typ-Nutzung | Bestehender WASM-Stack |

**Installation:** keine — alle Crates sind im Workspace vorhanden. `[VERIFIED: Cargo.toml]`

---

## Package Legitimacy Audit

Diese Phase installiert **keine neuen externen Pakete**. Alle Crates sind Workspace-Members oder bereits im Cargo.lock vorhanden. Audit entfällt.

| Package | Verdict | Disposition |
|---------|---------|-------------|
| (keine neuen) | — | — |

---

## Architecture Patterns

### System-Datenfluss für die neue Kennzahl

```
FE: EmployeeView (year, until_week bekannt)
  → api::get_employee_attendance_statistics(config, sales_person_id, year, until_week)
    → GET /report/{id}/attendance-statistics?year=Y&until_week=W
      → REST Handler (Axum)
        → ReportingService::get_employee_attendance_statistics(id, year, until_week, ctx, tx)
          → [1] check_permission(HR_PRIVILEGE, ctx)      ← HR-Gate first
          → [2] employee_work_details_service.find_by_sales_person_id(id)
                if !any(is_dynamic) → Ok(None)           ← is_dynamic Filter
          → [3] get_report_for_employee(id, year, until_week, ctx, tx)
                → by_week[*].days (Arc<[WorkingHoursDay]>)
          → [4] flatten days → &[WorkingHoursDay]
          → [5] average_hours_per_attendance_day(&days)  ← pure function
                → EmployeeAttendanceStatistics { avg: Option<f32>, days: u32, total: f32 }
          → Ok(Some(stats))
        → EmployeeAttendanceStatisticsTO::from(&stats)
        → JSON 200
      ← 403 wenn kein HR_PRIVILEGE
      ← 200 body: null / kein Feld wenn !is_dynamic (empfohlen: 200 mit None-Felder)
  ← Rc<EmployeeAttendanceStatisticsTO> (ok) | None (403 oder !is_dynamic)
FE: EmployeeStore.attendance_statistics = Some(Rc<TO>) | None
FE: employee_view.rs HR-Stats-Block:
  if should_show_hr_stats && attendance_statistics.is_some() {
    match stats.average_hours_per_attendance_day {
      Some(avg) → TupleRow { value: format_hours(avg, 2) }
      None      → TupleRow { value: "–" + title=AvgHoursPerAttendanceDayEmpty }
    }
  }
```

### Empfohlene Projekt-Struktur (neue Dateien / Änderungen)

```
service/src/reporting.rs
  + pub struct EmployeeAttendanceStatistics { avg: Option<f32>, attendance_days: u32, total_worked_hours: f32 }
  + pub fn average_hours_per_attendance_day(days: &[WorkingHoursDay]) -> EmployeeAttendanceStatistics
  + async fn get_employee_attendance_statistics(...) im trait ReportingService

service_impl/src/reporting.rs
  + impl get_employee_attendance_statistics (HR-gate → is_dynamic-check → report-fetch → pure fn)
  + mod test/reporting_avg_attendance   [NEU]

service_impl/src/test/reporting_avg_attendance.rs  [NEU — analog zu reporting_avg_weekly.rs]

rest-types/src/lib.rs
  + pub struct EmployeeAttendanceStatisticsTO { average_hours_per_attendance_day: Option<f32>, attendance_days: u32, total_worked_hours: f32 }
  + impl From<&EmployeeAttendanceStatistics> for EmployeeAttendanceStatisticsTO

rest/src/report.rs
  + fn get_attendance_statistics Handler
  + .route("/{id}/attendance-statistics", get(get_attendance_statistics))
  + ReportApiDoc: paths + components(schemas) ergänzt

shifty-dioxus/src/i18n/mod.rs    + AvgHoursPerAttendanceDay, AvgHoursPerAttendanceDayDescription, AvgHoursPerAttendanceDayEmpty
shifty-dioxus/src/i18n/de.rs     + 3 Keys (DE)
shifty-dioxus/src/i18n/en.rs     + 3 Keys (EN)
shifty-dioxus/src/i18n/cs.rs     + 3 Keys (CS)
shifty-dioxus/src/api.rs          + pub async fn get_employee_attendance_statistics(...)
shifty-dioxus/src/service/employee.rs  + attendance_statistics in EmployeeStore + load_employee_data
shifty-dioxus/src/component/employee_view.rs  + attendance_statistics Prop + Rendering
```

---

## Pattern 1: Neue Pure Funktion (analog A-22-1)

**Was:** Tagebasierte Anwesenheitstag-Aggregation aus einem `&[WorkingHoursDay]`-Slice.
**Wann:** Aufgerufen aus `get_employee_attendance_statistics` nach `report.by_week`-Flatten.

```rust
// service/src/reporting.rs — NEU, UNTER average_worked_hours_per_week
// Source: Vorlage average_worked_hours_per_week, reporting.rs:207-244 [VERIFIED]

/// Result of the AVG-01 attendance-day metric.
#[derive(Clone, Debug, PartialEq)]
pub struct EmployeeAttendanceStatistics {
    /// Average worked hours per attendance day, or None if fewer than 2 attendance days.
    pub average_hours_per_attendance_day: Option<f32>,
    /// Number of distinct calendar dates counted as attendance days (denominator).
    pub attendance_days: u32,
    /// Sum of worked hours across all attendance days (numerator).
    pub total_worked_hours: f32,
}

/// Pure AVG-01 formula: average worked hours per attendance day.
///
/// Attendance categories (D-AVG-02): Shiftplan, ExtraWork, VolunteerWork.
/// Absence categories (D-AVG-03): Vacation, SickLeave, Holiday, UnpaidLeave, Unavailable.
/// Custom(_) is NOT an attendance category.
///
/// A day counts as an attendance day iff it has at least one entry with
/// category in {Shiftplan, ExtraWork, VolunteerWork} and hours > 0.
///
/// Denominator < 2 → average is None (D-AVG-06).
pub fn average_hours_per_attendance_day(days: &[WorkingHoursDay]) -> EmployeeAttendanceStatistics {
    use std::collections::BTreeSet;
    use ExtraHoursReportCategory::{ExtraWork, Shiftplan, VolunteerWork};

    // Collect (date, hours) for work-category entries only.
    let work_entries: Vec<(time::Date, f32)> = days
        .iter()
        .filter(|d| {
            d.hours > 0.0
                && matches!(d.category, Shiftplan | ExtraWork | VolunteerWork)
        })
        .map(|d| (d.date.to_date(), d.hours))
        .collect();

    // Distinct attendance dates → denominator.
    let attendance_date_set: BTreeSet<time::Date> =
        work_entries.iter().map(|(date, _)| *date).collect();
    let attendance_days = attendance_date_set.len() as u32;

    // Sum all worked hours → numerator.
    let total_worked_hours: f32 = work_entries.iter().map(|(_, h)| h).sum();

    let average_hours_per_attendance_day = if attendance_days >= 2 {
        Some(total_worked_hours / attendance_days as f32)
    } else {
        None // D-AVG-06: nicht aussagekräftig
    };

    EmployeeAttendanceStatistics {
        average_hours_per_attendance_day,
        attendance_days,
        total_worked_hours,
    }
}
```

**Hinweis zu `d.date.to_date()`:** `WorkingHoursDay.date` ist `time::Date` direkt (kein `.to_date()` nötig — in der Vorlage ist `ShiftyDate` der Typ; muss im konkreten Code geprüft werden). `[VERIFIED: reporting.rs:44-48]` — `pub date: time::Date` direkt.

Korrigiert:
```rust
.map(|d| (d.date, d.hours))
// und
let attendance_date_set: BTreeSet<time::Date> =
    work_entries.iter().map(|(date, _)| *date).collect();
```

---

## Pattern 2: Service-Trait-Methode (analog get_employee_weekly_statistics)

```rust
// service/src/reporting.rs — Ergänzung im trait ReportingService
// Source: Vorlage get_employee_weekly_statistics, reporting.rs:287-294 [VERIFIED]

/// Returns average hours per attendance day (AVG-01) for the given sales person
/// over the specified report period. HR-gated (HR_PRIVILEGE).
/// Returns None for non-dynamic employees (D-AVG-05).
async fn get_employee_attendance_statistics(
    &self,
    sales_person_id: &Uuid,
    year: u32,
    until_week: u8,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Option<EmployeeAttendanceStatistics>, ServiceError>;
```

---

## Pattern 3: Service-Impl (HR-gate → is_dynamic → report → pure fn)

```rust
// service_impl/src/reporting.rs
// Source: Vorlage get_employee_weekly_statistics, service_impl/src/reporting.rs:1162-1192 [VERIFIED]

async fn get_employee_attendance_statistics(
    &self,
    sales_person_id: &Uuid,
    year: u32,
    until_week: u8,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Option<service::reporting::EmployeeAttendanceStatistics>, ServiceError> {
    // HR gate ist ERSTE Operation — kein Datenabruf vor Auth (D-AVG-05).
    self.permission_service
        .check_permission(HR_PRIVILEGE, context.clone())
        .await?;

    // is_dynamic-Filter: prüfe ob MA überhaupt flexible Stunden hat.
    let work_details = self
        .employee_work_details_service
        .find_by_sales_person_id(sales_person_id, Authentication::Full, tx.clone())
        .await?;
    let is_dynamic = work_details.iter().any(|w| w.is_dynamic);
    if !is_dynamic {
        return Ok(None); // D-AVG-05: nicht-flexible MA → kein Wert
    }

    let until_week = until_week.min(time::util::weeks_in_year(year as i32));

    // D-AVG-04: Report über den angezeigten Zeitraum.
    let report = self
        .get_report_for_employee(sales_person_id, year, until_week, context, tx)
        .await?;

    // Alle Tages-Einträge aus allen Wochen aggregieren.
    let all_days: Vec<service::reporting::WorkingHoursDay> = report
        .by_week
        .iter()
        .flat_map(|w| w.days.iter().cloned())
        .collect();

    // AVG-01 pure formula.
    let stats = service::reporting::average_hours_per_attendance_day(&all_days);
    Ok(Some(stats))
}
```

---

## Pattern 4: REST Endpoint (analog get_weekly_statistics)

**Entscheidung: NEUER Endpoint** `GET /report/{id}/attendance-statistics?year=Y&until_week=W`

**Begründung (Claude's Discretion):**
- Der bestehende `/weekly-statistics`-Endpoint hat keine Date-Parameter; er ist fest „year-to-date".
- Erweiterung um `year`/`until_week`-Params würde die bestehende Semantik brechen und die Signature des zugehörigen `get_employee_weekly_statistics`-Trait-Methode ändern.
- Ein eigener Endpoint folgt dem Single-Responsibility-Prinzip und stimmt mit dem „bewusst verschieden von A-22-1" aus D-AVG-01 überein.
- Das FE braucht `year` und `until_week` (die es bereits für den Haupt-Report kennt), was zum bestehenden `?year=Y&until_week=W`-Pattern des `GET /report/{id}`-Endpoints passt.

```rust
// rest/src/report.rs — NEUE Handler-Funktion
// Source: Vorlage get_weekly_statistics, report.rs:155-189 [VERIFIED]

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}/attendance-statistics",
    tags = ["Report"],
    params(
        ("id" = Uuid, Path, description = "Sales person ID"),
        ("year" = u32, Query, description = "Report year"),
        ("until_week" = u8, Query, description = "Report until calendar week (inclusive)")
    ),
    responses(
        (status = 200, description = "HR-only average hours per attendance day (flexible employees)", body = EmployeeAttendanceStatisticsTO, content_type = "application/json"),
        (status = 403, description = "Forbidden — HR role required"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_attendance_statistics<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Path(sales_person_id): Path<Uuid>,
    query: Query<ReportRequest>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let maybe_stats = rest_state
                .reporting_service()
                .get_employee_attendance_statistics(
                    &sales_person_id,
                    query.year,
                    query.until_week,
                    context.into(),
                    None,
                )
                .await?;
            // None = non-dynamic employee → 200 mit null-Body (FE speichert None)
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(
                    serde_json::to_string(&maybe_stats.as_ref().map(EmployeeAttendanceStatisticsTO::from)).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}
```

Route-Eintrag in `generate_route`:
```rust
.route("/{id}/attendance-statistics", get(get_attendance_statistics::<RestState>))
```

**ApiDoc-Eintrag (PFLICHT — ohne diesen Schritt fehlt der Endpoint in Swagger UI):**
```rust
#[openapi(
    paths(
        get_short_report_for_all,
        get_report,
        get_short_week_report,
        get_weekly_statistics,
        get_attendance_statistics  // ← NEU
    ),
    components(schemas(
        ShortEmployeeReportTO,
        EmployeeReportTO,
        ReportRequest,
        EmployeeWeeklyStatisticsTO,
        EmployeeAttendanceStatisticsTO  // ← NEU
    ))
)]
pub struct ReportApiDoc;
```

---

## Pattern 5: Transport Object in rest-types

```rust
// rest-types/src/lib.rs — nach EmployeeWeeklyStatisticsTO
// Source: Vorlage EmployeeWeeklyStatisticsTO, lib.rs:596-617 [VERIFIED]

/// DTO für die AVG-01 Ø-Anwesenheits-Kennzahl (Phase 41).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct EmployeeAttendanceStatisticsTO {
    /// Ø geleistete Stunden pro Anwesenheitstag, oder null wenn < 2 Anwesenheitstage (D-AVG-06).
    pub average_hours_per_attendance_day: Option<f32>,
    /// Anzahl Anwesenheitstage im Zeitraum (Nenner).
    pub attendance_days: u32,
    /// Summe geleisteter Stunden über alle Anwesenheitstage (Zähler).
    pub total_worked_hours: f32,
}

#[cfg(feature = "service-impl")]
impl From<&service::reporting::EmployeeAttendanceStatistics> for EmployeeAttendanceStatisticsTO {
    fn from(stats: &service::reporting::EmployeeAttendanceStatistics) -> Self {
        Self {
            average_hours_per_attendance_day: stats.average_hours_per_attendance_day,
            attendance_days: stats.attendance_days,
            total_worked_hours: stats.total_worked_hours,
        }
    }
}
```

---

## Pattern 6: FE-API-Funktion

```rust
// shifty-dioxus/src/api.rs — analog get_employee_weekly_statistics
// Source: Vorlage api.rs:387-401 [VERIFIED]

pub async fn get_employee_attendance_statistics(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
    until_week: u8,
) -> Result<Rc<EmployeeAttendanceStatisticsTO>, reqwest::Error> {
    let url = format!(
        "{}/report/{}/attendance-statistics?year={}&until_week={}",
        config.backend, sales_person_id, year, until_week
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res: Option<EmployeeAttendanceStatisticsTO> = response.json().await?;
    // None-Response (nicht-flexible MA): Fehler simulieren damit caller .ok() → None erhält
    // Alternativ: Option<Rc<...>> zurückgeben
    res.map(|s| Rc::new(s)).ok_or_else(|| /* kein echter Fehler */ /* Workaround: */ {
        panic!("unreachable — use Option return type instead")
    })
}
```

**Hinweis:** Besser `Result<Option<Rc<EmployeeAttendanceStatisticsTO>>, reqwest::Error>` zurückgeben, damit `None` (nicht-flexible MA) und Fehler unterschieden werden können. Der bestehende Pattern `.ok()` im Loader verarbeitet beide Fälle als `None`.

---

## Pattern 7: FE-Loader und EmployeeStore

```rust
// shifty-dioxus/src/service/employee.rs
// Ergänzung in EmployeeStore:
pub struct EmployeeStore {
    // ... bestehende Felder ...
    pub attendance_statistics: Option<Rc<EmployeeAttendanceStatisticsTO>>,  // NEU
}

// In load_employee_data — nach weekly_statistics-Abruf:
let attendance_statistics =
    api::get_employee_attendance_statistics(CONFIG.read().clone(), sales_person_id, year, until_week)
        .await
        .ok()
        .flatten();  // Option<Option<...>> → Option<...>
```

---

## Pattern 8: FE-Rendering in employee_view.rs

```rust
// shifty-dioxus/src/component/employee_view.rs — im HR-Stats-Block, nach Ø Std/Woche
// Source: employee_view.rs:519-541 [VERIFIED]; UI-SPEC.md §Component Inventory [VERIFIED]

// In EmployeeViewPlainProps: neues Feld
pub attendance_statistics: Option<Rc<EmployeeAttendanceStatisticsTO>>,  // NEU

// Im HR-Stats-Block (nach TupleRow AverageWorkedHoursPerWeek, vor StatisticsIncludedWeeks):
if let Some(att_stats) = props.attendance_statistics.as_ref() {
    let value_rsx = match att_stats.average_hours_per_attendance_day {
        Some(avg) => rsx! { span { class: "font-mono tabular-nums",
            {format_hours(avg, 2)}
        }},
        None => rsx! { span {
            class: "font-mono tabular-nums text-ink-muted",
            title: "{i18n.t(Key::AvgHoursPerAttendanceDayEmpty)}",
            "–"
        }},
    };
    rsx! {
        TupleRow {
            label: ImStr::from(i18n.t(Key::AvgHoursPerAttendanceDay).as_ref()),
            value: value_rsx,
            description: Some(rsx! {
                "{i18n.t(Key::AvgHoursPerAttendanceDayDescription)}"
            }),
        }
    }
}
```

---

## Pattern 9: i18n (3 neue Keys in de/en/cs)

```
// mod.rs — enum Key:
AvgHoursPerAttendanceDay,
AvgHoursPerAttendanceDayDescription,
AvgHoursPerAttendanceDayEmpty,

// de.rs:
"Ø Std/Anwesenheitstag"
"Durchschnittliche Arbeitsstunden pro Anwesenheitstag (nur flexible MA). Urlaub und Abwesenheiten sind nicht im Nenner."
"Nicht aussagekräftig (weniger als 2 Anwesenheitstage)"

// en.rs:
"Avg h/attendance day"
"Average working hours per day actually worked (flexible employees only). Absences excluded from the denominator."
"Not meaningful (fewer than 2 attendance days)"

// cs.rs:
"Prům. hod./den přítomnosti"
"Průměrné pracovní hodiny za den skutečné přítomnosti (pouze flexibilní zaměstnanci). Absence nejsou ve jmenovateli."
"Nevýznamné (méně než 2 dny přítomnosti)"
```

**Quelle:** UI-SPEC.md §Copywriting Contract `[VERIFIED: 41-UI-SPEC.md]`

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Datum-Deduplizierung | Eigene Hash-Map | `BTreeSet<time::Date>` | Compiler-verifiziert, kein Overhead |
| HTTP-Fehler-Handling | match status codes | `error_for_status_ref()` + `error_handler()` | Bestehender Pattern in allen REST-Handlers |
| Auth-Check | Flag-basiert in Handler | `check_permission(HR_PRIVILEGE, ctx)` als ERSTE Operation | Security-Invariante: nie Daten vor Auth lesen |
| Report-Datenabruf | Neue DAO-Query | `get_report_for_employee(...)` → `by_week.days` | Daten bereits vorhanden; keine neue DB-Abfrage nötig |
| Stunden-Formatierung | `format!("{:.2}", h)` | `format_hours(h, 2)` | Konsistenz mit bestehender UI |
| utoipa-Schema | manuell | `#[derive(ToSchema)]` + `components(schemas(...))` | CLAUDE.md-Pflicht |

---

## Snapshot-Versionsverifikation (D-AVG-08, A-22-1 bleibt unberührt)

**Bestätigung `[VERIFIED: service_impl/src/billing_period_report.rs:117]`:**

```
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;
```

Die neue Phase:
- fügt **keinen** neuen `BillingPeriodValueType` hinzu
- ändert **keinen** bestehenden Snapshot-Schreibpfad
- liest nur aus `get_report_for_employee` — dieser Pfad ist **nicht** Teil des Snapshot-Schreibzyklus
- modifiziert `average_worked_hours_per_week` (A-22-1) **nicht**

**Pflicht-Grep im Execute-Schritt:**

```bash
grep -rn "CURRENT_SNAPSHOT_SCHEMA_VERSION" /home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/
# Erwartet: genau eine Zeile: `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;`

grep -rn "average_worked_hours_per_week" /home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/reporting.rs
# Erwartet: Funktion UNVERÄNDERT vorhanden (Signatur-Check)
```

---

## Common Pitfalls

### Pitfall 1: A-22-1 versehentlich verändern
**Was schiefläuft:** `average_worked_hours_per_week` wird „nur kurz" für die neue Formel angepasst. Alle Tests in `reporting_avg_weekly.rs` brechen; Billing-Period-Snapshots können driften.
**Ursache:** Beide Funktionen operieren auf ähnlichen Daten; Versuchung zur Wiederverwendung.
**Vermeidung:** Neue Funktion hat ANDEREN Namen (`average_hours_per_attendance_day`), anderen Input-Typ (`&[WorkingHoursDay]` statt `&[GroupedReportHours]`) und eigenes Struct. A-22-1 wird **nie angefasst**.
**Frühwarnung:** `grep -n "average_worked_hours_per_week" service/src/reporting.rs` muss die Funktion **unverändert** zeigen.

### Pitfall 2: Absence-Tage zählen (Nenner-Fehler)
**Was schiefläuft:** Ein Tag mit `Vacation=8h` und keiner Arbeit wird als Anwesenheitstag gewertet, weil `hours > 0` geprüft wird ohne Kategorie-Filter.
**Ursache:** Category-Match fehlt oder `Custom(_)` wird als Arbeit behandelt.
**Vermeidung:** `matches!(d.category, Shiftplan | ExtraWork | VolunteerWork)` ist Pflicht. Unit-Test: `absence_day_not_counted`.

### Pitfall 3: Falscher Nenner (Wochen statt Tage)
**Was schiefläuft:** Nenner = Anzahl Wochen mit Arbeit statt Anzahl Tage. Ergebnis wäre identisch zu A-22-1.
**Ursache:** Verwechslung mit der bestehenden Wochenlogik.
**Vermeidung:** Deduplizierung über `BTreeSet<time::Date>` auf Ebene einzelner Tage; Unit-Test mit bekanntem Ergebnis (12 Tage, 54h → 4.5).

### Pitfall 4: ApiDoc-Eintrag vergessen
**Was schiefläuft:** Endpoint erreichbar, aber nicht in Swagger UI; CI-Run akzeptiert den Code trotzdem.
**Ursache:** `paths()` und `components(schemas())` in `ReportApiDoc` manuell gepflegt.
**Vermeidung:** Checkbox im Plan-Task explizit; `grep "get_attendance_statistics" rest/src/report.rs` muss sowohl Handler als auch ApiDoc-Eintrag zeigen.

### Pitfall 5: is_dynamic-Filter FE-seitig statt server-seitig
**Was schiefläuft:** FE rendert Kennzahl für nicht-flexible MA (weil `attendance_statistics.is_some()` durch alten Store-State wahr ist).
**Ursache:** is_dynamic-Check im FE vergessen; der Server gibt für nicht-flexible MA `None` zurück, aber der Store wird nicht geleert.
**Vermeidung:** `EmployeeStore.attendance_statistics` wird in `load_employee_data` immer neu gesetzt (auch bei `None`); Server-seitig ist der Filter die primäre Quelle der Wahrheit.

### Pitfall 6: Snapshot-Bump ausgelöst durch unbeabsichtigte Seiteneffekte
**Was schiefläuft:** Eine andere Änderung in `billing_period_report.rs` erhöht `CURRENT_SNAPSHOT_SCHEMA_VERSION` auf 13. Phase-41-Tests schlagen fehl.
**Ursache:** Gleichzeitige Änderungen.
**Vermeidung:** Pflicht-Grep vor dem Commit; Phase 41 berührt `billing_period_report.rs` **nicht**.

---

## Validation Architecture

> `nyquist_validation` ist in config.json nicht gesetzt → als aktiviert behandelt.

### Test Framework

| Eigenschaft | Wert |
|-------------|------|
| Framework | Rust built-in `#[test]` via `cargo test` |
| Konfiguration | Kein separates Config-File (Workspace-Standard) |
| Schnell-Run (BE) | `cargo test -p service_impl test_attendance` |
| Vollständiger Run | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |
| FE-Tests | `cargo test --manifest-path shifty-dioxus/Cargo.toml` |

### Requirements → Test Map

| Req ID | Verhalten | Test-Typ | Automatisierter Befehl | Datei vorhanden? |
|--------|----------|----------|------------------------|-----------------|
| AVG-01 | 12 Anwesenheitstage, 54h → 4.5 h/Tag | unit (pure fn) | `cargo test -p service_impl reporting_avg_attendance::user_example` | ❌ Wave 0 |
| AVG-01 | Absence-Tag (Vacation=8h, keine Arbeit) → nicht gezählt | unit (pure fn) | `cargo test -p service_impl reporting_avg_attendance::absence_day_not_counted` | ❌ Wave 0 |
| AVG-01 | Misch-Tag (Shiftplan=4h + Vacation=4h) → Anwesenheitstag, Zähler=4h | unit (pure fn) | `cargo test -p service_impl reporting_avg_attendance::mixed_day_counts_work_only` | ❌ Wave 0 |
| AVG-01 | Custom-Kategorie → kein Anwesenheitstag | unit (pure fn) | `cargo test -p service_impl reporting_avg_attendance::custom_category_not_attendance` | ❌ Wave 0 |
| AVG-01 | Leeres Slice → attendance_days=0, avg=None | unit (pure fn) | `cargo test -p service_impl reporting_avg_attendance::empty_slice_returns_none` | ❌ Wave 0 |
| AVG-02/06 | 1 Anwesenheitstag → avg=None (< 2) | unit (pure fn) | `cargo test -p service_impl reporting_avg_attendance::one_day_returns_none` | ❌ Wave 0 |
| AVG-02/06 | 2 Anwesenheitstage → avg=Some(f32) | unit (pure fn) | `cargo test -p service_impl reporting_avg_attendance::two_days_returns_some` | ❌ Wave 0 |
| AVG-01 | A-22-1 bleibt unverändert | unit (Regression) | `cargo test -p service_impl test::reporting_avg_weekly` | ✅ vorhanden |
| AVG-01 | Snapshot-Version bleibt 12 | Konstanten-Assertion | `grep "CURRENT_SNAPSHOT_SCHEMA_VERSION.*12" service_impl/src/billing_period_report.rs` | ✅ (grep) |
| AVG-02 | HR-Gate: Nicht-HR-User → 403 | unit (mock) | `cargo test -p service_impl reporting::attendance_statistics_requires_hr` | ❌ Wave 0 |
| AVG-02 | is_dynamic-Filter: nicht-flexible MA → None | unit (mock) | `cargo test -p service_impl reporting::attendance_statistics_returns_none_for_static` | ❌ Wave 0 |
| AVG-03 | i18n-Keys in allen 3 Locales vorhanden | unit (i18n) | `cargo test --manifest-path shifty-dioxus/Cargo.toml i18n_attendance_keys_present_in_all_locales` | ❌ Wave 0 |

### Sampling Rate

- **Pro Task-Commit:** `cargo test -p service_impl reporting_avg_attendance && cargo clippy --workspace -- -D warnings`
- **Pro Wave-Merge:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Phase Gate:** Vollständige Suite grün + Pflicht-Greps bestätigt vor `/gsd-verify-work`

### Wave 0 Lücken

- [ ] `service_impl/src/test/reporting_avg_attendance.rs` — deckt alle AVG-01/06-Fälle ab (analog `reporting_avg_weekly.rs`)
- [ ] Mock-Tests für HR-Gate + is_dynamic-Filter in `service_impl/src/reporting.rs` `#[cfg(test)]`-Block
- [ ] i18n-Test `i18n_attendance_keys_present_in_all_locales` in `shifty-dioxus/src/i18n/mod.rs` (analog existierende `i18n_week_status_keys_present_in_all_locales`)

---

## Security Domain

| ASVS Kategorie | Anwendbar | Standard-Kontrolle |
|----------------|----------|-------------------|
| V2 Authentication | nein | — |
| V3 Session Management | nein | — |
| V4 Access Control | **ja** | `check_permission(HR_PRIVILEGE, ctx)` als ERSTE Operation; identisch mit bestehendem `/weekly-statistics`-Gate |
| V5 Input Validation | gering | `year: u32`, `until_week: u8` — Rust-Typen; `until_week.min(weeks_in_year(...))` clampt den Wert |
| V6 Kryptographie | nein | — |

### Bekannte Bedrohungsmuster

| Pattern | STRIDE | Standardmitigation |
|---------|--------|-------------------|
| Unauthorized HR data access | Information Disclosure | HR_PRIVILEGE-Gate ist ERSTE Operation im Service; Handler prüft nie vorher |
| IDOR: anderer MA's Statistik | Information Disclosure | `sales_person_id` Path-Param; Service prüft HR_PRIVILEGE (HR sieht alle) |
| Input-Manipulation (year=0, week=99) | Tampering | `until_week.min(weeks_in_year(year))` — bestehender Pattern aus `get_employee_weekly_statistics` |

---

## Environment Availability

Alle Tools sind im Nix-Develop-Shell vorhanden. Phase 41 hat keine neuen externen Tools.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo test` | BE-Tests | ✓ | Rust stable (nix develop) | — |
| `cargo clippy` | CI-Gate | ✓ | Rust stable (nix develop) | — |
| Dioxus WASM-Build | FE-Validierung | ✓ | dx 0.6.x (gepinnt) | — |

---

## Project Constraints (aus CLAUDE.md)

- Clippy ist **Hard Gate**: `cargo clippy --workspace -- -D warnings` MUSS vor jedem Commit laufen — `cargo test` allein reicht nicht.
- OpenAPI: jeder neue REST-Endpoint braucht `#[utoipa::path]`-Annotation + ApiDoc-Eintrag (`components(schemas(...))`).
- i18n: alle benutzersichtbaren Texte in **de/en/cs**.
- Service-Tier: `ReportingService` ist Business-Logic-Tier; darf `EmployeeWorkDetailsService` (Basic-Tier) konsumieren — kein Zyklus.
- Transaktionen: jede Service-Methode akzeptiert `Option<Transaction>`.
- Snapshot-Bump: nur bei neuem/geändertem `BillingPeriodValueType` — Phase 41 bringt keinen → Version bleibt **12**.
- `jj`-Co-located: GSD-Auto-Commit via git (commit_docs: true); keine manuellen `git commit`-Aufrufe.
- Cargo SQLX Prepare: neue `query!`/`query_as!` → `cargo sqlx prepare --workspace` in `nix develop`. **Phase 41 hat keine neuen SQL-Queries** (alles über bestehende Service-Methoden) → kein sqlx-Prepare nötig.

---

## State of the Art

| Bisheriger Ansatz | Aktueller Ansatz (Phase 41) | Impact |
|-------------------|----------------------------|--------|
| Wöchentliche Durchschnitts-Stunden (A-22-1) als einzige HR-Statistik | Zusätzlich: tagebasierte Anwesenheits-Kennzahl für flexible MA | HR kann flexibles Arbeitsverhalten (kurze vs. lange Tage) besser bewerten |
| `get_employee_weekly_statistics` ohne Date-Range (year-to-date) | Neuer range-aware Endpoint mit `year`/`until_week` | Report-Zeitraum wird explizit verwendet statt implizit |

---

## Assumptions Log

| # | Claim | Abschnitt | Risiko wenn falsch |
|---|-------|-----------|-------------------|
| A1 | `find_by_sales_person_id` auf `employee_work_details_service` existiert und ist im `ReportingServiceDeps`-Trait verfügbar | Pattern 3 (Service-Impl) | Andere Methode muss verwendet werden (z.B. `all()` + Filter) |
| A2 | `WorkingHoursDay.date` ist `time::Date` (kein Wrapper) | Pattern 1 | Wrapper-Methode nötig statt direktem Vergleich |
| A3 | `BTreeSet<time::Date>` funktioniert (time::Date ist Ord) | Pattern 1 | HashSet oder manuelle Dedup nötig |
| A4 | `Option<EmployeeAttendanceStatisticsTO>` lässt sich via `serde_json::to_string` zu `null` serialisieren | Pattern 4 | Separater Response-Typ nötig |

**Claim A2 ist via Code verifiziert `[VERIFIED: service/src/reporting.rs:46]`:** `pub date: time::Date` — kein Wrapper. Kein `.to_date()` nötig.
**Claim A3:** `time::Date` implementiert `Ord` `[ASSUMED]` — Standard in der time-Crate, aber nicht im Code verifiziert.

---

## Open Questions (RESOLVED)

1. **`find_by_sales_person_id` auf EmployeeWorkDetailsService?**
   - Was wir wissen: `employee_work_details_service.all()` existiert; `filter(|w| w.sales_person_id == id)` ist im `get_reports_for_all_employees`-Impl vorhanden (service_impl/src/reporting.rs:290–294).
   - Was unklar ist: Ob eine direkte `find_by_sales_person_id`-Methode existiert oder `all()` + Filter verwendet werden muss.
   - Empfehlung: Bei der Implementierung prüfen; falls nicht vorhanden → `all()` + clientseitiger Filter (wie im bestehenden Code).
   - **RESOLVED:** Plan 41-02 Task 2 implementiert den `all()`+Filter-Fallback (kein harter Bedarf an einer dedizierten Methode).

2. **Serde-Verhalten von `Option<EmployeeAttendanceStatisticsTO>` als JSON**
   - Was wir wissen: `serde_json::to_string(&None::<T>)` → `"null"`.
   - Was unklar ist: Ob das FE `reqwest::Response::json::<Option<EmployeeAttendanceStatisticsTO>>()` korrekt deserialisiert.
   - Empfehlung: Im FE `response.json::<Option<EmployeeAttendanceStatisticsTO>>()` verwenden; testet mit Mock-Backend.
   - **RESOLVED:** Plan 41-04 Task 2 nutzt `response.json::<Option<EmployeeAttendanceStatisticsTO>>()` (null → None), mit i18n-/Leerzustand-Test.

---

## Sources

### Primary (HIGH confidence)
- `service/src/reporting.rs:13-295` [VERIFIED: direkt gelesen] — ExtraHoursReportCategory-Varianten, WorkingHoursDay, GroupedReportHours, EmployeeWeeklyStatistics, average_worked_hours_per_week (A-22-1 Vorlage), ReportingService-Trait
- `service_impl/src/reporting.rs:1162-1192` [VERIFIED: direkt gelesen] — get_employee_weekly_statistics Impl-Vorlage mit HR-Gate-Pattern
- `rest/src/report.rs:155-204` [VERIFIED: direkt gelesen] — get_weekly_statistics Handler + ReportApiDoc Kopiervorlage
- `rest-types/src/lib.rs:596-617` [VERIFIED: direkt gelesen] — EmployeeWeeklyStatisticsTO Vorlage
- `service_impl/src/test/reporting_avg_weekly.rs` [VERIFIED: direkt gelesen] — Test-Pattern für pure Funktion
- `.planning/phases/41-avg-anwesenheit-flexible/41-CONTEXT.md` [VERIFIED: direkt gelesen] — alle D-AVG-Entscheidungen
- `.planning/phases/41-avg-anwesenheit-flexible/41-UI-SPEC.md` [VERIFIED: direkt gelesen] — i18n-Texte, TupleRow-Props, Einfügeposition, Leerzustand
- `service_impl/src/billing_period_report.rs:117` [VERIFIED: grep] — `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12`
- `shifty-dioxus/src/component/employee_view.rs:27-94, 519-541` [VERIFIED: direkt gelesen] — Props-Struct, should_show_hr_stats, HR-Stats-Block
- `shifty-dioxus/src/service/employee.rs:28-36, 116-128` [VERIFIED: direkt gelesen] — EmployeeStore, load_employee_data
- `shifty-dioxus/src/api.rs:387-401` [VERIFIED: direkt gelesen] — get_employee_weekly_statistics API-Vorlage

### Secondary (MEDIUM confidence)
- `.planning/REQUIREMENTS.md:74-87` [VERIFIED: direkt gelesen] — AVG-01/02/03 Requirements

---

## Metadata

**Konfidenz-Aufschlüsselung:**
- Standard Stack: HIGH — keine neuen Dependencies; alle Muster im Code verifiziert
- Architektur: HIGH — Vorlage-Patterns direkt aus Codebase übernommen
- Pure Funktion: HIGH — Algorithmus einfach, Kategorien verifiziert
- Pitfalls: HIGH — aus Codebase-Inspektion abgeleitet
- i18n-Texte: HIGH — direkt aus UI-SPEC.md übernommen

**Research Datum:** 2026-07-02
**Gültig bis:** 2026-08-02 (stabile Codebase; Dioxus + Axum-Patterns ändern sich selten)
