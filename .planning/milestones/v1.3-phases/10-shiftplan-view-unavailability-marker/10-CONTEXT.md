# Phase 10: Shiftplan-View Unavailability-Marker - Context

**Gathered:** 2026-06-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Die Shiftplan-**Wochen**-View (`shifty-backend/shifty-dioxus/`) zeigt für die
**aktuell ausgewählte** Person pro Tag einen **neutralen „nicht verfügbar"-Marker**, wenn
für diesen Tag ein `UnavailabilityMarkerTO` gesetzt ist — **identisch** für alle drei
Varianten (`AbsencePeriod`, `ManualUnavailable`, `Both`). **Kein Grund, keine Kategorie,
keine Farbcodierung.**

> **⚖️ DSGVO Art. 9 (Gesundheitsdaten) — governing constraint:** Der Grund einer
> Abwesenheit (insbesondere `SickLeave` = Gesundheitsdatum) darf in der für alle
> Schichtplaner sichtbaren Wochenansicht **nicht** erkennbar sein. Daher wird die
> Kategorie im Marker **nicht** dargestellt — der Schichtplaner sieht nur „diese Person
> ist an diesem Tag nicht verfügbar". Diese Datenschutz-Entscheidung **überschreibt** die
> ursprünglichen ROADMAP-Success-Criteria 2 + 3 (Kategorie-Farben bzw. eigene
> `Both`-Visual-Indikation) — siehe D-12.

**Reine Frontend-Phase.** Der per-sales-person-Endpoint
(`GET /shiftplan-info/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}`) und
alle DTOs (`UnavailabilityMarkerTO`, `ShiftplanDayTO.unavailable`) existieren bereits im
Backend (v1.0 Phase 3). Phase 10 verdrahtet die Frontend-Glue: api → loader → state →
Render-Chip. Kernmechanik ist ein **Upgrade** des bestehenden `discourage`-Mechanismus
(heute nur manuelle Unavailability) auf die volle Marker-Information.

Anforderung: **FUI-A-07** (1 Requirement).

**Out of scope (eigene Phasen):** Deprecation-Banner für legacy `extra_hours` → Phase 11;
Slot-Capacity/`current_paid_count`-Rendering → Phase 12; cross-phase i18n-Audit → Phase 13.

</domain>

<decisions>
## Implementation Decisions

### Loading-Strategie
- **D-01:** Datenquelle = Umbau des **bestehenden** `load_unavailable_sales_person_days_for_week`
  (`shifty-dioxus/src/loader.rs:374`): statt der manual-only-Quelle ruft er den
  per-sales-person-Endpoint und extrahiert pro Tag das `ShiftplanDayTO.unavailable` →
  `UnavailabilityMarkerTO` für die ausgewählte Person. **Kleinster Blast-Radius** —
  nutzt die vorhandene `unavailable_days`/`discourage`-Verdrahtung weiter. Der globale
  `load_shift_plan` (`loader.rs:154`) für das Gesamt-Grid bleibt unverändert.
- **D-02:** Trigger zum Nachladen bleiben wie heute: bei Personen-Wechsel
  (`current_sales_person`) + Wochen-/Jahres-Wechsel (`week`/`year`).
- **D-03:** Ohne ausgewählte Person (`current_sales_person = None`) werden **keine
  Marker** angezeigt — Grid verhält sich wie heute. Marker beziehen sich konstruktiv
  immer auf genau eine Person (per-sales-person-Endpoint).

### Visual-Placement im Time-Grid
- **D-04:** Marker erscheint als **neutraler Chip im Tag-Spalten-Header** (eine Indikation
  pro Tag für die ausgewählte Person), gerendert im `DayView`-Header-Bereich
  (`shifty-dioxus/src/component/week_view.rs`, `DayViewProps.header`/title-Region). Der Chip
  ist **inhaltlich identisch** für alle drei Marker-Varianten — er signalisiert nur
  „nicht verfügbar", nicht den Grund.
- **D-05:** Die bestehende **Spalten-Tönung bleibt neutral `bad-soft` (rot)** für ALLE
  drei States — Semantik „diese Person hier nicht verplanen". Trennung der Concerns:
  **Spalte = „nicht buchen", Chip = „warum" (Kategorie/manuell/both)**. Der `discourage`-
  Mechanismus bleibt strukturell erhalten; jeder Tag mit gesetztem Marker ist
  `discourage = true`.
- **D-06:** Scope = nur die **Wochen**-View (FUI-A-07). Die Einzeltags-/Day-Aggregate-View
  ist nicht Teil dieser Phase.

### Marker-Darstellung (neutral, datenschutzkonform)
- **D-07 (REVIDIERT — DSGVO):** **Ein einziger neutraler Chip-Stil** für ALLE drei
  Marker-Varianten — **keine** Kategorie-Farben, **keine** `CategoryBadge`-Token-Wiederverwendung.
  Neutraler Stil: `text-ink-muted` / `bg-surface-2` + **dashed Border** (Mockup-Semantik des
  `ManualUnavailable`-„✕ frei"-Chips, aber jetzt für alle Fälle). **STATIC Tailwind-Classes**
  (Pitfall-5: niemals `format!`). Für das Rendering reicht effektiv die boolesche Information
  „Tag hat einen Marker (egal welcher)" — das `category`-Feld der Marker-Varianten wird
  **bewusst ignoriert**.
- **D-08 (REVIDIERT — DSGVO):** `Both` bekommt **keine** eigene Visual-Indikation und
  **keine** Kategorie-Farbe — es rendert **identisch** zu `AbsencePeriod` und
  `ManualUnavailable` (neutraler „nicht verfügbar"-Chip). Der ursprüngliche Mockup-
  `·!`/dashed-in-Kategorie-Farbe-Ansatz entfällt. (Begründung: eine eigene `Both`-Markierung
  würde verraten, dass ein `AbsencePeriod` existiert — Datenschutz-Konflikt. Die
  Redundanz-Auflösung/Aufräumen ist ohnehin deferred, siehe D-10.)
- **D-09:** Marker-Stil wird **jetzt festgezurrt** — kein separater `/gsd-ui-phase 10`-Schritt
  nötig (ein einzelner neutraler Stil, kein neues Design-Asset).
- **D-12 (DSGVO — governing):** Der Grund/die Kategorie einer Abwesenheit wird in der
  Wochenansicht **nicht** angezeigt (DSGVO Art. 9, `SickLeave` = Gesundheitsdatum). Der
  Marker ist rein binär „verfügbar / nicht verfügbar", identisch für alle drei
  `UnavailabilityMarkerTO`-Varianten. **Diese Entscheidung überschreibt ROADMAP-Phase-10
  Success-Criteria 2 (Kategorie-Farbe) und 3 (eigene `Both`-Visual-Indikation).**
  Plan-Phase/Verifier müssen gegen D-12 prüfen, **nicht** gegen die alten SC 2/3; die
  ROADMAP-Section sollte vor `/gsd:plan-phase 10` entsprechend korrigiert werden (siehe
  `<deferred>` → ROADMAP-Update).

### `Both`-State Aufräum-Button
- **D-10:** Phase 10 liefert **nur visuelle Indikation + Tooltip** für `Both`; die
  Aufräum-Aktion (Löschen des redundanten manuellen `sales_person_unavailable`-Eintrags)
  ist **deferred**. Begründung: Mockup `UnavailabilityChip` hat keinen Button; hält die
  Phase schlank; ROADMAP SC3 formuliert den Button explizit als „optional".

### i18n
- **D-11 (REVIDIERT — DSGVO):** **Ein einziger neutraler Key** für Label/Tooltip
  „nicht verfügbar" (z.B. `Key::ShiftplanUnavailableMarker`). **Keine** Kategorie-Namen,
  **keine** Both-/Redundanz-Texte (entfällt durch D-08/D-12). Vollständig in **De / En / Cs**
  (Pflicht-Locale-Coverage FUI-A-09; kein `Locale::En`-statt-`Locale::De`-Bug).

### Claude's Discretion
- Exakte Chip-Geometrie (Höhe/Padding/Icon, z.B. `✕`/`–`) und genaue Wortwahl des
  neutralen „nicht verfügbar"-Tooltips sind frei, solange der neutrale Stil (D-07), die
  Variant-Gleichheit (D-08/D-12) und die Locale-Parität (D-11) gewahrt bleiben.
- Ob der per-day-Marker als eigenes Atom/Component (`UnavailabilityChip` in Rust) oder
  inline in `DayView` gerendert wird, entscheidet die Plan-Phase (Test-/Reuse-Abwägung).
- Da die Kategorie ignoriert wird, kann das Frontend den Marker auf einen booleschen
  „unavailable"-Zustand pro Tag kollabieren (Loader-Detail, Plan-Phase).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase-Definition & Requirement
- `.planning/ROADMAP.md` § "Phase 10: Shiftplan-View Unavailability-Marker" (Zeilen 117–123) —
  Goal + Success Criteria 1–3. **⚠ SC 2 (Kategorie-Farbe) und SC 3 (eigene `Both`-Visual-Indikation)
  sind durch die DSGVO-Entscheidung D-12 überschrieben** — nur SC 1 (per-sales-person-Endpoint) bleibt
  unverändert gültig. Section sollte vor `/gsd:plan-phase 10` angepasst werden.
- `.planning/REQUIREMENTS.md` § "Shiftplan-View mit Unavailability-Marker" — FUI-A-07 (Zeilen 56–61).

### DTOs (Backend, single source of truth)
- `rest-types/src/lib.rs:1885` — `UnavailabilityMarkerTO` (Varianten `AbsencePeriod { absence_id, category }`,
  `ManualUnavailable`, `Both { absence_id, category }`; serde `tag="kind", content="data", snake_case`).
- `rest-types/src/lib.rs:991` — `ShiftplanDayTO` mit `unavailable: Option<UnavailabilityMarkerTO>`
  (nur per-sales-person-Endpoint befüllt; globale Endpoints liefern `None`).
- `rest-types/src/lib.rs:1567` — `AbsenceCategoryTO` (`Vacation` / `SickLeave` / `UnpaidLeave`).

### Backend-Endpoint (bereits vorhanden)
- `rest/src/shiftplan.rs` — Handler `get_shiftplan_week_for_sales_person`, Route
  `GET /shiftplan-info/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}`
  (Permission `HR ∨ self`), liefert `ShiftplanWeekTO` mit befülltem `unavailable`.

### Visual Reference & Konventionen
- `shifty-backend/shifty-dioxus/shifty-design/project/absences.jsx:111` — Mockup `UnavailabilityChip`
  (drei Visual-States; Farb-/Border-Logik; `✕ frei` / Kategorie-Kürzel / `·!`-Suffix für Both).
  Visuelle Referenz, **nicht** 1:1-Portierung.
- `shifty-backend/CLAUDE.md` § "i18n" / `shifty-dioxus/CLAUDE.md` — Locale-Parität De/En/Cs,
  `Locale::De`-Bug-Vermeidung.

### Prior-Phase-Decisions (carry-forward)
- `.planning/phases/08-absence-crud-page-foundation/08-CONTEXT.md` — D-01 (`UnavailabilityChip` → Phase 10,
  Zeile 151), `CategoryBadge`-Tokens, Dioxus-Dialog statt `window.confirm` (D-07 dort).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `shifty-dioxus/src/loader.rs:374` (`load_unavailable_sales_person_days_for_week`): wird auf den
  per-sales-person-Endpoint umgebaut (D-01) — bestehende Aufruf-/Trigger-Struktur bleibt.
- `shifty-dioxus/src/component/week_view.rs` (`DayView`, `ColumnViewSlot`, `discourage`-Flag): vorhandener
  `discourage`-Pfad (bad-soft Tönung) bleibt die Spalten-Semantik (D-05); Header-Region nimmt den Chip auf (D-04).
- `shifty-dioxus/src/page/absences.rs:260` (`CategoryBadge`): NICHT mehr als Farbquelle wiederverwenden
  (DSGVO, D-07) — nur noch als strukturelle Referenz für STATIC-Tailwind-Chip-Patterns relevant.
- **Hinweis:** `Key::AbsenceCategory*` werden für den Marker NICHT verwendet (D-11) — der neutrale
  Stil braucht genau einen neuen i18n-Key.

### Established Patterns
- **STATIC Tailwind-Classes per `match`**, niemals `format!` für Tailwind (Pitfall-5, dokumentiert in
  `CategoryBadge`). Gilt zwingend für den neuen Chip.
- i18n: jeder neue Key in en.rs / de.rs / cs.rs; Per-Locale-Reference-Matcher-Tests wie in Phase 8/9.
- dioxus-ssr Snapshot-Tests für neue Render-Logik (Pattern aus Phase 8/9).

### Integration Points
- `shifty-dioxus/src/page/shiftplan.rs:1041–1053` — `unavailable_days`-Signal speist heute
  `WeekView.discourage_weekdays`; hier muss zusätzlich der per-Tag-`UnavailabilityMarkerTO`
  durchgereicht werden (neuer State-Träger statt nur `day_of_week`-Liste).
- `shifty-dioxus/src/api.rs` (~Zeile 1016) — neue api-Funktion für den per-sales-person-Endpoint
  (analog zu `get_shiftplan_week`).

</code_context>

<specifics>
## Specific Ideas

- Visuelle Anlehnung: der **neutrale** Mockup-Chip (`absences.jsx:111`, `ManualUnavailable`-Fall
  „✕ frei" — dashed, neutral). Die kategorie-gefärbten und `Both`-`·!`-Darstellungen aus dem Mockup
  werden **bewusst NICHT** übernommen (DSGVO, D-07/D-08/D-12).
- Kern-Designprinzip dieser Phase (datenschutzkonform): **Spalte sagt „nicht buchen" (rot/bad-soft),
  Chip sagt nur „nicht verfügbar" — ohne Grund.** Der Schichtplaner erfährt NICHT, warum jemand
  abwesend ist.

</specifics>

<deferred>
## Deferred Ideas

- **`Both`-Aufräum-Button** (ROADMAP SC3 „optional"): Löschen des redundanten manuellen
  `sales_person_unavailable`-Eintrags. In Phase 10 entfällt durch DSGVO sogar die `Both`-Sicht-
  barkeit (D-08/D-12); ein Cleanup müsste auf der `/absences`-Maske oder einer Cutover-Cleanup-
  Phase erfolgen, nicht in der Wochenansicht. Kandidat für spätere Phase.
- **ROADMAP-Korrektur (vor `/gsd:plan-phase 10`):** `.planning/ROADMAP.md` Phase-10 SC 2 + SC 3
  müssen an die DSGVO-Entscheidung D-12 angeglichen werden (neutraler Marker statt Kategorie-Farbe
  / `Both`-Indikation), damit Plan-Phase und Verifier nicht gegen veraltete Kriterien prüfen.

</deferred>

---

*Phase: 10-shiftplan-view-unavailability-marker*
*Context gathered: 2026-06-19*
