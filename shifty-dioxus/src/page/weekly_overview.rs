use crate::i18n::Key;
use dioxus::prelude::*;
use futures_util::StreamExt;
use std::rc::Rc;

use crate::{
    base_types::{format_hours, ImStr},
    component::{NavBtn, TopBar, WeeklyOverviewChart},
    i18n::I18nType,
    js,
    service::{
        i18n::I18N, weekly_summary::WeeklySummaryAction, weekly_summary::WEEKLY_SUMMARY_STORE,
    },
    state::weekly_overview::WeeklySummary,
};

pub enum WeeklyOverviewPageAction {
    NextYear,
    PreviousYear,
}

fn diff_color_and_sign(diff: f32) -> (&'static str, &'static str) {
    if diff > 0.0 {
        ("text-good", "+")
    } else if diff < 0.0 {
        ("text-warn", "-")
    } else {
        ("text-ink", "")
    }
}

fn row_class(is_current: bool) -> &'static str {
    if is_current {
        "content-center bg-accent-soft"
    } else {
        "content-center"
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct WeeklyOverviewTableProps {
    pub weeks: Rc<[WeeklySummary]>,
    pub current_year: u32,
    pub current_week: u8,
    pub i18n: I18nType,
}

#[component]
pub fn WeeklyOverviewTable(props: WeeklyOverviewTableProps) -> Element {
    let WeeklyOverviewTableProps {
        weeks,
        current_year,
        current_week,
        i18n,
    } = props;

    let week_label = i18n.t(Key::WeekLabel);
    let paid_volunteer = i18n.t(Key::PaidVolunteer);
    let available_required_hours = i18n.t(Key::AvailableRequiredHours);
    let missing_hours = i18n.t(Key::MissingHours);
    let hours_short = i18n.t(Key::HoursShort);

    rsx! {
        section { class: "rounded-md border border-border bg-surface overflow-hidden",
            table { class: "w-full text-body",
                thead { class: "bg-surface-alt text-ink-muted text-left",
                    tr {
                        th { class: "px-3 py-2 text-micro font-bold uppercase",
                            "{week_label}"
                        }
                        th { class: "px-3 py-2 text-micro font-bold uppercase hidden md:table-cell",
                            "{paid_volunteer}"
                        }
                        th { class: "px-3 py-2 text-micro font-bold uppercase",
                            "{available_required_hours}"
                        }
                        th { class: "px-3 py-2 text-micro font-bold uppercase",
                            "{missing_hours}"
                        }
                    }
                }
                tbody { class: "divide-y divide-border",
                    for week in weeks.iter() {
                        {
                            let is_current_row = week.year == current_year && week.week == current_week;
                            let row_cls = row_class(is_current_row);
                            let diff = week.available_hours - week.required_hours;
                            let (diff_class, sign) = diff_color_and_sign(diff);
                            let diff_abs = diff.abs();
                            rsx! {
                                tr { class: "{row_cls}",
                                    td { class: "px-3 py-2",
                                        a { href: "/shiftplan/{week.year}/{week.week}",
                                            div { class: "font-semibold text-ink",
                                                "{week.year} / {week.week}"
                                            }
                                            div { class: "text-ink-muted text-small font-normal",
                                                "{i18n.format_date(&week.monday_date())} - {i18n.format_date(&week.sunday_date())}"
                                            }
                                        }
                                    }
                                    td { class: "hidden md:table-cell px-3 py-2 text-ink",
                                        {format!("💰{} | 🤝{}", format_hours(week.paid_hours, 2), format_hours(week.volunteer_hours, 2))}
                                    }
                                    td { class: "px-3 py-2 text-ink font-mono tabular-nums",
                                        div { {format!("{} / {}", format_hours(week.available_hours, 2), format_hours(week.required_hours, 2))} }
                                        div { class: "text-small font-normal text-ink-muted block md:hidden mt-1",
                                            {format!("💰{} | 🤝{}", format_hours(week.paid_hours, 2), format_hours(week.volunteer_hours, 2))}
                                        }
                                    }
                                    td { class: "px-3 py-2 {diff_class} font-mono tabular-nums",
                                        {format!("{sign} {}", format_hours(diff_abs, 2))}
                                    }
                                }
                                if !week.sales_person_absences.is_empty() {
                                    tr {
                                        td { class: "px-3 py-2 text-small font-normal text-ink-muted", colspan: "4",
                                            for absence in week.sales_person_absences.iter() {
                                                span { class: "mr-3",
                                                    {format!("{}: {} {hours_short}", absence.name, format_hours(absence.absence_hours, 2))}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn WeeklyOverview() -> Element {
    let year = use_signal(js::get_current_year);
    let weekly_overview_service = use_coroutine_handle::<WeeklySummaryAction>();
    let weekly_summary = WEEKLY_SUMMARY_STORE.read().clone();
    let i18n = I18N.read().clone();

    let title = i18n.t(Key::WeeklyOverviewTitle);
    let prev_year_label = i18n.t(Key::PreviousYear);
    let next_year_label = i18n.t(Key::NextYear);

    let current_year = js::get_current_year();
    let current_week = js::get_current_week();

    let cr = use_coroutine({
        to_owned![year];

        let load_data = move || async move {
            weekly_overview_service.send(WeeklySummaryAction::LoadYear(*year.read()))
        };

        move |mut rx: UnboundedReceiver<WeeklyOverviewPageAction>| async move {
            load_data().await;
            while let Some(action) = rx.next().await {
                match action {
                    WeeklyOverviewPageAction::NextYear => {
                        *year.write() += 1;
                        load_data().await;
                    }
                    WeeklyOverviewPageAction::PreviousYear => {
                        *year.write() -= 1;
                        load_data().await;
                    }
                }
            }
        }
    });

    rsx! {
        TopBar {}
        main { class: "mx-auto max-w-5xl w-full px-4 py-6 md:py-8 space-y-4",
            h1 { class: "text-h1 text-ink", "{title}" }
            if weekly_summary.data_loaded {
                div { class: "flex items-center gap-3 print:hidden",
                    NavBtn {
                        glyph: ImStr::from("‹"),
                        aria_label: Some(ImStr::from(prev_year_label.clone())),
                        on_click: Some(EventHandler::new(move |_| cr.send(WeeklyOverviewPageAction::PreviousYear))),
                    }
                    span { class: "font-mono text-lg text-ink min-w-[4ch] text-center", "{year.read()}" }
                    NavBtn {
                        glyph: ImStr::from("›"),
                        aria_label: Some(ImStr::from(next_year_label.clone())),
                        on_click: Some(EventHandler::new(move |_| cr.send(WeeklyOverviewPageAction::NextYear))),
                    }
                }
                section { class: "rounded-md border border-border bg-surface p-4 md:p-[18px]",
                    WeeklyOverviewChart {
                        weeks: weekly_summary.weekly_summary.clone(),
                        current_year,
                        current_week,
                    }
                }
                WeeklyOverviewTable {
                    weeks: weekly_summary.weekly_summary.clone(),
                    current_year,
                    current_week,
                    i18n: i18n.clone(),
                }
            } else {
                div { class: "text-ink-muted px-4 py-3", "Loading data..." }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{generate, Locale};
    use crate::state::weekly_overview::SalesPersonAbsence;
    use std::sync::Arc;

    fn sample_week(year: u32, week: u8, paid: f32, volunteer: f32, required: f32) -> WeeklySummary {
        WeeklySummary {
            week,
            year,
            available_hours: paid + volunteer,
            required_hours: required,
            paid_hours: paid,
            volunteer_hours: volunteer,
            monday_available_hours: 0.0,
            tuesday_available_hours: 0.0,
            wednesday_available_hours: 0.0,
            thursday_available_hours: 0.0,
            friday_available_hours: 0.0,
            saturday_available_hours: 0.0,
            sunday_available_hours: 0.0,
            sales_person_absences: vec![],
        }
    }

    fn render_table(props: WeeklyOverviewTableProps) -> String {
        fn app(p: WeeklyOverviewTableProps) -> Element {
            rsx! { WeeklyOverviewTable { ..p } }
        }
        let mut vdom = VirtualDom::new_with_props(app, props);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn diff_color_and_sign_returns_good_for_surplus() {
        let (cls, sign) = diff_color_and_sign(3.0);
        assert_eq!(cls, "text-good");
        assert_eq!(sign, "+");
    }

    #[test]
    fn diff_color_and_sign_returns_warn_for_deficit() {
        let (cls, sign) = diff_color_and_sign(-5.0);
        assert_eq!(cls, "text-warn");
        assert_eq!(sign, "-");
    }

    #[test]
    fn diff_color_and_sign_returns_warn_for_large_deficit() {
        // Even a large deficit stays warn — no text-bad tier.
        let (cls, sign) = diff_color_and_sign(-25.0);
        assert_eq!(cls, "text-warn");
        assert_eq!(sign, "-");
    }

    #[test]
    fn diff_color_and_sign_returns_ink_for_zero() {
        let (cls, sign) = diff_color_and_sign(0.0);
        assert_eq!(cls, "text-ink");
        assert_eq!(sign, "");
    }

    #[test]
    fn row_class_returns_accent_soft_for_current() {
        assert_eq!(row_class(true), "content-center bg-accent-soft");
        assert_eq!(row_class(false), "content-center");
    }

    fn build_full_year(year: u32) -> Rc<[WeeklySummary]> {
        (1..=52u8)
            .map(|w| sample_week(year, w, 20.0, 5.0, 30.0))
            .collect::<Vec<_>>()
            .into()
    }

    #[test]
    fn page_table_renders_all_provided_weeks() {
        let weeks = build_full_year(2026);
        let html = render_table(WeeklyOverviewTableProps {
            weeks,
            current_year: 2026,
            current_week: 27,
            i18n: generate(Locale::En),
        });
        // All 52 weeks are rendered — no window filter.
        for w in 1..=52u8 {
            let needle = format!("/shiftplan/2026/{w}");
            assert!(html.contains(&needle), "expected week {w} to render");
        }
    }

    #[test]
    fn page_table_different_year_shows_full_year() {
        let weeks = build_full_year(2024);
        let html = render_table(WeeklyOverviewTableProps {
            weeks,
            current_year: 2026,
            current_week: 27,
            i18n: generate(Locale::En),
        });
        // Full year still renders even if displayed year != current year.
        for w in 1..=52u8 {
            assert!(
                html.contains(&format!("/shiftplan/2024/{w}")),
                "expected week {w}"
            );
        }
        // No accent-soft tinting since displayed year != current year.
        assert!(
            !html.contains("bg-accent-soft"),
            "no tint when year mismatched: {html}"
        );
    }

    #[test]
    fn page_current_row_has_accent_soft_tint() {
        let weeks = build_full_year(2026);
        let html = render_table(WeeklyOverviewTableProps {
            weeks,
            current_year: 2026,
            current_week: 27,
            i18n: generate(Locale::En),
        });
        // The current week row contains both the accent-soft class AND week 27's link.
        // Find the segment for week 27 and check its row class.
        let week_27_marker = "/shiftplan/2026/27";
        let pos = html.find(week_27_marker).expect("week 27 should render");
        // Walk back to the preceding <tr ...> tag for week 27.
        let preceding = &html[..pos];
        let last_tr = preceding
            .rfind("<tr")
            .expect("expected a <tr before week 27 link");
        let tr_segment = &html[last_tr..pos];
        assert!(
            tr_segment.contains("bg-accent-soft"),
            "current row should have bg-accent-soft, got tr segment: {tr_segment}"
        );

        // Other rows (e.g. 25, 30) should NOT have the tint.
        for w in [25u8, 30] {
            let m = format!("/shiftplan/2026/{w}");
            let p = html
                .find(&m)
                .unwrap_or_else(|| panic!("week {w} should render"));
            let pre = &html[..p];
            let lt = pre.rfind("<tr").unwrap();
            let seg = &html[lt..p];
            assert!(
                !seg.contains("bg-accent-soft"),
                "non-current row {w} should NOT have bg-accent-soft, got: {seg}"
            );
        }
    }

    #[test]
    fn page_diff_column_two_tier_colors() {
        // Build weeks with different available/required combos to hit each tier.
        // Use weeks within the window for current_week=3 -> [1,8]
        let weeks: Rc<[WeeklySummary]> = vec![
            // week 1: surplus +3
            sample_week(2026, 1, 33.0, 0.0, 30.0),
            // week 2: deficit -5
            sample_week(2026, 2, 25.0, 0.0, 30.0),
            // week 3: large deficit -25
            sample_week(2026, 3, 5.0, 0.0, 30.0),
            // week 4: zero diff
            sample_week(2026, 4, 30.0, 0.0, 30.0),
        ]
        .into();
        let html = render_table(WeeklyOverviewTableProps {
            weeks,
            current_year: 2026,
            current_week: 3,
            i18n: generate(Locale::En),
        });
        assert!(html.contains("text-good"), "expected text-good: {html}");
        assert!(html.contains("text-warn"), "expected text-warn: {html}");
        assert!(
            !html.contains("text-bad"),
            "should NOT contain text-bad (legacy tier removed): {html}"
        );
        assert!(
            !html.contains("text-yellow-700"),
            "should NOT contain text-yellow-700: {html}"
        );
        assert!(
            !html.contains("text-red-500"),
            "should NOT contain text-red-500: {html}"
        );
        assert!(
            !html.contains("text-green-500"),
            "should NOT contain text-green-500: {html}"
        );
        // Sign + abs format
        assert!(html.contains("+ 3.00"), "expected `+ 3.00`: {html}");
        assert!(html.contains("- 5.00"), "expected `- 5.00`: {html}");
        assert!(html.contains("- 25.00"), "expected `- 25.00`: {html}");
    }

    #[test]
    fn page_absences_row_uses_tokens_and_no_tint() {
        let mut current = sample_week(2026, 27, 25.0, 5.0, 30.0);
        current.sales_person_absences = vec![SalesPersonAbsence {
            name: Arc::<str>::from("Lena"),
            absence_hours: 8.0,
        }];
        let weeks: Rc<[WeeklySummary]> = vec![current].into();
        let html = render_table(WeeklyOverviewTableProps {
            weeks,
            current_year: 2026,
            current_week: 27,
            i18n: generate(Locale::En),
        });
        // Absences td uses token classes
        assert!(
            html.contains("text-ink-muted"),
            "absences should use text-ink-muted: {html}"
        );
        assert!(
            !html.contains("text-gray-600"),
            "should not use legacy gray: {html}"
        );
        assert!(html.contains("Lena"), "absence name should appear: {html}");

        // Find the absences <tr> (the one without a Link to /shiftplan).
        // The current row has bg-accent-soft, but the absences tr should NOT.
        let absences_marker = "Lena: 8.00";
        let p = html.find(absences_marker).expect("absence text not found");
        let pre = &html[..p];
        let last_tr = pre.rfind("<tr").unwrap();
        let absences_tr = &html[last_tr..p];
        assert!(
            !absences_tr.contains("bg-accent-soft"),
            "absences row should NOT have tint: {absences_tr}"
        );
    }

    #[test]
    fn page_uses_navbtn_for_year_nav() {
        // Render the standalone NavBtn pair (the year-nav row uses these atoms)
        // with German aria labels to verify the wiring matches the spec.
        use crate::component::NavBtn;
        let prev_label = generate(Locale::De).t(Key::PreviousYear);
        let next_label = generate(Locale::De).t(Key::NextYear);

        #[derive(Props, Clone, PartialEq)]
        struct NavRowProps {
            prev: ImStr,
            next: ImStr,
        }

        fn nav_row(p: NavRowProps) -> Element {
            rsx! {
                div { class: "flex items-center gap-3 print:hidden",
                    NavBtn {
                        glyph: ImStr::from("‹"),
                        aria_label: Some(p.prev.clone()),
                    }
                    span { class: "font-mono text-lg text-ink", "2026" }
                    NavBtn {
                        glyph: ImStr::from("›"),
                        aria_label: Some(p.next.clone()),
                    }
                }
            }
        }

        let mut vdom = VirtualDom::new_with_props(
            nav_row,
            NavRowProps {
                prev: ImStr::from(prev_label.clone()),
                next: ImStr::from(next_label.clone()),
            },
        );
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);

        // No legacy thick-bordered button
        assert!(
            !html.contains("border-2 border-solid border-black"),
            "should not contain legacy button: {html}"
        );
        // print:hidden on wrapper
        assert!(
            html.contains("print:hidden"),
            "expected print:hidden: {html}"
        );
        // NavBtn classes (from atom build_class)
        assert!(
            html.contains("border-border-strong"),
            "expected NavBtn token border: {html}"
        );
        assert!(html.contains("font-mono"), "expected font-mono: {html}");
        // Aria labels (German)
        assert!(
            html.contains("Vorheriges Jahr"),
            "expected German prev aria-label: {html}"
        );
        assert!(
            html.contains("Nächstes Jahr"),
            "expected German next aria-label: {html}"
        );
    }

    #[test]
    fn page_table_uses_token_classes() {
        let weeks = build_full_year(2026);
        let html = render_table(WeeklyOverviewTableProps {
            weeks,
            current_year: 2026,
            current_week: 27,
            i18n: generate(Locale::En),
        });
        assert!(html.contains("bg-surface"), "expected bg-surface: {html}");
        assert!(
            html.contains("border-border"),
            "expected border-border: {html}"
        );
        assert!(
            html.contains("divide-y divide-border"),
            "expected divide tokens: {html}"
        );
        // Legacy classes must not appear
        for legacy in [
            "bg-gray-100",
            "bg-white",
            "text-gray-500",
            "text-gray-600",
            "text-green-500",
            "text-red-500",
            "text-yellow-700",
            "border-black",
        ] {
            assert!(
                !html.contains(legacy),
                "legacy class `{legacy}` found: {html}"
            );
        }
    }

    #[test]
    fn page_source_does_not_use_legacy_classes() {
        // Inspect the non-test source of weekly_overview.rs and chart component.
        let page_source = include_str!("weekly_overview.rs");
        // Strip the test module to avoid matching test assertions that *check for absence*.
        let cut = page_source
            .find("#[cfg(test)]")
            .expect("test module should be present");
        let prod_source = &page_source[..cut];
        for legacy in [
            "bg-gray-100",
            "bg-white",
            "text-gray-500",
            "text-gray-600",
            "text-green-500",
            "text-red-500",
            "text-yellow-700",
            "border-black",
            "border-2 border-solid",
        ] {
            assert!(
                !prod_source.contains(legacy),
                "legacy class `{legacy}` found in production source"
            );
        }

        let chart_source = include_str!("../component/weekly_overview_chart.rs");
        let chart_cut = chart_source
            .find("#[cfg(test)]")
            .expect("chart test module should be present");
        let chart_prod = &chart_source[..chart_cut];
        for hex in [
            "#3B82F6", "#10B981", "#EF4444", "#e5e7eb", "#6b7280", "#374151",
        ] {
            assert!(
                !chart_prod.contains(hex),
                "hardcoded hex `{hex}` found in chart production source"
            );
        }
    }
}
