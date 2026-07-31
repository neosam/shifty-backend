---
phase: 57-f14-rebooking-wirksamkeit-requirements-korrektur-balance-aus
reviewed: 2026-07-31T00:00:00Z
depth: standard
files_reviewed: 16
files_reviewed_list:
  - service_impl/src/reporting.rs
  - service_impl/src/rebooking_reconciliation.rs
  - service_impl/src/billing_period_report.rs
  - service_impl/src/test/mod.rs
  - service_impl/src/test/rebooking_reference_scenario.rs
  - service_impl/src/test/rebooking_neutrality.rs
  - service_impl/src/test/rebooking_reconciliation.rs
  - service_impl/src/test/rebooking_backfill.rs
  - service_impl/src/test/voluntary_rebooking.rs
  - service_impl/src/test/voluntary_rebooking_scheduler.rs
  - service_impl/src/test/billing_period_straddling.rs
  - service_impl/src/test/billing_period_snapshot_locking.rs
  - service_impl/src/test/booking_information.rs
  - service_impl/src/test/booking_information_vfa.rs
  - service_impl/src/test/pdf_export_config.rs
  - service_impl/src/test/shiftplan_report.rs
findings:
  critical: 3
  warning: 13
  info: 2
  total: 18
status: issues_found
---

# Phase 57: Code Review Report

**Reviewed:** 2026-07-31
**Depth:** standard
**Files Reviewed:** 16
**Status:** issues_found

## Summary

Geprüft wurden die drei Produktionsdateien (`reporting.rs`, `rebooking_reconciliation.rs`,
`billing_period_report.rs`) und die 13 Testdateien der Phase 57.

Was **hält**: Der Marker-Filter ist an allen vier von Phase 55 gesetzten Sites entfernt
(`grep ExtraHoursSource` findet in `service_impl/src` nur noch die eine bewusst
verbliebene Ausnahme in `assemble_weeks` und die beiden Schreiber in `rebook()`).
Die `availability()`-basierte Ausnahme in `assemble_weeks` ist notwendig und in beide
Rebooking-Richtungen korrekt. Der Snapshot-Bump 12→13 ist an **allen fünf**
Version-Locking-Gates nachgezogen (`billing_period_snapshot_locking`,
`booking_information`, `booking_information_vfa`, `pdf_export_config` inkl.
`include_str!`-Grep-Gate, `shiftplan_report`) und mit `docs/features/F08-billing-period.md`
+ `docs/domain/billing-period.md` synchron. `cargo clippy --workspace --all-targets`
läuft ohne Warnung. Die HR-Vorschlagsformel `proposed_hours` ist wie gefordert
unangetastet (kein Finding).

Was **nicht hält**: Drei Befunde sind Korrektheits-Blocker.

1. Der neue Balance-Read verankert die Jahres-Range an `ShiftyDate::first_day_in_year(iso_year)`.
   Das ist ein **Kalender**-Datum, das in Jahren mit Fr/Sa/So-Neujahr (2027, 2028, 2021,
   2022, 2023) in das **vorherige ISO-Jahr** fällt. Folge: `get_report_for_employee_range`
   liest das falsche Carryover-Jahr und zieht eine zusätzliche ISO-Woche des Vorjahres in
   die "year-to-date"-Balance. Der Deckel arbeitet damit ab 2027 auf einer falschen Zahl —
   der Auto-Cron bucht zu viel oder zu wenig.
2. Der Dry-Run-Preview des Backfill verletzt die Invariante I-7, die der Code-Kommentar an
   der neuen Stelle ausdrücklich behauptet: ohne Writes sieht jede Woche denselben
   Headroom, ein Apply-Lauf verbraucht ihn nach der ersten Woche.
3. Für dynamische Verträge (`expected_hours == 0`, Vertragszeile vorhanden) lässt der
   Filter-Fix Stunden **verschwinden**: der Range-Report hebt `expected_hours` um `h` mit
   (Soll=Ist) — Balance bleibt neutral — senkt aber `volunteer_hours` trotzdem um `h`. Vor
   Phase 57 war das Paar hier ein harmloses No-op.

Dazu 13 Warnungen, davon drei mit Substanz: fehlende Epsilon-Behandlung vor
`excess <= 0.0` (Micro-Pair + verbrannter UNIQUE-Slot), veraltete Trait-/Domänen-Doku, die
Nachfolger direkt in den behobenen Bug zurückführt, und eine Verdopplung der
YTD-Report-Roundtrips im Backfill, die den in Phase 56 explizit gesetzten DoS-Bound
aufweicht.

## Narrative Findings (AI reviewer)

### Critical Issues

#### CR-01: Balance-Range ist kalender-verankert — falsches Carryover-Jahr + Extra-ISO-Woche ab 2027

**File:** `service_impl/src/rebooking_reconciliation.rs:645-659` und `service_impl/src/rebooking_reconciliation.rs:776-790`

**Issue:**
Beide neuen Balance-Reads bauen den Range-Start als
`ShiftyDate::first_day_in_year(iso_year)`. `first_day_in_year` geht über
`ShiftyDate::from_ymd(year, 1, 1)` (`shifty-utils/src/date_utils.rs:200-203`) und
speichert das Ergebnis als **ISO**-Tripel. Fällt der 1. Januar auf Fr/Sa/So, gehört er
zum vorherigen ISO-Jahr:

| Jahr | 1. Jan | resultierendes `ShiftyDate.year` |
|---|---|---|
| 2026 | Do | 2026 (W01) — ok |
| 2027 | Fr | **2026** (W53) |
| 2028 | Sa | **2027** (W52) |
| 2021/2022/2023 | Fr/Sa/So | jeweils Vorjahr |

`get_report_for_employee_range` verwendet `from_date.year()` unmittelbar für den
Carryover-Lookup (`service_impl/src/reporting.rs:1787-1794`):

```rust
.get_carryover(*sales_person_id, from_date.year() - 1, ...)
```

Für `iso_year = 2027` liest der Deckel damit den Carryover von **2025** statt 2026 — die
tatsächliche Eröffnungsbilanz des Jahres fließt nie in den Headroom ein. Zusätzlich
iteriert `hours_per_week` ab `from_date.as_shifty_week()` = `(2026, 53)`, zieht also eine
volle ISO-Woche des Vorjahres in die als "year-to-date 2027" gedachte Balance — dieselben
Stunden, die der 2026er-Carryover bereits abbildet.

Das ist kein kosmetischer Off-by-one: der Wert steuert `max(-balance, 0)` und damit
direkt den `hours`-Betrag der geschriebenen `extra_hours`-Paare. Ein falsch positiver
Headroom hebt das bezahlte Stundenkonto über 0 (genau die Invariante I-4, die die Phase
herstellen will); ein falsch negativer unterdrückt legitime Überträge und verbrennt dabei
über den Batch-Slot-Claim nichts, aber der Mitarbeiter bleibt im Minus.

Das Muster ist aus `get_report_for_employee` (`reporting.rs:1643`) kopiert — dort ist es
ebenso fragwürdig, aber dort ist `year` ein Report-Anzeigejahr, hier steuert es einen
Write.

**Fix:** Die Range an ISO-Koordinaten verankern, damit `from_date.year() == iso_year`
gilt und die Wochen-Iteration exakt das ISO-Jahr abdeckt:

```rust
let year_start = ShiftyDate::new(iso_year, 1, DayOfWeek::Monday)
    .map_err(|_| ServiceError::InternalError)?;
let week_end = ShiftyDate::new(iso_year, iso_week, DayOfWeek::Sunday)
    .map_err(|_| ServiceError::InternalError)?;
```

An beiden Call-Sites identisch. Anschließend einen Test für `iso_year = 2027` ergänzen,
dessen `.withf` `from == ShiftyDate::new(2027, 1, DayOfWeek::Monday)` pinnt (siehe WR-09) —
der vorhandene Test in `voluntary_rebooking.rs::run_for_week_balance_cap_binds` pinnt nur
2026 und ist gegen diesen Fehler blind.

---

#### CR-02: Dry-Run-Preview des Backfill verletzt die Invariante I-7, die der Code selbst behauptet

**File:** `service_impl/src/rebooking_reconciliation.rs:764-830`

**Issue:**
Der Kommentar ab Zeile 764 begründet den Balance-Read im Backfill damit, dass ein
Platzhalter „den Dry-Run-Preview von dem abweichen ließe, was der spätere Apply-Lauf über
`run_for_week` tatsächlich bucht. Der Doc-Kommentar auf `BackfillPreviewItem.excess`
verspricht ‚der Überschuss, der gebucht würde‘ — das ist die Invariante I-7."

Genau diese Invariante hält die Implementierung im `dry_run`-Zweig nicht. Die im selben
Kommentar genannte Begründung („der Wochen-Loop teilt EINE `read_tx`, und SQLite sieht
innerhalb derselben Transaktion die in Woche N-1 geschriebenen Rows") setzt voraus, dass
in Woche N-1 überhaupt geschrieben wurde. Im Dry-Run wird nichts geschrieben (Zeile
825-830: `item.skipped = false; items.push(item); continue;`), also sieht jede Woche
denselben, unverbrauchten Headroom.

Konkret, Person mit Balance −10 zu Beginn und je 4 h freiwilligem Überschuss in KW 10/11/12:

| | KW 10 | KW 11 | KW 12 | Summe |
|---|---|---|---|---|
| Apply (`run_for_week`, Writes wirken) | 4 | 4 | 2 | **10** |
| Dry-Run-Preview | 4 | 4 | 4 | **12** |

Der Preview meldet 12 h, gebucht werden 10 h. Das ist die HR-facing Entscheidungsgrundlage
der `/backfill?dry_run=true`-Antwort (`rest/src/rebooking.rs:144-155`), und der Fehler
wächst linear mit der Anzahl der Backfill-Wochen — also genau im Mehrjahres-Szenario, für
das der Backfill existiert.

Der neue Test `rebooking_backfill.rs::backfill_dry_run_preview_respects_balance_cap`
verdeckt das, weil er mit `clock_2026_w12()` und `from = KW 11` **genau eine** Woche
iteriert (`assert_eq!(items.len(), 1, ...)`).

**Fix:** Im Dry-Run den bereits vergebenen Headroom mitführen, statt ihn jedes Mal frisch
aus dem Report zu lesen:

```rust
// vor dem Wochen-Loop
let mut dry_run_consumed: HashMap<Uuid, f32> = HashMap::new();
// ...
let balance = report.balance_hours
    + dry_run_consumed.get(&sales_person_id).copied().unwrap_or(0.0);
let excess = week_excess(ist, 0.0, committed, balance);
// im dry_run-Zweig, nachdem excess feststeht:
*dry_run_consumed.entry(sales_person_id).or_insert(0.0) += excess;
```

Alternativ die Invariante explizit zurücknehmen (Doc-Kommentar + `BackfillPreviewItem`-Doc
umschreiben auf „unabhängige Wochen-Obergrenze, Summe kann den Apply überschätzen") — dann
aber mit einem Test, der die Divergenz über >= 2 Wochen festnagelt, damit die Erwartung
dokumentiert bleibt.

---

#### CR-03: Bei dynamischen Verträgen lässt das Rebooking-Paar Stunden verschwinden

**File:** `service_impl/src/reporting.rs:2317-2323` und `service_impl/src/reporting.rs:2370-2371`, `service_impl/src/reporting.rs:2408-2414`

**Issue:**
`hours_per_week` behandelt den dynamischen Vertrag (Vertragszeile vorhanden,
`working_hours_for_week == 0.0`) als Soll=Ist:

```rust
let expected_hours = if !has_contract_row { 0.0 }
    else if working_hours_for_week == 0.0 { shiftplan_hours + extra_work_hours }  // <- +h landet hier
    else { working_hours_for_week };
// ...
balance: shiftplan_paid + extra_work_hours - expected_hours + absence_hours,
```

Das `+h ExtraWork`-Bein erhöht damit **zugleich** `overall` und `expected` → Balance
bleibt unverändert. Das `−h VolunteerWork`-Bein wird in `volunteer_hours` (Zeile 2408-2412,
reiner Kategorie-Filter, keine Source-Ausnahme) dagegen sehr wohl gezählt → Ehrenamt-Ist
sinkt um `h`.

Netto: `h` Stunden verlassen das Ehrenamt-Konto und kommen im bezahlten Konto **nicht** an.
Vor Phase 57 war das Paar hier durch den pauschalen Marker-Filter ein harmloses No-op.

Erreichbar ist das über den HR-Pfad: weder `rebook_manual`
(`rebooking_reconciliation.rs:255-321`) noch `approve` (`:416-548`) prüfen, ob die
Zielperson in der Zielwoche überhaupt einen nicht-dynamischen Vertrag hat — im Gegensatz
zum Auto-Cron/Backfill, der über `wd.cap_planned_hours_to_expected` filtert
(`:759-761`). `suggest` liefert für so eine Person außerdem ein
`balance_after = balance + proposed`, das nie eintritt.

Zusatzbefund derselben Klasse: `assemble_weeks` (Wochenübersicht) verhält sich für
dynamische Wochen **anders** — dort ist `expected_hours_for_cap = planned_hours (= 0)`
und `balance_hours = shiftplan_paid + extra_working_hours` (`reporting.rs:1087-1103`), das
`+h` wirkt also. Dieselbe Person zeigt in Wochenübersicht und Jahresreport unterschiedliche
Balancen für dasselbe Paar.

**Fix:** Zwei Optionen, beide vertretbar — die Entscheidung gehört ins Requirement:

a) Schreibseitiges Gate (minimal-invasiv, deckt den erreichbaren Pfad):
```rust
// in rebook_manual / approve, vor dem Pair-Write
let wd = self.employee_work_details_service
    .all_for_week(iso_week, iso_year, Authentication::Full, Some(tx.clone())).await?;
let eligible = wd.iter().any(|w| w.sales_person_id == sales_person_id
    && w.cap_planned_hours_to_expected);
if !eligible {
    return Err(ServiceError::ValidationError(Arc::from([
        service::ValidationFailureItem::InvalidValue(
            "rebooking requires a capped contract row for this week".into())])));
}
```

b) Leseseitig: `extra_work_hours` in der Soll=Ist-Formel (`reporting.rs:2320`) um
`source == Rebooking`-ExtraWork-Rows bereinigen, damit das `+h` nicht ins Soll leakt —
symmetrisch zur bereits vorhandenen Ausnahme in `assemble_weeks`.

In jedem Fall: ein Test mit `expected_hours = 0.0 / is_dynamic = true` in
`rebooking_reference_scenario.rs` ergänzen; das aktuelle Fixture nutzt durchgängig
`expected_hours: 5.0` und trifft den Zweig nie.

---

### Warnings

#### WR-01: `get_reports_for_all_employees` — no-contract-/dynamic-Zweige zählen das `−h`-Bein gar nicht

**File:** `service_impl/src/reporting.rs:1363-1421`

**Issue:** Beide Sonderzweige setzen `volunteer_hours` aus reinen Shiftplan-Größen
(`auto_volunteer_hours + shiftplan_hours` bzw. `auto_volunteer_hours`, Zeilen 1388 / 1417)
und summieren manuelle `VolunteerWork`-Rows **überhaupt nicht**. Das `+h ExtraWork`-Bein
wird dagegen über den Kategorie-Filter (Zeile 1370 / 1398) gezählt. Ergebnis im
no-contract-Zweig: Balance steigt um `h`, Ehrenamt-Ist bleibt unverändert — dieselben
Stunden zählen doppelt. Im dynamic-Zweig verschwindet das Paar komplett (`planned_hours =
overall_hours`, kein `−h`). Beides weicht von `hours_per_week` und `assemble_weeks` ab,
die für dieselbe Person und Woche andere Zahlen liefern.

**Fix:** In beiden Zweigen `volunteer_hours` um die manuelle Summe ergänzen, analog
`assemble_weeks:962-969`:

```rust
let manual_volunteer: f32 = extra_hours_array.iter()
    .filter(|eh| eh.category == ExtraHoursCategory::VolunteerWork
        && eh.date_time.iso_week() == week && eh.date_time.year() as u32 == year)
    .map(|eh| eh.amount).sum();
// ... volunteer_hours: manual_volunteer + auto_volunteer_hours + shiftplan_hours
```

---

#### WR-02: Kein Epsilon vor `excess <= 0.0` — Float-Residuum erzeugt Micro-Pair und verbrennt den UNIQUE-Slot

**File:** `service_impl/src/rebooking_reconciliation.rs:667-673` (und `:795`, `:819`)

**Issue:** `week_excess` liefert `min(unconstrained, -balance)`. `balance` ist eine
f32-Summe über Shiftplan-, ExtraHours- und Carryover-Beiträge; nach einem exakt
ausgleichenden Übertrag landet sie praktisch nie auf bit-exakt `0.0`, sondern auf einem
Residuum wie `-1.2e-6`. Der Guard prüft strikt `excess <= 0.0`, also läuft der
Write-Pfad an:

* `rebook()` schreibt ein `-0.0000012 / +0.0000012`-Paar (Datenrauschen in `extra_hours`),
* `rebooking_batch_service.create(...)` claimt den **UNIQUE-Slot** `(person, iso_year, iso_week)`.

Der zweite Effekt ist der teure: eine spätere legitime manuelle Umbuchung oder ein
HR-`suggest` für dieselbe Woche läuft danach in `EntityAlreadyExists` → HTTP 409, ohne dass
fachlich jemals etwas gebucht wurde. Im Backfill-Apply über viele Wochen trifft das
potenziell jede Woche nach der ersten, in der der Deckel gebunden hat.

Bemerkenswert: `rebook_manual` validiert `hours <= 0.0 || !hours.is_finite()` hart
(`:273-277`), der Auto-Pfad hat kein Pendant.

**Fix:** Eine gemeinsame Mindest-Buchungsgröße einführen und an beiden Stellen anwenden:

```rust
/// Kleinster buchbarer Uebertrag; darunter ist der Rest f32-Rauschen (WR-02).
pub(crate) const MIN_BOOKABLE_HOURS: f32 = 0.01;
// ...
if excess < MIN_BOOKABLE_HOURS {
    self.transaction_dao.commit(tx).await?;
    return Ok(RunForWeekOutcome::Skipped { reason: SkipReason::NoExcess });
}
```

Im Backfill analog für `would_book` / den `excess <= 0.0`-Zweig.

---

#### WR-03: Trait-Doku beschreibt weiter die ungedeckelte Formel

**File:** `service/src/rebooking_reconciliation.rs:115-116` und `service/src/rebooking_reconciliation.rs:195-197`

**Issue:** Zwei Doc-Kommentare auf der öffentlichen Trait-Oberfläche sind nach dem
Deckel-Einbau falsch:

* `BackfillPreviewItem.excess`: „`max(ist - (soll + committed), 0)` — der Ueberschuss, der
  gebucht wuerde." — der Deckel fehlt. Ausgerechnet dieser Satz wird von
  `test/rebooking_backfill.rs` als Vertrag zitiert.
* `run_for_week`, Ablauf-Schritt 3: „Ueberschuss = `Ist - (Soll + committed_voluntary)` fuer
  die Wochen-Range (Mo..=So); `<= 0` → `Skipped{NoExcess}`" — kein Wort vom Balance-Read
  und vom Headroom.

Laut CLAUDE.md-Docs-Freshness-Gate löst eine Änderung an `service/**/*.rs`
(Trait-Signaturen/Semantik) eine Doku-Pflicht aus; hier ist nicht einmal der Rustdoc am
geänderten Verhalten nachgezogen.

**Fix:**
```rust
/// `min( max(ist - (soll + committed), 0), max(-balance, 0) )` — der Ueberschuss,
/// der gebucht wuerde, gedeckelt auf den Headroom des bezahlten Stundenkontos
/// (REB-AUTO-06). Die `balance` ist die year-to-date-Balance inkl. Carryover bis
/// einschliesslich dieser Woche.
pub excess: f32,
```
und Schritt 3 des `run_for_week`-Ablaufs um den Balance-Read + `min(..., max(-balance,0))`
ergänzen.

---

#### WR-04: `ExtraHoursSource`-Domänendoku fordert weiterhin genau den entfernten Filter

**File:** `service/src/extra_hours.rs:52-58` und `service/src/extra_hours.rs:182-186`

**Issue:** Der Enum-Doc-Kommentar sagt unverändert:

> „F1-Ist- + F2-Soll-Aggregatoren MUESSEN `Rebooking`-Rows herausfiltern (Pitfall 1:
> Doppel-Zaehlung), sonst zaehlen +N/-N-Paare bei Rebooking die freiwilligen Stunden
> doppelt in Reports / Balance."

und die Variante `Rebooking` (`:65-68`) wiederholt es. Das ist nach Phase 57 nachweislich
falsch (das ist der Bug, den die Phase behebt) und steht an der Stelle, an der ein
Nachfolger zuerst nachschlägt. `reporting.rs:189-214` erklärt die neue Lage ausführlich —
aber nur dort, wo der Filter nicht mehr steht.

**Fix:** Beide Blöcke auf die neue Semantik umschreiben, mit expliziter Warnung:

```rust
/// Phase 57 (VOL-ACCT-04, D-57-06): `source` ist ein reines PROVENIENZ-Feld.
/// Die Read-Aggregate filtern NICHT mehr pauschal ueber `source` — die
/// Balance-Relevanz haengt allein an `ExtraHoursCategory::as_report_type()`.
/// Einzige Ausnahme: `assemble_weeks` nimmt die Kombination
/// (`VolunteerWork`, `Rebooking`) aus seiner `availability()`-Summe aus.
/// Ein pauschaler `source == Manual`-Vorfilter macht den Uebertrag
/// freiwillig->bezahlt wirkungslos (Phase-55-Regression).
```

---

#### WR-05: Toter No-op-Rebind nach der Filter-Entfernung

**File:** `service_impl/src/reporting.rs:1217-1218`

**Issue:**
```rust
let extra_hours_array: Arc<[ExtraHours]> =
    extra_hours_array.iter().cloned().collect();
```
Nach Wegfall des `.filter(...)` ist das eine vollständige Tiefkopie eines
`Arc<[ExtraHours]>` in ein identisches `Arc<[ExtraHours]>` — semantisch ein No-op, das nur
noch den Shadowing-Rest des alten Filters darstellt. Clippy schlägt hier nicht an, weil der
Typ formal neu gebunden wird.

**Fix:** Zeilen ersatzlos streichen; das Ergebnis von
`find_by_sales_person_id_and_year(...)` ist bereits `Arc<[ExtraHours]>`.

---

#### WR-06: Backfill liest Report + Stats vor dem Toggle-Gate und danach noch einmal in `run_for_week`

**File:** `service_impl/src/rebooking_reconciliation.rs:776-842`

**Issue:** Zwei zusammenhängende Inkonsistenzen:

1. In `run_for_week` steht der neue Balance-Read ausdrücklich **hinter** Toggle- und
   Locked-Gate, „damit ein ausgeschalteter Cron / eine gesperrte Woche ohne
   Report-Roundtrip zurückkehrt" (`:642-644`). Im Backfill wird derselbe Read
   **vor** dem Cutoff-Gate ausgeführt (`toggle_on` erst Zeile 798-799). Für einen Backfill,
   dessen `from` vor dem Stichtag liegt — der Normalfall bei einem historischen Backfill —
   zahlt jede (Person, Woche) einen vollen YTD-Report plus VoluntaryStats, nur um danach als
   `ToggleOff` verworfen zu werden.
2. Im Apply-Zweig ruft der Backfill `run_for_week` (`:833-842`), das denselben YTD-Report
   und dieselben VoluntaryStats **erneut** liest. Pro (Person, Woche) also zwei komplette
   Report-Aggregationen.

Der YTD-Report ist selbst O(Wochen im Jahr). Aus dem Backfill wird damit
O(Wochen × Personen × Wochen-im-Jahr) Report-Arbeit in **einer** SQLite-Transaktion, hinter
einem Endpunkt ohne untere Schranke für `from` (`rest/src/rebooking.rs:109-112` validiert nur
das Format). Phase 56 hatte den DoS-Bound (T-56-05) als bewusste Design-Eigenschaft geführt;
er ist jetzt deutlich weicher.

**Fix:** Das Cutoff-Gate vor die teuren Reads ziehen und den Doppel-Read auflösen:

```rust
let toggle_on = crate::voluntary_rebooking_gate::should_run_for_week(iso_year, iso_week, active_from);
if !toggle_on {
    items.push(BackfillPreviewItem { /* ... */ skip_reason: Some(SkipReason::ToggleOff), .. });
    continue;
}
// erst danach: report + week_voluntary_figures
```
Für den Apply-Zweig zusätzlich eine untere Schranke auf `from` (z. B. max. 3 Jahre
rückwärts) in `backfill_range` erwägen.

---

#### WR-07: `item.excess` wird nach dem Apply nicht aus dem tatsächlich gebuchten Betrag nachgeführt

**File:** `service_impl/src/rebooking_reconciliation.rs:843-852`

**Issue:** Der Apply-Zweig wertet `RunForWeekOutcome::Booked { .. }` aus und verwirft das
`hours`-Feld (`Booked { .. } => { item.skipped = false; }`). `item.excess` bleibt der Wert,
den `backfill_range` selbst berechnet hat. Sobald die beiden Berechnungen auseinanderlaufen
— und CR-01/WR-02 sind genau solche Fälle, ebenso jede künftige Änderung an `run_for_week` —
meldet die REST-Antwort einen Betrag, der nicht gebucht wurde, obwohl die tatsächliche Zahl
unmittelbar zur Hand ist.

**Fix:**
```rust
RunForWeekOutcome::Booked { hours } => {
    item.skipped = false;
    item.excess = hours; // authoritativ ist, was run_for_week gebucht hat
}
```

---

#### WR-08: `week_excess`-Parameter `soll` ist an beiden Call-Sites tot; `week_voluntary_figures` gibt denselben Wert doppelt zurück

**File:** `service_impl/src/rebooking_reconciliation.rs:82-94`, `:208-234`, `:667`, `:795`

**Issue:** `week_excess(ist, soll, committed_voluntary, balance)` wird ausschließlich mit
`soll = 0.0` aufgerufen (Zeilen 667 und 795). Der Parameter existiert nur noch, um in der
Formel zu stehen. Parallel liefert `week_voluntary_figures` das Tupel `(ist, soll, soll)`
(Zeile 233, Kommentar: „committed == soll_total") — der Aufrufer in `run_for_week`
destrukturiert es entsprechend als `(ist, _soll, committed)`. Zwei Freiheitsgrade, die
keine sind; sie laden dazu ein, versehentlich `soll` **und** `committed` zu übergeben und
den Überschuss doppelt abzuziehen.

**Fix:** Signatur auf die tatsächlichen Freiheitsgrade reduzieren und die Formel-Doku
entsprechend anpassen:

```rust
pub(crate) fn week_excess(ist: f32, committed_voluntary: f32, balance: f32) -> f32
```
und `week_voluntary_figures` auf `(f32, f32)` = `(ist, committed)` kürzen. Die
`week_excess_matrix` in `voluntary_rebooking.rs` entsprechend nachziehen.

---

#### WR-09: Keine Testabdeckung für die ISO-Jahresgrenze im neuen Balance-Read

**File:** `service_impl/src/test/voluntary_rebooking.rs:393-404`

**Issue:** Das `.withf`-Prädikat pinnt die Balance-Range strukturell — aber mit
`ShiftyDate::first_day_in_year(2026)`, und 2026 ist eines der Jahre, in denen Kalender- und
ISO-Jahresanfang zusammenfallen (1. Jan = Donnerstag = W01). Der Test ist damit gegen CR-01
konstruktionsbedingt blind. Für die Jahre, in denen der Fehler auftritt (2027, 2028), gibt
es keinen Fall. `rebooking_backfill.rs::backfill_year_boundary_walk` prüft nur den
Wochen-Walk, nicht den Balance-Range.

**Fix:** Einen zweiten Fall mit `iso_year = 2027` ergänzen, dessen `.withf` `from` auf den
Montag der ISO-KW 1/2027 (= 2027-01-04) pinnt. Der Test schlägt gegen den aktuellen Stand
rot aus und wird nach dem CR-01-Fix grün.

---

#### WR-10: `assert_snapshot_shift` überspringt zwei von vier Snapshot-Feldern für genau die verschobenen value_types

**File:** `service_impl/src/test/billing_period_straddling.rs:465-482`

**Issue:** Im `Some((_, expected_shift))`-Arm werden nur `value_delta` (Toleranzvergleich)
und `value_ytd_from` (Bit-Vergleich) geprüft. `value_ytd_to` und `value_full_year` werden
für Overall/Balance/ExtraWork/Volunteer **überhaupt nicht** verglichen — nur im
`None`-Arm (die unbetroffenen value_types) laufen alle vier Feldvergleiche. Genau diese
beiden Felder sind aber die, in denen der Bump-Grund („eine frische Neuberechnung liefert
andere Werte") sich manifestiert, und die im Doc-Kommentar der Datei ausdrücklich als
betroffen genannt werden. Ein Regressions-Fix, der `value_full_year` wieder rebooking-blind
macht, bliebe unentdeckt.

**Fix:** Im `Some`-Arm dieselbe Verschiebung auch für `value_ytd_to` und `value_full_year`
assertieren (die Rebooking-Woche liegt innerhalb der Periode und innerhalb des Jahres, also
gilt derselbe Shift):

```rust
for (field, act, exp) in [
    ("value_delta",     act_val.value_delta,     exp_val.value_delta),
    ("value_ytd_to",    act_val.value_ytd_to,    exp_val.value_ytd_to),
    ("value_full_year", act_val.value_full_year, exp_val.value_full_year),
] {
    let actual_shift = act - exp;
    assert!((actual_shift - expected_shift).abs() < 1e-3,
        "value_type `{name}` field `{field}`: erwartet {expected_shift}, gemessen {actual_shift}");
}
assert_f32_bit_eq(act_val.value_ytd_from, exp_val.value_ytd_from, &name, "value_ytd_from");
```

---

#### WR-11: Test- und Dateinamen behaupten das Gegenteil der Assertion

**File:** `service_impl/src/test/rebooking_neutrality.rs:373,383`, `service_impl/src/test/rebooking_reconciliation.rs:253`, `service_impl/src/test/booking_information.rs:449`

**Issue:** Vier Namen sind nach der Semantik-Umkehr aktiv irreführend:

* `rebooking_neutrality.rs::test_neutrality_balance` assertiert `delta == +2.0`.
* `rebooking_neutrality.rs::test_neutrality_f1_ist` assertiert `delta == -2.0`.
* `rebooking_reconciliation.rs::rebooking_neutrality_single_manual` assertiert
  Nicht-Neutralität.
* `booking_information.rs::snapshot_schema_version_pinned_at_10` pinnt 13.

Bei den ersten dreien ist der Modul-Doc-Kommentar korrigiert, die Namen nicht — ein
`cargo test neutrality`-Lauf oder ein CI-Report suggeriert weiterhin, Neutralität sei die
geprüfte Eigenschaft. Beim vierten begründet ein neu hinzugefügter Kommentar (Zeile 450-455)
den Verzicht auf ein Rename mit einem Querverweis aus `booking_information_vfa.rs` — das ist
ein Kommentar-Querverweis, kein Compile-Abhängigkeit; ein Rename hätte beide Stellen in
einem Commit korrigiert.

**Fix:** Umbenennen und die Querverweise mitziehen:
`test_neutrality_balance` → `rebooking_raises_balance`,
`test_neutrality_f1_ist` → `rebooking_lowers_volunteer_ist`,
`rebooking_neutrality_single_manual` → `rebooking_effective_single_manual`,
`snapshot_schema_version_pinned_at_10` → `snapshot_schema_version_pinned`.
Die Datei `rebooking_neutrality.rs` selbst trägt jetzt drei Wirksamkeits- und einen
Neutralitätstest — `rebooking_effectiveness.rs` wäre der ehrlichere Name.

---

#### WR-12: Keine Leseseiten-Abdeckung für die Richtung `ExtraToVolunteer`

**File:** `service_impl/src/test/rebooking_neutrality.rs:181-186`, `service_impl/src/test/rebooking_reference_scenario.rs:128-145`

**Issue:** Sämtliche neuen Leseseiten-Fixtures erzeugen ausschließlich das
`VolunteerToExtra`-Paar (`-h VolunteerWork` / `+h ExtraWork`). Die Korrekturrichtung
`ExtraToVolunteer` (`pair_categories`, `rebooking_reconciliation.rs:99-110`) ist über
`rebook_manual` erreichbar und läuft im Report durch dieselbe neue Filter-Ausnahme —
nur mit umgekehrten Vorzeichen. Ein Fehler in der Ausnahme (z. B. ein Vorzeichenfehler
oder eine auf `amount < 0.0` konditionierte Variante) fiele nur in dieser Richtung auf und
ist derzeit ungetestet.

**Fix:** In `rebooking_neutrality.rs` ein Gegenstück ergänzen:
`+h VolunteerWork` / `-h ExtraWork`, beide `source = Rebooking`, mit
`assert_delta(bal_before, bal_after, -2.0, ...)` und
`assert_delta(ist_before, ist_after, +2.0, ...)` — über `get_report_for_employee_range`
**und** `get_week`.

---

#### WR-13: Zweiter `run_for_week`-Lauf meldet jetzt `NoExcess` statt `AlreadyProcessed`

**File:** `service_impl/src/rebooking_reconciliation.rs:661-698`

**Issue:** `week_voluntary_figures` liest `ist` aus `VoluntaryStats`, und
`VoluntaryStatsServiceImpl` bezieht `ist_total` aus `EmployeeReport::volunteer_hours`
(`service_impl/src/voluntary_stats.rs:117`). Seit der Filter-Entfernung senkt das
`−h`-Bein diesen Wert. Ein zweiter Lauf für dieselbe Woche berechnet daher `excess == 0`
und kehrt bei Zeile 668 mit `Skipped{NoExcess}` zurück — er erreicht die Slot-Claim-Prüfung
gar nicht mehr, die bisher `Skipped{AlreadyProcessed}` lieferte. Kein Doppel-Write (der
UNIQUE-Slot bleibt die Absicherung), aber die Skip-Reason-Statistik von Scheduler und
Backfill-Preview verschiebt sich lautlos: HR sieht im Backfill-Preview „kein Überschuss"
statt „bereits verarbeitet". Der bestehende Test
`voluntary_rebooking.rs::run_for_week_already_processed` deckt das nicht ab, weil er den
Stats-Mock konstant hält.

**Fix:** Entweder den Slot-Check vor die Excess-Berechnung ziehen (via
`rebooking_batch_service.find_pending_for_sales_person` bzw. einen Existenz-Lookup), oder —
minimal — die Semantik-Änderung dokumentieren und mit einem Test festnageln, der den
Stats-Mock nach dem ersten Lauf um `h` reduziert und die erwartete Skip-Reason pinnt.

---

### Info

#### IN-01: Tautologische Assertion in `rebooking_reference_scenario.rs`

**File:** `service_impl/src/test/rebooking_reference_scenario.rs:383-384`

**Issue:** `let committed_over_range = 2.0 * work_details().committed_voluntary;` gefolgt
von `assert_close(committed_over_range, 10.0, ...)` prüft nur, dass das eigene Fixture
`committed_voluntary = 5.0` gesetzt hat — kein Produktionscode ist beteiligt. Die
nachfolgende Assertion (Zeile 385-389) ist die eigentlich tragende.

**Fix:** Die erste Assertion streichen oder in einen Kommentar überführen; das Fixture ist
zwölf Zeilen weiter oben sichtbar.

---

#### IN-02: `backfill_range`-Doku behauptet „keine Tx" im Dry-Run

**File:** `service/src/rebooking_reconciliation.rs:224-225` gegen `service_impl/src/rebooking_reconciliation.rs:746`

**Issue:** Der Trait-Doc sagt für `dry_run == true`: „reine Vorschau — schreibt NICHTS
(keine Tx, kein `create`)." Die Implementierung öffnet unbedingt
`self.transaction_dao.use_transaction(tx).await?` und committet am Ende (Zeile 858) — der
Impl-Kommentar (`:744-745`) benennt das korrekt („wir nutzen ein `Option<Transaction>` nur
für die Read-Aufrufe"), der Trait-Doc nicht. Vorbestehend, aber die Phase hat den Block
angefasst.

**Fix:** Im Trait-Doc „keine Tx" durch „nur Read-Tx, kein Write" ersetzen.

---

_Reviewed: 2026-07-31_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
