# Phase 50: pdf-renderer-browser-look — Discussion Log

**Session:** 2026-07-03 (Textform-Modus, per User-Präferenz)
**Areas discussed:** G1–G6 (6 Gray Areas, alle in einem Turn, plus 1 Rückfrage zu G6)

Diese Datei ist **nur für Menschen** (Audits, Retrospektiven) und wird von
Researcher/Planner/Executor NICHT konsumiert. Die Entscheidungen selbst
sind in `50-CONTEXT.md`.

---

## Discussion Trail

### G1 — Slot-Layout innerhalb der Tages-Spalte

**Options presented:**
- a) Time-Grid analog Browser (Y-Achse = Uhrzeit, proportional zu Dauer,
   Zeitachsen-Spalte links, Sub-Column-Logik für Überlappungen).
- b) Vertikaler Stack, feste Zellenhöhe (nach Startzeit sortiert, Uhrzeit in
   der Zelle, Dauer visuell nicht codiert).
- c) Hybrid: Stack (b), Zellenhöhe skaliert grob mit Dauer, kein echtes Grid.

**User answer:** c

**Notes:** Wenig Komplexität (keine Sub-Column-Logik), aber Dauer bleibt visuell
codiert. Sortierung nach Startzeit ist deterministisch. Overlap-Fall existiert
faktisch selten in Shifty-Slots — Stack ist ein akzeptabler Kompromiss.

→ **D-50-01, D-50-02**

### G2 — Multi-Page-Handling

**Options presented:**
- a) Best-Effort 1 Seite, Overflow abschneiden mit „+N weitere".
- b) 1 Seite pro Woche mit horizontalem/vertikalem Umbruch bei Overflow.
- c) Feste 1-Seite-Garantie mit Auto-Shrink Font-Size.

**User answer:** a

**Notes:** Realistisch: eine Woche mit 7 Tages-Spalten × wenigen Slots passt
immer auf Landscape-A4. Overflow ist ein Edge-Case, wird abgefangen. Der
Header-Renderer wird trotzdem multi-page-fähig gebaut (D-50-03), damit PDF-02
(„auf jeder Seite") vertragskonform bleibt.

→ **D-50-03, D-50-04**

### G3 — Namen-Rendering in der Slot-Zelle

**Options presented:**
- a) Plain-Text-Liste, ein Name pro Zeile.
- b) Chip-Look mit Rahmen + Hintergrundfarbe aus `sales_person.background_color`.
- c) Kommagetrennte Namen in einer Zeile, gepolstert.
- Sub-Frage: Unpaid-Marker (`is_paid=false`) mit Suffix „ (frw)" oder weglassen?

**User answer:** a

**Notes:** Unpaid-Marker-Sub-Frage wurde nicht explizit beantwortet — Claude
setzt „ (freiwillig)" als Default (D-50-07), damit in Plain-Text die
Unterscheidung Angestellte/Freiwillige nicht verloren geht. Trivial rückgängig
falls User später Marker weglassen will. Namen alphabetisch sortiert
(D-50-06, Claude's Discretion).

→ **D-50-05, D-50-06, D-50-07**

### G4 — Sonntag-Spalte, Header/Footer-Layout

**User answer (Freeform):**
- Sonntag ausblenden wie Browser (analog `has_sunday`).
- Header-Position ist mir egal.
- Timestamp-Position ist mir auch egal.

**Notes:** Sonntag-Ausblendung entspricht `week_view.rs` §1199–1229; das ändert
`DAY_COL_WIDTH_MM`-Berechnung → muss dynamisch werden (Anti-Pattern in
`code_context`). Header + Timestamp landen als Claude's Discretion:
Titel oben links, Timestamp oben rechts, gemeinsame Header-Zeile — maximiert
vertikalen Platz für das Grid.

→ **D-50-08, D-50-09**

### G5 — Slot-Rahmen (sichtbare Boxen)

**Options presented:** Ja/Nein.

**User answer:** Ja.

**Notes:** `printpdf` 0.7 unterstützt `Op::DrawLine` etc. Konkrete API-Wahl
ist Claude's Discretion für den Planner. 0.3–0.5pt Line, Schwarz, kein Fill.
Analog auch Tages-Spalten-Rahmen — Grid-Look wie im Browser.

→ **D-50-10**

### G6 — Renderer-Signatur / Timestamp-Timezone

**Initial options presented:**
- Renderer nimmt `OffsetDateTime` (Aufrufer konvertiert) vs.
- Renderer nimmt UTC + Timezone-String (Renderer konvertiert).

**User answer (initial):** „Wäre gut, wenn man es auf die User Timezone
umrechnen könnte. Die sollte es wegen iCal export geben."

**Follow-up Frage (Claude):** iCal-Service konvertiert selbst nicht — er
nutzt `TIMEZONE`-ENV nur als TZID-Attribut. Echte UTC→IANA-TZ-Konvertierung
im Renderer würde eine neue Dep (`time-tz`) brauchen und v2.3-„keine neuen
Deps" brechen. Drei Sub-Optionen präsentiert:
- 6a) `time::OffsetDateTime::now_local()` (OS-TZ), keine neue Dep.
- 6b) `ConfigService::timezone` + IANA-Konvertierung mit neuer Dep.
- 6c) `now_local()` + TZ-Suffix im Timestamp-Text (informativ, keine Konvertierung).

**User final answer:** 6a

**Notes:** Deckt 99%-Fall (Server-OS-TZ = ENV `TIMEZONE`). Fallback bei
`IndeterminateOffset` auf `now_utc()` mit Log-Warnung (kein panic-Pfad).
Renderer bleibt pure-testbar via Fixed-Timestamp-Injection.

→ **D-50-11, D-50-12, D-50-13**

---

## Deferred Ideas (aus Discussion)

- Multi-Page-Rendering (falls Overflow zum echten Problem wird).
- Chip-Look mit Farbe (G3-b, verworfen).
- Auto-Shrink Font-Size (G2-c, verworfen).
- Time-Grid analog Browser (G1-a, verworfen — spätere „PDF-Layout v2").
- TZ-Konvertierung via `time-tz` (G6-b, verworfen).
- TZ-Suffix im Timestamp (G6-c, verworfen).
- Seitennummerierung (nicht in Scope, sinnvoll bei Multi-Page).
- PDF-Preview im Browser (schon in Phase 49 deferred).

## Claude's Discretion Items

- Slot-Namen-Sortierung alphabetisch (D-50-06).
- Unpaid-Marker „ (freiwillig)"-Suffix (D-50-07).
- Header oben links, Timestamp oben rechts, gemeinsame Header-Zeile (D-50-09).
- `printpdf`-API-Wahl für Line-Draws.
- Konkrete Maß-Konstanten (Base-Height, Duration-Step, Font-Sizes).
- Renderer-Code-Struktur (Zerlegung in Sub-Fns / Sub-Modules).
- Hyphen statt En-Dash im „08:00 - 12:00"-Format.

---

*Log erstellt: 2026-07-03*
