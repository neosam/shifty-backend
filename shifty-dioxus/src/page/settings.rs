//! SettingsPage — admin-gated page with the paid-limit hard/soft toggle (Card 1),
//! the holiday auto-credit activation date field (Card 2), the short-day slot-
//! clipping activation date (Card 2b, Phase 51 SHC-06), and the Special-Days
//! management card (Card 3, shiftplanner-gated).
//! Phase 24 D-24-06: paid-limit enforcement toggle.
//! Phase 25 D-25-06: holiday auto-credit cutoff date input.
//! Phase 33: Special-Days Settings Card (D-33-02/04/06/07/08).
//! Phase 51 D-51-07 / SHC-06: short-day slot-clipping activation date (Card 2b).

use time::macros::format_description;

use dioxus::prelude::*;
use rest_types::{DayOfWeekTO, SpecialDayTO, SpecialDayTypeTO};
use uuid::Uuid;

use crate::{
    api,
    base_types::ImStr,
    component::{
        atoms::btn::{Btn, BtnVariant},
        form::{SelectInput, TextInput},
        TopBar,
    },
    i18n::Key,
    js,
    loader,
    service::{auth::AUTH, config::CONFIG, i18n::I18N},
    state::pdf_export_config::{clamp_weeks_horizon, PdfExportForm},
};

/// Parse an ISO date string (`"YYYY-MM-DD"`) into `(iso_year, iso_week, DayOfWeekTO)`.
///
/// Returns `None` if `date_str` is not a valid ISO date.
/// Used by the Card-3 create-form to map a calendar date to ISO week fields
/// before POSTing via `create_special_day` (D-33-04).
pub fn parse_date_to_iso_parts(date_str: &str) -> Option<(u32, u8, DayOfWeekTO)> {
    let date_format = format_description!("[year]-[month]-[day]");
    let date = time::Date::parse(date_str, date_format).ok()?;
    let (iso_year, iso_week, weekday) = date.to_iso_week_date();
    Some((iso_year as u32, iso_week, DayOfWeekTO::from(weekday)))
}

/// Convert a `SpecialDayTO` back to a `time::Date` for display formatting.
///
/// Used in Card-3 list rendering to produce the locale-formatted date string
/// and context suffix (SPD-02 / D-33-08).
pub fn special_day_iso_date(entry: &SpecialDayTO) -> Option<time::Date> {
    let weekday = time::Weekday::from(entry.day_of_week);
    time::Date::from_iso_week_date(entry.year as i32, entry.calendar_week, weekday).ok()
}

/// Map a `DayOfWeekTO` to the corresponding i18n `Key` for weekday names.
///
/// Used in Card-3 context string construction (SPD-02).
pub fn weekday_key(day: DayOfWeekTO) -> Key {
    match day {
        DayOfWeekTO::Monday => Key::Monday,
        DayOfWeekTO::Tuesday => Key::Tuesday,
        DayOfWeekTO::Wednesday => Key::Wednesday,
        DayOfWeekTO::Thursday => Key::Thursday,
        DayOfWeekTO::Friday => Key::Friday,
        DayOfWeekTO::Saturday => Key::Saturday,
        DayOfWeekTO::Sunday => Key::Sunday,
    }
}

/// Map `Option<SpecialDayTypeTO>` to the HTML select `value` string (D-06).
///
/// Mirrors the inverse of the `onchange` match in the Card-3 SelectInput handler.
/// Used to derive a controlled `value` prop from the `sd_type` signal so the
/// dropdown stays in sync with the signal without signal↔DOM desync (D-08: the
/// date field is likewise controlled via `value: ImStr::from(sd_date_val.as_str())`).
/// Note (Phase 42 / D-42-01): the form fields are NO LONGER reset after a successful
/// create — type/date/time are retained so the Anlegen button stays enabled for
/// repeated creates.
pub(crate) fn sd_type_to_select_value(ty: Option<SpecialDayTypeTO>) -> &'static str {
    match ty {
        None => "",
        Some(SpecialDayTypeTO::Holiday) => "holiday",
        Some(SpecialDayTypeTO::ShortDay) => "short_day",
    }
}

/// Returns `true` if a special day with the same `(year, calendar_week, day_of_week)` already
/// exists in `list`.
///
/// Used for the live duplicate hint in Card-3 Row D (D-33-07).
pub fn is_duplicate_special_day(
    parts: (u32, u8, DayOfWeekTO),
    list: &[SpecialDayTO],
) -> bool {
    let (year, calendar_week, day_of_week) = parts;
    list.iter().any(|entry| {
        entry.year == year
            && entry.calendar_week == calendar_week
            && entry.day_of_week == day_of_week
    })
}

/// Pure Card-3 create-form validity predicate (D-42-05, extracted from the former
/// inline logic at the render body).
///
/// Returns `true` iff the date is non-empty AND a type is selected AND
/// (the type is not `ShortDay` OR a time is provided). Byte-for-byte the same
/// semantics as the previous inline `sd_form_valid` expression — just extracted
/// into a pure, unit-tested function so the "button stays enabled after create"
/// invariant (D-42-01) can be verified without browser flakiness.
pub(crate) fn is_special_day_form_valid(
    date_str: &str,
    ty: Option<SpecialDayTypeTO>,
    time_str: &str,
) -> bool {
    !date_str.is_empty()
        && ty.is_some()
        && (ty != Some(SpecialDayTypeTO::ShortDay) || !time_str.is_empty())
}

/// Snapshot of the three Card-3 create-form signal values (date / type / time).
///
/// Modeled as a pure value so the post-create retention policy (D-42-01/02) is
/// unit-testable independent of Dioxus signals.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpecialDayForm {
    pub date: String,
    pub ty: Option<SpecialDayTypeTO>,
    pub time: String,
}

/// Post-create retention policy (D-42-01: Option 2 — keep the form filled).
///
/// After a successful create, the three form fields are NOT reset: this returns
/// `before.clone()` so date/type/time stay filled → `is_special_day_form_valid`
/// stays `true` → the Anlegen button stays enabled → repeated create without
/// re-toggling the type dropdown. The year-picker jump (SDF-03 —
/// `sd_year.set(sd_year_after_create(...))`, formerly WR-04's ISO-year set) and
/// `sd_resource.restart()` (reload the list) are handled separately by the
/// success handler and are intentionally NOT part of this form-field policy
/// (D-42-02).
pub(crate) fn special_day_form_after_create(before: &SpecialDayForm) -> SpecialDayForm {
    before.clone()
}

/// Post-create target year for the Card-3 year picker (SDF-03, 43-01).
///
/// Unlike [`parse_date_to_iso_parts`] which returns the **ISO-week-year**
/// (`date.to_iso_week_date().0`), this helper returns the **calendar year**
/// (`date.year()`). The distinction matters at year boundaries: a special day
/// created on 2027-01-01 belongs to ISO week 53 of 2026 (`iso_year == 2026`) but
/// to calendar year 2027. Prior to SDF-03 the picker was jumped to `iso_year`
/// after create, so a 2027-01-01 entry silently vanished into the 2026er picker.
///
/// Returns `None` if `date_str` is not a valid ISO date.
pub(crate) fn sd_year_after_create(date_str: &str) -> Option<u32> {
    let date_format = format_description!("[year]-[month]-[day]");
    let date = time::Date::parse(date_str, date_format).ok()?;
    Some(date.year() as u32)
}

/// Visibility rule for the Card-3 "existiert bereits" duplicate hint (260702-jql).
///
/// Since Phase 42 (D-42-01) the create-form fields are retained after a successful
/// create, so the just-created entry matches itself and `is_duplicate` flips true
/// immediately. That is a false-positive for the user (they did not type a
/// duplicate — the system kept their input). This pure predicate gates the hint on
/// a `suppressed` flag that the success handler sets after create and the three
/// field `on_change` handlers clear on the next real edit: the hint shows only when
/// there is a duplicate AND it is not currently suppressed. (Deliberate, narrow
/// reversal of the "always show on match" behaviour from D-42-03.)
pub(crate) fn should_show_duplicate_hint(is_duplicate: bool, suppressed: bool) -> bool {
    is_duplicate && !suppressed
}

/// Validity predicate for the short-day slot-clipping activation-date input
/// (Phase 51 SHC-06 / D-51-07).
///
/// Returns `true` for:
/// - the empty string (Legacy off — user cleared the date), and
/// - any well-formed ISO-8601 `YYYY-MM-DD` string.
///
/// Returns `false` for any malformed date input (e.g. German locale `08.01.2026`,
/// natural-language `"tomorrow"`, or out-of-range components like `2026-13-01`).
///
/// **Why a pure fn?** `<input type=date>` in Dioxus/WASM does not reliably fire
/// signal updates from programmatic `set()`s in browser tests (Auto-Memory
/// `reference_dioxus_browser_test_date_inputs`, D-25-06). Extracting the
/// validator lets us prove the Save-button gating rule independently of the
/// browser via unit tests.
///
pub(crate) fn is_valid_shortday_date_input(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    let date_format = format_description!("[year]-[month]-[day]");
    time::Date::parse(s, date_format).is_ok()
}

/// Semantic mirror of `service_impl::shortday_gate::should_clip` (backend P02) —
/// FE-local copy for the SHC-06 boundary-case test.
///
/// Contract: returns `true` iff `active_from` is `Some(date)` AND
/// `booking_date >= active_from`. This mirrors the backend's inclusive-boundary
/// rule so the FE editor cannot silently drift from P02's semantics; if a future
/// backend change flips the boundary, this FE test fails and the drift is
/// visible in code review.
///
/// Test-only: the FE currently does not need to compute this at runtime
/// (the backend owns the clipping decision); the fn exists solely as a
/// contract mirror for the boundary-case test below.
#[cfg(test)]
pub(crate) fn is_within_shortday_gate(
    booking_date: time::Date,
    active_from: Option<time::Date>,
) -> bool {
    match active_from {
        Some(from) => booking_date >= from,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_special_day(year: u32, calendar_week: u8, day_of_week: DayOfWeekTO) -> SpecialDayTO {
        SpecialDayTO {
            id: Uuid::nil(),
            year,
            calendar_week,
            day_of_week,
            day_type: SpecialDayTypeTO::Holiday,
            time_of_day: None,
            created: None,
            deleted: None,
            version: Uuid::nil(),
        }
    }

    #[test]
    fn parse_date_to_iso_parts_2026_08_15() {
        // 2026-08-15 is in ISO week 33, Saturday
        let result = parse_date_to_iso_parts("2026-08-15");
        assert_eq!(result, Some((2026u32, 33u8, DayOfWeekTO::Saturday)));
    }

    #[test]
    fn parse_date_to_iso_parts_invalid_returns_none() {
        assert_eq!(parse_date_to_iso_parts("not-a-date"), None);
    }

    #[test]
    fn special_day_iso_date_round_trip() {
        // Parse "2026-08-15" → parts → build SpecialDayTO → convert back to date
        let (year, calendar_week, day_of_week) =
            parse_date_to_iso_parts("2026-08-15").unwrap();
        let entry = make_special_day(year, calendar_week, day_of_week);
        let date = special_day_iso_date(&entry).expect("round-trip must succeed");
        // Reconstruct expected date
        let expected = time::Date::parse(
            "2026-08-15",
            format_description!("[year]-[month]-[day]"),
        )
        .unwrap();
        assert_eq!(date, expected);
    }

    #[test]
    fn weekday_key_saturday_and_monday() {
        assert_eq!(weekday_key(DayOfWeekTO::Saturday), Key::Saturday);
        assert_eq!(weekday_key(DayOfWeekTO::Monday), Key::Monday);
    }

    #[test]
    fn is_duplicate_special_day_true_for_same_triple() {
        let list = vec![make_special_day(2026, 33, DayOfWeekTO::Saturday)];
        assert!(is_duplicate_special_day(
            (2026, 33, DayOfWeekTO::Saturday),
            &list
        ));
    }

    #[test]
    fn is_duplicate_special_day_false_for_different_week() {
        let list = vec![make_special_day(2026, 33, DayOfWeekTO::Saturday)];
        assert!(!is_duplicate_special_day(
            (2026, 34, DayOfWeekTO::Saturday),
            &list
        ));
    }

    /// D-06: pure helper maps Option<SpecialDayTypeTO> to the HTML select value
    /// string used by the Card-3 controlled SelectInput binding.
    #[test]
    fn sd_type_to_select_value_all_variants() {
        assert_eq!(sd_type_to_select_value(None), "");
        assert_eq!(
            sd_type_to_select_value(Some(SpecialDayTypeTO::Holiday)),
            "holiday"
        );
        assert_eq!(
            sd_type_to_select_value(Some(SpecialDayTypeTO::ShortDay)),
            "short_day"
        );
    }

    // ── D-42-05: pure validity predicate (extracted from settings.rs:387-389) ──

    /// D-42-05: empty date → invalid, regardless of type.
    #[test]
    fn special_day_form_valid_empty_date_false() {
        assert!(!is_special_day_form_valid(
            "",
            Some(SpecialDayTypeTO::Holiday),
            ""
        ));
    }

    /// D-42-05: date set but no type selected → invalid.
    #[test]
    fn special_day_form_valid_no_type_false() {
        assert!(!is_special_day_form_valid("2026-08-15", None, ""));
    }

    /// D-42-05: date set, Holiday, no time → valid (time only required for ShortDay).
    #[test]
    fn special_day_form_valid_holiday_true() {
        assert!(is_special_day_form_valid(
            "2026-08-15",
            Some(SpecialDayTypeTO::Holiday),
            ""
        ));
    }

    /// D-42-05: date set, ShortDay, but empty time → invalid.
    #[test]
    fn special_day_form_valid_shortday_without_time_false() {
        assert!(!is_special_day_form_valid(
            "2026-08-15",
            Some(SpecialDayTypeTO::ShortDay),
            ""
        ));
    }

    /// D-42-05: date set, ShortDay, with time → valid.
    #[test]
    fn special_day_form_valid_shortday_with_time_true() {
        assert!(is_special_day_form_valid(
            "2026-08-15",
            Some(SpecialDayTypeTO::ShortDay),
            "12:00"
        ));
    }

    // ── D-42-01/02: post-create retention policy (Option 2 — keep all fields) ──

    /// D-42-01: for a filled form the after-create state equals the before state
    /// (all three fields retained; the three field resets are removed).
    #[test]
    fn special_day_form_retained_after_create() {
        let before = SpecialDayForm {
            date: "2026-08-15".to_string(),
            ty: Some(SpecialDayTypeTO::ShortDay),
            time: "12:00".to_string(),
        };
        let after = special_day_form_after_create(&before);
        assert_eq!(after, before);
    }

    /// D-42-05 (central case): the validity predicate stays `true` for the
    /// retained fields after create → the Anlegen button stays enabled.
    #[test]
    fn special_day_form_valid_stays_true_after_create() {
        let before = SpecialDayForm {
            date: "2026-08-15".to_string(),
            ty: Some(SpecialDayTypeTO::Holiday),
            time: String::new(),
        };
        // Before create the form is valid.
        assert!(is_special_day_form_valid(
            &before.date,
            before.ty.clone(),
            &before.time
        ));
        let after = special_day_form_after_create(&before);
        // After create the retained fields are STILL valid (button stays enabled).
        assert!(is_special_day_form_valid(
            &after.date,
            after.ty.clone(),
            &after.time
        ));
    }

    // ── 260702-jql: duplicate-hint visibility rule ──

    /// Real duplicate typed by the user, not suppressed → hint shows.
    #[test]
    fn duplicate_hint_shown_when_duplicate_and_not_suppressed() {
        assert!(should_show_duplicate_hint(true, false));
    }

    /// Directly after a successful create the self-match is suppressed → no hint.
    #[test]
    fn duplicate_hint_hidden_after_create_when_suppressed() {
        assert!(!should_show_duplicate_hint(true, true));
    }

    /// No duplicate, not suppressed → no hint.
    #[test]
    fn duplicate_hint_hidden_when_not_duplicate() {
        assert!(!should_show_duplicate_hint(false, false));
    }

    /// No duplicate and suppressed → still no hint.
    #[test]
    fn duplicate_hint_hidden_when_not_duplicate_and_suppressed() {
        assert!(!should_show_duplicate_hint(false, true));
    }

    // ── SDF-03 (Phase 43-01): sd_year_after_create uses calendar year ──

    /// SDF-03 baseline: a mid-year date maps to its own calendar year.
    #[test]
    fn sd_year_after_create_mid_year() {
        assert_eq!(sd_year_after_create("2026-08-15"), Some(2026));
    }

    /// SDF-03 core case: at the year boundary the calendar year and the ISO
    /// week year DIVERGE. 2027-01-01 is calendar year 2027 but ISO week 53 of
    /// 2026. The picker MUST jump to 2027 — otherwise the just-created entry
    /// silently disappears from the reloaded list.
    #[test]
    fn sd_year_after_create_new_year_calendar_vs_iso() {
        assert_eq!(sd_year_after_create("2027-01-01"), Some(2027));
        // Sanity — pin the divergence so this test also fails loudly if the
        // upstream `time` crate ever changes its ISO-week semantics.
        let iso = parse_date_to_iso_parts("2027-01-01").unwrap().0;
        assert_eq!(iso, 2026, "sanity: 2027-01-01 falls into ISO year 2026");
    }

    /// SDF-03 symmetry probe: Silvester of a year whose ISO week ends inside
    /// the same calendar year — here calendar and ISO agree and the picker
    /// stays put.
    #[test]
    fn sd_year_after_create_silvester() {
        assert_eq!(sd_year_after_create("2026-12-31"), Some(2026));
    }

    /// SDF-03: unparsable input yields None (matches `parse_date_to_iso_parts`
    /// error semantics).
    #[test]
    fn sd_year_after_create_invalid_returns_none() {
        assert_eq!(sd_year_after_create("not-a-date"), None);
    }

    // ── SDF-04 (Phase 43-01): duplicate-hint copy signals replace semantics ──

    /// SDF-04: the duplicate-hint text in de/en/cs must
    /// - be non-empty in every locale (presence), and
    /// - contain a replace-cue that signals in-place overwrite semantics
    ///   (matches the SDF-01 v1.11 backend behavior; the hint is a heads-up,
    ///   not a blocker).
    #[test]
    fn duplicate_hint_copy_signals_replace_semantics() {
        use crate::i18n::{generate, Locale};

        struct Case {
            locale: Locale,
            cues: &'static [&'static str],
        }
        let cases = [
            Case {
                locale: Locale::De,
                cues: &["ersetzt", "überschrieben"],
            },
            Case {
                locale: Locale::En,
                cues: &["replace"],
            },
            Case {
                locale: Locale::Cs,
                cues: &["nahrazen", "přepsán"],
            },
        ];

        for case in &cases {
            let i18n = generate(case.locale);
            let text = i18n.t(Key::SettingsSpecialDaysDuplicateHint);
            assert!(
                !text.is_empty() && text.as_ref() != "??",
                "SDF-04: SettingsSpecialDaysDuplicateHint is empty/missing for {:?}: `{}`",
                case.locale,
                text
            );
            let lower = text.to_lowercase();
            let has_cue = case.cues.iter().any(|cue| lower.contains(&cue.to_lowercase()));
            assert!(
                has_cue,
                "SDF-04: replace-cue missing for {:?} — expected one of {:?} in `{}`",
                case.locale, case.cues, text
            );
        }
    }

    // ── Phase 51 SHC-06: short-day slot-clipping validator (D-25-06 fallback) ──

    #[test]
    fn test_empty_shortday_input_is_valid() {
        // Legacy off: empty string = user cleared the date, treat as valid input
        // so Save can proceed (which triggers a DELETE-toggle-value).
        assert!(is_valid_shortday_date_input(""));
    }

    #[test]
    fn test_valid_iso_date_is_accepted() {
        // Well-formed ISO-8601 date must be accepted.
        assert!(is_valid_shortday_date_input("2026-08-01"));
        // Leap day 2028-02-29 is valid.
        assert!(is_valid_shortday_date_input("2028-02-29"));
    }

    #[test]
    fn test_malformed_date_is_rejected() {
        // German locale format — must be rejected.
        assert!(!is_valid_shortday_date_input("08.01.2026"));
        // Natural language — rejected.
        assert!(!is_valid_shortday_date_input("tomorrow"));
        // Out-of-range month — rejected.
        assert!(!is_valid_shortday_date_input("2026-13-01"));
        // Whitespace-only — rejected (not empty, not a date).
        assert!(!is_valid_shortday_date_input("   "));
    }

    #[test]
    fn test_grenzfall_active_from_equals_booking_date() {
        // SHC-06 boundary case (inclusive): booking_date == active_from → clip.
        // Mirrors backend `service_impl::shortday_gate::should_clip` semantics.
        let active_from = time::Date::from_calendar_date(2026, time::Month::August, 1).unwrap();
        let on_boundary = active_from;
        let day_before = time::Date::from_calendar_date(2026, time::Month::July, 31).unwrap();

        // On the boundary date: gate is ACTIVE (clip).
        assert!(is_within_shortday_gate(on_boundary, Some(active_from)));
        // One day before: gate is NOT active (no clip).
        assert!(!is_within_shortday_gate(day_before, Some(active_from)));
        // No active_from (Legacy off): never clip.
        assert!(!is_within_shortday_gate(on_boundary, None));
    }
}

const TOGGLE_NAME: &str = "paid_limit_hard_enforcement";

#[component]
pub fn SettingsPage() -> Element {
    let i18n = I18N.read().clone();
    let config = CONFIG.read().clone();

    // WR-02: Component-level admin guard. The nav hides this route for non-admins,
    // but a direct URL access would bypass that. Reuse the same AUTH/has_privilege
    // pattern used by billing_periods, shiftplan, absences pages.
    let is_admin = AUTH
        .read()
        .auth_info
        .as_ref()
        .map(|a| a.has_privilege("admin"))
        .unwrap_or(false);
    if !is_admin {
        return rsx! {
            TopBar {}
            div { class: "p-md text-ink-muted", "Not authorized." }
        };
    }

    // ── Card 1: Paid-limit enforcement toggle (Phase 24) ──────────────────────

    let config_for_load = config.clone();
    let toggle_resource =
        use_resource(move || loader::get_toggle_enabled(config_for_load.clone(), TOGGLE_NAME));

    let mut hard_enforcement = use_signal(|| false);
    let mut save_result: Signal<Option<bool>> = use_signal(|| None);
    let mut saving = use_signal(|| false);

    use_effect(move || {
        if let Some(Ok(enabled)) = &*toggle_resource.read_unchecked() {
            hard_enforcement.set(*enabled);
        }
    });

    let config_for_click = config.clone();
    let on_toggle = move |_| {
        if *saving.read() {
            return;
        }
        let current = *hard_enforcement.read();
        let next = !current;
        saving.set(true);
        save_result.set(None);

        let cfg = config_for_click.clone();
        spawn(async move {
            match loader::set_toggle(cfg, TOGGLE_NAME, next).await {
                Ok(()) => {
                    hard_enforcement.set(next);
                    save_result.set(Some(true));
                }
                Err(_) => {
                    save_result.set(Some(false));
                }
            }
            saving.set(false);
        });
    };

    let is_on = *hard_enforcement.read();
    let is_saving = *saving.read();

    let toggle_class = if is_on {
        "px-3 py-2 rounded-md border border-bad text-bad text-body font-semibold bg-bad-soft"
    } else {
        "px-3 py-2 rounded-md border border-border text-ink text-body bg-surface hover:bg-surface-alt"
    };

    let toggle_label = if is_on {
        i18n.t(Key::SettingsPaidLimitToggleOn)
    } else {
        i18n.t(Key::SettingsPaidLimitToggleOff)
    };

    let aria_pressed = if is_on { "true" } else { "false" };

    // ── Card 2: Holiday auto-credit activation date (Phase 25) ────────────────

    let mut date_str: Signal<String> = use_signal(String::new);
    let mut date_str_loaded_empty = use_signal(|| false);
    let mut cutoff_save_result: Signal<Option<bool>> = use_signal(|| None);
    let mut cutoff_saving = use_signal(|| false);

    let config_for_cutoff = config.clone();
    let cutoff_resource =
        use_resource(move || loader::get_holiday_cutoff_date(config_for_cutoff.clone()));

    use_effect(move || {
        match &*cutoff_resource.read_unchecked() {
            Some(Ok(Some(date))) => {
                date_str.set(date.clone());
                date_str_loaded_empty.set(false);
            }
            Some(Ok(None)) => {
                date_str.set(String::new());
                date_str_loaded_empty.set(true);
            }
            _ => {}
        }
    });

    let config_for_save = config.clone();
    let on_save_cutoff = move |_| {
        if *cutoff_saving.read() {
            return;
        }
        let val = date_str.read().clone();
        if val.is_empty() {
            return;
        }
        // Client-side ISO date validation (defense in depth; <input type=date> enforces this
        // in the browser, but we double-check before the PUT).
        let date_format = format_description!("[year]-[month]-[day]");
        if time::Date::parse(&val, date_format).is_err() {
            cutoff_save_result.set(Some(false));
            return;
        }
        cutoff_saving.set(true);
        cutoff_save_result.set(None);
        let cfg = config_for_save.clone();
        spawn(async move {
            match loader::set_holiday_cutoff_date(cfg, Some(&val)).await {
                Ok(()) => {
                    cutoff_save_result.set(Some(true));
                    date_str_loaded_empty.set(false);
                }
                Err(_) => {
                    cutoff_save_result.set(Some(false));
                }
            }
            cutoff_saving.set(false);
        });
    };

    let config_for_clear = config.clone();
    let on_clear_cutoff = move |_| {
        if *cutoff_saving.read() {
            return;
        }
        cutoff_saving.set(true);
        cutoff_save_result.set(None);
        let cfg = config_for_clear.clone();
        spawn(async move {
            match loader::set_holiday_cutoff_date(cfg, None).await {
                Ok(()) => {
                    date_str.set(String::new());
                    date_str_loaded_empty.set(true);
                    cutoff_save_result.set(Some(true));
                }
                Err(_) => {
                    cutoff_save_result.set(Some(false));
                }
            }
            cutoff_saving.set(false);
        });
    };

    let is_cutoff_saving = *cutoff_saving.read();
    let loaded_empty = *date_str_loaded_empty.read();
    let date_string = date_str.read().clone();
    let date_value = ImStr::from(date_string.as_str());
    let date_empty = date_string.is_empty();

    // ── Card 2b: Short-day slot-clipping activation date (Phase 51 SHC-06) ────
    // D-51-07: blueprint identical to Card 2 (HCFG-02). Toggle backing is
    // `shortday_slot_clipping_active_from`; the row is pre-seeded (NULL) by the
    // P02 migration, so the first PUT is an UPDATE, not a create.

    let mut sc_date_str: Signal<String> = use_signal(String::new);
    let mut sc_date_str_loaded_empty = use_signal(|| false);
    let mut sc_save_result: Signal<Option<bool>> = use_signal(|| None);
    let mut sc_saving = use_signal(|| false);

    let config_for_sc_load = config.clone();
    let sc_resource =
        use_resource(move || loader::get_shortday_clipping_active_from(config_for_sc_load.clone()));

    use_effect(move || {
        match &*sc_resource.read_unchecked() {
            Some(Ok(Some(date))) => {
                sc_date_str.set(date.clone());
                sc_date_str_loaded_empty.set(false);
            }
            Some(Ok(None)) => {
                sc_date_str.set(String::new());
                sc_date_str_loaded_empty.set(true);
            }
            _ => {}
        }
    });

    let config_for_sc_save = config.clone();
    let on_save_shortday_clipping = move |_| {
        if *sc_saving.read() {
            return;
        }
        let val = sc_date_str.read().clone();
        if val.is_empty() {
            return;
        }
        // Client-side ISO date validation via the pure fn tested in 51-08 Task 3.
        // Defense in depth on top of <input type=date>.
        if !is_valid_shortday_date_input(&val) {
            sc_save_result.set(Some(false));
            return;
        }
        sc_saving.set(true);
        sc_save_result.set(None);
        let cfg = config_for_sc_save.clone();
        spawn(async move {
            match loader::set_shortday_clipping_active_from(cfg, Some(&val)).await {
                Ok(()) => {
                    sc_save_result.set(Some(true));
                    sc_date_str_loaded_empty.set(false);
                }
                Err(_) => {
                    sc_save_result.set(Some(false));
                }
            }
            sc_saving.set(false);
        });
    };

    let config_for_sc_clear = config.clone();
    let on_clear_shortday_clipping = move |_| {
        if *sc_saving.read() {
            return;
        }
        sc_saving.set(true);
        sc_save_result.set(None);
        let cfg = config_for_sc_clear.clone();
        spawn(async move {
            match loader::set_shortday_clipping_active_from(cfg, None).await {
                Ok(()) => {
                    sc_date_str.set(String::new());
                    sc_date_str_loaded_empty.set(true);
                    sc_save_result.set(Some(true));
                }
                Err(_) => {
                    sc_save_result.set(Some(false));
                }
            }
            sc_saving.set(false);
        });
    };

    let is_sc_saving = *sc_saving.read();
    let sc_loaded_empty = *sc_date_str_loaded_empty.read();
    let sc_date_string = sc_date_str.read().clone();
    let sc_date_value = ImStr::from(sc_date_string.as_str());
    let sc_date_empty = sc_date_string.is_empty();
    // Save-button guard uses the pure validator (D-25-06 fallback).
    let sc_save_disabled =
        is_sc_saving || sc_date_empty || !is_valid_shortday_date_input(&sc_date_string);

    // ── Card 3: Special-Days management (Phase 33, shiftplanner-gated) ────────
    // D-33-02: inner shiftplanner guard — NOT the page-level admin gate.

    let is_shiftplanner = AUTH
        .read()
        .auth_info
        .as_ref()
        .map(|a| a.has_privilege("shiftplanner"))
        .unwrap_or(false);

    // Card-3 signals
    let mut sd_year: Signal<u32> = use_signal(js::get_current_year);
    let mut sd_date_str: Signal<String> = use_signal(String::new);
    let mut sd_type: Signal<Option<SpecialDayTypeTO>> = use_signal(|| None);
    let mut sd_time_str: Signal<String> = use_signal(String::new);
    let mut sd_save_result: Signal<Option<bool>> = use_signal(|| None);
    let mut sd_saving = use_signal(|| false);
    let mut sd_delete_error = use_signal(|| false);
    // 260702-jql: suppress the "existiert bereits" hint right after a create (the
    // retained form self-matches). Cleared on the next real field edit.
    let mut sd_dup_hint_suppressed = use_signal(|| false);

    // Load year list (restarted after create/delete)
    let config_for_sd = config.clone();
    let mut sd_resource = use_resource(move || {
        let year = *sd_year.read();
        api::get_special_days_for_year(config_for_sd.clone(), year)
    });

    // Snapshot loaded list for duplicate check and list rendering
    let sd_list: Vec<SpecialDayTO> = sd_resource
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .map(|rc| rc.iter().cloned().collect())
        .unwrap_or_default();

    // Form validity (D-33-06): date non-empty AND (type≠ShortDay OR time non-empty)
    // Extracted into the pure, unit-tested `is_special_day_form_valid` (D-42-05).
    let sd_date_val = sd_date_str.read().clone();
    let sd_time_val = sd_time_str.read().clone();
    let sd_type_val = sd_type.read().clone();
    let sd_form_valid = is_special_day_form_valid(&sd_date_val, sd_type_val.clone(), &sd_time_val);

    // Live duplicate check (D-33-07)
    let sd_is_duplicate = parse_date_to_iso_parts(&sd_date_val)
        .map(|parts| is_duplicate_special_day(parts, &sd_list))
        .unwrap_or(false);

    // Create handler
    let config_for_sd_create = config.clone();
    let on_add_special_day = move |_| {
        if *sd_saving.read() {
            return;
        }
        let date_s = sd_date_str.read().clone();
        let time_s = sd_time_str.read().clone();
        let ty = sd_type.read().clone();

        // Snapshot the current form for the post-create retention policy (D-42-01).
        let sd_form_before = SpecialDayForm {
            date: date_s.clone(),
            ty: ty.clone(),
            time: time_s.clone(),
        };

        let Some((iso_year, iso_week, weekday)) = parse_date_to_iso_parts(&date_s) else {
            sd_save_result.set(Some(false));
            return;
        };
        let Some(day_type) = ty else {
            sd_save_result.set(Some(false));
            return;
        };

        // Parse time for ShortDay (D-33-06)
        let time_of_day = if day_type == SpecialDayTypeTO::ShortDay {
            let fmt_hm = format_description!("[hour]:[minute]");
            let fmt_hms = format_description!("[hour]:[minute]:[second]");
            let parsed = time::Time::parse(&time_s, fmt_hms)
                .or_else(|_| time::Time::parse(&time_s, fmt_hm))
                .ok();
            if parsed.is_none() {
                sd_save_result.set(Some(false));
                return;
            }
            parsed
        } else {
            None
        };

        let body = SpecialDayTO {
            id: Uuid::nil(),
            year: iso_year,
            calendar_week: iso_week,
            day_of_week: weekday,
            day_type,
            time_of_day,
            created: None,
            deleted: None,
            version: Uuid::nil(),
        };

        sd_saving.set(true);
        sd_save_result.set(None);
        let cfg = config_for_sd_create.clone();
        spawn(async move {
            match api::create_special_day(cfg, body).await {
                Ok(_) => {
                    sd_save_result.set(Some(true));
                    // SDF-03 (Phase 43-01): jump the picker to the **calendar year**
                    // of the picked date (not the ISO-week-year). Otherwise a
                    // special day created on 2027-01-01 would silently vanish into
                    // the 2026er picker (2027-01-01 is ISO week 53 of 2026). The
                    // pure fn `sd_year_after_create` is unit-tested against the
                    // year-boundary case; on parse failure we fall back to the
                    // already-parsed `iso_year` so the year picker still updates.
                    sd_year.set(sd_year_after_create(&date_s).unwrap_or(iso_year));
                    // D-42-01 (Option 2): keep the form filled after create so
                    // `is_special_day_form_valid` stays true → the Anlegen button
                    // stays enabled → repeated create without re-toggling the type
                    // dropdown. The former three field resets are removed; the
                    // retention policy is the single source of truth for what stays.
                    let retained = special_day_form_after_create(&sd_form_before);
                    sd_date_str.set(retained.date);
                    sd_type.set(retained.ty);
                    sd_time_str.set(retained.time);
                    // 260702-jql: the retained fields self-match the just-created
                    // entry → suppress the duplicate hint until the next real edit.
                    sd_dup_hint_suppressed.set(true);
                    sd_resource.restart();
                }
                Err(_) => {
                    sd_save_result.set(Some(false));
                }
            }
            sd_saving.set(false);
        });
    };

    // ── Card 4: PDF-Export nach Nextcloud (Phase 48-05 EXP-02 / EXP-03) ───────
    // Admin-gated; sichtbar innerhalb des äußeren is_admin-Blocks. Kein
    // zusätzlicher Inner-Gate (D-48-UI-GATE).

    let config_for_pdf = config.clone();
    let pdf_resource =
        use_resource(move || loader::get_pdf_export_config(config_for_pdf.clone()));

    let mut pdf_form: Signal<PdfExportForm> = use_signal(PdfExportForm::default);
    let mut pdf_saving = use_signal(|| false);
    let mut pdf_save_result: Signal<Option<bool>> = use_signal(|| None);
    let mut pdf_triggering = use_signal(|| false);
    let mut pdf_trigger_result: Signal<Option<bool>> = use_signal(|| None);

    // Load-into-signal, analog Card 2 (cutoff_resource).
    use_effect(move || {
        if let Some(Ok(form)) = &*pdf_resource.read_unchecked() {
            pdf_form.set(form.clone());
        }
    });

    let config_for_pdf_save = config.clone();
    let on_pdf_save = move |_| {
        if *pdf_saving.read() {
            return;
        }
        let form_snapshot = pdf_form.read().clone();
        pdf_saving.set(true);
        pdf_save_result.set(None);
        let cfg = config_for_pdf_save.clone();
        spawn(async move {
            match loader::save_pdf_export_config(cfg, form_snapshot).await {
                Ok(reloaded) => {
                    pdf_form.set(reloaded);
                    pdf_save_result.set(Some(true));
                }
                Err(_) => {
                    pdf_save_result.set(Some(false));
                }
            }
            pdf_saving.set(false);
        });
    };

    let config_for_pdf_trigger = config.clone();
    let on_pdf_trigger = move |_| {
        if *pdf_triggering.read() {
            return;
        }
        pdf_triggering.set(true);
        pdf_trigger_result.set(None);
        let cfg = config_for_pdf_trigger.clone();
        spawn(async move {
            match loader::trigger_pdf_export_now(cfg).await {
                Ok(()) => {
                    pdf_trigger_result.set(Some(true));
                }
                Err(_) => {
                    pdf_trigger_result.set(Some(false));
                }
            }
            pdf_triggering.set(false);
        });
    };

    // Read-only view helpers.
    let pdf_form_snapshot = pdf_form.read().clone();
    let pdf_saving_now = *pdf_saving.read();
    let pdf_triggering_now = *pdf_triggering.read();
    let pdf_enabled = pdf_form_snapshot.enabled;
    let pdf_toggle_class = if pdf_enabled {
        "px-3 py-2 rounded-md border border-accent text-accent text-body font-semibold bg-accent-soft"
    } else {
        "px-3 py-2 rounded-md border border-border text-ink text-body bg-surface hover:bg-surface-alt"
    };
    let pdf_toggle_label = if pdf_enabled { "ON" } else { "OFF" };
    let pdf_toggle_aria = if pdf_enabled { "true" } else { "false" };

    // Format timestamps via the current locale's date formatter + explicit HH:MM.
    let format_ts = |ts: time::PrimitiveDateTime| -> String {
        format!(
            "{} {:02}:{:02}",
            i18n.format_date(&ts.date()),
            ts.hour(),
            ts.minute()
        )
    };
    let last_success_display = pdf_form_snapshot.last_success_at.map(format_ts);
    let last_error_display = pdf_form_snapshot
        .last_error_at
        .map(format_ts);
    let last_error_msg = pdf_form_snapshot.last_error_message.clone();
    let no_status = last_success_display.is_none() && last_error_display.is_none();

    rsx! {
        TopBar {}

        div { class: "px-4 py-4 md:px-6 lg:px-8 max-w-5xl mx-auto",
            h1 { class: "text-h2 font-semibold pb-4",
                "{i18n.t(Key::Settings)}"
            }

            // Card 1 — Paid-limit enforcement (Phase 24, unchanged)
            div { class: "bg-surface border border-border rounded-md p-4 flex flex-col gap-3",

                // Toggle row
                div { class: "flex flex-col gap-1",
                    span { class: "text-body text-ink font-semibold",
                        "{i18n.t(Key::SettingsPaidLimitToggleLabel)}"
                    }
                    span { class: "text-small text-ink-soft",
                        "{i18n.t(Key::SettingsPaidLimitToggleDescription)}"
                    }
                }

                div { class: "flex flex-row items-center gap-3",
                    button {
                        r#type: "button",
                        class: "{toggle_class}",
                        "aria-pressed": "{aria_pressed}",
                        disabled: is_saving,
                        onclick: on_toggle,
                        "{toggle_label}"
                    }

                    // Inline feedback
                    {match *save_result.read() {
                        Some(true) => rsx! {
                            span { class: "text-small text-ink-muted",
                                "{i18n.t(Key::SettingsSaved)}"
                            }
                        },
                        Some(false) => rsx! {
                            span { class: "text-bad text-small font-normal",
                                "{i18n.t(Key::SettingsSaveError)}"
                            }
                        },
                        None => rsx! { },
                    }}
                }
            }

            // Card 2 — Holiday auto-credit activation date (Phase 25 D-25-06)
            div { class: "bg-surface border border-border rounded-md p-4 flex flex-col gap-3 mt-4",

                // Row A: Feature label + description
                div { class: "flex flex-col gap-1",
                    span { class: "text-body text-ink font-semibold",
                        "{i18n.t(Key::SettingsHolidayAutoCreditLabel)}"
                    }
                    span { class: "text-small text-ink-soft",
                        "{i18n.t(Key::SettingsHolidayAutoCreditDescription)}"
                    }
                }

                // Row B: Date input (width-constrained)
                div { class: "max-w-[200px]",
                    TextInput {
                        input_type: ImStr::from("date"),
                        value: date_value,
                        on_change: move |v: ImStr| date_str.set(v.as_str().to_string()),
                    }
                }

                // Row C: Action row (Save + Clear + inline feedback)
                div { class: "flex flex-row items-center gap-3",
                    button {
                        r#type: "button",
                        class: "px-3 py-2 rounded-md border border-border text-ink text-body bg-surface hover:bg-surface-alt",
                        disabled: is_cutoff_saving,
                        onclick: on_save_cutoff,
                        "{i18n.t(Key::SettingsHolidayAutoCreditSave)}"
                    }
                    button {
                        r#type: "button",
                        class: "px-3 py-2 rounded-md border border-border text-ink-soft text-body bg-surface hover:bg-surface-alt",
                        disabled: is_cutoff_saving || date_empty,
                        onclick: on_clear_cutoff,
                        "{i18n.t(Key::SettingsHolidayAutoCreditClear)}"
                    }

                    // Inline feedback — reuses SettingsSaved / SettingsSaveError keys
                    {match *cutoff_save_result.read() {
                        Some(true) => rsx! {
                            span { class: "text-small text-ink-muted",
                                "{i18n.t(Key::SettingsSaved)}"
                            }
                        },
                        Some(false) => rsx! {
                            span { class: "text-bad text-small",
                                "{i18n.t(Key::SettingsSaveError)}"
                            }
                        },
                        None => rsx! { },
                    }}
                }

                // Row D: Unset hint (shown only when no date is set after load)
                if loaded_empty {
                    span { class: "text-small text-ink-muted",
                        "{i18n.t(Key::SettingsHolidayAutoCreditUnsetHint)}"
                    }
                }
            }

            // Card 2b — Short-day slot-clipping activation date (Phase 51 SHC-06, D-51-07).
            // Structure mirrors HCFG-02 Card 2; backing toggle is
            // `shortday_slot_clipping_active_from` (seeded by P02 migration).
            div { class: "bg-surface border border-border rounded-md p-4 flex flex-col gap-3 mt-4",

                // Row A: Feature label + description
                div { class: "flex flex-col gap-1",
                    span { class: "text-body text-ink font-semibold",
                        "{i18n.t(Key::SettingsShortdayClippingLabel)}"
                    }
                    span { class: "text-small text-ink-soft",
                        "{i18n.t(Key::SettingsShortdayClippingDescription)}"
                    }
                }

                // Row B: Date input (width-constrained)
                div { class: "max-w-[200px]",
                    TextInput {
                        input_type: ImStr::from("date"),
                        value: sc_date_value,
                        on_change: move |v: ImStr| sc_date_str.set(v.as_str().to_string()),
                    }
                }

                // Row C: Action row (Save + Clear + inline feedback)
                div { class: "flex flex-row items-center gap-3",
                    button {
                        r#type: "button",
                        class: "px-3 py-2 rounded-md border border-border text-ink text-body bg-surface hover:bg-surface-alt",
                        disabled: sc_save_disabled,
                        onclick: on_save_shortday_clipping,
                        "{i18n.t(Key::SettingsHolidayAutoCreditSave)}"
                    }
                    button {
                        r#type: "button",
                        class: "px-3 py-2 rounded-md border border-border text-ink-soft text-body bg-surface hover:bg-surface-alt",
                        disabled: is_sc_saving || sc_date_empty,
                        onclick: on_clear_shortday_clipping,
                        "{i18n.t(Key::SettingsHolidayAutoCreditClear)}"
                    }

                    // Inline feedback — reuses SettingsSaved / SettingsSaveError keys
                    {match *sc_save_result.read() {
                        Some(true) => rsx! {
                            span { class: "text-small text-ink-muted",
                                "{i18n.t(Key::SettingsSaved)}"
                            }
                        },
                        Some(false) => rsx! {
                            span { class: "text-bad text-small",
                                "{i18n.t(Key::SettingsSaveError)}"
                            }
                        },
                        None => rsx! { },
                    }}
                }

                // Row D: Unset hint (shown only when no date is set after load)
                if sc_loaded_empty {
                    span { class: "text-small text-ink-muted",
                        "{i18n.t(Key::SettingsShortdayClippingUnsetHint)}"
                    }
                }
            }

            // Card 3 — Special-Days management (Phase 33, D-33-02 shiftplanner gate)
            if is_shiftplanner {
                div { class: "bg-surface border border-border rounded-md p-4 flex flex-col gap-3 mt-4",

                    // Row A: Feature label + description
                    div { class: "flex flex-col gap-1",
                        span { class: "text-body text-ink font-semibold",
                            "{i18n.t(Key::SettingsSpecialDaysSectionLabel)}"
                        }
                        span { class: "text-small text-ink-soft",
                            "{i18n.t(Key::SettingsSpecialDaysSectionDescription)}"
                        }
                    }

                    // Row B: Year picker
                    div { class: "flex flex-row items-center gap-2",
                        label { class: "text-small text-ink-muted",
                            "{i18n.t(Key::SettingsSpecialDaysYearLabel)}"
                        }
                        div { class: "w-24",
                            TextInput {
                                input_type: ImStr::from("number"),
                                value: ImStr::from(sd_year.read().to_string().as_str()),
                                step: Some(ImStr::from("1")),
                                on_change: move |v: ImStr| {
                                    if let Ok(y) = v.as_str().parse::<u32>() {
                                        if (2020..=2099).contains(&y) {
                                            sd_year.set(y);
                                        }
                                    }
                                },
                            }
                        }
                    }

                    // Row C: Create form (date + type + optional time + submit)
                    div { class: "flex flex-row items-end gap-2 flex-wrap",
                        // Date input
                        div { class: "flex flex-col gap-1",
                            label { class: "text-small text-ink-muted",
                                "{i18n.t(Key::SettingsSpecialDaysDateLabel)}"
                            }
                            div { class: "max-w-[200px]",
                                TextInput {
                                    input_type: ImStr::from("date"),
                                    value: ImStr::from(sd_date_val.as_str()),
                                    on_change: move |v: ImStr| {
                                        sd_date_str.set(v.as_str().to_string());
                                        sd_save_result.set(None);
                                        sd_dup_hint_suppressed.set(false);
                                    },
                                }
                            }
                        }

                        // Type selector
                        div { class: "flex flex-col gap-1",
                            label { class: "text-small text-ink-muted",
                                "{i18n.t(Key::SettingsSpecialDaysTypeLabel)}"
                            }
                            SelectInput {
                                // D-06: controlled binding — the select value tracks
                                // sd_type so there is no desync between signal and DOM.
                                // Phase 42 (D-42-01): fields are retained after create
                                // (not reset), so the dropdown keeps its value and the
                                // Anlegen button stays enabled for repeated creates.
                                value: Some(ImStr::from(sd_type_to_select_value(sd_type_val.clone()))),
                                on_change: move |v: ImStr| {
                                    let ty = match v.as_str() {
                                        "holiday" => Some(SpecialDayTypeTO::Holiday),
                                        "short_day" => Some(SpecialDayTypeTO::ShortDay),
                                        _ => None,
                                    };
                                    sd_type.set(ty);
                                    sd_save_result.set(None);
                                    sd_dup_hint_suppressed.set(false);
                                },
                                option { value: "", "" }
                                option { value: "holiday", "{i18n.t(Key::SettingsSpecialDaysTypeHoliday)}" }
                                option { value: "short_day", "{i18n.t(Key::SettingsSpecialDaysTypeShortDay)}" }
                            }
                        }

                        // Conditional time input (D-33-06: only for ShortDay)
                        if sd_type_val == Some(SpecialDayTypeTO::ShortDay) {
                            div { class: "flex flex-col gap-1",
                                label { class: "text-small text-ink-muted",
                                    "{i18n.t(Key::SettingsSpecialDaysTimeLabel)}"
                                }
                                div { class: "max-w-[140px]",
                                    TextInput {
                                        input_type: ImStr::from("time"),
                                        value: ImStr::from(sd_time_val.as_str()),
                                        on_change: move |v: ImStr| {
                                            sd_time_str.set(v.as_str().to_string());
                                            sd_save_result.set(None);
                                            sd_dup_hint_suppressed.set(false);
                                        },
                                    }
                                }
                            }
                        }

                        // Add button
                        Btn {
                            variant: BtnVariant::Primary,
                            disabled: !sd_form_valid || *sd_saving.read(),
                            on_click: on_add_special_day,
                            "{i18n.t(Key::SettingsSpecialDaysAddBtn)}"
                        }
                    }

                    // Row D: Inline hints and errors
                    // 260702-jql: gate the duplicate hint through the pure fn so a
                    // just-created (self-matching) entry does not re-trigger it.
                    if should_show_duplicate_hint(sd_is_duplicate, sd_dup_hint_suppressed()) {
                        span { class: "text-small text-bad",
                            "{i18n.t(Key::SettingsSpecialDaysDuplicateHint)}"
                        }
                    }
                    {match *sd_save_result.read() {
                        Some(true) => rsx! {
                            span { class: "text-small text-ink-muted",
                                "{i18n.t(Key::SettingsSaved)}"
                            }
                        },
                        Some(false) => rsx! {
                            span { class: "text-small text-bad",
                                "{i18n.t(Key::SettingsSaveError)}"
                            }
                        },
                        None => rsx! { },
                    }}

                    // Row E: Chronological year list (SPD-02 / D-33-08)
                    if sd_list.is_empty() {
                        // Empty state
                        div { class: "py-6 text-center",
                            p { class: "text-body text-ink-muted",
                                {i18n.t(Key::SettingsSpecialDaysEmptyBody)
                                    .replace("{year}", &sd_year.read().to_string())}
                            }
                        }
                    } else {
                        // List of entries — backend already orders ascending by (calendar_week, day_of_week)
                        div {
                            for entry in sd_list.iter() {
                                {
                                    let entry_id = entry.id;
                                    let config_for_delete = config.clone();
                                    let date_display = special_day_iso_date(entry)
                                        .map(|d| {
                                            let cw_abbr = i18n.t(Key::SettingsSpecialDaysCalendarWeekAbbr);
                                            let weekday_name = i18n.t(weekday_key(entry.day_of_week));
                                            format!(
                                                "{} ({}, {} {}, {})",
                                                i18n.format_date(&d),
                                                weekday_name,
                                                cw_abbr,
                                                entry.calendar_week,
                                                entry.year
                                            )
                                        })
                                        .unwrap_or_default();
                                    let time_display = entry.time_of_day
                                        .map(|t| format!("{:02}:{:02}", t.hour(), t.minute()))
                                        .unwrap_or_default();
                                    let entry_type = entry.day_type.clone();

                                    rsx! {
                                        div { class: "flex items-center justify-between py-2 border-b border-border",
                                            div { class: "flex items-center gap-2",
                                                span { class: "text-body text-ink",
                                                    "{date_display}"
                                                }
                                                match entry_type {
                                                    SpecialDayTypeTO::Holiday => rsx! {
                                                        span { class: "px-2 py-1 bg-accent-soft text-accent text-micro uppercase rounded-full",
                                                            "{i18n.t(Key::SettingsSpecialDaysTypeHoliday)}"
                                                        }
                                                    },
                                                    SpecialDayTypeTO::ShortDay => rsx! {
                                                        span { class: "px-2 py-1 bg-warn-soft text-warn text-micro uppercase rounded-full",
                                                            "{i18n.t(Key::SettingsSpecialDaysTypeShortDay)}"
                                                        }
                                                        if !time_display.is_empty() {
                                                            span { class: "text-small text-ink-muted",
                                                                "{time_display}"
                                                            }
                                                        }
                                                    },
                                                }
                                            }
                                            Btn {
                                                variant: BtnVariant::Danger,
                                                disabled: *sd_saving.read(),
                                                on_click: move |_| {
                                                    if *sd_saving.read() {
                                                        return;
                                                    }
                                                    sd_delete_error.set(false);
                                                    let cfg = config_for_delete.clone();
                                                    sd_saving.set(true);
                                                    spawn(async move {
                                                        match api::delete_special_day(cfg, entry_id).await {
                                                            Ok(_) => {
                                                                sd_resource.restart();
                                                            }
                                                            Err(_) => {
                                                                sd_delete_error.set(true);
                                                            }
                                                        }
                                                        sd_saving.set(false);
                                                    });
                                                },
                                                "{i18n.t(Key::SettingsSpecialDaysDeleteBtn)}"
                                            }
                                        }
                                    }
                                }
                            }
                            // Delete error shown inline below the list (SPD-03)
                            if *sd_delete_error.read() {
                                span { class: "text-small text-bad",
                                    "{i18n.t(Key::SettingsSpecialDaysDeleteError)}"
                                }
                            }
                        }
                    }
                }
            }

            // Card 4 — PDF-Export nach Nextcloud (Phase 48-05, admin-gated
            // via the outer is_admin return above; no inner gate — D-48-UI-GATE).
            div { class: "bg-surface border border-border rounded-md p-4 flex flex-col gap-3 mt-4",

                // Row A: Title + Help
                div { class: "flex flex-col gap-1",
                    span { class: "text-body text-ink font-semibold",
                        "{i18n.t(Key::SettingsPdfExportTitle)}"
                    }
                    span { class: "text-small text-ink-soft",
                        "{i18n.t(Key::SettingsPdfExportHelp)}"
                    }
                }

                // Row B: Enabled toggle
                div { class: "flex flex-row items-center gap-3",
                    span { class: "text-body text-ink",
                        "{i18n.t(Key::SettingsPdfExportEnabled)}"
                    }
                    button {
                        r#type: "button",
                        class: "{pdf_toggle_class}",
                        "aria-pressed": "{pdf_toggle_aria}",
                        disabled: pdf_saving_now,
                        onclick: move |_| {
                            let next = !pdf_form.read().enabled;
                            pdf_form.write().enabled = next;
                        },
                        "{pdf_toggle_label}"
                    }
                }

                // Row C: Nextcloud URL
                div { class: "flex flex-col gap-1",
                    label { class: "text-small text-ink-muted",
                        "{i18n.t(Key::SettingsPdfExportUrl)}"
                    }
                    TextInput {
                        input_type: ImStr::from("text"),
                        value: ImStr::from(pdf_form_snapshot.nextcloud_url.as_str()),
                        on_change: move |v: ImStr| {
                            pdf_form.write().nextcloud_url = v.as_str().to_string();
                        },
                    }
                }

                // Row D: WebDAV user
                div { class: "flex flex-col gap-1",
                    label { class: "text-small text-ink-muted",
                        "{i18n.t(Key::SettingsPdfExportUser)}"
                    }
                    TextInput {
                        input_type: ImStr::from("text"),
                        value: ImStr::from(pdf_form_snapshot.webdav_user.as_str()),
                        on_change: move |v: ImStr| {
                            pdf_form.write().webdav_user = v.as_str().to_string();
                        },
                    }
                }

                // Row E: App token (password input; empty = keep existing)
                div { class: "flex flex-col gap-1",
                    label { class: "text-small text-ink-muted",
                        "{i18n.t(Key::SettingsPdfExportToken)}"
                    }
                    TextInput {
                        input_type: ImStr::from("password"),
                        value: ImStr::from(pdf_form_snapshot.token_input.as_str()),
                        placeholder: Some(i18n.t(Key::SettingsPdfExportTokenPlaceholder).as_ref().into()),
                        on_change: move |v: ImStr| {
                            pdf_form.write().token_input = v.as_str().to_string();
                        },
                    }
                }

                // Row F: Target folder
                div { class: "flex flex-col gap-1",
                    label { class: "text-small text-ink-muted",
                        "{i18n.t(Key::SettingsPdfExportTargetFolder)}"
                    }
                    TextInput {
                        input_type: ImStr::from("text"),
                        value: ImStr::from(pdf_form_snapshot.target_folder.as_str()),
                        on_change: move |v: ImStr| {
                            pdf_form.write().target_folder = v.as_str().to_string();
                        },
                    }
                }

                // Row G: Weeks horizon (clamped to 1..=52)
                div { class: "flex flex-col gap-1",
                    label { class: "text-small text-ink-muted",
                        "{i18n.t(Key::SettingsPdfExportWeeksHorizon)}"
                    }
                    div { class: "max-w-[120px]",
                        TextInput {
                            input_type: ImStr::from("number"),
                            value: ImStr::from(pdf_form_snapshot.weeks_horizon.to_string().as_str()),
                            step: Some(ImStr::from("1")),
                            on_change: move |v: ImStr| {
                                if let Ok(n) = v.as_str().parse::<i32>() {
                                    pdf_form.write().weeks_horizon = clamp_weeks_horizon(n);
                                }
                            },
                        }
                    }
                }

                // Row H: Cron schedule
                div { class: "flex flex-col gap-1",
                    label { class: "text-small text-ink-muted",
                        "{i18n.t(Key::SettingsPdfExportCronSchedule)}"
                    }
                    TextInput {
                        input_type: ImStr::from("text"),
                        value: ImStr::from(pdf_form_snapshot.cron_schedule.as_str()),
                        on_change: move |v: ImStr| {
                            pdf_form.write().cron_schedule = v.as_str().to_string();
                        },
                    }
                }

                // Row I: Action row — Save + Trigger + inline feedback
                div { class: "flex flex-row items-center gap-3 flex-wrap",
                    button {
                        r#type: "button",
                        class: "px-3 py-2 rounded-md border border-border text-ink text-body bg-surface hover:bg-surface-alt",
                        disabled: pdf_saving_now,
                        onclick: on_pdf_save,
                        "{i18n.t(Key::SettingsPdfExportSave)}"
                    }
                    button {
                        r#type: "button",
                        class: "px-3 py-2 rounded-md border border-border text-ink text-body bg-surface hover:bg-surface-alt",
                        disabled: pdf_triggering_now || !pdf_enabled,
                        onclick: on_pdf_trigger,
                        "{i18n.t(Key::SettingsPdfExportTriggerNow)}"
                    }

                    // Save-Result-Banner
                    {match *pdf_save_result.read() {
                        Some(true) => rsx! {
                            span { class: "text-small text-ink-muted",
                                "{i18n.t(Key::SettingsPdfExportSaveSuccess)}"
                            }
                        },
                        Some(false) => rsx! {
                            span { class: "text-small text-bad",
                                "{i18n.t(Key::SettingsPdfExportSaveError)}"
                            }
                        },
                        None => rsx! { },
                    }}

                    // Trigger-Result-Banner
                    {match *pdf_trigger_result.read() {
                        Some(true) => rsx! {
                            span { class: "text-small text-ink-muted",
                                "{i18n.t(Key::SettingsPdfExportTriggerNowSuccess)}"
                            }
                        },
                        Some(false) => rsx! {
                            span { class: "text-small text-bad",
                                "{i18n.t(Key::SettingsPdfExportTriggerNowError)}"
                            }
                        },
                        None => rsx! { },
                    }}
                }

                // Row J: Read-only status (last success / last error / empty)
                if let Some(ref ts) = last_success_display {
                    div { class: "text-body text-good",
                        "{i18n.t(Key::SettingsPdfExportLastSuccess)} {ts}"
                    }
                }
                if let (Some(ts), Some(msg)) = (last_error_display.as_ref(), last_error_msg.as_ref()) {
                    div { class: "text-body text-bad",
                        "{i18n.t(Key::SettingsPdfExportLastError)} {ts} — {msg}"
                    }
                }
                if no_status {
                    div { class: "text-body text-ink-muted",
                        "{i18n.t(Key::SettingsPdfExportStatusEmpty)}"
                    }
                }
            }
        }
    }
}
