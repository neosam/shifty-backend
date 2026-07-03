//! Deterministic PDF renderer for weekly shift plans (Phase 48 EXP-01).
//!
//! Pure module: takes domain structs, returns bytes. No I/O, no DAO,
//! no HTTP. `printpdf` is the only external dep (pure Rust, D-48-PDF).
//!
//! # Contract
//!
//! - **Caller must pre-filter `sales_persons` to active (non-deleted) rows**
//!   (D-48-PDF-ACTIVE-ONLY). The renderer does NOT filter by `deleted`.
//! - Sales-persons are internally sorted by `id` before rendering, so caller
//!   does not need to pre-sort (D-48-PDF-DETERMINISM).
//! - Metadata (creation_date, modification_date, producer, document title) is
//!   set to fixed values so that repeated runs with the same input diff only
//!   in the printpdf-generated random `/ID` trailer array (see
//!   [`Determinism`] section below).
//!
//! # Determinism
//!
//! `printpdf` 0.7 unconditionally generates a random 32-char `document_id`
//! (at `PdfDocument::new`) and a random 32-char `instance_id` (at
//! `save_to_bytes`) and writes them to the PDF trailer `/ID [ .. .. ]`
//! array. There is no public API to override these. This is a
//! documented printpdf limitation.
//!
//! Consequence: two calls to [`render_shiftplan_week_pdf`] with the same
//! input produce byte-sequences that agree everywhere **except** in the two
//! 32-char slots of the trailer `/ID` array. All other bytes — including the
//! `/CreationDate`, `/ModDate`, `/Producer` fields, the page tree, the
//! content stream, and the cross-reference table — are byte-identical.
//!
//! Downstream (Nextcloud upload in Plan 48-04) treats the PDFs as
//! whole-file uploads keyed by filename; the `/ID` array does not affect
//! filename or file semantics. For the deterministic-bytes test, we
//! normalize the two PDFs by stripping the `/ID` array before comparison
//! (see [`test::normalize_pdf_id`]).

use service::sales_person::SalesPerson;
use service::shiftplan::{ShiftplanBooking, ShiftplanWeek};
use service::ServiceError;
use shifty_utils::DayOfWeek;

use printpdf::{BuiltinFont, Mm, PdfDocument};

/// Fixed metadata timestamp — 2000-01-01T00:00:00Z. Used for
/// `creation_date`, `modification_date`, `metadata_date` so that repeated
/// renders emit identical metadata bytes.
const FIXED_METADATA_TIMESTAMP: time::OffsetDateTime = time::macros::datetime!(2000-01-01 0:00 UTC);

/// Fixed producer/creator string embedded in the PDF metadata.
const PDF_PRODUCER: &str = "shifty-pdf-export";

/// Header font size in points.
const HEADER_FONT_SIZE: f32 = 16.0;
/// Day-column header font size in points.
const DAY_HEADER_FONT_SIZE: f32 = 12.0;
/// Sales-person row font size in points.
const ROW_FONT_SIZE: f32 = 10.0;

/// Landscape A4 width in mm.
const PAGE_WIDTH_MM: f32 = 297.0;
/// Landscape A4 height in mm.
const PAGE_HEIGHT_MM: f32 = 210.0;

/// X position of the first day column (mm from left).
const FIRST_DAY_COL_X_MM: f32 = 40.0;
/// Width per day column (mm).
const DAY_COL_WIDTH_MM: f32 = 36.0;
/// Y position of the page header (mm from bottom).
const HEADER_Y_MM: f32 = 195.0;
/// Y position of the day-of-week header row (mm from bottom).
const DAY_HEADER_Y_MM: f32 = 180.0;
/// Y position of the first sales-person row (mm from bottom).
const FIRST_ROW_Y_MM: f32 = 170.0;
/// Vertical distance between sales-person rows (mm).
const ROW_STEP_MM: f32 = 8.0;
/// X position where sales-person names are written (mm from left).
const NAME_X_MM: f32 = 15.0;

/// Render a single weekly shift plan into a deterministic PDF.
///
/// # Arguments
///
/// - `week`: The shiftplan data for the week to render.
/// - `sales_persons`: The active sales-persons that should appear as rows.
///   **Caller MUST pre-filter to `deleted.is_none()`** (D-48-PDF-ACTIVE-ONLY);
///   the renderer does not skip deleted entries.
/// - `header_year`: Year value to embed in the page header (e.g. `2026`).
/// - `header_week`: ISO calendar week to embed in the page header (e.g. `27`).
///
/// # Determinism
///
/// See module docs. Two calls with identical inputs produce PDFs that
/// differ only in the printpdf-generated trailer `/ID` array.
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
) -> Result<Vec<u8>, ServiceError> {
    // Deterministic sort of sales-persons (D-48-PDF-DETERMINISM).
    let mut sp_sorted: Vec<&SalesPerson> = sales_persons.iter().collect();
    sp_sorted.sort_by_key(|s| s.id);

    let title = format!("Schichtplan KW {header_week:02} ({header_year})");

    let (doc, page_index, layer_index) =
        PdfDocument::new(&title, Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), "Layer 1");

    // Metadata: fixed timestamps + fixed producer (Determinism-guardrail).
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

    // Page header.
    let header_text = build_page_header(header_year, header_week);
    layer.use_text(
        header_text,
        HEADER_FONT_SIZE,
        Mm(NAME_X_MM),
        Mm(HEADER_Y_MM),
        &font_bold,
    );

    // Day-of-week column headers (Mo–So).
    for (i, dow_label) in build_day_column_headers().into_iter().enumerate() {
        let x = FIRST_DAY_COL_X_MM + (i as f32) * DAY_COL_WIDTH_MM;
        layer.use_text(
            dow_label,
            DAY_HEADER_FONT_SIZE,
            Mm(x),
            Mm(DAY_HEADER_Y_MM),
            &font_bold,
        );
    }

    // Sales-person rows.
    for (row_index, sp) in sp_sorted.iter().enumerate() {
        let y = FIRST_ROW_Y_MM - (row_index as f32) * ROW_STEP_MM;
        // Name cell.
        layer.use_text(sp.name.to_string(), ROW_FONT_SIZE, Mm(NAME_X_MM), Mm(y), &font);
        // Day cells.
        for (day_index, dow) in day_of_week_order().into_iter().enumerate() {
            let x = FIRST_DAY_COL_X_MM + (day_index as f32) * DAY_COL_WIDTH_MM;
            let cell_text = build_sales_person_day_cell(week, sp, dow);
            if !cell_text.is_empty() {
                layer.use_text(cell_text, ROW_FONT_SIZE, Mm(x), Mm(y), &font);
            }
        }
    }

    doc.save_to_bytes().map_err(|_| ServiceError::InternalError)
}

/// Build the page-header text for a given (year, week).
fn build_page_header(year: u32, week: u8) -> String {
    format!("Schichtplan KW {week:02} ({year})")
}

/// Ordered list of day-of-week short labels for the 7 columns (Mo–So).
fn build_day_column_headers() -> [&'static str; 7] {
    ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"]
}

/// Ordered list of [`DayOfWeek`] enum values for the 7 columns (Mo–So).
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

/// Build the cell text for one sales-person × one day.
///
/// Iterates over the day's slots and collects `"HH:MM-HH:MM"` for every
/// booking whose `sales_person.id == sp.id`. Multiple bookings are joined
/// with `", "`. Empty string if the sales-person has no booking that day.
fn build_sales_person_day_cell(
    week: &ShiftplanWeek,
    sp: &SalesPerson,
    dow: DayOfWeek,
) -> String {
    let Some(day) = week.days.iter().find(|d| d.day_of_week == dow) else {
        return String::new();
    };
    let mut entries: Vec<String> = Vec::new();
    for slot in &day.slots {
        for booking in &slot.bookings {
            if booking.sales_person.id == sp.id {
                entries.push(format_booking_time_range(booking, &slot.slot));
            }
        }
    }
    entries.join(", ")
}

/// Format a single booking as `"HH:MM-HH:MM"` using the slot's from/to.
fn format_booking_time_range(
    _booking: &ShiftplanBooking,
    slot: &service::slot::Slot,
) -> String {
    format!(
        "{:02}:{:02}-{:02}:{:02}",
        slot.from.hour(),
        slot.from.minute(),
        slot.to.hour(),
        slot.to.minute(),
    )
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
    /// Consumed as the 5th parameter of `render_shiftplan_week_pdf` after the
    /// Wave-2 rewrite (D-50-11). Wave-1 tests reference it via
    /// `let _ = FIXED_RENDER_TIMESTAMP;` to keep the constant alive against the
    /// `dead_code` lint until Wave 2 actually consumes it.
    #[allow(dead_code)]
    const FIXED_RENDER_TIMESTAMP: time::OffsetDateTime =
        time::macros::datetime!(2026-07-03 17:15 UTC);

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

    // ---------------------------------------------------------------
    // Test A — empty week produces a valid PDF (RED-first anchor).
    // ---------------------------------------------------------------
    #[test]
    fn empty_week_yields_valid_pdf_signature() {
        let week = empty_week(2026, 27);
        let bytes = render_shiftplan_week_pdf(&week, &[], 2026, 27).expect("render succeeds");
        assert!(bytes.len() > 500, "PDF should have plausible size, got {}", bytes.len());
        assert_eq!(&bytes[..4], b"%PDF", "PDF signature must be present");
    }

    // ---------------------------------------------------------------
    // Test B — page header embeds "Schichtplan KW NN (YYYY)".
    // ---------------------------------------------------------------
    #[test]
    fn header_contains_year_and_week() {
        let week = empty_week(2026, 27);
        let bytes = render_shiftplan_week_pdf(&week, &[], 2026, 27).expect("render succeeds");
        let needle = b"Schichtplan KW 27 (2026)";
        assert!(
            find_subsequence(&bytes, needle).is_some(),
            "header text '{}' not found in PDF bytes",
            String::from_utf8_lossy(needle),
        );
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
    // Test C — all active sales-persons appear as row labels.
    //
    // `printpdf` encodes `use_text(...)` output as an uppercase-hex string
    // in the content stream (e.g. "Alice" → `<416C696365>`), NOT as raw
    // ASCII. We search for the hex-encoded form.
    // ---------------------------------------------------------------
    #[test]
    fn all_active_sales_persons_appear() {
        let alice =
            make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0001, "Alice", Some(true));
        let bob =
            make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0002, "Bob", Some(true));
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
        let bytes =
            render_shiftplan_week_pdf(&week, &sales_persons, 2026, 27).expect("render succeeds");
        for name in ["Alice", "Bob", "Charlie"] {
            let hex = encode_ascii_to_pdf_hex(name);
            assert!(
                find_subsequence(&bytes, hex.as_bytes()).is_some(),
                "sales-person name '{name}' (hex '{hex}') not found in PDF bytes",
            );
        }
    }

    // ---------------------------------------------------------------
    // Test D — repeated renders with same input are byte-identical
    // (modulo the printpdf-generated /ID trailer array, see module docs).
    // ---------------------------------------------------------------
    #[test]
    fn deterministic_bytes_for_same_input() {
        let week = empty_week(2026, 27);
        let a = render_shiftplan_week_pdf(&week, &[], 2026, 27).expect("first render succeeds");
        let b = render_shiftplan_week_pdf(&week, &[], 2026, 27).expect("second render succeeds");

        let a_norm = normalize_pdf_id(&a);
        let b_norm = normalize_pdf_id(&b);
        assert_eq!(
            a_norm, b_norm,
            "same input must produce byte-identical PDF except for /ID array (len_a={}, len_b={}, len_a_norm={}, len_b_norm={})",
            a.len(), b.len(), a_norm.len(), b_norm.len(),
        );

        // Also assert that the /CreationDate is fixed and identical in both.
        let creation_marker = b"/CreationDate";
        let ca = find_subsequence(&a, creation_marker)
            .expect("/CreationDate must be present in first PDF");
        let cb = find_subsequence(&b, creation_marker)
            .expect("/CreationDate must be present in second PDF");
        // Extract 40 bytes after the marker and compare.
        let a_slice = &a[ca..ca + creation_marker.len() + 40];
        let b_slice = &b[cb..cb + creation_marker.len() + 40];
        assert_eq!(a_slice, b_slice, "/CreationDate bytes must match across renders");

        // Producer must also be the fixed constant.
        let producer_marker = b"/Producer";
        assert!(
            find_subsequence(&a, producer_marker).is_some(),
            "/Producer must be present"
        );
        assert!(
            find_subsequence(&a, PDF_PRODUCER.as_bytes()).is_some(),
            "producer constant '{}' must be embedded",
            PDF_PRODUCER,
        );
    }

    // ---------------------------------------------------------------
    // Test E — internal sort by sales_person.id: pre-sorted vs shuffled
    // input produces the same normalized bytes.
    // ---------------------------------------------------------------
    #[test]
    fn sales_persons_sorted_by_id() {
        let alice =
            make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0001, "Alice", Some(true));
        let bob =
            make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0002, "Bob", Some(true));
        let charlie = make_sales_person(
            0x0000_0000_0000_0000_0000_0000_0000_0003,
            "Charlie",
            Some(true),
        );

        let week = empty_week(2026, 27);

        let sorted = vec![alice.clone(), bob.clone(), charlie.clone()];
        let shuffled = vec![charlie.clone(), alice.clone(), bob.clone()];

        let bytes_sorted =
            render_shiftplan_week_pdf(&week, &sorted, 2026, 27).expect("render sorted succeeds");
        let bytes_shuffled =
            render_shiftplan_week_pdf(&week, &shuffled, 2026, 27).expect("render shuffled succeeds");

        assert_eq!(
            normalize_pdf_id(&bytes_sorted),
            normalize_pdf_id(&bytes_shuffled),
            "internal id-sort must make input order irrelevant",
        );
    }

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

    #[test]
    fn build_sales_person_row_lists_bookings_time_ranges() {
        let sp =
            make_sales_person(0x0000_0000_0000_0000_0000_0000_0000_0007, "Test", Some(true));
        let slot = make_slot(DayOfWeek::Wednesday, 9, 30, 14, 45);
        let booking = make_booking(&sp, slot.id, 2026, 27);
        let mut week = empty_week(2026, 27);
        // Wednesday is index 2 in day_of_week_order().
        week.days[2].slots.push(ShiftplanSlot {
            slot,
            bookings: vec![booking],
            current_paid_count: 1,
        });

        let cell = build_sales_person_day_cell(&week, &sp, DayOfWeek::Wednesday);
        assert_eq!(cell, "09:30-14:45");

        // Days with no booking yield empty string.
        assert!(build_sales_person_day_cell(&week, &sp, DayOfWeek::Monday).is_empty());
    }

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
}
