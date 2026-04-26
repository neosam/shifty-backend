use std::collections::BTreeMap;

use dioxus::prelude::*;
use rest_types::BlockTO;

use crate::{
    base_types::ImStr,
    component::{PersonChip, TopBar},
    i18n::{I18n, Key, Locale},
    js, loader,
    service::{config::CONFIG, i18n::I18N},
    state::{shiftplan::Weekday, week::Week},
};

const ALL_WEEKDAYS: [Weekday; 7] = [
    Weekday::Monday,
    Weekday::Tuesday,
    Weekday::Wednesday,
    Weekday::Thursday,
    Weekday::Friday,
    Weekday::Saturday,
    Weekday::Sunday,
];

const DAY_ROW_STYLE: &str =
    "display: grid; grid-template-columns: 110px 1fr 60px; align-items: start; gap: 12px;";
const DAY_ROW_STYLE_MOBILE: &str =
    "display: grid; grid-template-columns: 80px 1fr 50px; align-items: start; gap: 8px;";

pub fn block_hours(block: &BlockTO) -> f32 {
    let duration = block.to - block.from;
    duration.as_seconds_f32() / 3600.0
}

pub fn sum_hours<'a, I>(blocks: I) -> f32
where
    I: IntoIterator<Item = &'a BlockTO>,
{
    blocks.into_iter().map(block_hours).sum()
}

pub fn format_hours(hours: f32) -> String {
    // `0.0 + (-0.0)` collapses the negative-zero bit so tiny rounding error
    // never renders as `-0.0` in the UI.
    let normalized = hours + 0.0;
    format!("{:.1}", normalized)
}

pub fn format_time_range(from: time::Time, to: time::Time) -> String {
    format!(
        "{:02}:{:02}\u{2013}{:02}:{:02}",
        from.hour(),
        from.minute(),
        to.hour(),
        to.minute()
    )
}

#[derive(Clone, PartialEq)]
pub struct ShiftDisplay {
    pub time_range: ImStr,
    pub person_name: ImStr,
    pub person_color: Option<ImStr>,
}

#[derive(Clone, PartialEq)]
pub struct DayDisplay {
    pub label: ImStr,
    pub shifts: Vec<ShiftDisplay>,
    pub total_hours_text: ImStr,
    /// Future hook: a per-day note text. When a future backend extends
    /// `BlockTO`/`SlotTO` with notes, populate this field. Notes starting
    /// with `\u{26A0}` (warning sign) will switch the row to the
    /// `bg-warn-soft` styling (see design.md Decision 5). Currently always
    /// `None`.
    pub note: Option<ImStr>,
}

#[derive(Clone, PartialEq)]
pub struct WeekDisplay {
    pub header_label: ImStr,
    pub total_hours_text: ImStr,
    pub days: Vec<DayDisplay>,
}

fn build_shift_display(block: &BlockTO) -> ShiftDisplay {
    let time_range = ImStr::from(format_time_range(block.from, block.to));
    match block.sales_person.as_ref() {
        Some(sp) => ShiftDisplay {
            time_range,
            person_name: ImStr::from(sp.name.as_ref()),
            person_color: Some(ImStr::from(sp.background_color.as_ref())),
        },
        None => ShiftDisplay {
            time_range,
            person_name: ImStr::from("-"),
            person_color: None,
        },
    }
}

fn build_day_display(label: ImStr, blocks: &[&BlockTO]) -> DayDisplay {
    let total = sum_hours(blocks.iter().copied());
    let shifts = blocks.iter().map(|b| build_shift_display(b)).collect();
    DayDisplay {
        label,
        shifts,
        total_hours_text: ImStr::from(format_hours(total)),
        note: None,
    }
}

/// Groups a flat list of blocks by `(year, week)` and drops weeks whose
/// list is empty so all-empty weeks never produce a card.
pub fn group_blocks_by_week(blocks: &[BlockTO]) -> BTreeMap<(u32, u8), Vec<BlockTO>> {
    let mut grouped: BTreeMap<(u32, u8), Vec<BlockTO>> = BTreeMap::new();
    for block in blocks.iter() {
        grouped
            .entry((block.year, block.week))
            .or_default()
            .push(block.clone());
    }
    grouped.retain(|_, v| !v.is_empty());
    grouped
}

pub fn build_week_display(
    i18n: &I18n<Key, Locale>,
    year: u32,
    week: u8,
    blocks: &[BlockTO],
) -> WeekDisplay {
    let week_info = Week { year, week };
    let monday = week_info.monday().ok();
    let sunday = week_info.sunday().ok();
    let date_range = match (monday, sunday) {
        (Some(mon), Some(sun)) => {
            format!("{} – {}", i18n.format_date(&mon), i18n.format_date(&sun))
        }
        _ => String::new(),
    };
    let week_label = i18n.t(Key::WeekLabel);
    let header_label = ImStr::from(format!("{week_label} {week} \u{00B7} {date_range}"));

    let total = sum_hours(blocks.iter());
    let total_hours_text = ImStr::from(format_hours(total));

    let days: Vec<DayDisplay> = ALL_WEEKDAYS
        .iter()
        .map(|wd| {
            let label = ImStr::from(wd.i18n_short_string(i18n));
            let day_blocks: Vec<&BlockTO> = blocks
                .iter()
                .filter(|b| Weekday::from(b.day_of_week) == *wd)
                .collect();
            build_day_display(label, &day_blocks)
        })
        .collect();

    WeekDisplay {
        header_label,
        total_hours_text,
        days,
    }
}

#[component]
pub fn ShiftItemView(shift: ShiftDisplay) -> Element {
    rsx! {
        div { class: "flex items-center gap-2",
            span {
                class: "font-mono tabular-nums text-sm text-ink",
                "{shift.time_range}"
            }
            PersonChip {
                name: shift.person_name.clone(),
                color: shift.person_color.clone(),
            }
        }
    }
}

#[component]
pub fn DayRowView(day: DayDisplay) -> Element {
    let has_shifts = !day.shifts.is_empty();
    rsx! {
        div {
            style: "{DAY_ROW_STYLE}",
            class: "my-shifts-day-row py-2",
            span { class: "font-mono tabular-nums text-sm text-ink-soft",
                "{day.label}"
            }
            div { class: "flex flex-col gap-1",
                if has_shifts {
                    for shift in day.shifts.iter().cloned() {
                        ShiftItemView { shift: shift }
                    }
                } else {
                    span { class: "text-ink-muted", "\u{2014}" }
                }
            }
            span { class: "font-mono tabular-nums text-sm text-ink text-right",
                "{day.total_hours_text}"
            }
        }
    }
}

#[component]
pub fn WeekCardView(week: WeekDisplay) -> Element {
    rsx! {
        section { class: "rounded-md border border-border bg-surface overflow-hidden",
            div { class: "flex items-baseline justify-between px-4 py-3 border-b border-border",
                span { class: "text-base font-semibold text-ink",
                    "{week.header_label}"
                }
                span { class: "font-mono tabular-nums text-sm text-ink",
                    "{week.total_hours_text}"
                }
            }
            div { class: "px-4 py-2 divide-y divide-border",
                for day in week.days.iter().cloned() {
                    DayRowView { day: day }
                }
            }
        }
    }
}

#[component]
pub fn MyShifts() -> Element {
    let config = CONFIG.read().clone();
    let i18n = I18N.read().clone();

    let from_year = js::get_current_year();
    let from_week = js::get_current_week();

    let (to_year, to_week) = {
        let weeks_ahead = 10;
        let mut year = from_year;
        let mut week = from_week + weeks_ahead;

        while week > 52 {
            week -= 52;
            year += 1;
        }

        (year, week)
    };

    let blocks = use_resource(move || {
        let config = config.clone();
        async move { loader::load_blocks(config, from_year, from_week, to_year, to_week).await }
    });

    rsx! {
        TopBar {}
        main { class: "mx-auto max-w-[760px] w-full px-4 py-6 md:py-8 space-y-4",
            h1 { class: "text-xl font-semibold text-ink",
                "{i18n.t(Key::MyShifts)}"
            }

            // Mobile-only layout override for day rows.
            style { {format!("@media (max-width: 720px) {{ .my-shifts-day-row {{ {} }} }}", DAY_ROW_STYLE_MOBILE)} }

            match &*blocks.read() {
                Some(Ok(block_list)) => {
                    let grouped = group_blocks_by_week(block_list);

                    if grouped.is_empty() {
                        rsx! {
                            div { class: "text-ink-muted px-4 py-3",
                                "{i18n.t(Key::NoShiftsFound)}"
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "space-y-4",
                                for ((year, week), week_blocks) in grouped.iter() {
                                    {
                                        let view = build_week_display(&i18n, *year, *week, week_blocks);
                                        rsx! {
                                            WeekCardView { week: view }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(err)) => {
                    rsx! {
                        div { class: "text-bad px-4 py-3",
                            "Error loading shifts: {err}"
                        }
                    }
                }
                None => {
                    rsx! {
                        div { class: "text-ink-muted px-4 py-3",
                            "Loading..."
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rest_types::{BookingTO, DayOfWeekTO, SalesPersonTO, SlotTO};
    use std::sync::Arc;
    use uuid::Uuid;

    fn make_block(
        day: DayOfWeekTO,
        from_h: u8,
        from_m: u8,
        to_h: u8,
        to_m: u8,
        sp: Option<SalesPersonTO>,
    ) -> BlockTO {
        BlockTO {
            year: 2026,
            week: 17,
            sales_person: sp,
            day_of_week: day,
            from: time::Time::from_hms(from_h, from_m, 0).unwrap(),
            to: time::Time::from_hms(to_h, to_m, 0).unwrap(),
            bookings: Vec::<BookingTO>::new(),
            slots: Vec::<SlotTO>::new(),
        }
    }

    fn make_sp(name: &str, color: &str) -> SalesPersonTO {
        SalesPersonTO {
            id: Uuid::nil(),
            name: Arc::<str>::from(name),
            background_color: Arc::<str>::from(color),
            is_paid: None,
            inactive: false,
            deleted: None,
            version: Uuid::nil(),
        }
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    fn make_i18n() -> I18n<Key, Locale> {
        crate::i18n::generate(Locale::De)
    }

    #[test]
    fn format_hours_one_decimal() {
        assert_eq!(format_hours(0.0), "0.0");
        assert_eq!(format_hours(-0.0), "0.0");
        assert_eq!(format_hours(5.5), "5.5");
        // `format!("{:.1}", ..)` uses banker's rounding; 12.25 rounds to 12.2.
        assert_eq!(format_hours(12.5), "12.5");
        assert_eq!(format_hours(4.5), "4.5");
    }

    #[test]
    fn block_hours_for_four_and_a_half() {
        let block = make_block(DayOfWeekTO::Monday, 9, 0, 13, 30, None);
        assert!((block_hours(&block) - 4.5).abs() < 0.01);
    }

    #[test]
    fn block_hours_for_two_hours() {
        let block = make_block(DayOfWeekTO::Tuesday, 14, 0, 16, 0, None);
        assert!((block_hours(&block) - 2.0).abs() < 0.01);
    }

    #[test]
    fn sum_hours_aggregates_blocks() {
        let blocks = [
            make_block(DayOfWeekTO::Monday, 9, 0, 12, 30, None),
            make_block(DayOfWeekTO::Monday, 14, 0, 16, 0, None),
        ];
        let total = sum_hours(blocks.iter());
        assert!((total - 5.5).abs() < 0.01);
    }

    #[test]
    fn format_time_range_uses_en_dash() {
        let s = format_time_range(
            time::Time::from_hms(9, 0, 0).unwrap(),
            time::Time::from_hms(13, 30, 0).unwrap(),
        );
        assert_eq!(s, "09:00\u{2013}13:30");
    }

    #[test]
    fn build_week_display_label_contains_week_and_dates() {
        let i18n = make_i18n();
        let blocks = [make_block(
            DayOfWeekTO::Monday,
            9,
            0,
            13,
            30,
            Some(make_sp("Lena", "#ffd6c1")),
        )];
        let view = build_week_display(&i18n, 2026, 17, &blocks);
        assert!(
            view.header_label.as_str().contains("17"),
            "header missing week number: {}",
            view.header_label
        );
        // German locale formats date as `dd.mm.yyyy`.
        assert!(
            view.header_label.as_str().contains("2026"),
            "header missing year: {}",
            view.header_label
        );
        assert_eq!(view.days.len(), 7);
        assert_eq!(view.days[0].shifts.len(), 1, "Monday should have one shift");
        for d in &view.days[1..] {
            assert!(d.shifts.is_empty(), "non-Monday should be empty");
            assert_eq!(
                d.total_hours_text.as_str(),
                "0.0",
                "empty days must render 0.0 (no -0.0 sign)"
            );
        }
        assert_eq!(view.total_hours_text.as_str(), "4.5");
        assert_eq!(view.days[0].total_hours_text.as_str(), "4.5");
    }

    #[test]
    fn build_week_display_two_blocks_one_day_sums() {
        let i18n = make_i18n();
        let blocks = [
            make_block(
                DayOfWeekTO::Wednesday,
                9,
                0,
                12,
                30,
                Some(make_sp("Lena", "#ffd6c1")),
            ),
            make_block(
                DayOfWeekTO::Wednesday,
                14,
                0,
                16,
                0,
                Some(make_sp("Lena", "#ffd6c1")),
            ),
        ];
        let view = build_week_display(&i18n, 2026, 17, &blocks);
        assert_eq!(
            view.days[2].shifts.len(),
            2,
            "Wednesday should have two shifts"
        );
        assert_eq!(view.days[2].total_hours_text.as_str(), "5.5");
        assert_eq!(view.total_hours_text.as_str(), "5.5");
    }

    #[test]
    fn shift_item_renders_time_range_and_person_chip() {
        fn app() -> Element {
            rsx! {
                ShiftItemView {
                    shift: ShiftDisplay {
                        time_range: ImStr::from("09:00\u{2013}13:30"),
                        person_name: ImStr::from("Lena"),
                        person_color: Some(ImStr::from("#ffd6c1")),
                    }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("09:00"));
        assert!(html.contains("13:30"));
        assert!(html.contains("Lena"));
        assert!(html.contains("background-color: #ffd6c1"));
        assert!(
            html.contains("person-pill"),
            "expected person-pill class: {html}"
        );
        assert!(html.contains("font-mono"));
        assert!(html.contains("tabular-nums"));
    }

    #[test]
    fn shift_item_without_sales_person_uses_dashed_chip() {
        fn app() -> Element {
            rsx! {
                ShiftItemView {
                    shift: ShiftDisplay {
                        time_range: ImStr::from("09:00\u{2013}13:30"),
                        person_name: ImStr::from("-"),
                        person_color: None,
                    }
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("person-pill"),
            "no-color path leaked person-pill: {html}"
        );
        assert!(
            html.contains("border-dashed"),
            "expected dashed border: {html}"
        );
    }

    #[test]
    fn day_row_empty_renders_em_dash_and_zero_hours() {
        fn app() -> Element {
            rsx! {
                DayRowView {
                    day: DayDisplay {
                        label: ImStr::from("Di"),
                        shifts: Vec::new(),
                        total_hours_text: ImStr::from("0.0"),
                        note: None,
                    }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Di"));
        assert!(html.contains("\u{2014}"), "em-dash missing: {html}");
        assert!(html.contains("text-ink-muted"));
        assert!(html.contains("0.0"));
    }

    #[test]
    fn day_row_with_shifts_renders_each() {
        fn app() -> Element {
            rsx! {
                DayRowView {
                    day: DayDisplay {
                        label: ImStr::from("Mo"),
                        shifts: vec![
                            ShiftDisplay {
                                time_range: ImStr::from("09:00\u{2013}12:30"),
                                person_name: ImStr::from("Lena"),
                                person_color: Some(ImStr::from("#ffd6c1")),
                            },
                            ShiftDisplay {
                                time_range: ImStr::from("14:00\u{2013}16:00"),
                                person_name: ImStr::from("Mara"),
                                person_color: Some(ImStr::from("#dbe0ff")),
                            },
                        ],
                        total_hours_text: ImStr::from("5.5"),
                        note: None,
                    }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("09:00"));
        assert!(html.contains("12:30"));
        assert!(html.contains("Lena"));
        assert!(html.contains("14:00"));
        assert!(html.contains("16:00"));
        assert!(html.contains("Mara"));
        assert!(html.contains("5.5"));
    }

    #[test]
    fn week_card_renders_header_and_seven_day_rows() {
        let i18n = make_i18n();
        let blocks = [make_block(
            DayOfWeekTO::Monday,
            9,
            0,
            13,
            30,
            Some(make_sp("Lena", "#ffd6c1")),
        )];
        let view = build_week_display(&i18n, 2026, 17, &blocks);
        let view_clone = view.clone();
        let app = move || {
            let v = view_clone.clone();
            rsx! { WeekCardView { week: v } }
        };
        let mut vdom = VirtualDom::new_with_props(app, ());
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        // Card surface uses tokens, no legacy gray.
        assert!(html.contains("bg-surface"), "missing bg-surface: {html}");
        assert!(html.contains("border-border"));
        assert!(!html.contains("bg-gray-100"));
        assert!(!html.contains("bg-white"));
        // Header text contains week number.
        assert!(html.contains("17"), "missing week number: {html}");
        // Total hours present in mono.
        assert!(html.contains("4.5"));
        // Lena (the one sales person) appears.
        assert!(html.contains("Lena"));
        // Seven day labels each appear at least once via short weekday strings (Mo, Di, ...).
        let count_em_dash = html.matches('\u{2014}').count();
        assert_eq!(
            count_em_dash, 6,
            "expected 6 empty-day em-dashes in a week with one shift, got {count_em_dash}: {html}"
        );
    }

    #[test]
    fn group_blocks_by_week_omits_empty_keys() {
        // A vec of blocks all in the same week — the result is a single entry,
        // never a key for a week we never had.
        let blocks = vec![make_block(DayOfWeekTO::Monday, 9, 0, 13, 0, None)];
        let grouped = group_blocks_by_week(&blocks);
        assert_eq!(grouped.len(), 1);
        assert!(grouped.contains_key(&(2026, 17)));
        // Empty input yields empty map (no zombie keys).
        let empty: Vec<BlockTO> = Vec::new();
        let grouped_empty = group_blocks_by_week(&empty);
        assert!(grouped_empty.is_empty());
    }

    #[test]
    fn group_blocks_by_week_groups_per_year_week_pair() {
        let mut a = make_block(DayOfWeekTO::Monday, 9, 0, 13, 0, None);
        a.week = 17;
        let mut b = make_block(DayOfWeekTO::Tuesday, 14, 0, 16, 0, None);
        b.week = 17;
        let mut c = make_block(DayOfWeekTO::Wednesday, 10, 0, 12, 0, None);
        c.week = 18;
        let grouped = group_blocks_by_week(&[a, b, c]);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get(&(2026, 17)).map(|v| v.len()), Some(2));
        assert_eq!(grouped.get(&(2026, 18)).map(|v| v.len()), Some(1));
    }

    #[test]
    fn source_does_not_use_legacy_gray_classes() {
        // Mirror of the banned list with sentinel-prefixed strings, so the
        // include_str! self-scan never matches its own array entries.
        let banned: &[&str] = &[
            concat!("bg-", "gray-100"),
            concat!("bg-", "gray-200"),
            concat!("bg-", "white"),
            concat!("text-", "gray-500"),
            concat!("text-", "gray-600"),
            concat!("text-", "gray-800"),
            concat!("text-", "red-600"),
        ];
        // We render the source string from a non-cfg(test) function so the
        // raw class names never appear in this test body literally.
        let src = include_str!("my_shifts.rs");
        // Strip the `#[cfg(test)] mod tests` block so test literals don't
        // trip the audit.
        let head = match src.find("#[cfg(test)]") {
            Some(idx) => &src[..idx],
            None => src,
        };
        for needle in banned {
            assert!(
                !head.contains(needle),
                "legacy class {needle} present in non-test source of my_shifts.rs"
            );
        }
    }
}
