# Phase 64: Planungs-Soll für dynamische Verträge - Research

**Researched:** 2026-09-29
**Domain:** Backend-Reporting-Aggregation (`service_impl/src/reporting.rs`), DTO-Durchreichung, Dioxus-Frontend-Anzeige
**Confidence:** HIGH (alles Code-verifiziert, kein externes Ökosystem-Risiko)

## Summary

Die Phase fügt ein neues, rein additives Feld `planning_target_hours` an
`ShortEmployeeReport`/`ShortEmployeeReportTO` hinzu — das feiertags-/
abwesenheitsreduzierte Vertrags-Soll für **alle** Vertragstypen, auch
dynamische, ohne die bilanzrelevanten Felder `expected_hours` und
`dynamic_hours` anzurühren. Es gibt **keine** externe Abhängigkeit, **keine**
Migration und **keinen** Snapshot-Bump — reines Innen-Refactoring zweier
bereits vorhandener Aggregations-Schleifen plus eine Durchreichung bis ins
Frontend.

Der naheliegende Fix (`dynamic_hours` als Anzeige-Target nehmen) ist bereits
im Code selbst als falsch dokumentiert (Kommentar in
`working_hours_mini_overview.rs:108-112`): `dynamic_hours` durchläuft die
Feiertags-Reduktion (HSP-03-Band-Guard) nicht, `expected_hours` schon. Ein
Wechsel auf `dynamic_hours` würde für **alle** Verträge (auch die heute
korrekten festen) automatisch angerechnete Feiertage aus dem angezeigten Soll
verschwinden lassen — eine Regression, kein Fix.

Die eigentliche Falle liegt tiefer, als der Ursprungs-Todo
(`.planning/todos/pending/2026-09-15-sollstunden-fehlen-bei-dynamischen-vertraegen-unter-schichtplan.md`)
sie beschreibt: Die Feiertags-/Abwesenheits-Reduktionsterme sind in **beiden**
Konstruktionsstellen an eine Gate-Bedingung gekoppelt, die für dynamische
Verträge **immer** wahr ist (`planned_hours <= 0.0` bzw. `expected_hours <=
0.0` — beides bereits auf 0 gezwungene Werte). Ein naives Wiederverwenden
dieser Terme für `planning_target_hours` würde die Reduktion für dynamische
Verträge komplett überspringen und nur den unreduzierten Rohwert liefern —
das verletzt Success Criterion 3 (Feiertage/Abwesenheiten müssen auch im
Planungs-Soll dynamischer Verträge wirken). Abschnitt „Kern-Mechanismus"
unten zeigt exakt, wo diese Kopplung sitzt und wie eine parallele,
korrekt-gegatete Berechnung aussehen müsste.

**Primary recommendation:** Neues Feld `planning_target_hours: f32` an
`service::reporting::ShortEmployeeReport` und `rest_types::ShortEmployeeReportTO`
(additiv, `#[serde(default)]`), berechnet an **beiden** Konstruktionsstellen
(`assemble_weeks` und `get_reports_for_all_employees`) über eine **eigene**
Kopie der Feiertags-/Abwesenheits-Reduktionspipeline, die auf dem
un-gezwungenen Rohwert (`weight_for_week`-Tupel-Element 1, bereits heute als
Zwischenwert vorhanden) gated wird statt auf dem für dynamische Verträge
immer-null gezwungenen Wert — plus ein Cross-Site-Paritätstest nach dem
Muster von `reporting_additive_merge.rs`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Planungs-Soll-Berechnung (Feiertags-/Abwesenheits-Reduktion) | API / Backend (`ReportingServiceImpl`, Business-Logic-Tier) | — | Bereits vorhandene Aggregationslogik in `service_impl/src/reporting.rs`; „Fat Backend" ist hier bereits die geltende Konvention (siehe Kommentar in `working_hours_mini_overview.rs` und PADM-06-Präzedenz) |
| DTO-Transport | API / Backend (`rest-types`) | — | `ShortEmployeeReportTO::from(&ShortEmployeeReport)`, additiv wie beim Phase-55-Präzedenz-Feld `has_pending_rebooking` |
| Anzeige (Karten/Tabellen-Layout, Fortschrittsbalken, Farbschwellen) | Frontend Server / Client (Dioxus SSR + WASM) | — | `shifty-dioxus/src/component/working_hours_mini_overview.rs` liest das Feld nur, berechnet nichts (PSOLL-02: „keine Fallback-Regel im Frontend") |
| Datenfluss Ladepfad | Frontend Server / Client (Dioxus) | — | `loader.rs::build_working_hours_mini` reicht Felder 1:1 durch, keine Ableitung |

Keine Datenbank-/Storage-Tier-Beteiligung — reines In-Memory-Aggregat über
bereits geladene `EmployeeWorkDetails`/`ExtraHours`/`AbsencePeriod`-Daten.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PSOLL-01 | Übersicht unterhalb des Schichtplans zeigt bei dynamischen Verträgen ein Sollstunden-Ziel > 0 statt 0 | Kern-Mechanismus-Abschnitt zeigt exakt, warum heute 0 zurückkommt (`weight_for_week`-Zeroing) und welche Reduktionspipeline für das neue Feld kopiert werden muss |
| PSOLL-02 | Planungs-Soll kommt fertig aus dem Backend, keine Frontend-Fallback-Regel | Bestehender Kommentar in `working_hours_mini_overview.rs` demonstriert das Fat-Backend-Muster bereits (Feld wird nur gelesen, `target = working_hour.expected_hours`) — neues Feld folgt demselben Muster 1:1 |
| PSOLL-03 | Planungs-Soll in BEIDEN Report-Pfaden gefüllt, gegeneinander getestet | „Zwei-Konstruktionsstellen"-Abschnitt lokalisiert beide Stellen mit exakten Zeilenbereichen; „Test-Infrastruktur"-Abschnitt liefert ein direkt wiederverwendbares Cross-Site-Paritätsmuster (`reporting_additive_merge.rs`) |
| PSOLL-04 | Stundenkonto-Report liefert unveränderte Balance-Werte, `available_hours` bleibt genullt | `booking_information.rs:644`/`:818` exakt lokalisiert und zitiert; `billing_period_report.rs` liest nachweislich nicht von `ShortEmployeeReport`, sondern von `EmployeeReport` (anderer Typ) — Snapshot-Version-Unberührtheit ist damit strukturell, nicht nur behauptet |
</phase_requirements>

## Kern-Mechanismus: Warum `expected_hours` heute 0 ist

**Ursprung:** `weight_for_week` in `service_impl/src/reporting.rs:2137-2193`.
Es gibt ein 4-Tupel zurück:

```rust
// service_impl/src/reporting.rs:2185-2192
(
    if employee_work_details.is_dynamic { 0.0 } else {
        employee_work_details.expected_hours * relation
    },                                               // Element 0 -> "planned_hours"/"expected_hours"
    employee_work_details.expected_hours * relation, // Element 1 -> "dynamic_hours" (IMMER unreduziert, IMMER gefüllt)
    num_potential_workdays_in_week as u8,
    employee_work_details.workdays_per_week as f32 * relation,
)
```

Element 0 wird für `is_dynamic == true` **hart auf 0.0 gezwungen** — Element 1
(`dynamic_hours`) trägt denselben Rohwert **unabhängig** von `is_dynamic`.
Für nicht-dynamische Verträge sind beide Elemente identisch.

**Beide Konstruktionsstellen konsumieren dieses Tupel identisch benannt:**

- `assemble_weeks` (`service_impl/src/reporting.rs:996-1000`):
  ```rust
  let (planned_hours, dynamic_hours): (f32, f32) =
      find_working_hours_for_calendar_week(working_hours, year, week)
          .map(|wh| weight_for_week(year, week, wh))
          .map(|wfw| (wfw.0, wfw.1))
          .fold((0.0, 0.0), |(acc_a, acc_b), (a, b)| (acc_a + a, acc_b + b));
  ```
- `get_reports_for_all_employees` (`:1318-1332`): identisches Muster, lokale
  Variablen heißen dort `(expected_hours, dynamic_hours)`.

**Die eigentliche Falle — Reduktionsterme sind an den gezwungenen Wert
gegatet, nicht an „gibt es überhaupt einen Vertrag":**

In `assemble_weeks` (`:1045-1084`):

```rust
let abense_hours_for_balance = if !has_contract_row || planned_hours <= 0.0 {
    0.0f32
} else {
    abense_hours
};
let absence_derived_balance_total = if !has_contract_row || planned_hours <= 0.0 {
    0.0f32
} else {
    absence_derived_vacation_hours + absence_derived_sick_leave_hours + absence_derived_unpaid_leave_hours
};
// ...
let holiday_derived_gated = if !has_contract_row || planned_hours <= 0.0 {
    0.0f32
} else {
    derived_holiday_for_week
};
let expected_hours_for_cap = planned_hours - abense_hours_for_balance - absence_derived_balance_total;
// ...
let expected_hours = expected_hours_for_cap - holiday_derived_gated;
```

`planned_hours` ist für dynamische Verträge **immer** 0 (Element 0 des
Tupels). Die Bedingung `planned_hours <= 0.0` ist für **jede** Woche eines
dynamischen Vertrags wahr — unabhängig davon, ob diese Woche tatsächlich
Urlaub, Krankheit oder einen Feiertag enthält. Konsequenz: `expected_hours`
wird für dynamische Verträge nicht nur „nicht reduziert", sondern der ganze
Reduktionspfad wird **übersprungen** und landet exakt bei `0 - 0 - 0 - 0 =
0`.

In `get_reports_for_all_employees` ist die Kopplung strukturell identisch,
nur anders verdrahtet: der `else if expected_hours <= 0.0`-Zweig
(`:1411-1456`, „Dynamischer Vertrag … Soll=Ist") berechnet **gar keine**
`derived_holiday_for_week`/Abwesenheits-Reduktion für die Woche — diese
Berechnung existiert nur im `else`-Zweig (`:1457-1543`, der NUR bei
nicht-dynamischen bzw. echten positiven Wochen läuft). Die Felder
`absense_hours`/`holiday_hours` werden im dynamischen Zweig hart auf `0.0`
gesetzt (`:1444`, `:1449`).

**Was das für `planning_target_hours` bedeutet:** Ein einfaches
"`dynamic_hours` (Element 1, immer gefüllt) minus dieselben, bereits
vorhandenen Reduktionsterme" **funktioniert nicht**, weil diese
Reduktionsterme für dynamische Wochen selbst auf 0 gezwungen sind (das ist ja
der ganze Bug). Die Phase muss stattdessen an **beiden** Stellen eine
**parallele** Berechnung einführen, die:

1. als Basis Element 1 des Tupels nimmt (`dynamic_hours` in `assemble_weeks`
   vor dem späteren Shadowing in Zeile 1100-1101; `dynamic_hours` in
   `get_reports_for_all_employees`, dort bereits unverändert vorhanden),
2. die Feiertags-/Abwesenheits-Reduktion **unabhängig davon** anwendet, ob
   der Vertrag dynamisch ist — das Gate darf nur noch auf
   `!has_contract_row` (bzw. „gibt es diese Woche überhaupt eine
   Vertragszeile") geprüft werden, nicht mehr auf `planned_hours <= 0.0` /
   `expected_hours <= 0.0`,
3. in `get_reports_for_all_employees` zusätzlich die
   `derived_holiday_for_week`- und `week_extra_hours`-Berechnung (heute nur
   im `else`-Zweig vorhanden, `:1483-1499`) auch für den dynamischen Zweig
   verfügbar machen muss, um die Reduktion dort überhaupt füttern zu können.

**Byte-Identität für nicht-dynamische Fixtures (PSOLL-Erfolgskriterium 4)
folgt automatisch aus dieser Konstruktion:** Für `is_dynamic == false` sind
Element 0 und Element 1 des `weight_for_week`-Tupels identisch, also ist
`planned_hours == dynamic_hours` (der Rohwert), das neue Gate
(`!has_contract_row`, ohne `<= 0.0`-Zusatz) verhält sich für positive
`planned_hours` identisch zum alten Gate, und `planning_target_hours` fällt
exakt auf `expected_hours` zurück — ohne dass die beiden Formeln explizit
aufeinander verweisen müssen. Dieser Zusammenhang sollte als eigener,
expliziter Test verifiziert werden (nicht nur als Nebenprodukt angenommen).

`weight_for_week` selbst muss dafür **nicht** verändert werden — sein
Rückgabetupel liefert die nötige Rohgröße (Element 1) bereits heute.

## Zwei unabhängige `ShortEmployeeReport`-Konstruktionsstellen

Bestätigt exakt wie in ROADMAP.md behauptet — **es gibt nur diese zwei**,
kein drittes verstecktes Vorkommen:

1. **`assemble_weeks`** (`service_impl/src/reporting.rs:769-1154`, Push bei
   `:1126-1147`). Wird von **beiden** `get_week` (`:1900-1979`, ein einzelnes
   `(year, week)`-Element) und `get_year` (`:1981ff`, alle ISO-Wochen des
   Jahres) aufgerufen — `get_year` ist also **kein** drittes
   Konstruktions-Standbein, sondern derselbe Helfer mit mehr Wochen (Kommentar
   im Code bestätigt das explizit: „Phase 52 … Batch-Variante von `get_week`
   … Byte-Identität … strukturell garantiert"). D-64-02 (soll `get_year` das
   Feld auch führen) ist damit **kostenlos** — es fällt automatisch mit an,
   sobald `assemble_weeks` das Feld füllt. Es bleibt nur die
   Frontend-Konsumfrage offen, ob die Jahresansicht die Spalte überhaupt
   rendert (heute laut ROADMAP: nein).

2. **`get_reports_for_all_employees`** (`:1164-1668`, Push bei
   `:1650-1665`). Eigenständige, komplett unabhängig geschriebene
   Implementierung — andere Variablennamen (`WeeklyHours`-Struct statt
   direkter lokaler `let`s), andere Datenquellen-Ladereihenfolge
   (`self.absence_service.derive_hours_for_range` einmal für den ganzen
   Range statt `derive_hours_for_week_pure` pro Woche in-memory), aber
   dieselbe fachliche Semantik.

**Bereits vorhandenes Präzedenzbeispiel für „ein Feld ist in einer Stelle
gefüllt, in der anderen bewusst neutral"** — zur Einordnung, dass PSOLL-03
explizit das Gegenteil verlangt: `has_pending_rebooking`/
`pending_rebooking_id` (Phase 55) werden in `assemble_weeks` **hart auf**
`false`/`None` gesetzt (Kommentar `:1139-1144`: „der HR-Alert-Banner wird
über den short-report `get_reports_for_all_employees` getrieben … Hier
bleiben die Felder daher neutral"), während `get_reports_for_all_employees`
sie tatsächlich berechnet (`:1602-1648`). Das ist dort eine **bewusste**
Design-Entscheidung (D-55-B) — genau die Art von stillem Auseinanderlaufen,
die für `planning_target_hours` laut PSOLL-03 **nicht** passieren darf. Der
Planner sollte diesen Unterschied nicht als Vorlage kopieren.

**Verbindung zum „Phase-57-Bug" (ROADMAP-Zitat):** Der tatsächliche
Phase-57-Fund (`f14-rebooking-filter-blockiert-feature`-Memory) betraf einen
`source == Manual`-Filter, der an **vier** unabhängigen Report-Aggregat-
Stellen dieselbe Korrektur brauchte — drei wurden gefixt, eine bewusst
anders belassen. Die übertragbare Lehre ist nicht der konkrete Filter,
sondern die **Form der Falle**: mehrere strukturell parallele, aber
unabhängig geschriebene Aggregations-Codepfade für dasselbe fachliche
Konzept laufen ohne gemeinsamen Test lautlos auseinander. PSOLL-03 fordert
deshalb explizit einen Test, der **beide** Stellen gegeneinander hält (siehe
„Test-Infrastruktur" unten) — nicht zwei isolierte Tests, die je für sich
grün werden können, obwohl sie unterschiedliche Werte liefern.

## `available_hours` — Regressions-Lock (PSOLL-04)

Beide vom ROADMAP zitierten Zeilen exakt bestätigt, identischer Code an
beiden Stellen:

```rust
// service_impl/src/booking_information.rs:644
available_hours: report.expected_hours,
// service_impl/src/booking_information.rs:818
available_hours: report.expected_hours,
```

`report` ist hier ein `&ShortEmployeeReport`, `available_hours` fließt in
`WorkingHoursPerSalesPerson` (`service/src/booking_information.rs:25-35`) —
das DTO hinter der Stundenkonto-/Wochenübersicht-Verfügbarkeitsberechnung.
Beide Stellen lesen **explizit `expected_hours`**, nicht `dynamic_hours` und
nicht das neue Feld. Solange die Implementierung `expected_hours` selbst
nicht anfasst (nur ein zusätzliches Feld ergänzt), bleiben diese zwei Zeilen
automatisch unverändert — das Regressions-Risiko liegt einzig darin, dass
jemand versehentlich `expected_hours` durch `planning_target_hours` ersetzt,
weil beide „irgendwie das Soll" bedeuten. Ein Test, der `available_hours`
für eine dynamische Fixture weiterhin bei `0.0` erwartet (oder allgemeiner:
unverändert gegenüber dem Pre-Phase-Stand), gehört als expliziter
Regressions-Test in die Testsuite dieser Phase.

**Snapshot-Version-Unberührtheit ist strukturell, nicht nur behauptet:**
`service_impl/src/billing_period_report.rs` (der Snapshot-Writer, der
`CURRENT_SNAPSHOT_SCHEMA_VERSION` schreibt) liest ausschließlich Felder von
`service::reporting::EmployeeReport` (`report_delta.expected_hours`,
`report_start.expected_hours` usw., Zeilen 234-237) — **nicht** von
`ShortEmployeeReport`. `EmployeeReport` wird von
`get_report_for_employee`/`get_report_for_employee_range` erzeugt, einem
**dritten**, von dieser Phase überhaupt nicht berührten Codepfad. Der
Snapshot-Schreiber kann das neue Feld also gar nicht sehen — Version bleibt
**13** ohne zusätzliche Prüfung nötig, nur eine grep-Verifikation im finalen
Plan empfehlenswert (`grep -n "planning_target_hours" service_impl/src/billing_period_report.rs`
muss leer bleiben).

## DTO-/Datenfluss-Kette (alle Stationen, exakt lokalisiert)

```
service::reporting::ShortEmployeeReport          (service/src/reporting.rs:148-171)
        │  neues Feld: planning_target_hours: f32
        ▼
rest_types::ShortEmployeeReportTO                 (rest-types/src/lib.rs:381-413)
        │  neues Feld + #[serde(default)] (additiv, Präzedenz: has_pending_rebooking)
        │  From<&ShortEmployeeReport> Impl aktualisieren
        ▼
GET /report/week/{year}/{calendar_week}           (rest/src/report.rs:145-178)
GET /report?year&until_week                        (rest/src/report.rs:64-92, "get_short_report_for_all")
        │  keine neue Route nötig — bestehende Endpoints liefern das erweiterte DTO automatisch mit
        ▼
shifty-dioxus::api::get_working_hours_for_week /
       get_balance_until_week                      (shifty-dioxus/src/api.rs:850, :858)
        ▼
shifty-dioxus::loader::build_working_hours_mini     (shifty-dioxus/src/loader.rs:434-460)
        │  Feld 1:1 durchreichen (Fat-Backend-Konvention, PADM-06-Präzedenz)
        ▼
shifty-dioxus::state::employee_work_details::WorkingHoursMini  (shifty-dioxus/src/state/employee_work_details.rs:8-30)
        │  neues Feld + Default::default() ergänzen
        ▼
shifty-dioxus::component::working_hours_mini_overview.rs
        │  CardsLayout (Zeile ~113): let target = working_hour.expected_hours;
        │  TableLayout (Zeilen ~171, ~216): dito, plus total_target-Summe
        │  D-64-01: JEDE dieser vier Stellen einzeln auf planning_target_hours
        │           umstellen oder bei expected_hours belassen — keine
        │           pauschale Suchen-und-Ersetzen-Aktion (siehe unten)
```

**Keine neue REST-Routen-Familie** — der in ROADMAP-Trap #6 beschriebene
`Dioxus.toml`-Proxy-Vergessen-Bug (dreimal in diesem Repo passiert) **greift
hier nicht**: `/report/week/...` und `/report` existieren bereits und sind im
Dev-Proxy längst eingetragen. Kein neuer Proxy-Eintrag nötig.

### D-64-01 — welche Konsumenten umstellen (unbeantwortete Design-Frage)

`working_hours_mini_overview.rs` hat **vier** Stellen, die heute
`working_hour.expected_hours` als „Target" lesen:

| Stelle | Zeile (aktuell) | Funktion |
|---|---|---|
| `CardsLayout` — `target` | `:113` | speist `progress_bar_class`, `hours_text_class`, `progress_bar_percent`, Anzeige-String |
| `TableLayout` — `total_target` (Summenzeile) | `:171` | Fußzeilen-Summe über alle Personen |
| `TableLayout` — `target` (pro Zeile) | `:216` | dito wie CardsLayout, pro Tabellenzeile |

Da keine discuss-phase stattfand, ist **nicht entschieden**, ob alle vier
Stellen umgestellt werden oder nur ein Teil. Der Origin-Todo nennt das
explizit als „bewusste Entscheidung pro Stelle, kein pauschales
Suchen-und-Ersetzen" — plausibelster Default (aber nicht gepinnt): alle vier
auf `planning_target_hours` umstellen, weil das exakt der User-Wunsch aus
dem Todo ist („für die Planung… die Sollstunden") und weil ein gemischter
Zustand (manche Stellen zeigen das alte 0-Soll, andere das neue) verwirrender
wäre als eine konsistente Umstellung. Der Planner sollte diese Entscheidung
explizit im Plan festhalten, nicht stillschweigend treffen.

### D-64-03 — visueller Hinweis auf reines Planungsziel (unbeantwortete Design-Frage)

Kein bestehender i18n-Key für „Planungsziel"/„Planning target" gefunden
(`grep -n "PlanningTarget\|Planungsziel" shifty-dioxus/src/i18n/mod.rs` leer).
Falls der Plan einen visuellen Hinweis vorsieht (z. B. Tooltip oder Icon,
dass diese Zahl die Balance nicht beeinflusst), braucht das einen neuen
i18n-Key in **allen drei** Locales (de/en/cs) — Projekt-Konvention laut
`shifty-dioxus/CLAUDE.md`. Das ist reine Zusatzarbeit, keine Voraussetzung
für PSOLL-01..04.

## Test-Infrastruktur

**Test-Framework:** `cargo test` (Rust-Standard) + `mockall` für
Service-Mocks. Kein separates Test-Runner-Setup nötig, alles bereits
vorhanden. Backend-Test-Kommando aus `CLAUDE.md`: `cargo test --workspace`
plus Pflicht-Gate `cargo clippy --workspace -- -D warnings`. Frontend-Tests
(nur falls D-64-01/D-64-03-Anzeigeteile geändert werden):
`cargo test --manifest-path shifty-dioxus/Cargo.toml` (Baseline 858 bestanden)
und eigenes Clippy-Gate.

**Wiederverwendbare Fixtures** — bereits exakt für diesen Zweck gebaut:
`service_impl/src/test/reporting_phase2_fixtures.rs` liefert ein
Spiegel-Paar:

```rust
pub fn fixture_work_details_8h_mon_fri() -> EmployeeWorkDetails { /* is_dynamic: false, 40h, Mo-Fr, KW22-25/2024 */ }
pub fn fixture_work_details_dynamic_mon_fri() -> EmployeeWorkDetails {
    EmployeeWorkDetails { id: ..., is_dynamic: true, ..fixture_work_details_8h_mon_fri() }
}
```

Kommentar an der dynamischen Fixture (`:75-77`) dokumentiert bereits die
heutige Zero-Semantik — nach dieser Phase muss dieser Kommentar aktualisiert
werden, da `planned_hours`/`expected_hours` weiterhin 0 bleiben (Balance
unverändert), aber `planning_target_hours` jetzt > 0 wird.

**Cross-Site-Paritätsmuster bereits im Repo vorhanden** —
`service_impl/src/test/reporting_additive_merge.rs` enthält exakt das
Muster, das PSOLL-03 braucht: dieselbe Fixture zweimal durch **beide**
Entry-Points laufen lassen (`get_reports_for_all_employees` und `get_week`)
und die Ergebnisfelder gegeneinander assertieren
(`test_balance_parity_dynamic_all_employees` +
`test_balance_parity_dynamic_get_week`, Zeilen 962-1241). Dieses Muster
sollte für `planning_target_hours` direkt kopiert werden:

1. Ein Test, der `get_week(2024, 23, ...)` und
   `get_reports_for_all_employees(2024, 23, ...)` mit **identischer**
   Fixture (`fixture_work_details_dynamic_mon_fri`, gleiche Abwesenheits-/
   Feiertagsdaten) aufruft und `planning_target_hours` beider Resultate auf
   Gleichheit prüft (analog zu `report_a`/`report_b` im Muster oben).
2. Ein zweiter Test mit `fixture_work_details_8h_mon_fri` (nicht-dynamisch),
   der `planning_target_hours == expected_hours` (Byte-Identität, PSOLL
   Erfolgskriterium 4) für beide Entry-Points prüft.
3. Ein Regressions-Test, der `available_hours` (über
   `booking_information.rs`) für die dynamische Fixture weiterhin bei `0.0`
   hält (PSOLL-04).

Die `ReportingMocks`/`TestDeps`-Boilerplate aus `reporting_additive_merge.rs`
(Zeilen 100-176) ist 1:1 wiederverwendbar — beide neuen Tests können in
derselben Datei oder einer neuen `reporting_planning_target.rs` liegen; da
`tdd_mode: true` in `.planning/config.json` gesetzt ist, sollten diese Tests
RED vor der Implementierung geschrieben werden (Muster: Datei-Kopfkommentare
in `reporting_additive_merge.rs` markieren das explizit: „RED vor Task X,
GREEN nach Task X").

**Inline-Unit-Test-Präzedenz für `weight_for_week` selbst:**
`service_impl/src/reporting.rs:2486-2520` (`mod test_dynamic_vacation_days`)
zeigt das Muster für einen lokalen `create_work_details(is_dynamic: bool)`-
Helfer direkt im Produktionsmodul — falls `weight_for_week` einen neuen
Rückgabewert bekommt (nicht empfohlen, siehe Kern-Mechanismus-Abschnitt —
Element 1 reicht als Basis), wäre das der Ort für Unit-Tests der reinen
Funktion.

### Validation Architecture

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust `#[tokio::test]`/`#[test]`) + `mockall` |
| Config file | keine separate Config — Workspace-Cargo.toml |
| Quick run command | `cargo test -p service_impl reporting` |
| Full suite command | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PSOLL-01 | Dynamischer Vertrag liefert `planning_target_hours > 0` in `get_week` | unit | `cargo test -p service_impl test_planning_target_dynamic_get_week` | ❌ Wave 0 |
| PSOLL-02 | Kein Fallback im Frontend (grep-Assertion) | statisch | `grep -n "planning_target_hours\|expected_hours\|dynamic_hours" shifty-dioxus/src/component/working_hours_mini_overview.rs` manuell geprüft, kein Berechnungsausdruck | n/a (Review, kein automatisierter Test sinnvoll) |
| PSOLL-03 | Beide Entry-Points liefern denselben Wert für dieselbe Person/Woche | unit | `cargo test -p service_impl test_planning_target_parity_both_sites` | ❌ Wave 0 |
| PSOLL-04 | `available_hours`/Balance unverändert für dynamische Fixture | unit | `cargo test -p service_impl test_available_hours_unchanged_dynamic` | ❌ Wave 0 (kann auf `reporting_additive_merge.rs`-Fixtures aufbauen) |
| PSOLL-04 (Byte-Identität nicht-dynamisch) | `planning_target_hours == expected_hours` für bestehende Fixtures | unit | `cargo test -p service_impl test_planning_target_matches_expected_for_fixed_contract` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p service_impl reporting` (+ `cargo test --manifest-path shifty-dioxus/Cargo.toml` falls FE-Dateien geändert)
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Phase gate:** Volle Suite grün vor `/gsd-verify-work`, inkl. FE-WASM-Build falls D-64-01/D-64-03 umgesetzt werden

### Wave 0 Gaps
- [ ] Neue Testdatei (z. B. `service_impl/src/test/reporting_planning_target.rs`) — deckt PSOLL-01/03/04, registriert in `service_impl/src/test/mod.rs`
- [ ] Kein neuer Fixture-Bedarf — `reporting_phase2_fixtures.rs` deckt dynamisch + fix bereits ab
- [ ] Falls D-64-01 umgesetzt wird: Anpassung der bestehenden `working_hours_mini_overview.rs`-Tests (`make_row`-Helfer, `WorkingHoursMini`-Literale) um `planning_target_hours`-Feld — sonst Compile-Fehler durch fehlendes Struct-Feld

## Common Pitfalls

### Pitfall 1: Reduktionsterme naiv wiederverwenden
**What goes wrong:** `planning_target_hours` wird als `dynamic_hours_roh -
abense_hours_for_balance - absence_derived_balance_total -
holiday_derived_gated` berechnet, indem die **bestehenden** (bereits
gegateten) Variablen aus dem `expected_hours`-Pfad wiederverwendet werden.
**Why it happens:** Die Variablennamen legen nahe, dass sie „die Reduktion"
sind — dass sie zusätzlich auf `planned_hours <= 0.0` gegatet sind, ist nicht
offensichtlich, ohne den Gate-Ausdruck genau zu lesen.
**How to avoid:** Reduktionsterme für `planning_target_hours` mit einem
eigenen Gate (`!has_contract_row`, ohne `<= 0.0`-Zusatz) parallel berechnen —
siehe Kern-Mechanismus-Abschnitt.
**Warning signs:** Test `test_planning_target_dynamic_get_week` mit
Urlaub/Feiertag in der Testwoche liefert denselben (unreduzierten) Wert wie
ohne Urlaub/Feiertag → Reduktion greift nicht.

### Pitfall 2: `get_reports_for_all_employees`-Dynamik-Zweig fehlt die Holiday-Berechnung komplett
**What goes wrong:** Der `else if expected_hours <= 0.0`-Zweig
(`:1411-1456`) hat heute **keinen** Zugriff auf `derived_holiday_for_week`/
`week_extra_hours` — diese werden nur im `else`-Zweig berechnet
(`:1457-1543`). Ein naiver Fix könnte versuchen, `holiday_hours: 0.0` einfach
durch eine im dynamischen Zweig nicht vorhandene Variable zu ersetzen und
bricht die Kompilierung, oder schlimmer: kopiert die Berechnung nur teilweise
und produziert stille Fehlwerte.
**Why it happens:** Die beiden `if`/`else if`/`else`-Zweige wurden historisch
für unterschiedliche fachliche Fälle geschrieben (kein Vertrag /
dynamischer Vertrag / normaler Vertrag) und teilen keine gemeinsame
Hilfsfunktion.
**How to avoid:** Die Holiday-/Absence-Berechnung (`derived_holiday_for_week`,
`week_extra_hours`-Filterung) als eigene lokale Berechnung **vor** der
Drei-Wege-Verzweigung hochziehen, sodass sie in allen Zweigen verfügbar ist
— ODER als kleine private Hilfsfunktion extrahieren, die aus beiden Zweigen
aufgerufen wird.
**Warning signs:** `cargo build` schlägt fehl mit „cannot find value … in
this scope" beim Versuch, die Reduktion im dynamischen Zweig zu berechnen.

### Pitfall 3: `expected_hours` versehentlich durch `planning_target_hours` ersetzen
**What goes wrong:** Beim Patchen von `booking_information.rs:644`/`:818`
wird `report.expected_hours` versehentlich zu `report.planning_target_hours`
geändert, weil beide „das Soll" bedeuten.
**Why it happens:** Namensähnlichkeit, und `available_hours` klingt so, als
sollte es das „richtige" (nicht genullte) Soll tragen.
**How to avoid:** Diese zwei Zeilen bewusst **nicht anfassen** — expliziter
Kommentar-Anker im Code (ROADMAP zitiert sie bereits als v2.5-Regression-
Lock). Ein Regressions-Test, der `available_hours == 0.0` für eine
dynamische Fixture erwartet, fängt eine versehentliche Änderung sofort ab.
**Warning signs:** `test_available_hours_unchanged_dynamic` schlägt fehl,
`CURRENT_SNAPSHOT_SCHEMA_VERSION`-Grep in `billing_period_report.rs` bleibt
zwar weiterhin leer (anderer Typ), aber Live-Balance-Werte verschieben sich
in Browser-Smokes.

### Pitfall 4: `has_pending_rebooking`-Präzedenz als Vorbild missverstehen
**What goes wrong:** Der Planner sieht, dass `has_pending_rebooking` in
`assemble_weeks` bewusst neutral bleibt (`false`/`None`), und überträgt
dieses Muster fälschlich auf `planning_target_hours` („eine Stelle reicht,
die andere bleibt 0").
**Why it happens:** Es ist das einzige bestehende Beispiel für „ein Feld ist
nur an einer Konstruktionsstelle sinnvoll berechnet" im selben Struct.
**How to avoid:** PSOLL-03 explizit lesen — das Requirement fordert
ausdrücklich Gleichheit an **beiden** Stellen, anders als beim
Rebooking-Feld. Die beiden Fälle sind fachlich verschieden: der Rebooking-
Banner ist bewusst an den Jahres-/Gesamt-Kontext gebunden, das Planungs-Soll
ist pro-Woche-Kontext-unabhängig sinnvoll.

## Code Examples

### Aktuelle Anzeige-Stelle (zu ändern gemäß D-64-01)
```rust
// Source: shifty-dioxus/src/component/working_hours_mini_overview.rs:104-113
let sales_person_id = working_hour.sales_person_id;
let actual = working_hour.actual_hours;
// TARGET = expected_hours (das feiertags-/absence-reduzierte Soll), NICHT
// dynamic_hours: der Band-Guard (HSP-03) hält dynamic_hours frei vom automatisch
// angerechneten Feiertag, sodass ein derived Holiday sonst nie im angezeigten
// Soll erschiene. expected_hours reflektiert manuelle UND automatische Feiertage
// gleich — konsistent zum Stundenkonto/Report.
let target = working_hour.expected_hours;
```

### Cross-Site-Paritätsmuster (zu kopieren für PSOLL-03)
```rust
// Source: service_impl/src/test/reporting_additive_merge.rs:962-1090
// (test_balance_parity_dynamic_all_employees / test_balance_parity_dynamic_get_week)
// Muster: identische Fixture (fixture_work_details_dynamic_mon_fri) durch
// get_reports_for_all_employees UND get_week laufen lassen, Ergebnisfelder
// mit (a - b).abs() < 0.01 gegeneinander assertieren.
```

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | D-64-01 Default-Empfehlung: alle vier Anzeige-Konsumenten in `working_hours_mini_overview.rs` auf `planning_target_hours` umstellen | D-64-01 | Falls User nur Teilumstellung will, muss der Plan angepasst werden — geringes Risiko, da rein UI-seitig und leicht revidierbar |
| A2 | Das empfohlene neue Gate (`!has_contract_row` statt `!has_contract_row \|\| planned_hours <= 0.0`) ist die korrekte Reparatur-Richtung für die Reduktions-Kopplung | Kern-Mechanismus | Falls falsch, könnte die Reduktion in einem nicht bedachten Edge-Case (z. B. `expected_hours` explizit auf 0 konfiguriert bei einem NICHT-dynamischen Vertrag) unerwartet aktiv werden — sollte mit einem gezielten Edge-Case-Test abgesichert werden, nicht blind implementiert |

**Kein LOW-confidence-Befund** — alle Claims sind entweder direkt aus dem
gelesenen Produktionscode zitiert (`[VERIFIED: Codebase]`) oder aus
Live-HTTP-Antworten des laufenden Dev-Servers bestätigt (`[VERIFIED: live
:3000]`). Kein externes Web/Docs-Research nötig, da keine neue Bibliothek
involviert ist.

## Open Questions

1. **Sollte `weight_for_week` eine explizite fünfte Rückgabegröße für den
   „Planungs-Soll-Rohwert vor Reduktion" bekommen, statt Element 1 doppelt
   zu interpretieren?**
   - What we know: Element 1 liefert heute bereits exakt den benötigten
     Rohwert (unreduziert, aber korrekt `relation`-skaliert für
     Teilwochen/Vertragsstart/-ende).
   - What's unclear: Ob eine Umbenennung (`dynamic_hours` → z. B.
     `raw_target_hours`) die Lesbarkeit verbessert, ohne die
     Byte-Identität aller **anderen** Konsumenten von `weight_for_week`
     (es gibt nur die zwei hier untersuchten Aufrufer) zu gefährden.
   - Recommendation: Keine Signatur-Änderung — Element 1 direkt
     wiederverwenden, ggf. mit einem erklärenden Kommentar an der
     Destrukturierungsstelle. Eine Umbenennung ist rein kosmetisch und
     erhöht das Diff ohne fachlichen Nutzen.

2. **Zeigt die Jahresansicht (`get_year`-Konsument im Frontend) das neue Feld
   überhaupt an (D-64-02)?**
   - What we know: `get_year` liefert das Feld automatisch mit (da
     `assemble_weeks`-basiert), sobald `assemble_weeks` es füllt — keine
     zusätzliche Backend-Arbeit nötig.
   - What's unclear: Ob die Jahresansicht im Frontend heute überhaupt eine
     Soll-Spalte rendert (ROADMAP behauptet: nein — nicht in dieser
     Recherche gegen den Frontend-Code verifiziert, da `get_year` in dieser
     Phase keine Pflicht-Änderung ist).
   - Recommendation: Falls die Jahresansicht keine Soll-Spalte hat, ist
     D-64-02 automatisch erledigt (Feld ist im DTO vorhanden, wird nur
     nicht gerendert) — der Plan muss dafür nichts Zusätzliches tun.

## Environment Availability

Nicht anwendbar — reine Code-Änderung in bereits vorhandenen Rust-/Dioxus-
Crates, kein neues externes Tool, kein neuer Service, keine neue
Registry-Abhängigkeit. `cargo`/`node` sind laut Toolchain-Hinweis über den
gegebenen `PATH`-Präfix erreichbar (in dieser Recherche für Live-Curl-Checks
gegen `:3000` verwendet, siehe Sources).

## Package Legitimacy Audit

Nicht anwendbar — diese Phase installiert **keine** neue externe
Abhängigkeit. Keine `Cargo.toml`-Änderung an Crate-Dependencies erwartet.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | nein | keine Änderung an Auth-Flows |
| V3 Session Management | nein | unberührt |
| V4 Access Control | ja (unverändert) | `GET /report` ist auf `HR_PRIVILEGE` gegated (`permission_service.check_permission(HR_PRIVILEGE, ...)`, `reporting.rs:1173-1175`); `GET /report/week/{year}/{week}` hat laut Code-Kommentar keinen eigenen Gate-Check auf REST-Ebene, sondern verlässt sich auf den darunterliegenden `employee_work_details_service` — **das ist bestehendes, von dieser Phase nicht verändertes Verhalten**, kein neuer Befund |
| V5 Input Validation | nein | keine neuen Eingabeparameter — reines zusätzliches Ausgabefeld |
| V6 Cryptography | nein | nicht betroffen |

### Known Threat Patterns für diesen Scope

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Informationspreisgabe über ein neues, ungegatetes Feld | Information Disclosure | Das neue Feld trägt keine sensibleren Informationen als die bereits im selben DTO vorhandenen `expected_hours`/`dynamic_hours` — kein zusätzliches Datenschutz-Risiko. Kein zusätzlicher Gate-Check nötig, da der bestehende Endpoint-Gate unverändert bleibt |

Da diese Phase keine neuen Eingaben, keine neue Authentifizierungs- oder
Autorisierungslogik und keine Kryptografie einführt, ist der
Sicherheits-Impact minimal — die Tabelle dient der Vollständigkeit gemäß
Projekt-Policy (`security_enforcement` nicht explizit deaktiviert in
`.planning/config.json`).

## Sources

### Primary (HIGH confidence — direkt aus dem Repository gelesen)
- `service_impl/src/reporting.rs` (Zeilen 769-1154, 1164-1668, 1900-1986, 2137-2193, 2486-2520) — beide Konstruktionsstellen, `weight_for_week`, existierende Test-Fixtures
- `service/src/reporting.rs` (Zeilen 148-171) — `ShortEmployeeReport`-Struct
- `rest-types/src/lib.rs` (Zeilen 380-413) — `ShortEmployeeReportTO` + `From`-Impl
- `service_impl/src/booking_information.rs` (Zeilen 600-660, 790-825) — `available_hours`-Regressions-Lock
- `service_impl/src/billing_period_report.rs` (Zeilen 58-68, 234-237) — Snapshot-Writer liest `EmployeeReport`, nicht `ShortEmployeeReport`
- `shifty-dioxus/src/component/working_hours_mini_overview.rs` (komplett gelesen) — vier Anzeige-Konsumenten, bestehender Fat-Backend-Kommentar
- `shifty-dioxus/src/loader.rs` (Zeilen 410-460) — Ladepfad `build_working_hours_mini`
- `shifty-dioxus/src/state/employee_work_details.rs` (Zeilen 1-30) — `WorkingHoursMini`-Struct
- `shifty-dioxus/src/api.rs` (Zeilen 850, 858-870) — REST-Client-Aufrufe für Wochen-/Jahres-Report
- `rest/src/report.rs` (Zeilen 64-92, 140-178) — REST-Handler beider Endpoints, Auth-Gate-Lage
- `service_impl/src/test/reporting_phase2_fixtures.rs` (komplett gelesen) — dynamisch/fix-Spiegel-Fixture
- `service_impl/src/test/reporting_additive_merge.rs` (Zeilen 1-1241 gelesen) — Cross-Site-Paritätsmuster, Mock-Boilerplate
- `.planning/ROADMAP.md` (Zeilen 160-172, 590-690) — Phase-64-Sektion, PITFALLS-Verweise
- `.planning/REQUIREMENTS.md` (Zeilen 42-47, 98-101) — PSOLL-01..04
- `.planning/STATE.md` (Zeilen 1-323) — Projekt-Historie, aktueller Stand
- `.planning/todos/pending/2026-09-15-sollstunden-fehlen-bei-dynamischen-vertraegen-unter-schichtplan.md` (komplett gelesen) — Origin-Meldung, bereits vorhandene Fehlanalyse
- `/home/neosam/.claude/projects/.../memory/f14-rebooking-filter-blockiert-feature.md` — Herkunft des „Phase-57-Bug"-Zitats
- `.planning/config.json` — `tdd_mode: true`, `commit_docs: true`, kein `nyquist_validation`/`security_enforcement`-Override

### Secondary (MEDIUM confidence)
- Live-HTTP-Antworten von `http://localhost:3000/report/week/2026/40` (200, `[]` — keine Vertragsdaten für diese Woche) und `http://localhost:3000/report?year=2026&until_week=40` (200, zeigt aktuelle DTO-Shape inkl. aller Felder, beide Beispielpersonen bei `expected_hours: 0.0, dynamic_hours: 0.0` — kein direkter Live-Beleg für einen dynamischen Vertrag mit tatsächlichen Stunden gefunden, da keine sqlite3-CLI zur DB-Introspektion verfügbar war)

### Tertiary
- keine — kein Web-Research nötig, da kein externes Ökosystem betroffen

## Metadata

**Confidence breakdown:**
- Kern-Mechanismus (`weight_for_week`-Zeroing, Gate-Kopplung): HIGH — Zeile für Zeile aus Produktionscode zitiert
- Zwei-Konstruktionsstellen-Befund: HIGH — beide Stellen vollständig gelesen, `get_year`-Delegation an `assemble_weeks` code-bestätigt
- `available_hours`-Regressions-Lock: HIGH — exakte Zeilen bestätigt, Snapshot-Unberührtheit strukturell nachgewiesen
- Frontend-Datenfluss-Kette: HIGH — jede Station gelesen, keine Lücke
- D-64-01/D-64-03-Empfehlungen: MEDIUM — plausible Defaults, aber keine gepinnte User-Entscheidung (kein discuss-phase durchlaufen)
- Live-DB-Beleg für einen tatsächlichen dynamischen Vertrag mit Stunden: LOW/nicht verifiziert — sqlite3-CLI nicht verfügbar, Live-Endpoints lieferten keine positive Beispielzeile; Code-Analyse ist davon unabhängig und ausreichend belastbar

**Research date:** 2026-09-29
**Valid until:** stabil bis zur nächsten Änderung an `service_impl/src/reporting.rs` — keine zeitliche Verfallsgrenze nötig, da rein interne Codebase-Recherche ohne externe Ökosystem-Abhängigkeit
