---
phase: quick-260624-ujk
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - service_impl/src/reporting.rs
  - service_impl/src/billing_period_report.rs
  - service_impl/src/test/reporting_no_contract_volunteer.rs
  - service_impl/src/test/mod.rs
autonomous: true
requirements:
  - UJK-01
  - UJK-02
  - UJK-03
must_haves:
  truths:
    - "Geleistete Shiftplan-Stunden in einer KW OHNE EmployeeWorkDetails-Zeile zaehlen als volunteer_hours (Ehrenamt), nicht als Soll=Ist"
    - "Eine KW MIT EmployeeWorkDetails-Zeile und expected=0 (dynamisch) behaelt unveraendert das Soll=Ist-Verhalten (planned_hours = overall_hours, Saldo +-0)"
    - "Eine KW MIT EmployeeWorkDetails-Zeile und expected>0 bleibt im Verhalten unveraendert"
    - "Detail-Report (Range/Year), All-Employees-Summary und Week-Report klassifizieren no-contract-Stunden konsistent als Ehrenamt"
    - "CURRENT_SNAPSHOT_SCHEMA_VERSION wurde um 1 erhoeht (7->8 bereits; jetzt 8->9), weil sich die Berechnung des persistierten value_type volunteer_hours aendert"
  artifacts:
    - path: "service_impl/src/reporting.rs"
      provides: "no-contract-Erkennung + Ehrenamt-Klassifikation in hours_per_week, get_reports_for_all_employees, get_week"
      contains: "is_none()"
    - path: "service_impl/src/test/reporting_no_contract_volunteer.rs"
      provides: "Tests fuer alle vier Faelle (no-contract / dynamic / expected>0 / Konsistenz)"
      min_lines: 80
    - path: "service_impl/src/billing_period_report.rs"
      provides: "CURRENT_SNAPSHOT_SCHEMA_VERSION-Bump"
      contains: "CURRENT_SNAPSHOT_SCHEMA_VERSION"
  key_links:
    - from: "service_impl/src/reporting.rs:hours_per_week"
      to: "find_working_hours_for_calendar_week(...).next().is_none()"
      via: "no-contract-week detection"
      pattern: "find_working_hours_for_calendar_week.*is_none"
---

<objective>
Im Mitarbeiter-Report sollen geleistete Stunden, die in eine KW OHNE Arbeitsvertrag
(kein EmployeeWorkDetails-Record fuer die Woche) fallen, als Ehrenamt
(`volunteer_hours`) gezaehlt werden — statt wie bisher als "Soll = Ist"
neutralisiert zu werden.

Praezise Regel (woertlich vom User, massgeblich):
- KEIN Vertragsobjekt fuer die KW vorhanden -> geleistete Stunden sind freiwillig (Ehrenamt).
- Vertragsobjekt vorhanden UND expected = 0 -> wird wie dynamischer Vertrag behandelt
  (heutige Soll=Ist-Neutralisierung BLEIBT unveraendert).
- Vertragsobjekt mit expected > 0 -> normales Verhalten, unveraendert.

Knackpunkt: Der heutige Code triggert allein auf `expected_hours == 0.0` bzw.
`working_hours_for_week == 0.0`. "Keine Zeile fuer die Woche" ergibt durch die leere
Summe ebenfalls 0 — deshalb behandelt der Code aktuell BEIDE Faelle gleich. Die neue
Logik muss explizit unterscheiden via
`find_working_hours_for_calendar_week(working_hours, year, week).next().is_none()`.

Purpose: Korrekte Ehrenamt-Erfassung fuer Mitarbeiter, die in vertragslosen Wochen
geleistet haben (z.B. vor Vertragsbeginn / nach Vertragsende), ohne die bewusste
Soll=Ist-Neutralisierung dynamischer Vertraege zu beruehren.
Output: Geaenderte reporting.rs (3 Pfade), Snapshot-Schema-Bump, neue Test-Datei.
</objective>

<execution_context>
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/workflows/execute-plan.md
</execution_context>

<vcs_jj_only>
Dieses Repo ist jj-managed (co-located mit git). Der Executor committet NICHTS —
kein `git`, kein `jj`. Der User committet manuell. Fuehre KEINE Commit-Schritte aus.
Lass die Working Copy nach Abschluss mit den Aenderungen stehen.
</vcs_jj_only>

<context>
@service_impl/src/reporting.rs
@service/src/reporting.rs
@service_impl/src/billing_period_report.rs
@service_impl/src/test/reporting_cap_overflow.rs
@service_impl/src/test/reporting_phase2_fixtures.rs

<interfaces>
Schluessel-Helper (bereits vorhanden in service_impl/src/reporting.rs):

```rust
// Liefert alle in der ISO-Woche aktiven EmployeeWorkDetails-Rows.
// .next().is_none() == "keine Vertragszeile fuer diese KW".
pub fn find_working_hours_for_calendar_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> impl Iterator<Item = &EmployeeWorkDetails>;

// Liefert (capped_shiftplan_hours, auto_volunteer_hours).
// Cap nur aktiv wenn cap_active && shiftplan > expected.
pub fn apply_weekly_cap(
    cap_active: bool, shiftplan_hours: f32, expected_hours_for_week: f32,
) -> (f32, f32);
```

Wiederverwendbare Test-Fixtures (service_impl/src/test/reporting_phase2_fixtures.rs):
```rust
fn fixture_sales_person_id() -> Uuid;       // deterministische Id
fn fixture_sales_person() -> SalesPerson;   // is_paid = Some(true)
fn fixture_work_details_8h_mon_fri() -> EmployeeWorkDetails; // expected 40h, KW22-25/2024, is_dynamic=false
```

Test-Mock-Setup-Muster: siehe reporting_cap_overflow.rs (vollstaendige
ReportingServiceDeps-Impl + alle Mock-Returns). Kopiere dieses TestDeps/Builder-Muster.
</interfaces>
</context>

<design_notes>
## Drei betroffene Report-Pfade (KEINEN vergessen)

1. **`hours_per_week(...)`** (reporting.rs ~Z.978-1170) — Kern des Detail-Reports.
   Wird von `get_report_for_employee_range` (Z.507) und transitiv von
   `get_report_for_employee` (Z.479) benutzt.
   Neutralisierung sitzt bei Z.1097-1101:
   ```rust
   let expected_hours = if working_hours_for_week == 0.0 {
       shiftplan_hours + extra_work_hours   // <- Soll=Ist
   } else {
       working_hours_for_week
   };
   ```
   `volunteer_hours` der Woche (Z.1159-1164) = manuelle VolunteerWork-ExtraHours
   + `auto_volunteer_hours` (Cap-Ueberlauf).

2. **`get_reports_for_all_employees(...)`** (reporting.rs Z.139) — Summary.
   Inline-Zweig bei Z.306-332: `if expected_hours <= 0.0 { ... planned_hours: overall_hours,
   volunteer_hours: auto_volunteer_hours ... }`.

3. **`get_week(...)`** (reporting.rs Z.714) — Wochen-Report.
   Inline (Z.879-885): `expected_hours = planned_hours - ...`; bei planned=0 wird NICHT
   neutralisiert (expected bleibt 0, overall = shiftplan, balance = shiftplan). D.h.
   no-contract-Stunden lecken heute in overall_hours/balance statt in volunteer_hours.

## Neue Erkennungs-Logik (einheitlich in allen drei Pfaden)

```rust
let has_contract_row =
    find_working_hours_for_calendar_week(working_hours, year, week).next().is_some();
```

Drei Faelle:
- `!has_contract_row` (keine Zeile)            -> NEU: Ehrenamt-Pfad (s.u.)
- `has_contract_row && working_hours_for_week == 0.0` (dynamisch) -> ALT: Soll=Ist beibehalten
- `has_contract_row && working_hours_for_week  > 0.0`             -> ALT: normal

## Ehrenamt-Pfad-Semantik (no contract row)

Geleistete Shiftplan-Stunden der Woche (`shiftplan_hours`, nach `apply_weekly_cap`
— hier mit expected=0, cap typischerweise inaktiv da keine Zeile, also durchgereicht)
fliessen in `volunteer_hours` und gerade NICHT in `overall_hours`/`expected_hours`:

- `expected_hours = 0.0`   (kein Vertrag => kein Soll)
- `overall_hours`  enthaelt die no-contract-Shiftplan-Stunden NICHT
  (sie sind Ehrenamt, keine bezahlte Leistung) => `balance = overall - expected = 0`
- `volunteer_hours += shiftplan_hours` (+ ggf. manuelle VolunteerWork-ExtraHours)

Begruendung fuer "overall enthaelt sie nicht": Bei Soll=Ist (dynamisch) sind die
Stunden bezahlt-neutral (planned == overall => balance 0, aber overall zeigt die
geleisteten Stunden). Beim Ehrenamt sind sie KEINE bezahlte Leistung; sie gehoeren
ausschliesslich in die Ehrenamt-Achse. Damit bleibt der Saldo +-0, aber die Stunden
erscheinen als Ehrenamt statt als "gearbeitet". Das ist genau die vom User gewuenschte
Aenderung gegenueber heute (heute: planned=overall=geleistet => als gearbeitet
sichtbar, Saldo 0).

## OFFENE DETAIL-ENTSCHEIDUNG (mit Default-Empfehlung)

Zaehlen NUR Shiftplan-Stunden ohne Vertrag als Ehrenamt, oder auch ExtraWork-Stunden
ohne Vertrag?

**Default-Empfehlung (im Plan umgesetzt):** NUR geplante/geleistete Shiftplan-Stunden
werden zu `volunteer_hours`. Explizit erfasste Kategorien bleiben UNBERUEHRT:
- `ExtraWork`-ExtraHours: bleiben in `extra_work_hours` und fliessen weiterhin in
  `overall_hours` (eine explizit erfasste bezahlte Mehrarbeit ist eine bewusste
  Erfassung, kein "versehentliches Leisten ohne Vertrag").
- Vacation/SickLeave/Holiday/UnpaidLeave/Custom: bleiben unveraendert in ihren
  Display-Spalten (kein Vertrag => sie reduzieren ohnehin kein Soll).
- Manuelle VolunteerWork-ExtraHours: weiterhin additiv in volunteer_hours (wie heute).

Begruendung: Die User-Regel spricht von "geleisteten Stunden" im Sinne der
Shiftplan-Leistung; ExtraWork ist eine separate, explizit gepflegte Achse. Sollte der
User spaeter ExtraWork-ohne-Vertrag ebenfalls als Ehrenamt wollen, ist das ein
isolierter Folge-Change am selben Branch.

## Verhaeltnis zur booking_information-Band-Logik (Doppelzaehlung ausgeschlossen)

`booking_information.rs` (Zwei-Band-Modell: `committed_voluntary` Band 1,
`volunteer_surplus_band2` Band 2) ist auf `volunteer_ids` gegated =
`sales_person.is_paid == false` (Z.164, Z.205, Z.342). Es betrifft also
**unbezahlte Freiwillige** und speist die Available-Hours-/Kapazitaets-Sicht.

Der hier geaenderte Reporting-Pfad betrifft **bezahlte Mitarbeiter**
(`get_reports_for_all_employees` filtert `is_paid == true` Z.164; `get_week` skippt
`!is_paid` Z.894) in Wochen OHNE Vertragszeile. Beide Pfade sind disjunkt
(unbezahlt vs. bezahlt), schreiben in verschiedene Aggregate und teilen keine
Summen. => Keine Doppelzaehlung. Diese Trennung wird im Task-Kommentar dokumentiert.

## Snapshot-Schema-Bump (PFLICHT)

`volunteer_hours` ist ein PERSISTIERTER billing_period value_type
(`BillingPeriodValueType::Volunteer`, billing_period_report.rs Z.253). Diese Aenderung
veraendert dessen Berechnung (no-contract-Stunden fliessen neu hinein). Laut CLAUDE.md
("Change the computation that produces an existing value_type") MUSS
`CURRENT_SNAPSHOT_SCHEMA_VERSION` (billing_period_report.rs Z.85, aktuell 8) um 1 auf
**9** erhoeht werden. Begruendung als Code-Kommentar dokumentieren.
</design_notes>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: no-contract-Erkennung + Ehrenamt-Klassifikation in allen drei Report-Pfaden</name>
  <files>service_impl/src/reporting.rs</files>
  <behavior>
    - hours_per_week, KW ohne Vertragszeile, 30h Shiftplan:
        expected_hours == 0, overall_hours == 0 (Shiftplan NICHT in overall),
        balance == 0, volunteer_hours == 30.
    - hours_per_week, KW mit dynamischer Zeile (expected weighted = 0), 30h Shiftplan:
        UNVERAENDERT — expected_hours == 30 (Soll=Ist), overall == 30, balance == 0.
    - hours_per_week, KW mit Zeile expected>0 (40h), 30h Shiftplan:
        UNVERAENDERT — expected_hours == 40, overall == 30, balance == -10.
    - get_reports_for_all_employees: analog (no-contract => volunteer, kein planned=overall).
    - get_week: no-contract-Stunden => volunteer_hours, NICHT in overall/balance.
  </behavior>
  <action>
Aendere die drei neutralisierenden Stellen so, dass "keine Vertragszeile fuer die KW"
vom "dynamischen Vertrag (Zeile vorhanden, expected=0)" unterschieden wird. Erkennung
ueberall einheitlich:
`let has_contract_row = find_working_hours_for_calendar_week(working_hours, year, week).next().is_some();`

(1) `hours_per_week` (~Z.1038-1101): VOR `apply_weekly_cap` `has_contract_row`
    berechnen. Ersetze den `expected_hours`-Block (Z.1097-1101):
    - `!has_contract_row` => Ehrenamt-Zweig:
        `expected_hours = 0.0`; die `shiftplan_hours` der Woche NICHT in `overall_hours`
        einrechnen (overall_hours der Woche = `extra_work_hours` ohne shiftplan, oder 0
        wenn keine ExtraWork — siehe Default-Entscheidung: ExtraWork bleibt bezahlt);
        `volunteer_hours` der Woche += `shiftplan_hours`.
        Konkret: setze ein lokales `let shiftplan_paid = if has_contract_row { shiftplan_hours } else { 0.0 };`
        und ein `let no_contract_volunteer = if has_contract_row { 0.0 } else { shiftplan_hours };`.
        Nutze `shiftplan_paid` ueberall dort, wo bisher `shiftplan_hours` in
        overall_hours/balance/`GroupedReportHours.shiftplan_hours` einfloss (Z.1129,
        1130, 1131). `GroupedReportHours.volunteer_hours` (Z.1159-1164) erhaelt
        zusaetzlich `+ no_contract_volunteer`.
        WICHTIG: `apply_weekly_cap` mit expected=0 und cap typischerweise inaktiv
        reicht shiftplan unveraendert durch — die no-contract-Stunden landen NICHT
        doppelt (weder via auto_volunteer noch via shiftplan_paid in overall).
    - `has_contract_row && working_hours_for_week == 0.0` => ALT (Soll=Ist):
        `expected_hours = shiftplan_hours + extra_work_hours`, shiftplan_paid =
        shiftplan_hours (unveraendert).
    - `has_contract_row && working_hours_for_week > 0.0` => ALT (normal).

(2) `get_reports_for_all_employees` (~Z.294-332): Im aktuellen `if expected_hours <= 0.0`-
    Zweig zusaetzlich `has_contract_row` unterscheiden:
    - `!has_contract_row` => Ehrenamt: `WeeklyHours { shiftplan_hours: 0.0,
        planned_hours: 0.0, volunteer_hours: auto_volunteer_hours + shiftplan_hours, ... }`
        (shiftplan geht in volunteer statt in planned/shiftplan; extra_work bleibt in
        extra_working_hours und overall). Achte darauf, dass die spaetere
        `overall_hours = shiftplan_hours + extra_working_hours`-Berechnung (Z.454) die
        no-contract-Stunden NICHT mehr enthaelt (deshalb shiftplan_hours auf 0).
    - `has_contract_row && expected_hours <= 0.0` => ALT (Soll=Ist, planned_hours = overall_hours).
    - sonst => ALT.

(3) `get_week` (~Z.836-913): `has_contract_row` aus `working_hours` (HashMap-Wert
    fuer diese person) bestimmen. Bei `!has_contract_row`:
    `shiftplan_hours` NICHT in `overall_hours`/`balance` (overall = extra_working_hours),
    `volunteer_hours += shiftplan_hours`, `expected_hours = 0`. Bei dynamischer Zeile
    (has_contract_row && planned_hours == 0) bleibt das heutige Verhalten.

Fuege an JEDER der drei Stellen einen kurzen Kommentar hinzu, der (a) die User-Regel
zitiert und (b) die Abgrenzung zur booking_information-Band-Logik (is_paid=false,
disjunkt) festhaelt — gegen kuenftige Doppelzaehlungs-Regressionen.
  </action>
  <verify>
    <automated>nix develop --command cargo build --workspace 2>&1 | tail -5</automated>
  </verify>
  <done>cargo build --workspace gruen; alle drei Pfade unterscheiden no-contract von dynamic; Default-Entscheidung (nur Shiftplan, ExtraWork bleibt bezahlt) im Code umgesetzt und kommentiert.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Tests fuer alle vier Faelle + Pfad-Konsistenz</name>
  <files>service_impl/src/test/reporting_no_contract_volunteer.rs, service_impl/src/test/mod.rs</files>
  <behavior>
    - Fall A (no contract): KW ohne EmployeeWorkDetails-Zeile, 30h Shiftplan =>
        report.volunteer_hours == 30, report.overall_hours == 0, report.balance_hours == 0,
        report.expected_hours == 0. (get_report_for_employee_range)
    - Fall B (dynamic regression): Zeile mit is_dynamic=true (expected weighted 0), 30h
        Shiftplan => overall_hours == 30, expected_hours == 30, balance_hours == 0
        (Soll=Ist unveraendert), volunteer_hours == 0.
    - Fall C (expected>0 regression): Zeile 40h, 30h Shiftplan => expected_hours == 40,
        overall_hours == 30, balance_hours == -10, volunteer_hours == 0.
    - Fall D (Konsistenz): gleiche no-contract-Daten durch get_week => derselbe
        ShortEmployeeReport.volunteer_hours == 30, overall == 0, balance == 0
        (Detail- und Week-Report divergieren nicht).
  </behavior>
  <action>
Erzeuge `service_impl/src/test/reporting_no_contract_volunteer.rs`. Kopiere das
TestDeps-/Mock-Builder-Muster aus `reporting_cap_overflow.rs` (vollstaendige
ReportingServiceDeps-Impl + alle Mock-Returns) und die Fixtures aus
`reporting_phase2_fixtures.rs` (`fixture_sales_person`, `fixture_sales_person_id`,
`fixture_work_details_8h_mon_fri`).

- Fall A: `find_by_sales_person_id` / `all` liefert LEERES `Arc<[EmployeeWorkDetails]>`
  (keine Zeile fuer die KW); Shiftplan 30h in KW23/2024; assert wie in <behavior>.
- Fall B: Work-Details mit `is_dynamic: true, ..fixture_work_details_8h_mon_fri()`
  (Zeile vorhanden, weight => 0); assert Soll=Ist unveraendert.
- Fall C: `fixture_work_details_8h_mon_fri()` unveraendert (expected 40); assert
  Regression.
- Fall D: gleiche no-contract-Konstellation via `get_week` (KW23/2024); fuer get_week
  liefert `all_for_week` ggf. eine HashMap ohne Zeile fuer die person — pruefe im
  vorhandenen get_week-Pfad, wie eine person OHNE working_hours-Zeile ueberhaupt in die
  result-Schleife kommt (Schleife iteriert ueber `working_hours`-Map-Keys). Falls
  get_week eine person ohne Zeile NICHT iteriert, dokumentiere das im Test-Kommentar
  und verifiziere stattdessen das Detail-vs-Summary-Konsistenzpaar
  (get_report_for_employee_range vs. get_reports_for_all_employees) mit identischen
  no-contract-Daten => identische volunteer_hours.

Registriere das Modul in `service_impl/src/test/mod.rs` (`mod reporting_no_contract_volunteer;`).
  </action>
  <verify>
    <automated>nix develop --command cargo test --workspace reporting_no_contract 2>&1 | tail -20</automated>
  </verify>
  <done>Alle vier Faelle (A/B/C/D bzw. Detail-vs-Summary-Konsistenz) gruen; Regressionsfaelle B+C beweisen unveraendertes Alt-Verhalten.</done>
</task>

<task type="auto">
  <name>Task 3: Snapshot-Schema-Version-Bump + Begruendung</name>
  <files>service_impl/src/billing_period_report.rs</files>
  <action>
Erhoehe `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION` in
`service_impl/src/billing_period_report.rs` (Z.85) von `8` auf `9`.

Fuege direkt darueber einen Kommentar hinzu, der den Bump begruendet:
"Bump 8->9 (quick-260624-ujk): Die Berechnung des persistierten value_type
`volunteer_hours` aendert sich — geleistete Shiftplan-Stunden in Wochen OHNE
EmployeeWorkDetails-Vertragszeile zaehlen jetzt als Ehrenamt (volunteer) statt
Soll=Ist-neutralisiert. Laut CLAUDE.md (Snapshot Schema Versioning: 'Change the
computation that produces an existing value_type') ist ein Bump Pflicht, damit
Snapshot-Validatoren Schema-Drift von echten Datenfehlern unterscheiden koennen."

Pruefe, ob es einen Test gibt, der `CURRENT_SNAPSHOT_SCHEMA_VERSION` hart auf einen
Wert assertet (z.B. in test/billing_period_report.rs oder
billing_period_snapshot_locking.rs); falls ja, ziehe den erwarteten Wert auf 9 nach.
  </action>
  <verify>
    <automated>nix develop --command cargo test --workspace 2>&1 | tail -20</automated>
  </verify>
  <done>CURRENT_SNAPSHOT_SCHEMA_VERSION == 9 mit Begruendungs-Kommentar; cargo test --workspace komplett gruen (kein hart-codierter Versions-Assert mehr rot).</done>
</task>

</tasks>

<verification>
- `nix develop --command cargo build --workspace` gruen.
- `nix develop --command cargo test --workspace` gruen (inkl. neuer no-contract-Tests
  und bestehender reporting_cap_overflow / billing_period-Snapshot-Tests).
- Manuelle Code-Pruefung: alle drei Report-Pfade (hours_per_week,
  get_reports_for_all_employees, get_week) unterscheiden no-contract von dynamic.
</verification>

<success_criteria>
- no-contract-Woche: geleistete Shiftplan-Stunden => volunteer_hours, overall/balance
  unbeeinflusst (Saldo +-0, Stunden als Ehrenamt sichtbar).
- dynamic-Woche (expected=0, Zeile vorhanden): Soll=Ist unveraendert.
- expected>0-Woche: unveraendert.
- Detail-Report und Summary divergieren nicht.
- CURRENT_SNAPSHOT_SCHEMA_VERSION um 1 erhoeht, begruendet.
- Keine Doppelzaehlung mit booking_information-Band-Logik (disjunkt: is_paid).
</success_criteria>

<output>
Nach Abschluss: Working Copy mit Aenderungen stehen lassen. NICHTS committen
(kein git, kein jj) — der User committet manuell.
</output>
