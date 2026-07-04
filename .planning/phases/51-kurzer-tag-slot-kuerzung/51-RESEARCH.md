---
phase: 51
type: research
question: Q-01
created: 2026-07-04
---

# Phase 51 — RESEARCH (Q-01 Answer)

## TL;DR (for the planner)

**Die Zwei-Ketten-Hypothese aus CONTEXT D-51-06 stimmt NICHT.** Es sind
mindestens **vier** BE-Aggregat-Punkte, die den Cutoff kennen müssen — nicht
zwei. Reporting/Ist-Stunden fließt in Wahrheit **nicht durch `BlockService`**,
sondern durch `ShiftplanReportDao` mit **rohem SQL**, das `slot.time_to -
slot.time_from` in SQLite direkt subtrahiert (`dao_impl_sqlite/src/
shiftplan_report.rs:77, 114, 147`). `BookingInformationServiceImpl` rechnet
`slot_hours` und `required_hours_by_day` an **drei** Stellen direkt aus
`slot.to - slot.from`, mit einer nur-filternden ShortDay-Behandlung
(`service_impl/src/booking_information.rs:399, 407, 517, 524, 686`). Der
`ShiftplanWeek`-Aufbauer in `service_impl/src/shiftplan.rs:42-66` hat schon
heute eine **falsche** ShortDay-Filter-Logik — Slots werden komplett verworfen
statt geclippt (Bug pre-existing).

`SlotTO` ist bidirektional (Read + Write via `POST/PUT /slot`) — daher darf
`SlotTO.to` NICHT gemutiert werden; das würde in `update_slot`/`create_slot`
zurückgeschrieben. Empfehlung: **neues Feld** `effective_to: time::Time`
(nicht `Option`), Präzedenz aus `AbsencePeriodTO.derived_days` (rest-types
:1793) und `VacationBalanceTO.computed_entitled_days`
(rest-types:2167). Für die Rückwärts-Serialisierung `#[serde(default)]` +
Default via `From<&Slot>` = `slot.to`.

FE braucht nur zwei tiny Änderungen: `shifty-dioxus/src/loader.rs:101 + 154`
liest `slot.slot.to` — hier auf `slot.slot.effective_to` umstellen. Alles
andere im FE lebt auf dem gemappten `state::Slot`-Modell und braucht keine
Anpassung.

## Q-01.1 — ShiftplanWeek DTO Builder (chain B)

- **Assembler:** `service_impl/src/shiftplan.rs:218-305`
  (`ShiftplanViewServiceImpl::get_shiftplan_week`) und Parallel-Variante
  `:401-524` (`get_shiftplan_week_for_sales_person`) und
  `get_shiftplan_day` (`:307-399`) und
  `get_shiftplan_day_for_sales_person` (`:526+`). Alle vier delegieren an
  den gemeinsamen Helper `build_shiftplan_day` (`service_impl/src/
  shiftplan.rs:27-131`) bzw. `build_shiftplan_day_for_sales_person`
  (`:147-196`).
- **DTO shape:** `rest-types/src/lib.rs:1094-1098` (`ShiftplanWeekTO {
  year, calendar_week, days: Vec<ShiftplanDayTO> }`) →
  `ShiftplanDayTO` (`:1083-1091`, `{ day_of_week, slots:
  Vec<ShiftplanSlotTO>, unavailable }`) → `ShiftplanSlotTO`
  (`:1069-1080`, `{ slot: SlotTO, bookings, current_paid_count }`) →
  `SlotTO` (`:308-332`).
- **REST endpoints (vier):**
  - `GET /shiftplan/{shiftplan_id}/{year}/{week}` (`rest/src/
    shiftplan.rs:16-18, 50-74`) — global WeekView.
  - `GET /shiftplan/day/{year}/{week}/{day_of_week}` (`:19, 91`) —
    global DayView.
  - `GET /shiftplan/{shiftplan_id}/{year}/{week}/sales-person/
    {sales_person_id}` (`:25-28, 144`) — per-SP WeekView.
  - `GET /shiftplan/day/{year}/{week}/{day_of_week}/sales-person/
    {sales_person_id}` (`:29-32, 198`) — per-SP DayView.
- **Wo Slot-Zeiten in das DTO wandern:** `service_impl/src/shiftplan.rs:114`
  (`ShiftplanSlot { slot: slot.clone(), ... }` im Helper
  `build_shiftplan_day`) → `rest-types/src/lib.rs:1115` (`slot:
  (&slot.slot).into()` im `From<&ShiftplanSlot> for ShiftplanSlotTO`) →
  `rest-types/src/lib.rs:334-350` (`From<&Slot> for SlotTO` — kopiert
  `from`, `to` 1:1).

## Q-01.2 — SpecialDay Lookup Pattern (reusable)

- **Existierender Konsument im Reporting (Holiday derive-on-read):**
  `service_impl/src/reporting.rs:188-198` — Loop über
  `week.iter_until(&to_week)`, pro Woche `special_day_service.get_by_week(
  week.year, week.week, context.clone()).await?`, dann Filter auf
  `sd.day_type != SpecialDayType::Holiday`. **Genau dieses Pattern
  wiederverwenden**, nur mit ShortDay-Filter statt Holiday.
- **Existierender ShortDay-Konsument im Shiftplan-Bauer (aber buggy):**
  `service_impl/src/shiftplan.rs:42-51` (Helper `build_shiftplan_day`)
  sucht schon heute den `time_of_day` per `special_days.iter().find_map`.
  Der find_map ist korrekt geschrieben (`day_of_week == day` + `day_type
  == ShortDay` + `time_of_day.is_some()`). **Das Problem ist
  ausschließlich der Konsum bei `:62-66`:** `if slot.to > cutoff {
  continue; }` verwirft den ganzen Slot statt zu clippen (verletzt D-04
  Zeile 3 der Cutoff-Tabelle: „`slot.start < cutoff < slot.end`
  gekürzt").
- **Existierender ShortDay-Konsument im Booking-Info (auch buggy):**
  `service_impl/src/booking_information.rs:394-401, 512-519` — identisches
  Filter-Anti-Pattern: `special_days.iter().any(day => day.day_type ==
  ShortDay && slot.to > day.time_of_day.unwrap()) → skip`.
- **DAO-Service-Surface:** `SpecialDayService::get_by_week(year, week,
  context) -> Arc<[SpecialDay]>` (`service/src/special_days.rs:85-90`);
  Impl `service_impl/src/special_days.rs:58-71` delegiert an
  `special_day_dao.find_by_week(year, week)`. Für Bereichs-Lookups (mehrere
  Wochen) existiert keine kombinierte API — Konsumenten loopen (siehe
  Reporting-Muster oben).
- **ShortDay-Variante:** `service/src/special_days.rs:13-16`
  (`pub enum SpecialDayType { Holiday, ShortDay }`).
  `SpecialDay.time_of_day: Option<time::Time>` an `service/src/
  special_days.rs:41` ist das Cutoff-Feld — bei `Holiday` `None`, bei
  `ShortDay` erwartet-`Some` (aber die Insert-Validation in `service_impl/
  src/special_days.rs:133` erlaubt technisch auch `ShortDay` mit `None` —
  dann skip). Der Clip muss also nur `ShortDay + time_of_day.is_some()`
  Einträge berücksichtigen.

## Q-01.3 — PDF Renderer Input

- **`render_shiftplan_week_pdf` Signatur (verbatim, 5-Parameter):**
  `service_impl/src/pdf_render.rs:156-162`:
  ```rust
  pub fn render_shiftplan_week_pdf(
      week: &ShiftplanWeek,
      sales_persons: &[SalesPerson],
      header_year: u32,
      header_week: u8,
      render_timestamp: time::OffsetDateTime,
  ) -> Result<Vec<u8>, ServiceError>
  ```
- **Konsumiert PDF-Renderer dasselbe Aggregat wie WeekView?**
  **JA**, exakt dasselbe `service::shiftplan::ShiftplanWeek`-Struct
  (`service/src/shiftplan.rs:8-12`). Der Assembler `PdfShiftplanServiceImpl`
  (`service_impl/src/pdf_shiftplan.rs:126-170`) ruft an `:149-152`
  `self.shiftplan_view_service.get_shiftplan_week(shiftplan_id, year,
  calendar_week, ...)` — **kein separater Bauer**. Das ist
  Fat-Backend-Konsequenz: PDF ist read-only View der gleichen Daten.
- **Wo der Renderer die Slot-Zeit liest:** `service_impl/src/
  pdf_render.rs:494-497` (`compute_slot_duration_hours(slot: &Slot)`)
  liest `slot.from` und `slot.to` direkt vom inneren `service::slot::Slot`
  (das im `ShiftplanSlot` steckt). Auch `:581-584` (`format!("{}:{}",
  slot.from.hour(), slot.from.minute(), slot.to.hour(), slot.to.minute())`
  — das rendert die Slot-Beschriftung im Kasten.

**Implikation:** Wenn `build_shiftplan_day` beim Aufbau in `service_impl/
src/shiftplan.rs:113-117` den `slot.to` schon geclippt in den ausgelieferten
`ShiftplanSlot.slot` steckt (also Slot-Klon mit korrigiertem `to`), ist der
PDF-Renderer **automatisch korrekt** — kein Fix in `pdf_render.rs` nötig.
Diese Variante ist auch für die WeekView semantisch richtig, weil der
Frontend-Slot in `loader.rs:100-101` genau die `slot.slot.to` liest.

## Q-01.4 — BlockService als Sole Chain-A Aggregate — NEIN

- **BlockService ist NICHT der einzige Aggregat-Punkt für Reporting/
  Ist-Stunden.** BlockService flowt nur in iCal + insufficiently-booked-
  Report + MyBlocks — nicht ins Ist-/Balance-Reporting.
- **`Block`-Konstruktions-Ort:** `service_impl/src/block.rs:47-180`
  (`get_blocks_for_sales_person_week`), erzeugt `Block` an
  `:142-151` und `:165-174`. Fetches `SlotService::get_slot` per Booking
  an `:89-92` — hier müsste der Slot geclippt werden ODER die Slots
  müssten vor dem `Block { from: block_from.unwrap(), to:
  block_to.unwrap() }` re-mapped werden, sonst führt der Merge
  ("consecutive slot" via `slot.from == to`) zu falschen Ergebnissen wenn
  der vorherige Slot geclippt endet ≠ nächster Slot start. **Reihenfolge
  kritisch: erst pro-Slot-Clip, dann Merge.**
- **`get_unsufficiently_booked_blocks`:** `service_impl/src/block.rs:227-346`
  hat einen zweiten Slot-Aggregat-Pfad via
  `SlotService::get_slots_for_week_all_plans` (`:237-240`) — separater
  Clip nötig, gleiche Merge-Semantik.
- **Grep-Sweep für Bypass-Sites, die direkt `slot.to - slot.from`
  rechnen und NICHT durch `Block` gehen** (`rg "\.to\s*-\s*.*\.from"
  service_impl/src`, gefiltert um pdf_render + tests + slot_edit-CRUD):
  - `service_impl/src/booking_information.rs:407`
    (`get_weekly_summary` — `slot_hours`)
  - `service_impl/src/booking_information.rs:524`
    (`get_summery_for_week` — `slot_hours`, gleiches Muster)
  - `service_impl/src/booking_information.rs:686`
    (`get_summery_for_week` — `required_hours_by_day`, pro Wochentag)
  - `dao_impl_sqlite/src/shiftplan_report.rs:77`
    (`extract_shiftplan_report` — SUM in SQL, für Balance)
  - `dao_impl_sqlite/src/shiftplan_report.rs:114`
    (`extract_quick_shiftplan_report` — SUM in SQL, für Overview)
  - `dao_impl_sqlite/src/shiftplan_report.rs:147`
    (`extract_shiftplan_report_for_week` — SUM in SQL, für pro-Woche-Sicht)
- **Reporting/Balance-Konsum:** `service_impl/src/reporting.rs`
  ruft `shiftplan_report_service.extract_shiftplan_report_for_week(...)` +
  `extract_shiftplan_report(...)` — **niemals** `block_service`. Der
  Ist-Stunden-Kanal ist SQL-Aggregation im DAO. Grep bestätigt:
  `grep -n "block_service" service_impl/src/reporting.rs` → leer.
- **BookingInformationService-Konsum:** Direktes Slot-Fetch via
  `SlotService::get_slots_for_week_all_plans` (`booking_information.rs:388,
  506`), dann direkte `.to - .from`-Arithmetik. Kein `Block` involved.

**Konsequenz für die Wave-Struktur:** Es reicht **nicht**, an zwei Punkten
zu clippen. Das Feature braucht Anpassungen an mindestens **fünf** Stellen
(reihenfolge-unabhängig):

1. `service_impl/src/shiftplan.rs:42-66` — Filter durch Clip ersetzen
   (Chain B: WeekView + PDF, Read-Aggregat).
2. `service_impl/src/block.rs:47-180` + `:227-346` — Slot-Clip vor dem
   Merge einbauen (Chain A': iCal + insufficient-booked + MyBlocks).
3. `service_impl/src/booking_information.rs:388-409, 506-525, 680-697` —
   Direkt-Arithmetik durch clip-aware Version ersetzen (Chain C:
   Weekly-Summary + Booking-Conflicts).
4. `dao_impl_sqlite/src/shiftplan_report.rs:77, 114, 147` — SQL-Aggregation
   muss ShortDay-Cutoff kennen (Chain D: Balance / Ist-Stunden). Alternative:
   Reporting-Layer clippt außerhalb der DAO, indem das rohe SQL nicht mehr
   aggregiert, sondern Slot-Details liefert und der Rust-Layer aggregiert.
5. (Optional) `service_impl/src/shiftplan_edit.rs` — Prüfen ob Booking-
   Create bei Post-Cutoff-Slots weiter erlaubt bleibt (D-51-03 sagt: ja).

Das ist ein **wesentlich größerer Scope** als in CONTEXT.md angenommen. Die
`plan-phase` muss das entweder als vier separate Waves aufziehen oder das
Feature so umbauen, dass **einer** dieser Pfade der kanonische Ist-Stunden-
Rechner wird und die anderen an ihm hängen. Empfehlung: das ist ein
Rediscovery-Punkt — vor plan-phase kurz mit User klären.

## Q-01.5 — SlotTO Field-Design Precedent

- **Aktuelle SlotTO-Form (`rest-types/src/lib.rs:307-332`, verbatim):**
  ```rust
  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
  pub struct SlotTO {
      #[serde(default)]
      pub id: Uuid,
      pub day_of_week: DayOfWeekTO,
      #[schema(value_type = String, format = "time")]
      pub from: time::Time,
      #[schema(value_type = String, format = "time")]
      pub to: time::Time,
      pub min_resources: u8,
      #[serde(default)]
      pub max_paid_employees: Option<u8>,
      pub valid_from: time::Date,
      pub valid_to: Option<time::Date>,
      #[serde(default)]
      pub deleted: Option<time::PrimitiveDateTime>,
      #[serde(rename = "$version")]
      #[serde(default)]
      pub version: Uuid,
      #[serde(default)]
      pub shiftplan_id: Option<Uuid>,
  }
  ```
- **Precedents für "derived / effective / computed" Felder auf TOs:**
  - `rest-types/src/lib.rs:1793` — `AbsencePeriodTO.derived_days: f32`
    mit Kommentar-Muster:
    > „Read-only Anzeige-Feld: die abgeleiteten [...] Vom List-Endpoint
    > befüllt — Single Source of Truth ist [...]. Auf Create/Update-
    > Responses und Wire-Roundtrips Default 0.0 (nicht persistiert)."
  - `rest-types/src/lib.rs:2167` — `VacationBalanceTO.computed_entitled_days:
    Option<f32>` mit `#[serde(default)]`.
  - `rest-types/src/lib.rs:1079` — `ShiftplanSlotTO.current_paid_count: u8`
    (dieses lebt allerdings am **Wrapper** `ShiftplanSlotTO`, nicht am
    `SlotTO` selbst — genau derselbe Grund wie wir jetzt haben:
    „bidirektionale-DTO-Corruption vermeiden").
- **Empfehlung (Feld-Name, Typ, Semantik):**
  - **NICHT** `SlotTO.to` mutieren. `SlotTO` ist bidirektional
    (`rest/src/slot.rs:100, 124` — `Json<SlotTO>` in `POST/PUT /slot`).
    Ein geclippter `to` würde beim Frontend-Save zurück in die DB
    geschrieben.
  - **Neues Feld am `ShiftplanSlotTO`, nicht am `SlotTO`.** Präzedenz:
    `current_paid_count` lebt exakt aus diesem Grund oben (Wrapper trägt
    Anzeige-Daten, `SlotTO` bleibt persistenz-treu).
  - **Signatur:** `pub effective_to: time::Time` (nicht `Option`;
    Default = `slot.to` wenn kein Cutoff greift, dann `effective_to ==
    to`; das Frontend rendert konsistent immer `effective_to`). Kommentar
    im Muster von `derived_days`.
  - **Optional zusätzlich:** `pub effective_visible: bool` — false wenn
    `slot.from >= cutoff` (dann rendert das FE den Slot gar nicht). Alternative:
    komplett-hinter-Cutoff-Slots aus `days[*].slots` **weggelassen**
    (fehlen im Vec) — das ist bereits das heutige Verhalten in
    `shiftplan.rs:62-66` (verwerfen). Empfehlung: bei Q-01 bleiben —
    Slots komplett hinter Cutoff werden im Aggregate gar nicht erst
    hinzugefügt; nur überlappende Slots landen mit `effective_to <=
    cutoff` im DTO. So bleibt das DTO minimal.

## Slot Struct — signature for `clip_to`

- **`service/src/slot.rs:12-25` verbatim:**
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub struct Slot {
      pub id: Uuid,
      pub day_of_week: DayOfWeek,
      pub from: time::Time,
      pub to: time::Time,
      pub min_resources: u8,
      pub max_paid_employees: Option<u8>,
      pub valid_from: time::Date,
      pub valid_to: Option<time::Date>,
      pub deleted: Option<time::PrimitiveDateTime>,
      pub version: Uuid,
      pub shiftplan_id: Option<Uuid>,
  }
  ```
- **Feld-Typ für Zeitpunkte:** `time::Time` (nicht `chrono::NaiveTime`,
  nicht `u16`). Die `SpecialDay.time_of_day` ist genauso `time::Time`
  (`service/src/special_days.rs:41`) — Direct-Compare mit `<`, `>`, `==`
  ohne Konvertierung möglich. Subtraktion liefert `time::Duration`
  (`.as_seconds_f32() / 3600.0` für Stunden — Muster existiert in
  `booking_information.rs:407`).
- **Empfohlene Signatur (D-51-01):**
  ```rust
  impl Slot {
      /// Wendet den ShortDay-Cutoff auf diesen Slot an. Gibt `None` zurück,
      /// wenn der Slot komplett hinter dem Cutoff liegt (D-04 Zeile 3).
      /// Sonst einen (ggf. verkürzten) Slot-Klon. Reine Fachlogik, keine
      /// Seiteneffekte.
      pub fn clip_to(&self, cutoff: time::Time) -> Option<Slot> {
          if self.from >= cutoff { return None; }          // komplett raus
          if self.to <= cutoff { return Some(self.clone()); } // unverändert
          Some(Slot { to: cutoff, ..self.clone() })           // gekürzt
      }
  }
  ```
- **Test-Modul:** `service/src/slot.rs` hat **KEIN** eigenes
  `#[cfg(test)] mod tests` (Datei ist 130 Zeilen, endet an `line 130` mit
  `impl SlotService` — kein `mod tests`). Der Planner muss ein neues
  Test-Modul anlegen. Konvention im Repo: `#[cfg(test)] mod tests` am
  Dateiende, mit `use super::*;`.

## Injection Points (planner will convert these into tasks)

### Chain B (Read-Aggregat: WeekView + PDF)
- `service_impl/src/shiftplan.rs:53-67` — **existierender ShortDay-Filter
  ist BUG.** Ersetzen: statt `if slot.to > cutoff { continue; }` →
  `let slot = match slot.clip_to(cutoff) { None => continue, Some(s) => s
  }; ... day_slots.push(ShiftplanSlot { slot, ... })`. Der cutoff-Lookup
  an `:42-51` bleibt wie er ist.
- **Automatisch mit-korrigiert:** PDF (`service_impl/src/pdf_render.rs`),
  FE-WeekView (via `loader.rs:100-101`), FE-DayView (`loader.rs:151-155`)
  — alle lesen den `slot.to` aus dem ShiftplanSlot-Struct, das jetzt
  geclippt ist.

### Chain A' (Block-Aggregat: iCal + insufficient-booked + MyBlocks)
- `service_impl/src/block.rs:87-96` (`get_blocks_for_sales_person_week`)
  — nach dem `slot_service.get_slot(...)` (Zeile 89) SpecialDay-Lookup pro
  Woche einbauen (Muster: `reporting.rs:189-192`), dann pro
  `(booking, slot)`-Paar `slot.clip_to(cutoff_for(slot.day_of_week))`;
  `None` → das ganze Paar überspringen. Anschließend Merge-Logik wie
  gehabt.
- `service_impl/src/block.rs:237-269` (`get_unsufficiently_booked_blocks`)
  — analog: SpecialDay-Lookup, dann per-Slot-Clip **vor** dem
  `day_map.entry(...)` insertion. Merge-Kette bleibt.

### Chain C (Booking-Info-Aggregat: Weekly Summary + Conflicts)
- `service_impl/src/booking_information.rs:388-409` — der ShortDay-Filter
  (`:394-401`) muss durch Clip ersetzt werden; die `slot_hours`-
  Berechnung (`:404-409`) rechnet dann `slot.to - slot.from` auf dem
  geclippten Slot.
- `service_impl/src/booking_information.rs:506-525` — identisches Muster.
- `service_impl/src/booking_information.rs:680-697` — hier ist
  KEIN ShortDay-Filter vorgeschaltet (nur der oben in Zeile 506-521
  gefilterte `slots: Arc<[Slot]>` wird konsumiert). Ergo: wenn wir oben
  clippen, ist die per-Tag-Aggregation an :686 automatisch korrekt —
  **außer** die per-Tag-Rechnung braucht auch die "0 vs. gekürzt vs.
  voll"-Semantik pro Wochentag; hier lohnt es sich, die per-Tag-Version
  von `slots` (bereits geclippt) direkt zu verwenden.

### Chain D (SQL-Aggregat: Balance / Ist-Stunden)
- `dao_impl_sqlite/src/shiftplan_report.rs:77, 114, 147` — der aktuelle
  SQL rechnet `SUM(slot.time_to - slot.time_from)` **ohne** JOIN auf
  `special_day`. Zwei Optionen:
  - **Option A (SQL-Change):** SQL um `LEFT JOIN special_day ON
    (special_day.day_type='ShortDay' AND slot.day_of_week ==
    special_day.day_of_week AND booking.year==special_day.year AND
    booking.calendar_week==special_day.calendar_week AND
    special_day.deleted IS NULL)` erweitern, dann in `MIN(slot.time_to,
    COALESCE(special_day.time_of_day, slot.time_to))` clippen. Riskant
    (SQL-Query-Verhalten unter Snapshot-Version-12 dokumentiert; siehe
    CLAUDE.md § Billing-Period-Snapshot-Versioning). Prüfen ob
    Snapshot-Wert-Types betroffen sind.
  - **Option B (Rust-Layer):** DAO liefert Rohdaten pro Slot (`SELECT
    slot.time_from, slot.time_to, day_of_week, ...`), Rust im
    `ShiftplanReportServiceImpl` (`service_impl/src/shiftplan_report.rs`)
    aggregiert dann per `Slot::clip_to` — konsistent zu Chain B/C.
    Vorteil: eine kanonische Clip-Funktion, keine SQL-Duplikation der
    Semantik. Nachteil: möglicher Performance-Impact bei großen
    Zeit-Ranges (Balance-Historie kann mehrere Jahre umspannen).

**Für Snapshot-Immunität (D-51-03: nur zukünftig, keine
Rückrechnung):** Chain D muss zwischen "live-Read" und "historischer
Snapshot" unterscheiden können. Der Snapshot-Reader (`billing_period_
report.rs`) bleibt unverändert; nur der Live-Aggregator ändert sich.
Da Chain D den Live-Wert liefert (via ShiftplanReport, nicht via
billing_period-Snapshot), ist das automatisch korrekt.

### FE-Konsumenten (read-only, DTO-driven, minimal)
- `shifty-dioxus/src/loader.rs:101` — `to: slot.slot.to` →
  `to: slot.slot.effective_to`. Analog `:154`.
- `shifty-dioxus/src/component/week_view.rs:339-340, 417-418,
  1042-1043` — liest `slot.from_hour()` / `slot.to_hour()` vom
  gemappten `state::Slot`. Kein Change nötig, weil `loader.rs`
  bereits das geclippte `to` in `state::Slot.to` schreibt.
- `shifty-dioxus/src/page/shiftplan.rs:1374-1375` — formatiert
  `slot.from` und `slot.to` (`state::Slot`); auto-korrekt via loader.
- `shifty-dioxus/src/component/slot_edit.rs:132-133` — **Edit-Screen**;
  liest `props.slot.from/to`. Das ist der Edit-Path, muss also die
  **echten** DB-Werte anzeigen, nicht die geclippten. Empfehlung: Edit
  läuft weiterhin über das separate `SlotTO`-Endpoint (`GET /slot/{id}`),
  das kein `effective_to` hat. Ein SlotTO-Edit-Roundtrip berührt
  `effective_to` nie. Sicher.

## Risks / Gotchas Discovered

1. **CONTEXT.md D-51-05 stimmt nicht.** BlockService covert Ist-Stunden
   NICHT. Reporting/Balance geht durch `ShiftplanReportDao` (raw SQL) —
   ein vierter, in der Discuss-Phase übersehener Aggregat-Pfad. Der
   Planer muss entweder Chain D auch adressieren (Option A oder B oben)
   oder das Feature scope-reduzieren.
2. **Pre-existing Bug in `shiftplan.rs:62-66`.** Die heutige "ShortDay-
   Filter"-Logik verletzt D-04: Slots mit `slot.to > cutoff` werden
   komplett verworfen statt geclippt. Selbes Muster auch in
   `booking_information.rs:394-401` und `:512-519`. Diese Bug-Sites sind
   die zu ändernden Zeilen — d.h. Wave 2 fixt gleichzeitig einen
   bestehenden Bug + implementiert das neue Feature.
3. **SlotTO ist bidirektional.** `POST /slot` und `PUT /slot/{id}`
   akzeptieren `Json<SlotTO>` (`rest/src/slot.rs:100, 124`). Mutieren
   von `SlotTO.to` beim Read-Path würde die Slot-DB beim nächsten
   FE-Edit-Save korrumpieren. Deshalb: `effective_to` als NEUES Feld,
   nicht Mutation.
4. **Merge-Reihenfolge in BlockService kritisch.** `block.rs:131` merged
   consecutive Slots durch Vergleich `slot.from == to`. Wenn Slot A auf
   `cutoff` geclippt wird und Slot B danach kommt mit `B.from > cutoff`
   (also `B` wird ganz verworfen), ist der Merge automatisch korrekt.
   Aber: Wenn Slot A auf `cutoff` geclippt wird und Slot B `from ==
   originalA.to != cutoff` hat, bricht die "consecutive"-Detection.
   Das ist semantisch ok (der Block endet bei Cutoff), muss aber im Test
   abgedeckt sein.
5. **`MyBlockService` (`service/src/my_block.rs`) ist dead code.** Trait
   ohne Impl, keine Konsumenten (`grep -rln MyBlockService` in
   service_impl/rest/shifty_bin → leer). CONTEXT.md verweist auf ihn als
   Konsequenz von D-51-05, aber der reale Endpoint (`rest/src/
   my_block.rs`) delegiert an `BlockService::get_blocks_for_current_user`
   direkt. Kein separater Injection-Point nötig; nur die eine
   BlockServiceImpl-Änderung reicht.
6. **SpecialDay time_of_day ist `Option<time::Time>`.** Selbst bei
   `day_type == ShortDay` kann `time_of_day == None` sein (die Insert-
   Validation `service_impl/src/special_days.rs:133` skippt das nur, ist
   aber nicht garantiert für Legacy-Daten). Der Clip-Konsument muss
   `.time_of_day.and_then(...)` verwenden — Slot-Filter bei
   `slot.time_of_day == None` == keine Kürzung (== D-04 unverändert).
7. **PDF-Snapshot-Tests werden möglicherweise brechen.** Der v2.3-PDF-
   Renderer hat deterministische Fixtures (`empty_week` etc. in
   `pdf_render.rs:689+`). Sobald `build_shiftplan_day` ShortDay-Clip
   sauber macht, ändern sich Test-Woche-Aggregate für Tests, die
   ShortDays enthalten. Der einzige aktuelle Test mit ShortDay
   (`service_impl/src/test/shiftplan.rs:251` — `test_get_shiftplan_week_
   with_special_days`) muss auf D-04-Erwartung angepasst werden (Slot
   sollte gekürzt in der Response landen statt komplett zu fehlen).
