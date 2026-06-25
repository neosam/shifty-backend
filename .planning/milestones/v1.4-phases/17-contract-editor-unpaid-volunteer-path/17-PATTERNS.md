# Phase 17: Contract editor input + „alle"-Filter / unpaid-volunteer path - Pattern Map

**Mapped:** 2026-06-24
**Files analyzed:** 7 (4 Frontend, 3 Backend)
**Analogs found:** 7 / 7

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `shifty-dioxus/src/state/employee_work_details.rs` | state/model | transform (TryFrom) | sich selbst (Erweiterung des bestehenden Patterns) | exact |
| `shifty-dioxus/src/component/contract_modal.rs` | component | request-response | sich selbst (`expected_hours` TextInput, Z.298–319) | exact |
| `shifty-dioxus/src/component/employees_list.rs` | component | request-response | `src/page/billing_period_details.rs` (`show_paid`-Toggle Z.54, 318–391) | role-match |
| `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs` | config/i18n | — | sich selbst (`Key::ShowPaid`/`Key::ShowUnpaid`, `Key::Committed`-Block Z.814–843) | exact |
| `service_impl/src/booking_information.rs` | service | request-response | sich selbst (erste `get_weekly_summary`-Variante Z.136–295) | exact |
| `service_impl/src/reporting.rs` | service | request-response | sich selbst (`get_week` Z.709–808) + `get_reports_for_all_employees` Z.139–204 | exact |
| `service_impl/src/test/booking_information.rs` | test | — | sich selbst (CVC-06/Band-2-Test-Suite Z.1–392) | exact |

---

## Pattern Assignments

### `shifty-dioxus/src/state/employee_work_details.rs` (state, transform)

**Analog:** sich selbst — Erweiterung der bestehenden TryFrom-Blöcke

**Struct-Felder** (Z.43–69, `EmployeeWorkDetails`):
```rust
#[derive(PartialEq, Clone, Debug)]
pub struct EmployeeWorkDetails {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub expected_hours: f32,
    // ... alle Wochentag-Bools ...
    pub dynamic: bool,
    pub cap_planned_hours_to_expected: bool,
    pub vacation_days: u8,
    // Phase 17: NEU hinzufügen direkt nach cap_planned_hours_to_expected:
    // pub committed_voluntary: f32,
    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
```

**blank_standard-Muster** (Z.72–99) — Pflicht: neues Feld initialisieren:
```rust
pub fn blank_standard(sales_person_id: Uuid) -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::nil(),
        sales_person_id,
        expected_hours: 0.0,
        // ... alle Felder ...
        cap_planned_hours_to_expected: false,
        // Phase 17 NEU:
        committed_voluntary: 0.0,
        vacation_days: 0,
        created: None,
        deleted: None,
        version: Uuid::nil(),
    }
}
```

**TryFrom TO→State** (Z.145–183) — D-02, Richtung 1:
```rust
impl TryFrom<&EmployeeWorkDetailsTO> for EmployeeWorkDetails {
    type Error = ComponentRange;
    fn try_from(details: &EmployeeWorkDetailsTO) -> Result<Self, ComponentRange> {
        Ok(Self {
            // ... alle bestehenden Felder ...
            cap_planned_hours_to_expected: details.cap_planned_hours_to_expected,
            // Phase 17 NEU (ersetzt fehlendes Feld):
            committed_voluntary: details.committed_voluntary,
            vacation_days: details.vacation_days,
            // ...
        })
    }
}
```

**TryFrom State→TO** (Z.185–231) — D-02, Richtung 2 (Phase-17-Gap bei Z.218):
```rust
impl TryFrom<&EmployeeWorkDetails> for EmployeeWorkDetailsTO {
    type Error = ComponentRange;
    fn try_from(details: &EmployeeWorkDetails) -> Result<Self, ComponentRange> {
        Ok(Self {
            // ... alle bestehenden Felder ...
            cap_planned_hours_to_expected: details.cap_planned_hours_to_expected,
            // Phase 17: ersetze Hardcode Z.218 durch echtes Feld:
            // ALT: committed_voluntary: 0.0,
            committed_voluntary: details.committed_voluntary,
            vacation_days: details.vacation_days,
            // ...
        })
    }
}
```

**Pitfall:** Beide TryFrom-Richtungen MÜSSEN gepatcht werden. Der Hardcode Z.218 sitzt nur im zweiten Block (State→TO). Der erste Block (TO→State) ist bei Z.145 und fehlt das Feld komplett — es muss eingefügt werden.

---

### `shifty-dioxus/src/component/contract_modal.rs` (component, request-response)

**Analog:** sich selbst — `expected_hours`-TextInput Z.296–319 als 1:1-Vorlage; Cap-Toggle Z.380–398 liefert Sichtbarkeits-Signal

**Imports** (Z.1–20, unverändert):
```rust
use crate::component::form::{Field, FormCheckbox, TextInput};
use crate::i18n::Key;
use crate::service::{
    employee_work_details::EmployeeWorkDetailsAction,
    employee_work_details::EMPLOYEE_WORK_DETAILS_STORE, i18n::I18N,
};
use crate::state::employee_work_details::EmployeeWorkDetails;
```

**Dispatch-Pattern** (Z.155–157) — identisch für neues Feld:
```rust
let dispatch = move |updated: EmployeeWorkDetails| {
    work_details_service.send(EmployeeWorkDetailsAction::UpdateWorkingHours(updated));
};
```

**i18n-Label-Muster** (Z.135–145) — neue Keys analog dazu:
```rust
let expected_hours_label = ImStr::from(i18n.t(Key::ExpectedHoursPerWeekLabel).as_ref());
// Phase 17 NEU:
// let committed_voluntary_label = ImStr::from(i18n.t(Key::CommittedVoluntaryLabel).as_ref());
```

**Vorlage: expected_hours-TextInput** (Z.296–319) — 1:1 kopieren, Feld-Name austauschen:
```rust
// Numeric fields
div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
    Field {
        label: expected_hours_label,
        TextInput {
            value: ImStr::from(details.expected_hours.to_string()),
            input_type: ImStr::from("number"),
            step: Some(ImStr::from("0.01")),
            disabled: read_only,
            on_change: {
                let details = details.clone();
                move |value: ImStr| {
                    if read_only {
                        return;
                    }
                    if let Ok(n) = value.as_str().parse::<f32>() {
                        let mut next = details.clone();
                        next.expected_hours = n;  // → next.committed_voluntary = n;
                        dispatch(next);
                    }
                }
            },
        }
    }
    // ...
}
```

**D-01 Sichtbarkeits-Bedingung** — direkt vor dem committed-Field-Block:
```rust
// D-01: Feld sichtbar wenn cap=true ODER expected_hours=0 (rein freiwillig)
let show_committed = details.cap_planned_hours_to_expected || details.expected_hours == 0.0;

if show_committed {
    Field {
        label: committed_voluntary_label,
        TextInput {
            value: ImStr::from(details.committed_voluntary.to_string()),
            input_type: ImStr::from("number"),
            step: Some(ImStr::from("0.01")),
            disabled: read_only,
            on_change: {
                let details = details.clone();
                move |value: ImStr| {
                    if read_only { return; }
                    if let Ok(n) = value.as_str().parse::<f32>() {
                        let mut next = details.clone();
                        next.committed_voluntary = n;
                        dispatch(next);
                    }
                }
            },
        }
    }
}
```

**Cap-Toggle als Referenz für Signal** (Z.380–398):
```rust
div { class: "flex flex-col gap-1",
    FormCheckbox {
        value: details.cap_planned_hours_to_expected,
        // details.cap_planned_hours_to_expected ist das Signal für show_committed
        // ...
    }
    span { class: "text-small font-normal text-ink-muted", "{cap_help}" }
}
```

**Test-Pattern in dieser Datei** (Z.409–): SSR-Render über `dioxus_ssr::render(&vdom)`:
```rust
fn render(comp: fn() -> Element) -> String {
    let mut vdom = VirtualDom::new(comp);
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}
```

---

### `shifty-dioxus/src/component/employees_list.rs` (component, request-response)

**Analog:** `shifty-dioxus/src/page/billing_period_details.rs` — `show_paid`/`show_active`-Toggle mit `use_signal` + inline-Filter-Kette

**Bestehende Filter-Kette** (Z.81–88) — Einhängepunkt für is_paid-Filter:
```rust
let mut filtered: Vec<Employee> = list
    .iter()
    .filter(|e| !e.sales_person.inactive)
    // Phase 17 NEU: is_paid-Filter einfügen
    .filter(|e| matches_search(&e.sales_person.name, &term))
    .cloned()
    .collect();
```

**show_all Signal** — Analog zu `show_paid` in `billing_period_details.rs` Z.54:
```rust
// billing_period_details.rs Z.54–55 (Vorlage):
let mut show_paid = use_signal(|| true);   // Default: nur bezahlt
let mut show_active = use_signal(|| true); // Default: nur aktiv

// Phase 17 Variante (invertierte Logik — Default zeigt paid, Toggle zeigt all):
let mut show_all = use_signal(|| false);  // Default: false = nur bezahlt
```

**Checkbox-Toggle-Rendering** — Analog zu `billing_period_details.rs` Z.318–338:
```rust
// billing_period_details.rs Z.319–338 (Vorlage):
div { class: "flex gap-6",
    label { class: "flex items-center gap-2 text-body text-ink",
        input {
            r#type: "checkbox",
            class: "rounded border-border accent-accent",
            checked: *show_paid.read(),
            onchange: move |event| show_paid.set(event.checked()),
        }
        span { "{i18n.t(Key::ShowPaid)}" }
    }
}

// Phase 17 Variante:
label { class: "flex items-center gap-2 text-body text-ink",
    input {
        r#type: "checkbox",
        class: "rounded border-border accent-accent",
        checked: *show_all.read(),
        onchange: move |event| show_all.set(event.checked()),
    }
    span { "{i18n.t(Key::ShowUnpaid)}" }  // bestehender Key (de.rs Z.418)
}
```

**Filter-Logik** — Analog zu `billing_period_details.rs` Z.357–391:
```rust
// billing_period_details.rs Z.384–388 (Vorlage):
let paid_filter_matches = if show_paid_val {
    is_paid          // nur bezahlt
} else {
    true             // alle
};

// Phase 17 Variante (invertierte Semantik: show_all=false → nur bezahlt):
.filter(|e| *show_all.read() || e.sales_person.is_paid)
```

**KRITISCH — Backend-Ladelücke:** `loader::load_employees` (Z.335–342) ruft `api::get_short_reports` auf, was backend-seitig `get_reports_for_all_employees` nutzt. Diese Funktion filtert explizit auf `is_paid=true` (`reporting.rs` Z.164). Im `show_all`-Modus müssen unbezahlte Personen zusätzlich über `api::get_sales_persons` (Z.287–295) geladen werden.

**api::get_sales_persons** (Z.287–295 in `api.rs`):
```rust
pub async fn get_sales_persons(config: Config) -> Result<Rc<[SalesPersonTO]>, reqwest::Error> {
    let url = format!("{}/sales-person", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    Ok(res)
}
```

**use_resource-Muster** (Z.52–59 in `employees_list.rs`) — Vorlage für zweiten Resource-Call:
```rust
let employees = use_resource(move || {
    let _refresh_token = *EMPLOYEES_LIST_REFRESH.read();
    loader::load_employees(config.to_owned(), *year.read(), week_until)
});
```

**Employee-Struct aus `state/employee.rs`** — Dummy-Konstruktion für unbezahlte Personen analog zu:
```rust
// employees_list.rs Z.153–176 (Test zeigt Struct-Felder):
Employee {
    sales_person: SalesPerson::default(),
    working_hours_by_week: Rc::from([]),
    working_hours_by_month: Rc::from([]),
    overall_working_hours: 0.0,
    expected_working_hours: 0.0,
    balance: 0.0,
    // alle weiteren Felder auf 0.0/Rc::from([])
}
```

---

### `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs` (config, i18n)

**Analog:** sich selbst — bestehende `ShowPaid`/`ShowUnpaid`-Keys + `Committed`-Block als Referenz

**Key-Enum-Einfügung** (mod.rs Z.264–267, Kontext):
```rust
ShowActive,
ShowInactive,
ShowPaid,
ShowUnpaid,   // ← bereits vorhanden
// Phase 17 NEU: Key::CommittedVoluntaryLabel (oder ähnlich)
// Direkt nach ShowUnpaid oder thematisch passend
```

**Translations-Pattern de.rs** (Z.415–418):
```rust
i18n.add_text(Locale::De, Key::ShowActive, "Aktiv");
i18n.add_text(Locale::De, Key::ShowInactive, "Inaktive anzeigen");
i18n.add_text(Locale::De, Key::ShowPaid, "Bezahlt");
i18n.add_text(Locale::De, Key::ShowUnpaid, "Unbezahlte anzeigen");
// Phase 17 NEU (in de.rs, en.rs, cs.rs gleichzeitig):
// i18n.add_text(Locale::De, Key::CommittedVoluntaryLabel, "Freiwillig zugesagt (Zusage)");
```

**Locale-Matcher-Test-Pattern** (mod.rs Z.813–843) — für Phase-17-Keys kopieren:
```rust
#[test]
fn i18n_committed_keys_match_german_reference() {
    let i18n = generate(Locale::De);
    assert_eq!(i18n.t(Key::Committed).as_ref(), "Freiwillig zugesagt");
    assert_eq!(
        i18n.t(Key::PaidCommittedVolunteer).as_ref(),
        "Bezahlt / Freiwillig zugesagt / Freiwillig"
    );
}
// Phase 17: analog für Key::CommittedVoluntaryLabel in allen drei Locales
```

**KRITISCH — Locale-Swap-Pitfall** (aus Phase-16-Erfahrung): In `de.rs` immer `Locale::De` verwenden, NIE `Locale::En`. Schutzmechanismus: Per-Locale-Matcher-Test (s.o.) immer mitliefern.

---

### `service_impl/src/booking_information.rs` (service, request-response)

**Analog:** sich selbst — erste `get_weekly_summary`-Variante Z.136–295

**Gate-Erweiterung D-05** — zwei Stellen in der ersten Variante:

**Band-2-Loop** (Z.207–216):
```rust
let volunteer_hours = volunteer_surplus_band2(per_day_actuals, |sp_id| {
    find_working_hours_for_calendar_week(&all_work_details, year, week)
        .filter(|wh| {
            wh.sales_person_id == sp_id
                && wh.cap_planned_hours_to_expected  // CVC-06 per row — AKTUELL
                // Phase 17 D-05: && (wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0)
        })
        .map(|wh| wh.committed_voluntary)
        .sum()
});
```

**Band-1-Gate** (Z.219–226):
```rust
let committed_voluntary_hours: f32 = find_working_hours_for_calendar_week(
    &all_work_details,
    year,
    week,
)
.filter(|wh| wh.cap_planned_hours_to_expected) // CVC-06 gate — AKTUELL
// Phase 17 D-05: .filter(|wh| wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0)
.map(|wh| wh.committed_voluntary)
.sum();
```

**KRITISCH — Zweite Variante NICHT anfassen:** Es gibt eine zweite `get_weekly_summary`-Implementierung ab ca. Z.297 (Single-Week/Day-Level). Diese hat `committed_voluntary_hours: 0.0` als bewussten Placeholder (Phase-15-Kommentar). D-05 betrifft NUR die erste Variante (Z.136–295).

**volunteer_ids-Filter** (Z.159–166) — bestehend, beeinflusst Band-2-Akkumulation:
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

---

### `service_impl/src/reporting.rs` (service, request-response) — is_paid-Gating

**Site 1: `get_week`** (Z.709–808) — KEIN is_paid-Gate vorhanden (Haupt-Leak-Risiko):
```rust
async fn get_week(
    &self,
    year: u32,
    week: u8,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[ShortEmployeeReport]>, ServiceError> {
    let working_hours = self
        .employee_work_details_service
        .all_for_week(week, year, context.clone(), tx.clone())
        .await?
        .iter()
        .cloned()
        .collect_to_hash_map_by(|wh| wh.sales_person_id);
    // ... Schleife über alle working_hours (inkl. unbezahlte) ...
    for (sales_person_id, working_hours) in working_hours {
        // → Phase 17: Lookup SalesPerson.is_paid, skip wenn !is_paid
    }
}
```

**Site 3: `get_reports_for_all_employees`** (Z.139–204) — BEREITS korrekt gegated, als Referenz-Muster:
```rust
// reporting.rs Z.159–165 — Vorlage für is_paid-Gate-Stil:
let mut short_employee_report: Vec<ShortEmployeeReport> = Vec::new();
// ...
.filter(|employee| employee.is_paid.unwrap_or(false))
```

---

### `service_impl/src/test/booking_information.rs` (test)

**Analog:** sich selbst — bestehende Band-1/Band-2-Fixture-Suite

**Test-Struktur-Muster** (Z.1–17):
```rust
//! D-05 two-band fixture suite für committed_voluntary_hours (Band 1) + volunteer_hours (Band 2).

use crate::booking_information::{volunteer_surplus_above_committed, volunteer_surplus_band2};

/// Epsilon helper — niemals == für f32-Vergleiche
fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}
```

**Fixture-Pattern für neuen Gate-Test** (Z.112–130, `cvc06_cap_false_zero` als Vorlage):
```rust
#[test]
fn cvc06_cap_false_zero() {
    // Beschreibung des Szenarios als Kommentar
    let committed_after_cap_gate: f32 = 0.0;
    let actual: f32 = 7.0;

    let band2 = volunteer_surplus_above_committed(actual, committed_after_cap_gate);
    let band1 = committed_after_cap_gate;

    assert!(approx(band2, 7.0), "...fehlerbeschreibung..., got {band2}");
    assert!(approx(band1, 0.0), "...fehlerbeschreibung..., got {band1}");
}
```

**Phase-17-Neue Tests analog diesem Muster:**
- `d05_expected_hours_zero_flows_into_band1` — Person mit `expected_hours=0, cap=false, committed=5` → Band 1 erhält 5.0 (D-05-Gate greift)
- `get_week_unpaid_not_in_paid_hours` — Integration: `is_paid=false`-Person erscheint nicht in `paid_hours`
- `snapshot_schema_version_unchanged_at_7` — bleibt (Z.382–392), Phase-17-Kommentar aktualisieren

**Snapshot-Version-Regressionstest** (Z.382–392, unveränderter Pflicht-Test):
```rust
#[test]
fn snapshot_schema_version_unchanged_at_7() {
    assert_eq!(
        crate::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION,
        7
    );
}
```

---

## Shared Patterns

### Dispatch-Pattern (Frontend — für alle Feld-Mutations)
**Quelle:** `shifty-dioxus/src/component/contract_modal.rs` Z.155–157
**Anwenden auf:** `contract_modal.rs` neues committed-Feld
```rust
let dispatch = move |updated: EmployeeWorkDetails| {
    work_details_service.send(EmployeeWorkDetailsAction::UpdateWorkingHours(updated));
};
// In on_change: clone → mutate → dispatch(next)
let details = details.clone();
move |value: ImStr| {
    if read_only { return; }
    if let Ok(n) = value.as_str().parse::<f32>() {
        let mut next = details.clone();
        next.committed_voluntary = n;
        dispatch(next);
    }
}
```

### Boolean-Signal-Toggle (Frontend — für show_all)
**Quelle:** `shifty-dioxus/src/page/billing_period_details.rs` Z.54–55
**Anwenden auf:** `employees_list.rs` show_all-Toggle
```rust
let mut show_all = use_signal(|| false);
// Checkbox: onchange: move |event| show_all.set(event.checked())
// Filter: .filter(|e| *show_all.read() || e.sales_person.is_paid)
```

### SSR-Test-Pattern (Frontend)
**Quelle:** `shifty-dioxus/src/component/contract_modal.rs` Z.414–418
**Anwenden auf:** Neue Tests für Sichtbarkeits-Bedingung D-01
```rust
fn render(comp: fn() -> Element) -> String {
    let mut vdom = VirtualDom::new(comp);
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}
```

### is_paid-Gate (Backend — für at-risk-Sites)
**Quelle:** `service_impl/src/reporting.rs` Z.164 (`get_reports_for_all_employees`)
**Anwenden auf:** `get_week` in `reporting.rs` (Site 1)
```rust
.filter(|employee| employee.is_paid.unwrap_or(false))
// Variante für result-Vec-push am Ende des get_week-Loops:
// Lookup SalesPerson.is_paid vor dem result.push(...)
```

### f32-Epsilon-Vergleich (Backend-Tests)
**Quelle:** `service_impl/src/test/booking_information.rs` Z.15–17
**Anwenden auf:** Alle neuen f32-Assertions
```rust
fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}
```

### i18n-Locale-Matcher-Test (Frontend)
**Quelle:** `shifty-dioxus/src/i18n/mod.rs` Z.813–843
**Anwenden auf:** Neue Phase-17-Keys (CommittedVoluntaryLabel o.ä.)
```rust
#[test]
fn i18n_committed_keys_match_german_reference() {
    let i18n = generate(Locale::De);
    assert_eq!(i18n.t(Key::Committed).as_ref(), "Freiwillig zugesagt");
}
// Analog für Locale::En und Locale::Cs — alle drei Locales in je eigenem Test
```

---

## No Analog Found

Alle Phase-17-Dateien haben starke Analogs im bestehenden Codebase. Keine datei-ohne-Analog.

---

## Wichtige Anti-Pattern (aus RESEARCH.md)

| Anti-Pattern | Richtiges Muster | Analogquelle |
|-------------|-----------------|-------------|
| Nur eine TryFrom-Richtung patchen | Beide Blöcke Z.145 UND Z.185 anpassen | `employee_work_details.rs` Z.145 + Z.185 |
| `committed_voluntary: 0.0` in State→TO behalten | Durch `details.committed_voluntary` ersetzen | Z.218 (Gap-Kommentar) |
| D-05 in zweiter `get_weekly_summary`-Variante (Z.~337+) | Nur erste Variante (Z.207–226) | `booking_information.rs` Z.207+224 |
| Sichtbarkeit nur bei `cap` (nicht `expected_hours==0`) | `cap || expected_hours == 0.0` (D-01) | `contract_modal.rs` Cap-Toggle Z.382 |
| Frontend-Only-Filter für `show_all` | Zusätz­licher Backend-Call `GET /sales-person` | `api.rs::get_sales_persons` Z.287 |
| Gating auf Record-Präsenz statt `is_paid` | `sales_person.is_paid` direkt prüfen | `reporting.rs` Z.164 |
| Snapshot-Version bumpen | Version bleibt 7 (D-05 ist Achse-B-only) | `billing_period_report.rs` Z.75 |

---

## Metadata

**Analog-Suchbereich:** `shifty-dioxus/src/{component,state,page,i18n,api.rs,loader.rs}`, `service_impl/src/{booking_information,reporting,billing_period_report,vacation_balance}.rs`, `service_impl/src/test/`
**Dateien gescannt:** ~15
**Pattern-Extraktions-Datum:** 2026-06-24
