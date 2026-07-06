# Phase 53: freiwilligen-abwesenheiten-jahresansicht — Pattern Map

**Mapped:** 2026-07-06
**Files analyzed:** 8
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `service/src/booking_information.rs` | Service-Trait-Struct | CRUD | `WorkingHoursPerSalesPerson` (gleiche Datei, Zeile 24–35) | exact |
| `service_impl/src/booking_information.rs` (get_weekly_summary Fill-Site) | Service-Impl-Fill-Site | request-response | `committed_voluntary_hours`-Formel Zeile 509–517 (gleiche Datei) | exact |
| `service_impl/src/booking_information.rs` (get_summery_for_week Fill-Site) | Service-Impl-Fill-Site | request-response | `absent_volunteer_ids`-Block aus `get_weekly_summary` Zeile 422–442 | role-match |
| `rest-types/src/lib.rs` | DTO + From-Impl | request-response | `WorkingHoursPerSalesPersonTO` + `From<&WorkingHoursPerSalesPerson>` (Zeile 955–987) | exact |
| `shifty-dioxus/src/state/weekly_overview.rs` | FE-State-Mapper | transform | bestehender `sales_person_absences`-Merge-Loop Zeile 47–62 (gleiche Datei) | exact |
| `service_impl/src/test/booking_information_vaa.rs` | Test | CRUD | `service_impl/src/test/booking_information_vfa.rs` (komplette Datei) | exact |
| `service_impl/src/test/mod.rs` | Test-Registry | — | Zeile 6 `pub mod booking_information_vfa;` (gleiche Datei) | exact |
| OpenAPI-Registry | keine Aktion nötig | — | `WeeklySummaryTO` hat kein `ToSchema`; `SalesPersonAbsenceTO` braucht kein ApiDoc-Eintrag | — |

---

## Pattern Assignments

### 1. `service/src/booking_information.rs` — neues Struct + neues Feld auf `WeeklySummary`

**Rolle:** Service-Trait-Struct
**Analog:** `WorkingHoursPerSalesPerson` + `WeeklySummary` in derselben Datei (Zeile 24–55)

**Original-Code-Excerpt** (Zeile 24–55):
```rust
#[derive(Clone, Debug, PartialEq)]
pub struct WorkingHoursPerSalesPerson {
    pub sales_person_id: Uuid,
    pub sales_person_name: Arc<str>,
    pub available_hours: f32,
    // ... weitere Felder
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeeklySummary {
    pub year: u32,
    pub week: u8,
    // ...
    pub working_hours_per_sales_person: Arc<[WorkingHoursPerSalesPerson]>,
}
```

**Ziel-Delta:**
```rust
// Nach `WorkingHoursPerSalesPerson` einfügen — neuer Struct (D-53-01):
#[derive(Clone, Debug, PartialEq)]
pub struct SalesPersonAbsence {
    pub sales_person_id: Uuid,
    pub name: Arc<str>,
    pub hours: f32,
}

// In WeeklySummary nach `working_hours_per_sales_person` neues Feld:
pub working_hours_per_sales_person: Arc<[WorkingHoursPerSalesPerson]>,
pub sales_person_absences: Arc<[SalesPersonAbsence]>,  // NEU D-53-01
```

**Arc<str>-Konsistenz:** `name: Arc<str>` — analog zu `sales_person_name: Arc<str>` in `WorkingHoursPerSalesPerson`.

---

### 2. `service_impl/src/booking_information.rs` — Fill-Site `get_weekly_summary`

**Rolle:** Service-Impl-Fill-Site (Assembly-Loop)
**Analog:** `committed_voluntary_hours`-Formel Zeile 509–517 + `absent_volunteer_ids`-Block Zeile 422–442

**Schritt A — `all_sales_persons` Refactor (Zeile 290–297, Pitfall 1 vermeiden):**

Original (Zeile 290–297):
```rust
let volunteer_ids: Arc<[Uuid]> = self
    .sales_person_service
    .get_all(Authentication::Full, tx.clone().into())
    .await?
    .iter()
    .filter(|sales_person| !sales_person.is_paid.unwrap_or(false))
    .map(|sales_person| sales_person.id)
    .collect();
```

Ziel-Delta:
```rust
// Einmal laden, zweimal nutzen — kein zweites get_all():
let all_sales_persons = self
    .sales_person_service
    .get_all(Authentication::Full, tx.clone().into())
    .await?;
let volunteer_ids: Arc<[Uuid]> = all_sales_persons
    .iter()
    .filter(|sales_person| !sales_person.is_paid.unwrap_or(false))
    .map(|sales_person| sales_person.id)
    .collect();
// `all_sales_persons` wird im Per-Woche-Loop für name-Lookup verwendet.
```

**Schritt B — VAA-02-Formel nach `committed_voluntary_hours` (Zeile 517):**

Analog (Zeile 509–517 — Präzedenzformel):
```rust
let committed_voluntary_hours: f32 = find_working_hours_for_calendar_week(
    &all_work_details,
    year,
    week,
)
.filter(|wh| wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0)
.filter(|wh| !absent_volunteer_ids.contains(&wh.sales_person_id)) // absent → 0
.map(|wh| wh.committed_voluntary)
.sum();
```

Neuer Code (nach Zeile 517, im Per-Woche-Loop):
```rust
// VAA-01/02 (D-53-01/02): Freiwilligen-Absencen für DTO-Feld befüllen.
// Identische Cap-Gate-Filterung wie committed_voluntary_hours (Zeile 509–517),
// aber OHNE absent-Filter — hier sind gerade die Abwesenden gesucht.
// Pitfall 2: wh.sales_person_id == sp_id-Filter ist Pflicht (Iterator ist personenübergreifend).
let sales_person_absences: Arc<[service::booking_information::SalesPersonAbsence]> =
    absent_volunteer_ids
        .iter()
        .filter_map(|&sp_id| {
            let name = all_sales_persons
                .iter()
                .find(|sp| sp.id == sp_id)
                .map(|sp| sp.name.clone())?;
            let hours: f32 = find_working_hours_for_calendar_week(&all_work_details, year, week)
                .filter(|wh| {
                    wh.sales_person_id == sp_id
                        && (wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0)
                })
                .map(|wh| wh.committed_voluntary)
                .sum();
            Some(service::booking_information::SalesPersonAbsence {
                sales_person_id: sp_id,
                name,
                hours,
            })
        })
        .collect();
```

**Schritt C — `WeeklySummary { ... }` Literal (Zeile 620–636) um neues Feld erweitern:**

Original (Zeile 620–636):
```rust
weekly_report.push(WeeklySummary {
    year,
    week,
    // ...
    working_hours_per_sales_person: working_hours_per_sales_person.into(),
    // ...
});
```

Ziel-Delta:
```rust
weekly_report.push(WeeklySummary {
    // alle bestehenden Felder unverändert
    working_hours_per_sales_person: working_hours_per_sales_person.into(),
    sales_person_absences,  // NEU
    // ...
});
```

---

### 3. `service_impl/src/booking_information.rs` — Fill-Site `get_summery_for_week`

**Rolle:** Service-Impl-Fill-Site (Single-Week-Variante, D-53-06)
**Analog:** `absent_volunteer_ids`-Block aus `get_weekly_summary` Zeile 422–442

**Schritt A — `all_sales_persons` laden (nach `volunteer_ids`-Block, Zeile 666–673):**

Analog zu `get_weekly_summary`-Refactor (oben):
```rust
// Nach dem bestehenden volunteer_ids-Block:
let all_sales_persons = self
    .sales_person_service
    .get_all(Authentication::Full, tx.clone().into())
    .await?;
let volunteer_ids: Arc<[Uuid]> = all_sales_persons
    .iter()
    .filter(|sp| !sp.is_paid.unwrap_or(false))
    .map(|sp| sp.id)
    .collect();
```

**Schritt B — `all_absences` laden (neu, analog zu `get_weekly_summary` Zeile 308–311):**

Analog (Zeile 308–311):
```rust
let all_absences = self
    .absence_service
    .find_all(Authentication::Full, tx.clone().into())
    .await?;
```

**Schritt C — `absent_volunteer_ids` inline bauen (identisch zu `get_weekly_summary` Zeile 422–442):**

Analog (Zeile 422–442):
```rust
let absent_volunteer_ids: std::collections::HashSet<Uuid> =
    if let (Ok(week_monday), Ok(week_sunday)) = (
        time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday),
        time::Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday),
    ) {
        all_absences
            .iter()
            .filter(|period| {
                volunteer_ids.contains(&period.sales_person_id)
                    && period_overlaps_week(
                        period.from_date,
                        period.to_date,
                        week_monday,
                        week_sunday,
                    )
            })
            .map(|period| period.sales_person_id)
            .collect()
    } else {
        std::collections::HashSet::new()
    };
```

**Schritt D — `sales_person_absences` befüllen (identisch zu Fill-Site A, Schritt B):**

Gleiche VAA-02-Formel wie oben; `all_work_details` muss ebenfalls geladen werden — analog:
```rust
let all_work_details = self
    .employee_work_details_service
    .all(Authentication::Full, tx.clone().into())
    .await?;
```

**Schritt E — `WeeklySummary`-Literal (Zeile 901–921) erweitern:**

Original (Zeile 901–921):
```rust
let summary = WeeklySummary {
    // ...
    committed_voluntary_hours: 0.0,
    working_hours_per_sales_person: working_hours_per_sales_person.into(),
    // ...
};
```

Ziel-Delta:
```rust
let summary = WeeklySummary {
    // alle bestehenden Felder
    committed_voluntary_hours: 0.0,
    working_hours_per_sales_person: working_hours_per_sales_person.into(),
    sales_person_absences,  // NEU
    // ...
};
```

---

### 4. `rest-types/src/lib.rs` — neues DTO-Struct + `WeeklySummaryTO`-Feld + From-Impl

**Rolle:** DTO + From-Impl
**Analog A (Struct mit ToSchema):** `WorkingHoursPerSalesPersonTO` Zeile 955–966
**Analog B (From-Impl mit iter().map().collect()):** `From<&WeeklySummary> for WeeklySummaryTO` Zeile 1009–1036
**Analog C (`#[serde(default)]`):** `committed_voluntary_hours` Zeile 997–998

**Original Analog A (Zeile 955–966):**
```rust
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct WorkingHoursPerSalesPersonTO {
    pub sales_person_id: Uuid,
    pub sales_person_name: Arc<str>,
    // ...
}
```

**Neues Struct (nach `WorkingHoursPerSalesPersonTO`, vor `WeeklySummaryTO`):**
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct SalesPersonAbsenceTO {
    pub sales_person_id: Uuid,
    pub name: Arc<str>,
    pub hours: f32,
}

#[cfg(feature = "service-impl")]
impl From<&service::booking_information::SalesPersonAbsence> for SalesPersonAbsenceTO {
    fn from(a: &service::booking_information::SalesPersonAbsence) -> Self {
        Self {
            sales_person_id: a.sales_person_id,
            name: a.name.clone(),
            hours: a.hours,
        }
    }
}
```

**Original Analog C — `#[serde(default)]` Pattern (Zeile 997–998):**
```rust
#[serde(default)]
pub committed_voluntary_hours: f32,
```

**`WeeklySummaryTO` — neues Feld (nach `working_hours_per_sales_person` Zeile 1006):**
```rust
pub working_hours_per_sales_person: Arc<[WorkingHoursPerSalesPersonTO]>,
#[serde(default)]  // Pitfall 3: ohne default = Deser-Fehler für alte JSON-Antworten
pub sales_person_absences: Arc<[SalesPersonAbsenceTO]>,  // NEU D-53-01
```

**Original Analog B — From-Impl (Zeile 1026–1033):**
```rust
working_hours_per_sales_person: weekly_summary
    .working_hours_per_sales_person
    .iter()
    .map(|working_hours_per_sales_person| {
        WorkingHoursPerSalesPersonTO::from(working_hours_per_sales_person)
    })
    .collect::<Vec<_>>()
    .into(),
```

**Neues Feld im From-Impl (nach dem obigen Block):**
```rust
sales_person_absences: weekly_summary
    .sales_person_absences
    .iter()
    .map(SalesPersonAbsenceTO::from)
    .collect::<Vec<_>>()
    .into(),
```

---

### 5. `shifty-dioxus/src/state/weekly_overview.rs` — FE-State-Mapper Union-Refactor

**Rolle:** FE-State-Mapper
**Analog:** bestehender `sales_person_absences`-Block Zeile 47–62 (gleiche Datei) + neues DTO-Feld

**Original-Code-Excerpt (Zeile 47–62):**
```rust
sales_person_absences: summary
    .working_hours_per_sales_person
    .iter()
    .filter_map(|sp| {
        let effective_absence =
            sp.absence_hours - sp.holiday_hours + sp.unavailable_hours;
        if effective_absence >= 0.1 {
            Some(SalesPersonAbsence {
                name: sp.sales_person_name.clone(),
                absence_hours: effective_absence,
            })
        } else {
            None
        }
    })
    .collect(),
```

**Wichtig:** `WeeklySummaryTO` in der `make_to()`-Fixture (Zeile 83–101 desselben Files,
Testabschnitt) muss ebenfalls das neue `sales_person_absences`-Feld bekommen:
```rust
// In make_to()-Fixture (Zeile ~99):
working_hours_per_sales_person: Vec::new().into(),
sales_person_absences: Vec::new().into(),  // NEU — sonst compile error nach DTO-Erweiterung
```

**Ziel-Delta (D-53-04/05 Union-Merge):**
```rust
sales_person_absences: {
    // Bezahlte — bestehende Logik, für Regression-Lock VAA-03 #3 UNVERÄNDERT:
    let mut v: Vec<SalesPersonAbsence> = summary
        .working_hours_per_sales_person
        .iter()
        .filter_map(|sp| {
            let effective_absence =
                sp.absence_hours - sp.holiday_hours + sp.unavailable_hours;
            if effective_absence >= 0.1 {
                Some(SalesPersonAbsence {
                    name: sp.sales_person_name.clone(),
                    absence_hours: effective_absence,
                })
            } else {
                None
            }
        })
        .collect();
    // Freiwillige aus neuem DTO-Feld (D-53-01); Filter >= 0.1 = Randfall Zusage=0 raus (D-53-04):
    v.extend(
        summary
            .sales_person_absences
            .iter()
            .filter(|a| a.hours >= 0.1)
            .map(|a| SalesPersonAbsence {
                name: a.name.clone(),
                absence_hours: a.hours,
            }),
    );
    // Sort case-insensitive nach Name (D-53-04); Vec::sort_by ist stabil (Namensduplikate OK):
    v.sort_by(|x, y| x.name.to_lowercase().cmp(&y.name.to_lowercase()));
    v
},
```

---

### 6. `service_impl/src/test/booking_information_vaa.rs` — neue Test-Datei VAA-03

**Rolle:** Test
**Analog:** `service_impl/src/test/booking_information_vfa.rs` (komplette Datei)

**Bestätigte Existenz:** Datei liegt unter
`/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/test/booking_information_vfa.rs`

**TestDeps-Block (Zeile 45–66 aus vfa.rs — 1:1 übernehmen):**
```rust
struct TestDeps;

impl BookingInformationServiceDeps for TestDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type ShiftplanReportService = MockShiftplanReportService;
    type SlotService = MockSlotService;
    type ShiftplanService = service::shiftplan_catalog::MockShiftplanService;
    type BookingService = MockBookingService;
    type SalesPersonService = MockSalesPersonService;
    type SalesPersonUnavailableService = MockSalesPersonUnavailableService;
    type ReportingService = MockReportingService;
    type SpecialDayService = MockSpecialDayService;
    type ToggleService = MockToggleService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type AbsenceService = MockAbsenceService;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = dao::MockTransactionDao;
}
```

**Fixture-Konstanten für VAA-03:**
```rust
const YEAR: u32 = 2026;
const WEEK_UNDER_TEST: u8 = 20;  // 2026-W20 (Mon May 11 – Sun May 17)

fn volunteer_id_absent() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_VAA0_0001)
}
fn volunteer_id_present() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_VAA0_0002)
}
fn paid_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_VAA0_0003)
}
```

**SalesPerson-Fixtures:** Analog `volunteer_sales_person()` aus vfa.rs (Zeile 88–99),
mit `is_paid: Some(false)` für Freiwillige, `is_paid: Some(true)` für Bezahlten.

**EmployeeWorkDetails-Fixture:** Analog `volunteer_work_details()` aus vfa.rs (Zeile 106–133),
`cap_planned_hours_to_expected: true`, `committed_voluntary: 5.0`, Zeitraum 2026-W01..2027-W03.

**AbsencePeriod-Fixture:** Analog `volunteer_absence_period()` aus vfa.rs (Zeile 138–153),
`sales_person_id: volunteer_id_absent()`, Woche W20.

**Mock-Setup-Pattern (Zeile 193–299 aus vfa.rs — vollständig übernehmen):**
- `permission_service.expect_check_permission().returning(|_, _| Ok(()))`
- `sales_person_service.expect_get_all().returning(...)` — liefert alle 3 Persons
- `employee_work_details_service.expect_all().returning(...)` — liefert Fixture
- `absence_service.expect_find_all().returning(...)` — nur W20-Period für `volunteer_id_absent()`
- `toggle_service.expect_get_toggle_value().returning(|_, _, _| Ok(None))`
- `special_day_service.expect_get_by_week()/get_by_iso_year()` — leer
- `reporting_service.expect_get_year()` — leere Reports pro Woche
- `shiftplan_report_service.expect_extract_shiftplan_report_for_iso_year()` — leer
- `slot_service.expect_get_slots().returning(...)` + `get_slots_for_week_all_plans` — leer
- `shiftplan_service_mock.expect_get_all()` — leer
- `transaction_dao.expect_use_transaction()/commit()` — passthrough

**Service-Konstruktion (Zeile 304–316 aus vfa.rs — 1:1 übernehmen):**
```rust
let service = BookingInformationServiceImpl::<TestDeps> {
    shiftplan_report_service: Arc::new(shiftplan_report_service),
    slot_service: Arc::new(slot_service),
    shiftplan_service: Arc::new(shiftplan_service_mock),
    booking_service: Arc::new(MockBookingService::new()),
    sales_person_service: Arc::new(sales_person_service),
    // ... alle weiteren Felder analog
};
```

**3 Test-Funktionen (VAA-03-Assertions):**
```rust
#[tokio::test]
async fn vaa03_volunteer_with_period_appears_with_correct_hours() {
    // ... setup wie oben
    let summaries = service.get_weekly_summary(YEAR, Authentication::Full, None).await.unwrap();
    let week = summaries.iter().find(|s| s.week == WEEK_UNDER_TEST).unwrap();
    assert!(week.sales_person_absences.iter().any(|a| a.sales_person_id == volunteer_id_absent()));
    let entry = week.sales_person_absences.iter()
        .find(|a| a.sales_person_id == volunteer_id_absent()).unwrap();
    assert!((entry.hours - 5.0).abs() < 0.001);
}

#[tokio::test]
async fn vaa03_volunteer_without_period_not_in_list() {
    // ... setup: volunteer_id_present() hat keine AbsencePeriod
    let week = ...;
    assert!(!week.sales_person_absences.iter().any(|a| a.sales_person_id == volunteer_id_present()));
}

#[tokio::test]
async fn vaa03_paid_employee_unchanged_regression_lock() {
    // ... setup: paid_id() hat is_paid=true + report mit absence_hours
    let week = ...;
    assert!(!week.sales_person_absences.iter().any(|a| a.sales_person_id == paid_id()));
    assert!(week.working_hours_per_sales_person.iter().any(|wh| wh.sales_person_id == paid_id()));
}
```

---

### 7. `service_impl/src/test/mod.rs` — Registrierung

**Rolle:** Test-Registry
**Analog:** Zeile 6 in derselben Datei

```rust
// Analog zu bestehenden Zeilen 5–6:
#[cfg(test)]
pub mod booking_information_vfa;

// Neu einfügen (nach booking_information_vfa):
#[cfg(test)]
pub mod booking_information_vaa;
```

---

### 8. OpenAPI-Schema-Registrierung

**Aktion: keine erforderlich.**

Begründung (verifiziert aus `rest-types/src/lib.rs` Zeile 989):
- `WeeklySummaryTO` hat **kein** `#[derive(..., ToSchema)]` — es ist kein OpenAPI-Schema-Objekt.
- `SalesPersonAbsenceTO` bekommt `ToSchema` per D-53-01 (als Struct-Annotation), aber da
  `WeeklySummaryTO` nicht in einer `ApiDoc`-Struct registriert ist, braucht `SalesPersonAbsenceTO`
  keinen separaten ApiDoc-Eintrag.
- `WorkingHoursPerSalesPersonTO` hat `ToSchema` (Zeile 955), ist aber auch nicht explizit in
  einer zentralen `#[openapi(components(schemas(...)))]`-Liste eingetragen — es wird inline
  über `WeeklySummaryTO` konsumiert, ohne registriert zu sein.

**Kein Handlungsbedarf** für `rest/src/*.rs`.

---

## Shared Patterns

### Arc<str> in Struct-Feldern

**Quelle:** `WorkingHoursPerSalesPerson.sales_person_name: Arc<str>` (service/src/booking_information.rs:27)
**Anwenden auf:** `SalesPersonAbsence.name` (Service-Layer) + `SalesPersonAbsenceTO.name` (DTO-Layer)
```rust
pub name: Arc<str>,  // nicht String, nicht &str
```

### `#[serde(default)]` für additive DTO-Felder

**Quelle:** `rest-types/src/lib.rs` Zeile 997–998
```rust
#[serde(default)]
pub committed_voluntary_hours: f32,
```
**Anwenden auf:** `WeeklySummaryTO.sales_person_absences` — verhindert Deser-Fehler bei alten
JSON-Responses ohne das neue Feld (Pitfall 3).

### `find_working_hours_for_calendar_week` — niemals neu implementieren

**Quelle:** `crate::reporting` — bereits importiert in `booking_information.rs` Zeile 2.
**Anwenden auf:** VAA-02-Formel in beiden Fill-Sites. Kein eigener Date-Range-Iterator.

### `period_overlaps_week` — niemals neu implementieren

**Quelle:** `booking_information.rs:81` (lokaler Helper).
**Anwenden auf:** `absent_volunteer_ids`-Bau in `get_summery_for_week` (Pitfall 6: kein
eigener Date-Range-Overlap-Check).

### Mock-Setup-Template für BookingInformationService-Tests

**Quelle:** `service_impl/src/test/booking_information_vfa.rs` Zeile 193–316
**Anwenden auf:** neue Test-Datei `booking_information_vaa.rs` — komplettes Template kopieren,
nur Sales-Persons + AbsencePeriods anpassen.

---

## No Analog Found

Keine — alle 8 Zieldateien haben direkte Analogien im Codebase.

---

## Metadata

**Analog search scope:** `service/`, `service_impl/`, `rest-types/`, `shifty-dioxus/src/state/`, `shifty-dioxus/src/page/`
**Files scanned:** 8 direkte Code-Reads + vfa.rs vollständig analysiert
**Pattern extraction date:** 2026-07-06
