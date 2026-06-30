# Phase 34: Feiertags-Soll im Schichtplan - Context

**Gathered:** 2026-06-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Die bestehende Feiertags-Automatik aus Phase 25 (`build_derived_holiday_map`,
derive-on-read) wird in den **`get_week`-Report-Pfad** (`service_impl/src/reporting.rs:884`)
verdrahtet — als **vierter Injektionspunkt** analog zu den existierenden 1a/1b/1c.
Damit reduziert ein automatisch angerechneter Feiertag das **angezeigte Soll pro
Mitarbeiter** (`expected_hours` → durchgereicht als `WorkingHoursPerSalesPerson.available_hours`)
und erscheint als `holiday_hours` — **konsistent zum übrigen Stundenkonto**, in dem
die Automatik schon korrekt rechnet.

**Kernbefund:** Die Feiertags-Anrechnung ist im gesamten übrigen Report bereits drin;
in `get_week` fehlt sie versehentlich. Phase 34 ist also ein **gezielter Konsistenz-Fix
einer Auslassung**, keine neue Mechanik und keine zweite Berechnung.

**Strikt außerhalb:** Kapazitätsbänder (`paid_hours`/`dynamic_hours`/`committed_voluntary`/
`volunteer`) bleiben in derselben Woche unverändert (D-25-08-Grenze, HSP-03). Reine
Backend-Phase — keine FE-Änderung, keine neue UI-Spalte.

</domain>

<decisions>
## Implementation Decisions

### Welches „Soll" reduziert wird (HSP-01)
- **D-34-01:** Der Feiertag reduziert **ausschließlich** das **per-Mitarbeiter-Soll**
  über `get_week` (`reporting.rs:884`) — `expected_hours` (und damit
  `WorkingHoursPerSalesPerson.available_hours = report.expected_hours`,
  `booking_information.rs:322/331`). Die separat berechnete **per-Tag-Aggregat-Zeile
  Mo–So** (`monday_available_hours…`, `booking_information.rs:517`, Verteilung
  `details.expected_hours / working_days`) wird **NICHT** angefasst — sie ignoriert
  Feiertage weiterhin und grenzt an die Kapazitäts-/Band-Seite (HSP-03-Schutz).
  Begründung User: „Es ist schon im gesamten Report drin, aber hier [get_week] fehlt
  es bzw. wurde aus Versehen so umgesetzt." → minimaler, requirement-treuer Eingriff.

### Sichtbarkeit der Feiertagsstunden (HSP-02)
- **D-34-02:** **Backend-only.** HSP-02 gilt als erfüllt, sobald
  `WorkingHoursPerSalesPerson.holiday_hours` (= `report.holiday_hours`) korrekt
  gefüllt ist und das Soll sichtbar sinkt — **keine** neue FE-Spalte/Badge/Tooltip,
  keine i18n. Phase bleibt (BE). Das FE ist bereits auf ein nicht-null `holiday_hours`
  vorbereitet: `shifty-dioxus/src/state/weekly_overview.rs:52` rechnet
  `absence_hours − holiday_hours + unavailable_hours` → Feiertage werden bewusst aus
  der Absence-Anzeige herausgerechnet, kein FE-Regress. Ein sichtbarer Feiertags-
  Indikator bleibt deferred (siehe Deferred Ideas).

### HOL-03-Regressionstest (HSP-03 / HSP-04)
- **D-34-03:** `test_holiday_auto_credit_no_year_view_impact`
  (`service_impl/src/test/reporting_holiday_auto_credit.rs:545`) wird **in place
  umgebaut**. Der bisherige Test verbietet jeden Holiday-Aufruf in `get_week`
  (special_day/toggle-Mocks ohne Expectation → Panic) — das ist nach Phase 34 falsch.
  Neue Form:
  1. special_day-Mock liefert 1 Feiertag, toggle-Mock liefert aktiven Stichtag (≤ Feiertagsdatum).
  2. **Band-Guard behalten:** `dynamic_hours == 40.0` UNVERÄNDERT (HSP-03 — Bänder dürfen nicht sinken).
  3. **Neu positiv asserten:** `expected_hours == 32.0` (40 − 8 Feiertag) und `holiday_hours == 8.0` (HSP-01/HSP-02).
  - **Plus separater Test (HSP-04):** Feiertag **vor** dem Stichtag → keine Wirkung
    (`expected_hours == 40`, `holiday_hours == 0`); und manueller `ExtraHours(Holiday)`
    gewinnt → keine Doppelzählung (Wiederverwendung der `build_derived_holiday_map`-Logik,
    nicht neu implementieren).

### Snapshot-Schema-Version (HSNAP / billing_period)
- **D-34-04:** **KEIN Bump** — `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **12**.
  Begründung: `billing_period`-Snapshots speisen sich aus dem `reporting.rs`-
  `holiday_hours`-Pfad (`build_and_persist_billing_period_report` → `EmployeeReport.holiday_hours`),
  **nicht** aus `get_week`/`booking_information`. Phase 34 fasst ausschließlich `get_week`
  an → kein persistierter `BillingPeriodValueType` ändert seine Computation.
  **Plan-Task (Verifikation, Pflicht):** per grep/Lese-Check bestätigen, dass kein
  Snapshot-Writer aus `get_week` oder `booking_information` liest. Falls doch (unerwartet)
  → Bump 12→13 + Begründung. Default-Erwartung: bleibt 12.

### Claude's Discretion
- Exakte Stelle/Form des 4. Injektionspunkts in `get_week` (eigener `holiday_derived`-
  Term vs. Einbau in den bestehenden absence/expected-Reduktionspfad) — solange:
  (a) `expected_hours` korrekt um den derived-Holiday sinkt,
  (b) `holiday_hours` den derived-Beitrag enthält (additiv zu manuellem Holiday, manual-wins),
  (c) `dynamic_hours`/Bänder unberührt bleiben,
  (d) der dynamic-Week-Guard (`planned_hours <= 0.0 → 0`) konsistent zu den anderen
      Report-Pfaden angewandt wird (kein negatives expected).
- Genaue Fixture-/Test-Struktur (in-place erweitern vs. zusätzlicher Helper) im Rahmen von D-34-03.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` — HSP-01..04 (Z. 50–70), inkl. offener D-NN-Hinweis zur HOL-03-Neuformulierung; „Out of Scope" (Snapshot-Bump-Default, Z. 105+).
- `.planning/ROADMAP.md` §"Phase 34" (Z. 137–148) — Goal, Success Criteria, **Phase-Note (Snapshot & HOL-03)**.

### Vorgänger-Decisions (locked, NICHT neu entscheiden)
- `.planning/phases/25-feiertags-auto-anrechnung-stichtag-konfiguration/25-CONTEXT.md`
  — D-25-01 (derive-on-read, keine DB-Rows), D-25-02 (`EmployeeWorkDetails::holiday_hours()`),
  D-25-03 (manuell gewinnt), D-25-05 (`holiday_auto_credit`-Toggle + ISO-Datum-Stichtag-Gate),
  **D-25-08 (Bänder/Year-View-Kern unangetastet — die Grenze für HSP-03)**.

### Snapshot-Versioning-Regel
- `shifty-backend/CLAUDE.md` §"Billing Period Snapshot Schema Versioning" — wann ein Bump
  Pflicht ist (Begründung für D-34-04: get_week speist keinen persistierten value_type).

### Code — Implementierungspfad
- `service_impl/src/reporting.rs:884` — **`get_week`** (Ziel des 4. Injektionspunkts).
- `service_impl/src/reporting.rs:151` — **`build_derived_holiday_map`** (wiederzuverwendender Helper; Stichtag-Gate + manual-wins + jahresgrenzen-sichere ISO-Wochen).
- `service_impl/src/reporting.rs:361` / `:754` — existierende Aufrufstellen von `build_derived_holiday_map` (Vorbild 1a/1b).
- `service_impl/src/reporting.rs:402-406` — heutige `holiday_hours`-Summe (manueller Holiday); Vorbild-Injektionspunkt 1c.
- `service/src/reporting.rs:148` — **`ShortEmployeeReport`** (hat `expected_hours` + `holiday_hours`, KEIN separates `available_hours`).

### Code — Propagation (read-only-Verständnis, NICHT ändern)
- `service_impl/src/booking_information.rs:236` / `:396` — `week_report = get_week(...)` (beide `get_weekly_summary`-Pfade).
- `service_impl/src/booking_information.rs:317-334` — Bau von `WorkingHoursPerSalesPerson` (`available_hours = report.expected_hours`, `holiday_hours = report.holiday_hours`); `paid_hours = Σ report.dynamic_hours` (Band — muss konstant bleiben).
- `service_impl/src/booking_information.rs:517` — **per-Tag-Soll-Block** (NICHT anfassen, D-34-01).
- `service/src/booking_information.rs:25` — `WorkingHoursPerSalesPerson`-Struct.
- `shifty-dioxus/src/state/weekly_overview.rs:52` — FE `absence − holiday` (belegt: kein FE-Regress bei nicht-null holiday_hours).

### Test
- `service_impl/src/test/reporting_holiday_auto_credit.rs:545` — **`test_holiday_auto_credit_no_year_view_impact`** (umzubauen, D-34-03); HOL-01/02-Tests darüber als Fixture-/Assert-Vorbild.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `build_derived_holiday_map` (`reporting.rs:151`): liefert `HashMap<time::Date, f32>`
  (derived Feiertagsstunden pro Datum), inkl. Stichtag-Gate, manual-wins und
  jahresgrenzen-sicherer ISO-Wochen-Arithmetik. Wird in `get_week` wiederverwendet —
  identisch zu den bestehenden Aufrufstellen (`:361`, `:754`).
- `EmployeeWorkDetails::holiday_hours()` / `has_day_of_week()`: Anrechnungsbetrag pro
  Feiertag, bereits in Phase 25 etabliert.

### Established Patterns
- get_week aggregiert per Mitarbeiter: `expected_hours = planned_hours −
  abense_hours_for_balance − absence_derived_balance_total`. Der derived-Holiday muss
  symmetrisch (analog `absence_derived_*`) in die expected-Reduktion und in
  `holiday_hours` einfließen — mit demselben `planned_hours <= 0.0 → 0`-Guard
  (dynamic-Wochen), damit kein negatives expected/aufgeblähte Balance entsteht.
- `dynamic_hours` wird in get_week separat geführt und darf vom Feiertag **nicht**
  berührt werden (Band-Guard, HSP-03).

### Integration Points
- Einziger Schreib-Eingriff: `get_week`. Propagation nach `WorkingHoursPerSalesPerson`
  (available_hours/holiday_hours) erfolgt automatisch, weil booking_information das
  get_week-Ergebnis durchreicht. Keine weiteren Stellen anzufassen.

</code_context>

<specifics>
## Specific Ideas

- HSP-01 wörtlich („pro Mitarbeiter", „get_week", „vierter Injektionspunkt analog 1a/1b/1c")
  ist die maßgebliche Vorgabe und deckt sich exakt mit D-34-01.
- Erwartetes Test-Beispiel (40h Mon–Fri-Vertrag, 1 Feiertag, Stichtag aktiv):
  `expected_hours 40 → 32`, `holiday_hours 0 → 8`, `dynamic_hours == 40` (unverändert).

</specifics>

<deferred>
## Deferred Ideas

- **Sichtbarer Feiertags-Indikator/Spalte/Tooltip in der Schichtplan-Tabelle** — bewusst
  nicht Teil von Phase 34 (D-34-02 = BE-only). Deckt sich mit dem bereits in
  `REQUIREMENTS.md` §"Future Requirements" geführten „Hover-Tooltip auf Feiertags-Zelle"
  (rein additiv). Future-Phase.
- **Feiertags-Berücksichtigung der per-Tag-Aggregat-Zeile (Mo–So)** unter dem Schichtplan —
  nicht Teil dieser Phase (D-34-01); berührt die Kapazitäts-/Band-Seite und wäre eine
  eigene Entscheidung gegen HSP-03.

### Reviewed Todos (not folded)
None — keine offenen Todos mit Phase-34-Scope.

</deferred>

---

*Phase: 34-feiertags-soll-schichtplan*
*Context gathered: 2026-06-30*
