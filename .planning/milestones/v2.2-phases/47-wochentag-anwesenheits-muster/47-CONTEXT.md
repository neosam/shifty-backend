# Phase 47: Wochentag-Anwesenheits-Muster (BE + FE) - Context

**Gathered:** 2026-07-02
**Status:** Ready for planning
**Mode:** Autonomous — ersetzt v2.1-AVG-Feature durch Wochentag-Aufschlüsselung. Success Criteria präzise, keine offenen Design-Fragen (Layout „Mo: 8 (80%) · Di: 3 (30%)" in ROADMAP festgezogen).

<domain>
## Phase Boundary

Ersetzt die v2.1-„Ø Std/Anwesenheitstag"-Kennzahl im HR-Stats-Block des Mitarbeiter-Reports durch eine pro-Wochentag-Anzeige mit Anzahl und Anteil in Prozent.

- **RPT-01**: `/report/{id}/attendance-statistics` liefert für flexible Mitarbeiter (HR-gated) pro Wochentag `count` (Anwesenheitstage) + `share` (Anteil an gezählten KWs im Zeitraum). Pure-fn getestet mit dem Wochentag-Kategorie-Muster aus v2.1 (Shiftplan/ExtraWork/VolunteerWork mit `hours > 0`).
- **RPT-02**: Bisheriger `average_hours_per_attendance_day`-Wert wird nicht mehr im HR-Stats-Block gerendert; die neue Zeile („Mo: 8 (80%) · Di: 3 (30%) · Mi: …") ist an gleicher Stelle sichtbar.
- **RPT-03**: Snapshot-Version bleibt 12 (grep-verifiziert); i18n de/en/cs für Wochentag-Labels + Tooltip + Leerzustand.

Kein Snapshot-Bump (RPT ist reines Read-Aggregat, kein neuer `BillingPeriodValueType`), keine Migration, keine neuen Deps.

</domain>

<decisions>
## Implementation Decisions

### Endpoint-Struktur (RPT-01)
- Reuse existierender Endpoint `/report/{id}/attendance-statistics` (v2.1 AVG). Response-Shape ändert sich: statt `average_hours_per_attendance_day` neue Felder `attendance_by_weekday: WeekdayAttendanceTO[]` mit `{ weekday: DayOfWeekTO, count: u32, share: f32 }` (7 Einträge).
- Endpoint bleibt HR-gated (analog v2.1).
- „Anwesenheitstag" identisch zu v2.1 D-AVG-02/03: mindestens ein Tages-Eintrag der Kategorie `Shiftplan` / `ExtraWork` / `VolunteerWork` mit `hours > 0`.
- „Gezählte Kalenderwochen" identisch v2.1 (nur KWs, in denen der Mitarbeiter im Zeitraum aktiv war). `share = count / kalenderwochen_im_zeitraum`.

### Pure-fn (RPT-01)
- Berechnungslogik in `service_impl/src/reporting.rs` (o.ä.), Funktion `weekday_attendance_distribution(days: &[AttendanceDay], weeks_in_range: u32) -> [WeekdayStat; 7]`.
- Pure-fn Tests: leere Eingabe (alle 0/0), einheitliche Verteilung, Rand-KWs, gemischte Kategorien, floating-point-Rundung.

### FE-Rendering (RPT-02)
- `shifty-dioxus/src/page/employee_view.rs` HR-Stats-Block: der Ø-Std-Anwesenheitstag-Absatz wird entfernt und durch die neue Zeile ersetzt.
- Format: `Mo: 8 (80%) · Di: 3 (30%) · Mi: 7 (70%) · Do: 5 (50%) · Fr: 2 (20%) · Sa: 0 (0%) · So: 0 (0%)` — trennendes `·` zwischen Wochentagen.
- Leerzustand (`count == 0` in allen 7 Tagen): eine Zeile mit einem lokalisierten Placeholder-Text („Keine Anwesenheitstage im Zeitraum" / „No attendance days in range" / „Žádné pracovní dny v období").
- Tooltip auf der Zeile: „Anzahl Anwesenheitstage pro Wochentag und Anteil an den gezählten Kalenderwochen im Zeitraum" (lokalisiert).

### Snapshot bleibt 12 (RPT-03)
- Grep-Verifikation: `grep -rn "BillingPeriodValueType\|CURRENT_SNAPSHOT_SCHEMA_VERSION"` — sicherstellen, dass Phase-47-Änderungen keinen neuen `BillingPeriodValueType` einführen und die Konstante nicht anfassen.
- i18n de/en/cs für Wochentag-Kurz-Labels (`Mo, Di, Mi, Do, Fr, Sa, So` de / `Mon, Tue, Wed, Thu, Fri, Sat, Sun` en / `Po, Út, St, Čt, Pá, So, Ne` cs) — falls bereits Wochentag-Labels im i18n-Bundle vorhanden sind, wiederverwenden. Sonst neue Keys.

### Claude's Discretion
- Genaue Rundungs-Semantik `share` (`0.0..=1.0` oder `0..=100`): Empfehlung `f32 0.0..=1.0` in TO, FE formatiert `%.0f%%`.
- Ob der `share = count/weeks_in_range` rundet oder floort (Empfehlung: `round`).
- WeekdayTO-Import: Reuse existierender `DayOfWeekTO` aus `rest-types`.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- v2.1 AVG-Feature (`average_hours_per_attendance_day`) — Endpoint-Struktur + HR-Gate + FE-Section-Placement bereits vorhanden. Phase 47 refactored den Rückgabewert.
- `DayOfWeekTO` in `rest-types` — bereits vorhanden (Special-Day-Feature).
- Wochentag-Kategorie-Muster aus v2.1 (Shiftplan/ExtraWork/VolunteerWork mit `hours > 0`).
- Wochenzählung im Zeitraum: existierende Helper (siehe v2.1 AVG-Plan-02).

### Established Patterns
- Pure-fn im `service_impl`-Level für Rechen-Logik + Unit-Tests im gleichen Modul.
- HR-Gate via `PermissionService::is_hr` + `Authentication<Context>`.
- Frontend Report-Rendering in `shifty-dioxus/src/page/employee_view.rs` mit klaren HR-Stats-Blöcken.

### Integration Points
- **BE**: `service_impl/src/reporting.rs` (o.ä.) — neue pure-fn; `service_impl/src/attendance_statistics.rs` — Endpoint-Handler-Adapter; `rest-types/src/attendance.rs` — neue DTO `WeekdayAttendanceTO`.
- **REST**: `rest/src/attendance_statistics.rs` — Response-Shape geändert, `#[utoipa::path]` aktualisiert.
- **FE**: `shifty-dioxus/src/page/employee_view.rs` — HR-Stats-Block; `shifty-dioxus/src/i18n/{en,de,cs}.rs` — neue Keys.

</code_context>

<specifics>
## Specific Ideas

- Reihenfolge Wochentage in Response: Mo → So (analog EU-Kalender).
- Prozent auf ganze Zahl runden (`round`), 0% ausgeblendet in Copy? Empfehlung: **anzeigen als (0%)** für Transparenz, kein Ausblenden.
- Migrationstest: alte AVG-Snapshot-Werte in DB werden unverändert stehen gelassen (kein Data-Cleanup); nur neue Reports zeigen die neue Zeile. Nicht als Migration umgesetzt — es ist ein reines Read-Rendering-Change.

</specifics>

<deferred>
## Deferred Ideas

Nichts — Scope-treu innerhalb Phase 47.

</deferred>
