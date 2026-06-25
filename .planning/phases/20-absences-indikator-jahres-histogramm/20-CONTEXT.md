# Phase 20: Absences-Indikator & Jahres-Histogramm (Frontend) - Context

**Gathered:** 2026-06-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Vier Frontend-Lesbarkeits-/Erkennbarkeits-Verbesserungen. **Reine Frontend-Phase.**

- **UV-03 — Warn-Indikator:** Stundenbasierte (noch nicht konvertierte) Marker auf
  `/absences` (`HourlyMarkerRow`, `absences.rs:1679-1783`) bekommen einen ⚠️-Indikator
  am Zeilenanfang (Spalte 1, vor dem Personennamen, `absences.rs:1727-1740`). Grund:
  ein stundenbasierter Eintrag ist noch **kein echter Urlaub** (Absence Period) und
  sollte konvertiert werden. Es existiert bereits ein „stundenbasiert"-Badge in Spalte 3
  (`absences.rs:1755-1758`) — der Indikator ist ein **zusätzlicher**, prominenterer
  Hinweis am Zeilenanfang.

- **YV-01/YV-02/YV-03 — Jahres-Histogramm auf `/employees/:id`:** Betrifft
  `EmployeeWeeklyHistogram` (`shifty-dioxus/src/component/employee_weekly_histogram.rs`)
  und die aufklappbare KW-Liste (`expand_weeks`-Toggle) / `WeekDetailPanel` in
  `shifty-dioxus/src/component/employee_view.rs` (Histogramm gerendert Z. 412-433).
  **NICHT** `weekly_overview_chart.rs` (das ist die separate Team-Route `/weekly_overview/`,
  dort sind Datum + volunteer bereits vorhanden). Heute: jeder Histogramm-Balken kodiert
  `overall_hours` als **Einzelbalken** (eine Farbe — warn unter, accent über
  `expected_hours`, `employee_weekly_histogram.rs:44-48`); X-Achse zeigt nur die KW-Nummer;
  kein KW+Datum-Hover. Datenmodell `state::employee::WorkingHours` (`state/employee.rs:182-198`)
  hält je Woche `from`, `to`, `overall_hours`, `expected_hours`, `volunteer_hours` — **alle
  benötigten Felder sind vorhanden.**
  - **YV-01:** Balken-Hover-Tooltip zeigt KW + von–bis Datum (`from`–`to`).
  - **YV-02:** Wo nur die KW-Nummer steht (X-Achse + aufgeklappte KW-Liste), zusätzlich
    das von–bis Datum — Format „KW XY" + Zeilenumbruch + „von–bis".
  - **YV-03:** Histogramm-Balken stellt `volunteer_hours` als **gestapeltes** Segment dar
    (statt nur `overall_hours` als Einzelbalken); die aufgeklappte KW-Liste /
    `WeekDetailPanel` weist die Freiwilligen-Stunden als **separaten Wert** aus.
</domain>

<decisions>
## Implementation Decisions

- **D-20-01 (Warn-Indikator UV-03):** ⚠️ als führendes Element in Spalte 1 der
  `HourlyMarkerRow` (vor dem Namen). Statisches Tailwind (Pitfall 5: keine
  format!()-Klassen). Ggf. `title`/aria für Screenreader + i18n-Tooltip-Text (3 Locales).
  Das bestehende „stundenbasiert"-Badge bleibt.
- **D-20-02 (Hover YV-01):** Nativer HTML-`title` am Balken (wie es das Team-Chart in
  `weekly_overview_chart.rs:124` vormacht), Inhalt: KW-Nummer + `from`–`to` via
  `i18n.format_date(...)`. Kein eigenes Tooltip-Overlay nötig.
- **D-20-03 (KW+Datum YV-02):** In der aufgeklappten KW-Liste / `WeekDetailPanel` das
  Datum unter der KW-Nummer (zweizeilig). Für die Histogramm-X-Achse: Datum dezent
  ergänzen, ohne die Achse zu überladen (Planner-Discretion zur konkreten Darstellung —
  das `from`/`to` der `WorkingHours` liefert die Werte).
- **D-20-04 (Stacking YV-03):** Der Balken wird in (mindestens) zwei Segmente zerlegt:
  „reguläre" Stunden und `volunteer_hours`, gestapelt. Bezugsgröße/Skalierung
  (`compute_max_y`, `employee_weekly_histogram.rs:28`) muss die Stapel-Summe berücksichtigen,
  damit nichts über den Rahmen ragt. Farbwahl konsistent mit bestehender Token-Logik
  (volunteer dezent, z.B. `var(--ink-muted)` wie im Team-Chart) — `bar_color_token`
  (`:44-48`) ggf. erweitern. Die Referenzlinie `expected_hours` bleibt.
- **D-20-05 (Tests + i18n):** SSR-Snapshot-Tests für: ⚠️-Indikator vorhanden;
  Histogramm-Balken enthält volunteer-Segment wenn `volunteer_hours > 0`; KW-Liste zeigt
  Datum + separaten volunteer-Wert. Neue i18n-Keys in De/En/Cs.

### Claude's Discretion
- Genaue X-Achsen-Datums-Darstellung (Platz begrenzt) — YV-02.
- Ob YV-03 die regulären Stunden weiter nach paid/committed aufschlüsselt oder nur
  „regulär vs. volunteer" stapelt (Mindestanforderung: volunteer separat sichtbar).
</decisions>

<canonical_refs>
## Canonical References

### Code (verifiziert)
- `shifty-dioxus/src/page/absences.rs:1679-1783` — `HourlyMarkerRow` (UV-03; Badge 1755-1758; Spalte 1 1727-1740).
- `shifty-dioxus/src/component/employee_weekly_histogram.rs` — Histogramm (compute_max_y 28; bar_color_token 44-48; week_year_week 54-56).
- `shifty-dioxus/src/component/employee_view.rs:412-433` — Histogramm-Render + `expand_weeks` + `WeekDetailPanel`.
- `shifty-dioxus/src/state/employee.rs:182-198` — `WorkingHours` (from/to/overall_hours/expected_hours/volunteer_hours).
- Vorlage Hover/Stacking: `shifty-dioxus/src/component/weekly_overview_chart.rs:113-139` (Team-Chart, macht title-Tooltip + gestapelte Segmente bereits vor).

### Regeln
- `shifty-dioxus/CLAUDE.md` (i18n 3 Locales; Pitfall 5 statische Tailwind-Arms; WASM-Build-Gate).

### Requirements / Roadmap
- `.planning/REQUIREMENTS.md` — UV-03, YV-01, YV-02, YV-03. `.planning/ROADMAP.md` § Phase 20.
</canonical_refs>
