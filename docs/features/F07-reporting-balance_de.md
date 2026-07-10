# Feature: Reporting & Balance-Berechnung

> **Kurzform:** Rechnet für jeden bezahlten Mitarbeiter das Stundenkonto
> (Balance) über beliebige Zeitscheiben — pro Woche, pro Jahr, pro Range —
> und aggregiert dabei Buchungen, Extra-Hours, Absences, Carryover und
> Feiertage in einer einzigen Formel: *Balance = Ist − Erwartung + Extras*.

**Cluster-ID:** F07
**Status:** produktiv (Kern-Feature)
**Erstmalig eingeführt:** vor v1.0 (Stundenkonto-Ur-Feature); mehrfach
tiefgreifend überarbeitet (Phasen 8, 15, 17, 25, 34, 47, 51 sowie v2.2 RPT-01)
**Zuständige Crates:**
- `service::reporting`, `service::block`, `service::block_report`,
  `service::my_block`
- `service_impl::reporting` (2205 Zeilen, Business-Logic-Tier),
  `service_impl::block`, `service_impl::block_report`
- `rest::report`, `rest::block_report`, `rest::my_block`
- `rest-types` (`ShortEmployeeReportTO`, `EmployeeReportTO`,
  `WorkingHoursReportTO`, `BlockTO`, `EmployeeWeeklyStatisticsTO`,
  `EmployeeAttendanceStatisticsTO`)
- Frontend: `shifty-dioxus/src/page/weekly_overview.rs`,
  `shifty-dioxus/src/page/my_shifts.rs`, `shifty-dioxus/src/page/report.rs`
  (via `Employee`-Aggregat)

**Verwandt aber separat dokumentiert:**
- **Carryover** (Jahresrollover, Vorjahres-Saldo) → siehe
  [F06 Vacation Management](./F06-vacation-management.md), Sektion
  *Carryover*.
- **Billing Period Snapshots** (persistierte Perioden-Aggregate mit
  Schema-Versionierung) → siehe [F08 Billing Period](./F08-billing-period.md).
- **Extra Hours** (Legacy Zeit-Erfassung + Custom-Kategorien) → siehe
  [F04 Extra Hours](./F04-extra-hours.md).
- **Absence** (Range-basiertes Urlaubs-/Krankheits-System, v1.0+) →
  siehe [F05 Absence System](./F05-absence-system.md).

---

## 1. Was ist das? (Fachlich)

Das Reporting-Cluster ist **das Rechenwerk hinter dem Stundenkonto**. Es
beantwortet für jeden bezahlten Mitarbeiter die Kernfrage:
*„Wie viele Stunden hat er/sie geleistet, wie viele wurden erwartet, wie
viele Stunden Urlaub/Krank/Feiertag zählen mit — und was steht unterm
Strich?"*

Aus User-Sicht speist es drei Ansichten:

1. **HR-Übersicht (Report-Seite)** — Liste aller bezahlten Mitarbeiter mit
   Jahres-Balance, Ist-Stunden, Erwartungs-Stunden, Urlaubstagen usw.
2. **Employee-Detail** — pro Mitarbeiter eine Zeile pro Kalenderwoche
   (`by_week`) mit den Kategorien-Splits. Aus den Wochen werden Jahres-
   Summen aufgebaut.
3. **My-Shifts / Weekly-Overview** — die persönliche Sicht: „Was habe ich
   diese Woche gearbeitet, was steht im Balance-Konto, welche Blöcke
   habe ich als nächstes?"

Zusätzlich liefert das Cluster **Blöcke** (`Block`, `MyBlock`,
`BlockReport`): Zusammengefasste, zusammenhängende Buchungen pro
Wochentag — die operative Sicht auf „meine Schicht heute" oder „welche
Blöcke sind noch nicht ausreichend besetzt".

**Beispiel-Workflow aus User-Sicht:**

1. Mitarbeiter loggt sich ein → sieht auf `my_shifts` seine Blöcke der
   nächsten Wochen (kommen aus `MyBlockService` → `BlockService`).
2. HR öffnet Report-Seite eines Mitarbeiters → Frontend ruft
   `GET /report/{id}?year=…&until_week=…` → Backend rechnet:
   Bookings (aus `ShiftplanReportService`) + Extra-Hours (aus
   `ExtraHoursService`) + Absences (aus `AbsenceService.derive_hours…`) +
   Carryover (aus `CarryoverService`) + Feiertage (aus `SpecialDayService`,
   gegated durch `holiday_auto_credit`-Toggle) → gibt `EmployeeReport`
   zurück.
3. Frontend rendert die Balance in einer einzigen Zeile und den Split in
   `by_week`.

**Warum ist das das komplexeste Feature?** Weil es die Ausgabeschicht für
*alle* darunter liegenden Aggregate ist: jede Sonderregel (Cap,
Ehrenamt-ohne-Vertrag, dynamischer Vertrag, Feiertags-Auto-Credit,
Absence-Merge, Custom-Kategorien, Cutover-alt/neu) muss hier korrekt
zusammenkommen — und darf nicht mit den persistierten Snapshots
(`billing_period_sales_person`) driften.

---

## 2. Fachliche Regeln

### 2.1 Die Kern-Formel

Das Stundenkonto ist konzeptionell simpel; die Komplexität steckt in den
Quellen und Gates.

```
balance = worked_hours − expected_hours + carryover_prev_year
```

Mit:

```
worked_hours   = shiftplan_hours (gedeckelt, per-Woche)
               + extra_work_hours          # ExtraHoursCategory::ExtraWork
               + custom_working_hours      # modifies_balance == true

expected_hours = Σ (contract_expected_for_week
                     − absence_reducing_expected_for_week)

absence_reducing_expected
             = extra_hours(AbsenceHours)   # Vacation, SickLeave, Holiday, UnpaidLeave
             + derived_absence             # aus AbsenceService (V/S/U)
             + derived_holiday             # aus SpecialDayService, gegated

# Volunteer/Ehrenamt zählt NICHT in worked (aber wird ausgewiesen).
# Unavailable zählt weder in worked noch reduziert es expected.
```

Referenz (implementiert): `service_impl/src/reporting.rs:635`
(`balance_hours = overall_hours − expected_hours + previous_year_carryover`
für die Jahres-Übersicht) und `reporting.rs:1502` (`balance = shiftplan_paid
+ extra_work_hours − expected_hours + absence_hours` pro Woche).

> **Wichtig:** `expected_hours` wird in der Rechnung bereits *nach* dem
> Absence-Abzug geführt (`planned_hours − absence_hours` pro Woche); der
> Term `+ absence_hours` in der Wochen-Formel zaubert die Absence auf die
> Ist-Seite *nur für die Balance*, damit sie nicht doppelt reduziert.
> Konkret: das Modell ist mathematisch äquivalent zu
> `balance = worked_hours − (planned − absence) + carryover`.

### 2.2 `ExtraHoursCategory` — Semantik pro Kategorie

Definiert in `service/src/extra_hours.rs:41-97` über zwei Getter:
`as_report_type()` → `ReportType` und `availability()` → `Availability`.

| Kategorie | `ReportType` | reduziert `expected`? | erhöht `worked`? | ausgewiesen in Report als … |
| --- | --- | --- | --- | --- |
| `ExtraWork` | `WorkingHours` | nein | **ja** (`overall_hours`) | `extra_work_hours` |
| `Vacation` | `AbsenceHours` | **ja** | nein | `vacation_hours` (+ `vacation_days`) |
| `SickLeave` | `AbsenceHours` | **ja** | nein | `sick_leave_hours` (+ `sick_leave_days`) |
| `Holiday` | `AbsenceHours` | **ja** | nein | `holiday_hours` (+ `holiday_days`) |
| `UnpaidLeave` | `AbsenceHours` | **ja** | nein | `unpaid_leave_hours` (NICHT in `vacation_days`) |
| `Unavailable` | `None` | nein | nein | `unavailable_hours` (nur Info) |
| `VolunteerWork` | `Documented` | nein | nein | `volunteer_hours` |
| `CustomExtraHours(id)` — `modifies_balance=true` | `WorkingHours` | nein | **ja** | in `custom_extra_hours` und implizit in `overall_hours` |
| `CustomExtraHours(id)` — `modifies_balance=false` | `None` | nein | nein | nur in `custom_extra_hours` |

Das ist die verbindliche Tabelle. Wer eine neue Kategorie einführt, muss
beide Getter setzen und diese Zeile hier ergänzen (siehe
[edge-cases §1.5](../domain/edge-cases.md#15-balance-perimeter--was-zählt-zur-balance)).

**Sonderfall `UnpaidLeave`:** Reduziert die Erwartung, addiert nichts.
Das ist die einzige Kategorie, die `vacation_days()`-Berechnung **nicht**
beeinflusst, aber in `absence_days()` einfließt. Tests dazu:
`reporting.rs:1753-1849` (`test_unpaid_leave_tracked_separately`,
`test_unpaid_leave_does_not_affect_vacation_days`,
`test_unpaid_leave_included_in_absence_days`,
`test_unpaid_leave_reduces_expected_hours`).

### 2.3 `ExtraHoursReportCategory` — Reporting-Layer

Erweitert `ExtraHoursCategory` um genau eine Variante: `Shiftplan` (aus
`ShiftplanReportService` abgeleitete Tages-Buchungen). Alle acht
Extra-Hours-Kategorien werden 1:1 gemappt (`service/src/reporting.rs:26-41`).

Das TO (`ExtraHoursReportCategoryTO`) klappt `CustomExtraHours(LazyLoad)`
auf `Custom(Uuid)` platt (`rest-types/src/lib.rs:437`), weil auf der Wire
kein `LazyLoad` transportierbar ist.

### 2.4 Carryover als vor-persistierter Vorjahres-Saldo

Der `CarryoverService` liefert für `(sales_person_id, year - 1)` einen
Snapshot mit `carryover_hours` (f32) und `vacation` (i32).
`ReportingService` addiert `carryover_hours` **einmal** auf die
Jahres-Balance und `vacation` auf `vacation_entitlement`
(`reporting.rs:806-819, 844-853`).

- Der Wert **wird nicht neu berechnet**, wenn man in einem
  abgeschlossenen Jahr rückwirkend etwas ändert. Der Live-Report zeigt
  neue Wahrheit; der persistierte Carryover-Wert driftet.
  → siehe [edge-cases §1.1](../domain/edge-cases.md#11-carryover-grenze--jahresrollover).
- Der Carryover-Read läuft mit `Authentication::Full` (intern-Aggregat),
  damit auch beim HR-Aufruf ohne Sales-Person-Kontext geladen werden
  kann (`reporting.rs:811`).
- Wenn der Consumer nicht die Jahres-Aggregation braucht (Range-Report),
  kann `include_carryover: false` gesetzt werden — dann `carryover=0.0`.

### 2.5 Special-Days-Einfluss auf Erwartung

Feiertage senken die Erwartung auf zwei Wegen:

1. **Manueller Weg (klassisch):** HR trägt `ExtraHours(Holiday, 8h,
   2026-05-01, …)` ein → `Holiday` ist `AbsenceHours` → reduziert
   expected.
2. **Automatischer Weg (Phase 25, HOL-01/02, HCFG-01/03):**
   Der Toggle `holiday_auto_credit` speichert einen ISO-Stichtag. Für
   jeden Feiertag ≥ Stichtag baut `build_derived_holiday_map`
   (`reporting.rs:151-242`) für den Range/das Jahr einen Eintrag
   `(date, hours)` — mit *derived hours* = `EmployeeWorkDetails::holiday_hours()`
   (Vertragsstunden × 1/Wochentage). Diese Stunden gehen in
   `holiday_hours` UND in `absence_hours` (siehe *Pitfall 3* im Code:
   `reporting.rs:533`).

**Manual-wins-Regel (D-25-03 / HCFG-03):** Wenn für denselben Employee
+ selbem Tag bereits ein manuelles `ExtraHours(Holiday)` existiert, wird
der Auto-Credit übersprungen (`reporting.rs:218-224`). Umgekehrt gilt:
Auto-Credit wird nur eingetragen, wenn `wh.has_day_of_week(dow) && wh.holiday_hours() > 0`
— d.h. der Vertrag deckt diesen Wochentag ab.

**Cutoff-Gate:** Fehlt der Toggle-Wert (Automation aus) oder bekommt der
Toggle-Read `Unauthorized` (Mock/interne Aufrufer ohne User-Kontext),
liefert der Helper eine leere Map (`reporting.rs:169-179`). Das ist der
Legacy-Off-Zweig.

Für die *dynamische* Woche (kein/expected=0 Vertrag) wird derived
Holiday auf 0 gegated — sonst würde die Erwartung negativ und die Balance
aufblasen (`reporting.rs:1097-1098, 1406-1414`, „Dynamic-week guard").

### 2.6 Weitere Regeln, die nicht offensichtlich sind

- **Weekly Cap** (`cap_planned_hours_to_expected`, seit HRPX-01): wenn
  auf dem `EmployeeWorkDetails`-Record gesetzt und `shiftplan_hours >
  expected_hours`, wird der Überschuss in `auto_volunteer_hours` verschoben
  (`apply_weekly_cap`, `reporting.rs:124-137`). Der gedeckelte Wert ist die
  **einzige Quelle** für `overall_hours`/`balance_hours`/`shiftplan_hours`
  — der rohe ungedeckelte Wert existiert nur zwischenzeitlich
  (`reporting.rs:763-767, 785, 836-847`).
- **Kein-Vertrag-Woche** (User-Regel `quick-260624-ujk`): Fehlt komplett
  eine `EmployeeWorkDetails`-Zeile für die Woche → Shiftplan-Stunden
  gehen als Ehrenamt (`volunteer_hours`), nicht in `overall_hours`.
  Abgrenzung zum dynamischen Vertrag: dort *existiert* die Zeile, hat aber
  `expected_hours == 0` → dann gilt `Soll = Ist`, kein Ehrenamt-Umleiten.
  Siehe die Drei-Fälle-Unterscheidung in `hours_per_week`
  (`reporting.rs:1444-1454`) und die parallele Logik in `get_reports_for_all_employees`
  (`reporting.rs:388-500`) sowie `get_week` (`reporting.rs:1006-1156`).
- **Volunteer-Merge**: `volunteer_hours` einer Woche ist
  `manual_volunteer + auto_volunteer (cap) + no_contract_volunteer`
  (`reporting.rs:1539-1545`).
- **Dynamic-Guard auf Absence** (Phase 8.4 / CR-01, WR-01):
  Absence-Stunden (extra_hours + derived) reduzieren die Erwartung **nur**,
  wenn `working_hours_for_week > 0`. Sonst würde bei dynamischem Vertrag
  die Erwartung negativ (`reporting.rs:1378-1386, 1390-1398`).
- **Additive Merge Extra-Hours + Absence-derived** (Phase 8.4, D-01):
  Aus beiden Quellen wird per-Woche summiert. Konvertierte Extra-Hours
  sind vorher via `soft_delete_bulk` als deleted markiert, deshalb keine
  Doppelzählung (Verweis `reporting.rs:731-745`).
- **`by_week` als Single-Source-of-Truth** (UV-05, D-18-04):
  Ab Phase 18 werden die Top-Level-`vacation_hours`/`sick_leave_hours`/
  `holiday_hours`/`unpaid_leave_hours`/`volunteer_hours` durch Summe über
  `by_week` gefüllt — die alten Jahres-Lumps sind entfernt
  (`reporting.rs:861-874`). So schleicht sich keine Doppelzählung mehr ein.

---

## 3. Datenmodell

Der Reporting-Layer schreibt selbst **keine** Daten. Er ist ein reines
Aggregat über andere Aggregate.

### Wo kommt „Ist" her?

| Aggregat | Woher | Feld |
| --- | --- | --- |
| `shiftplan_hours` (per Wochentag) | `shiftplan_report_service.extract_shiftplan_report` → aggregiert `bookings` × `slots` in `ShiftplanReportDay` | pro Tag / Woche / Sales-Person |
| `extra_hours` (`ExtraWork` und Custom mit `modifies_balance=true`) | `extra_hours_service.find_by_sales_person_id_and_year_range` → Tabelle `extra_hours` | `amount`, `category`, `date_time` |

### Wo kommt „Erwartung" her?

| Aggregat | Woher | Feld |
| --- | --- | --- |
| Contract-Wochenstunden | `employee_work_details_service.find_by_sales_person_id` → Tabelle `employee_work_details` | `expected_hours`, `workdays_per_week`, `is_dynamic`, `cap_planned_hours_to_expected`, `monday…sunday`, `from_(year|calendar_week|day_of_week)`, `to_(…)`, `vacation_days`, `holiday_hours()` (abgeleitet) |
| Absence-Reduktion (Range-basiert, v1.0+) | `absence_service.derive_hours_for_range` → Tabelle `absence_period` | `date → ResolvedAbsence { hours, category (Vacation/SickLeave/UnpaidLeave) }` |
| Absence-Reduktion (Legacy, single-day) | via `extra_hours` (Kategorien mit `ReportType::AbsenceHours`) | `amount`, `category`, `date_time` |
| Feiertag-Reduktion (manuell) | via `extra_hours` mit Kategorie `Holiday` | s.o. |
| Feiertag-Reduktion (derived) | `special_day_service.get_by_week` (Tabelle `special_day`) → `build_derived_holiday_map` mit Cutoff aus `toggle_service.get_toggle_value("holiday_auto_credit")` | pro Woche, gefiltert nach Vertragswochentag |

### Wo kommt „Carryover" her?

- `carryover_service.get_carryover(sales_person_id, year - 1)` →
  Tabellen `employee_yearly_carryover` (`carryover_hours`) und
  `employee_yearly_vacation_carryover` (`vacation`, i32).

### Migrationen, die den Reporting-Read direkt betreffen

Chronologisch:

- `20241020064536_add-special-day-table.sql` — Feiertag/ShortDay-Tabelle
  (Basis für `SpecialDayService`).
- `20241215063132_add_employee-yearly-carryover.sql` — Basis für
  `carryover_service.get_carryover(...).carryover_hours`.
- `20241231065409_add_employee-yearly-vacation-carryover.sql` — Basis für
  `.vacation` (Urlaubstags-Übertrag).
- `20250413073750_add-custom-extra-hours-table.sql` — Custom-Kategorien,
  seit v1.x im Reporting integriert.
- `20250418200122_insert-custom-column-to-extra-hours.sql` — verknüpft
  `extra_hours` mit Custom-Kategorien.
- `20260428101456_add-logical-id-to-extra-hours.sql` — Logical-ID für
  soft-delete/replace-Semantik.
- `20260502170000_create-absence-period.sql` — Range-basiertes
  Absence-Aggregat (v1.0), Quelle für `derive_hours_for_range`.
- `20260517120000_add-day-fraction-to-absence-period.sql` — Tages-Bruchteile
  für Absence-derived Stunden.
- `20260628000001_seed-holiday-auto-credit-toggle.sql` — Toggle-Row für
  Phase 25 Feiertag-Auto-Credit.
- `20260707000001_add-source-column-to-extra-hours.sql` — Phase 54
  (Milestone v2.6) Marker-Spalte `extra_hours.source TEXT NOT NULL
  DEFAULT 'manual'` (Werte: `manual` \| `rebooking`). Volle Regeln
  siehe Feature [F14](./F14-rebooking.md). **Reader-Impact:**
  Balance-Ketten-Aggregate in `service_impl/src/reporting.rs` und
  ihre nachgelagerten Konsumenten filtern ab Phase 55
  `source = 'manual'` — erster Live-Konsument ist
  `voluntary_ist_total_in_range(..)` (Plan 54-03 lieferte den ersten
  Full-Year-Reader; Plan 54-07 Gap G1 löste ihn durch die Range-Variante
  ab).
  In Phase 54 setzt kein Writer `rebooking`, daher gehen alle Bestandszeilen
  weiterhin identisch in die Balance ein (Backfill via
  Column-DEFAULT).

Reporting selbst schreibt in **keine** dieser Tabellen.

### Beziehungen

```
                       ┌───────────────────┐
                       │  employee_work_   │  (Contract-Zeilen pro Zeitraum)
                       │  details          │
                       └────────┬──────────┘
                                │  expected_hours, is_dynamic, cap, workdays
                                ▼
sales_person ─────► ReportingService ◄───── extra_hours (WorkingHours + AbsenceHours + Custom)
                        │      ▲            └─ Cutover-Split mit …
                        │      │
                        │      ├───── absence_period (Range-Absence, derived-Stunden)
                        │      │
                        │      ├───── booking + slot (Ist via ShiftplanReportService)
                        │      │
                        │      ├───── special_day + holiday_auto_credit-Toggle
                        │      │      (derived Feiertage)
                        │      │
                        │      └───── employee_yearly_carryover +
                        │             employee_yearly_vacation_carryover
                        ▼
                 EmployeeReport / ShortEmployeeReport
                        │
                        ▼
                    REST /report/…
                        │
                        ▼
                 Frontend (Employee, WeeklySummary)
```

Der Block-Bereich (`Block`, `MyBlock`, `BlockReport`) ist ein *anderer*
Rechenweg: er aggregiert `Booking + Slot` in konsekutive Zeitscheiben pro
Wochentag — komplett ohne Erwartungs-/Absence-Rechnung. Beide Bereiche
teilen sich lediglich die Quelldaten (Bookings, Slots, Special Days).

---

## 4. Service-API

### 4.1 `ReportingService` — Business-Logic-Tier

Trait: `service::reporting::ReportingService`
(`service/src/reporting.rs:366-438`).

```rust
#[async_trait]
pub trait ReportingService {
    type Context: …;
    type Transaction: dao::Transaction;

    async fn get_reports_for_all_employees(
        &self, year: u32, until_week: u8,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError>;

    async fn get_report_for_employee(
        &self, sales_person_id: &Uuid, year: u32, until_week: u8,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<EmployeeReport, ServiceError>;

    async fn get_report_for_employee_range(
        &self, sales_person_id: &Uuid,
        from_date: ShiftyDate, to_date: ShiftyDate,
        include_carryover: bool,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<EmployeeReport, ServiceError>;

    async fn get_week(
        &self, year: u32, week: u8,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError>;

    async fn get_employee_weekly_statistics(
        &self, sales_person_id: &Uuid,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<EmployeeWeeklyStatistics, ServiceError>;

    async fn get_employee_attendance_statistics(
        &self, sales_person_id: &Uuid, year: u32, until_week: u8,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<Option<EmployeeAttendanceStatistics>, ServiceError>;
}
```

### 4.2 Auth-Gates

Verbindliche Regeln (`Authentication::Full` ist der Bypass, den nur
interne Aggregate benutzen):

| Methode | Gate | Bemerkung |
| --- | --- | --- |
| `get_reports_for_all_employees` | `HR_PRIVILEGE` (früh, vor jedem Read) | REST-Handler ruft mit User-Context. Iteriert intern mit `Authentication::Full` weiter, damit nicht jeder Sub-Read einen User-Context braucht. |
| `get_report_for_employee`, `get_report_for_employee_range` | **oder-Gate**: `HR_PRIVILEGE` ODER `verify_user_is_sales_person(id, ctx)` (Reporting-`join!`, siehe `reporting.rs:691-700`) | Mitarbeiter darf den eigenen Report ziehen. |
| `get_week` | Auth-Check delegiert an `employee_work_details_service.all_for_week(…, ctx, …)` | Nutzer-Context wird durchgereicht; internal-Reads danach mit `Full`. |
| `get_employee_weekly_statistics` (A-22-1) | `HR_PRIVILEGE` als **erste** Anweisung (STAT-01/D-22-05) | Kein Daten-Fetch vor der Auth. Ausschließlich HR-only. |
| `get_employee_attendance_statistics` (RPT-01/v2.2) | `HR_PRIVILEGE` als **erster** await (D-AVG-05) | Ebenfalls HR-only. Ab v2.2 post-ship gilt der Filter für **alle** Mitarbeiter (is_dynamic-Filter entfernt, siehe `reporting.rs:1207-1210`). |

**`ToggleService`-Read für `holiday_auto_credit`:** wird intern mit dem
übergebenen `context` gemacht (nicht `Full`!). `Unauthorized` wird als
„Automation aus" interpretiert — das ist der Legacy-Off-Zweig und
verhindert, dass Mock-/interne Aufrufer den Toggle scheitern lassen
(`reporting.rs:163-172`). Siehe auch
[edge-cases §6.1 (Full-Bypass)](../domain/edge-cases.md#61-authenticationfull-bypass)
und `ToggleService`-Full-Context-Fix in `service_impl/src/toggle.rs`.

### 4.3 TX-Verhalten

`ReportingService` hat `TransactionDao` als Dep, aber **keine der öffentlichen
Methoden öffnet oder committet selbst eine TX**. Sie akzeptieren `Option<Transaction>`
und reichen sie an alle Sub-Aggregate weiter (`tx.clone()`). Wenn der Consumer
`None` liefert, arbeitet jeder Sub-Service in seiner eigenen impliziten TX
(bzw. der jeweilige `use_transaction`-Aufruf öffnet eine).

Für die Snapshot-Erzeugung (Billing-Period-Report, siehe F08) ist das
kritisch: wenn dort ein Report unter einer laufenden TX berechnet wird,
läuft *diese* TX auch durch alle Sub-Reads (Read-Konsistenz-Set). Der
Reader **committet** nichts selbst.

**[Zu prüfen]** Ob die `Authentication::Full`-Sub-Reads auch die
`tx`-Übergabe respektieren (Stichprobe bestätigt `tx.clone()` überall);
insbesondere die parallele `join!` in `reporting.rs:691-700` läuft in derselben
TX-Klammer.

### 4.4 Dependencies

`ReportingServiceDeps` (`service_impl/src/reporting.rs:61-82`):

- Basic-Tier-Konsumenten: `ExtraHoursService`, `ShiftplanReportService`,
  `EmployeeWorkDetailsService`, `SalesPersonService`, `CarryoverService`,
  `PermissionService`, `ClockService`, `UuidService`, `SpecialDayService`,
  `ToggleService`, `TransactionDao`.
- Business-Logic-Konsument: `AbsenceService` (auch Business-Logic, aber
  in disjunkter Sub-Domain — keine Kreise). Die Absence-derived-Hours
  werden nach dem additiven Merge-Modell konsumiert (D-01, Phase 8.4).

Klassifiziert nach CLAUDE.md-Konvention:
`ReportingService` ist **Business-Logic-Tier**, weil es über mehrere
Aggregate liest und Cross-Entity-Invarianten pflegt (Balance-Formel).

### 4.5 `BlockService` — Basic-Tier für Zeitscheiben

Trait: `service::block::BlockService` (`service/src/block.rs:70-123`).

Der `Block` ist **nicht persistiert** — ein reines Read-Aggregat:
konsekutive `Booking + Slot`-Paare auf demselben Wochentag mit
`slot_prev.to == slot_next.from` werden zu einem Block gemergt
(`service_impl/src/block.rs:150-215`).

Methoden:

| Methode | Zweck | Auth |
| --- | --- | --- |
| `get_blocks_for_sales_person_week` | Blöcke einer Person in einer KW | delegiert an `SalesPersonService.get(…)` |
| `get_blocks_for_next_weeks_as_ical` | iCal-String über die nächsten 12 Wochen (rückwärts −2 → +10) | intern `Authentication::Full` |
| `get_unsufficiently_booked_blocks` | Blöcke, deren summierte `min_resources` nicht durch Bookings gedeckt sind | `context` durchgereicht |
| `get_blocks_for_current_user` | Für den aktuell eingeloggten User über einen ShiftyWeek-Range | delegiert an `sales_person_service.get_sales_person_current_user(ctx, …)` |

**Phase 51 (D-51-06 Chain A' + D-51-07 Stichtag-Gate):** Vor dem
Merge-Loop läuft `clip_slot_for_week` (`shortday_gate.rs`) — pro Slot ein
ShortDay-Clip, gegated durch den `shortday_gate_active_from`-Toggle
(via `shortday_gate::read_active_from`; `Unauthorized → None` = Legacy off).
Reihenfolge kritisch: erst clippen, dann `slot.from == to`-Merge, sonst
verschiebt sich die Consecutive-Detection.

### 4.6 `BlockReportService`

Trait: `service::block_report::BlockReportService`.

Nimmt eine `template_id` (verweist auf einen `TextTemplate`), lädt drei
Wochen (`current`, `next`, `week_after_next`), filtert nur zukünftige
Blöcke (`is_block_in_future` gegen `clock_service.date_time_now()`) und
rendert entweder mit **Tera** oder **MiniJinja** in einen `Arc<str>`
(`service_impl/src/block_report.rs:178-228`).

Kontext-Variablen im Template:
- `current_week_blocks`, `next_week_blocks`, `week_after_next_blocks`
  (als `SimpleBlock`-Liste, siehe `service_impl/src/block_report.rs:19-42`),
- `unsufficiently_booked_blocks` (aggregiert über die drei Wochen),
- `current_(week|year)`, `next_(week|year)`, `week_after_next_(week|year)`.

Auth: `HR_PRIVILEGE` als erste Anweisung.

### 4.7 `MyBlockService`

Trait: `service::my_block::MyBlockService`.

**[Zu prüfen]** Trotz existierendem Trait findet sich keine `MyBlockServiceImpl`
im Repo (`grep -rn "MyBlockService" service_impl/` liefert 0 Treffer).
Der REST-Handler `rest/src/my_block.rs:52-53` ruft stattdessen direkt
`rest_state.block_service().get_blocks_for_current_user(…)`. Das
`MyBlockService`-Trait ist damit aktuell **ungenutzt** — vermutlich
historisch entstanden, bevor die Methode in `BlockService` gewandert ist,
oder noch nicht sauber entfernt. Verweis anhand Grep-Ergebnis:
`service_impl/src/…` (keine Fundstelle für `MyBlockService`).

---

## 5. REST-Endpoints

Alle Reporting-Endpoints sind unter `/report/` gemountet
(`rest/src/lib.rs:650`). Block-bezogene unter `/blocks/` und
`/block-report/`.

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/report/?year=…&until_week=…` | Kurz-Report aller bezahlten Mitarbeiter im Jahr bis KW N | Query: `ReportRequest { year, until_week }` | `Vec<ShortEmployeeReportTO>` | 401, 403, 500 |
| `GET` | `/report/{id}?year=…&until_week=…` | Voller Employee-Report inkl. `by_week` | Path: `Uuid`, Query s.o. | `EmployeeReportTO` | 401, 403 (weder HR noch eigen), 500 |
| `GET` | `/report/week/{year}/{calendar_week}` | Kurz-Report aller Personen in EINER KW | Path: `(year, week)` | `Vec<ShortEmployeeReportTO>` | 500 |
| `GET` | `/report/{id}/weekly-statistics` | Ø gearbeitete Std./Woche (aktuelles Jahr bis heutige KW) | Path: `Uuid` | `EmployeeWeeklyStatisticsTO` | 403 HR-only, 500 |
| `GET` | `/report/{id}/attendance-statistics?year=…&until_week=…` | Per-Wochentag-Anwesenheits-Verteilung (7 Einträge Mo..So) | Path + Query | `Option<EmployeeAttendanceStatisticsTO>` (heute immer `Some`) | 403 HR-only, 500 |
| `GET` | `/blocks/{from_year}/{from_week}/{until_year}/{until_week}` | Blöcke des aktuellen Users im Range | Path 4× | `Vec<BlockTO>` | 401, 403, 500 |
| `GET` | `/block-report/{template_id}` | Rendert Template mit den 3-Wochen-Blöcken | Path: `Uuid` | `text/plain` (String) | 401, 403 HR-only, 404, 500 |

DTOs (Wire-Format):

- `ShortEmployeeReportTO` (`rest-types/src/lib.rs:371-393`): kompakte
  Zeile für die HR-Übersicht.
- `EmployeeReportTO` (`rest-types/src/lib.rs:523-596`): voller
  Report; enthält `by_week: Arc<[WorkingHoursReportTO]>` und
  `by_month: Arc<[…]>` (aktuell immer leer, `reporting.rs:877` schreibt
  `Arc::new([])` — [Zu prüfen] ob je verwendet).
- `WorkingHoursReportTO` (`rest-types/src/lib.rs:459-520`): eine
  Wochenzeile mit Tages-Split.
- `EmployeeWeeklyStatisticsTO`, `EmployeeAttendanceStatisticsTO`,
  `WeekdayAttendanceTO`: A-22-1 / RPT-01-Aggregate.
- `BlockTO` (`rest-types/src/lib.rs:1603-…`).

---

## 6. Frontend-Integration

Frontend liegt komplett in `shifty-backend/shifty-dioxus/`. Der wichtige
Punkt: das **gesamte** Balance-Rechenwerk läuft im Backend. Das Frontend
ist reiner View-Layer — es liest die DTOs und rendert.

### 6.1 Pages

- **`page/report.rs`** — HR-Übersicht: `GET /report/?year=…&until_week=…`.
- **`page/employee_details.rs`** (via `Employee`-Aggregat, Loader
  `loader.rs:294`): `GET /report/{id}?year=…&until_week=…`.
- **`page/weekly_overview.rs`** — persönliche Wochen-Übersicht mit
  Diff-Farbe (`text-good`/`text-warn`, siehe `diff_color_and_sign`).
  Nutzt den `WeeklySummary`-State.
- **`page/my_shifts.rs`** — persönliche Block-Sicht der nächsten
  Wochen: `GET /blocks/{from_year}/{from_week}/{until_year}/{until_week}`.
  Formatiert Stunden mit `format_hours_norm(hours, 1)`, um `-0.0`-Anzeige
  zu vermeiden.

### 6.2 Services / Loader

- `shifty-dioxus/src/api.rs` — Wrapper `get_short_reports`,
  `get_employee_reports`, `get_working_hours_for_week`,
  `get_balance_until_week` (siehe `loader.rs:267,294,360,363`).
- `shifty-dioxus/src/state/weekly_overview.rs` — `WeeklySummary` mit
  gewordenen Zahlen (bereits als `f32` aus dem TO).

### 6.3 i18n-Keys (Reporting-relevant, aus `weekly_overview.rs`)

`WeekLabel`, `PaidCommittedVolunteer`, `AvailableRequiredHours`,
`MissingHours`, `HoursShort` — jeweils in allen drei Locales (En, De, Cs)
zu pflegen (siehe CLAUDE.md-Regel).

### 6.4 Dioxus.toml Proxy

Die Reporting-Endpoints existieren schon seit v1.0 im Proxy. Neue Sub-Pfade
(z.B. `/report/{id}/weekly-statistics`, `/report/{id}/attendance-statistics`
in v2.2) sind über die generische `/report/{**}`-Weiterleitung erreichbar
— **[Zu prüfen]** ob die konkrete Wildcard-Regel im Proxy die
Sub-Segmente wirklich mitnimmt. Siehe
[edge-cases §11 Frontend-Backend-Kopplung](../domain/edge-cases.md#11-frontend-backend-kopplung)
und den Memory-Hinweis „Dioxus.toml Proxy für neue Backend-Endpoints".

---

## 7. Randfälle

**Zentrale Referenzen — Pflichtlektüre vor jeder Änderung am Reporting:**

- [edge-cases §1 Stundenkonto](../domain/edge-cases.md#1-stundenkonto)
  — Carryover-Grenze, Contract-Wechsel, Sales-Person-Zeitgrenzen,
  Special-Days, Balance-Perimeter.
- [edge-cases §2 Absence & Extra Hours](../domain/edge-cases.md#2-absence--extra-hours)
  — Cutover-Split, Range-Randfälle, Legacy-Delete-Semantik.
- [edge-cases §5 Rundung & Genauigkeit](../domain/edge-cases.md#5-rundung--genauigkeit)
  — f32-Präzision, Assoziativität, Anzeige-vs-Persistenz-Rundung.

### 7.1 Feature-spezifische Kanten des Reportings

- **Rundungs-Konsistenz zwischen Anzeige und Summe.**
  Das Frontend zeigt Wochenwerte mit einer Nachkommastelle
  (`format_hours(hours, 1)`, `my_shifts.rs:44`). Das Backend rechnet in
  `f32` und aggregiert die Wochen zu Jahres-Summen *im Backend*, bevor
  gerundet wird. Wenn ein Client aus den gerundeten Wochenwerten neu
  summiert, weicht die Anzeige-Summe von der Backend-Balance ab. → **Regel:**
  Immer die Backend-Gesamtsumme anzeigen, nie im Client aus gerundeten
  Wochenwerten neu aufaddieren. Siehe
  [edge-cases §5](../domain/edge-cases.md#5-rundung--genauigkeit).

- **Cross-Period-Konsistenz (Live vs Snapshot).**
  `EmployeeReport` ist **immer** ein Live-Read. Wenn parallel ein
  `billing_period`-Snapshot mit einer bestimmten
  `snapshot_schema_version` existiert (siehe F08), können Live-Report und
  Snapshot für dieselbe Periode voneinander abweichen — legitim, wenn
  Regel-/Kategorien-Änderungen zwischenzeitlich passiert sind. **Immer** die
  Version zusammen mit dem Snapshot lesen (Validator-Muster). Siehe
  [edge-cases §3.1/3.3](../domain/edge-cases.md#3-billing-period--snapshots).

- **`by_month` ist leer.**
  Die Struct hat `by_month: Arc<[GroupedReportHours]>`, aber der Reader
  schreibt aktuell `Arc::new([])` (`reporting.rs:877`). Frontend ignoriert
  das entsprechend. **[Zu prüfen]** ob je aktiviert oder ob entfernt
  werden sollte.

- **Range-Report + `include_carryover=false`.**
  `get_report_for_employee_range` mit `include_carryover=false` liefert
  `carryover_hours=0.0` und `vacation_carryover=0` — der Consumer muss
  wissen, dass die Balance dann *nicht* den Vorjahres-Übertrag enthält.
  Wird u.a. für Sub-Perioden verwendet, in denen der Carryover schon
  extern verrechnet wurde.

- **Dynamischer Vertrag + Absence in derselben Woche.**
  Getested (`reporting.rs:1615-1707`). Kernaussage: bei
  `is_dynamic == true` bleibt die Balance immer 0 (Soll = Ist), auch mit
  Vacation-Extras. Der Grund liegt in der doppelten `if working_hours_for_week <= 0.0`-
  Guard-Kette (Zeilen 1378-1414, 1097-1122).

- **Cap + Feiertag-Auto-Credit gleichzeitig (HSP-03 Band-Guard).**
  Wenn Cap aktiv ist und ein derived-Holiday in dieselbe Woche fällt,
  DARF der Holiday **nicht** in die Cap-Baseline. Sonst würde der
  Holiday-Delta als `auto_volunteer_hours` in die Volunteer-Bänder von
  `booking_information` lecken (violet D-25-08). Der Guard sitzt in
  `reporting.rs:1113-1122`: `expected_hours_for_cap = planned − absence −
  absence_derived_balance` (ohne Holiday), dann apply_cap, dann
  `expected_hours = expected_hours_for_cap − holiday_derived_gated`.

- **`vacation_days()`-Divisor bei `workdays_per_week == 0`.**
  Die Getter `hours_per_day()`, `hours_per_holiday()`, `vacation_days()`
  etc. auf `GroupedReportHours` schützen explizit gegen Div-durch-0
  (`service/src/reporting.rs:105-145`). Wichtig, weil dynamische Verträge
  `workdays_per_week=0.0` liefern können.

- **KW 53 in Nicht-53-Jahren.**
  `until_week.min(time::util::weeks_in_year(year as i32))` in
  `get_report_for_employee` (`reporting.rs:664`) — clamped auf die
  tatsächliche Wochenanzahl. **Wichtig:** die REST-Query `until_week=53`
  in einem 52-Wochen-Jahr wird stumm auf 52 reduziert.

- **`get_reports_for_all_employees` iteriert `additional_weeks=1` in
  Jahres-Randlagen** (`reporting.rs:317-321`), um Wochen mit
  `iso_week == 1` im Folgejahr / `iso_week == 53` im Vorjahr korrekt zu
  erfassen. Historischer Grund: ISO 8601-Wochen laufen über
  Jahreswechsel. Wenn du diese Schleife anfasst, prüfe die A-22-1-Tests
  auf Jahreswechsel-Randlagen (`reporting_avg_weekly.rs`).

- **Blocks — leere Bookings-Menge.**
  Wenn keine Bookings für Person+Woche existieren, gibt
  `get_blocks_for_sales_person_week` `Arc::from([])` zurück (leer, kein
  Fehler). `MyShifts`-Seite muss den leeren State handhaben.

- **iCal — `sales_person_id == Uuid::nil()`.**
  Historische Konvention: `nil()` → *nur* unbesetzte/unterbesetzte Blöcke
  in den nächsten 12 Wochen. Siehe `block.rs:232-249`.

- **RPT-01 (`weekday_attendance_distribution`) — leerer Nenner.**
  `counted_calendar_weeks == 0` → alle `share=0.0`, kein NaN
  (`reporting.rs:322-328`). Rundung: zwei Nachkommastellen via
  `(x * 100.0).round() / 100.0`. Achtung: `share_of_hours` summiert genau
  zu 1.0, wenn `total_hours > 0`, sonst 0.0 (`reporting.rs:330-335`).

- **A-22-1 (`average_worked_hours_per_week`) — leere „includiert"-Menge.**
  Fully-absent-Wochen (`worked==0 && absence>0`) fliegen aus dem Nenner
  (`reporting.rs:225-231`). Wenn *alle* Wochen fully-absent sind → Avg=0.0,
  `included_weeks=0` — der Consumer muss das UI-seitig behandeln.

---

## 8. Tests

Umfangreichster Test-Bereich der Codebase.

### 8.1 Reporting

- `service_impl/src/test/reporting_additive_merge.rs` (1456 Zeilen) —
  Kern-Regressionssuite für den Absence-derived + Extra-Hours-Merge.
  Deckt no-contract, dynamic, cap, Custom-Kategorien, cross-week Absence
  und die Gap-Fixes aus Phase 8.4 (M-02/CR-01/WR-01) ab.
- `service_impl/src/test/reporting_holiday_auto_credit.rs` (929 Zeilen) —
  Phase 25: `holiday_auto_credit`-Toggle, Cutoff-Gate, Manual-Wins,
  4×Injection-Points (`hours_per_week` / all_employees / range /
  `get_week`), HSP-03 Band-Guard.
- `service_impl/src/test/reporting_no_contract_volunteer.rs` (447 Zeilen)
  — User-Regel `quick-260624-ujk`: Kein-Vertrags-Woche → Ehrenamt.
- `service_impl/src/test/reporting_cap_overflow.rs` (297 Zeilen) — Weekly
  Cap: Überschuss → `auto_volunteer`, korrekt bei Absence & Extra-Work.
- `service_impl/src/test/reporting_avg_weekly.rs` (175 Zeilen) — A-22-1
  Pure-Formel + Service-Wrapper, HR-Gate zuerst.
- `service_impl/src/test/reporting_weekday_attendance.rs` (251 Zeilen) —
  RPT-01 v2.2: Distinct-Date-Dedup, Filter (Shiftplan/ExtraWork/VolunteerWork),
  `share_of_hours`-Summe = 1.0.
- `service_impl/src/test/reporting_attendance_gate.rs` (318 Zeilen) —
  HR-Gate + is_dynamic-Filter (in v2.2 post-ship auf „für alle" umgestellt,
  siehe `reporting.rs:1207-1210`).
- `service_impl/src/test/reporting_phase2_fixtures.rs` (141 Zeilen) —
  Fixtures für die frühe Phase-2-Regression.
- Inline-Tests im Impl selbst: `test_dynamic_vacation_days`,
  `test_unpaid_leave_*` (`reporting.rs:1553-1849`),
  `test_weekly_planned_hours_cap` (ab `reporting.rs:1852`).

### 8.2 Blocks

- `service_impl/src/test/block.rs` (1045 Zeilen) — Merge-Logik, ShortDay-
  Clipping (Phase 51), Unavailable-Blöcke, iCal-Export-Roundtrip,
  Insufficient-Booking-Detection.
- `service_impl/src/test/block_report.rs` (259 Zeilen) — Tera/MiniJinja
  Template-Rendering, Future-Only-Filter, HR-Gate.

### 8.3 Bekannte Lücken

- **[Zu prüfen]** Rückwirkende Contract-Änderung + Live-Report vs
  Carryover-Drift — kein expliziter Regression-Test.
- **[Zu prüfen]** `by_month`-Feld (aktuell immer leer) — kein Test, weil
  kein Verhalten.
- **[Zu prüfen]** `MyBlockService`-Trait ohne Impl — dead code, sollte
  entfernt oder implementiert werden.
- **[Zu prüfen]** DST-Umschaltung (März/Oktober) in Blöcken über die
  Nachtstunden — siehe [edge-cases §4](../domain/edge-cases.md#4-zeit--zeitzone).

---

## 9. Historie & Kontext

Reporting ist **das älteste durchgehend produktive Feature** von Shifty
und hat entsprechend viele Rewrites hinter sich. Die wichtigsten Meilensteine:

- **v0.x (vor v1.0):** Ur-Report mit `extra_hours` als einziger
  Absence-Quelle, Balance in Wochenmodus. `EmployeeReport`,
  `ShortEmployeeReport`, `GroupedReportHours` als Grundstruktur bereits
  hier definiert.
- **v1.0:** Cutover zum Range-basierten `AbsenceService`
  (`.derive_hours_for_range`). Der Legacy-Pfad über `extra_hours` bleibt
  **koexistierend** — die konvertierten Legacy-Zeilen werden per
  `soft_delete_bulk` markiert, damit kein Doppelzählen entsteht.
- **Phase 8.4 (D-01, CR-01, WR-01, M-02, M-03):** Additiver Merge —
  Absence-derived unbedingt aufaddiert, Feature-Flag-Switch entfernt.
  Symmetrische Dynamic-Guards auf beiden Absence-Beiträgen. Fix für die
  Dynamic-Contract Balance-Asymmetrie.
- **Phase 15 (D-01 „report-ehrenamt-gesamtstunden"):** `overall_hours` /
  `balance_hours` / `shiftplan_hours` nutzen ausschließlich den per-Woche
  gedeckelten `shiftplan_hours_by_week` — der rohe Wert leakt nicht mehr.
- **Phase 17 (D-06, CVC-10):** `is_paid=false` (Unbezahlte Freiwillige)
  werden aus `paid_hours` / `WorkingHoursPerSalesPerson` /
  Year-Summary rausgehalten. Beide `get_reports_for_all_employees` und
  `get_week` filtern jetzt `if !sales_person.is_paid.unwrap_or(false) { continue; }`.
- **Phase 18 (UV-05, D-18-03/04/05):** `by_week` wird zur Single-Source
  für Top-Level-Kategorien-Summen. Alte Jahres-Lumps entfernt. Display-
  vs Balance-Split (Display ungegate, Balance gegated).
- **Phase 25 (HOL-01/02, HCFG-01/03):** Holiday derive-on-read über
  `holiday_auto_credit`-Toggle. Vier Injection-Points, Manual-Wins,
  Cutoff-Gate, HSP-03 Band-Guard.
- **Phase 34 (HSP-01/02, D-34-01):** 4. Injection-Point in `get_week`.
- **v2.2 (RPT-01):** Per-Weekday-Attendance-Distribution — ersetzt die
  frühere Skalar-Ø-hours-Metrik durch Count + Share pro Wochentag mit
  `share_of_hours` (v2.2.1).
- **A-22-1 (STAT-01, D-22-05, D-22-06):** Weekly Statistics — HR-only,
  Avg exkl. fully-absent Wochen. Reused per-Woche-Daten aus
  `get_report_for_employee`.
- **Phase 47 (D-47-BE):** `EmployeeAttendanceStatistics`-Reshape auf
  7-Wochentage-Array + `counted_calendar_weeks`.
- **Phase 51 (D-51-06/07):** ShortDay-Cutoff pro Wochentag + Stichtag-Gate
  in `BlockService`. `ToggleService`-Full-Context-Bypass für interne
  Aggregat-Aufrufer (siehe Memory-Notiz).
- **Phase 52 (WOP-01/02, D-52-06/D-52-08):** Additive Batch-Trait-Methode
  `ReportingService::get_year(year)` sowie
  `ShiftplanReportService::extract_shiftplan_report_for_year(year)` /
  `ExtraHoursService::find_by_year(year)`. Balance-Formel, CVC-06
  Cap-Semantik, `is_paid`-Filter und der `assemble_weeks`-Per-Woche-
  Aggregations-Body bleiben unverändert. Byte-Identität zwischen
  Batch- und Einzel-Woche-Aufruf ist strukturell garantiert durch den
  gemeinsamen `pub(crate) assemble_weeks`-Helper. Konsument:
  `BookingInformationServiceImpl::get_weekly_summary` nutzt diese
  Bulk-Loads jetzt, um ~55 sequenzielle Service-Calls durch 7
  konstante Bulk-Loads zu ersetzen (byte-identisch, ~2× Latenz-
  Reduktion auf Dev-DB).

### Fat-Backend-Prinzip

Die gesamte Balance-Rechnung sitzt im Backend. Das Frontend nimmt die
DTOs (`ShortEmployeeReportTO`, `EmployeeReportTO`, `WorkingHoursReportTO`)
und rendert nur — kein einziger Kategorien-Summand wird im FE neu
gerechnet. Grund: Zweit-Client-Fähigkeit (Mobile-App etc.) ohne
Domain-Regel-Duplikation. Siehe
[Fat-Backend-Memory-Notiz](../../CLAUDE.md) und die Empfehlung, das
Prinzip in jeder discuss-phase als Default zu verankern.

### Verhältnis zu Billing Period (F08)

`EmployeeReport` ist die **Live-Rechnung**. `billing_period_sales_person`
persistiert dieselben Kategorien als Snapshot mit einer
`snapshot_schema_version` (aktuell **12**, siehe
`billing_period_report.rs:117`). Der Snapshot-Writer konsumiert
`EmployeeReport` und schreibt eine Reihe von `value_type`-Zeilen. Details
siehe [F08 Billing Period](./F08-billing-period.md).

**Regel für Reporting-Änderungen:** Jede Änderung an der Berechnung
eines im Snapshot-persistierten `value_type` (Formel, Inputs, Filter)
MUSS die `CURRENT_SNAPSHOT_SCHEMA_VERSION` bumpen — sonst driftet der
Validator (siehe CLAUDE.md-Absatz „Billing Period Snapshot Schema
Versioning").

---

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.

---

**Fazit:** Reporting ist die Ausgabeschicht, die alle
Balance-relevanten Aggregate (Bookings, Extra-Hours, Absence, Carryover,
Special Days, Toggles) in eine einzige Formel gießt — mit vielen
über Jahre hinzugewachsenen Guards für Cap-Overflow, Dynamic-Contract,
Kein-Vertrag-Woche und Feiertag-Auto-Credit. Wer hier etwas ändert,
prüft zuerst die Snapshot-Version (F08) und die Randfall-Referenz, sonst
driftet Live-Report von persistierter Wahrheit weg.
