# Phase 26: Freiwilligen-Abwesenheit & Cross-Navigation - Context

**Gathered:** 2026-06-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Zwei zusammengehörige Lieferungen für v1.7:
- **VFA (Achse B, Booking-Information):** Urlaub/Abwesenheit eines **Freiwilligen**
  (`is_paid=false`, `committed_voluntary>0`) **reduziert** seine committed-Zusage 🎯 in der
  Jahresansicht (`service_impl/src/booking_information.rs::get_weekly_summary`). Feiertage
  tun das bewusst **nicht** (Asymmetrie, Regressions-Guard).
- **NAV (Frontend):** Gegenseitige Deep-Links zwischen der bestehenden Abwesenheitsansicht
  (`/absences`) und dem Mitarbeiterreport pro Mitarbeiter (Sales: eigener Report; HR:
  Mitarbeiter-Report).

**Requirements:** VFA-01, VFA-02, NAV-01.

**Liefert NICHT:**
- Keine volle Urlaubsverwaltung/-Balance für Freiwillige (Future-Story; nur Jahresansicht-Reduktion).
- Keinen Snapshot-Bump (VFA betrifft nur die live berechnete Jahresansicht, nicht den
  persistierten Billing-Period-Snapshot aus `reporting.rs`).
- Keinen neuen Stichtag/keine Config-UI für VFA (Entscheidung D-26-02).
- Keine neue Abwesenheitsansicht — `/absences` (`AbsencesPage`) existiert bereits.

</domain>

<decisions>
## Implementation Decisions

### VFA-01: Welche Abwesenheiten reduzieren die committed-Zusage
- **D-26-01 (Alle drei Kategorien):** `Vacation`, `SickLeave` **und** `UnpaidLeave` reduzieren
  die committed-Zusage. Logik: Abwesenheit = der Freiwillige ist diese Woche nicht verfügbar,
  unabhängig vom Grund. (`AbsenceCategory` aus `service/src/absence.rs:28-32`.)

### VFA-01: Reduktions-Formel
- **D-26-03 (Ganze Woche raus):** Hat ein Freiwilliger in einer Kalenderwoche **irgendeine**
  Abwesenheit (eine der drei Kategorien, beliebige `day_fraction`), fällt sein
  `committed_voluntary`-Beitrag für **diese ganze Woche auf 0** — er wird aus der
  `committed_voluntary_hours`-Summe in `get_weekly_summary` (booking_information.rs:219-226)
  ausgeschlossen. **Nicht** pro-Tag anteilig. Bewusst einfach gehalten (User-Entscheidung
  gegen die anteilige Variante).

### VFA-01: Stichtag
- **D-26-02 (Kein Stichtag, immer aktiv):** Die VFA-Reduktion ist **immer** aktiv, **kein**
  konfigurierbarer Stichtag. Begründung: `get_weekly_summary` ist eine **live** berechnete
  Jahresansicht und wird **nicht** persistiert (anders als der Billing-Period-Snapshot) —
  es gibt keine historische Reproduzierbarkeit zu schützen. Daher **kein Snapshot-Bump**,
  **keine** Config-/Toggle-UI. Löst die in **D-25-09** nach Phase 26 verschobene Frage.

### VFA-02: Feiertags-Asymmetrie (Regressions-Guard)
- **D-26-04 (Strukturell garantiert + Test):** Feiertage reduzieren `committed_voluntary`
  **nicht**. Das ist bereits strukturell so: Die Feiertags-Automatik (Phase 25) berührt
  ausschließlich `reporting.rs` (`holiday_hours`) und fasst `get_weekly_summary` /
  `committed_voluntary` nie an (HOL-03/D-25-08). VFA-02 wird per **Regressionstest**
  abgesichert: ein Feiertag in der Woche eines Freiwilligen senkt seine committed-Zusage
  NICHT, eine Abwesenheit (VFA-01) schon — die Asymmetrie ist explizit getestet.

### NAV-01: Deep-Link-Mechanismus
- **D-26-05 (Route-Param mit employee_id):** Eine neue Route
  `/absences/:employee_id` (zusätzlich zur bestehenden param-losen `/absences/`). Der Param
  belegt den **bestehenden** Personen-Selektor der `AbsencesPage` vor. Bookmarkbar, konsistent
  mit `/employees/:employee_id`. Die param-lose Route bleibt der heutige HR-Gesamt-/Self-View.

### NAV-01: Cross-Links (gegenseitig, pro Mitarbeiter)
- **D-26-06 (Beide Report-Einstiege):**
  - **Sales-Rolle:** „Mein Zeitkonto" (`MyEmployeeDetails`, `/my_employee_details/`) ↔ eigene
    Abwesenheiten (`/absences`, eigener Kontext).
  - **HR-Rolle:** Mitarbeiterseite (`EmployeeDetails`, `/employees/:employee_id/`) ↔
    Abwesenheiten des Mitarbeiters (`/absences/:employee_id`).
  - Beide Richtungen verlinkt (Report → Absences UND Absences → Report).
  - Alle Beschriftungen **i18n de/en/cs**.
  - Genaue Link-Platzierung/Styling/Icons = **UI-Phase**.

### Claude's Discretion
- Exakte Wochen-Overlap-Detektion: Datumsbereich der Kalenderwoche bilden und
  `AbsenceService::find_overlapping_for_booking(sales_person_id, week_range)` nutzen;
  „irgendein Overlap" → committed_voluntary dieser Person/Woche ausschließen.
- DI-Verdrahtung: `BookingInformationService` (Business-Logic) bekommt eine
  `AbsenceService`-Dependency (AbsenceService ist Business-Logic-Tier; sicherstellen, dass
  kein Zyklus entsteht — AbsenceService konsumiert BookingInformationService nicht).
- Ob `/absences/:employee_id` als separate Route-Variante oder optionaler Param modelliert wird.
- Genaue i18n-Texte (de/en/cs), Link-Platzierung, Icons (final in UI-Phase).
- Wie Band-2 (`volunteer_hours`-Surplus) auf die Woche-raus-Reduktion reagiert (konsistent
  neu berechnen, falls relevant).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Vorphase
- `.planning/REQUIREMENTS.md` — VFA-01, VFA-02, NAV-01 + „Maßgebliche Design-Vorgaben" (Asymmetrie).
- `.planning/ROADMAP.md` § "Phase 26" — Goal + 4 Success Criteria.
- `.planning/phases/25-feiertags-auto-anrechnung-stichtag-konfiguration/25-CONTEXT.md` —
  D-25-08 (HOL-03-Guard: Feiertags-Automatik fasst booking_information nie an),
  D-25-09 (VFA-Stichtag nach Phase 26 verschoben → hier als D-26-02 entschieden).

### VFA-01/02 Backend
- `service_impl/src/booking_information.rs:159-273` — `get_weekly_summary`:
  `volunteer_ids` (159-166, `!is_paid.unwrap_or(false)`), `committed_voluntary_hours`
  (219-226, die zu reduzierende Summe), `volunteer_hours` Band-2 (203-216).
- `service/src/absence.rs:28-32` — `AbsenceCategory` (Vacation/SickLeave/UnpaidLeave);
  `:194-199` `find_by_sales_person`; `:231-237` `find_overlapping_for_booking(person, range)`.
- `dao/src/absence.rs:16-21,42-54` — `AbsenceCategoryEntity`, `AbsencePeriodEntity`
  (`from_date`/`to_date` inklusive, `day_fraction` Full/Half, soft-delete).
- `service_impl/src/absence.rs:558-586` — `find_overlapping_for_booking`-Impl (HR ∨ self).
- `service/src/sales_person.rs:17` — `is_paid: Option<bool>`.
- `service_impl/src/reporting.rs:84` — `find_working_hours_for_calendar_week` (am Tag gültige Verträge).
- `service_impl/src/test/booking_information.rs` — bestehende Band-1/Band-2-Tests (Vorlage für VFA-Tests).
- `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen" — BookingInformationService =
  Business-Logic, darf AbsenceService konsumieren (kein Zyklus).

### NAV-01 Frontend
- `shifty-dioxus/src/router.rs:29-62` — Route-Enum; `Absences {}` (59-60, param-los, →
  neue `/absences/:employee_id`-Variante), `EmployeeDetails { employee_id }` (41-42),
  `MyEmployeeDetails {}` (43-44), `WeeklyOverview {}` (35-36).
- `shifty-dioxus/src/page/absences.rs` — `AbsencesPage` (HR-vs-self via `has_privilege("hr")`,
  Personen-Selektor/Dropdowns, `is_selectable_employee`); Param-Vorbelegung des Selektors.
- `shifty-dioxus/src/page/my_employee_details.rs` — Sales eigener Report (Link-Quelle/-Ziel).
- `shifty-dioxus/src/page/employee_details.rs` — HR Mitarbeiter-Report (Link-Quelle/-Ziel).
- `shifty-dioxus/src/i18n/mod.rs` (Key-Enum) + `en.rs`/`de.rs`/`cs.rs` — NAV-Link-Labels.
- `shifty-dioxus/src/component/employees_list.rs` — `Link { to: Route::... }`-Muster + `navigator()`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`AbsenceService::find_overlapping_for_booking(person, range)`** — liefert Abwesenheiten,
  die einen Datumsbereich überlappen; ideal für „hat dieser Freiwillige in Woche W eine
  Abwesenheit?".
- **`volunteer_ids` + `committed_voluntary_hours`-Summe** in `get_weekly_summary` — die
  Reduktion ist ein Filter/Ausschluss in dieser Summe (Person mit Wochen-Overlap raus).
- **Bestehende `AbsencesPage` mit Personen-Selektor + HR/self-Branch** — Deep-Link belegt den
  Selektor vor, keine neue Ansicht nötig.
- **`Link { to: Route::… }` / `navigator().push(Route::…)`** — etabliertes Deep-Link-Muster.

### Established Patterns
- **Service-Tier:** BookingInformationService (Business-Logic) konsumiert AbsenceService
  (Business-Logic) — erlaubt, sofern kein Zyklus. DI in `shifty_bin/src/main.rs` nach Tier-Reihenfolge.
- **`WHERE deleted IS NULL`** / Soft-Delete in Absence-DAO.
- **i18n de/en/cs** für alle neuen Frontend-Texte; Key-Enum + 3 Locale-Maps.
- **Year-View ist live** (nicht persistiert) → keine Snapshot-Versionierung betroffen.

### Integration Points
- **VFA-01:** In `get_weekly_summary` pro Woche die Abwesenheiten der Freiwilligen laden;
  Personen mit Overlap aus `committed_voluntary_hours` ausschließen (ganze Woche).
- **NAV-01:** Router um `/absences/:employee_id` erweitern; Cross-Links auf MyEmployeeDetails
  + EmployeeDetails ↔ AbsencesPage; AbsencesPage liest den Param und belegt den Selektor vor.

</code_context>

<specifics>
## Specific Ideas

- **VFA-01-Test:** Freiwilliger mit `committed_voluntary>0`, eine Woche mit Abwesenheit →
  committed-Zusage dieser Woche = 0; Woche ohne Abwesenheit → volle committed-Zusage.
- **VFA-02-Test (Asymmetrie):** Feiertag in der Woche eines Freiwilligen → committed-Zusage
  UNVERÄNDERT; Abwesenheit in der Woche → reduziert. Beide im selben Testkontext gegenübergestellt.
- **NAV-01:** Deep-Link von HR-EmployeeDetails(:id) → `/absences/:id` zeigt direkt die
  Abwesenheiten dieses Mitarbeiters (Selektor vorbelegt); umgekehrt von AbsencesPage zurück
  zum Report des Mitarbeiters.

</specifics>

<deferred>
## Deferred Ideas

- **Volle Urlaubsverwaltung/-Balance für Freiwillige** (`get_all_paid` um `is_paid=false`
  erweitern) — Future-Story, außer Scope.
- **Anteilige Pro-Tag-Reduktion** für VFA-01 — bewusst zugunsten der „ganze Woche raus"-Regel
  verworfen (D-26-03); könnte später verfeinert werden.
- **Per-Mitarbeiter-Jahresansicht-Deep-Link** (vom aggregierten `weekly_overview`) — NAV-01
  fokussiert die Report-Einstiege (MyEmployeeDetails/EmployeeDetails) ↔ Absences.

### Reviewed Todos (not folded)
None — discussion stayed within phase scope.

</deferred>

---

*Phase: 26-Freiwilligen-Abwesenheit & Cross-Navigation*
*Context gathered: 2026-06-28*
