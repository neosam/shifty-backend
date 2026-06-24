# Phase 17: Contract editor input + „alle"-Filter / unpaid-volunteer path - Research

**Researched:** 2026-06-24
**Domain:** Dioxus Frontend (contract_modal.rs, employees_list.rs) + Backend Reporting Gate (booking_information.rs)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01 (Sichtbarkeits-Bedingung):** Das `committed_voluntary`-Feld ist sichtbar/editierbar wenn `cap_planned_hours_to_expected == true` ODER `expected_hours == 0`. Vorlage: bestehender `expected_hours`-TextInput (`input_type="number"`, `step="0.01"`, parse f32 → dispatch).

**D-02 (State-Threading):** `committed_voluntary` wird als Feld auf die Frontend-`EmployeeWorkDetails`-State-Struct gezogen und in beiden `TryFrom`-Richtungen durchgezogen. Der `committed_voluntary: 0.0`-Hardcode (~Zeile 218, `state/employee_work_details.rs`) wird durch den echten Feldwert ersetzt. Open→Save-unverändert-Round-Trip muss den Backend-Wert bewahren (CVC-09).

**D-03 (Filter-Semantik):** Default zeigt bezahlte Mitarbeiter; ein einblendbarer „alle"-Toggle deckt zusätzlich rein unbezahlte Freiwillige (`is_paid = false`) auf. Inaktive bleiben weiterhin ausgeblendet. Die Paid-Restriktion sitzt IM BACKEND (`get_reports_for_all_employees` filtert auf `is_paid = true`, Z.164); das Frontend filtert heute nur auf `!inactive` (Z.84 `employees_list.rs`).

**D-04 (Erzeugungs-Pfad):** Über den bestehenden Vertrags-Editor. `SalesPerson.is_paid = false` wird in `sales_person_details.rs` gesetzt (bestehende Checkbox). KEIN neues `is_paid`-Control im `contract_modal`. `EmployeeWorkDetails`-Record (`expected_hours = 0`, `committed_voluntary > 0`) entsteht über `contract_modal`.

**D-05 (Read-Gate erweitern, KEIN Snapshot-Bump):** Der committed_voluntary-Read-Gate in `booking_information.rs::get_weekly_summary` wird von `cap_planned_hours_to_expected == true` auf `cap_planned_hours_to_expected == true || expected_hours == 0` erweitert. KEIN `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump (bleibt 7). Betrifft nur `get_weekly_summary` (Achse B, erster Variant Zeilen ~207-280; der zweite Variant ~337-562 ist Single-Week / Day-Level und bleibt `committed_voluntary_hours: 0.0`-Placeholder).

**D-06 (Interaktion committed-Read vs. is_paid-Gate):** committed-Read-Gate (D-05) ist unabhängig vom is_paid-Gate. Unbezahlte Personen tragen ihre Zusage auf der freiwilligen Achse bei, während ihre 0 bezahlten Stunden durch is_paid-Gating NICHT in paid_hours/Billing lecken.

**D-07 (schlicht „0"):** `committed == 0` wird in der Mitarbeiteransicht schlicht als `0` (bzw. `🎯0.00`) gezeigt — KEINE blank/Strich-Sonderlogik.

### Claude's Discretion
- **is_paid-Gating-Stil (D-GATING-STYLE):** Planner entscheidet site-by-site (zentraler Helper vs. inline per Site). Invariante: jede at-risk-Site MUSS auf `sales_person.is_paid` gegated sein (nicht auf Record-Präsenz). Ein `get_week`-Seiteneffekt-Integrationstest sichert CVC-10 ab.

### Deferred Ideas (OUT OF SCOPE)
- Inline-Banner „Zusage nicht erfüllt" (committed > actual) → v1.5 (CVC-F-01)
- Blank/Strich-Darstellung statt „0" → final verworfen (D-07)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Beschreibung | Research-Support |
|----|-------------|------------------|
| CVC-09 | `committed_voluntary` im Vertrags-Editor (`contract_modal.rs`, neben dem Cap-Toggle) als numerisches Feld editierbar; Open→Save-unverändert-Round-Trip bewahrt den Backend-Wert. | State-Struct trägt das Feld bereits im Backend; Frontend-Gap in beiden `TryFrom`-Richtungen in `state/employee_work_details.rs` verifiziert (Z.218 Hardcode). |
| CVC-10 | Mitarbeiteransicht bekommt einen einblendbaren „alle"-Filter; unbezahlte Freiwillige können `EmployeeWorkDetails`-Record halten und werden sichtbar/auswählbar. Jede work-details-iterierende paid-only-Site explizit auf `sales_person.is_paid` gegated — kein Leak; `get_week`-Seiteneffekt durch Integrationstest abgesichert. | At-risk-Sites verifiziert (s.u.). `get_week` iteriert über `all_for_week` OHNE paid-Filter — Haupt-Leak-Risiko bestätigt. Billing-Pfad iteriert `get_all` ohne paid-Filter, but per-`SalesPerson`-Berechnungen in `build_billing_period_report_for_sales_person` produzieren dort bereits korrekte Werte (0 Stunden bei 0 Soll). `get_reports_for_all_employees` filtert explizit auf `is_paid=true` (Z.164) — sicher. |
</phase_requirements>

---

## Summary

Phase 17 schließt den v1.4-Milestone ab. Es gibt drei unabhängige Implementierungsachsen:

**Achse 1 — Frontend-Editor (CVC-09):** `committed_voluntary` fehlt als Feld auf der Frontend-`EmployeeWorkDetails`-State-Struct. Der Backend-Service-Struct trägt das Feld bereits vollständig (Phase 14). `EmployeeWorkDetailsTO` trägt es (`rest-types` Z.613). Beide `TryFrom`-Richtungen in `state/employee_work_details.rs` müssen ergänzt werden: `EmployeeWorkDetailsTO → EmployeeWorkDetails` (Z.145, Feld fehlt im Mapping) und `EmployeeWorkDetails → EmployeeWorkDetailsTO` (Z.218, Hardcode `committed_voluntary: 0.0`). Im `contract_modal.rs` wird ein neues `TextInput`-Block nach dem `expected_hours`-Feld eingefügt, sichtbar wenn `details.cap_planned_hours_to_expected || details.expected_hours == 0` (D-01). Dispatch-Muster: identisch zu `expected_hours` (clone → mutate → dispatch).

**Achse 2 — „alle"-Filter (CVC-10, Frontend):** Der Loader (`load_employees` in `loader.rs`) ruft `GET /report?year=...&until_week=...` auf, was backend-seitig `get_reports_for_all_employees` triggert. Diese Funktion filtert auf `is_paid=true` (Z.164). Unbezahlte Freiwillige mit `EmployeeWorkDetails`-Record tauchen dort nicht auf. Der Toggle muss entweder (a) einen separaten Lade-Pfad triggern (der `get_all` statt `get_reports_for_all_employees` nutzt) oder (b) das Backend muss eine neue Endpunktvariante bieten. Der sicherste Weg: `employees_list.rs` hält einen `show_all`-Bool-Signal; im Show-All-Modus werden unbezahlte Personen aus einem separaten API-Aufruf (`GET /sales-person`) geladen und mit leeren Stunden-Daten in die Liste eingefügt. Die `is_paid`-Gating-Invariante ist dabei unabhängig vom Anzeige-Filter.

**Achse 3 — Reporting-Gate-Erweiterung (D-05, Backend):** In `get_weekly_summary` (erste Variante, Z.224) lautet das aktuelle Gate `.filter(|wh| wh.cap_planned_hours_to_expected)`. Die Erweiterung auf `cap_planned_hours_to_expected || expected_hours == 0` lässt auch rein unbezahlte Freiwillige mit 0-Soll-Stunden in Band 1 einfließen. Betrifft exakt zwei `.filter`-Stellen (Z.212 für Band-2-Surplus-Berechnung, Z.224 für Band-1-committed). Version 7 bleibt.

**Primäre Empfehlung:** Achse 1 (Editor) und Achse 3 (Gate) sind rein Backend/Frontend-Code-Änderungen ohne neue Endpunkte. Achse 2 (Filter) erfordert eine Entscheidung, ob ein neuer Backend-Endpunkt nötig ist oder ob alle Sales Persons bereits über `GET /sales-person` geladen werden und der Filter rein im Frontend sitzt.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| committed_voluntary Editor-Feld | Frontend (component) | Frontend (state) | Reine UI + State-Threading; kein neuer Backend-Endpunkt nötig (EmployeeWorkDetailsTO bereits vollständig) |
| „alle"-Filter Toggle | Frontend (component) | Backend (API) | Filter-UI ist Frontend; die Frage, welche Personen geladen werden, bestimmt den Backend-Call |
| Unpaid-volunteer Sichtbarkeit | Backend (service/REST) | Frontend (loader) | `get_reports_for_all_employees` filtert auf is_paid; Entscheidung welche Personen zurückgegeben werden liegt im Backend |
| is_paid-Gating der at-risk-Sites | Backend (service_impl) | — | Invariante: kein Leak in paid_hours/Billing/Year-Summary |
| D-05 Read-Gate-Erweiterung | Backend (service_impl/booking_information.rs) | — | Reine Berechungslogik-Erweiterung in get_weekly_summary |

---

## Standard Stack

### Bereits etabliert (keine Änderung)

| Schicht | Crate / Modul | Pattern |
|---------|---------------|---------|
| State-Threading | `shifty-dioxus/src/state/employee_work_details.rs` | `TryFrom<&EmployeeWorkDetailsTO>` + `TryFrom<&EmployeeWorkDetails>` |
| Frontend-Dispatch | `contract_modal.rs` | `clone → mutate → EMPLOYEE_WORK_DETAILS_STORE coroutine send EmployeeWorkDetailsAction::UpdateWorkingHours` |
| i18n | `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs` | `Key` enum + `i18n.add_text(Locale::XX, Key::YY, "...")` in allen drei Locales |
| Backend Service | `service_impl/src/booking_information.rs` | `find_working_hours_for_calendar_week` + `.filter()` Gate |
| Tests | `service_impl/src/test/booking_information.rs` + `shifty-dioxus` binary tests | mockall (unit), in-memory SQLite (Integration), dioxus-ssr (Frontend) |

---

## Architecture Patterns

### System Architecture Diagram

```
[Frontend: EmployeesList]
  |-- show_all Signal (neu)
  |-- Filter: !inactive && (show_all || is_paid)
  |-- Loader: GET /report (paid only) + Optional: GET /sales-person (für Anzeige unpaid)
       |
       v
[Backend: GET /report → get_reports_for_all_employees]
  |-- Filter: is_paid=true (heute: Z.164; bleibt für paid_hours-Berechnungen)
  |-- Leak-safe: paid_hours-Loop berührt nie unbezahlte Personen

[Frontend: ContractModal → ContractModalBody]
  |-- EMPLOYEE_WORK_DETAILS_STORE.selected_employee_work_details
  |-- Details: EmployeeWorkDetails (State-Struct, NEU: +committed_voluntary)
  |-- Sichtbarkeit: cap || expected_hours == 0 (D-01)
  |-- Dispatch: EmployeeWorkDetailsAction::UpdateWorkingHours
       |
       v
[Backend: PUT /employee-work-details]
  |-- EmployeeWorkDetailsTO.committed_voluntary (bereits vorhanden, Phase 14)
  |-- DAO: UPDATE ... committed_voluntary = ? (bereits migriert)

[Backend: get_weekly_summary (erste Variante, Zeilen ~136-295)]
  |-- Band-1-Gate (D-05): cap_planned_hours_to_expected || expected_hours == 0 (NEU)
  |-- Band-2-Gate: identisch (NEU für Band-2-Surplus-Loop)
  |-- Ergebnis: committed_voluntary_hours fliesst für unbezahlte 0-Soll-Personen ein
       |
       v
[Frontend: WeeklyOverview] (Phase 16, abgeschlossen)
```

### Recommended Project Structure

```
shifty-dioxus/src/
├── component/
│   ├── contract_modal.rs       # +committed_voluntary TextInput (D-01/D-02)
│   └── employees_list.rs       # +show_all Toggle Signal (D-03)
├── state/
│   └── employee_work_details.rs # +committed_voluntary Feld + TryFrom-Gap schliessen
└── i18n/
    ├── mod.rs                  # +Key::CommittedVoluntaryLabel, Key::EmployeesShowAll o.ä.
    ├── de.rs                   # +neue Keys in Deutsch
    ├── en.rs                   # +neue Keys in Englisch
    └── cs.rs                   # +neue Keys in Tschechisch

service_impl/src/
└── booking_information.rs      # +D-05 Gate-Erweiterung (Zeilen ~212+224)

service_impl/src/test/
└── booking_information.rs      # +get_week-Seiteneffekt-Integrationstests (CVC-10)
```

### Pattern 1: committed_voluntary State-Threading

```rust
// Source: shifty-dioxus/src/state/employee_work_details.rs (VERIFIZIERT)

// Struct — NEU: Feld hinzufügen
pub struct EmployeeWorkDetails {
    // ...alle bestehenden Felder...
    pub committed_voluntary: f32,  // NEU
}

// TryFrom<&EmployeeWorkDetailsTO> — NEU: Feld aus TO lesen
impl TryFrom<&EmployeeWorkDetailsTO> for EmployeeWorkDetails {
    fn try_from(details: &EmployeeWorkDetailsTO) -> Result<Self, ComponentRange> {
        Ok(Self {
            // ...alle bestehenden Felder...
            committed_voluntary: details.committed_voluntary,  // NEU (statt 0.0)
        })
    }
}

// TryFrom<&EmployeeWorkDetails> für EmployeeWorkDetailsTO — Phase-17-Gap schliessen
impl TryFrom<&EmployeeWorkDetails> for EmployeeWorkDetailsTO {
    fn try_from(details: &EmployeeWorkDetails) -> Result<Self, ComponentRange> {
        Ok(Self {
            // ...alle bestehenden Felder...
            committed_voluntary: details.committed_voluntary,  // NEU (statt 0.0)
        })
    }
}

// blank_standard — NEU: Feld auf 0.0 initialisieren
pub fn blank_standard(sales_person_id: Uuid) -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        // ...
        committed_voluntary: 0.0,  // NEU
    }
}
```

### Pattern 2: committed_voluntary TextInput in ContractModalBody

```rust
// Source: shifty-dioxus/src/component/contract_modal.rs (VERIFIZIERT als Vorlage)
// Einhängepunkt: nach dem expected_hours-TextInput (~Z.319), vor oder nach
// vacation_days-Block, innerhalb des div class="grid grid-cols-1 md:grid-cols-2 gap-3"

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

### Pattern 3: D-05 Gate-Erweiterung in get_weekly_summary

```rust
// Source: service_impl/src/booking_information.rs (VERIFIZIERT)
// Beide Stellen in der ERSTEN Variante (~Z.207-280) anpassen:

// Band-2-Surplus-Loop (Z.~207-216):
let volunteer_hours = volunteer_surplus_band2(per_day_actuals, |sp_id| {
    find_working_hours_for_calendar_week(&all_work_details, year, week)
        .filter(|wh| {
            wh.sales_person_id == sp_id
                && (wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0) // D-05
        })
        .map(|wh| wh.committed_voluntary)
        .sum()
});

// Band-1-committed (Z.~219-226):
let committed_voluntary_hours: f32 = find_working_hours_for_calendar_week(
    &all_work_details, year, week,
)
.filter(|wh| wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0) // D-05
.map(|wh| wh.committed_voluntary)
.sum();
```

### Pattern 4: „alle"-Filter Toggle in EmployeesList

```rust
// Source: shifty-dioxus/src/component/employees_list.rs Z.82-88 (VERIFIZIERT)
// Bestehende Filter-Kette:
//   .filter(|e| !e.sales_person.inactive)
//   .filter(|e| matches_search(&e.sales_person.name, &term))
//
// D-03: show_all Signal + is_paid-Filter hinzufügen:

let show_all = use_signal(|| false);

// Im Render-Teil:
let mut filtered: Vec<Employee> = list
    .iter()
    .filter(|e| !e.sales_person.inactive)
    .filter(|e| *show_all.read() || e.sales_person.is_paid)  // NEU: D-03
    .filter(|e| matches_search(&e.sales_person.name, &term))
    .cloned()
    .collect();
```

**KRITISCH:** Das `Employee`-Struct kommt aus `load_employees`, das `GET /report` aufruft, was `get_reports_for_all_employees` triggert. Diese Funktion filtert backend-seitig auf `is_paid=true`. Daher gibt sie für unbezahlte Freiwillige KEINE Einträge zurück, selbst wenn der Frontend-Filter auf `show_all` steht. Für den „alle"-Modus braucht das Frontend entweder:

**Option A:** Separater Lade-Pfad für SalesPersons (`GET /sales-person`), merge mit bestehenden Employee-Daten.
**Option B:** Ein neuer Backend-Endpoint `GET /report/all` der alle Sales Persons zurückgibt (inkl. unbezahlte mit `expected_hours=0, balance=0` etc.).

Empfehlung (Claude's Discretion): **Option A** — `GET /sales-person` liefert alle Personen inkl. Paid-Flag. Im `show_all`-Modus werden zusätzlich Personen mit `is_paid=false && !inactive` aus dem Sales-Person-Endpoint geladen und als `Employee`-Dummies mit Null-Stunden in die Liste eingefügt. Das vermeidet einen neuen Backend-Endpoint und ist konsistent mit dem Prinzip „Backend-Reporting-Pfad ist immer paid-only".

### Anti-Patterns to Avoid

- **Gating auf Record-Präsenz statt `is_paid`:** Eine unbezahlte Person KANN einen `EmployeeWorkDetails`-Record haben (das ist D-04). Gating auf „hat work-details-Record" ist daher kein Paid-Proxy.
- **`expected_hours == 0` allein als Sichtbarkeits-Bedingung:** Auch eine gedeckelte Person kann `expected_hours = 0` haben in einem Kurzzeit-Vertrag. Die Bedingung ist `cap || expected_hours == 0` (D-01), nicht nur das eine.
- **`committed_voluntary` in der zweiten `get_weekly_summary`-Variante** (Z.~337-562, Single-Week / Day-Level) wiren: Diese Variante liefert Tages-Auflösung und ist laut Phase-15-CONTEXT bewusst `0.0`-Placeholder. Phase 17 berührt NICHT diese Variante.
- **Snapshot-Bump:** D-05 berührt keinen persistierten `BillingPeriodValueType`. Version bleibt 7.

---

## At-Risk-Sites für is_paid-Gating (CVC-10 — jede verifiziert)

### Site 1: `reporting.rs::get_week` (HAUPT-LEAK-RISIKO)

**Datei:** `service_impl/src/reporting.rs:709`
**Was sie macht:** Iteriert über `all_for_week(week, year)` — das sind ALLE Sales Persons mit einem aktiven Vertrag in dieser Woche (ohne paid-Filter, verifiziert Z.719). Sie berechnet für jede Person `dynamic_hours`, `balance_hours`, `volunteer_hours` etc.
**Ist heute paid-gegated?** NEIN. `all_for_week` liefert work-details für alle Personen, auch unbezahlte.
**Leak-Risiko:** Eine unbezahlte Freiwillige (`is_paid=false`, `expected_hours=0`) mit einem Vertrag würde in `get_week` einen `ShortEmployeeReport`-Eintrag produzieren. Dieser landet über `get_weekly_summary` (die `get_week` aufruft, Z.~181) in `paid_hours` (Z.250: `paid_hours += report.dynamic_hours`). Bei `expected_hours=0` ist `dynamic_hours=0.0` — kein numerischer Leak, aber der Person-Set-Kontext-Fehler bleibt.
**Gating-Stil:** Da `expected_hours=0` bei einer unbezahlten Person `dynamic_hours=0` ergibt, ist der numerische Paid-Hours-Leak heute faktisch 0. ABER: die Person erscheint in `WorkingHoursPerSalesPerson` (Z.257-267) wenn is_shiftplanner=true — das ist ein Personen-Set-Leak. Explizites `is_paid`-Gating in `get_week` (filtert die Result-Vec vor dem push) ist die saubere Lösung.
**CVC-10-Testanforderung:** Ein Integrationstest muss zeigen, dass eine unbezahlte Person mit `expected_hours=0, committed_voluntary=5` NICHT in `paid_hours` auftaucht.

### Site 2: `booking_information.rs::get_weekly_summary` — `paid_hours`-Akkumulation (Z.~250)

**Was sie macht:** Ruft `reporting_service.get_week()` auf (Site 1) und summiert `report.dynamic_hours` in `paid_hours`.
**Ist heute paid-gegated?** Indirekt: wenn Site 1 ungegated ist, landet 0.0 hier. Mit `expected_hours=0` kein numerischer Leak, aber Personen-Set-Inkonsistenz in `WorkingHoursPerSalesPerson`.
**Gating-Stil:** Abhängig von Site-1-Fix. Wenn Site 1 paid-filtert, ist Site 2 automatisch sicher.

### Site 3: `reporting.rs::get_reports_for_all_employees` (Z.139)

**Was sie macht:** Iteriert über alle Sales Persons und berechnet pro-Person Jahres-Reports.
**Ist heute paid-gegated?** JA — explizit `filter(|employee| employee.is_paid.unwrap_or(false))` Z.164.
**Gating-Stil:** Bereits korrekt. Keine Änderung nötig. [VERIFIED: service_impl/src/reporting.rs:162-165]

### Site 4: `billing_period_report.rs::build_new_billing_period` (Z.299)

**Was sie macht:** Ruft `get_all()` für Sales Persons (Z.318) und erstellt für JEDE Person (auch unbezahlte) einen `BillingPeriodSalesPerson`-Report.
**Ist heute paid-gegated?** NEIN — `get_all()` ohne paid-Filter (Z.318). [VERIFIED: billing_period_report.rs:316-319]
**Leak-Risiko:** Eine unbezahlte Person mit `expected_hours=0` produziert über `reporting_service.get_report_for_employee_range()` einen Report mit `volunteer_hours=0, overall_hours=0, balance_hours=0` etc. `BillingPeriodValueType::Volunteer` wird nur eingetragen wenn `!= 0.0` (Z.241). Numerisch: kein Leak bei 0-Soll-Personen. ABER: sie erscheinen als `BillingPeriodSalesPerson`-Einträge in der Billing-Liste.
**Gating-Entscheidung (Claude's Discretion):** Für Phase 17 reicht ein explizites `is_paid`-Gate vor dem Billing-Loop, um Person-Set-Konsistenz zu garantieren. Ohne dieses Gate sind Billing-Snapshots mit unbezahlten Personen angereichert, was zwar numerisch neutral ist (alle Werte 0), aber konzeptuell inkorrekt und schwer zu debuggen. [ASSUMED — Auswirkung auf bestehende Snapshots unklar, sollte User-Konfirmation für Gating im Billing-Pfad einholen wenn bereits Snapshots existieren]

### Site 5: `vacation_balance.rs::get_team` (Z.131)

**Was sie macht:** Ruft `get_all_paid()` auf (Z.146).
**Ist heute paid-gegated?** JA — `get_all_paid` ist explizit auf bezahlte Personen beschränkt. [VERIFIED: vacation_balance.rs:145-147]
**Gating-Stil:** Bereits korrekt. Keine Änderung nötig.

---

## Don't Hand-Roll

| Problem | Nicht selbst bauen | Stattdessen nutzen | Grund |
|---------|-------------------|-------------------|-------|
| Numerisches Input | Eigenes Number-Parsing | `TextInput { input_type: "number", step: "0.01" }` + `value.parse::<f32>()` | Vorlage: `expected_hours`-Block in `contract_modal.rs:300-318` |
| Toggle-State | Eigener Bool-Store | `use_signal(|| false)` | Standard Dioxus-Pattern; bereits überall im Codebase verwendet |
| is_paid-Lookup | Eigene Hilfstabelle | `sales_person.is_paid` direkt (liegt auf `SalesPerson` struct) | `SalesPerson.is_paid: bool` in `state/shiftplan.rs:128` |
| i18n-Keys in allen Locales | Nur Deutsch pflegen | Alle drei Locales gleichzeitig (`de.rs` + `en.rs` + `cs.rs`) | Pitfall: `Locale::En`-statt-`Locale::De`-Bug aus v1.3 |

---

## Common Pitfalls

### Pitfall 1: `committed_voluntary: 0.0`-Hardcode in TO-Richtung übersehen

**Was schiefgeht:** Nur die `TryFrom<&EmployeeWorkDetailsTO>` (TO→State) Richtung wird gefixed; die TO→ (State→TO) Richtung in `TryFrom<&EmployeeWorkDetails>` (Z.185-231) behält den `committed_voluntary: 0.0`-Hardcode.
**Warum:** Zwei separate `impl TryFrom`-Blöcke in derselben Datei; Lesen nur des ersten Blocks.
**Wie vermeiden:** Beide Blöcke (Z.145 und Z.185) explizit patchen. Round-Trip-Regressionstest pinnen.
**Frühwarnung:** CVC-09-Round-Trip-Test schlägt fehl: Wert öffnet korrekt, speichern setzt ihn auf 0.0 zurück.

### Pitfall 2: „alle"-Filter ohne Backend-Unterstützung

**Was schiefgeht:** Frontend-`employees_list.rs` setzt `show_all = true`, aber `load_employees` ruft weiterhin `GET /report` auf, das `get_reports_for_all_employees` nutzt — welche backend-seitig auf `is_paid=true` filtert (Z.164). Die Liste bleibt leer für unbezahlte Personen.
**Warum:** Der Filter sitzt im Backend, nicht im Frontend. Frontend-only-Filter reicht nicht.
**Wie vermeiden:** Im `show_all`-Modus einen zweiten API-Call auf `GET /sales-person` machen; unbezahlte nicht-inaktive Personen mit Null-Stunden-Dummy als `Employee` in die Liste einfügen.
**Frühwarnung:** Filter-Toggle aktiviert — Liste zeigt keine neuen Personen.

### Pitfall 3: Gate-Erweiterung D-05 in der falschen `get_weekly_summary`-Variante

**Was schiefgeht:** `booking_information.rs` enthält ZWEI `get_weekly_summary`-Varianten. Die zweite (Z.~337-562) ist der Single-Week/Day-Level-Pfad mit `committed_voluntary_hours: 0.0`-Placeholder (Phase-15-Kommentar Z.544-547). Wenn D-05 dort statt in der ersten Variante (Z.~136-295) angewendet wird, hat es keine Wirkung auf den Jahresansicht-Pfad.
**Warum:** Zwei identisch benannte Methoden in zwei `impl`-Blöcken; die erste (~Z.136) ist der Year-View-Pfad, die zweite (~Z.297) ist der Weekly/Day-Pfad.
**Wie vermeiden:** D-05 nur in der ersten Variante anwenden (Zeilenbereich ~207-226); Kommentar laut Phase-15-CONTEXT bestätigt: zweite Variante ist bewusst Placeholder.
**Frühwarnung:** Jahresansicht zeigt weiterhin 0 für unbezahlte Freiwillige trotz `committed_voluntary > 0`.

### Pitfall 4: Sichtbarkeits-Bedingung nicht symmetrisch zu Gate-Bedingung

**Was schiefgeht:** Editor-Feld sichtbar bei `cap`, aber Gate erweitert auf `cap || expected_hours == 0`. Dann können Personen mit `expected_hours == 0, cap=false` in der Jahresansicht erscheinen (Achse B), aber das Feld nicht editieren (Sichtbarkeits-Bedingung fehlt).
**Wie vermeiden:** D-01 (Sichtbarkeit) und D-05 (Gate) MÜSSEN dieselbe Bedingung `cap || expected_hours == 0` verwenden. Symmetrie ist Designziel (CONTEXT.md §Specific Ideas).

### Pitfall 5: `blank_standard()` ohne `committed_voluntary`

**Was schiefgeht:** `EmployeeWorkDetails::blank_standard(sales_person_id)` (Z.72-99) initialisiert die Struct ohne `committed_voluntary`-Feld, wenn es zur Struct hinzugefügt wird → Compile-Error oder fehlender Default 0.0.
**Wie vermeiden:** `blank_standard()` explizit um `committed_voluntary: 0.0` erweitern.

### Pitfall 6: i18n-Locale-Fehler (Locale::En statt Locale::De)

**Was schiefgeht:** In `de.rs` wird versehentlich `Locale::En` für neue Keys verwendet → Deutsche UI zeigt Englischen Text.
**Warum:** Bekannter Pitfall aus v1.3, dokumentiert in CLAUDE.md und Dioxus CLAUDE.md.
**Wie vermeiden:** Per-Locale-Reference-Matcher-Tests für neue Keys (Muster aus Phase 16: `i18n_committed_keys_match_german_reference` in `mod.rs` Z.814).

---

## Code Examples

### Bestehender expected_hours-TextInput (Vorlage für committed_voluntary)

```rust
// Source: shifty-dioxus/src/component/contract_modal.rs:298-319 [VERIFIED]
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
                if read_only { return; }
                if let Ok(n) = value.as_str().parse::<f32>() {
                    let mut next = details.clone();
                    next.expected_hours = n;
                    dispatch(next);
                }
            }
        },
    }
}
```

### Bestehender Cap-Toggle (Referenz für die Sichtbarkeits-Bedingung)

```rust
// Source: shifty-dioxus/src/component/contract_modal.rs:381-397 [VERIFIED]
// details.cap_planned_hours_to_expected ist der Bool-Wert der den committed-Block
// (D-01) steuert.
FormCheckbox {
    value: details.cap_planned_hours_to_expected,
    // ...
}
```

### TryFrom-Gap: Phase-17-Hardcode-Zeile

```rust
// Source: shifty-dioxus/src/state/employee_work_details.rs:213-218 [VERIFIED]
// Pre-existing gap (HEAD): the frontend EmployeeWorkDetails state does
// not yet carry committed_voluntary (editor wiring is Phase 17 scope).
committed_voluntary: 0.0,
// → Phase 17: ersetzen durch: committed_voluntary: details.committed_voluntary,
```

### Bestehende Filter-Kette in EmployeesList

```rust
// Source: shifty-dioxus/src/component/employees_list.rs:82-88 [VERIFIED]
let mut filtered: Vec<Employee> = list
    .iter()
    .filter(|e| !e.sales_person.inactive)
    .filter(|e| matches_search(&e.sales_person.name, &term))
    .cloned()
    .collect();
// → Phase 17: is_paid-Filter für show_all=false einfügen
```

### D-05 aktuelle Gate-Bedingung (zu erweitern)

```rust
// Source: service_impl/src/booking_information.rs:224 [VERIFIED]
.filter(|wh| wh.cap_planned_hours_to_expected) // CVC-06 gate, per row
// → Phase 17: .filter(|wh| wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0)
```

---

## State of the Art

| Alter Ansatz | Aktueller Stand | Geändert | Auswirkung |
|--------------|-----------------|---------|------------|
| `committed_voluntary: 0.0` Hardcode in TO-Richtung | Hardcode noch vorhanden (Phase-17-Gap-Kommentar Z.218) | Phase 14 hat Backend, Phase 17 schließt Frontend | Phase 17 muss beide TryFrom-Richtungen patchen |
| `get_weekly_summary` Gate: nur `cap == true` | Erweitern auf `cap || expected_hours == 0` | Phase 17 | Rein unbezahlte Freiwillige erscheinen in Jahresansicht-Kapazität |
| `employees_list.rs`: nur `!inactive` Filter | `!inactive && (show_all || is_paid)` | Phase 17 | Unbezahlte Freiwillige werden sichtbar |

**Noch nicht deprecated / unverändert:**
- `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7` — bleibt (D-05 ist Achse-B-only, kein persistierter value_type berührt)
- `get_reports_for_all_employees` `is_paid=true`-Filter — bleibt (correct by design)
- `vacation_balance.rs::get_team` `get_all_paid` — bleibt (correct by design)

---

## Assumptions Log

| # | Claim | Abschnitt | Risiko wenn falsch |
|---|-------|-----------|-------------------|
| A1 | `billing_period_report.rs::build_new_billing_period` iteriert über `get_all()` und produziert für eine unbezahlte 0-Soll-Person numerisch korrekte (0-Wert) Reports; ein `is_paid`-Gate vor dem Billing-Loop wäre optional für Phase 17 | At-Risk-Sites (Site 4) | Wenn bestehende Snapshots schon unbezahlte Personen enthalten und das Verhalten sich ändert, könnten Validatoren fehlschlagen |
| A2 | `GET /sales-person` liefert alle Personen inkl. `is_paid=false`-Freiwillige (nicht nur aktive) | „alle"-Filter / Achse 2 | Wenn der Endpoint aktive-only ist, braucht der Frontend-Loader eine andere Quelle |

---

## Open Questions (RESOLVED)

1. **Wie werden unbezahlte Freiwillige für den „alle"-Filter geladen?**
   - Was wir wissen: `GET /report` ist paid-only (backend-Filter). `GET /sales-person` liefert alle.
   - Was unklar: Ob `GET /sales-person` bereits vom Frontend verwendet wird und welches DTO verwendet wird.
   - Empfehlung: `GET /sales-person` für die vollständige Personenliste; im `show_all`-Modus werden is_paid=false+!inactive-Personen mit Null-Stunden-Dummy als `Employee` in die Liste eingefügt. Keine neuen Backend-Endpoints nötig.
   - **RESOLVED: Option A — `GET /sales-person` als zweiter Resource-Call, gemerged im Loader (Plan 17-04).**

2. **Soll `billing_period_report.rs::build_new_billing_period` ein `is_paid`-Gate bekommen?**
   - Was wir wissen: Numerisch kein Leak bei `expected_hours=0`. Person erscheint aber als `BillingPeriodSalesPerson`-Eintrag mit leeren Werten.
   - Empfehlung: Für Phase 17 ein explizites Gate hinzufügen um Person-Set-Konsistenz zwischen Billing und Year-Summary zu garantieren. Alternativ: als Low-Priority-Defizit dokumentieren und in v1.5 klären.
   - **RESOLVED: explizites `is_paid`-Gate ergänzt (Plan 17-01 Task 2).**

---

## Environment Availability

Step 2.6 SKIPPED — Phase 17 ist reine Code-/Config-Änderung ohne externe Tool-Dependencies. Alle nötigen Werkzeuge (nix develop, cargo, dx) sind in NixOS-Umgebung verfügbar.

---

## Validation Architecture

### Test Framework

| Eigenschaft | Wert |
|------------|------|
| Backend Framework | Rust `cargo test` (unit + in-memory SQLite Integration) |
| Frontend Framework | `cargo test` in `shifty-dioxus/` (dioxus-ssr Snapshot-Tests) |
| Quick-Run Backend | `nix develop --command cargo test -p service_impl` |
| Quick-Run Frontend | `cd shifty-dioxus && nix develop ../ --command cargo test` |
| Full Suite | `nix develop --command cargo test --workspace` + Frontend-Suite |
| WASM-Gate | `cd shifty-dioxus && nix develop ../ --command cargo build --target wasm32-unknown-unknown` |

### Baseline (2026-06-24 verifiziert)

| Suite | Tests |
|-------|-------|
| service_impl | 440 |
| shifty_bin | 61 |
| shifty-dioxus | 614 |
| Gesamt Workspace | ~590 (inkl. alle Crates) |

### Phase Requirements → Test Map

| Req ID | Verhalten | Testtyp | Automatisierter Befehl | Datei vorhanden? |
|--------|-----------|---------|----------------------|-----------------|
| CVC-09 | committed_voluntary überlebt Open→Save-Round-Trip | Unit-Test (State-Mapping) | `cargo test -p service_impl committed_voluntary` | ❌ Wave-0-Gap: neuer Test in `shifty-dioxus` |
| CVC-09 | committed_voluntary TextInput in ContractModalBody sichtbar wenn cap=true | SSR-Snapshot (dioxus-ssr) | `cd shifty-dioxus && cargo test committed` | ❌ Wave-0-Gap |
| CVC-09 | committed_voluntary TextInput in ContractModalBody sichtbar wenn expected_hours=0 | SSR-Snapshot | `cd shifty-dioxus && cargo test committed` | ❌ Wave-0-Gap |
| CVC-09 | committed_voluntary TextInput NICHT sichtbar wenn cap=false && expected_hours>0 | SSR-Snapshot | `cd shifty-dioxus && cargo test committed` | ❌ Wave-0-Gap |
| CVC-09 | TryFrom<&EmployeeWorkDetailsTO>→EmployeeWorkDetails: Feld korrekt gemappt | Unit-Test | `cd shifty-dioxus && cargo test employee_work_details` | ❌ Wave-0-Gap |
| CVC-10 | get_week: unbezahlte Freiwillige lecken NICHT in paid_hours | Mockall-Integrationstest | `cargo test -p service_impl get_week_unpaid` | ❌ Wave-0-Gap (neues Testmodul) |
| CVC-10 | D-05: committed_voluntary_hours für expected_hours=0-Person fließt in get_weekly_summary ein | Mockall-Integrationstest | `cargo test -p service_impl weekly_summary` | ❌ Wave-0-Gap |
| CVC-10 | Personen-Set-Konsistenz: is_paid=false erscheint nicht in WorkingHoursPerSalesPerson | Mockall-Test | `cargo test -p service_impl` | ❌ Wave-0-Gap |
| CVC-10 | CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 7 | Regressionstest | `cargo test -p service_impl schema_version` | ✅ Vorhanden (booking_information tests Z.9) |
| CVC-09 | i18n neue Keys (CommittedVoluntaryLabel etc.) in allen 3 Locales | Per-Locale-Matcher-Test | `cd shifty-dioxus && cargo test i18n_committed` | ❌ Wave-0-Gap (erweitern) |

### Wave 0 Gaps

- [ ] `service_impl/src/test/booking_information.rs` erweitern: `get_week`-Seiteneffekt-Test mit is_paid=false-Person (paid_hours-Leak-Check)
- [ ] `service_impl/src/test/booking_information.rs` erweitern: D-05-Gate-Test (expected_hours=0 → committed fließt in get_weekly_summary)
- [ ] `shifty-dioxus/src/state/employee_work_details.rs` — Prüfung ob existing TryFrom-Tests erweitert werden müssen (committed_voluntary Round-Trip)
- [ ] `shifty-dioxus/src/component/contract_modal.rs` — 3 SSR-Tests für Sichtbarkeits-Bedingungen (cap=true/false × expected_hours=0/>0)
- [ ] `shifty-dioxus/src/i18n/mod.rs` — Per-Locale-Reference-Matcher für neue Keys (Vorlage: `i18n_committed_keys_match_*_reference` Z.814-840)

*(CURRENT_SNAPSHOT_SCHEMA_VERSION=7-Regressionstest bereits vorhanden: `service_impl/src/test/booking_information.rs` enthält Kommentar Z.9 „regression test asserting CURRENT_SNAPSHOT_SCHEMA_VERSION stays 7")*

### Sampling Rate

- **Pro Task-Commit:** `cargo test -p service_impl` (440 Tests, ~2s) + `cd shifty-dioxus && cargo test` (614 Tests, <1s)
- **Pro Wave-Merge:** `nix develop --command cargo test --workspace` (vollständige Suite)
- **Phase-Gate:** Full suite grün + WASM-Build-Gate grün vor `/gsd-verify-work`

---

## Security Domain

`security_enforcement` ist in `.planning/config.json` nicht gesetzt → behandle als aktiviert.

### Applicable ASVS Categories

| ASVS Category | Betrifft Phase 17 | Standard Control |
|---------------|------------------|-----------------|
| V2 Authentication | nein | — |
| V3 Session Management | nein | — |
| V4 Access Control | ja | `is_paid`-Gating in Backend-Services; Permission-Checks (HR_PRIVILEGE, SHIFTPLANNER_PRIVILEGE) bereits vorhanden |
| V5 Input Validation | ja | `f32`-Parse in TextInput mit `if let Ok(n) = value.parse::<f32>()` — kein Panic, kein NaN-Leak |
| V6 Cryptography | nein | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard-Mitigation |
|---------|--------|---------------------|
| Unbezahlte Person erscheint in paid_hours via get_week | Elevation of Privilege | Explizites is_paid-Gate in get_week vor dem Result-Vec-push |
| `committed_voluntary` mit NaN/Infinity via TextInput | Tampering | Rust `parse::<f32>().is_ok()`-Gate; NaN-Guard analog `sanitize`-Funktion in billing_period_report.rs |

---

## Projekt-Constraints (aus CLAUDE.md)

| Constraint | Quelle | Relevanz für Phase 17 |
|-----------|--------|----------------------|
| Tests immer ausführen: `cargo test` + `cargo build` | CLAUDE.md global + shifty-backend | Pflicht nach jedem Plan |
| i18n: alle 3 Locales (En, De, Cs) gleichzeitig | shifty-backend/CLAUDE.md | Neue Keys für committed-Label + Filter-Toggle |
| Service-Tier: Basic vs. Business-Logic | shifty-backend/CLAUDE.md | `EmployeeWorkDetailsService` bleibt Basic; `ReportingService` / `BookingInformationService` bleibt Business-Logic |
| Transaction-Pattern: `Option<Transaction>` durchgängig | shifty-backend/CLAUDE.md | Alle Service-Calls in get_weekly_summary-Erweiterung |
| KEIN `git commit` aus Agents | CLAUDE.local.md | jj-Commits nur durch User manuell |
| NixOS: `nix develop` statt `nix-shell` | CLAUDE.local.md + MEMORY.md | Build-Commands immer via `nix develop` |
| WASM-Build-Gate | shifty-dioxus/CLAUDE.md | Abschluss-Check: `cargo build --target wasm32-unknown-unknown` |
| Snapshot-Versioning: Version 7 bleibt | CLAUDE.md + D-05 | Kein Bump; Kommentar in billing_period_report.rs Z.74 bestätigt |
| OpenAPI: `#[utoipa::path]` für neue REST-Endpoints | shifty-backend/CLAUDE.md | EmployeeWorkDetails-Endpoint-Familie hat bewusst keine OpenAPI-Annotation (per REQUIREMENTS.md Out-of-Scope); kein Phantom-Task |

---

## Sources

### Primary (HIGH confidence)

- `service_impl/src/booking_information.rs:136-620` — Beide `get_weekly_summary`-Varianten verifiziert; Gate-Bedingungen bei Z.212+224; `paid_hours`-Loop Z.250
- `service_impl/src/reporting.rs:709-902` — `get_week`-Implementierung: `all_for_week` ohne is_paid-Filter verifiziert (Z.719)
- `service_impl/src/reporting.rs:139-204` — `get_reports_for_all_employees` is_paid-Gate Z.164 verifiziert
- `service_impl/src/billing_period_report.rs:299-348` — `build_new_billing_period` `get_all()` ohne paid-Filter Z.318 verifiziert
- `service_impl/src/vacation_balance.rs:131-157` — `get_all_paid` für `get_team` Z.146 verifiziert
- `shifty-dioxus/src/state/employee_work_details.rs:44-231` — Struct-Feld fehlt; beide TryFrom-Richtungen verifiziert; Hardcode Z.218 verifiziert
- `shifty-dioxus/src/component/contract_modal.rs:297-407` — `expected_hours`-TextInput-Vorlage Z.300-318; Cap-Toggle Z.381-397 verifiziert
- `shifty-dioxus/src/component/employees_list.rs:82-88` — Filter-Kette ohne is_paid verifiziert
- `shifty-dioxus/src/loader.rs:335-342` — `load_employees` ruft `GET /report` verifiziert
- `rest-types/src/lib.rs:613` — `committed_voluntary: f32` auf `EmployeeWorkDetailsTO` vorhanden verifiziert
- `service/src/employee_work_details.rs:27` — `committed_voluntary: f32` auf Service-Struct vorhanden verifiziert
- `service_impl/src/billing_period_report.rs:75` — `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7` verifiziert
- `shifty-dioxus/src/i18n/mod.rs:266-267` — `Key::ShowPaid` + `Key::ShowUnpaid` bereits vorhanden; kein `EmployeesShowAll`-Key yet verifiziert

### Secondary (MEDIUM confidence)

- `service_impl/src/test/booking_information.rs:1-10` — Kommentar zu D-05 + Version-7-Regressionstest vorhanden (Datei verifiziert)
- `service_impl/src/test/reporting_additive_merge.rs` — Vorlage für Mockall-Teststruktur bei Reporting-Service-Tests verifiziert

### Tertiary (LOW confidence / [ASSUMED])

- A1: Billing-Pfad für unbezahlte Personen ist numerisch harmlos (0-Werte) — aus Code-Analyse abgeleitet, kein expliziter Test bestätigt das für den Phase-17-Fall
- A2: `GET /sales-person` liefert alle Personen inkl. unbezahlte — aus API-Aufruf-Struktur in `api.rs` plausibel, aber nicht explizit für den Filter-Fall verifiziert

---

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — alle Präzedenz-Code-Stellen direkt verifiziert
- Architecture: HIGH — beide TryFrom-Richtungen, alle at-risk-Sites, Gate-Zeilen im Code bestätigt
- Pitfalls: HIGH — aus Vorgänger-Phasen-Kommentaren + direkter Code-Verifikation
- At-Risk-Sites: HIGH — jede der 5 Sites einzeln im Quellcode verifiziert
- D-05 Gate-Erweiterung: HIGH — exakte Zeilen lokalisiert (Z.212 + Z.224)

**Research date:** 2026-06-24
**Valid until:** 2026-07-24 (stabile Architektur, wenig Drift erwartet)
