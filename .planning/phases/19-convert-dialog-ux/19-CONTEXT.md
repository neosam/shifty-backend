# Phase 19: Convert-Dialog UX (Frontend) - Context

**Gathered:** 2026-06-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Das HR-Modal „In Zeitraum umwandeln" (`AbsenceConvertModal`,
`shifty-dioxus/src/component/absence_convert_modal.rs`) belegt heute beide Datumsfelder
mit demselben `initial_date` (`start_str = end_str = initial_date`, Z. 52-56) und kennt
nur `extra_hours_id`, `initial_date`, `amount`, `category` (Props Z. 35-42). Es schlägt
also stur „von = bis = Eintragstag" vor. Diese Phase macht das **bis-Datum smart**:

- **UV-01:** „bis" wird beim Öffnen **arbeitstagbasiert** vorbelegt (Wochenenden +
  Feiertage übersprungen), sodass der Zeitraum den berechneten Urlaubstagen entspricht.
  Die Tageszahl liegt bereits als `marker.derived_days` vor (Backend: `amount / hours_per_day`,
  angezeigt in `absences.rs:1712-1718`).
- **UV-02:** Entsprechen die Urlaubsstunden **exakt** dem Wochen-Soll des Vertrags, wird
  der Eintrag als „1 Woche" dargestellt und das Modal schlägt **Montag–Sonntag** der
  betroffenen Kalenderwoche vor. Jeder andere Wert → UV-01-Verhalten. **Keine Vielfachen,
  keine Teilwochen** (explizite User-Entscheidung).

Aufruf-Kontext: `absences.rs:2066-2095` (Modal-Render via `convert_target`), Marker-Quelle
`HourlyMarkerRow` (`absences.rs:1679-1783`).
</domain>

<decisions>
## Implementation Decisions

- **D-19-01 (Datenbedarf des Modals):** Für arbeitstagbasiertes bis-Datum + exakte
  Wochen-Erkennung braucht das Modal/der Aufrufer drei Dinge: (a) `derived_days`
  (liegt auf dem Marker vor), (b) das **Wochen-Soll** des Vertrags der Person (für die
  Exakt-Match-Erkennung UV-02), (c) den **Arbeitstag-Kalender**: welche Wochentage die
  Person arbeitet + Feiertage. **Plan-phase MUSS die Quelle dieser Daten festlegen** —
  bevorzugt prüfen, ob FE-State (EmployeeWorkDetails/SpecialDays) das schon hält; sonst
  Backend liefert die Vorschlagswerte (z.B. `suggested_end` + `is_full_week` auf dem
  Marker/Convert-Payload). **Empfehlung:** Backend-seitige Vorberechnung ist robuster
  (Arbeitstag-/Feiertags-Logik lebt schon backend-nah), vermeidet FE-Duplizierung der
  Kalender-Logik.
- **D-19-02 (Arbeitstag-Definition):** „Arbeitstag" = Wochentag, an dem die Person laut
  `EmployeeWorkDetails` arbeitet (`potential_weekday_list`-Analogon, vgl. backend
  `reporting.rs:988`), **minus** Feiertage (`SpecialDayService`). bis-Datum = von-Datum
  + so viele Arbeitstage vorrücken, bis `derived_days` erreicht ist (von zählt als
  erster Tag).
- **D-19-03 (Exakt-Wochen-Regel, UV-02):** „1 Woche" + Mo–So-Vorschlag **nur** wenn
  `amount == Wochen-Soll` (exakt, mit f32-Epsilon-Toleranz). Sonst Tage-Darstellung +
  D-19-02. Half-Day (`derived_days == 0.5`, `DayFractionTO::Half`) bleibt von = bis.
- **D-19-04 (Submit unverändert robust):** Die bestehende P-7-Submit-Defense
  (`absence_convert_modal.rs:154-181`, parse + `s <= e` + inline error) bleibt; das
  Pre-Fill ändert nur die Default-Werte, nicht die Validierung. HR kann den Vorschlag
  jederzeit überschreiben.
- **D-19-05 (i18n):** Für die „1 Woche"/„N Tage"-Darstellung ggf. neue i18n-Keys in
  **allen drei** Locales (De/En/Cs) — kein `Locale::En`-statt-`De`-Bug.

### Claude's Discretion
- Genaue Aufteilung Pre-Compute im Aufrufer (`absences.rs`) vs. im Modal.
- Ob die Arbeitstag-/Wochen-Logik als FE-Helper oder Backend-Feld realisiert wird
  (D-19-01) — Planner entscheidet nach Daten-Verfügbarkeit; Empfehlung Backend.
</decisions>

<canonical_refs>
## Canonical References

### Code (verifiziert)
- `shifty-dioxus/src/component/absence_convert_modal.rs` — Modal (Props 35-42; Pre-Fill 52-56; Submit-Defense 154-181).
- `shifty-dioxus/src/page/absences.rs:2066-2095` — Modal-Render + `on_submit` → `AbsenceAction::ConvertExtraHours`.
- `shifty-dioxus/src/page/absences.rs:1679-1783` — `HourlyMarkerRow`; `derived_days` 1712-1718.
- Backend Arbeitstag-Referenz: `service_impl/src/reporting.rs:988` (`potential_weekday_list`); `SpecialDayService` (Feiertage).

### Regeln
- `shifty-dioxus/CLAUDE.md` (i18n 3 Locales, WASM-Build-Gate, Dioxus-Signale).
- WASM-Build-Gate: `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown`.

### Requirements / Roadmap
- `.planning/REQUIREMENTS.md` — UV-01, UV-02. `.planning/ROADMAP.md` § Phase 19.
</canonical_refs>
