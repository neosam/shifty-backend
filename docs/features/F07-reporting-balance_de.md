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
- `service_impl::reporting` (3261 Zeilen, Business-Logic-Tier),
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
  **Ausnahme seit Phase 57 (CR-03):** Rebooking-getaggte WorkingHours-Rows
  (`source == Rebooking`) werden aus dem Soll dieses Zweigs
  herausgerechnet — sonst hebt das `+h`-ExtraWork-Bein eines
  Umbuchungs-Paars zugleich `overall` und `expected` (Balance bleibt
  neutral), während das `−h`-VolunteerWork-Bein `volunteer_hours` sehr
  wohl senkt: `h` Stunden würden verschwinden. Die Bereinigung betrifft
  **nur** das Soll — in `overall_hours` und `balance` zählt die Row
  regulär mit (VOL-ACCT-04). Symmetrisch zur `availability()`-Ausnahme in
  `assemble_weeks`.
  Siehe die Drei-Fälle-Unterscheidung in `hours_per_week`
  (`reporting.rs:1444-1454`) und die parallele Logik in `get_reports_for_all_employees`
  (`reporting.rs:388-500`) sowie `get_week` (`reporting.rs:1006-1156`).
- **Volunteer-Merge**: `volunteer_hours` einer Woche ist
  `manual_volunteer + auto_volunteer (cap) + no_contract_volunteer`
  (`reporting.rs:1539-1545`). **Seit Phase 57 (WR-01) gilt das in allen
  drei Read-Pfaden.** Der no-contract- und der dynamische Zweig von
  `get_reports_for_all_employees` bauten den Wert bis dahin aus reinen
  Shiftplan-Größen und summierten manuelle `VolunteerWork`-Rows
  überhaupt nicht, während das `+h`-ExtraWork-Bein eines Rebooking-Paars
  über den Kategorie-Filter sehr wohl zählte — im no-contract-Zweig
  zählten dieselben Stunden damit doppelt, im dynamischen Zweig
  verschwand das Paar komplett (`planned_hours = overall_hours`). Beide
  Zweige spiegeln jetzt `hours_per_week` / `assemble_weeks`, inklusive
  der CR-03-Soll-Bereinigung für Rebooking-getaggte ExtraWork-Rows im
  dynamischen Zweig. Festgenagelt von
  `rebooking_effectiveness.rs::rebooking_conserves_hours_in_summary_without_contract_row`
  und `::rebooking_effective_in_summary_on_dynamic_contract`.
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
- **Rebooking-Aggregation — Kategorie-Semantik statt Marker**
  (VOL-ACCT-04, D-57-06, seit Phase 57, Milestone v2.7): die
  `extra_hours`-Read-Aggregate filtern **nicht** nach Provenienz. Was
  eine Zeile beiträgt, entscheidet ihre Kategorie:
  `ExtraHoursCategory::as_report_type()` entscheidet über die
  Balance-Relevanz (`ExtraWork` → `WorkingHours`, zählt in die bezahlte
  Balance; `VolunteerWork` → `Documented`, zählt nicht), und
  `ExtraHoursCategory::availability()` über die Anwesenheit. Ein
  gleich-großes Gegenpaar `(−h VolunteerWork, +h ExtraWork)`, beides mit
  `source = 'rebooking'` gestempelt, **hebt damit die Balance um `h` und
  senkt das Ehrenamt-Ist um `h`** — das ist der Zweck des Übertrags,
  kein Versehen. Absence/Holiday-Aggregate sind nicht betroffen
  (Rebooking-Zeilen landen dort nie — `rebook()` schreibt nur
  `VolunteerWork`/`ExtraWork`).

  **Die einzige verbliebene source-abhängige Stelle** ist
  `assemble_weeks` (`service_impl/src/reporting.rs`, der
  Wochenübersichts-Pfad): dessen balance-relevante
  `extra_working_hours`-Summe wird über `availability()` gebildet, und
  `VolunteerWork.availability()` ist `Available`. Dort wird die
  Konjunktion `VolunteerWork` **und** `source = Rebooking` ausgenommen,
  weil sonst das `−h`-Bein die Anhebung des `+h`-Beins neutralisieren
  würde und das Paar an dieser einen Stelle netto 0 wäre. Gewöhnliche
  (Nicht-Rebooking-)`VolunteerWork`-Zeilen zählen dort unverändert mit —
  das ist die vorbestehende Wochen-gegen-Jahres-Inkonsistenz (RISK-01),
  die Phase 57 bewusst nicht angetastet hat. In `manual_volunteer_hours`
  derselben Funktion bleibt die Rebooking-Zeile absichtlich drin: sie
  ist es, die das Ehrenamt-Ist senkt.

  **Was noch neutral ist:** ausschließlich das VOL-ACCT-01-**Soll**. Es
  stammt aus `committed_voluntary` im Vertrag und liest überhaupt keine
  `extra_hours`, ist über eine Umbuchung hinweg also bit-identisch
  (`test_neutrality_f2_soll`). Die Delta-Zusage — Balance `+h`,
  Ehrenamt-Ist `−h` — nageln `rebooking_raises_balance` /
  `rebooking_lowers_volunteer_ist`
  (`service_impl/src/test/rebooking_effectiveness.rs`),
  `rebooking_effective_single_manual`
  (`service_impl/src/test/rebooking_reconciliation.rs`), der
  Wochenübersichts-Wächter `rebooking_effective_in_weekly_summary` für
  die `assemble_weeks`-Site und das Ende-zu-Ende-Referenz-Szenario in
  `service_impl/src/test/rebooking_reference_scenario.rs`
  (8 h bezahlt / 12 h Ehrenamt / Balance −2 → 10 / 10 / 0) fest. Das
  sind die stehenden CI-Wächter: sie werden rot, sobald ein
  Rebooking-Paar die Balance nicht mehr bewegt.

  **Snapshot-Konsequenz:** weil Rebooking-Zeilen jetzt die persistierten
  Werte erreichen, wurde `CURRENT_SNAPSHOT_SCHEMA_VERSION` in Phase 57
  auf **12 → 13** gebumpt (D-57-10) — die Phase-56-Aussage „bleibt bei
  12" ist hinfällig. Siehe F08 §7.4.

  **Historien-Hinweis.** Bis Phase 56 stand an den
  Aggregations-Eingängen aller vier Report-Pfade (`get_week`,
  `get_reports_for_all_employees`, `hours_per_week`,
  `overall_extra_work_hours`) ein unbedingter Marker-Filter
  (`source == ExtraHoursSource::Manual`, D-55-A1), begründet mit
  „Doppelzählung". Er stammte aus der ursprünglichen Fassung von
  VOL-ACCT-03, die Balance-Neutralität forderte — und diese Forderung
  war selbst der Fehler: der Filter machte den gesamten Übertrag ein
  ganzes Milestone lang wirkungslos. Neu gefasst in Phase 57
  (VOL-ACCT-04). Siehe Feature
  [F14 §3 / §8 / §9](./F14-rebooking.md).

### 2.7 Planungs-Soll vs. bilanzrelevantes Soll (Phase 64, PSOLL-01…04)

Seit Phase 64 trägt `ShortEmployeeReport` **zwei** Soll-Größen. Sie sehen
sich ähnlich, sind aber nicht austauschbar: wer sie verwechselt, baut
entweder den Fehler wieder ein, den diese Phase behoben hat, oder bricht
die Verfügbarkeitsrechnung.

| | `expected_hours` — *bilanzrelevantes Soll* | `planning_target_hours` — *Planungs-Soll* |
| --- | --- | --- |
| Wofür der Wert da ist | das Soll-Bein von `balance = worked − expected + carryover` | die Zahl, gegen die ein Planer die verteilten Schichten hält |
| Dynamischer Vertrag (`is_dynamic == true`) | absichtlich **`0.0`** — *Soll = Ist*, die Balance muss `0` bleiben | absichtlich **`> 0`** — die reduzierte Zusage (Vertragsstunden **plus** ehrenamtliche Zusage) |
| Vertrag mit `expected_hours == 0` und Zusage > 0 | `0.0` | **`> 0`** — seit Plan 64-05; das ursprüngliche Gate lieferte hier `0.0` |
| Nicht-dynamischer Vertrag, **Zusage 0** | Vertragsstunden minus Abwesenheits- und Feiertagsreduktion | **dieselbe Zahl**, strukturell (siehe unten) |
| Nicht-dynamischer Vertrag, **Zusage > 0** | wie oben, ohne Ehrenamt | **größer** — um genau die um Abwesenheiten und Feiertage reduzierte Zusage |
| Enthält die ehrenamtliche Zusage (`committed_voluntary`) | **nein — nie** | **ja** [D-64-05] |
| Geht in Balance / Carryover / Billing-Snapshot / `available_hours` ein | **ja** | **nein — nie** |
| Wo der Wert sichtbar wird | die Stundenkonto-Flächen (Mitarbeiter-Detail, Wochen-Histogramm, Mitarbeiterliste) | ausschließlich die Übersicht unterhalb des Schichtplans (§6.1) |

**Was das Feld ist.** `planning_target_hours` ist die feiertags- und
abwesenheitsreduzierte **Zusage** der Woche
(`service/src/reporting.rs:164`, gespiegelt auf `ShortEmployeeReportTO`,
`rest-types/src/lib.rs:392`, `#[serde(default)]`). Es beantwortet
*„für wie viele Stunden soll diese Person eingeplant werden"* — nicht
*„wie viele Stunden erwartet die Bilanz"*.

**Es ist ausdrücklich KEINE Vertragsgröße** [D-64-05, Plan 64-05]. Der
Zweck des Feldes ist die Schichtplanung: die planende Person muss sehen,
*was die jeweilige Person für die Woche zusagt*. Ob diese Stunden bezahlt
sind, ist dafür unerheblich — deshalb fließt die ehrenamtliche Zusage
(`employee_work_details.committed_voluntary`) ein. Die durchgehende
Trennung von `expected_hours` und `committed_voluntary` gilt für die
**Bilanz** und bleibt dort unberührt; für die **Planung** gilt sie nicht.
Wer die Zusage hier wieder heraustrennt, „weil die Bilanz das auch so
macht", bricht den Zweck des Feldes.

Abwesenheiten und Feiertage reduzieren den Ehrenamts-Anteil **mit** — wer
in der Zeit weniger da ist, sagt auch ehrenamtlich weniger zu. Die
Reduktion greift *proportional*, nicht alles-oder-nichts (§ D-64-07
unten).

**Wofür es ausdrücklich NICHT benutzt wird.** Balance, Carryover, der
Billing-Period-Snapshot und die Verfügbarkeitsrechnung lesen weiterhin
ausschließlich `expected_hours` / `dynamic_hours`.
`WorkingHoursPerSalesPerson.available_hours` liest an beiden seiner
Stellen unverändert `report.expected_hours`
(`service_impl/src/booking_information.rs:644` und `:818`) und ist damit
für einen dynamischen Vertrag weiterhin `0.0` — das ist gewollt und
durch `test_available_hours_unchanged_dynamic` festgenagelt.

**Die Formel (an beiden Konstruktionsstellen wortgleich).**

```rust
// service_impl/src/reporting.rs:1162-1186 (assemble_weeks),
// wortgleich gespiegelt bei :1562-1577 (get_reports_for_all_employees)
let planning_base_for_target = dynamic_hours + committed_voluntary_for_target;
let target_reduction_scale = if dynamic_hours > 0.0 {
    planning_base_for_target / dynamic_hours
} else {
    1.0f32
};
let reduce_target = has_contract_row && planning_base_for_target > 0.0;
let planning_target_hours = if reduce_target {
    planning_base_for_target
        - (absence_manual_for_target
            + absence_derived_for_target
            + holiday_derived_for_target)
            * target_reduction_scale
} else {
    0.0f32
};
```

`committed_voluntary_for_target` ist die Summe von
`committed_voluntary_weight_for_week(year, week, wh)` über die in der
Woche aktiven Vertragszeilen (`reporting.rs:2451`, Aggregation per SUM
nach dem Präzedenzfall D-OVERLAP-AGG).

**Warum das Feld ein eigenes Gate hat, und warum es sich mit Plan 64-05
geändert hat.** Ursprünglich (64-01) lautete es
`has_contract_row && dynamic_hours > 0.0`. Das war korrekt, solange die
Basis `dynamic_hours` war — mit der Zusage in der Basis ist es der Defekt
selbst. Die drei Eigenschaften, die das heutige Gate trägt:

- *Es darf das Bilanz-Gate nicht mitbenutzen.* Der Bilanz-Zweig hängt an
  `planned_hours`, Element 0 des `weight_for_week`-Rückgabetupels, und
  dieser Wert wird bei `is_dynamic` **hart auf `0.0` gezwungen**. Seine
  Bedingung `!has_contract_row || planned_hours <= 0.0`
  (`reporting.rs:1045`) ist deshalb für *jede* Woche eines dynamischen
  Vertrags wahr und würde die Reduktion genau dort überspringen, wo sie
  gebraucht wird. Der bezahlte Anteil der Planungs-Basis wird deshalb an
  Element 1 desselben Tupels (`dynamic_hours`) gemessen — dem einzigen
  ungezwungenen Wert der Kette. **Die beiden Bedingungen beim nächsten
  Refactoring NICHT wieder „vereinheitlichen".**
- *Es feuert für die Klasse `expected_hours == 0` mit Zusage > 0.* Genau
  hier hat die alte zweite Hälfte am **falschen Wert** gemessen: eine
  Person mit 0 Vertragsstunden und echter Zusage hat `dynamic_hours == 0`,
  das alte Gate lieferte für sie `0.0`. Seit Plan 64-05 wird an
  `planning_base_for_target` gemessen, und die Zusage ist Teil davon.
  Festgenagelt durch
  `planning_target_zero_expected_hours_with_commitment`.
- *Es zieht nie von einer Null-Basis ab.* Sind Vertragsstunden UND Zusage
  0 — oder liegt die Woche außerhalb der Vertragsgrenzen
  (`relation == 0.0`) — ist `planning_base_for_target` exakt `0`, das Gate
  schaltet nicht, und das Planungs-Soll ist byte-identisch `0.0` wie
  `expected_hours`. Das ist dieselbe Garantie wie vorher, nur an der
  richtigen Basis gemessen. Eine Vertragszeile ohne einen einzigen
  gesetzten Wochentag ergibt `relation = 0/0 = NaN` (vorbestehend in
  `weight_for_week` seit v1); `NaN > 0.0` ist `false`, das Gate schaltet
  also auch dort nicht und das Planungs-Soll ist `0.0` statt `NaN`.

**Warum die Byte-Identität für nicht-dynamische Verträge strukturell
folgt — aber nur bei Zusage 0.** Bei `committed_voluntary == 0` ist
`committed_voluntary_for_target == 0`, also
`planning_base_for_target == dynamic_hours`, also
`target_reduction_scale == 1.0` — damit steht Zeile für Zeile dieselbe
Rechnung da, die 64-01 geschrieben hat, und bei nicht-dynamischem Vertrag
tragen Element 0 und Element 1 des `weight_for_week`-Tupels denselben
Wert. Also gilt `planning_target_hours == expected_hours`
konstruktionsbedingt und nicht zufällig. Festgenagelt durch
`test_planning_target_matches_expected_for_fixed_contract` und durch
Block 3 von `planning_target_fixed_contract_differs_by_commitment`.

**Mit Zusage ist die Differenz genau die reduzierte Zusage.** Gemessen an
`planning_target_fixed_contract_differs_by_commitment` (40-h-Vertrag,
Zusage 10.0): in einer ungestörten Woche `50.0` gegen `40.0`, Differenz
exakt **10.0**; in einer Woche mit drei abwesenden Werktagen (24 h)
`20.0` gegen `16.0`, Differenz exakt **4.0** (= `10 × (1 − 24/40)`).

**Bewusst kein `.max(0.0)`.** Der Bilanzpfad klemmt ebenfalls nicht. Das
Gate verhindert den *strukturellen* Nullbasis-Fall; es verhindert nicht,
dass Reduktionen größer als die Basis werden. Seit Plan 64-05 ist das
**assertiertes** Verhalten: `planning_target_not_clamped_when_reductions_exceed_base`
misst für einen 0-h-Vertrag mit Zusage 10.0 und einer manuellen
`Vacation`-Zeile von 20.0 h ein Planungs-Soll von **−10.0**. Ein
`.max(0.0)` liefert dort `0.0` und bricht diesen Test. Wer später ein
Klemmen nachrüstet, muss beide Größen klemmen oder keine — und zuerst die
Produktfrage stellen, ob ein negatives Planungs-Soll für die planende
Person lesbar ist.

**D-64-06 — die Zusage wird mit `relation` skaliert, nicht tages-basiert
pro-rata.** Für den Wochen-Betrag der Zusage existieren im Code drei
Kandidaten. An einer Randwoche (Vertrag Mo–Fr, endet Mittwoch, Zusage
7.0) liefern sie drei verschiedene Zahlen:

| Kandidat | Wochen-Betrag | Bewertung |
| --- | --- | --- |
| `committed_voluntary_for_calendar_week` (`reporting.rs:130`) — SUM ohne Skalierung | **7.0** | verworfen: meldet die ganze Wochenzusage für eine Woche, die der Vertrag nur zu drei Fünfteln abdeckt |
| `committed_voluntary_prorata_for_week` (`reporting.rs:163`) — `committed_voluntary / 7.0` je Tag (D-F2-01) | **3.0** (3/7) | verworfen: mischt Kalendertags- und Werktags-Skalierung in einer angezeigten Zahl; die abgezogenen Reduktionsterme sind werktags-basiert |
| **`committed_voluntary_weight_for_week` (`reporting.rs:2451`) — `committed_voluntary * relation`** | **4.2** (3/5) | **gewählt**: eine Skalierung für die ganze Zahl, dieselbe Quelle für `relation` wie der bezahlte Anteil (`potential_workdays_for_week`) |

Festgenagelt durch
`committed_voluntary_weight_pins_relation_scaling_d_64_06`, das die
beiden verworfenen Zahlen als explizite Gegen-Assertions führt.

**Warum `committed_voluntary_target_in_range` NICHT wiederverwendet
wird.** Seine Abwesenheits-Semantik ist **whole-week-out** (D-54.5-01):
eine ISO-Woche mit *einem* Abwesenheitstag trägt **0** zum Soll bei. Für
das F2-Soll (Ehrenamtskonto) ist das richtig — es ist eine
**Bilanz**größe und stellt Ist/Soll-Symmetrie her. Für das Planungs-Soll
wäre es falsch: ein Urlaubstag würde die *ganze* Wochenzusage löschen,
während der bezahlte Anteil nur einen Tag verliert. D-64-05 verlangt
ausdrücklich *proportional*, nicht alles-oder-nichts. **Folge:
Planungs-Soll und F2-Soll dürfen sich an Randwochen unterscheiden. Das
ist gewollt und keine Drift, die jemand „aufräumen" soll.**

**D-64-07 — die Reduktion greift proportional, über
`target_reduction_scale`.** Eine reine Subtraktion der Reduktionsterme
von der vergrößerten Basis würde **nur den bezahlten Anteil** kürzen: die
Abwesenheitsstunden sind aus `expected_hours` abgeleitet (Stunden je
Werktag), nicht aus `expected_hours + committed_voluntary`. Beispiel: 40 h
Vertrag + 10 h Zusage, ein Abwesenheitstag von fünf → Reduktion 8 h →
`50 − 8 = 42`, obwohl die Person nur 4/5 der Woche da ist und 4/5 von 50
= **40** zusagt. Deshalb wird die Reduktion um genau den Faktor
hochskaliert, um den die Basis gewachsen ist. Festgenagelt durch
`planning_target_commitment_reduced_by_absence_parity` (gemessen: 50.0 in
der ungestörten Woche, 20.0 in der Abwesenheitswoche; die reine
Subtraktion ergäbe 26.0).

**Vorbestehende Asymmetrie, von Phase 64 bewusst nicht angefasst.** Die
beiden Konstruktionsstellen (§3) bilden ihren *manuellen*
Abwesenheits-Rohwert über **verschiedene** Filter: `assemble_weeks`
summiert über `category.availability() == Availability::Unavailable`
(`reporting.rs:917`), `get_reports_for_all_employees` dagegen über
`category.as_report_type() == ReportType::AbsenceHours`
(`reporting.rs:1450`). Die Asymmetrie ist älter als Phase 64 und betrifft
heute schon die **Bilanz**pfade; sie zu vereinheitlichen wäre eine
Änderung an der Bilanz gewesen und lag damit außerhalb des Scopes. Hier
festgehalten, damit die nächste Leserin sie nicht für etwas hält, das
Phase 64 eingeführt hat.

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

| Ehrenamtliche Zusage (nur Planungs-Soll) | `employee_work_details.committed_voluntary`, skaliert über `committed_voluntary_weight_for_week` (`reporting.rs:2451`) | `committed_voluntary` |

Das **Planungs-Soll** (`planning_target_hours`, §2.7) speist sich aus
**vier** Quellen: Vertrag, Abwesenheiten, Feiertage **und** der
ehrenamtlichen Zusage aus `employee_work_details.committed_voluntary`.
Gegenüber dem bilanzrelevanten Soll unterscheiden sich **Basis**
(Vertrag + Zusage statt nur Vertrag), **Gate**
(`planning_base_for_target > 0.0` statt `planned_hours > 0.0`) **und** die
**Skalierung der Reduktion** (`target_reduction_scale`, D-64-07). Es ist
also nicht „dieselbe Rechnung mit einem anderen Gate".

#### Die zwei `ShortEmployeeReport`-Konstruktionsstellen

`ShortEmployeeReport` wird an **zwei unabhängigen Stellen** in
`service_impl/src/reporting.rs` zusammengebaut, und keine delegiert an
die andere:

| Stelle | Funktion | Konsumiert von |
| --- | --- | --- |
| **A** | `assemble_weeks` (`reporting.rs:769`, Pro-Wochen-Körper um `:1121`) | `get_week` (`:2020`) **und** `get_year` (`:2101`) — sie teilen sich diesen Helfer, und genau das hält sie byte-identisch (Phase 52, D-52-06) |
| **B** | `get_reports_for_all_employees` (`reporting.rs:1212`, Pro-Wochen-Körper um `:1476`) | `GET /report/?year=…&until_week=…` |

**Die Regel, die Phase 64 auf die harte Tour gelernt hat:** Ein neues
Feld an diesem Struct muss an **beiden** Stellen gefüllt werden, und es
braucht einen Test, der die beiden *gegeneinander* hält — nicht zwei
Tests, die je für sich grün werden können. Phase 64s
`test_planning_target_parity_both_sites` ist genau dieser
Cross-Site-Paritätstest; im RED-Lauf wäre er ohne seine dritte Assertion
(`> 0.0`) auf `0.0 == 0.0` grün geworden. Zwei isolierte Einzeltests
hätten nicht gefunden, dass der dynamische Zweig von Stelle B die
Feiertags-Terme vor dieser Phase gar nicht kannte.

**`has_pending_rebooking` / `pending_rebooking_id` sind die dokumentierte
Ausnahme, nicht die Vorlage.** Beide werden ausschließlich an Stelle B
berechnet (`reporting.rs:1712`) und an Stelle A hart auf `false`/`None`
gesetzt (`reporting.rs:1193`). Der Grund steht im Code: der
HR-Alert-Banner (Phase 55, D-55-B) wird ausschließlich über den
`/report`-Listenpfad getrieben und hat im Pro-Woche-Pfad
`get_week`/`assemble_weeks` keine Bedeutung. Jedes weitere Ein-Stellen-Feld
braucht dieselbe ausdrückliche Begründung — der Default ist Parität.

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
  siehe Feature [F14](./F14-rebooking.md). **Reader-Impact (Stand seit
  Phase 57):** die Aggregations-Eingänge in
  `service_impl/src/reporting.rs` filtern **nicht** nach Provenienz; was
  eine Zeile beiträgt, entscheidet die Kategorie-Semantik (siehe §2.6).
  Der `source`-Wert bleibt als Herkunftskennzeichnung am Datensatz und
  wird für die Aggregation an genau einer Stelle ausgewertet — der
  `availability()`-basierten Summe in `assemble_weeks`. Das
  Voluntary-Stats-Ist-Aggregat delegiert an
  `EmployeeReport::volunteer_hours` (via
  `ReportingService::get_report_for_employee_range`) und erbt damit
  dasselbe Verhalten. Zeilen ohne `rebooking`-Writer gehen weiterhin
  identisch in die Balance ein (Backfill landet per Column-DEFAULT auf
  `manual`).

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

#### `ShortEmployeeReport` — die zwei Soll-Felder

Auszug aus `service/src/reporting.rs:148-…` (Phase 64 hat die letzte der
vier Stunden-Größen ergänzt):

```rust
pub struct ShortEmployeeReport {
    pub sales_person: Arc<SalesPerson>,
    pub balance_hours: f32,
    pub dynamic_hours: f32,
    pub expected_hours: f32,          // bilanzrelevantes Soll — 0.0 bei dynamischem Vertrag
    pub overall_hours: f32,
    pub planning_target_hours: f32,   // Phase 64 (PSOLL-01…04) — Planungs-Soll: die
                                      // reduzierte ZUSAGE der Woche inkl. Ehrenamt
                                      // [D-64-05]; > 0 auch bei dynamischem und bei
                                      // 0-Stunden-Vertrag mit Zusage; geht in KEINE
                                      // Bilanz ein (§2.7)
    …
}
```

1:1 gespiegelt auf `ShortEmployeeReportTO`
(`rest-types/src/lib.rs:392`, `#[serde(default)]`, damit die Ergänzung
Wire-additiv bleibt) plus das Feld-Mapping im `From`-Impl (`:414`).

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

- `ShortEmployeeReportTO` (`rest-types/src/lib.rs:380-402`): kompakte
  Zeile für die HR-Übersicht. **Seit Phase 64** trägt sie das zusätzliche
  Antwortfeld `planning_target_hours: f32` (§2.7) — die reduzierte
  **Zusage** der Woche inklusive Ehrenamt [D-64-05] —, ausgeliefert von
  beiden Kurz-Report-Endpoints — `GET /report/?year=…&until_week=…` und
  `GET /report/week/{year}/{calendar_week}`. Die Ergänzung ist rein
  additiv: **keine neue Route, kein neuer Handler und kein neuer
  `Dioxus.toml`-Proxy-Eintrag** ist entstanden, und `#[serde(default)]`
  hält ein altes Payload ohne das Feld deserialisierbar (zu `0.0`).
  **Plan 64-05 hat die Wire-Form NICHT angefasst:** dasselbe Feld,
  derselbe Typ, dieselben Endpoints — nur die Semantik des Werts ist jetzt
  eine Zusage- statt einer Vertragsgröße.
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
- **`page/shiftplan.rs`** — der Schichtplan selbst; unterhalb des Plans
  rendert er `component/working_hours_mini_overview.rs`
  (`WorkingHoursMiniOverview`, `shiftplan.rs:1724`), gespeist aus
  `GET /report/week/{year}/{calendar_week}` über
  `loader::load_working_hours_minified_for_week` →
  `build_working_hours_mini` → `WorkingHoursMini`. **Das ist die einzige
  Fläche, die das Planungs-Soll zeigt** (§6.5).

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

### 6.5 Welche Fläche zeigt welches „Soll" (Phase 64)

`ShortEmployeeReportTO` trägt jetzt zwei Soll-Größen (§2.7) — eine
Bilanz-Größe und eine Zusage-Größe. Die
Aufteilung im Frontend ist gewollt und folgt dem Transport-Objekt, das
die jeweilige Fläche liest — die meisten Stundenkonto-Flächen *können*
das Planungs-Soll gar nicht zeigen, weil ihr DTO es nicht hat.

**Zeigt das Planungs-Soll (`planning_target_hours`)** — genau drei
Anzeigestellen, alle in `component/working_hours_mini_overview.rs`, alle
gespeist aus `ShortEmployeeReportTO`:

| Stelle | Dateiposition | Was daran hängt |
| --- | --- | --- |
| Karten-Layout, `target` | `working_hours_mini_overview.rs:123` | die Soll-Zahl der `Ist / Soll`-Zeile, dazu Fortschrittsbalken-Füllung und Farbschwelle (`progress_bar_class`, `hours_text_class`, `progress_bar_percent` lesen dieselbe Bindung) |
| Tabellen-Layout, `target` pro Zeile | `:238` | die Soll-Spalte einer Zeile, ihr Auslastungsbalken und der Prozentwert |
| Tabellen-Layout, `total_target` in der Fußzeile | `:186` | die Soll-Summe der Spalte darüber |

Ein Quellcode-Invariantentest
(`working_hours_mini_overview_reads_planning_target_only`) hält das bei
exakt drei Lesezugriffen, verbietet jeden Rückfall auf
`.expected_hours` / `.dynamic_hours` und verbietet Arithmetik unmittelbar
am Feld (PSOLL-02: das Frontend *liest* den Wert, es leitet ihn nicht
ab).

**Zeigt weiterhin das bilanzrelevante Soll (`expected_hours`)** — diese
Flächen lesen `WorkingHoursReportTO` bzw. `EmployeeReportTO`, die Phase
64 nicht angefasst hat:

| Fläche | Quelle |
| --- | --- |
| Mitarbeiter-Detail: Jahres-/Wochenliste, Wochen-Detailkarte, Differenzspalte (`component/employee_view.rs:805,867,869`) | `WorkingHoursReportTO.expected_hours` |
| Wochen-Histogramm: Referenzlinie und Balkenfarbe (`component/employee_weekly_histogram.rs:119,126,167`) | `WorkingHoursReportTO.expected_hours` |
| Mitarbeiterliste: `target_hours_for` (letzte Woche) (`component/employees_list.rs:51`) | `WorkingHoursReportTO.expected_hours` |
| `state/employee.rs:368` — `expected_working_hours` | `EmployeeReportTO.expected_hours` |

**Merksatz:** Das Planungs-Soll erscheint **ausschließlich** in der
Wochen-Übersicht unter dem Schichtplan; jede Stundenkonto-Fläche zeigt
unverändert das bilanzrelevante Soll. Die drei Anzeigestellen zeigen
damit eine **Zusage**-Größe inklusive Ehrenamt [D-64-05] — sie kann für
dieselbe Person größer sein als jede Zahl auf den Stundenkonto-Flächen,
und das ist gewollt.

Der zweite Konsument von `ShortEmployeeReportTO` im Frontend,
`impl From<&ShortEmployeeReportTO> for Employee`
(`state/employee.rs:297`), setzt `expected_working_hours: 0.0`
unbedingt und liest überhaupt kein Soll-Feld des Reports — er kann
folglich nicht stillschweigend von der Mini-Übersicht auseinanderlaufen.

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
  **Einzige Ausnahme seit Phase 57 (CR-03):** ein Rebooking-Paar
  (`source == Rebooking`) hebt die Balance auch in der dynamischen Woche um
  `h` und senkt `volunteer_hours` um `h` — das `+h`-Bein wird aus dem
  Soll=Ist-Soll herausgerechnet. Andernfalls würden die umgebuchten
  Stunden das Ehrenamt-Konto verlassen, ohne im bezahlten anzukommen.
  Festgenagelt von
  `rebooking_effectiveness.rs::rebooking_conserves_hours_on_dynamic_contract`,
  das zusätzlich prüft, dass `hours_per_week` und `assemble_weeks`
  dieselbe Wirkung ausweisen.

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

- `service_impl/src/test/reporting_additive_merge.rs` (1575 Zeilen) —
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
- `service_impl/src/test/reporting_phase2_fixtures.rs` (199 Zeilen) —
  Fixtures für die frühe Phase-2-Regression; zugleich das Spiegel-Paar
  (dynamischer vs. fester 40-h-Vertrag plus Urlaubs-/Krankheits-Perioden),
  auf dem die Phase-64-Planungs-Soll-Suite aufsetzt.
- `service_impl/src/test/reporting_planning_target.rs` (1087 Zeilen) —
  Phase 64 (PSOLL-01/03/04), zehn Tests rund um das Planungs-Soll
  (§2.7). Die ersten fünf stammen aus 64-01 (Vertrags-Semantik, alle
  Fixtures mit Zusage 0), die letzten fünf aus Plan 64-05
  (Zusage-Semantik):

  | Test | Zweck | gefangene Fehlimplementierung |
  | --- | --- | --- |
  | `test_planning_target_dynamic_get_week` | ein dynamischer Vertrag meldet in `get_week` `planning_target_hours = 40.0`, während `expected_hours` bei `0.0` bleibt — Konstruktionsstelle A | Feld neutral (`0.0`) |
  | `test_planning_target_dynamic_all_employees` | derselbe Vertrag meldet in `get_reports_for_all_employees` über zwei Vertragswochen `80.0` — Konstruktionsstelle B | Stelle B nicht gefüllt |
  | `test_planning_target_parity_both_sites` | **der Cross-Site-Paritätstest**: beide Stellen liefern für dieselbe Eingabe denselben Wert (`56.0`), und Abwesenheiten reduzieren ihn nachweislich (`0 < 56 < 80`). Erst die dritte Assertion (`> 0.0`) macht ihn zum Treiber statt zu einem `0.0 == 0.0`-Durchläufer | Wiederverwendung der auf `0` gezwungenen Bilanz-Reduktionsterme (`80.0`) |
  | `test_planning_target_matches_expected_for_fixed_contract` | Byte-Identität mit `expected_hours` für nicht-dynamische Verträge **bei Zusage 0**, inkl. des 0-Stunden-Vertrags-Regressionsblocks | Wegfall der Null-Basis-Garantie (negativer Wert) |
  | `test_available_hours_unchanged_dynamic` | Regressionsschloss: `available_hours` bleibt bei dynamischem Vertrag `0.0`, obwohl der Report jetzt ein Planungs-Soll von `40.0` trägt | Umstellung der `available_hours`-Zuweisungen auf das neue Feld (T-64-02) |
  | `planning_target_includes_commitment_dynamic_both_sites` | Zusage 10.0 ⇒ `50.0` (A) und `100.0` (B), `expected_hours` bleibt `0.0` | Zusage ignoriert (`40.0` / `80.0` — der Zustand vor Plan 64-05) |
  | `planning_target_commitment_reduced_by_absence_parity` | KW 22 `50.0`, KW 23 `20.0` (einzeln assertiert), Summe `70.0` an beiden Stellen | Zusage ignoriert (`56.0`); reine Subtraktion ohne `target_reduction_scale` (`76.0`); Feld neutral (`0.0`) |
  | `planning_target_fixed_contract_differs_by_commitment` | PSOLL-04 in seiner neuen Fassung: Differenz exakt `10.0` (ungestört) bzw. `4.0` (drei Abwesenheitstage), und Byte-Identität bei Zusage 0 | Zusage schlägt in `expected_hours` durch; Byte-Identität bei Zusage 0 verloren |
  | `planning_target_zero_expected_hours_with_commitment` | `expected_hours == 0` mit Zusage 10.0 ⇒ `10.0` (A) und `20.0` (B) | das 64-01-Gate bleibt stehen (`0.0`) |
  | `planning_target_not_clamped_when_reductions_exceed_base` | Reduktionen > Basis ⇒ **`−10.0`**; an Stelle B über zwei Fenster zerlegt (bis KW 22 `10.0`, bis KW 23 `0.0`, Wochenbeitrag `−10.0`) | ein `.max(0.0)`-Clamp (`0.0`) |

  Dazu vier Pure-fn-Tests in `reporting.rs` (Modul
  `test_committed_voluntary_weight_for_week`, ab `:3439`) — die einzige
  Stelle, an der die Verteilungsentscheidung D-64-06 direkt messbar ist
  (die Integrationstests messen sie nur über die Vollwoche, wo alle drei
  Kandidaten dieselbe Zahl liefern):

  | Test | gemessen | gefangene Fehlimplementierung |
  | --- | --- | --- |
  | `committed_voluntary_weight_full_week_is_unscaled` | `10.0` | eine grundsätzliche Division (durch 7 bzw. `workdays_per_week`) |
  | `committed_voluntary_weight_pins_relation_scaling_d_64_06` | `4.2` | `3.0` = tages-basierte 7-Tage-Verteilung; `7.0` = unskalierte SUM (beide als explizite Gegen-Assertions) |
  | `committed_voluntary_weight_outside_contract_is_zero` | `0.0` | ungeprüftes Durchreichen von `committed_voluntary` |
  | `committed_voluntary_weight_zero_commitment_is_zero` | `0.0` | Skalierung von `expected_hours` statt `committed_voluntary` |

  Das quellseitige Gegenstück liegt im Frontend:
  `working_hours_mini_overview_reads_planning_target_only` (§6.5) nagelt
  PSOLL-02 fest und wurde gegen zwei absichtlich wiedereingeführte
  Verletzungen geprüft (Rückfall auf ein Altfeld, Arithmetik am Feld) —
  beide ließen ihn fallen.
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
- **Phase 64 (PSOLL-01…04, v2.7):** `planning_target_hours` — eine
  zweite, *nicht* bilanzrelevante Soll-Größe an `ShortEmployeeReport` /
  `ShortEmployeeReportTO`, damit ein dynamischer Vertrag endlich ein
  planbares Ziel hat statt der `0`, die er aus der Bilanz-Semantik geerbt
  hat (§2.7, §6.5). Jede Bilanzgröße bleibt unverändert. Die Phase hat
  zwei konservierenswerte Fallen gefunden:

  1. **Das Reduktions-Gate hing am gezwungenen Nullwert.** Die
     naheliegende Umsetzung übernimmt die Bedingung des Bilanz-Zweigs,
     und die hängt an `planned_hours` — einem Wert, den
     `weight_for_week` bei dynamischen Verträgen auf `0.0` *zwingt*. Die
     Reduktion wäre damit genau für die Verträge ausgefallen, für die
     das Feature gebaut wurde. Der bezahlte Anteil der Basis wird deshalb
     an `dynamic_hours` gemessen, dem ungezwungenen Element desselben
     Tupels, und `has_contract_row` bleibt die zweite Gate-Hälfte.
  2. **Der zweiten Konstruktionsstelle fehlte die Feiertags-Rechnung
     komplett.** Im dynamischen Zweig von
     `get_reports_for_all_employees` gab es die Feiertags- und
     Abwesenheits-Bausteine vor dieser Phase gar nicht; sie mussten vor
     die Dreiwege-Verzweigung gehoben werden. Ein Test pro Stelle wäre
     auf jeder Stelle für sich grün geworden — nur der
     Cross-Site-Paritätstest (§8.1) hält die beiden gegeneinander.

  3. **Das Reduktions-Gate war mechanisch richtig und fachlich falsch.**
     Es war korrekt für eine *Vertrags*größe und falsch für eine
     *Zusage*größe: die ehrenamtliche Zusage (`committed_voluntary`) kam
     in keiner der beiden Formeln vor, und eine Person mit
     `expected_hours == 0` und echter Zusage bekam `0.0` angezeigt. Der
     Fehler lag nicht in der Mechanik, sondern in der **Fachdefinition** —
     und er wurde erst im UAT gefunden, nicht von neun grünen Tests. Neun
     Tests können eine falsche Definition beliebig genau absichern.
     Geschlossen durch **Plan 64-05** (Gap-Closure, D-64-05/06/07): das
     Planungs-Soll ist eine Zusage-Größe, die Zusage fließt ein, und
     Abwesenheiten reduzieren sie proportional mit.

  `CURRENT_SNAPSHOT_SCHEMA_VERSION` **bleibt bei 13** — warum das
  strukturell gilt, steht unten. Plan 64-05 hat die Computation an
  `ShortEmployeeReport` geändert und trotzdem **nicht** gebumpt; der
  Beweis dafür steht ebenfalls unten.

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
`snapshot_schema_version` (aktuell **13** seit Phase 57 / D-57-10, siehe
`service_impl/src/billing_period_report.rs:140`). Der Snapshot-Writer
konsumiert `EmployeeReport` und schreibt eine Reihe von
`value_type`-Zeilen. Details siehe
[F08 Billing Period](./F08-billing-period.md).

**Regel für Reporting-Änderungen:** Jede Änderung an der Berechnung
eines im Snapshot-persistierten `value_type` (Formel, Inputs, Filter)
MUSS die `CURRENT_SNAPSHOT_SCHEMA_VERSION` bumpen — sonst driftet der
Validator (siehe CLAUDE.md-Absatz „Billing Period Snapshot Schema
Versioning").

**Warum Phase 64 nicht gebumpt hat.** `planning_target_hours` (§2.7)
sitzt an `ShortEmployeeReport`. Der Snapshot-Writer
(`build_and_persist_billing_period_report`) konsumiert aber
**`EmployeeReport`** — einen anderen Typ, erzeugt von einem dritten
Codepfad (`get_report_for_employee_range` → `hours_per_week`), der dieses
Feld überhaupt nicht hat. Der neue Wert kann einen persistierten
`value_type` also nicht einmal versehentlich erreichen, und die
Berechnung bzw. die Input-Menge keines bestehenden `value_type` wurde
angefasst. Das ist ein *struktureller* Grund, keine Zusicherung: ein Grep
nach `planning_target_hours` in `billing_period_report.rs` liefert nichts.
Die Version bleibt bei **13**.

**Warum Plan 64-05 ebenfalls nicht gebumpt hat, obwohl es eine
Computation geändert hat.** Geändert wurden die beiden
`ShortEmployeeReport`-Konstruktionsstellen (`assemble_weeks` und
`get_reports_for_all_employees`) — und dort ausschließlich der
Planungs-Soll-Block, der in keine Bilanzgröße eingeht. Der dritte
`weight_for_week`-Aufrufer, `hours_per_week`, speist `EmployeeReport` und
damit den Snapshot; er ist von 64-05 nicht angefasst, ebenso wenig
`booking_information.rs` oder `billing_period_report.rs`. Belegt durch
Zählergates statt durch Zusicherung: `available_hours: report.expected_hours`
kommt weiterhin genau **zweimal** vor, `CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13`
genau **einmal**, und die vier Dateien stehen in keinem Diff des Plans.
Die volle Test-Suite (1182 Tests, davon 9 neue) ist ohne eine einzige
angepasste Erwartung grün — insbesondere die Billing- und
Carryover-Abschnitte.

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
