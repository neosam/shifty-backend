# Phase 53: freiwilligen-abwesenheiten-jahresansicht — Context

**Gathered:** 2026-07-06
**Status:** Ready for planning
**Mode:** discuss (Textform, 6 Gray Areas)

<domain>
## Phase Boundary

Ergänzung der Jahresübersicht (`/weekly_overview/` Frontend, Backend
`GET /booking-information/weekly-resource-report/{year}` +
`get_summery_for_week`) um sichtbare **Freiwilligen-Absencen**. Freiwillige
mit aktiver `Vacation` / `SickLeave` / `UnpaidLeave`-Absence-Period, die
eine Kalenderwoche überlappt, tauchen in der Wochen-Absencen-Zeile
namentlich auf — zusammen mit dem Stunden-Wert ihrer weggefallenen
Wochen-Zusage.

**Fat Backend, Thin Client (D-51-02-Präzedenz):** Backend liefert Name +
Stunden fertig geformt im DTO. Der bestehende FE-Merge in
`state::WeeklySummary::from(WeeklySummaryTO)` — der heute
`sales_person_absences` aus `absence_hours − holiday_hours +
unavailable_hours` je `WorkingHoursPerSalesPerson`-Zeile rekonstruiert —
wird durch reines DTO-Lesen ersetzt.

**Baut auf Phase 52 auf:**
- `all_absences` (`AbsenceService::find_all`, kategorie-agnostisch) ist
  bereits load-once in `get_weekly_summary` geladen.
- `absent_volunteer_ids: HashSet<Uuid>` wird pro Woche berechnet
  (VFA-01 whole-week-out) — genau der Anker-Punkt für VAA-Anzeige.
- `sales_person_service.get_all()` → Freiwilligen-Namen bereits verfügbar.

**Nicht-Ziele (aus REQUIREMENTS.md v2.5 + Discuss-Entscheidungen):**
- Keine Änderung an der Verfügbarkeits-Berechnung — VFA-01 whole-week-out
  bleibt unangetastet, VAA erweitert **reine Anzeige**.
- Kein Snapshot-Schema-Bump (bleibt 12), keine Migration, kein neuer
  Cargo-Dep.
- Kein Frontend-Redesign — nur Feld-Umverdrahtung, keine neuen Farben /
  Icons / Suffixe (G4).
- Keine Änderung an `WorkingHoursPerSalesPerson` (bleibt bezahlten-only) —
  Freiwillige gehen in ein neues, dediziertes Feld (G1).
- Keine Kategorie-Trennung im DTO (Vacation vs. SickLeave vs. UnpaidLeave
  landet in einer flachen Liste — genau wie VFA-01 kategorie-agnostisch
  arbeitet).
</domain>

<spec_lock>
## Locked Requirements (REQUIREMENTS.md v2.5 §"Freiwilligen-Abwesenheiten (VAA)")

**MUST READ vor Planning:** `.planning/REQUIREMENTS.md` Zeilen 100–128.

- **VAA-01:** Freiwillige mit aktiver Vacation/SickLeave/UnpaidLeave-
  Absence-Period in Woche N erscheinen zusätzlich zu bezahlten
  Mitarbeitern in der Wochen-Absencen-Anzeige. Backend liefert Name +
  Stunden-Wert fertig im DTO. Kein FE-Merge.
- **VAA-02:** Angezeigter Stunden-Wert = `committed_voluntary` der Person
  für die Woche, cap-gated (siehe D-53-02). In discuss-phase fixiert.
- **VAA-03:** Backend-Test verifiziert:
  1. Freiwilliger mit Vacation-Period überlappt Woche N → erscheint mit
     dem in VAA-02 fixierten Stunden-Wert.
  2. Freiwilliger ohne aktive Period → **nicht** in der Liste.
  3. Bezahlter Mitarbeiter bleibt unverändert (Regression-Lock).
- **VAA-04:** FE-Rendering (`page/weekly_overview.rs:121` — Zeile
  `"{name}: {hours} h"`) rendert Freiwilligen-Einträge visuell
  konsistent mit bezahlten (keine Farbe / Icon / Suffix — G4 entschieden:
  „ein Topf, sortiert nach Name"). Kein Redesign.
</spec_lock>

<decisions>
## Implementation Decisions

### G1 — Backend-Träger: Neues DTO-Feld

- **D-53-01 (G1-a):** Neues Feld `sales_person_absences: Arc<[SalesPersonAbsence]>`
  auf `service::booking_information::WeeklySummary` **UND**
  `rest_types::WeeklySummaryTO`. Struct-Neuling in **beiden** Ebenen:

  ```rust
  // service/src/booking_information.rs
  #[derive(Clone, Debug, PartialEq)]
  pub struct SalesPersonAbsence {
      pub sales_person_id: Uuid,
      pub name: Arc<str>,
      pub hours: f32,
  }

  // rest-types/src/lib.rs
  #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
  pub struct SalesPersonAbsenceTO {
      pub sales_person_id: Uuid,
      pub name: Arc<str>,
      pub hours: f32,
  }
  ```

  Backend füllt die Liste im Assembly. FE-`state::WeeklySummary::from()`
  ersetzt den bestehenden Merge-Loop durch reines Kopieren des Felds. Der
  Freiwilligen-Fall wird über das neue Feld getragen; bezahlte
  Mitarbeiter fließen **weiterhin** über `working_hours_per_sales_person`
  → FE-Mapper baut die Anzeige-Liste durch **Union** beider Quellen
  (siehe D-53-05 FE-Merge).

  **Begründung:** REQUIREMENTS.md nennt „sales_person_absences der
  Jahresansicht" als DTO-Feld explizit. Diese Variante ist die einzige,
  die den Fat-Backend-Thin-Client-Anspruch strukturell erfüllt und
  gleichzeitig `WorkingHoursPerSalesPerson` (bezahlten-only-Kontrakt mit
  vacation_hours/sick_leave_hours/holiday_hours/…) semantisch sauber
  hält.

### G2 — Angezeigter Stunden-Wert: `committed_voluntary` cap-gated

- **D-53-02 (G2-a):** Der pro Freiwilligen angezeigte Stunden-Wert ist
  identisch zur cap-gated Wochen-Zusage, die durch VFA-01 aus
  `committed_voluntary_hours` (Band 1) wegfällt:

  ```
  hours = Σ over active EmployeeWorkDetails rows of person p for (year, week) where
              wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0
          of wh.committed_voluntary
  ```

  **Präzedenzquelle:** identische Formel wie in
  `booking_information.rs:495-503` (Berechnung von
  `committed_voluntary_hours`, Zeile 500 im aktuellen Code) — nur
  **ohne** den `!absent_volunteer_ids.contains(...)`-Filter, weil hier
  gerade die Abwesenden gemeint sind.

  **Randfall Zusage=0:** Freiwilliger ohne cap-gated Zusage bekommt
  `hours = 0.0`. Whether die Zeile trotzdem angezeigt wird, entscheidet
  D-53-04.

  **Kategorie-Agnostik (mitgeerbt von VFA-01):** Der Wert ist derselbe
  egal ob Vacation, SickLeave oder UnpaidLeave — das ist konsistent mit
  `find_all` und dem bestehenden VFA-01-Whole-Week-Out (D-26-01).

### G3 — Whole-Week-Out spiegeln

- **D-53-03 (G3-a):** Sichtbarkeitskriterium **exakt identisch** zu
  VFA-01 (D-26-01): `absent_volunteer_ids` — die pro-Woche berechnete
  Menge (bereits im Code, `booking_information.rs:421-444`) — ist genau
  die Menge, die im DTO-Feld auftaucht. Keine partielle Anteiligkeit.

  **Konsequenz:** Ein Freiwilliger mit nur einem Absence-Tag in der
  Woche taucht mit vollem Wochenwert `committed_voluntary` auf — genauso
  wie VFA-01 diese Zusage voll aus Band 1 herausrechnet. Symmetrie
  zwischen Berechnung und Anzeige.

### G4 — Visuelle Präsentation: Ein Topf, keine Markierung

- **D-53-04 (G4-a):** Backend-Feld `sales_person_absences` wird **nicht
  sortiert** (Insertion-Order). FE-Anzeige-Reihenfolge:

  1. FE-Mapper baut eine Union-Liste aus:
     - bestehenden bezahlten Absencen aus `working_hours_per_sales_person`
       (existierender Merge-Code — bleibt für Bezahlte erhalten, siehe
       D-53-05)
     - neuen Freiwilligen-Absencen aus dem neuen DTO-Feld
  2. FE sortiert die Union nach `name` (case-insensitive, Locale-agnostisch —
     einfache `str::to_lowercase().cmp(...)` reicht, kein neuer Dep).
  3. Keine visuelle Unterscheidung Freiwilliger/Bezahlt (kein Suffix,
     kein Icon, keine Farbe). „Visuell konsistent" per VAA-04.

  **Randfall Zusage=0:** Ein Freiwilliger mit `hours = 0.0` würde vom
  bestehenden FE-Filter `effective_absence >= 0.1` weggefiltert — was
  gewünscht ist: Freiwillige ohne Zusage sind für die Verfügbarkeits-
  Wahrnehmung uninteressant. Backend liefert die Zeile trotzdem (Fat
  Backend, thin Filter), FE filtert.

### G5 — Fill-Site: `get_weekly_summary` (Assembly-Loop) direkt

- **D-53-05 (G5-c → a):** Das neue Feld wird direkt in
  `BookingInformationServiceImpl::get_weekly_summary`
  (`service_impl/src/booking_information.rs:267`) im
  Assembly-Loop gefüllt, **NICHT** in `assemble_weeks`
  (`service_impl/src/reporting.rs`).

  **Begründung:** D-52-09 (Phase 52 MUST-preserve) hält
  `assemble_weeks` bewusst clean von Business-Logic-Cross-Cutting
  (Toggle-Read, Slot-Filter, VFA-01). Freiwilligen-Absencen-Anzeige ist
  ein `BookingInformationService`-Concern (Endpoint-DTO-Assembly),
  keine `ReportingService`-Semantik. `assemble_weeks` bleibt reines
  Wochen-Report-Aggregat pro Person.

  **Konsumierte Inputs** (alle bereits im Loop verfügbar):
  - `absent_volunteer_ids: HashSet<Uuid>` → Filter „nur Abwesende"
  - `sales_person_service.get_all()` Result (`volunteer_ids`-Quelle) →
    Name + `id`-Lookup
  - `all_work_details` (load-once seit Phase 52) → `committed_voluntary`-
    Berechnung via `find_working_hours_for_calendar_week` (bestehende
    Helper-Funktion)

  **FE-Mapper (D-53-01-Folgeschritt):** Der bestehende Loop in
  `state::WeeklySummary::from()` (Zeilen 47–62) baut für Bezahlte weiter
  die effective-absence-Liste (`absence_hours − holiday_hours +
  unavailable_hours >= 0.1`). Für Freiwillige kommt eine zusätzliche
  Iteration über das neue DTO-Feld — Union-Merge, sortieren nach Name
  (D-53-04). Bestehender FE-Filter bleibt für Bezahlte unverändert
  (Regression-Lock VAA-03 #3).

### G6 — `get_summery_for_week` mitziehen

- **D-53-06 (G6-a):** `BookingInformationServiceImpl::get_summery_for_week`
  (`service_impl/src/booking_information.rs:643` — die Single-Week-
  Variante) füllt das neue Feld analog. Motivation: Wenn beide
  Endpoints denselben `WeeklySummary`-Typ zurückgeben (was sie tun),
  muss das Feld an beiden Fill-Sites ehrlich befüllt werden — sonst
  entstünde eine stille DTO-Semantik-Inkonsistenz.

  **Kosten:** ~15–25 LOC — die Single-Week-Methode berechnet
  `absent_volunteer_ids` noch nicht (VFA-01 wurde für die Jahresansicht
  eingebaut, die Wochen-Methode lief davor separat). Der Planner muss
  entscheiden: kleiner In-Line-Bau der Menge für diese eine Woche, oder
  Extraktion in einen kleinen `absent_volunteer_ids_for_week`-Helper,
  den beide Methoden konsumieren. Empfehlung: In-Line für Wave 1,
  Helper-Extraktion als optionaler Cleanup wenn Duplikation zu groß
  wird.
</decisions>

<code_context>
## Reusable Assets

### Backend

- `service_impl/src/booking_information.rs:267–638` — `get_weekly_summary`:
  Fill-Site für D-53-05. Bereits verfügbar im Loop: `absent_volunteer_ids`,
  `volunteer_ids`, `all_work_details`, `find_working_hours_for_calendar_week`.
- `service_impl/src/booking_information.rs:643+` — `get_summery_for_week`:
  Zweiter Fill-Site für D-53-06.
- `service_impl/src/booking_information.rs:495–503` — Präzedenzformel für
  `committed_voluntary_hours` (D-53-02 baut identisch, ohne
  Absenten-Filter).
- `service/src/booking_information.rs:24–35` — `WorkingHoursPerSalesPerson`
  bleibt unberührt (bezahlten-only).
- `service/src/absence.rs` — `AbsencePeriod` mit `category` (kategorie-
  agnostischer Konsum via `all_absences` erledigt).

### DTO

- `rest-types/src/lib.rs:956–1035` — `WorkingHoursPerSalesPersonTO`,
  `WeeklySummaryTO`, `From<&WeeklySummary>`-Impl. Feld-Erweiterung hier.

### Frontend

- `shifty-dioxus/src/state/weekly_overview.rs:5–65` — `SalesPersonAbsence`
  (FE-Typ) + `From<&WeeklySummaryTO>`-Mapper. D-53-05-Folgeschritt: Union-
  Merge aus `working_hours_per_sales_person` + neuem DTO-Feld, Sort by
  name.
- `shifty-dioxus/src/page/weekly_overview.rs:121–130` — Rendering-Zeile
  bleibt **wörtlich unverändert** (`"{name}: {hours} h"`). VAA-04.
- `shifty-dioxus/src/loader.rs:533` + `component/weekly_overview_chart.rs:188`
  + `page/weekly_overview.rs:247,429` — weitere Konstruktions-Sites von
  `WeeklySummary`/`SalesPersonAbsence` (kein aktueller Cargo-Test müsste
  crashen, aber Planner muss FE-Type-Init-Sites auf leere Vec / Default
  erweitern, wo nötig).
</code_context>

<canonical_refs>
## Canonical References (MUST READ before planning)

- **`.planning/REQUIREMENTS.md`** — VAA-01..04 (Zeilen 100–128).
- **`.planning/ROADMAP.md`** — Phase-53-Definition mit 5 Success
  Criteria (Zeilen 75–91).
- **`.planning/phases/52-weekly-overview-performance-refactor/52-CONTEXT.md`** —
  Phase-52-Assembly-Kontext, Load-once-Muster, D-52-09 MUST-preserve für
  `assemble_weeks`.
- **`service/src/booking_information.rs`** — Trait + `WeeklySummary`/
  `WorkingHoursPerSalesPerson` Structs (Feld-Erweiterung).
- **`service_impl/src/booking_information.rs`** — beide Fill-Sites
  (`get_weekly_summary` + `get_summery_for_week`).
- **`rest-types/src/lib.rs`** — DTO-Ebene (`WeeklySummaryTO` +
  `WorkingHoursPerSalesPersonTO` + neue `SalesPersonAbsenceTO`).
- **`shifty-dioxus/src/state/weekly_overview.rs`** — FE-State-Mapper
  Union-Refactor.
- **`shifty-dioxus/src/page/weekly_overview.rs`** — FE-Render (bleibt).
- **`service/src/absence.rs`** — `AbsencePeriod` + Kategorien
  (Vacation/SickLeave/UnpaidLeave).
- **`docs/features/F07-reporting-balance.md`** (+ `_de.md`) — Docs-
  Freshness-Gate: Balance-Formel + Kategorie-Semantik. Prüfen ob
  VAA-01/02 Sichtbarkeit dort erwähnt werden muss (Empfehlung: nur ein
  kurzer Nebensatz, DTO-Erweiterung ist Additiv).
- **Docs-Freshness-Trigger:** DTO-Feld-Erweiterung → keine der harten
  Trigger-Dateien (kein `migrations/`, kein `permission.rs`,
  kein Schema-Bump). `docs/features/F08` oder ähnlich muss NUR touched
  werden, falls Anzeige-Semantik dokumentiert ist — Planner prüft.
</canonical_refs>

<deferred>
## Noted for Later

Keine — Discuss-Phase war fokussiert, kein Scope-Creep aufgetreten.

**Optionale Follow-ups** (nicht scope-blockierend):
- FE-Filter-Threshold `>= 0.1` in `state/weekly_overview.rs:53` könnte auf
  eine Konstante gehoben werden, wenn er auch für Freiwillige gilt. Kein
  Muss.
- `absent_volunteer_ids_for_week`-Helper-Extraktion aus D-53-06 —
  optional wenn Duplikation zwischen `get_weekly_summary` und
  `get_summery_for_week` unschön wird.
</deferred>

<next_steps>
## Next Steps

`/clear` then:

`/gsd-plan-phase 53`

**Also available:** `--chain` for auto plan+execute after; review/edit this
CONTEXT.md before continuing.
</next_steps>
