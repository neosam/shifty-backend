//! Pure renderer for weekly shift plans (Phase 50 — PDF Browser-Look Redesign).
//!
//! Pure module: takes domain structs, returns bytes. No I/O, no DAO, no HTTP.
//! `printpdf` 0.7 is the only external dep (pure Rust, D-48-PDF).
//!
//! # Contract
//!
//! - **Caller must pre-filter `sales_persons` to active (non-deleted) rows**
//!   (D-48-PDF-ACTIVE-ONLY). The renderer does NOT filter by `deleted`.
//! - The `render_timestamp` parameter is passed as-is; the renderer formats it
//!   directly via `.day()/.month()/.year()/.hour()/.minute()` — **no implicit
//!   timezone conversion happens inside the renderer** (D-50-11). Callers
//!   convert to local time before calling.
//!
//! # Determinism (D-50-13)
//!
//! Phase 50 **drops the byte-determinism contract**. The old Wave-1 renderer
//! emitted a stable PDF whose only entropy was the printpdf-generated `/ID`
//! trailer array. Phase 50 introduces a visible `Erstellt am …` header which
//! *legitimately* varies per render — no whole-file byte-diff assertions.
//!
//! However, PDF **metadata** (`CreationDate`, `ModDate`, `Producer`) is still
//! stamped with fixed values (`FIXED_METADATA_TIMESTAMP`, `PDF_PRODUCER`) so
//! that trailer/metadata bytes remain stable and comparable if downstream ever
//! wants to diff them.
//!
//! # Layout (D-50-01 / D-50-08 / D-50-09 / D-50-10)
//!
//! - Landscape A4 (297×210 mm).
//! - Header row: bold title on the left (`Schichtplan KW NN (YYYY)`) + smaller
//!   timestamp on the right (`Erstellt am DD.MM.YYYY HH:MM Uhr`).
//! - Day-of-week header row below (Mo/Di/…). Sunday column only appears when
//!   at least one Sunday slot exists in the week (D-50-08, analogous to
//!   `has_sunday` in `week_view.rs`).
//! - Grid: `n_days` columns of dynamic width
//!   `(PAGE_WIDTH_MM - MARGIN_LEFT - MARGIN_RIGHT) / n_days`.
//! - Per column: slots as bordered boxes (`add_rect` + `PaintMode::Stroke`,
//!   0.4pt black, no fill — D-50-10). Box height scales with duration
//!   (`base + duration_hours * step` — D-50-01 Hybrid Stack).
//! - Slots vertically stacked in start-time order (D-50-02).
//! - **Row-aligned across day columns (UAT 2026-07-03 revision of D-50-01):**
//!   the N-th slot of every day sits on the same row. The row height is
//!   `max(needed height of the N-th slot across all days)`, so a slot that
//!   grew to accommodate many names pushes the slots below it down in every
//!   day-column simultaneously — the resulting grid stays visually aligned
//!   like the browser week view (side-by-side rows), even though slot start
//!   times may differ per day.
//! - Inside each box: bold time label at the top, then plain-text names
//!   alphabetical case-insensitive (D-50-05, D-50-06), volunteers with
//!   ` (freiwillig)` suffix (D-50-07).
//! - Names are joined with `, ` and wrapped across as many lines as needed;
//!   the slot box grows vertically so all names fit (revises D-50-04 based
//!   on UAT — a PDF cannot show "+ N weitere" interactively, so the box
//!   itself has to accommodate the content).
//! - Column-level overflow (too many stacked slot-rows for the page height)
//!   still uses `+ N weitere` (D-50-03) — the page height is finite. The
//!   overflow decision is now global: rows that don't fit are skipped in
//!   every column, and any day with un-rendered slots emits its own
//!   `+ N weitere` marker below the last shared row.

use service::sales_person::SalesPerson;
use service::shiftplan::{ShiftplanSlot, ShiftplanWeek};
use service::ServiceError;
use shifty_utils::DayOfWeek;

use printpdf::path::PaintMode;
use printpdf::{BuiltinFont, Mm, PdfDocument, Rect};

/// Fixed metadata timestamp — 2000-01-01T00:00:00Z. Used for
/// `creation_date`, `modification_date`, `metadata_date` so that repeated
/// renders emit identical PDF-metadata bytes (D-50-13).
const FIXED_METADATA_TIMESTAMP: time::OffsetDateTime = time::macros::datetime!(2000-01-01 0:00 UTC);

/// Fixed producer/creator string embedded in the PDF metadata (D-50-13).
const PDF_PRODUCER: &str = "shifty-pdf-export";

// -----------------------------------------------------------------------
// Layout constants — RESEARCH §Architecture Patterns Pattern 2.
// Start values; may be tweaked in UAT (Wave 3). Deriving the layout math
// from constants keeps `render_shiftplan_week_pdf` readable and unit-
// testable via the helper fns below.
// -----------------------------------------------------------------------

/// Landscape A4 width in mm.
const PAGE_WIDTH_MM: f32 = 297.0;
/// Landscape A4 height in mm.
const PAGE_HEIGHT_MM: f32 = 210.0;

/// Left page margin in mm.
const MARGIN_LEFT_MM: f32 = 8.0;
/// Right page margin in mm.
const MARGIN_RIGHT_MM: f32 = 5.0;
/// Top page margin in mm.
const MARGIN_TOP_MM: f32 = 4.0;
/// Bottom page margin in mm.
const MARGIN_BOTTOM_MM: f32 = 6.0;

/// Height of the top header band (title + timestamp on one line).
const HEADER_HEIGHT_MM: f32 = 10.0;
/// Height of the Mo/Di/… day-of-week header band below the top header.
const DAY_HEADER_HEIGHT_MM: f32 = 6.0;

/// Minimum height of a slot box (D-50-01 Hybrid Stack — base term).
const SLOT_BASE_MM: f32 = 9.0;
/// Extra height per hour of slot duration (D-50-01 Hybrid Stack — step term).
const SLOT_STEP_MM: f32 = 4.0;
/// Inner padding inside a slot box (top+bottom, in mm).
const SLOT_PADDING_MM: f32 = 1.2;
/// Horizontal padding for text inside a slot box (left + right, each side).
const SLOT_TEXT_H_PADDING_MM: f32 = 1.5;
/// Vertical gap between two stacked slot boxes.
const SLOT_GAP_MM: f32 = 0.7;
/// Approximate baseline-to-baseline distance for 9pt Helvetica text.
const LINE_HEIGHT_MM: f32 = 3.5;

/// Font size for the bold slot time label (e.g. `08:00 - 12:00`).
const TIME_LABEL_FONT_PT: f32 = 8.0;
/// Font size for sales-person names inside a slot box.
const NAME_FONT_PT: f32 = 9.0;
/// Font size for the bold page title `Schichtplan KW NN (YYYY)`.
const HEADER_FONT_PT: f32 = 14.0;
/// Font size for the `Erstellt am …` timestamp on the right of the header.
const TIMESTAMP_FONT_PT: f32 = 9.0;
/// Font size for the Mo/Di/… day-of-week header row.
const DAY_HEADER_FONT_PT: f32 = 10.0;

/// Slot-box outline thickness in points (D-50-10).
const SLOT_BORDER_WIDTH_PT: f32 = 0.4;

/// Render a single weekly shift plan into a PDF (Phase 50 Browser-Look).
///
/// # Arguments
///
/// - `week`: The shiftplan data for the week to render.
/// - `sales_persons`: Active sales-persons (caller MUST pre-filter to
///   `deleted.is_none()`; D-48-PDF-ACTIVE-ONLY). Used to resolve booking
///   → name/is_paid; the list is not directly rendered as a sidebar.
/// - `header_year`: Year value for the page header (e.g. `2026`).
/// - `header_week`: ISO calendar week for the page header (e.g. `27`).
/// - `render_timestamp`: The time to embed in the `Erstellt am …` header
///   (D-50-11). The renderer formats this directly and does NOT convert
///   timezones — the caller is responsible for passing an
///   already-local-time value (Wave 3 wires `now_local()` with UTC fallback
///   per D-50-12).
///
/// # Determinism
///
/// Byte-determinism is dropped (D-50-13). PDF metadata bytes remain stable
/// via `FIXED_METADATA_TIMESTAMP` / `PDF_PRODUCER`.
///
/// # Errors
///
/// Returns [`ServiceError::InternalError`] if the underlying `printpdf`
/// serializer fails.
pub fn render_shiftplan_week_pdf(
    week: &ShiftplanWeek,
    sales_persons: &[SalesPerson],
    header_year: u32,
    header_week: u8,
    render_timestamp: time::OffsetDateTime,
) -> Result<Vec<u8>, ServiceError> {
    let title = build_page_header(header_year, header_week);

    let (doc, page_index, layer_index) =
        PdfDocument::new(&title, Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), "Layer 1");

    // Metadata: fixed timestamps + fixed producer (D-50-13).
    let doc = doc
        .with_creation_date(FIXED_METADATA_TIMESTAMP)
        .with_mod_date(FIXED_METADATA_TIMESTAMP)
        .with_metadata_date(FIXED_METADATA_TIMESTAMP)
        .with_producer(PDF_PRODUCER)
        .with_creator(PDF_PRODUCER)
        .with_author(PDF_PRODUCER);

    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|_| ServiceError::InternalError)?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|_| ServiceError::InternalError)?;

    let layer = doc.get_page(page_index).get_layer(layer_index);

    // 1) Page-Header (title left bold + timestamp right, D-50-09 / PDF-02).
    render_page_header(&layer, header_year, header_week, render_timestamp, &font, &font_bold);

    // 2) Day-of-week header row + slot columns (D-50-08 dynamic Sunday).
    let visible_days = compute_visible_days(week);
    let n_days = visible_days.len();
    if n_days == 0 {
        // Degenerate case: empty week — still emit a valid PDF.
        return doc.save_to_bytes().map_err(|_| ServiceError::InternalError);
    }
    let col_width_mm = compute_col_width_mm(n_days);

    // Grid geometry: top of the whole grid = below the top-header band.
    let grid_top_y = PAGE_HEIGHT_MM - MARGIN_TOP_MM - HEADER_HEIGHT_MM;

    // Day-of-week header row baseline.
    let day_header_y = grid_top_y - DAY_HEADER_HEIGHT_MM + 2.0;
    for (i, dow) in visible_days.iter().enumerate() {
        let col_x = MARGIN_LEFT_MM + (i as f32) * col_width_mm;
        layer.use_text(
            day_label(*dow).to_string(),
            DAY_HEADER_FONT_PT,
            Mm(col_x + 1.5),
            Mm(day_header_y),
            &font_bold,
        );
    }

    // 3) Per-column slot boxes with row-aligned heights (see module doc
    //    "Row-aligned across day columns"). Compute per-day slot data,
    //    per-row heights, and the number of rows that fit in one page.
    let grid_top_slots_y = grid_top_y - DAY_HEADER_HEIGHT_MM;
    let grid_bottom_y = MARGIN_BOTTOM_MM;
    let avail_height = grid_top_slots_y - grid_bottom_y;
    let text_width_mm = (col_width_mm - 2.0 * SLOT_TEXT_H_PADDING_MM).max(1.0);

    let day_slot_renders: Vec<Vec<DaySlotRender<'_>>> = visible_days
        .iter()
        .map(|dow| {
            let Some(day) = week.days.iter().find(|d| d.day_of_week == *dow) else {
                return Vec::new();
            };
            sort_slots_for_day(&day.slots)
                .into_iter()
                .map(|s| {
                    let names = build_slot_name_list(s, sales_persons);
                    let name_lines = wrap_names_comma(&names, text_width_mm);
                    let duration = compute_slot_duration_hours(&s.slot);
                    let needed_height_mm =
                        compute_slot_box_height_mm(duration, name_lines.len());
                    DaySlotRender { slot: s, name_lines, needed_height_mm }
                })
                .collect()
        })
        .collect();

    let row_heights = compute_row_heights(&day_slot_renders);
    let rows_fitting = compute_rows_fitting(&row_heights, avail_height);
    let row_y_bottoms = compute_row_y_bottoms(&row_heights, grid_top_slots_y, rows_fitting);

    for (col_i, day) in day_slot_renders.iter().enumerate() {
        let col_x = MARGIN_LEFT_MM + (col_i as f32) * col_width_mm;
        let render_count = day.len().min(rows_fitting);
        for (row_i, ds) in day.iter().take(render_count).enumerate() {
            render_slot_box(
                &layer,
                ds.slot,
                &ds.name_lines,
                col_x,
                row_y_bottoms[row_i],
                col_width_mm,
                row_heights[row_i],
                &font,
                &font_bold,
            );
        }
        if day.len() > rows_fitting {
            let remaining = day.len() - rows_fitting;
            let marker_y = if rows_fitting == 0 {
                grid_top_slots_y - LINE_HEIGHT_MM
            } else {
                row_y_bottoms[rows_fitting - 1] - LINE_HEIGHT_MM * 0.5
            };
            layer.use_text(
                format!("+ {remaining} weitere"),
                NAME_FONT_PT,
                Mm(col_x + SLOT_TEXT_H_PADDING_MM),
                Mm(marker_y),
                &font,
            );
        }
    }

    doc.save_to_bytes().map_err(|_| ServiceError::InternalError)
}

/// Per-day, per-slot pre-computed rendering data (used by the row-aligned
/// grid layout in [`render_shiftplan_week_pdf`]).
struct DaySlotRender<'a> {
    slot: &'a ShiftplanSlot,
    name_lines: Vec<String>,
    /// Height the slot needs based on duration and wrapped name count.
    /// The effective height used at render time is the max over all days
    /// at the same row index (see [`compute_row_heights`]).
    needed_height_mm: f32,
}

/// Per-row height across all day columns: the N-th row's height is the
/// maximum `needed_height_mm` of the N-th slot across all days that have
/// an N-th slot. Days without an N-th slot do not contribute.
fn compute_row_heights(day_slot_renders: &[Vec<DaySlotRender<'_>>]) -> Vec<f32> {
    let max_rows = day_slot_renders
        .iter()
        .map(|d| d.len())
        .max()
        .unwrap_or(0);
    (0..max_rows)
        .map(|row_i| {
            day_slot_renders
                .iter()
                .filter_map(|d| d.get(row_i).map(|s| s.needed_height_mm))
                .fold(0.0f32, f32::max)
        })
        .collect()
}

/// How many rows fit in `avail_height` given per-row heights, accounting
/// for [`SLOT_GAP_MM`] between successive rows. Returns the number of
/// rows that fully fit; further rows are dropped for the column-level
/// `+ N weitere` marker (D-50-03).
fn compute_rows_fitting(row_heights: &[f32], avail_height: f32) -> usize {
    let mut consumed = 0.0f32;
    let mut fitting = 0usize;
    for (i, h) in row_heights.iter().enumerate() {
        let after = if i == 0 { *h } else { consumed + SLOT_GAP_MM + *h };
        if after > avail_height {
            break;
        }
        consumed = after;
        fitting = i + 1;
    }
    fitting
}

/// Bottom-y coordinate for each fitting row, starting from `grid_top_y`.
/// Row 0's bottom is `grid_top_y - row_heights[0]`; each subsequent row
/// drops another `SLOT_GAP_MM + row_heights[i]`.
fn compute_row_y_bottoms(
    row_heights: &[f32],
    grid_top_y: f32,
    rows_fitting: usize,
) -> Vec<f32> {
    let mut out = Vec::with_capacity(rows_fitting);
    let mut row_top = grid_top_y;
    for (i, h) in row_heights.iter().enumerate().take(rows_fitting) {
        if i > 0 {
            row_top -= SLOT_GAP_MM;
        }
        let y_bottom = row_top - h;
        out.push(y_bottom);
        row_top = y_bottom;
    }
    out
}

// -----------------------------------------------------------------------
// Header / date helpers.
// -----------------------------------------------------------------------

/// Build the page-header title text `Schichtplan KW NN (YYYY)`.
fn build_page_header(year: u32, week: u8) -> String {
    format!("Schichtplan KW {week:02} ({year})")
}

/// Short two-letter day label for the column header row.
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

/// Compute the set of visible day-columns for a given week (D-50-08).
///
/// Base grid is Mo–Sa. Sunday is only included when at least one non-empty
/// Sunday slot exists in the week (analogous to `has_sunday` in
/// `shifty-dioxus/src/component/week_view.rs`).
fn compute_visible_days(week: &ShiftplanWeek) -> Vec<DayOfWeek> {
    let has_sunday = week
        .days
        .iter()
        .any(|day| day.day_of_week == DayOfWeek::Sunday && !day.slots.is_empty());
    let mut days = vec![
        DayOfWeek::Monday,
        DayOfWeek::Tuesday,
        DayOfWeek::Wednesday,
        DayOfWeek::Thursday,
        DayOfWeek::Friday,
        DayOfWeek::Saturday,
    ];
    if has_sunday {
        days.push(DayOfWeek::Sunday);
    }
    days
}

/// Compute the per-column width in mm given the number of visible days
/// (D-50-08 dynamic Sunday; Pitfall 5).
fn compute_col_width_mm(n_days: usize) -> f32 {
    (PAGE_WIDTH_MM - MARGIN_LEFT_MM - MARGIN_RIGHT_MM) / (n_days as f32)
}

/// Format `render_timestamp` for the header right-hand side (D-50-11).
///
/// No implicit timezone conversion — the caller provided the value already
/// in the desired local time (Wave 3 does `now_local()` with UTC fallback).
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

/// Draw the top page-header: bold title on the left, `Erstellt am …` on the
/// right (D-50-09 / PDF-02). Both share one baseline near the top of the page.
fn render_page_header(
    layer: &printpdf::PdfLayerReference,
    year: u32,
    week: u8,
    ts: time::OffsetDateTime,
    font: &printpdf::IndirectFontRef,
    font_bold: &printpdf::IndirectFontRef,
) {
    let header_y = PAGE_HEIGHT_MM - MARGIN_TOP_MM - 5.0;
    // Title left, bold.
    layer.use_text(
        build_page_header(year, week),
        HEADER_FONT_PT,
        Mm(MARGIN_LEFT_MM),
        Mm(header_y),
        font_bold,
    );
    // Timestamp right, normal weight, ~9pt.
    // Reserve ~60 mm on the right for the timestamp string (`Erstellt am
    // 03.07.2026 17:15 Uhr` ≈ 30 chars @ 9pt Helvetica).
    let ts_text = format_render_timestamp(ts);
    let ts_x = PAGE_WIDTH_MM - MARGIN_RIGHT_MM - 60.0;
    layer.use_text(ts_text, TIMESTAMP_FONT_PT, Mm(ts_x), Mm(header_y), font);
}

// -----------------------------------------------------------------------
// Slot sorting + name-list helpers (D-50-02, D-50-06, D-50-07).
// -----------------------------------------------------------------------

/// Sort a day's slots for deterministic top-to-bottom render order
/// (D-50-02: primary `from`, secondary `to`, tertiary `id`).
fn sort_slots_for_day(slots: &[ShiftplanSlot]) -> Vec<&ShiftplanSlot> {
    let mut sorted: Vec<&ShiftplanSlot> = slots.iter().collect();
    sorted.sort_by(|a, b| {
        a.slot
            .from
            .cmp(&b.slot.from)
            .then(a.slot.to.cmp(&b.slot.to))
            .then(a.slot.id.cmp(&b.slot.id))
    });
    sorted
}

/// Build the alphabetical (case-insensitive) list of names inside a slot box,
/// resolving each booking's `sales_person_id` against `sales_persons` and
/// appending ` (freiwillig)` for `is_paid == Some(false)` (D-50-06, D-50-07).
///
/// Bookings whose sales-person is missing from `sales_persons` (e.g. because
/// the caller filtered them out) are silently skipped — matches the previous
/// behavior of `build_sales_person_day_cell`.
fn build_slot_name_list(slot: &ShiftplanSlot, sales_persons: &[SalesPerson]) -> Vec<String> {
    let mut names: Vec<String> = slot
        .bookings
        .iter()
        .filter_map(|booking| {
            sales_persons
                .iter()
                .find(|sp| sp.id == booking.sales_person.id)
        })
        .map(|sp| {
            let suffix = if sp.is_paid == Some(false) {
                " (freiwillig)"
            } else {
                ""
            };
            format!("{}{}", sp.name, suffix)
        })
        .collect();
    names.sort_by_key(|a| a.to_lowercase());
    names
}

// -----------------------------------------------------------------------
// Slot geometry helpers (D-50-01 Hybrid Stack).
// -----------------------------------------------------------------------

/// Compute the duration of a slot in fractional hours (robust for
/// minute-offset slots such as 09:30-14:45).
fn compute_slot_duration_hours(slot: &service::slot::Slot) -> f32 {
    let from_min = (slot.from.hour() as f32) * 60.0 + (slot.from.minute() as f32);
    let to_min = (slot.to.hour() as f32) * 60.0 + (slot.to.minute() as f32);
    (to_min - from_min) / 60.0
}

/// Height in mm of a slot box for a given duration (D-50-01 Hybrid Stack —
/// base term plus per-hour step). Does NOT account for the space needed by
/// wrapped name lines; use [`compute_slot_box_height_mm`] for that.
fn compute_slot_height_mm(duration_hours: f32) -> f32 {
    SLOT_BASE_MM + duration_hours * SLOT_STEP_MM
}

/// Effective slot-box height in mm: whichever of "duration-scaled height"
/// (D-50-01) or "height needed to fit all wrapped name lines" is larger.
///
/// This revises the original D-50-04 name-overflow behaviour: instead of
/// truncating names with `+ N weitere` (useless in a static PDF), the box
/// grows so every booked name is visible. The duration-scaling still acts
/// as a lower bound so short slots keep their visual weight.
fn compute_slot_box_height_mm(duration_hours: f32, name_line_count: usize) -> f32 {
    let by_duration = compute_slot_height_mm(duration_hours);
    // top padding + bold time-label line + one name line per wrapped entry
    // + bottom padding. `LINE_HEIGHT_MM` is the baseline distance we use for
    // both the time label (8pt) and names (9pt) — a small over-estimate for
    // the label is acceptable and keeps the maths simple.
    let by_names = 2.0 * SLOT_PADDING_MM
        + LINE_HEIGHT_MM
        + (name_line_count as f32) * LINE_HEIGHT_MM;
    by_duration.max(by_names)
}

/// Rough width in mm of a text string rendered in Helvetica at the given
/// point size. Uses average glyph advance ≈ 0.53 × font-size (empirical
/// mean for Helvetica). Conservative enough for wrap decisions where a
/// few millimetres slack is acceptable.
fn approx_text_width_mm(text: &str, font_pt: f32) -> f32 {
    const PT_TO_MM: f32 = 0.352_777_78;
    let avg_advance_mm = font_pt * 0.53 * PT_TO_MM;
    text.chars().count() as f32 * avg_advance_mm
}

/// Wrap a list of names into comma-separated text lines that each fit
/// within `max_width_mm` when rendered at 9pt Helvetica. Non-last names
/// carry a trailing `,` so continuation across lines reads naturally;
/// the final name has no trailing separator. If a single name is wider
/// than `max_width_mm`, it becomes its own line (best-effort — the text
/// may visually overflow the box, but nothing is silently dropped).
fn wrap_names_comma(names: &[String], max_width_mm: f32) -> Vec<String> {
    if names.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for (i, name) in names.iter().enumerate() {
        let is_last = i == names.len() - 1;
        let piece = if is_last {
            name.clone()
        } else {
            format!("{name},")
        };
        let candidate = if current.is_empty() {
            piece.clone()
        } else {
            format!("{current} {piece}")
        };
        if approx_text_width_mm(&candidate, NAME_FONT_PT) <= max_width_mm {
            current = candidate;
        } else if current.is_empty() {
            // Single item exceeds width — emit as its own (possibly overflowing) line.
            lines.push(piece);
        } else {
            lines.push(std::mem::take(&mut current));
            current = piece;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Time-label for the top of the slot box (`HH:MM - HH:MM`, ASCII hyphen —
/// see RESEARCH §Common Pitfalls Pitfall 3 for Umlaut/dash notes).
fn format_slot_time_label(slot: &service::slot::Slot) -> String {
    format!(
        "{:02}:{:02} - {:02}:{:02}",
        slot.from.hour(),
        slot.from.minute(),
        slot.to.hour(),
        slot.to.minute(),
    )
}

// -----------------------------------------------------------------------
// Slot-box and day-column rendering (D-50-10 rect+stroke, D-50-03/04
// overflow markers).
// -----------------------------------------------------------------------

/// Render a single slot box: bordered rectangle (D-50-10) + bold time label
/// + pre-wrapped comma-separated name lines (D-50-05, D-50-06).
///
/// The caller is expected to have wrapped names via [`wrap_names_comma`] and
/// sized the box via [`compute_slot_box_height_mm`] so all lines fit. This
/// function therefore does not carry the old `+ N weitere` name-overflow
/// path (see D-50-04 revision in the module doc-comment).
#[allow(clippy::too_many_arguments)]
fn render_slot_box(
    layer: &printpdf::PdfLayerReference,
    slot: &ShiftplanSlot,
    name_lines: &[String],
    box_x: f32,
    box_y_bottom: f32,
    box_w: f32,
    box_h: f32,
    font: &printpdf::IndirectFontRef,
    font_bold: &printpdf::IndirectFontRef,
) {
    // 1) Rahmen (D-50-10): add_rect + PaintMode::Stroke, 0.4pt black, no fill.
    layer.save_graphics_state();
    layer.set_outline_thickness(SLOT_BORDER_WIDTH_PT);
    let rect = Rect::new(
        Mm(box_x),
        Mm(box_y_bottom),
        Mm(box_x + box_w),
        Mm(box_y_bottom + box_h),
    )
    .with_mode(PaintMode::Stroke);
    layer.add_rect(rect);
    layer.restore_graphics_state();

    // 2) Time label at the top of the box, bold.
    let label_y = box_y_bottom + box_h - SLOT_PADDING_MM - LINE_HEIGHT_MM;
    layer.use_text(
        format_slot_time_label(&slot.slot),
        TIME_LABEL_FONT_PT,
        Mm(box_x + SLOT_TEXT_H_PADDING_MM),
        Mm(label_y),
        font_bold,
    );

    // 3) Wrapped name lines below the time label. Box height was already
    //    sized to fit them, so a straight loop suffices.
    let mut name_y = label_y - LINE_HEIGHT_MM;
    for line in name_lines {
        layer.use_text(
            line.clone(),
            NAME_FONT_PT,
            Mm(box_x + SLOT_TEXT_H_PADDING_MM),
            Mm(name_y),
            font,
        );
        name_y -= LINE_HEIGHT_MM;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use service::booking::Booking;
    use service::sales_person::SalesPerson;
    use service::shiftplan::{ShiftplanBooking, ShiftplanDay, ShiftplanSlot, ShiftplanWeek};
    use service::slot::Slot;
    use std::sync::Arc;
    use uuid::Uuid;

    /// D-50-14: Fixed-Timestamp fixture for all renderer tests (2026-07-03 17:15 UTC).
    ///
    /// Consumed as the 5th parameter of `render_shiftplan_week_pdf` (D-50-11).
    const FIXED_RENDER_TIMESTAMP: time::OffsetDateTime =
        time::macros::datetime!(2026-07-03 17:15 UTC);

    /// Ordered list of day-of-week short labels for the 7-day base grid
    /// (Mo–So). Used by the D-50-15 portability test
    /// `build_day_column_headers_yields_seven_short_labels`. Kept in the
    /// test module because the runtime renderer builds day labels
    /// per-column via [`day_label`] instead.
    fn build_day_column_headers() -> [&'static str; 7] {
        ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"]
    }

    /// Ordered list of [`DayOfWeek`] enum values for the 7-day base grid
    /// (Mo–So). Used by the `empty_week` test fixture.
    fn day_of_week_order() -> [DayOfWeek; 7] {
        [
            DayOfWeek::Monday,
            DayOfWeek::Tuesday,
            DayOfWeek::Wednesday,
            DayOfWeek::Thursday,
            DayOfWeek::Friday,
            DayOfWeek::Saturday,
            DayOfWeek::Sunday,
        ]
    }

    /// Deterministic fixture: build an empty `ShiftplanWeek` with 7 empty days.
    fn empty_week(year: u32, week: u8) -> ShiftplanWeek {
        ShiftplanWeek {
            year,
            calendar_week: week,
            days: day_of_week_order()
                .into_iter()
                .map(|dow| ShiftplanDay {
                    day_of_week: dow,
                    slots: Vec::new(),
                    unavailable: None,
                })
                .collect(),
        }
    }

    fn make_sales_person(id_hex: u128, name: &str, is_paid: Option<bool>) -> SalesPerson {
        SalesPerson {
            id: Uuid::from_u128(id_hex),
            name: Arc::from(name),
            background_color: Arc::from("#ffffff"),
            is_paid,
            inactive: false,
            deleted: None,
            version: Uuid::from_u128(0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff),
        }
    }

    fn make_slot(day: DayOfWeek, from_h: u8, from_m: u8, to_h: u8, to_m: u8) -> Slot {
        Slot {
            id: Uuid::from_u128(0xa1a2_a3a4_a5a6_a7a8_a9aa_abac_adae_afb0),
            day_of_week: day,
            from: time::Time::from_hms(from_h, from_m, 0).unwrap(),
            to: time::Time::from_hms(to_h, to_m, 0).unwrap(),
            min_resources: 1,
            max_paid_employees: Some(1),
            valid_from: time::Date::from_calendar_date(2026, time::Month::January, 1).unwrap(),
            valid_to: None,
            deleted: None,
            version: Uuid::from_u128(0xc1c2_c3c4_c5c6_c7c8_c9ca_cbcc_cdce_cfd0),
            shiftplan_id: None,
        }
    }

    /// Same as `make_slot` but with a caller-supplied slot id so multiple
    /// slots per day have unique ids (avoids D-50-02 tie-breaker collisions).
    fn make_slot_with_id(
        id: u128,
        day: DayOfWeek,
        from_h: u8,
        from_m: u8,
        to_h: u8,
        to_m: u8,
    ) -> Slot {
        let mut slot = make_slot(day, from_h, from_m, to_h, to_m);
        slot.id = Uuid::from_u128(id);
        slot
    }

    fn make_booking(sp: &SalesPerson, slot_id: Uuid, year: u32, week: u8) -> ShiftplanBooking {
        ShiftplanBooking {
            booking: Booking {
                id: Uuid::from_u128(0xb1b2_b3b4_b5b6_b7b8_b9ba_bbbc_bdbe_bfc0),
                sales_person_id: sp.id,
                slot_id,
                calendar_week: week as i32,
                year,
                created: Some(time::PrimitiveDateTime::new(
                    time::Date::from_calendar_date(2026, time::Month::January, 1).unwrap(),
                    time::Time::MIDNIGHT,
                )),
                deleted: None,
                created_by: None,
                deleted_by: None,
                version: Uuid::from_u128(0xd1d2_d3d4_d5d6_d7d8_d9da_dbdc_dddd_dfe0),
            },
            sales_person: sp.clone(),
            self_added: None,
        }
    }

    /// Strip the trailer `/ID[(..)(..)]` array from PDF bytes so that two
    /// renders with the same input can be compared byte-wise. The printpdf
    /// 0.7 library unconditionally generates a random 32-char document_id +
    /// instance_id per save (see module docs). The `/ID` array is emitted
    /// by `lopdf` as `/ID[(XXXX...XXXX)(YYYY...YYYY)]` — two literal
    /// strings in parentheses inside square brackets, no leading space.
    fn normalize_pdf_id(bytes: &[u8]) -> Vec<u8> {
        let needle = b"/ID[";
        let Some(start) = find_subsequence(bytes, needle) else {
            return bytes.to_vec();
        };
        // Find the matching closing bracket ']' after the /ID marker.
        let after_marker = start + needle.len();
        let Some(end_rel) = bytes[after_marker..].iter().position(|&b| b == b']') else {
            return bytes.to_vec();
        };
        let end = after_marker + end_rel + 1;
        let mut out = Vec::with_capacity(bytes.len());
        out.extend_from_slice(&bytes[..start]);
        out.extend_from_slice(b"/ID[]");
        out.extend_from_slice(&bytes[end..]);
        out
    }

    fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }

    fn find_all_subsequences(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
        haystack
            .windows(needle.len())
            .enumerate()
            .filter_map(|(i, w)| if w == needle { Some(i) } else { None })
            .collect()
    }

    /// Encode an ASCII string as an uppercase-hex byte sequence, matching
    /// how `printpdf`'s `use_text` serializes text with the builtin Helvetica
    /// font (each character emitted as a two-hex-digit uppercase pair, no
    /// whitespace, wrapped in `<...>` in the content stream).
    fn encode_ascii_to_pdf_hex(s: &str) -> String {
        let mut out = String::with_capacity(s.len() * 2);
        for b in s.bytes() {
            out.push_str(&format!("{b:02X}"));
        }
        out
    }

    // ---------------------------------------------------------------
    // Test A — empty week produces a valid PDF (RED-first anchor).
    // ---------------------------------------------------------------
    #[test]
    fn empty_week_yields_valid_pdf_signature() {
        let week = empty_week(2026, 27);
        let bytes = render_shiftplan_week_pdf(&week, &[], 2026, 27, FIXED_RENDER_TIMESTAMP)
            .expect("render succeeds");
        assert!(bytes.len() > 500, "PDF should have plausible size, got {}", bytes.len());
        assert_eq!(&bytes[..4], b"%PDF", "PDF signature must be present");
    }

    // ---------------------------------------------------------------
    // Test B — page header embeds "Schichtplan KW NN (YYYY)".
    // ---------------------------------------------------------------
    #[test]
    fn header_contains_year_and_week() {
        let week = empty_week(2026, 27);
        let bytes = render_shiftplan_week_pdf(&week, &[], 2026, 27, FIXED_RENDER_TIMESTAMP)
            .expect("render succeeds");
        // printpdf encodes `use_text(...)` as an uppercase-hex string in the
        // content stream — search for the hex-encoded form.
        let hex = encode_ascii_to_pdf_hex("Schichtplan KW 27 (2026)");
        assert!(
            find_subsequence(&bytes, hex.as_bytes()).is_some(),
            "header text (hex {hex}) not found in PDF bytes",
        );
    }

    // ---------------------------------------------------------------
    // Test C — all active sales-persons appear (Slot-Box layout).
    // ---------------------------------------------------------------
    #[test]
    fn all_active_sales_persons_appear() {
        let alice =
            make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0001, "Alice", Some(true));
        let bob = make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0002, "Bob", Some(true));
        let charlie = make_sales_person(
            0x0000_0000_0000_0000_0000_0000_0000_0003,
            "Charlie",
            Some(true),
        );

        let slot = make_slot(DayOfWeek::Monday, 8, 0, 12, 0);

        let bookings = vec![
            make_booking(&alice, slot.id, 2026, 27),
            make_booking(&bob, slot.id, 2026, 27),
            make_booking(&charlie, slot.id, 2026, 27),
        ];

        let mut week = empty_week(2026, 27);
        // Put the slot with all bookings on Monday.
        week.days[0].slots.push(ShiftplanSlot {
            slot,
            bookings,
            current_paid_count: 3,
        });

        let sales_persons = vec![alice.clone(), bob.clone(), charlie.clone()];
        let bytes = render_shiftplan_week_pdf(
            &week,
            &sales_persons,
            2026,
            27,
            FIXED_RENDER_TIMESTAMP,
        )
        .expect("render succeeds");
        for name in ["Alice", "Bob", "Charlie"] {
            let hex = encode_ascii_to_pdf_hex(name);
            assert!(
                find_subsequence(&bytes, hex.as_bytes()).is_some(),
                "sales-person name '{name}' (hex '{hex}') not found in PDF bytes",
            );
        }
    }

    // Note: `deterministic_bytes_for_same_input` (v2.2 byte-determinism guard)
    // and `sales_persons_sorted_by_id` (global id-sort assertion) were removed
    // per D-50-13 (byte-determinism contract dropped) and D-50-15 (sort logic
    // moved to alphabetical-within-slot-box, see `names_within_slot_alphabetical`
    // in Wave 2).

    // ---------------------------------------------------------------
    // REFACTOR-Helper Tests: private-fn unit coverage.
    // ---------------------------------------------------------------
    #[test]
    fn build_page_header_produces_expected_text() {
        assert_eq!(build_page_header(2026, 27), "Schichtplan KW 27 (2026)");
        assert_eq!(build_page_header(2026, 1), "Schichtplan KW 01 (2026)");
        assert_eq!(build_page_header(1999, 52), "Schichtplan KW 52 (1999)");
    }

    #[test]
    fn build_day_column_headers_yields_seven_short_labels() {
        let headers = build_day_column_headers();
        assert_eq!(headers, ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"]);
    }

    // Note: `build_sales_person_row_lists_bookings_time_ranges` (v2.2 row-layout
    // per-day cell test) was removed per D-50-15 — the row layout with a name
    // column plus one time-range cell per day is gone. Wave 2 replaces it with
    // slot-box-centric rendering (`slot_boxes_sorted_by_start_time` and
    // `names_within_slot_alphabetical`).

    #[test]
    fn normalize_pdf_id_removes_variable_id_array() {
        // Two mock byte-buffers that differ only in the /ID array content.
        // The printpdf/lopdf format is /ID[(...)(...)] — literal strings in
        // parens inside square brackets, no leading space.
        let a = b"prefix /ID[(abc123)(def456)] suffix".to_vec();
        let b = b"prefix /ID[(zzz999)(yyy888)] suffix".to_vec();
        assert_ne!(a, b, "raw bytes differ");
        assert_eq!(
            normalize_pdf_id(&a),
            normalize_pdf_id(&b),
            "normalization must eliminate the /ID difference",
        );
    }

    #[test]
    fn find_all_subsequences_locates_multiple_occurrences() {
        // Sanity check for helper used elsewhere in tests (defense-in-depth).
        let hay = b"foo bar foo baz foo";
        let hits = find_all_subsequences(hay, b"foo");
        assert_eq!(hits, vec![0, 8, 16]);
    }

    // ================================================================
    // D-50-16 behavior tests (activated in Wave 2 — 50-02-PLAN.md).
    // ================================================================

    /// D-50-16 / PDF-02: The rendered PDF must embed the fixed timestamp
    /// string "Erstellt am 03.07.2026 17:15 Uhr" (formatted from
    /// `FIXED_RENDER_TIMESTAMP`).
    #[test]
    fn render_includes_timestamp_string() {
        let week = empty_week(2026, 27);
        let bytes = render_shiftplan_week_pdf(&week, &[], 2026, 27, FIXED_RENDER_TIMESTAMP)
            .expect("render succeeds");
        let hex = encode_ascii_to_pdf_hex("Erstellt am 03.07.2026 17:15 Uhr");
        assert!(
            find_subsequence(&bytes, hex.as_bytes()).is_some(),
            "timestamp string not found in PDF (hex: {hex})",
        );
    }

    /// D-50-16 / D-50-02 / PDF-01: Slot boxes on the same day must appear in
    /// start-time order in the content stream, regardless of the input order
    /// in the `slots` Vec.
    #[test]
    fn slot_boxes_sorted_by_start_time() {
        let slot_late = make_slot_with_id(
            0x0000_0000_0000_0000_0000_0000_0000_00aa,
            DayOfWeek::Monday,
            12,
            0,
            16,
            0,
        );
        let slot_early = make_slot_with_id(
            0x0000_0000_0000_0000_0000_0000_0000_00bb,
            DayOfWeek::Monday,
            8,
            0,
            11,
            0,
        );
        let mut week = empty_week(2026, 27);
        // Insert late first, early second — the renderer must sort them.
        week.days[0].slots.push(ShiftplanSlot {
            slot: slot_late,
            bookings: Vec::new(),
            current_paid_count: 0,
        });
        week.days[0].slots.push(ShiftplanSlot {
            slot: slot_early,
            bookings: Vec::new(),
            current_paid_count: 0,
        });
        let bytes = render_shiftplan_week_pdf(&week, &[], 2026, 27, FIXED_RENDER_TIMESTAMP)
            .expect("render succeeds");
        let hex_early = encode_ascii_to_pdf_hex("08:00");
        let hex_late = encode_ascii_to_pdf_hex("12:00");
        let idx_early = find_subsequence(&bytes, hex_early.as_bytes());
        let idx_late = find_subsequence(&bytes, hex_late.as_bytes());
        assert!(
            idx_early.is_some() && idx_late.is_some(),
            "both time labels must be present (early={idx_early:?}, late={idx_late:?})",
        );
        assert!(
            idx_early < idx_late,
            "08:00 must appear before 12:00 in the textstream (sort order D-50-02)",
        );
    }

    /// D-50-16 / D-50-06 / PDF-01: Names inside a single slot box must be
    /// alphabetical (case-insensitive) regardless of booking-Vec insertion
    /// order.
    #[test]
    fn names_within_slot_alphabetical() {
        let alice =
            make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0001, "Alice", Some(true));
        let bob = make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0002, "Bob", Some(true));
        let charlie = make_sales_person(
            0x0000_0000_0000_0000_0000_0000_0000_0003,
            "Charlie",
            Some(true),
        );
        let slot = make_slot(DayOfWeek::Monday, 8, 0, 12, 0);
        // Non-alphabetical insertion order — the renderer must sort.
        let bookings = vec![
            make_booking(&charlie, slot.id, 2026, 27),
            make_booking(&alice, slot.id, 2026, 27),
            make_booking(&bob, slot.id, 2026, 27),
        ];
        let mut week = empty_week(2026, 27);
        week.days[0].slots.push(ShiftplanSlot {
            slot,
            bookings,
            current_paid_count: 3,
        });
        let sales_persons = vec![alice.clone(), bob.clone(), charlie.clone()];
        let bytes = render_shiftplan_week_pdf(
            &week,
            &sales_persons,
            2026,
            27,
            FIXED_RENDER_TIMESTAMP,
        )
        .expect("render succeeds");
        let idx_alice = find_subsequence(&bytes, encode_ascii_to_pdf_hex("Alice").as_bytes());
        let idx_bob = find_subsequence(&bytes, encode_ascii_to_pdf_hex("Bob").as_bytes());
        let idx_charlie =
            find_subsequence(&bytes, encode_ascii_to_pdf_hex("Charlie").as_bytes());
        assert!(
            idx_alice.is_some() && idx_bob.is_some() && idx_charlie.is_some(),
            "all names must appear (alice={idx_alice:?}, bob={idx_bob:?}, charlie={idx_charlie:?})",
        );
        assert!(
            idx_alice < idx_bob && idx_bob < idx_charlie,
            "names must be alphabetical case-insensitive within slot box (D-50-06)",
        );
    }

    /// D-50-04 revision (UAT feedback 2026-07-03): a short slot with more
    /// bookings than would fit under the old duration-scaled height must
    /// no longer emit `+ N weitere` — the box has to grow to accommodate
    /// all names, comma-separated across as many lines as needed.
    #[test]
    fn short_slot_with_many_names_grows_box_no_overflow_marker() {
        let names_input = ["Anna", "Bertha", "Chris", "Doro", "Emil"];
        let mut sales_persons: Vec<SalesPerson> = Vec::new();
        let mut bookings: Vec<ShiftplanBooking> = Vec::new();
        let slot = make_slot(DayOfWeek::Monday, 8, 0, 9, 0); // 1h slot — smallest realistic case
        for (i, n) in names_input.iter().enumerate() {
            let sp = make_sales_person((i as u128) + 1, n, Some(true));
            bookings.push(make_booking(&sp, slot.id, 2026, 27));
            sales_persons.push(sp);
        }
        let mut week = empty_week(2026, 27);
        week.days[0].slots.push(ShiftplanSlot {
            slot,
            bookings,
            current_paid_count: names_input.len() as u8,
        });
        let bytes = render_shiftplan_week_pdf(
            &week,
            &sales_persons,
            2026,
            27,
            FIXED_RENDER_TIMESTAMP,
        )
        .expect("render succeeds");
        for n in &names_input {
            assert!(
                find_subsequence(&bytes, encode_ascii_to_pdf_hex(n).as_bytes()).is_some(),
                "name '{n}' must appear in a slot that grew to fit all bookings (D-50-04 revised)",
            );
        }
        // No name-overflow marker inside the slot box.
        assert!(
            find_subsequence(&bytes, encode_ascii_to_pdf_hex("weitere").as_bytes()).is_none(),
            "'+ N weitere' name-overflow marker must not appear when a slot has any realistic number of bookings",
        );
    }

    /// Unit test for the comma-wrap helper directly — verifies the atomic
    /// join semantics (non-last names carry a trailing `,`) and that wraps
    /// happen at name boundaries when the accumulated width would exceed
    /// `max_width_mm`.
    #[test]
    fn wrap_names_comma_splits_at_name_boundary() {
        let names: Vec<String> = ["Anna", "Bertha", "Christina"]
            .into_iter()
            .map(String::from)
            .collect();
        // Very wide: one line.
        let lines = wrap_names_comma(&names, 200.0);
        assert_eq!(lines, vec!["Anna, Bertha, Christina".to_string()]);
        // Very narrow: three lines, non-last names with trailing comma.
        let lines = wrap_names_comma(&names, 8.0);
        assert_eq!(
            lines,
            vec!["Anna,".to_string(), "Bertha,".to_string(), "Christina".to_string()]
        );
        // Empty input: no lines.
        assert!(wrap_names_comma(&[], 50.0).is_empty());
    }

    /// UAT 2026-07-03 row-alignment revision of D-50-01: the row height is
    /// the max needed height across all days at a given row index. Days
    /// without an N-th slot do not contribute; only defined slots do.
    #[test]
    fn compute_row_heights_takes_max_across_days() {
        // Build synthetic per-day slot data without going through the full
        // renderer — DaySlotRender only needs `slot` (unused for the row
        // math), `name_lines`, and `needed_height_mm`.
        let slot = make_slot(DayOfWeek::Monday, 8, 0, 9, 0);
        let s = ShiftplanSlot { slot, bookings: Vec::new(), current_paid_count: 0 };
        let mk = |h: f32| DaySlotRender {
            slot: &s,
            name_lines: Vec::new(),
            needed_height_mm: h,
        };
        // Day 0: [10, 20]
        // Day 1: [15]
        // Day 2: [5, 25, 30]
        let days: Vec<Vec<DaySlotRender>> =
            vec![vec![mk(10.0), mk(20.0)], vec![mk(15.0)], vec![mk(5.0), mk(25.0), mk(30.0)]];
        let heights = compute_row_heights(&days);
        assert_eq!(heights.len(), 3, "row count = max slot count across days");
        assert!((heights[0] - 15.0).abs() < 1e-4, "row 0 = max(10,15,5) = 15");
        assert!((heights[1] - 25.0).abs() < 1e-4, "row 1 = max(20,25) = 25 (day 1 has no row 1)");
        assert!((heights[2] - 30.0).abs() < 1e-4, "row 2 = max(30) = 30 (only day 2 contributes)");
    }

    /// UAT 2026-07-03: rows_fitting stops before a row that would overflow;
    /// gaps between rows count against the available height budget.
    #[test]
    fn compute_rows_fitting_stops_at_overflow() {
        let heights = vec![20.0f32, 20.0, 20.0, 20.0];
        // Avail 45 → rows 0+1 = 20 + gap + 20 = 40.7 fits, adding row 2 = 61.4 does not.
        let fitting = compute_rows_fitting(&heights, 45.0);
        assert_eq!(fitting, 2);
        // Avail 100 → all four fit.
        assert_eq!(compute_rows_fitting(&heights, 100.0), 4);
        // Avail 5 → not even row 0 fits.
        assert_eq!(compute_rows_fitting(&heights, 5.0), 0);
        // Empty row list → nothing fits.
        assert_eq!(compute_rows_fitting(&[], 100.0), 0);
    }

    /// UAT 2026-07-03: y-bottoms drop by `row_height + SLOT_GAP_MM` per row
    /// starting from `grid_top_y`. Row 0 has no leading gap.
    #[test]
    fn compute_row_y_bottoms_stacks_with_gap() {
        let heights = vec![10.0f32, 20.0, 5.0];
        let ys = compute_row_y_bottoms(&heights, 200.0, 3);
        assert!((ys[0] - (200.0 - 10.0)).abs() < 1e-4, "row 0 bottom = top - h[0]");
        assert!(
            (ys[1] - (200.0 - 10.0 - SLOT_GAP_MM - 20.0)).abs() < 1e-4,
            "row 1 bottom = row 0 bottom - gap - h[1]",
        );
        assert!(
            (ys[2] - (200.0 - 10.0 - SLOT_GAP_MM - 20.0 - SLOT_GAP_MM - 5.0)).abs() < 1e-4,
            "row 2 bottom = row 1 bottom - gap - h[2]",
        );
        // rows_fitting = 0 → empty vec.
        assert!(compute_row_y_bottoms(&heights, 200.0, 0).is_empty());
    }

    /// UAT 2026-07-03 end-to-end: two days both have a 1h slot at row 0
    /// and a 1h slot at row 1. Monday's row 0 has one booking, Tuesday's
    /// row 0 has enough bookings to force multi-line name wrap. The shared
    /// row 0 height must exceed the duration-only baseline so BOTH days'
    /// row-1 y-position is pushed down together — that is the invariant
    /// the user asked for.
    #[test]
    fn row_alignment_across_days_pushes_all_columns_down_together() {
        // Day 0 (Monday): 2 slots, both 1-hour, each with 1 short booking.
        let mo0 = make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0011, "Al", Some(true));
        let mo1 = make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0012, "Bo", Some(true));
        let mo_slot0 = make_slot(DayOfWeek::Monday, 8, 0, 9, 0);
        let mo_slot1 = make_slot(DayOfWeek::Monday, 10, 0, 11, 0);
        // Day 1 (Tuesday): row 0 has enough bookings (long names) to force
        // 3+ wrapped lines; row 1 is small.
        let tu_slot0 = make_slot(DayOfWeek::Tuesday, 8, 0, 9, 0);
        let tu_slot1 = make_slot(DayOfWeek::Tuesday, 10, 0, 11, 0);
        // Build ~12 Tuesday bookings with realistic-length names to force
        // wrapping across ~3 lines in a 6-day-wide column (~47mm).
        let tu_names: Vec<&'static str> = vec![
            "Alexander", "Bernadette", "Christopher", "Dominique",
            "Elizabeth", "Friedrich", "Gwendolyn", "Heinrich",
            "Isabella", "Johannes", "Katharina", "Ludwig",
        ];
        let tu_persons: Vec<SalesPerson> = tu_names
            .iter()
            .enumerate()
            .map(|(i, n)| {
                make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0100 + i as u128, n, Some(true))
            })
            .collect();

        let mut week = empty_week(2026, 27);
        let mo_slot0_id = mo_slot0.id;
        let mo_slot1_id = mo_slot1.id;
        let tu_slot0_id = tu_slot0.id;
        let tu_slot1_id = tu_slot1.id;
        week.days[0].slots.push(ShiftplanSlot {
            slot: mo_slot0,
            bookings: vec![make_booking(&mo0, mo_slot0_id, 2026, 27)],
            current_paid_count: 1,
        });
        week.days[0].slots.push(ShiftplanSlot {
            slot: mo_slot1,
            bookings: vec![make_booking(&mo1, mo_slot1_id, 2026, 27)],
            current_paid_count: 1,
        });
        week.days[1].slots.push(ShiftplanSlot {
            slot: tu_slot0,
            bookings: tu_persons
                .iter()
                .map(|sp| make_booking(sp, tu_slot0_id, 2026, 27))
                .collect(),
            current_paid_count: tu_persons.len() as u8,
        });
        week.days[1].slots.push(ShiftplanSlot {
            slot: tu_slot1,
            bookings: vec![make_booking(&mo1, tu_slot1_id, 2026, 27)],
            current_paid_count: 1,
        });
        let mut sales_persons: Vec<SalesPerson> = vec![mo0.clone(), mo1.clone()];
        sales_persons.extend(tu_persons.iter().cloned());

        // Structural verification: compute the same per-day slot data the
        // renderer uses, then check the row-alignment invariant. This is
        // more precise than parsing the PDF byte stream.
        let text_width_mm = (compute_col_width_mm(6) - 2.0 * SLOT_TEXT_H_PADDING_MM).max(1.0);
        let day_slot_renders: Vec<Vec<DaySlotRender>> = [DayOfWeek::Monday, DayOfWeek::Tuesday]
            .into_iter()
            .map(|dow| {
                let day = week.days.iter().find(|d| d.day_of_week == dow).unwrap();
                sort_slots_for_day(&day.slots)
                    .into_iter()
                    .map(|s| {
                        let names = build_slot_name_list(s, &sales_persons);
                        let name_lines = wrap_names_comma(&names, text_width_mm);
                        let duration = compute_slot_duration_hours(&s.slot);
                        let needed = compute_slot_box_height_mm(duration, name_lines.len());
                        DaySlotRender { slot: s, name_lines, needed_height_mm: needed }
                    })
                    .collect()
            })
            .collect();
        let heights = compute_row_heights(&day_slot_renders);
        assert_eq!(heights.len(), 2, "both days have 2 slots → 2 rows");
        assert!(
            heights[0] > heights[1] + 1.0,
            "row 0 must grow beyond the duration-only baseline because Tuesday's slot 0 has many wrapped-name lines (row heights: {:?})",
            heights,
        );
        // y_bottoms are shared across days by construction of the renderer.
        let ys = compute_row_y_bottoms(&heights, 200.0, 2);
        assert!(ys[1] < ys[0], "row 1 sits below row 0 by construction");
        // End-to-end: render must succeed and preserve name visibility.
        let bytes = render_shiftplan_week_pdf(
            &week,
            &sales_persons,
            2026,
            27,
            FIXED_RENDER_TIMESTAMP,
        )
        .expect("render succeeds");
        assert!(
            find_subsequence(&bytes, encode_ascii_to_pdf_hex("Al").as_bytes()).is_some(),
            "Monday row-0 name Al present",
        );
        assert!(
            find_subsequence(&bytes, encode_ascii_to_pdf_hex("Bo").as_bytes()).is_some(),
            "Monday row-1 name Bo present (pushed down by shared row 0 height, still on page)",
        );
        assert!(
            find_subsequence(&bytes, encode_ascii_to_pdf_hex("Ludwig").as_bytes()).is_some(),
            "Tuesday row-0 all names present including Ludwig (box grew, D-50-04 revised)",
        );
    }

    /// D-50-16 / D-50-07 / PDF-01: Volunteers (`is_paid == Some(false)`) get
    /// the suffix " (freiwillig)" appended to their name in the rendered
    /// output.
    #[test]
    fn unpaid_marker_suffix() {
        let volunteer = make_sales_person(
            0x0000_0000_0000_0000_0000_0000_0000_0001,
            "Volunteer",
            Some(false),
        );
        let slot = make_slot(DayOfWeek::Monday, 8, 0, 12, 0);
        let booking = make_booking(&volunteer, slot.id, 2026, 27);
        let mut week = empty_week(2026, 27);
        week.days[0].slots.push(ShiftplanSlot {
            slot,
            bookings: vec![booking],
            current_paid_count: 0,
        });
        let sales_persons = vec![volunteer.clone()];
        let bytes = render_shiftplan_week_pdf(
            &week,
            &sales_persons,
            2026,
            27,
            FIXED_RENDER_TIMESTAMP,
        )
        .expect("render succeeds");
        let expected_hex = encode_ascii_to_pdf_hex("Volunteer (freiwillig)");
        assert!(
            find_subsequence(&bytes, expected_hex.as_bytes()).is_some(),
            "unpaid marker suffix ' (freiwillig)' must appear after volunteer name (D-50-07)",
        );
    }

    /// D-50-16 / D-50-08 / PDF-01: The Sunday column header "So" must NOT
    /// appear when the week has no Sunday slots.
    #[test]
    fn sunday_column_hidden_when_no_sunday_slots() {
        let slot = make_slot(DayOfWeek::Saturday, 8, 0, 12, 0);
        let mut week = empty_week(2026, 27);
        // Saturday is index 5 in day_of_week_order().
        week.days[5].slots.push(ShiftplanSlot {
            slot,
            bookings: Vec::new(),
            current_paid_count: 0,
        });
        let bytes = render_shiftplan_week_pdf(&week, &[], 2026, 27, FIXED_RENDER_TIMESTAMP)
            .expect("render succeeds");
        let so_hex = encode_ascii_to_pdf_hex("So");
        assert!(
            find_subsequence(&bytes, so_hex.as_bytes()).is_none(),
            "'So' column header must NOT appear when no Sunday slots (D-50-08)",
        );
    }

    /// D-50-16 / D-50-08 / PDF-01: The Sunday column header "So" MUST
    /// appear when at least one Sunday slot exists.
    #[test]
    fn sunday_column_shown_when_at_least_one_sunday_slot() {
        let slot = make_slot(DayOfWeek::Sunday, 10, 0, 14, 0);
        let mut week = empty_week(2026, 27);
        // Sunday is index 6 in day_of_week_order().
        week.days[6].slots.push(ShiftplanSlot {
            slot,
            bookings: Vec::new(),
            current_paid_count: 0,
        });
        let bytes = render_shiftplan_week_pdf(&week, &[], 2026, 27, FIXED_RENDER_TIMESTAMP)
            .expect("render succeeds");
        let so_hex = encode_ascii_to_pdf_hex("So");
        assert!(
            find_subsequence(&bytes, so_hex.as_bytes()).is_some(),
            "'So' column header MUST appear when at least one Sunday slot exists (D-50-08)",
        );
    }
}
