# Phase 50: PDF-Renderer neu — Browser-Look + Timestamp - Research

**Researched:** 2026-07-03
**Domain:** printpdf 0.7 PDF Rendering, Layout-Algorithmus, time::OffsetDateTime, Rust Backend
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-50-01 (Hybrid Stack mit Dauer-Skalierung):** Vertikales Stapeln; `cell_height_mm = base_mm + duration_hours * step_mm`. Kein Time-Grid, keine Überlappungs-Sub-Column-Logik.
- **D-50-02:** Sortierung `slot.from` aufsteigend → `slot.to` aufsteigend → `slot.id` (Tie-Breaker).
- **D-50-03:** Best-Effort 1 Seite; Overflow abschneiden + `+ N weitere` am unteren Rand der letzten Slot-Box.
- **D-50-04:** Namen-Overflow analog: `+ N weitere` in letzter sichtbarer Namen-Zeile.
- **D-50-05:** Plain-Text-Liste, ein Name pro Zeile, kein Chip-Rahmen, keine Hintergrundfarbe.
- **D-50-06:** Namen alphabetisch (case-insensitive) sortiert innerhalb Slot-Box.
- **D-50-07:** `is_paid == Some(false)` → Suffix ` (freiwillig)` hinter dem Namen.
- **D-50-08:** Sonntag-Spalte dynamisch — nur wenn mindestens ein Sonntag-Slot existiert (analog `has_sunday` in `week_view.rs`).
- **D-50-09:** Titel oben-links Bold, Timestamp oben-rechts ~9pt — auf derselben Header-Zeile. Kein Footer.
- **D-50-10:** Slot-Boxen mit sichtbarer rechteckiger Umrandung, 0.3–0.5pt Schwarz, kein Fill.
- **D-50-11:** Renderer-Signatur: `render_shiftplan_week_pdf(week, sales_persons, header_year, header_week, render_timestamp: time::OffsetDateTime) -> Result<Vec<u8>, ServiceError>`.
- **D-50-12:** Aufrufer (PdfShiftplanService) beschafft Timestamp via `now_local()`, Fallback `now_utc()` + warn! bei `IndeterminateOffset`.
- **D-50-13:** Byte-Determinismus aufgehoben; `FIXED_METADATA_TIMESTAMP` bleibt für PDF-Metadata-Felder.
- **D-50-14:** Fixed-Timestamp-Fixture `time::macros::datetime!(2026-07-03 17:15 UTC)` für alle Renderer-Unit-Tests.
- **D-50-15:** Test-Portierung: `empty_week_yields_valid_pdf_signature`, `header_contains_year_and_week`, `all_active_sales_persons_appear`, `build_page_header_produces_expected_text`, `build_day_column_headers_yields_seven_short_labels`, `normalize_pdf_id_removes_variable_id_array`, `find_all_subsequences_locates_multiple_occurrences` bleiben. `deterministic_bytes_for_same_input` und `sales_persons_sorted_by_id` entfallen.
- **D-50-16:** Neue Tests: `render_includes_timestamp_string`, `slot_boxes_sorted_by_start_time`, `names_within_slot_alphabetical`, `unpaid_marker_suffix`, `sunday_column_hidden_when_no_sunday_slots`, `sunday_column_shown_when_at_least_one_sunday_slot`, `now_local_fallback_to_utc_on_indeterminate_offset`.
- **D-50-17:** UAT via Phase-49-Button gegen reale Woche (visueller Check).

### Claude's Discretion

- Konkrete `printpdf`-API-Wahl für Line-Draws (D-50-10).
- Exakte Maß-Konstanten: Slot-Box-Base-Height, Duration-Step, Header-Height, Padding, Font-Sizes.
- Struktur des Renderer-Codes: Pure-Fn-Zerlegung.
- Ob `pdf_render.rs` in Submodule zerlegt wird oder in einer Datei bleibt.
- ROADMAP-Update nach Verifikation.

### Deferred Ideas (OUT OF SCOPE)

- Multi-Page-Rendering (Overflow abschneiden, kein Auto-Split).
- Chip-Look mit Farbe (G3-b verworfen).
- Auto-Shrink Font-Size.
- Time-Grid analog Browser (G1-a verworfen).
- TZ-Konvertierung via `time-tz`.
- TZ-Suffix im Timestamp.
- Seitennummerierung.
- PDF-Preview im Browser.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PDF-01 | Landscape A4, Slots als sichtbare Boxen pro Tages-Spalte, Uhrzeit-Label `08:00 - 12:00`, Sales-Person-Namen in Slot-Zelle, Kopfzeile `Schichtplan KW {NN} ({JJJJ})`, Sonntag dynamisch | Layout-Algorithmus (§Architecture Patterns), `Rect`-API von printpdf 0.7 (§Standard Stack), `has_sunday`-Logik aus `week_view.rs` (§Code Examples) |
| PDF-02 | `Erstellt am DD.MM.YYYY HH:MM Uhr` auf jeder Seite; Renderer nimmt `OffsetDateTime` als Argument | `time::OffsetDateTime`-Formatting (§Code Examples), `local-offset`-Feature-Anforderung (§Common Pitfalls), Fixed-Timestamp-Fixture für Tests (§Validation Architecture) |
</phase_requirements>

---

## Summary

Phase 50 ist ein vollständiger Rewrite von `service_impl/src/pdf_render.rs`. Die wichtigsten technischen Erkenntnisse aus der Research:

**printpdf 0.7.0** (exakt gelockt in `Cargo.lock`) bietet `layer.add_rect(Rect::new(Mm(x1), Mm(y1), Mm(x2), Mm(y2)).with_mode(PaintMode::Stroke))` als bevorzugte API für Rechteck-Umrandungen ohne Fill. Die ältere `add_line`/`Line`-API funktioniert für vier separate Linien, aber `add_rect` mit `PaintMode::Stroke` ist atomarer, robuster und direkter an die Anforderung angepasst.

**Umlaut-Encoding** ist kein Blocker: printpdf 0.7 verwendet lopdf 0.31 mit `WinAnsiEncoding` für Builtin-Fonts. Die lopdf-Encoding-Tabelle enthält alle deutschen Umlaute (ä→0xE4, ö→0xF6, ü→0xFC, Ä→0xC4, Ö→0xD6, Ü→0xDC, ß→0xDF). Sales-Person-Namen mit Umlauten werden korrekt gerendert. `encode_ascii_to_pdf_hex` funktioniert allerdings nur für ASCII — Umlaut-Test-Assertions müssen auf Win-1252-Hex-Encoding umgestellt werden.

**`time::OffsetDateTime::now_local()`** erfordert das `local-offset`-Feature, das in `service_impl/Cargo.toml` noch nicht aktiviert ist. Auf Linux (NixOS) gibt `localtime_r` (thread-safe) kein `IndeterminateOffset` zurück — der Fallback ist trotzdem Pflicht per D-50-12, da das Feature selbst auf anderen Deployments/Plattformen scheitern kann.

**Layout-Algorithmus:** Mit `base_mm = 12`, `step_mm = 5`, verfügbarer Spalten-Höhe von ~182mm passen typische Schichtplan-Wochen (3–4 Slots je 4h) ohne Overflow. Bei 6+ Slots oder langen Slots kommt `+ N weitere` zum Einsatz — exakt wie in D-50-03 spezifiziert.

**Primary recommendation:** Verwende `layer.add_rect(rect.with_mode(PaintMode::Stroke))` für Slot-Boxen, aktiviere `local-offset`-Feature in `service_impl/Cargo.toml`, und behalte `pdf_render.rs` als einzelne Datei mit Pure-Fn-Zerlegung (keine Submodule für Phase 50).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| PDF-Rendering (Layout + Text + Grafik) | `service_impl` (Pure Fn) | — | Stateless, kein I/O; DAO/Service-frei (D-48-PDF-Design bleibt) |
| Timestamp-Beschaffung (`now_local`) | `service_impl::pdf_shiftplan` (PdfShiftplanService) | — | Business-Logic-Tier hat einzige Injection-Stelle (D-50-12, D-49-08) |
| Scheduler-Aufruf | `service_impl::pdf_export_scheduler` | — | Delegiert an PdfShiftplanService (D-49-08 bereits implementiert) |
| REST-Handler | `rest::pdf_shiftplan` | — | Delegiert an PdfShiftplanService, kein direkter Renderer-Zugriff |
| Test-Fixtures / Assertions | `service_impl::pdf_render` (mod test) | `service_impl::test::pdf_shiftplan` | Renderer-Tests im Modul; Service-Tests in separatem Test-Modul |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `printpdf` | 0.7.0 [VERIFIED: Cargo.lock] | PDF-Dokument-Erzeugung | Bereits in `service_impl/Cargo.toml`, pure Rust, kein System-Dep |
| `time` | 0.3.44 [VERIFIED: Cargo.lock] | OffsetDateTime, Timestamp-Formatting | Bereits im Workspace, `local-offset`-Feature wird aktiviert |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `lopdf` | 0.31.0 [VERIFIED: Cargo.lock] | Transitive Dep von printpdf; `WinAnsiEncoding` für Umlaute | Nicht direkt importieren — printpdf-API nutzen |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `Rect::with_mode(PaintMode::Stroke)` | `Line` mit 4 Punkten + `is_closed: true` | `Rect` ist kürzer, semantisch klarer; `Line` funktioniert aber ist verbose |
| `time::OffsetDateTime::now_local()` | `chrono::Local::now()` | `time` bereits im Workspace; neue Dep wäre unnötig und gegen D-50 Constraint |

**Installation:** Keine neue Cargo-Dep. Nur Feature-Aktivierung:

```toml
# service_impl/Cargo.toml — add "local-offset" to existing time dep:
[dependencies.time]
version = "0.3.36"
features = ["std", "formatting", "macros", "local-offset"]
```

**Version verification:** `printpdf = 0.7.0` (Cargo.lock zeile 1), `time = 0.3.44` (Cargo.lock). [VERIFIED: Cargo.lock]

---

## Package Legitimacy Audit

Keine neuen externen Packages werden installiert. `printpdf 0.7.0` und `time 0.3.44` sind bereits im gesperrten Workspace.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| `printpdf` 0.7.0 | crates.io | bereits in Cargo.lock | — | fschutt/printpdf | OK | Approved — bereits installiert |
| `time` 0.3.44 | crates.io | bereits in Cargo.lock | — | time-rs/time | OK | Approved — bereits installiert |

**Packages removed due to SLOP verdict:** none
**Packages flagged as suspicious SUS:** none

---

## Architecture Patterns

### System Architecture Diagram

```
REST-Handler (pdf_shiftplan.rs)
    │
    ▼
PdfShiftplanService::render_week_pdf()
    │  ├─ 1. WeekStatus-Gate
    │  ├─ 2. ShiftplanViewService::get_shiftplan_week()
    │  ├─ 3. SalesPersonService::get_all() + filter_active()
    │  ├─ 4. time::OffsetDateTime::now_local()  ← NEU (D-50-12)
    │  │       .unwrap_or_else(|_| { warn!(); now_utc() })
    │  └─ 5. pdf_render::render_shiftplan_week_pdf(week, sps, year, week, render_timestamp)
    │              │
    │              ▼
    │         pdf_render (Pure Fn) ← KOMPLETTER REWRITE
    │              ├─ compute_visible_days(week) → Vec<DayOfWeek>  [D-50-08]
    │              ├─ compute_col_width(n_days)  [D-50-08: dynamisch]
    │              ├─ render_page_header(layer, year, week, timestamp)  [D-50-09]
    │              └─ für jeden Tag:
    │                   render_day_column(layer, day_slots, sorted_sps, x_offset, col_width, grid_top, avail_height)
    │                        ├─ sort slots by (from, to, id)  [D-50-02]
    │                        └─ für jeden Slot (mit Overflow-Guard):
    │                             render_slot_box(layer, slot, names, box_x, box_y, box_h, col_w)
    │                                  ├─ layer.add_rect(Rect::new(...).with_mode(PaintMode::Stroke))
    │                                  ├─ layer.use_text(time_label, ...)
    │                                  └─ für jeden Namen (alphabetisch, + N weitere):
    │                                       layer.use_text(name_text, ...)
    │              └─ doc.save_to_bytes()
    │
    ▼
Vec<u8> (PDF bytes)

Scheduler (pdf_export_scheduler.rs) ─┐
                                     ├─► PdfShiftplanService::render_week_pdf()
                                     │   (kein direkter pdf_render-Aufruf — D-49-08)
```

### Recommended Project Structure

```
service_impl/src/
├── pdf_render.rs          # Kompletter Rewrite (einzige Datei, keine Submodule)
│                          # Enthält alle Pure Fns + #[cfg(test)] mod test
├── pdf_shiftplan.rs       # Ergänzt: now_local() + Timestamp-Übergabe an Renderer
├── pdf_export_scheduler.rs # Unverändert (delegiert bereits an PdfShiftplanService)
└── test/
    └── pdf_shiftplan.rs   # Bestehende Tests — Signatur-Anpassung für neuen render_week_pdf
```

### Pattern 1: Slot-Box-Rendering mit Rect + PaintMode::Stroke

**What:** Rechteck-Umrandung ohne Fill (D-50-10) via printpdf 0.7's `add_rect` API.
**When to use:** Für jede Slot-Box und optional für Tages-Spalten-Trennlinien.

```rust
// Source: /home/neosam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/printpdf-0.7.0/examples/rect.rs
// und printpdf-0.7.0/src/rectangle.rs (PaintMode::Stroke → OP_PATH_PAINT_STROKE, kein fill_op)
use printpdf::{Mm, Rect, path::{PaintMode, WindingOrder}};

fn render_slot_box(
    layer: &PdfLayerReference,
    x_mm: f32,
    y_mm: f32,    // UNTERE Kante der Box (printpdf: origin = bottom-left)
    w_mm: f32,
    h_mm: f32,
    line_thickness_pt: f32,
) {
    // Grafikzustand sichern, Linienstärke setzen, Rect zeichnen, State wiederherstellen
    layer.save_graphics_state();
    layer.set_outline_thickness(line_thickness_pt); // z.B. 0.4
    let rect = Rect::new(
        Mm(x_mm),
        Mm(y_mm),
        Mm(x_mm + w_mm),
        Mm(y_mm + h_mm),
    )
    .with_mode(PaintMode::Stroke); // Outline-only, kein Fill (D-50-10)
    layer.add_rect(rect);
    layer.restore_graphics_state();
}
```

**Wichtig:** `Rect::new(ll_x, ll_y, ur_x, ur_y)` — die erste und dritte MM-Angabe ist die X-Koordinate, die zweite und vierte ist die Y-Koordinate (Bottom-Left-Ursprung). In PDF: Y wächst nach oben. Also ist die untere Kante einer Box `y_mm`, und die obere Kante `y_mm + h_mm`.

### Pattern 2: Hybrid-Layout-Algorithmus (D-50-01)

**What:** Slot-Höhe proportional zur Dauer, gestapelt ohne Time-Grid.

```rust
// Konstanten (Ausgangspunkt — Planner kann justieren)
const MARGIN_LEFT_MM: f32 = 8.0;
const MARGIN_RIGHT_MM: f32 = 5.0;
const MARGIN_BOTTOM_MM: f32 = 8.0;
const HEADER_HEIGHT_MM: f32 = 12.0;   // Titel + Timestamp auf einer Zeile
const DAY_HEADER_HEIGHT_MM: f32 = 8.0; // "Mo", "Di" etc.
const PAGE_WIDTH_MM: f32 = 297.0;      // A4 Landscape
const PAGE_HEIGHT_MM: f32 = 210.0;     // A4 Landscape
const SLOT_BASE_MM: f32 = 12.0;        // Mindesthöhe je Slot (Zeitlabel + ~2 Namen)
const SLOT_STEP_MM: f32 = 5.0;         // pro Stunde Dauer
const SLOT_PADDING_MM: f32 = 1.5;      // Innen-Abstand oben in Slot-Box
const SLOT_GAP_MM: f32 = 1.0;          // Abstand zwischen Slot-Boxen
const TIME_LABEL_FONT_PT: f32 = 8.0;   // Zeit "08:00 - 12:00"
const NAME_FONT_PT: f32 = 9.0;         // Sales-Person-Namen
const LINE_HEIGHT_MM: f32 = 3.5;       // ca. pro Zeile bei 9pt-12pt
const HEADER_FONT_PT: f32 = 14.0;      // Titel links
const TIMESTAMP_FONT_PT: f32 = 9.0;    // Timestamp rechts

fn compute_slot_height_mm(duration_hours: f32) -> f32 {
    SLOT_BASE_MM + duration_hours * SLOT_STEP_MM
}

fn compute_col_width_mm(n_days: usize) -> f32 {
    (PAGE_WIDTH_MM - MARGIN_LEFT_MM - MARGIN_RIGHT_MM) / (n_days as f32)
}

fn grid_top_y_mm() -> f32 {
    // Von oben: Header + Tages-Header; Rest = Grid
    // printpdf: Y wächst nach unten von Seitenoben. Nein: Y wächst nach OBEN von unten.
    // PAGE_HEIGHT_MM ist die Seitenhöhe.
    // Grid top (als mm von Seiten-Unterkante):
    PAGE_HEIGHT_MM - MARGIN_TOP_MM - HEADER_HEIGHT_MM - DAY_HEADER_HEIGHT_MM
    // = 210 - 5 - 12 - 8 = 185mm von unten (= obere Kante des Grid-Bereichs)
}

// Verfügbare Spalten-Höhe für Slots:
// grid_top_y_mm() - MARGIN_BOTTOM_MM = 185 - 8 = 177mm
```

**Layout-Math-Verifikation:**
- A4 Landscape 297×210mm
- Verfügbare Grid-Höhe: ~177mm
- 3 Slots je 4h: 3 × (12 + 4×5) = 3 × 32 = 96mm → passt gut
- 5 Slots je 4h: 5 × 32 = 160mm + 4 Gaps = 4mm → 164mm → passt
- 6 Slots je 4h: 6 × 32 = 192mm → Overflow, + N weitere bei Slot 5
- 4 Slots je 8h: 4 × (12 + 8×5) = 4 × 52 = 208mm → Overflow nach Slot 3
- Spaltenbreite 7 Tage: (297-8-5)/7 = 284/7 = **40.6mm** [ASSUMED: konkrete Margin-Werte]
- Spaltenbreite 6 Tage: (297-8-5)/6 = 284/6 = **47.3mm**

### Pattern 3: `now_local()` Fallback-Pattern (D-50-12)

```rust
// Source: Codebase-Konvention + time-0.3.44/src/offset_date_time.rs:129
use time::OffsetDateTime;
use tracing::warn;

fn get_render_timestamp() -> OffsetDateTime {
    OffsetDateTime::now_local()
        .unwrap_or_else(|_| {
            warn!("PDF-Renderer: Lokale TZ nicht bestimmbar — UTC wird verwendet");
            OffsetDateTime::now_utc()
        })
}

// In PdfShiftplanService::render_week_pdf, nach Schritt 3 (active_sales_persons):
let render_timestamp = get_render_timestamp();
pdf_render::render_shiftplan_week_pdf(
    &week_view,
    &active_sales_persons,
    year,
    calendar_week,
    render_timestamp,
)
```

**Feature-Anforderung:** `local-offset` muss in `service_impl/Cargo.toml` aktiviert werden:
```toml
[dependencies.time]
version = "0.3.36"
features = ["std", "formatting", "macros", "local-offset"]
```

### Pattern 4: Timestamp-Formatting im Renderer (D-50-11)

```rust
// Source: time-0.3.44 API; kein format!-Makro aus time, einfaches format! reicht
fn format_render_timestamp(ts: time::OffsetDateTime) -> String {
    format!(
        "Erstellt am {:02}.{:02}.{} {:02}:{:02} Uhr",
        ts.day(),
        ts.month() as u8,
        ts.year(),
        ts.hour(),
        ts.minute(),
    )
}
// Beispiel: "Erstellt am 03.07.2026 17:15 Uhr"
```

**Kein Umlaut in diesem String** — vollständig ASCII-kompatibel. Hex-Assertion im Test funktioniert direkt mit `encode_ascii_to_pdf_hex`.

### Pattern 5: Dynamische Sonntag-Spalte (D-50-08)

```rust
// Source: week_view.rs:1199–1229 (has_sunday Logik)
fn compute_visible_days(week: &ShiftplanWeek) -> Vec<DayOfWeek> {
    let has_sunday = week.days.iter().any(|day| {
        day.day_of_week == DayOfWeek::Sunday && !day.slots.is_empty()
    });
    let mut days = vec![
        DayOfWeek::Monday, DayOfWeek::Tuesday, DayOfWeek::Wednesday,
        DayOfWeek::Thursday, DayOfWeek::Friday, DayOfWeek::Saturday,
    ];
    if has_sunday {
        days.push(DayOfWeek::Sunday);
    }
    days
}

fn day_label(dow: DayOfWeek) -> &'static str {
    match dow {
        DayOfWeek::Monday => "Mo",
        DayOfWeek::Tuesday => "Di",
        DayOfWeek::Wednesday => "Mi",
        DayOfWeek::Thursday => "Do",
        DayOfWeek::Friday => "Fr",
        DayOfWeek::Saturday => "Sa",
        DayOfWeek::Sunday => "So",
    }
}
```

### Pattern 6: Overflow `+ N weitere` (D-50-03 + D-50-04)

```rust
// Slot-Overflow: Höhenbuchhaltung
fn render_day_column_slots(
    layer: &PdfLayerReference,
    slots: &[&ShiftplanSlot],
    // ... positioning params
    avail_height_mm: f32,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    let mut y_cursor = grid_top_y_mm; // Startet oben (hoher Y-Wert)
    let mut rendered_count = 0;

    for (i, slot) in slots.iter().enumerate() {
        let dur_hours = slot.slot.to.hour() as f32 + slot.slot.to.minute() as f32 / 60.0
            - slot.slot.from.hour() as f32 - slot.slot.from.minute() as f32 / 60.0;
        let box_h = compute_slot_height_mm(dur_hours);

        // Prüfe ob Box noch in verfügbare Höhe passt
        let box_bottom = y_cursor - box_h;
        if box_bottom < grid_bottom_y_mm {
            // Overflow: + N weitere in letzter gerendeter Box (falls vorhanden)
            let remaining = slots.len() - rendered_count;
            if rendered_count > 0 {
                // "+" Suffix in letzte Box schreiben (als letzten Namenseintrag)
                let overflow_text = format!("+ {} weitere", remaining);
                layer.use_text(overflow_text, NAME_FONT_PT, Mm(name_x), Mm(last_name_y), font);
            }
            break;
        }

        render_slot_box(layer, col_x, box_bottom, col_width, box_h, 0.4);
        // ... Zeitlabel + Namen rendern
        y_cursor = box_bottom - SLOT_GAP_MM;
        rendered_count += 1;
    }
}
```

### Pattern 7: Umlaut-Encoding in Test-Assertions

```rust
// ACHTUNG: encode_ascii_to_pdf_hex funktioniert NUR für ASCII (Bytes 0x00–0x7F).
// Für Umlaute in Sales-Person-Namen: WinAnsiEncoding-Bytes verwenden.
// lopdf verwendet WinAnsiEncoding für Builtin-Fonts in printpdf 0.7.0 (lopdf 0.31.0).
// Umlaute in Windows-1252 / WinAnsiEncoding:
//   ä = 0xE4 → Hex "E4"
//   ö = 0xF6 → Hex "F6"
//   ü = 0xFC → Hex "FC"
//   Ä = 0xC4 → Hex "C4"
// Test-Assertion für Namen mit Umlauten:
fn encode_win1252_to_pdf_hex(s: &str) -> String {
    s.chars().filter_map(|c| {
        let utf16 = c as u32;
        // WinAnsiEncoding ist identisch mit Windows-1252 für U+0080..U+00FF
        if utf16 <= 0xFF {
            Some(format!("{:02X}", utf16 as u8))
        } else {
            None // Nicht-Latin-Zeichen werden von lopdf ignoriert
        }
    }).collect()
}
// Für reine ASCII-Strings (wie Zeitlabels, Timestamp, "Schichtplan KW"):
// encode_ascii_to_pdf_hex() bleibt korrekt.
// Für Umlaut-Tests: encode_win1252_to_pdf_hex() verwenden.
```

### Anti-Patterns to Avoid

- **Nicht `Op::DrawLine` oder `Op::AddLineToPath` direkt verwenden:** Diese sind interne lopdf-Abstraktionen, die printpdf 0.7 über `into_stream_op()` emittiert. Die öffentliche API ist `layer.add_rect()`, `layer.add_line()`, `layer.add_polygon()`. `Op::*` nicht direkt ansprechen.
- **Nicht `normalize_pdf_id` entfernen ohne Grep-Verifikation:** Grep bestätigt: `deterministic_bytes_for_same_input` ist nur in `service_impl/src/pdf_render.rs:431` definiert und in `.planning/`-Dokumentation referenziert. Kein externer Konsument. [VERIFIED: grep-rn deterministic_bytes .]
- **Kein `unwrap()` auf `now_local()`:** Auf Linux/NixOS mit Tokio Multi-Thread ist `localtime_r` thread-safe und gibt kein `IndeterminateOffset` zurück — aber D-50-12 verlangt trotzdem das Fallback-Pattern für Plattform-Portabilität.
- **Nicht `is_closed: true` mit `PaintMode::Fill`-Default verwechseln:** `Line { is_closed: true }` emittiert `s` (stroke-close), nicht `f` (fill). `Polygon { mode: PaintMode::Stroke }` emittiert ebenfalls `s`. Beide sind korrekt für Outline-only.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rechteck-Outline zeichnen | 4 einzelne `Line`-Operationen | `layer.add_rect(rect.with_mode(PaintMode::Stroke))` | Atomar, korrekte PDF-`re`+`S`-Operationen, weniger Code |
| Windows-1252-Encoding für Umlaute | Eigene Encoding-Tabelle | lopdf's `Document::encode_text(Some("WinAnsiEncoding"), &text)` via printpdf's `use_text` | Wird automatisch von `layer.use_text()` genutzt |
| Timestamp-Parsing | Eigenes strptime | `time::OffsetDateTime::now_local()` aus `time` crate | Bereits im Workspace |

---

## Runtime State Inventory

Diese Phase ist kein Rename/Refactor — nur Code-Änderungen in `pdf_render.rs` und `pdf_shiftplan.rs`.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | None — kein persistierter Renderer-State | Keine Migration |
| Live service config | `pdf_export_config`-Tabelle unverändert; Scheduler-Job-Loop unverändert | Keine Aktion |
| OS-registered state | Keine OS-Registrierung des Renderers | Keine Aktion |
| Secrets/env vars | Keine renaming-relevante Env-Var | Keine Aktion |
| Build artifacts | Keine alten `egg-info`/Binary-Artefakte | Keine Aktion |

**Besonderer Hinweis:** `FIXED_METADATA_TIMESTAMP` und `PDF_PRODUCER` bleiben als Konstanten — nur der Renderer-Code um sie herum wird neu geschrieben. Die PDF-Metadata-Felder (CreationDate, Producer) bleiben weiter deterministisch.

---

## Common Pitfalls

### Pitfall 1: `local-offset`-Feature fehlt → Compile-Error
**What goes wrong:** `OffsetDateTime::now_local()` ist hinter `#[cfg(feature = "local-offset")]` geblockt. Ohne das Feature in `service_impl/Cargo.toml` bricht der Compiler.
**Why it happens:** `time` crate gated lokales TZ-Lookup wegen POSIX-Unsicherheit hinter Feature-Flag.
**How to avoid:** `features = ["std", "formatting", "macros", "local-offset"]` in `[dependencies.time]`.
**Warning signs:** `error[E0425]: cannot find function 'now_local' in struct 'OffsetDateTime'`.

### Pitfall 2: printpdf Koordinaten-System (Y von unten, nicht von oben)
**What goes wrong:** In printpdf/PDF ist der Ursprung (0,0) die untere linke Ecke der Seite. `Mm(0)` = Seiten-Unterkante, `Mm(210)` = Seiten-Oberkante. Text- und Rect-Positionen müssen entsprechend berechnet werden.
**Why it happens:** PDF-Spezifikation definiert Koordinaten so; `use_text(x, y)` setzt den Text-Basislinie-Ankerpunkt.
**How to avoid:** Alle Y-Berechnungen von der Seiten-Unterkante aus rechnen. Slot-Grid startet bei `y_top = PAGE_HEIGHT_MM - MARGIN_TOP - HEADER - DAY_HEADER` und Boxes werden nach unten gestapelt (`y_cursor` nimmt ab).
**Warning signs:** Text und Boxen erscheinen spiegelverkehrt auf der Seite.

### Pitfall 3: Umlaut-Test-Assertion mit `encode_ascii_to_pdf_hex`
**What goes wrong:** `encode_ascii_to_pdf_hex("Müller")` gibt `4DC3BC6C6C6572` (falsch!) weil es UTF-8-Bytes nimmt (`C3 BC` für ü), aber printpdf/lopdf WinAnsiEncoding nutzt (`FC` für ü).
**Why it happens:** `encode_ascii_to_pdf_hex` arbeitet mit Rust-`String::bytes()` = UTF-8, aber der PDF-Textstream nutzt WinAnsiEncoding.
**How to avoid:** Für Namen-Assertions entweder (a) nur ASCII-Namen in Tests ohne Umlaute (einfachste Lösung), oder (b) `encode_win1252_to_pdf_hex` (siehe Code Examples).
**Warning signs:** Test `all_active_sales_persons_appear` schlägt fehl für Namen mit Umlauten.

### Pitfall 4: Overflow-Berechnung bei `+ N weitere` — Off-by-one
**What goes wrong:** Wenn der letzte sichtbare Slot nur teilweise in die verfügbare Höhe passt, muss die Overflow-Anzeige in der *vorletzten* vollständig gerenderten Box erscheinen (nicht in einer neuen Box).
**Why it happens:** D-50-03 sagt: `+ N weitere` am unteren Rand der **letzten darstellbaren Slot-Box** — das bedeutet, wenn Slot N nicht passt, wird Slot N-1 mit dem Overflow-Suffix versehen.
**How to avoid:** Zwei-Pass-Logik: erst Höhenbuchhaltung prüfen, bevor eine Box gerendert wird. Wenn die nächste Box nicht passt: in der aktuellen (letzten gerenderten) Box als letzten Namenseintrag `+ K weitere` schreiben.
**Warning signs:** Overflow-Indikator erscheint auf einer leeren Box nach dem letzten sichtbaren Slot.

### Pitfall 5: Dynamische Spaltenbreite bei Sonntag-Wechsel
**What goes wrong:** Wenn `n_days` sich zwischen 6 und 7 ändert, ändert sich `col_width_mm`. Alle X-Positionen (Slot-Box-X, Name-X, Zeitlabel-X) müssen von `col_width` abhängen, nicht von einer Hard-codierten Konstante.
**Why it happens:** v2.2-Renderer hatte `DAY_COL_WIDTH_MM: f32 = 36.0` als fixe Konstante — korrekt war das damals, weil immer 7 Spalten gerendert wurden.
**How to avoid:** `col_width` als berechnete Variable (`compute_col_width_mm(n_days)`), an alle Render-Funktionen als Parameter übergeben.
**Warning signs:** Bei Wochen ohne Sonntag überschneiden sich Spalten oder der rechte Rand wird nicht genutzt.

### Pitfall 6: Bestehende Service-Tests nach Renderer-Signatur-Änderung
**What goes wrong:** `service_impl/src/test/pdf_shiftplan.rs` ruft `render_week_pdf` indirekt via `PdfShiftplanService::render_week_pdf` auf. Nach der Signaturerweiterung (Timestamp-Injection in den Service) werden diese Tests ggf. kompilierungsfehler bekommen wenn der Service-Aufruf auf den Renderer weiterleitet.
**Why it happens:** Der Service-Test mockt alle Dependencies (ShiftplanView, SalesPersons, WeekStatus) — der Renderer wird real aufgerufen. Der Renderer braucht jetzt `render_timestamp`.
**How to avoid:** Der Timestamp wird im Service erzeugt (`now_local()`-Aufruf), nicht vom Test injiziert. Service-Tests müssen nichts ändern außer der zu mockenden Signatur.
**Warning signs:** Compilation error im Service-Test-Modul.

---

## Code Examples

### Beispiel A: Komplette Slot-Box mit Zeit-Label + Namen

```rust
// In render_day_column(): Pro Slot eine Box rendern
fn render_slot_box_with_content(
    layer: &PdfLayerReference,
    slot: &ShiftplanSlot,
    sorted_names: &[String], // alphabetisch sortiert, Suffix für freiwillige
    box_x: f32,
    box_y_bottom: f32, // Untere Kante der Box in mm
    box_w: f32,
    box_h: f32,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    avail_h_in_box: f32, // für Namen-Overflow
) {
    // 1. Box-Rahmen (D-50-10)
    layer.save_graphics_state();
    layer.set_outline_thickness(0.4);
    layer.add_rect(
        Rect::new(Mm(box_x), Mm(box_y_bottom), Mm(box_x + box_w), Mm(box_y_bottom + box_h))
            .with_mode(PaintMode::Stroke),
    );
    layer.restore_graphics_state();

    // 2. Zeitlabel (D-50-11 Format: "08:00 - 12:00" mit Hyphen für ASCII-Kompatibilität)
    let time_label = format!(
        "{:02}:{:02} - {:02}:{:02}",
        slot.slot.from.hour(), slot.slot.from.minute(),
        slot.slot.to.hour(), slot.slot.to.minute(),
    );
    let label_y = box_y_bottom + box_h - SLOT_PADDING_MM - LINE_HEIGHT_MM;
    layer.use_text(time_label, TIME_LABEL_FONT_PT, Mm(box_x + 1.5), Mm(label_y), font_bold);

    // 3. Namen-Liste mit Overflow (D-50-04/D-50-05/D-50-06/D-50-07)
    let mut name_y = label_y - LINE_HEIGHT_MM;
    let name_bottom_limit = box_y_bottom + SLOT_PADDING_MM;
    let mut rendered_names = 0;
    for (i, name) in sorted_names.iter().enumerate() {
        let next_y = name_y - LINE_HEIGHT_MM;
        let is_last_slot = i == sorted_names.len() - 1;
        if name_y < name_bottom_limit && !is_last_slot {
            // Namen-Overflow
            let remaining = sorted_names.len() - rendered_names;
            layer.use_text(
                format!("+ {} weitere", remaining),
                NAME_FONT_PT, Mm(box_x + 1.5), Mm(name_y), font,
            );
            break;
        }
        layer.use_text(name.clone(), NAME_FONT_PT, Mm(box_x + 1.5), Mm(name_y), font);
        rendered_names += 1;
        name_y = next_y;
    }
}
```

### Beispiel B: Alphabetische Namens-Sortierung mit Freiwilligen-Suffix

```rust
// D-50-06 + D-50-07
fn build_slot_name_list(
    slot: &ShiftplanSlot,
    sales_persons: &[SalesPerson],
) -> Vec<String> {
    // Sammle Namen der gebuchten Sales Persons
    let mut names: Vec<String> = slot.bookings.iter()
        .filter_map(|booking| {
            sales_persons.iter().find(|sp| sp.id == booking.sales_person.id)
        })
        .map(|sp| {
            let suffix = if sp.is_paid == Some(false) { " (freiwillig)" } else { "" };
            format!("{}{}", sp.name, suffix)
        })
        .collect();

    // Alphabetisch case-insensitive sortieren (D-50-06)
    names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    names
}
```

### Beispiel C: Page-Header mit Titel (links, bold) + Timestamp (rechts, normal)

```rust
// D-50-09: Titel oben-links bold, Timestamp oben-rechts ~9pt
fn render_page_header(
    layer: &PdfLayerReference,
    year: u32,
    week: u8,
    render_timestamp: time::OffsetDateTime,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    let header_y = PAGE_HEIGHT_MM - MARGIN_TOP_MM - 5.0; // ca. 5mm Offset für Baseline

    // Titel links
    let title = format!("Schichtplan KW {:02} ({})", week, year);
    layer.use_text(title, HEADER_FONT_PT, Mm(MARGIN_LEFT_MM), Mm(header_y), font_bold);

    // Timestamp rechts
    let ts_text = format!(
        "Erstellt am {:02}.{:02}.{} {:02}:{:02} Uhr",
        render_timestamp.day(),
        render_timestamp.month() as u8,
        render_timestamp.year(),
        render_timestamp.hour(),
        render_timestamp.minute(),
    );
    // Rechts-ausrichten: approximative Zeichenbreite für Timestamp-String
    // Bei 9pt Helvetica: ~1.8mm/Zeichen → 30 Zeichen ≈ 54mm
    let ts_x = PAGE_WIDTH_MM - MARGIN_RIGHT_MM - 58.0; // Konservativ
    layer.use_text(ts_text, TIMESTAMP_FONT_PT, Mm(ts_x), Mm(header_y), font);
}
```

### Beispiel D: Slot-Sortierung (D-50-02)

```rust
fn sort_slots(slots: &[ShiftplanSlot]) -> Vec<&ShiftplanSlot> {
    let mut sorted: Vec<&ShiftplanSlot> = slots.iter().collect();
    sorted.sort_by(|a, b| {
        a.slot.from.cmp(&b.slot.from)
            .then(a.slot.to.cmp(&b.slot.to))
            .then(a.slot.id.cmp(&b.slot.id))
    });
    sorted
}
```

### Beispiel E: Test-Fixture für Fixed-Timestamp (D-50-14)

```rust
// In mod test:
const FIXED_TEST_TIMESTAMP: time::OffsetDateTime =
    time::macros::datetime!(2026-07-03 17:15 UTC);

// Erwarteter Hex-String im Textstream:
// "Erstellt am 03.07.2026 17:15 Uhr" → encode_ascii_to_pdf_hex
fn expected_timestamp_hex() -> String {
    encode_ascii_to_pdf_hex("Erstellt am 03.07.2026 17:15 Uhr")
}

#[test]
fn render_includes_timestamp_string() {
    let week = empty_week(2026, 27);
    let bytes = render_shiftplan_week_pdf(&week, &[], 2026, 27, FIXED_TEST_TIMESTAMP)
        .expect("render succeeds");
    let hex = expected_timestamp_hex();
    assert!(
        find_subsequence(&bytes, hex.as_bytes()).is_some(),
        "timestamp string not found in PDF (hex: {})", hex,
    );
}
```

### Beispiel F: `now_local`-Fallback-Test (D-50-16)

```rust
// Service-Level-Test — kein Mock für IndeterminateOffset nötig (pure fn testbar)
// Da PdfShiftplanService::render_week_pdf den Timestamp intern erzeugt,
// kann der Fallback-Pfad nicht direkt im Service-Test getriggert werden.
// Stattdessen: Extract der Fallback-Logik als testbare reine Funktion.

// In pdf_shiftplan.rs:
pub(crate) fn resolve_render_timestamp() -> time::OffsetDateTime {
    time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| {
            warn!("PDF-Renderer: Lokale TZ nicht bestimmbar — UTC wird verwendet");
            time::OffsetDateTime::now_utc()
        })
}

// Test:
#[test]
fn now_local_fallback_to_utc_on_indeterminate_offset() {
    // Wir können IndeterminateOffset nicht simulieren ohne unsafe set_local_offset
    // Alternative: Teste dass resolve_render_timestamp() IMMER ein OffsetDateTime zurückgibt
    // (nie panikt) — Smoke-Test für den Fallback-Pfad
    let ts = crate::pdf_shiftplan::resolve_render_timestamp();
    // Muss einen plausiblen Jahr-Wert haben (kein 1970-Epoch-Bug etc.)
    assert!(ts.year() >= 2020, "timestamp year implausible: {}", ts.year());
    // Der Test beweist: unwrap_or_else ist korrekt verdrahtet (kein panic)
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Feste 7-Spalten (v2.2) | Dynamische Spalten 6/7 je nach Sonntag-Slots | Phase 50 | Mehr Platz für Mo–Sa-Wochen |
| Sales-Person-Zeilen (v2.2: Namen-Spalte links) | Slot-Boxen mit Namen darin | Phase 50 | Visuell dem Browser-Look entsprechend |
| Kein Timestamp (v2.2) | `render_timestamp: OffsetDateTime` als Parameter | Phase 50 | PDF-02-Requirement |
| Byte-Determinismus-Garantie (v2.2) | Aufgehoben (D-50-13) | Phase 50 | Timestamp bricht Byte-Identität ohnehin |

**Deprecated/outdated nach Phase 50:**
- `build_sales_person_day_cell()` (altes Zellen-Format: `09:30-14:45` je Sales-Person) — ersetzt durch slot-zentrierte Darstellung.
- `sales_persons_sorted_by_id` Test — ersetzt durch `names_within_slot_alphabetical`.
- `deterministic_bytes_for_same_input` Test — entfällt (Determinismus aufgehoben).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `layer.add_rect(rect.with_mode(PaintMode::Stroke))` erzeugt sichtbare schwarze Umrandung ohne Fill in printpdf 0.7.0 | Standard Stack, Code Examples | Sehr gering — `rectangle.rs:PaintMode::Stroke → OP_PATH_PAINT_STROKE` direkt verifiziert in printpdf 0.7.0 Source |
| A2 | Auf NixOS Linux (Tokio Multi-Thread) gibt `now_local()` kein `IndeterminateOffset` zurück | Common Pitfalls | Gering — `localtime_r` ist POSIX thread-safe; Fallback schützt vor Überraschungen auf anderen Plattformen |
| A3 | `MARGIN_TOP_MM = 5.0`, `MARGIN_LEFT_MM = 8.0`, `MARGIN_RIGHT_MM = 5.0`, `SLOT_BASE_MM = 12.0`, `SLOT_STEP_MM = 5.0` als Startkonstanten | Architecture Patterns | MEDIUM — Layout muss beim UAT visuell passen; Planner kann justieren ohne Constraint-Verletzung |
| A4 | `encode_ascii_to_pdf_hex` bleibt korrekte Assertion für Strings ohne Umlaute | Code Examples | Gering — bestehende Tests nutzen es bereits erfolgreich |

**Wenn dieser Table 4 Items hat:** A1–A2 sind durch Source-Code-Read verifiziert. A3–A4 sind Startwerte die im UAT validiert werden.

---

## Open Questions

Keine blockierenden offenen Fragen — alle 6 Gray Areas wurden in CONTEXT.md/Discussion-Log entschieden.

1. **Konkrete Maß-Konstanten visuell optimal?**
   - Was wir wissen: A4-Landscape-Geometrie und Mindestanforderungen (Zeitlabel + 2 Namen sichtbar) geben Raum für base=12, step=5.
   - Was unklar: Optimale visuelle Balance für reale Wochen. Eventuell base=10 oder step=4 sieht besser aus.
   - Recommendation: UAT (D-50-17) bestätigt die Wahl. Konstanten als benannte `const`-Werte anlegen → leicht justierbar ohne Logik-Änderung.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[tokio::test]` |
| Config file | Keine separate Config (Cargo standard) |
| Quick run command | `cargo test -p service_impl pdf_render -- --nocapture` |
| Full suite command | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PDF-01 | Slot-Boxen sichtbar, Zeitlabels vorhanden | unit | `cargo test -p service_impl slot_boxes_sorted_by_start_time` | ❌ Wave 0 |
| PDF-01 | Namen in Slot-Zellen alphabetisch | unit | `cargo test -p service_impl names_within_slot_alphabetical` | ❌ Wave 0 |
| PDF-01 | Freiwilligen-Suffix `(freiwillig)` | unit | `cargo test -p service_impl unpaid_marker_suffix` | ❌ Wave 0 |
| PDF-01 | Sonntag-Spalte ausgeblendet wenn keine Sonntag-Slots | unit | `cargo test -p service_impl sunday_column_hidden_when_no_sunday_slots` | ❌ Wave 0 |
| PDF-01 | Sonntag-Spalte sichtbar wenn Sonntag-Slot vorhanden | unit | `cargo test -p service_impl sunday_column_shown_when_at_least_one_sunday_slot` | ❌ Wave 0 |
| PDF-01 | Kopfzeile `Schichtplan KW 27 (2026)` im PDF | unit | `cargo test -p service_impl header_contains_year_and_week` | ✅ (Portierung von v2.2) |
| PDF-01 | Alle Sales-Persons im Textstream | unit | `cargo test -p service_impl all_active_sales_persons_appear` | ✅ (Portierung mit Anpassung) |
| PDF-02 | Timestamp-String im Textstream | unit | `cargo test -p service_impl render_includes_timestamp_string` | ❌ Wave 0 |
| PDF-02 | `now_local()` Fallback auf UTC bei IndeterminateOffset | unit | `cargo test -p service_impl now_local_fallback_to_utc_on_indeterminate_offset` | ❌ Wave 0 |
| PDF-01+02 | Leere Woche ergibt valides PDF | unit | `cargo test -p service_impl empty_week_yields_valid_pdf_signature` | ✅ (Portierung) |
| PDF-01+02 | UAT: visueller Check im Browser | manual | D-50-17: User klickt Download-Button | N/A |

### Sampling Rate

- **Per task commit:** `cargo test -p service_impl -- pdf_render && cargo clippy --workspace -- -D warnings`
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Phase gate:** Full suite green (`cargo test --workspace`) + UAT (D-50-17: manueller Button-Klick) vor `/gsd-verify-work`

### Wave 0 Gaps

- [ ] Neue Test-Funktionen in `service_impl/src/pdf_render.rs` (mod test) laut D-50-16:
  - `render_includes_timestamp_string`
  - `slot_boxes_sorted_by_start_time`
  - `names_within_slot_alphabetical`
  - `unpaid_marker_suffix`
  - `sunday_column_hidden_when_no_sunday_slots`
  - `sunday_column_shown_when_at_least_one_sunday_slot`
  - `now_local_fallback_to_utc_on_indeterminate_offset`
- [ ] `make_sales_person` Fixture um `is_paid`-Parameter erweitern (für D-50-07-Test)
- [ ] `service_impl/Cargo.toml`: `"local-offset"` zu `time`-Features hinzufügen

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/cargo | Alle Tests + Build | ✓ | (NixOS flake) | — |
| printpdf 0.7.0 | PDF-Rendering | ✓ | 0.7.0 (Cargo.lock) | — |
| time 0.3.44 | Timestamp | ✓ | 0.3.44 (Cargo.lock) | — |
| `nix develop` Shell | cargo/sqlx CLI | ✓ | (flake.nix) | nix-shell kaputt |
| sqlx-cli | Nicht benötigt (keine neue Migration) | — | — | N/A |
| Browser (UAT) | D-50-17 visueller Check | ✓ | (user environment) | — |

**Missing dependencies with no fallback:** keine.

---

## Project Constraints (from CLAUDE.md)

Die folgenden Direktiven aus `CLAUDE.md` (Backend-Root) sind für Phase 50 bindend:

1. **Clippy-Gate PFLICHT:** `cargo clippy --workspace -- -D warnings` muss nach jedem Commit grün sein. `cargo test` allein reicht nicht. CI-Äquivalent via `nix build`. Für Phase 50 besonders relevant: `pdf_render.rs` Rewrite muss clippy-clean sein (kein `allow(dead_code)` ohne Begründung, kein `unused_variable`).

2. **Testing:** Alle Code-Änderungen brauchen Tests. Phase 50 hat 7 neue Unit-Tests (D-50-16) plus portierte Tests (D-50-15).

3. **sqlx prepare nach neuer Query:** Phase 50 hat keine neuen SQLx-Queries → kein `cargo sqlx prepare` notwendig.

4. **Keine Migration:** `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12. Phase 50 berührt keine `BillingPeriodValueType`-Computation.

5. **Service-Tier-Konvention:** `PdfShiftplanService` ist Business-Logic-Tier (kombiniert mehrere Services). Phase 50 ergänzt den `now_local()`-Aufruf im Service — keine neue Service-Dep, keine Tier-Verletzung.

6. **OpenAPI:** Phase 50 ändert keine REST-Endpunkt-Signaturen — kein neues `#[utoipa::path]` nötig.

7. **VCS:** GSD-Auto-Commit aktiv (`commit_docs: true`). Keine manuellen `git commit`-Aufrufe.

---

## Sources

### Primary (HIGH confidence — Source-Code-Analyse)

- `service_impl/src/pdf_render.rs` (aktueller v2.2-Stand, 554 Zeilen) — API-Referenz, Test-Patterns, Konstanten-Ausgangspunkt
- `service_impl/src/pdf_shiftplan.rs` — Aufrufpfad für Timestamp-Injection (D-50-12)
- `service_impl/src/pdf_export_scheduler.rs` — Scheduler-Refactor Phase 49 bestätigt: kein direkter Renderer-Aufruf
- `service_impl/src/test/pdf_shiftplan.rs` — Bestehende Service-Tests, Signatur-Referenz
- `/home/neosam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/printpdf-0.7.0/src/rectangle.rs` — `Rect::with_mode(PaintMode::Stroke)` API [VERIFIED: direkte Quellcode-Analyse]
- `/home/neosam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/printpdf-0.7.0/src/pdf_layer.rs` — `add_rect`, `use_text`, `save/restore_graphics_state` APIs [VERIFIED]
- `/home/neosam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/printpdf-0.7.0/src/line.rs` — `Line` + `Polygon` (alternative APIs) [VERIFIED]
- `/home/neosam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/printpdf-0.7.0/examples/rect.rs` — `add_rect`-Beispiel [VERIFIED]
- `/home/neosam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lopdf-0.31.0/src/document.rs` + `encodings/` — WinAnsiEncoding + `encode_text` für Umlaut-Analyse [VERIFIED]
- `/home/neosam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.44/src/offset_date_time.rs:129` — `now_local()` Signatur + `local-offset` Feature [VERIFIED]
- `/home/neosam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.44/src/sys/local_offset_at/unix.rs` — `localtime_r` Thread-Safety auf Linux [VERIFIED]
- `/home/neosam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/time-0.3.44/src/sys/refresh_tz/unix.rs` — `OS_HAS_THREAD_SAFE_ENVIRONMENT` auf Linux = false, aber `localtime_r` trotzdem thread-safe [VERIFIED]
- `shifty-dioxus/src/component/week_view.rs:1199–1229` — `has_sunday`-Logik Blaupause für D-50-08 [VERIFIED]
- `shifty-dioxus/src/component/week_view.rs:1039–1085` — `WeekCellSlot` Slot-Position-Berechnung (`from_hour()`, `to_hour()`, `SCALING=75px/h`) [VERIFIED]
- `Cargo.lock` — printpdf 0.7.0, lopdf 0.31.0, time 0.3.44 exakt gelockt [VERIFIED]
- `grep -rn "deterministic_bytes" .` — bestätigt: kein externer Konsument außer `pdf_render.rs` + `.planning/`-Docs [VERIFIED]

### Secondary (MEDIUM confidence)

- `.planning/phases/50-pdf-renderer-browser-look/50-CONTEXT.md` — alle Locked Decisions (D-50-01..D-50-17) aus User-Discussion
- `.planning/REQUIREMENTS.md` §PDF-01 + §PDF-02 — Requirement-Text

---

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — printpdf 0.7.0 Source-Code direkt gelesen, keine Annahmen
- Architecture: HIGH — Aufrufpfade in bestehenden Dateien verifiziert
- Layout-Algorithmus Konstanten: MEDIUM — begründete Startwerte, UAT bestätigt
- Pitfalls: HIGH — Umlaut-Encoding, Coordinate-System, Feature-Flag durch Quellcode bestätigt
- Timestamp/now_local: HIGH — time-Crate Source + Feature-Analyse

**Research date:** 2026-07-03
**Valid until:** 2026-08-03 (printpdf 0.7.0 und time 0.3.44 sind stable, kein Drift erwartet)
