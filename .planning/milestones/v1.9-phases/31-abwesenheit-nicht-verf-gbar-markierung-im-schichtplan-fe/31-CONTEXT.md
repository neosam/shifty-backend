# Phase 31: Abwesenheit → Nicht-Verfügbar-Markierung im Schichtplan (FE) - Context

**Gathered:** 2026-06-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Tage mit Abwesenheits-Zeiträumen der **im Schichtplan aktuell gewählten Person**
(`current_sales_person`) erscheinen im Schichtplan-Grid **proaktiv** als „Nicht
Verfügbar" (discourage) — **bevor** eine Buchung versucht wird, nicht erst als
nachträgliche `BookingOnAbsenceDay`-Warnung. Reiner Frontend-Join über den bestehenden
Absence-Period-Endpoint; **kein** Backend, **kein** neues API/DTO.

**Kerngedanke (User-Vorgabe):** Der Absence-Marker soll sich **EXAKT verhalten wie ein
hinterlegter Unavailable-Tag** (`sales_person_unavailable`). Gleiche Quell-Person, gleiche
discourage-Darstellung, in dasselbe `discourage_weekdays` gemergt — nur dass die Quelle
Datums-Bereiche (Absences) statt wochentag-wiederkehrender Muster sind.

</domain>

<decisions>
## Implementation Decisions

### Kategorie-Whitelist (SHP-01, D-NN aufgelöst)
- **D-31-01:** Die Markierung wird durch **alle drei** `AbsenceCategory`-Varianten
  ausgelöst — **Vacation + SickLeave + UnpaidLeave** — und **nur für Ganztags-Absences**;
  **Halbtags-Absences (`day_fraction == Half`) lösen KEINE Markierung aus**. Das ist
  **exakt** das Set der bestehenden `BookingOnAbsenceDay`-Warnung
  (`service_impl/src/shiftplan_edit.rs:530-545`: cross-Kategorie-Lookup
  `find_overlapping_for_booking`, einziger Filter `if ap.day_fraction == Half { continue }`).
  → **Null Kategorie-Drift** zwischen Discourage-Marker und Post-Booking-Warnung
  (ROADMAP SC2). User-Entscheidung: „Alle 3, exakt wie die Buchungs-Warnung".

### Personen-Scope (User-Vorgabe, präzisiert die offene Frage)
- **D-31-02:** Der Marker wird für **`current_sales_person`** geladen und angezeigt —
  also für **dieselbe Person, deren `unavailable_days` heute schon den discourage-Marker
  treiben**. `current_sales_person` ist der eingeloggte User als Default
  (`shiftplan.rs:349` `load_current_sales_person`), wird aber per Personen-Dropdown des
  Schichtplaners umgestellt (`shiftplan.rs:538` write, `:838-842` select). Damit sehen
  **sowohl der Schichtplaner (wenn er die Person auswählt) als auch die Person selbst**
  die Markierung — wörtlich die User-Vorgabe „EXAKT so wie ein Unavailable-Tag". **Kein**
  „alle Personen der Woche"-Scope (out of scope, würde Last + Semantik sprengen).

### Mirror-Architektur (analog `reload_unavailable_days`)
- **D-31-03:** Neuer Async-Pfad `reload_absence_days` **analog zu `reload_unavailable_days`**
  (`shiftplan.rs:355-381`): lädt die Absences der `current_sales_person` für die angezeigte
  Woche über den bestehenden Absence-Period-Endpoint, schreibt ein neues
  `person_absences`-Signal. Aufgerufen an **denselben Triggern** wie
  `reload_unavailable_days` (Initial-Load + Personen-Wechsel + NextWeek/PreviousWeek).
- **D-31-04 (pure Helfer):** Reine, per `cargo test` testbare Funktion
  `absence_periods_to_discourage_days(person_absences, week_dates) -> Vec<Weekday>` (genaue
  Signatur Planner-Discretion): für die **konkreten Daten der angezeigten Woche** ermittelt
  sie, welche Wochentage von einer Ganztags-Absence der drei Kategorien getroffen sind.
  Da `discourage_weekdays` ein `Vec<Weekday>` (day-of-week) ist und das Grid genau **eine**
  Woche zeigt, ist die Abbildung Datum→Wochentag-der-Woche eindeutig.
- **D-31-05 (Merge):** Das Ergebnis wird in `discourage_weekdays` (`shiftplan.rs:1138-1142`)
  **mit** den bestehenden `unavailable_days`-Wochentagen **vereinigt** (Union, keine
  Ersetzung) — WeekView bekommt weiterhin `Vec<Weekday>`, gleiche rote „Nicht
  Verfügbar"-Zelle (`week_view.rs:975-1065`), keine neue UI, kein neuer Marker-Typ.

### Stale-Guard (Abhängigkeit auf Phase 30)
- **D-31-06:** Der neue `reload_absence_days`-Pfad MUSS durch den **Phase-30-`(year,week)`-
  Guard** geschützt werden (ROADMAP SC3): req-`(year,week)` **vor** dem `await` capturen,
  nach dem `await` nur schreiben wenn `is_current_selection((req_year,req_week),
  *SELECTED_WEEK.read())` — exakt das Muster der vier bestehenden Loader aus Phase 30
  (`service/week_guard.rs`). `set_selected_week` wird bereits synchron vor allen Dispatches
  gesetzt; der neue Pfad reiht sich dort ein. Beim schnellen Wochenwechsel bleibt der
  Absence-Marker konsistent.

### Claude's Discretion
- Exakter Loader/Endpoint für die Person-Absences pro Woche: der Planner liest den Code
  (`/absence-period`-Endpoint, ggf. bestehender FE-`loader::`-Helfer; Backend aggregiert
  Absences bereits pro Jahr/Woche in `booking_information.rs:70-99`). Empfehlung: kleinster
  Join, der die Absences der `current_sales_person` für die Woche liefert.
- Ob `person_absences` die rohen AbsencePeriods hält und der Helfer filtert, oder ob schon
  beim Laden gefiltert wird — Planner-Discretion, solange die Filter-Logik (3 Kategorien,
  nur Ganztags) **eine** Stelle ist (kein Drift) und testbar bleibt.
- Half-day-Filter: `day_fraction != Half` muss in derselben pure-Helfer-Logik sitzen wie
  die Kategorie-Prüfung (Single Source, mirrort `shiftplan_edit.rs:538`).

### Folded Todos
- **„Eigener Urlaub markiert nicht als Nicht-Verfügbar im Schichtplan"**
  (`.planning/todos/pending/2026-06-29-eigener-urlaub-markiert-nicht-als-nicht-verfuegbar-im-schich.md`,
  `resolves_phase: 31`). Verifizierte Ursache: `discourage_weekdays` (`shiftplan.rs:1120`)
  speist sich nur aus `unavailable_days` (sales_person_unavailable), Absences fließen nicht
  ein. Folded → durch D-31-01..06 vollständig abgedeckt. Der User bestätigte: war
  ursprünglich Teil des Scopes (Bug/Lücke, kein Neufeature).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirement & Roadmap
- `.planning/REQUIREMENTS.md` §SHP-01 — proaktive discourage-Markierung, FE-Join,
  `reload_absence_days` + `person_absences` + Helfer + Merge in `discourage_weekdays`.
- `.planning/ROADMAP.md` §"Phase 31" — Goal + Success Criteria 1–3.

### Code — Konsistenz-Anker (Single Source of Truth für SC2, NUR lesen)
- `service_impl/src/shiftplan_edit.rs:485-546` — die bestehende `BookingOnAbsenceDay`-
  Emission: `find_overlapping_for_booking` (cross-Kategorie) + `day_fraction == Half`-Skip.
  Die neue Marker-Filterlogik MUSS dieses Set spiegeln (D-31-01).
- `service/src/warning.rs:26` + `rest-types/src/lib.rs:1828` — `BookingOnAbsenceDay`-Shape
  (trägt `category`).

### Code — die zu ändernden/spiegelnden Stellen (FE)
- `shifty-dioxus/src/page/shiftplan.rs` — `current_sales_person`-Signal (`:199`, init
  `:349`, person-switch `:538`/`:838-842`); `reload_unavailable_days`-Closure (`:355-381`,
  das Muster für `reload_absence_days`); `discourage_weekdays`-Bau (`:1138-1142`, der
  Merge-Punkt); die 3 Dispatch-Trigger (Initial `:330+`, NextWeek `:471-478`, PreviousWeek
  `:506-513`).
- `shifty-dioxus/src/service/week_guard.rs` — `SELECTED_WEEK` + `is_current_selection`
  (Phase 30), der Guard für den neuen Pfad (D-31-06).
- `shifty-dioxus/src/state/absence_period.rs` — `AbsenceCategory` (3 Varianten) +
  AbsencePeriod-State-Typ; `day_fraction`-Feld für den Half-Filter.
- `shifty-dioxus/src/component/week_view.rs:975-1065` — wie `discourage_weekdays` zur roten
  Zelle wird (NUR lesen; keine Änderung am Visual).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets / Mirror Targets
- `reload_unavailable_days` (`shiftplan.rs:355-381`): exaktes Muster für `reload_absence_days`
  (Closure, capture-before-await, Guard-Write). Nach Phase 30 ist es bereits guarded.
- `current_sales_person`-Signal: die EINE Quell-Person für beide Marker (unavailable +
  absence). Wird beim Personen-Wechsel (`:538`) neu gesetzt → der Absence-Reload muss auch
  an diesem Trigger feuern (D-31-03).
- `week_guard::is_current_selection` + `SELECTED_WEEK` (Phase 30): wiederverwenden (D-31-06).

### Established Patterns
- `discourage_weekdays` ist `Vec<Weekday>` (day-of-week), WeekView markt ganze
  Wochentag-Spalten. Absences (Datums-Bereiche) werden für die EINE angezeigte Woche auf
  Wochentage abgebildet (eindeutig, da Grid = 1 Woche).
- Pure-Helfer-Testbarkeit (analog Phase 29 `compute_vacation_bar` / Phase 30
  `is_current_selection`): die Datum→Wochentag-+-Kategorie-+-Halbtag-Logik als reine
  Funktion → `cargo test` (Browser/SSR-Tests laut Memory unzuverlässig).

### Integration Points
- Ein neues Signal `person_absences`, ein neuer Closure `reload_absence_days`, ein pure
  Helfer, ein Union-Merge an `shiftplan.rs:1138-1142`. Kein neuer WarningTO, keine neue
  WeekView-Prop-Semantik, kein Backend.

</code_context>

<specifics>
## Specific Ideas

Akzeptanz-Szenario: User/ausgewählte Person hat Vacation Di–Do dieser Woche → Di/Mi/Do
erscheinen rot „Nicht Verfügbar" im Grid, bevor gebucht wird — identisch zu einem
hinterlegten Unavailable-Tag. SickLeave/UnpaidLeave (ganztags) genauso. Eine Halbtags-
Vacation → KEINE Markierung (Mitarbeiter arbeitet die andere Hälfte), konsistent zur
Buchungs-Warnung. Schichtplaner wählt im Dropdown eine andere Person → deren Absence-Tage
erscheinen.

</specifics>

<deferred>
## Deferred Ideas

- **Hover-Tooltip auf der discourage-Zelle (Absence-Typ + Daten)** — in REQUIREMENTS.md als
  Future deferred. Nicht in Phase 31 (nur die Markierung selbst).
- **„Alle Personen der Woche"-Scope** — bewusst verworfen (D-31-02): passt nicht zur
  Buchungssicht-Semantik, sprengt Scope/Last.

### Reviewed Todos (not folded)
- Keine weiteren — Discuss blieb im Phasen-Scope.

</deferred>

---

*Phase: 31-Abwesenheit → Nicht-Verfügbar-Markierung im Schichtplan (FE)*
*Context gathered: 2026-06-29*
