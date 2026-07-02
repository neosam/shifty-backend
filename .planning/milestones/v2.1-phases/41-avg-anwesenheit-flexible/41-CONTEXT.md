# Phase 41: Ø-Anwesenheit bei flexiblen Stunden (BE+FE) - Context

**Gathered:** 2026-07-02
**Status:** Ready for planning

<domain>
## Phase Boundary

HR sieht im **Mitarbeiter-Report-Detail** eine **neue, tagebasierte Kennzahl** für
**flexible** Mitarbeiter (`EmployeeWorkDetails.is_dynamic == true`):

> **Ø Stunden pro Anwesenheitstag** = Summe geleistete Stunden ÷ Anzahl Anwesenheitstage,
> berechnet über den **angezeigten Report-Zeitraum**.

Die Zahl beantwortet „wie lang arbeitet der flexible MA an den Tagen, an denen er
tatsächlich da ist" — bewusst **anders** als die bereits existierende A-22-1-Zahl
„Ø geleistete Stunden/Woche". Urlaub (und jede andere Absence) ist per Konstruktion aus
dem Nenner heraus, weil nur echte Arbeitstage gezählt werden (Goal „Urlaub aus dem Nenner"
erfüllt).

**In Scope (BE):** Neue **pure Aggregat-Funktion** in `service/src/reporting.rs` (analog,
aber getrennt von `average_worked_hours_per_week` — A-22-1 wird NICHT geändert und NICHT
blind wiederverwendet). Reines **Read-Aggregat** im `ReportingService` (Business-Logic-Tier),
das über den Report-Zeitraum aggregiert. **HR-gated** REST-Endpoint (`HR_PRIVILEGE`,
`#[utoipa::path]`), analog zum bestehenden `/{id}/weekly-statistics`. Server-seitiger
`is_dynamic`-Filter: für nicht-flexible MA wird die Kennzahl weder berechnet noch geliefert.

**In Scope (FE):** Neue Zahl im **bestehenden HR-Statistik-Bereich** des Reports
(`shifty-dioxus/src/component/employee_view.rs`), **neben** der vorhandenen „Ø Std/Woche"-Zahl.
Leerzustand bei zu wenig Daten. i18n de/en/cs (Label, Tooltip, Leerzustand).

**Out of Scope:** Kein neuer `BillingPeriodValueType`, kein Snapshot-Bump
(`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **12**), keine neue Persistenz, keine Migration.
Keine Multi-MA-Übersichtsliste (Kennzahl ist pro-MA im Report). Kein separater
Datums-/Zeitraum-Picker (Zeitraum = der ohnehin angezeigte Report-Zeitraum). AVG-Trend über
mehrere Perioden (AVG-04) und konfigurierbare Exclusion-Kategorien (AVG-05) sind Backlog.
Änderung an A-22-1 selbst.
</domain>

<decisions>
## Implementation Decisions

### Kennzahl-Definition (AVG-01, D-AVG-02)
- **D-AVG-01 (Kennzahl):** **Ø Stunden pro Anwesenheitstag** = `Σ geleistete Stunden /
  Anzahl Anwesenheitstage` über den Report-Zeitraum. Zähler = geleistete Stunden
  (Arbeitskategorien, siehe D-AVG-03), Nenner = Anzahl Anwesenheitstage.
  Bewusst **verschieden** von der bestehenden „Ø Std/Woche" (A-22-1), damit keine redundante
  zweite Wochen-Stundenzahl entsteht — der User wollte explizit eine **tagebasierte** Größe.

### Anwesenheitstag & Nenner-Regel (AVG-01, D-AVG-03)
- **D-AVG-02 (Anwesenheitstag):** Ein **Kalendertag mit tatsächlicher Arbeit** — mindestens
  ein Tages-Eintrag der Kategorie `Shiftplan`, `ExtraWork` oder `VolunteerWork` mit
  `hours > 0`. Wochentag ist egal (auch Wochenende zählt, falls gearbeitet).
- **D-AVG-03 (Exclusion by construction):** Tage der Kategorien `Vacation`, `SickLeave`,
  `Holiday`, `UnpaidLeave`, `Unavailable` sind **keine** Anwesenheitstage und damit **nicht
  im Nenner**. „Urlaub aus dem Nenner herausgerechnet" (Goal) ist so per Konstruktion erfüllt
  — die ursprünglich offene Frage „nur Urlaub vs. alle Absence-Kategorien ausschließen" löst
  sich auf: es zählen ausschließlich echte Arbeitstage, jede Absence bleibt draußen.
  Anteilige/wochenweise Sonderfälle entfallen, weil rein tagebasiert gerechnet wird.

### Zeitraum (AVG-01, D-AVG-01)
- **D-AVG-04 (Zeitraum):** Aggregation über den **angezeigten Report-Zeitraum** (die
  `from`/`to`-Spanne, die der Report ohnehin darstellt). **Kein** separater Datums-Picker,
  keine eigene Perioden-Auswahl.

### Mitarbeiter-Scope (AVG-01)
- **D-AVG-05 (Scope):** Nur Mitarbeiter mit `is_dynamic == true`. Server-seitiger Filter —
  für nicht-flexible MA wird die Kennzahl weder berechnet noch ausgeliefert (erscheint im FE
  nicht). Nicht-HR-Rollen erhalten keinen Zugriff (`HR_PRIVILEGE`, analog `weekly-statistics`).

### Mindest-Datenschwelle & Leerzustand (AVG-02, AVG-03)
- **D-AVG-06 (Schwelle):** Bei **< 2 Anwesenheitstagen** im Zeitraum wird **keine Zahl**
  gezeigt, sondern ein „nicht aussagekräftig"/Leerzustand (1 Tag ⇒ sinnloser Durchschnitt).
  Gleicher Leerzustand-Pfad gilt, wenn im Zeitraum gar keine Arbeitstage vorliegen.

### Anzeige-Ort (AVG-02)
- **D-AVG-07 (Ort):** Im **bestehenden HR-Statistik-Bereich** des Mitarbeiter-Reports
  (`employee_view.rs`, hinter `should_show_hr_stats`), **neben** der vorhandenen
  „Ø Std/Woche"-Zahl. Keine eigenständige Route/Sicht.

### Persistenz / Snapshot (AVG-02, Success-Criterion 4)
- **D-AVG-08 (No-Persist):** Reines Read-Aggregat. **Kein** neuer `BillingPeriodValueType`,
  **kein** Snapshot-Bump (`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **12**), keine Persistenz,
  keine Migration. Neue **eigene** pure Funktion; A-22-1 (`average_worked_hours_per_week`)
  bleibt unangetastet. In der Plan-/Execute-Phase grep-/test-verifizieren, dass die Version
  unverändert 12 ist.

### i18n (AVG-03)
- **D-AVG-09 (i18n):** Label, Tooltip und Leerzustand der neuen Kennzahl in **de/en/cs**.

### Claude's Discretion
- Exakte Wiederverwendung von Report-Daten: Die Tages-Daten stehen in
  `GroupedReportHours.days: Arc<[WorkingHoursDay]>` (pro Woche), die Report-Range liefert
  `get_report_for_employee_range`. Ob die neue Aggregat-Funktion direkt auf dem
  `EmployeeReport`/`by_week.days` arbeitet oder über eine neue schlanke Struktur — Planner
  entscheidet, solange A-22-1 unberührt bleibt.
- Ob die Zahl über den bestehenden `/{id}/weekly-statistics`-Endpoint (um ein Feld erweitert,
  range-aware) oder einen neuen Endpoint geliefert wird — Planner entscheidet. Beachten:
  der bestehende Endpoint/`EmployeeWeeklyStatistics` ist heute „Jahr bis heute", die neue
  Zahl braucht den Report-Zeitraum → ggf. sauberer als eigener range-aware Endpoint.
- Rundung/Formatierung der Std-Zahl (bestehendes `format_hours(..)` nutzen).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap
- `.planning/ROADMAP.md` §"Phase 41" — Goal, Success Criteria, Scope, offene Entscheidungen
- `.planning/REQUIREMENTS.md` — AVG-01/02/03 (Zeilen 69–78), Out-of-Scope AVG-04/05, Traceability

### Bestehende Formel & Datenstrukturen (NICHT verändern, als Vorlage lesen)
- `service/src/reporting.rs:207-244` — A-22-1 `average_worked_hours_per_week` (pure Funktion,
  Vorlage für die neue Funktion; **nicht ändern**)
- `service/src/reporting.rs:44-49` — `WorkingHoursDay { date, hours, category }` (Tages-Daten)
- `service/src/reporting.rs:14` — `enum ExtraHoursReportCategory`
  (`Shiftplan`/`ExtraWork`/`VolunteerWork` = Arbeit; `Vacation`/`SickLeave`/`Holiday`/
  `UnpaidLeave`/`Unavailable` = Absence)
- `service/src/reporting.rs:78-101` — `GroupedReportHours` (`days: Arc<[WorkingHoursDay]>`)
- `service/src/reporting.rs:246-295` — `trait ReportingService` (u.a. `get_report_for_employee_range`,
  `get_employee_weekly_statistics`)

### REST / Gating / FE (Muster)
- `rest/src/report.rs:26,158-202` — bestehender `/{id}/weekly-statistics`-Endpoint (HR-gated,
  `#[utoipa::path]`, ApiDoc-Eintrag) als Kopiervorlage
- `service_impl/src/reporting.rs:262` — `HR_PRIVILEGE`-Gate-Muster im Service-Impl
- `shifty-dioxus/src/component/employee_view.rs:520-540` — `should_show_hr_stats` +
  Rendering der bestehenden „Ø Std/Woche"-Zahl (`format_hours`) — hier die neue Zahl daneben
- `shifty-dioxus/src/service/employee.rs:116-126` — FE-Loader `get_employee_weekly_statistics`

### Snapshot-Versionierung (Gate)
- `shifty-backend/CLAUDE.md` §"Billing Period Snapshot Schema Versioning" — bestätigt, dass
  ein reines Read-Aggregat **keinen** Bump erfordert (`CURRENT_SNAPSHOT_SCHEMA_VERSION` = 12)
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `average_worked_hours_per_week` (A-22-1): Vorlage für eine neue, **getrennte** pure Funktion
  (z.B. `average_hours_per_attendance_day`) im selben Modul.
- `EmployeeWeeklyStatistics` (`service/src/reporting.rs:196-205`) + `EmployeeWeeklyStatisticsTO`
  (rest-types): Muster für ein Result-Struct/TO — ggf. um ein Feld erweitern **oder** neues
  TO, Planner-Entscheidung.
- Bestehender HR-gated Endpoint `/{id}/weekly-statistics` + FE-Loader + `should_show_hr_stats`:
  kompletter Vertikal-Slice als Kopiervorlage.

### Established Patterns
- Alle Service-Methoden: `Option<Transaction>` + `permission_service.check_permission(HR_PRIVILEGE, ..)`.
- REST: `#[utoipa::path]` + ApiDoc-`components(schemas(..))`-Eintrag Pflicht.
- Tages-Kategorien liegen bereits pro Tag vor → Anwesenheitstag/Absence-Klassifikation ist
  ein simpler Kategorie-Match, keine neue DAO-Query nötig (Daten kommen aus dem Report).

### Integration Points
- Aggregation baut auf `get_report_for_employee_range` (liefert `by_week[].days`) über den
  angezeigten Zeitraum auf.
- FE: neue Zahl in denselben HR-Stats-Block wie die A-22-1-Zahl.
</code_context>

<specifics>
## Specific Ideas

- Beispiel des Users zur Semantik: „MA arbeitet im Zeitraum an 12 Tagen insgesamt 54 Std →
  4,5 Std/Anwesenheitstag."
- User-Formulierung: „Wir haben schon eine Stunden-pro-Woche-Berechnung. Wenn dann brauchen
  wir das auf Wochentage." → tagebasiert, nicht nochmal Wochen-Stunden.
</specifics>

<deferred>
## Deferred Ideas

- **AVG-04** — AVG-Trend über mehrere Abrechnungsperioden hinweg (Backlog, eigene Phase).
- **AVG-05** — konfigurierbare Absence-Exclusion-Kategorien (Backlog; v2.1 fixiert die Regel
  „nur echte Arbeitstage im Nenner" per Konstruktion).
- Multi-MA-Übersichtsliste (Ø aller flexiblen MA nebeneinander) — nicht angefragt; falls
  später gewünscht eigene Phase.
</deferred>
