# Phase 64: Planungs-Soll für dynamische Verträge - Pattern Map

**Gemappt:** 2026-09-29
**Analysierte Dateien:** 6 (4 Backend, 2 Frontend)
**Analoge gefunden:** 6 / 6 — diese Phase ist reines Innen-Refactoring bereits vorhandener
Aggregationsstellen, deshalb liegt der beste Analog-Code jeweils **im selben File direkt
neben der Änderungsstelle** (bestehendes Feld als 1:1-Vorlage), nicht in einer fremden Datei.

## File Classification

| Zu ändernde Datei | Rolle | Data Flow | Analog (im selben File) | Match |
|---|---|---|---|---|
| `service_impl/src/reporting.rs` (`assemble_weeks`, ~996-1147) | service (business-logic tier) | CRUD/aggregation (in-memory) | `dynamic_hours`/`expected_hours`-Berechnung selbst (~996-1000, 1045-1100) | exact — gleicher Konstruktionspunkt, neues Feld folgt exakt demselben Push-Pattern |
| `service_impl/src/reporting.rs` (`get_reports_for_all_employees`, ~1318-1665) | service (business-logic tier) | CRUD/aggregation (in-memory) | `expected_hours`/`dynamic_hours`-Dreiwege-Verzweigung (~1318-1543) | exact — gleicher Konstruktionspunkt |
| `service/src/reporting.rs` (`ShortEmployeeReport`-Struct, ~147-170) | model | — | `has_pending_rebooking`/`pending_rebooking_id`-Felder (Phase-55-Präzedenz, ~163-169) | exact — additives Feld am selben Struct, gleiches Kommentar-Muster |
| `rest-types/src/lib.rs` (`ShortEmployeeReportTO` + `From`-Impl, ~380-410) | model (DTO) | request-response | `has_pending_rebooking`/`pending_rebooking_id` in TO + `From`-Impl (~388-410) | exact — additives `#[serde(default)]`-Feld, identisches Muster |
| `service_impl/src/test/reporting_planning_target.rs` (NEU) | test | CRUD/aggregation | `service_impl/src/test/reporting_additive_merge.rs:100-176` (Mock-Boilerplate) + `:962-1091` (Cross-Site-Paritätstest) | exact — 1:1 kopierbares Skelett laut RESEARCH.md |
| `shifty-dioxus/src/component/working_hours_mini_overview.rs` (3 Anzeigestellen) | component | request-response (reine Anzeige) | jede Stelle liest `working_hour.expected_hours` bzw. `r.expected_hours` (Zeilen 113, 171, 216) | exact — reiner Feldzugriff, kein Berechnungsausdruck |

**Korrektur zu RESEARCH.md:** RESEARCH.md spricht in der Prosa von „vier" Anzeigestellen,
listet in der eigenen Tabelle (Abschnitt „D-64-01") aber nur **drei**: `CardsLayout`-`target`
(:113), `TableLayout`-`total_target` (:171), `TableLayout`-`target` pro Zeile (:216). Ich habe
die Datei komplett gegrept (`grep -n "expected_hours"`) — es gibt tatsächlich nur diese drei
Lesestellen von `expected_hours` als Anzeige-Target. Der Planner soll mit **drei** Stellen
planen, nicht vier.

## Pattern Assignments

### `service_impl/src/reporting.rs::assemble_weeks` (service, aggregation)

**Analog:** derselbe Funktionskörper, Konstruktion von `dynamic_hours`/`expected_hours`.

**Rohwert-Extraktion aus dem `weight_for_week`-Tupel** (Zeilen 996-1000):
```rust
let (planned_hours, dynamic_hours): (f32, f32) =
    find_working_hours_for_calendar_week(working_hours, year, week)
        .map(|wh| weight_for_week(year, week, wh))
        .map(|wfw| (wfw.0, wfw.1))
        .fold((0.0, 0.0), |(acc_a, acc_b), (a, b)| (acc_a + a, acc_b + b));
```
→ `dynamic_hours` (Element 1) ist die Basis für `planning_target_hours` — unreduziert, aber
`relation`-skaliert für Teilwochen. **Nicht** neu ableiten, dieselbe Destrukturierung
wiederverwenden.

**Bestehendes (falsches Vorbild für die Reduktion) Gate-Pattern** (Zeilen 1045-1084):
```rust
let abense_hours_for_balance = if !has_contract_row || planned_hours <= 0.0 {
    0.0f32
} else {
    abense_hours
};
// ... absence_derived_balance_total, holiday_derived_gated: identisches Gate
let expected_hours_for_cap =
    planned_hours - abense_hours_for_balance - absence_derived_balance_total;
let expected_hours = expected_hours_for_cap - holiday_derived_gated;
```
**Wichtig:** Dieses Gate (`planned_hours <= 0.0`) NICHT für `planning_target_hours`
wiederverwenden — für dynamische Verträge ist `planned_hours` immer 0, das Gate würde die
Reduktion komplett überspringen (Pitfall 1 aus RESEARCH.md). Die neue Berechnung braucht ein
**eigenes** Gate, das nur auf `!has_contract_row` prüft, angewendet auf `dynamic_hours`
(Element 1) statt auf `planned_hours`.

**Push-Stelle mit Kommentar-Präzedenz für additive Felder** (Zeilen 1126-1147):
```rust
result.push(ShortEmployeeReport {
    sales_person: Arc::new(sales_person),
    balance_hours,
    dynamic_hours,
    expected_hours,
    overall_hours,
    // ...
    // Phase 55 (55-03): der HR-Alert-Banner wird ueber den
    // short-report `get_reports_for_all_employees` getrieben ...
    has_pending_rebooking: false,
    pending_rebooking_id: None,
});
```
→ `planning_target_hours` wird hier als neues Feld ergänzt (analog zu `expected_hours`,
NICHT neutral wie `has_pending_rebooking` — Pitfall 4).

---

### `service_impl/src/reporting.rs::get_reports_for_all_employees` (service, aggregation)

**Analog:** Dreiwege-Verzweigung `if !has_contract_row { ... } else if expected_hours <= 0.0
{ ... } else { ... }` (Zeilen ~1400-1543), Push-Stelle Zeilen ~1650-1665.

**Kein-Vertrag-Zweig** (Zeilen 1400-1409, gekürzt):
```rust
sick_leave_hours: 0.0,
holiday_hours: 0.0,
// ...
volunteer_hours: manual_volunteer_hours + auto_volunteer_hours + shiftplan_hours,
custom_absence_hours: HashMap::new(),
absence_derived_balance_hours: 0.0,
cap_active,
```

**Dynamik-Zweig — fehlt heute die Holiday-Berechnung komplett** (Zeilen 1411-1449):
```rust
} else if expected_hours <= 0.0 {
    // Dynamischer Vertrag (Zeile vorhanden, expected=0): Soll=Ist.
    let extra_work: f32 = extra_hours_array.iter()
        .filter(|extra_hours| extra_hours.category == ExtraHoursCategory::ExtraWork
            && extra_hours.date_time.iso_week() == week
            && extra_hours.date_time.year() as u32 == year)
        .map(|extra_hours| extra_hours.amount).sum();
    // ... rebooked_extra_work ...
    WeeklyHours {
        shiftplan_hours,
        extra_working_hours: extra_work,
        absense_hours: 0.0,
        planned_hours: overall_hours - rebooked_extra_work,
        dynamic_hours,
        vacation_hours: 0.0, sick_leave_hours: 0.0, holiday_hours: 0.0,
        // ...
    }
}
```
Diese `WeeklyHours`-Struct-Instanz hat **kein** Feld, das der neuen `planning_target_hours`
entspricht — der Struct selbst muss um ein Feld erweitert werden (analog zu `dynamic_hours`
im selben Struct-Literal), UND die Holiday-/Absence-Reduktions-Berechnung, die aktuell nur im
`else`-Zweig existiert (Zeilen 1457-1499: `derived_holiday_for_week`, `week_extra_hours`,
gefiltert nach `ReportType::AbsenceHours`), muss so hochgezogen werden, dass beide Zweige sie
lesen können (Pitfall 2 — Struct-Feld fehlt sonst im Scope, Compile-Fehler).

**Nicht-dynamischer/positiver Zweig — die vorhandene Reduktionspipeline, als Vorbild für die
neue parallele Berechnung** (Zeilen 1451-1499):
```rust
let week_extra_hours: Vec<_> = extra_hours_array.iter()
    .filter(|eh| eh.date_time.iso_week() == week && eh.date_time.year() as u32 == year)
    .collect();
// ... absense_hours (ReportType::AbsenceHours) ...
let derived_holiday_for_week: f32 = derived_holiday.iter()
    .filter(|(date, _)| { let w = ShiftyDate::from(**date).as_shifty_week();
        w.year == year && w.week == week })
    .map(|(_, h)| h).sum();
let holiday_hours = manual_holiday_hours + derived_holiday_for_week;
let absense_hours = absense_hours + derived_holiday_for_week;
```

**Fold-Akkumulation** (Zeilen ~1517-1543) — jedes `WeeklyHours`-Feld hat eine `acc.X += week.X`
Zeile; ein neues Feld braucht dieselbe Ergänzung.

**Final-Berechnung + Push** (Zeilen ~1540-1665):
```rust
let expected_hours = weekly_hours.planned_hours - weekly_hours.absense_hours
    - weekly_hours.absence_derived_balance_hours;
let dynamic_hours = weekly_hours.dynamic_hours - weekly_hours.absense_hours
    - weekly_hours.absence_derived_balance_hours;
// ...
short_employee_report.push(ShortEmployeeReport {
    sales_person: Arc::new(paid_employee.clone()),
    balance_hours, dynamic_hours, expected_hours, overall_hours,
    // ...
    has_pending_rebooking, pending_rebooking_id,
});
```
→ `planning_target_hours` analog aus `weekly_hours.planning_target_hours` (neues
`WeeklyHours`-Feld) minus dem parallel, ungegateten Reduktionsterm berechnen und hier
ergänzen.

---

### `service/src/reporting.rs::ShortEmployeeReport` (model)

**Analog** (Zeilen 147-170), Feld-Deklaration + Kommentar-Präzedenz für additive Felder:
```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ShortEmployeeReport {
    pub sales_person: Arc<SalesPerson>,
    pub balance_hours: f32,
    pub dynamic_hours: f32,
    pub expected_hours: f32,
    pub overall_hours: f32,
    // ...
    // Phase 55 (HR-ALERT-01/03, 55-03): backend-berechneter Alert-Flag ...
    pub has_pending_rebooking: bool,
    pub pending_rebooking_id: Option<Uuid>,
}
```
→ `pub planning_target_hours: f32,` als neues Feld ergänzen, mit eigenem Kommentar analog zum
`has_pending_rebooking`-Block, der auf Phase 64 / PSOLL-01..04 verweist und erklärt, dass das
Feld unabhängig von `is_dynamic` gefüllt wird und die Balance nicht beeinflusst.

---

### `rest-types/src/lib.rs::ShortEmployeeReportTO` (model, DTO)

**Analog** (Zeilen 380-410):
```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShortEmployeeReportTO {
    pub sales_person: SalesPersonTO,
    pub balance_hours: f32,
    pub expected_hours: f32,
    pub dynamic_hours: f32,
    pub overall_hours: f32,
    #[serde(default)]
    pub volunteer_hours: f32,
    // Phase 55 (HR-ALERT-01/03, 55-03): ... #[serde(default)] ...
    #[serde(default)]
    pub has_pending_rebooking: bool,
    #[serde(default)]
    pub pending_rebooking_id: Option<Uuid>,
}

#[cfg(feature = "service-impl")]
impl From<&service::reporting::ShortEmployeeReport> for ShortEmployeeReportTO {
    fn from(report: &service::reporting::ShortEmployeeReport) -> Self {
        Self {
            sales_person: SalesPersonTO::from(report.sales_person.as_ref()),
            balance_hours: report.balance_hours,
            expected_hours: report.expected_hours,
            dynamic_hours: report.dynamic_hours,
            overall_hours: report.overall_hours,
            volunteer_hours: report.volunteer_hours,
            has_pending_rebooking: report.has_pending_rebooking,
            pending_rebooking_id: report.pending_rebooking_id,
        }
    }
}
```
→ Neues Feld `#[serde(default)] pub planning_target_hours: f32,` im TO ergänzen
(bereits `ToSchema`-derived am Struct — Anforderung aus CLAUDE.md ist damit automatisch
erfüllt, keine separate Annotation nötig) und in `From`-Impl mappen:
`planning_target_hours: report.planning_target_hours,`.

---

### `service_impl/src/test/reporting_planning_target.rs` (NEU — test)

**Analog 1 — Mock-Boilerplate** (`reporting_additive_merge.rs:100-176`):
```rust
struct ReportingMocks { /* ... alle Service-Mocks ... */ }
struct TestDeps;
impl ReportingServiceDeps for TestDeps { /* Transaction/Context-Typalias */ }
impl ReportingMocks {
    fn new() -> Self { /* Default-Mocks, toggle_service off */ }
    fn build(self) -> ReportingServiceImpl<TestDeps> { /* Arc::new(...) je Feld */ }
}
```
1:1 kopierbar — identischer Deps-Satz für beide Konstruktionsstellen.

**Analog 2 — Cross-Site-Paritätstest-Skelett** (`reporting_additive_merge.rs:962-1091`,
Funktion `test_balance_parity_dynamic_all_employees`):
```rust
#[tokio::test]
async fn test_balance_parity_dynamic_all_employees() {
    let vacation_day = time::macros::date!(2024 - 06 - 03);
    let mut mocks_a = ReportingMocks::new();
    mocks_a.permission_service.expect_check_permission().returning(|_, _| Ok(()));
    mocks_a.sales_person_service.expect_get_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_sales_person()])));
    mocks_a.employee_work_details_service.expect_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_work_details_dynamic_mon_fri()])));
    // ... extra_hours/shiftplan/carryover/tx/absence mocks ...
    let service_a = mocks_a.build();
    let reports_a = service_a
        .get_reports_for_all_employees(2024, 23, service::permission::Authentication::Full, None)
        .await.expect("...");
    let report_a = &reports_a[0];
    // Lauf B mit anderer Datenquelle (derived statt extra_hours) ...
    assert!((report_a.expected_hours - report_b.expected_hours).abs() < 0.01, "...");
}
```
Für PSOLL-03 direkt übertragbar: **statt** zwei Läufe mit unterschiedlicher Datenquelle
(`extra_hours` vs. `derive_hours_for_range`) braucht der neue Test **einen** Fixture-Satz,
zweimal durch die **beiden Entry-Points** (`get_week` und `get_reports_for_all_employees`)
geschickt, und `planning_target_hours` der Resultate verglichen — Grundgerüst
(Mocks/Fixture-Aufbau) bleibt identisch, nur der Vergleichsgegenstand ändert sich von
„Lauf A vs Lauf B (gleicher Entry-Point)" zu „Entry-Point A vs Entry-Point B (gleicher Lauf)".

**Analog 3 — Fixture-Spiegelpaar** (`reporting_phase2_fixtures.rs`, komplett, keine Änderung
nötig):
```rust
pub fn fixture_work_details_8h_mon_fri() -> EmployeeWorkDetails { /* is_dynamic: false, 40h */ }
pub fn fixture_work_details_dynamic_mon_fri() -> EmployeeWorkDetails {
    EmployeeWorkDetails { id: ..., is_dynamic: true, ..fixture_work_details_8h_mon_fri() }
}
```
Beide Fixtures direkt importierbar, kein neuer Fixture-Bedarf laut RESEARCH.md und
VALIDATION.md — Wave-0-Gap bestätigt das explizit.

**Registrierung:** neue Datei muss in `service_impl/src/test/mod.rs` als `mod
reporting_planning_target;` eingetragen werden (wie alle Schwester-Testdateien im selben
Verzeichnis — Muster dort mit `grep -n "mod " service_impl/src/test/mod.rs` bestätigen).

**TDD-Hinweis:** `.planning/config.json` hat `tdd_mode: true` — Kopfkommentar der neuen Datei
sollte wie in `reporting_additive_merge.rs` explizit "RED vor Task X, GREEN nach Task X"
markieren.

---

### `shifty-dioxus/src/component/working_hours_mini_overview.rs` (component, request-response)

**Analog — bestehender Fat-Backend-Kommentar + drei Lesestellen** (bestätigter Fund: drei,
nicht vier — siehe Korrektur oben):

**Stelle 1 — CardsLayout, Zeile 104-113:**
```rust
let sales_person_id = working_hour.sales_person_id;
let actual = working_hour.actual_hours;
// TARGET = expected_hours (das feiertags-/absence-reduzierte Soll), NICHT
// dynamic_hours: der Band-Guard (HSP-03) haelt dynamic_hours frei vom automatisch
// angerechneten Feiertag ...
let target = working_hour.expected_hours;
```

**Stelle 2 — TableLayout Summenzeile, Zeile 170-171:**
```rust
// TARGET total uses expected_hours (holiday-/absence-reduced Soll) — see the per-row note.
let total_target: f32 = props.rows.iter().map(|r| r.expected_hours).sum();
```

**Stelle 3 — TableLayout pro Zeile, Zeile 211-216:**
```rust
// TARGET = expected_hours (das feiertags-/absence-reduzierte Soll), NICHT
// ... (identischer Kommentar wie Stelle 1)
let target = working_hour.expected_hours;
```

Bei Umsetzung von D-64-01 (alle drei auf `planning_target_hours` umstellen): jede Stelle
bekommt einen eigenen, aktualisierten Kommentar (kein Copy-Paste-Blindwechsel — RESEARCH.md
fordert "eine bewusste Entscheidung pro Stelle"), z. B. ersetzt "expected_hours (das
feiertags-/absence-reduzierte Soll)" durch "planning_target_hours (das Planungs-Soll,
feiertags-/absence-reduziert, auch für dynamische Verträge > 0, beeinflusst NICHT die
Balance)".

**Test-Helfer, die mitgeändert werden müssen (sonst Compile-Fehler durch fehlendes
Struct-Feld):**
```rust
// Zeile 381-388
fn make_row(name: &str, color: &str, actual: f32, target: f32) -> WorkingHoursMini {
    WorkingHoursMini {
        // ...
        expected_hours: target,
        dynamic_hours: target,
        // planning_target_hours: target,  <- muss ergänzt werden
    }
}
// Zeile 473-477 und weitere direkte WorkingHoursMini{...}-Literale im Testmodul
// (grep "WorkingHoursMini {" in derselben Datei für alle Vorkommen)
```
Vorgehen: `grep -n "WorkingHoursMini {" shifty-dioxus/src/component/working_hours_mini_overview.rs`
liefert alle Literal-Stellen im Testmodul, die um das neue Feld ergänzt werden müssen.

**Vorgelagerte Kette (nicht Teil dieser Datei, aber muss mitgezogen werden, sonst Compile-
Fehler kaskadiert):**
- `shifty-dioxus/src/state/employee_work_details.rs:8-30` — `WorkingHoursMini`-Struct selbst
  braucht das neue Feld + `Default::default()`-Eintrag.
- `shifty-dioxus/src/loader.rs:434-460` — `build_working_hours_mini` reicht Felder 1:1 durch
  (Fat-Backend-Konvention, PADM-06-Präzedenz), neues Feld analog zu `expected_hours`
  durchreichen, keine Ableitung.

## Shared Patterns

### Additives Feld an `ShortEmployeeReport`/`ShortEmployeeReportTO`
**Quelle:** `has_pending_rebooking`/`pending_rebooking_id` (Phase 55), Struct-Deklaration
`service/src/reporting.rs:163-169`, TO `rest-types/src/lib.rs:388-396`, `From`-Impl
`:398-410`.
**Anwenden auf:** `service/src/reporting.rs::ShortEmployeeReport`,
`rest-types/src/lib.rs::ShortEmployeeReportTO` + `From`-Impl.
```rust
// Struct (service):
pub has_pending_rebooking: bool,
pub pending_rebooking_id: Option<Uuid>,
// TO (rest-types):
#[serde(default)]
pub has_pending_rebooking: bool,
// From-Impl:
has_pending_rebooking: report.has_pending_rebooking,
```
**Abweichung von diesem Präzedenzfall:** anders als `has_pending_rebooking` (das laut Design
D-55-B in `assemble_weeks` bewusst neutral bleibt) MUSS `planning_target_hours` laut PSOLL-03
an **beiden** Konstruktionsstellen tatsächlich berechnet werden — Pitfall 4 in RESEARCH.md
warnt explizit davor, das Rebooking-Muster falsch zu übertragen.

### Zwei-Gate-Falle bei dynamischen Verträgen
**Quelle:** `service_impl/src/reporting.rs:1045-1084` (Gate `!has_contract_row ||
planned_hours <= 0.0`) und `:1411` (Gate `expected_hours <= 0.0`).
**Anwenden auf:** beide Konstruktionsstellen bei der Berechnung des neuen Reduktionsterms.
**Regel:** Das neue Gate für `planning_target_hours` darf **nur** `!has_contract_row` prüfen,
niemals einen `<= 0.0`-Zusatz — sonst wird die Reduktion für dynamische Verträge (bei denen
`planned_hours`/`expected_hours` immer 0 sind) strukturell übersprungen.

### `available_hours`-Regressionslock
**Quelle:** `service_impl/src/booking_information.rs:644` und `:818`:
```rust
available_hours: report.expected_hours,
```
**Anwenden auf:** diese zwei Zeilen bewusst NICHT anfassen — Pitfall 3. Ein Regressionstest
(`test_available_hours_unchanged_dynamic`, siehe VALIDATION.md) hält das ab.

## No Analog Found

Keine — alle sechs Änderungsstellen haben einen unmittelbaren, direkten Analog im selben File
(bestehendes Parallel-Feld) oder in einer bereits für genau diesen Zweck gebauten
Test-Utility-Datei.

## Metadata

**Analog search scope:** `service_impl/src/reporting.rs`, `service/src/reporting.rs`,
`rest-types/src/lib.rs`, `service_impl/src/test/reporting_additive_merge.rs`,
`service_impl/src/test/reporting_phase2_fixtures.rs`,
`shifty-dioxus/src/component/working_hours_mini_overview.rs`
**Files scanned:** 6 (alle vollständig oder gezielt in nicht-überlappenden Bereichen gelesen)
**Pattern extraction date:** 2026-09-29
