//! SettingsPage — admin-gated page with the paid-limit hard/soft toggle (Card 1),
//! the holiday auto-credit activation date field (Card 2), and the Special-Days
//! management card (Card 3, shiftplanner-gated).
//! Phase 24 D-24-06: paid-limit enforcement toggle.
//! Phase 25 D-25-06: holiday auto-credit cutoff date input.
//! Phase 33: Special-Days Settings Card (D-33-02/04/06/07/08).

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
    let sd_date_val = sd_date_str.read().clone();
    let sd_time_val = sd_time_str.read().clone();
    let sd_type_val = sd_type.read().clone();
    let sd_form_valid = !sd_date_val.is_empty()
        && (sd_type_val != Some(SpecialDayTypeTO::ShortDay) || !sd_time_val.is_empty());

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
                    sd_resource.restart();
                }
                Err(_) => {
                    sd_save_result.set(Some(false));
                }
            }
            sd_saving.set(false);
        });
    };

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
                                        if y >= 2020 && y <= 2099 {
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
                                on_change: move |v: ImStr| {
                                    let ty = match v.as_str() {
                                        "holiday" => Some(SpecialDayTypeTO::Holiday),
                                        "short_day" => Some(SpecialDayTypeTO::ShortDay),
                                        _ => None,
                                    };
                                    sd_type.set(ty);
                                    sd_save_result.set(None);
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
                    if sd_is_duplicate {
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
        }
    }
}
