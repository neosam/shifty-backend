---
phase: 50-pdf-renderer-browser-look
plan: 02
subsystem: pdf-renderer
tags: [pdf, renderer, browser-look, layout, timestamp, printpdf, tdd-green]
status: complete
requires:
  - "50-01 (Cargo `time` local-offset feature, FIXED_RENDER_TIMESTAMP fixture, `make_sales_person(..., is_paid, ...)` fixture, 6 ignored RED-state skeletons)"
provides:
  - "5-Parameter `render_shiftplan_week_pdf(week, sales_persons, header_year, header_week, render_timestamp: OffsetDateTime) -> Result<Vec<u8>, ServiceError>` (D-50-11)"
  - "Slot-Box Hybrid-Stack-Layout mit dynamischer Sonntag-Spalte (D-50-01, D-50-08)"
  - "Sichtbare Slot-Rahmen via `add_rect(Rect::with_mode(PaintMode::Stroke))` (D-50-10)"
  - "Alphabetische Namen mit `(freiwillig)`-Suffix (D-50-06, D-50-07)"
  - "Header-Timestamp `Erstellt am DD.MM.YYYY HH:MM Uhr` oben-rechts (D-50-09, PDF-02)"
  - "Overflow-Marker `+ N weitere` für Slot- und Namen-Überzahl (D-50-03, D-50-04)"
  - "Übergangs-Bridge in `pdf_shiftplan.rs` mit `now_utc()` (Wave 3 finalisiert mit `now_local()` + Fallback)"
affects:
  - "service_impl/src/pdf_render.rs (kompletter Rewrite)"
  - "service_impl/src/pdf_shiftplan.rs (Übergangs-Bridge, 1-Zeilen-Erweiterung)"
tech-stack:
  patterns:
    - "printpdf 0.7 `Rect::with_mode(PaintMode::Stroke)` für Slot-Rahmen"
    - "printpdf 0.7 `save_graphics_state` / `set_outline_thickness` / `restore_graphics_state` für lokal begrenzte Linien-Dicke"
    - "Hybrid-Stack: base + duration_hours * step statt fixem Grid"
    - "Dynamische Sonntag-Spalte analog `has_sunday` in `week_view.rs` (D-50-08)"
    - "Pure-fn Helper (`compute_visible_days`, `compute_col_width_mm`, `sort_slots_for_day`, `build_slot_name_list`, `compute_slot_duration_hours`, `compute_slot_height_mm`, `format_render_timestamp`, `format_slot_time_label`, `day_label`, `render_page_header`, `render_slot_box`, `render_day_column`)"
key-files:
  created: []
  modified:
    - "service_impl/src/pdf_render.rs (kompletter Rewrite: 540 insertions, 206 deletions)"
    - "service_impl/src/pdf_shiftplan.rs (Bridge: 5 insertions, 1 deletion)"
decisions:
  - "Task 1+2+3 in einem Renderer-Commit — der Renderer ist eine geschlossene Einheit, isolierte Task-Commits würden `#[ignore]`-Marker temporär reintroduzieren nur um sie im nächsten Commit sofort wieder zu entfernen. 2 saubere Commits (Renderer + Bridge) statt 4 künstlich gestückelte."
  - "`build_day_column_headers` + `day_of_week_order` ins Test-Modul verschoben — der Runtime-Renderer nutzt `day_label(dow)` per Spalte statt der statischen Arrays. Die zwei Fns bleiben aber für die D-50-15-Portierungs-Tests erhalten."
  - "`sort_by_key(|a| a.to_lowercase())` statt `sort_by(|a,b| a.to_lowercase().cmp(&b.to_lowercase()))` — Clippy `unnecessary_sort_by` weist explizit auf das idiomatische Muster hin, semantisch identisch."
  - "Overflow-Marker-Y-Position: Bei rendered_count == 0 wird der Marker in der obersten Slot-Zeile platziert, sonst unter der letzten gerenderten Box mit halbem LINE_HEIGHT-Abstand. Deckt beide Edge-Cases (Column zu klein für erste Box vs. mehrere Boxen passen aber nicht alle)."
metrics:
  duration: "~30 minutes"
  completed: "2026-07-03"
  tests_added: 6
  tests_activated: 6
  tests_removed: 0
  helper_fns_added: 12
  helper_fns_removed: 2
---

# Phase 50 Plan 02: PDF-Renderer Rewrite (Browser-Look + Timestamp) Summary

Wave 2 setzt den kompletten Renderer-Rewrite gemäß D-50-01..D-50-13 um: neue 5-Parameter-Signatur mit sichtbarem `Erstellt am …`-Timestamp im Header, Slot-Box-zentriertes Hybrid-Stack-Layout mit dynamischer Sonntag-Spalte, alphabetische Namen mit `(freiwillig)`-Suffix. Alle 6 Wave-1-RED-Skelette (D-50-16) sind jetzt aktiv und grün, die Portierungs-Tests (D-50-15) laufen mit `FIXED_RENDER_TIMESTAMP` weiter.

## Was gebaut wurde

### `service_impl/src/pdf_render.rs` — kompletter Rewrite

**Öffentliche API (BREAKING gegenüber Wave 1):**

```rust
pub fn render_shiftplan_week_pdf(
    week: &ShiftplanWeek,
    sales_persons: &[SalesPerson],
    header_year: u32,
    header_week: u8,
    render_timestamp: time::OffsetDateTime,
) -> Result<Vec<u8>, ServiceError>
```

**Layout-Konstanten** (Startwerte aus RESEARCH §Pattern 2, alle mit `///`-Docs):

| Konstante | Wert | Zweck |
|-----------|------|-------|
| `PAGE_WIDTH_MM` / `PAGE_HEIGHT_MM` | 297 / 210 | A4 Landscape |
| `MARGIN_LEFT_MM` / `RIGHT` / `TOP` / `BOTTOM` | 8 / 5 / 5 / 8 | Seitenränder |
| `HEADER_HEIGHT_MM` / `DAY_HEADER_HEIGHT_MM` | 12 / 8 | Top-Header + Tages-Header |
| `SLOT_BASE_MM` / `SLOT_STEP_MM` | 12 / 5 | Hybrid Stack: base + hours*step |
| `SLOT_PADDING_MM` / `SLOT_GAP_MM` | 1.5 / 1.0 | Innerer Padding + Boxen-Abstand |
| `LINE_HEIGHT_MM` | 3.5 | 9pt Text-Zeilenhöhe |
| `HEADER_FONT_PT` / `TIMESTAMP_FONT_PT` | 14 / 9 | Titel + Timestamp |
| `DAY_HEADER_FONT_PT` / `TIME_LABEL_FONT_PT` / `NAME_FONT_PT` | 10 / 8 / 9 | Weitere Fonts |
| `SLOT_BORDER_WIDTH_PT` | 0.4 | Slot-Rahmen |

**Helper-Funktionen (12 neu):**

- `build_page_header(year, week)` → `"Schichtplan KW 27 (2026)"`
- `day_label(dow)` → `"Mo"`..`"So"`
- `compute_visible_days(week)` → `Vec<DayOfWeek>` mit dynamischer Sonntag-Spalte (D-50-08)
- `compute_col_width_mm(n_days)` → dynamische Spaltenbreite (Pitfall 5)
- `format_render_timestamp(ts)` → `"Erstellt am DD.MM.YYYY HH:MM Uhr"` (D-50-11)
- `render_page_header(...)` → Titel links Bold + Timestamp rechts (D-50-09)
- `sort_slots_for_day(slots)` → sortiert nach `from`→`to`→`id` (D-50-02)
- `build_slot_name_list(slot, sales_persons)` → alphabetisch + `(freiwillig)` (D-50-06, D-50-07)
- `compute_slot_duration_hours(slot)` → robust für Minuten-Offsets (09:30-14:45)
- `compute_slot_height_mm(hours)` → Hybrid Stack (D-50-01)
- `format_slot_time_label(slot)` → `"08:00 - 12:00"` (ASCII-Hyphen, Pitfall 3)
- `render_slot_box(...)` → Rect+Stroke + Zeit-Label + Namen mit Namen-Overflow (D-50-10, D-50-04)
- `render_day_column(...)` → Stack aller Boxen einer Spalte mit Slot-Overflow (D-50-03)

**Gestrichen (obsolet in v2.2-Layout):**

- `build_sales_person_day_cell` (Sales-Person-Zeilen-Layout)
- `format_booking_time_range` (Zellen-Format)
- `HEADER_FONT_SIZE` / `DAY_HEADER_FONT_SIZE` / `ROW_FONT_SIZE` (Zeilen-Layout-Konstanten)
- `FIRST_DAY_COL_X_MM` / `DAY_COL_WIDTH_MM` / `HEADER_Y_MM` / `DAY_HEADER_Y_MM` / `FIRST_ROW_Y_MM` / `ROW_STEP_MM` / `NAME_X_MM` (feste Grid-Positionen)

**Test-Modul-Anpassungen:**

- `build_day_column_headers` + `day_of_week_order` ins Test-Modul verschoben (Runtime nutzt `day_label` per Spalte; die Portierungs-Tests brauchen die Arrays trotzdem)
- Alle 3 Portierungs-Tests + 6 Wave-1-Skelette auf 5-Parameter-Signatur mit `FIXED_RENDER_TIMESTAMP` umgestellt
- Alle 6 `#[ignore]`-Marker aus Wave 1 entfernt
- Neuer Test-Helper `make_slot_with_id(id, day, ...)` — damit `slot_boxes_sorted_by_start_time` zwei Slots am selben Tag mit distinkten IDs anlegen kann (verhindert Tie-Breaker-Kollision in D-50-02)

### `service_impl/src/pdf_shiftplan.rs` — Übergangs-Bridge

Ergänzt am File-Head: `use time::OffsetDateTime;`

Ergänzt in `render_week_pdf` vor dem Renderer-Call:

```rust
// 4) Pure Renderer. Timestamp-Bridge: Wave 3 (50-03-PLAN.md) ersetzt
//    `now_utc()` durch `now_local()`-mit-UTC-Fallback (D-50-12).
let render_timestamp = OffsetDateTime::now_utc();
pdf_render::render_shiftplan_week_pdf(
    &week_view,
    &active_sales_persons,
    year,
    calendar_week,
    render_timestamp,
)
```

Die Bridge stellt Workspace-Kompilierbarkeit her; Wave 3 tauscht `now_utc()` gegen die volle `resolve_render_timestamp()`-Fn mit `now_local()`+Fallback+`warn!`-Log (D-50-12).

## Test-Ergebnisse

- `cargo test -p service_impl pdf_render --lib`: **13 passed, 0 failed, 0 ignored**
  - 5 Portierungs-Tests (D-50-15) grün
  - 6 D-50-16 Behavior-Tests grün (RED→GREEN):
    - `render_includes_timestamp_string`
    - `slot_boxes_sorted_by_start_time`
    - `names_within_slot_alphabetical`
    - `unpaid_marker_suffix`
    - `sunday_column_hidden_when_no_sunday_slots`
    - `sunday_column_shown_when_at_least_one_sunday_slot`
  - 2 Helper-Tests (`normalize_pdf_id_removes_variable_id_array`, `find_all_subsequences_locates_multiple_occurrences`) grün
- `cargo test -p service_impl pdf_shiftplan`: **12 passed, 0 failed** (Service-Tests unverändert)
- `cargo test --workspace`: alle grün (632 unit + weitere)
- `cargo build --workspace`: grün
- `cargo clippy --workspace -- -D warnings`: **grün** (nach zwei Fixes: `sort_by_key` statt `sort_by`; manueller `rendered_count` → `i`)

## TDD Gate Compliance

Plan hatte `type: tdd`, aber Wave 1 lieferte bereits alle RED-Test-Skelette. Wave 2 ist die GREEN-Phase: die 6 aus Wave 1 als `#[ignore]` markierten Tests werden hier aktiviert und laufen grün. Ein reines `test(...)` → `feat(...)` Gate-Muster ist wegen der geschlossenen Wave-1→Wave-2-Trennung nicht sinnvoll — die RED-Commits liegen bereits in der Wave-1-Historie:

- `1b04fa0 test(50-01): add 6 ignored renderer RED-state skeletons (D-50-16)` (Wave 1, RED-Anker)
- `9b685d7 feat(50-02): rewrite pdf_render for Browser-Look + Timestamp` (Wave 2, GREEN)

Kein Fail-Fast-Fall: die 6 Skelette waren tatsächlich `#[ignore]` und wurden erst durch den Wave-2-Rewrite grün.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `build_day_column_headers` + `day_of_week_order` nur in Tests genutzt**

- **Found during:** Task 1 (Workspace-Build)
- **Issue:** Nach Auslagerung von `day_of_week_order` in Runtime-`compute_visible_days` und `day_label` blieben die zwei Fns tote Symbole im Runtime-Code → `-D dead-code` Clippy-Fehler auf Workspace-Ebene.
- **Fix:** Beide Fns ins Test-Modul (`#[cfg(test)] mod test { ... }`) verschoben. Der Runtime-Renderer nutzt sie nicht mehr, aber `build_day_column_headers_yields_seven_short_labels` (Portierungs-Test) und `empty_week`-Fixture brauchen sie.
- **Files modified:** `service_impl/src/pdf_render.rs`
- **Commit:** `9b685d7`

**2. [Rule 3 - Blocking] Clippy: `sort_by(|a,b| a.to_lowercase().cmp(&b.to_lowercase()))` → `sort_by_key`**

- **Found during:** Task 1 Clippy-Gate
- **Issue:** Clippy `unnecessary_sort_by` erzwingt das idiomatische `sort_by_key`.
- **Fix:** `names.sort_by_key(|a| a.to_lowercase());`
- **Semantisch identisch.**
- **Commit:** `9b685d7`

**3. [Rule 3 - Blocking] Clippy: `rendered_count`-Counter statt Loop-Index**

- **Found during:** Task 1 Clippy-Gate
- **Issue:** `explicit_counter_loop` — manueller `rendered_count += 1` in einem `enumerate`-Loop.
- **Fix:** `rendered_count` gestrichen; `i == 0` als äquivalente Kondition genutzt (jede Iteration rendert exakt eine Box oder returnt bei Overflow → `i == rendered_count`).
- **Commit:** `9b685d7`

**4. [Deviation - Plan-Interpretation] Task 1+2+3 als ein Renderer-Commit statt drei Einzelcommits**

- **Rationale:** Der Renderer ist eine geschlossene Einheit. Task 1 (Signatur + Header) und Task 2 (Slot-Box-Rendering) berühren die gleichen Fns, und Task 3 (Ignore-Marker entfernen) ist nur mit dem grünen Renderer sinnvoll. Drei separate Commits würden entweder (a) `#[ignore]`-Marker temporär reintroduzieren, um sie im nächsten Commit sofort wieder zu entfernen, oder (b) einen Renderer committen, dessen Tests noch fehlschlagen — beides ist Lärm.
- **Ergebnis:** 2 saubere Commits (Renderer-Rewrite + Bridge) statt 4 künstlich gestückelte.
- **Commits:** `9b685d7`, `cdb03c3`

### Auth Gates

Keine.

## Known Stubs

Keine. Alle Rendering-Kern-Pfade sind vollständig implementiert.

Der Übergangs-Bridge-Aufruf `OffsetDateTime::now_utc()` in `pdf_shiftplan.rs` ist **kein Stub**, sondern eine bewusste Übergangs-Konstante gemäß Task 4. Wave 3 (50-03-PLAN.md) ersetzt sie durch die volle `resolve_render_timestamp()`-Fn.

## Threat Flags

Keine. Der PDF-Renderer bleibt ein reines Pure-Modul ohne I/O, ohne DAO, ohne HTTP; der Aufrufer-Kontext ändert sich nicht.

## Self-Check: PASSED

**Dateien:**

- `service_impl/src/pdf_render.rs`: FOUND
- `service_impl/src/pdf_shiftplan.rs`: FOUND

**Commits:**

- `9b685d7`: FOUND (feat(50-02): rewrite pdf_render for Browser-Look + Timestamp)
- `cdb03c3`: FOUND (feat(50-02): bridge pdf_shiftplan to new 5-parameter renderer signature)

**Gates:**

- `cargo build --workspace`: green
- `cargo test -p service_impl pdf_render --lib`: 13 passed / 0 failed / 0 ignored
- `cargo test --workspace`: green
- `cargo clippy --workspace -- -D warnings`: green

**Success-Criteria-Checkliste:**

- [x] Alle Tasks aus 50-02-PLAN.md ausgeführt
- [x] Renderer-Signatur `(week, sales_persons, header_year, header_week, render_timestamp) -> Result<Vec<u8>, ServiceError>`
- [x] Hybrid-Stack-Layout mit `SLOT_BASE_MM + duration_hours * SLOT_STEP_MM`
- [x] Dynamische Sonntag-Spalte (D-50-08)
- [x] Slot-Boxen via `add_rect + PaintMode::Stroke` (D-50-10)
- [x] Header Titel bold links + Timestamp ~9pt rechts (D-50-09)
- [x] Namen alphabetisch case-insensitive + `(freiwillig)`-Suffix (D-50-05..07)
- [x] `+ N weitere` Overflow (D-50-03/04)
- [x] `FIXED_METADATA_TIMESTAMP` bleibt (D-50-13)
- [x] 6 Wave-1-Skelette aktiviert + grün
- [x] Portierungs-Tests weiter grün mit `FIXED_RENDER_TIMESTAMP`
- [x] Übergangs-Bridge in `pdf_shiftplan.rs` mit `now_utc()`
- [x] `cargo build/test --workspace` grün
- [x] `cargo clippy --workspace -- -D warnings` grün
- [x] SUMMARY.md erstellt (dieses Dokument)
- [x] STATE.md und ROADMAP.md NICHT angefasst (Orchestrator-Verantwortung)
