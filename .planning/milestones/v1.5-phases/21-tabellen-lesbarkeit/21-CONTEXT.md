# Phase 21: Tabellen-Lesbarkeit (Frontend) - Context

**Gathered:** 2026-06-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Zwei kleine, rein visuelle Frontend-Polish-Änderungen zur Zeilen-Lesbarkeit breiter
Tabellen auf großen Bildschirmen. **Reine Frontend-Phase**, kein API-/State-Change.

- **UI-01:** Die Stunden-Tabelle unterhalb des Schichtplans
  (`WorkingHoursMiniOverview`, `TableLayout`, `shifty-dioxus/src/component/working_hours_mini_overview.rs`,
  gerendert in `shifty-dioxus/src/page/shiftplan.rs:1140`) ist heute `w-full` ohne
  max-width und ohne Zebra-Striping. Auf großen Bildschirmen verrutscht man in der Zeile.
  → **maximale Breite** + **Zebra-Layout** (abwechselnde Zeilen-Hintergründe).
- **UI-02:** In der `/absences`-Tabelle (`AbsenceList`) ist die erste Spalte (Mitarbeiter)
  als `1.5fr` definiert — auf großen Bildschirmen gigantisch breit. Das Grid
  `grid-cols-[1.5fr_170px_140px_90px_70px]` steht an **drei** Stellen identisch:
  Header `absences.rs:1632`, `HourlyMarkerRow` `absences.rs:1725`, `AbsenceListRow`
  `absences.rs:1817`. → Mitarbeiter-Spalte **deutlich schmaler** (alle drei Stellen
  konsistent ändern), optional max-width der Tabelle.
</domain>

<decisions>
## Implementation Decisions

- **D-21-01 (UI-01 max-width + Zebra):** TableLayout-Container bekommt eine sinnvolle
  max-width (Planner wählt konkreten Tailwind-Wert passend zum Layout) und
  Zebra-Striping über abwechselnde Zeilen-Hintergründe (z.B. `odd:bg-…`/`even:bg-…` oder
  `[&:nth-child(odd)]`-Pattern mit bestehenden Surface-Tokens). Bestehende
  Selekt/Hover-/`is_selected`-Hervorhebung (`working_hours_mini_overview.rs:216-219`)
  muss visuell Vorrang vor dem Zebra behalten.
- **D-21-02 (UI-02 Spaltenbreite):** Die `1.5fr`-Mitarbeiterspalte wird schmaler — z.B.
  feste Breite oder kleinerer fr-Anteil. **Alle drei** Grid-Definitionen
  (`absences.rs:1632/1725/1817`) müssen denselben neuen Wert tragen (sonst verrutschen
  Header und Zeilen gegeneinander). Truncation der Personennamen (`truncate`,
  `absences.rs:1728`) bleibt erhalten.
- **D-21-03 (kein Funktions-Change):** Reine CSS/Tailwind-Anpassung — keine Logik,
  keine neuen Props, keine i18n. SSR-Snapshot-Tests dürfen sich nur in Klassen ändern.

### Claude's Discretion
- Konkrete max-width- und Spaltenbreiten-Werte (am Design/Layout ausrichten).
- Zebra über Tailwind-`odd:`/`even:` vs. expliziter Index-Parität.
</decisions>

<canonical_refs>
## Canonical References

### Code (verifiziert)
- `shifty-dioxus/src/component/working_hours_mini_overview.rs` — `TableLayout` (162+), `w-full` Tabelle (173), `is_selected`-Row (216-219).
- `shifty-dioxus/src/page/shiftplan.rs:1140` — Render-Stelle der Tabelle unter dem Schichtplan.
- `shifty-dioxus/src/page/absences.rs:1632` / `:1725` / `:1817` — die drei `grid-cols-[1.5fr_…]`-Definitionen.

### Regeln
- `shifty-dioxus/CLAUDE.md` (Tailwind via `npx tailwindcss …`; WASM-Build-Gate; Pitfall 5 statische Klassen).

### Requirements / Roadmap
- `.planning/REQUIREMENTS.md` — UI-01, UI-02. `.planning/ROADMAP.md` § Phase 21.
