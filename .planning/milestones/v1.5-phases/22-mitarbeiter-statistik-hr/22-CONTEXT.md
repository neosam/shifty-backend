# Phase 22: Mitarbeiter-Statistik HR (Backend + Frontend) - Context

**Gathered:** 2026-06-25 (Discuss-Entscheidungen direkt mit User geklärt)
**Status:** Ready for planning

<domain>
## Phase Boundary

HR bekommt pro Mitarbeiter eine Statistik. Kern-Kennzahl: **durchschnittlich gearbeitete
Stunden pro Woche** (urlaubsbereinigt). Setzt das Todo `AVG-01` um
(`.planning/todos/pending/2026-06-09-auswertung-durchschnittliche-anwesenheit-flexible-stunden.md`).
Backend (neue Berechnung + REST) **und** Frontend (HR-only Block).

**Gilt nicht in dieser Phase:** weitere Kennzahlen über den Wochenschnitt hinaus
(Vergleichs-/Team-Übersicht etc.) — der Schnitt pro Woche ist die Kern-Anforderung.
</domain>

<decisions>
## Implementation Decisions (User-bestätigt)

- **D-22-01 (Bezugszeitraum):** Aktuelles Jahr **bis heute** (ISO-Wochen vom Jahresanfang
  bis zur aktuellen KW). Konsistent mit der bestehenden Jahresansicht.
- **D-22-02 (Zähler — „gearbeitet"):** Gesamte Anwesenheit = **Shiftplan + ExtraWork +
  Ehrenamt (volunteer)**. Pro Woche entspricht das `overall_hours + volunteer_hours` der
  `GroupedReportHours` (`overall_hours = shiftplan_paid + extra_work`, `reporting.rs:1217`;
  `volunteer_hours` separat, `:1249-1255`).
- **D-22-03 (Nenner — Abwesenheiten raus):** Wochen, die **vollständig** durch Abwesenheit
  abgedeckt sind, fallen aus dem Nenner. „Abwesenheit" umfasst **alle vier** Kategorien:
  **Vacation, SickLeave, UnpaidLeave, Holiday** (User-Auswahl).
- **D-22-04 (Geltungsbereich + Ort):** Für **alle** Mitarbeiter; angezeigt als **HR-only**
  Block auf der Mitarbeiter-Detailseite `/employees/:id` (kein eigener Route-Neubau).
- **D-22-05 (HR-Gating):** Zugriff/Anzeige nur mit HR-Rolle — Frontend analog `is_hr`
  (vgl. `absences.rs`), Backend mit Permission-Check im neuen Service-/REST-Pfad
  (kein Leak an Nicht-HR).

### Pinned Berechnungs-Definition (ASSUMPTION A-22-1 — autonom verwendet, falls nicht widersprochen)
- **Eingeschlossene Wochen** = ISO-Wochen [Jahresanfang … aktuelle KW], **ausgenommen**
  Wochen, die **voll abwesend** sind: worked (`overall_hours + volunteer_hours`) == 0
  **und** Abwesenheitsstunden der Woche (Vacation+SickLeave+UnpaidLeave+Holiday) > 0.
- **Schnitt** = Σ(`overall_hours + volunteer_hours`) über eingeschlossene Wochen ÷ Anzahl
  eingeschlossener Wochen.
- Wochen mit **teilweiser** Abwesenheit zählen **mit** ihren tatsächlich geleisteten
  Stunden (die Person war teilweise da). Wochen mit 0 Stunden **ohne** Abwesenheit zählen
  als 0 (kein Sonderfall). Diese Regel funktioniert auch für flexible/dynamische Verträge
  (keine `expected_hours`-Abhängigkeit). **Wenn der User eine andere Nenner-Semantik will
  (z.B. anteilige Teilwochen), hier vor dem autonomen Lauf anpassen.**

### Architektur
- **D-22-06 (Reuse Reporting):** Die Berechnung baut auf der bestehenden per-Woche-Struktur
  des `ReportingService` auf (`get_report_for_employee` → `by_week: Arc<[GroupedReportHours]>`,
  `reporting.rs:633-640`). Bevorzugt eine neue Methode am `ReportingService`
  (Business-Logic-Tier) statt Neuberechnung — die Wochen-Daten inkl. worked/volunteer/
  absence liegen dort bereits vor. **Hängt auf Phase 18** (korrekte per-Woche-Absence-Werte
  nach D-18-03 → saubere Voll-Abwesenheits-Erkennung).
- **D-22-07 (REST + OpenAPI):** Neuer Endpoint mit `#[utoipa::path]` + `ToSchema`-DTO
  (gemäß `CLAUDE.md`). Permission-gated (HR).
- **D-22-08 (Tests):** Backend-Unit-Tests für die Schnitt-Berechnung (inkl. voll-abwesende
  Woche ausgeschlossen, Teilwoche eingeschlossen, flexibler Vertrag); Frontend-SSR-Test
  für den HR-only Block (sichtbar mit HR, unsichtbar ohne). i18n De/En/Cs.

### Claude's Discretion
- Genaue DTO-Form + ob weitere abgeleitete Werte (z.B. Anzahl eingeschlossener Wochen)
  mit ausgewiesen werden.
- Platzierung des Blocks innerhalb `/employees/:id` (EmployeeView).
</decisions>

<canonical_refs>
## Canonical References

### Code (verifiziert)
- `service_impl/src/reporting.rs:517-640` — `get_report_for_employee` + `by_week`-Aufbau.
- `service_impl/src/reporting.rs:1041-1261` — `hours_per_week` (per-Woche worked/volunteer/absence).
- `service/src/reporting.rs:85-145` — `GroupedReportHours` (overall_hours/volunteer_hours/vacation_hours/sick_leave_hours/unpaid_leave_hours/holiday_hours/from/to).
- `shifty-dioxus/src/component/employee_view.rs` — Ziel-Ort des HR-Blocks; `is_hr`-Gating-Vorlage in `shifty-dioxus/src/page/absences.rs`.
- `.planning/todos/pending/2026-06-09-…-flexible-stunden.md` — `AVG-01` (resolves_phase: 22).

### Regeln
- `CLAUDE.md` § Service-Tier (ReportingService = Business-Logic), § OpenAPI (`#[utoipa::path]`/`ToSchema`), § Transactions, § Permissions. jj-only Commits.
- `shifty-dioxus/CLAUDE.md` (i18n 3 Locales, WASM-Build-Gate).

### Requirements / Roadmap
- `.planning/REQUIREMENTS.md` — STAT-01, STAT-02. `.planning/ROADMAP.md` § Phase 22.
</canonical_refs>
