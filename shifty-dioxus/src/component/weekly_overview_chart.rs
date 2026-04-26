use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    base_types::format_hours, i18n::Key, service::i18n::I18N, state::weekly_overview::WeeklySummary,
};

const CHART_HEIGHT_PX: u32 = 160;
const NON_CURRENT_BAR_COLOR: &str = "#7787e8"; // Designed dimmer accent (per reference)

fn compute_max_hours(weeks: &[WeeklySummary]) -> f32 {
    let max_val = weeks
        .iter()
        .map(|w| {
            let bar_total = w.paid_hours + w.volunteer_hours;
            if bar_total > w.required_hours {
                bar_total
            } else {
                w.required_hours
            }
        })
        .fold(0.0_f32, f32::max);
    if max_val < 1.0 {
        10.0
    } else {
        max_val
    }
}

fn axis_label_weeks() -> [u8; 5] {
    [1, 13, 26, 39, 52]
}

#[component]
pub fn WeeklyOverviewChart(
    weeks: Rc<[WeeklySummary]>,
    current_year: u32,
    current_week: u8,
) -> Element {
    let i18n = I18N.read().clone();
    rsx! {
        WeeklyOverviewChartView {
            weeks,
            current_year,
            current_week,
            paid_label: i18n.t(Key::Paid).to_string(),
            volunteer_label: i18n.t(Key::Volunteer).to_string(),
            required_label: i18n.t(Key::ChartRequiredHours).to_string(),
            week_short: i18n.t(Key::WeekShort).to_string(),
        }
    }
}

#[component]
pub(crate) fn WeeklyOverviewChartView(
    weeks: Rc<[WeeklySummary]>,
    current_year: u32,
    current_week: u8,
    paid_label: String,
    volunteer_label: String,
    required_label: String,
    week_short: String,
) -> Element {
    if weeks.is_empty() {
        return rsx! {};
    }

    let max_hours = compute_max_hours(&weeks);
    let any_current_in_view = weeks
        .iter()
        .any(|w| w.year == current_year && w.week == current_week);

    rsx! {
        div {
            // HTML legend
            div { class: "flex items-center gap-4 mb-3 text-xs text-ink",
                span { class: "inline-flex items-center gap-1.5",
                    span { style: "background: var(--accent); width: 12px; height: 12px; border-radius: 2px; display: inline-block;" }
                    "{paid_label}"
                }
                span { class: "inline-flex items-center gap-1.5",
                    span { style: "background: var(--ink-muted); opacity: 0.4; width: 12px; height: 12px; border-radius: 2px; display: inline-block;" }
                    "{volunteer_label}"
                }
                span { class: "inline-flex items-center gap-1.5",
                    span { style: "background: var(--bad); width: 12px; height: 2px; display: inline-block;" }
                    "{required_label}"
                }
            }

            // Bars: HTML flex container of column-bars
            div {
                class: "relative",
                style: "display: flex; align-items: flex-end; gap: 1.5px; height: {CHART_HEIGHT_PX}px;",

                for week in weeks.iter() {
                    {
                        let paid_pct = (week.paid_hours / max_hours) * 100.0;
                        let vol_pct = (week.volunteer_hours / max_hours) * 100.0;
                        let req_top_pct = (1.0 - week.required_hours / max_hours) * 100.0;
                        let is_current = week.year == current_year && week.week == current_week;
                        let bar_opacity = if any_current_in_view && !is_current { 0.85 } else { 1.0 };
                        let paid_bg = if is_current { "var(--accent)".to_string() } else { NON_CURRENT_BAR_COLOR.to_string() };
                        let nav_url = format!("/shiftplan/{}/{}", week.year, week.week);
                        let tooltip = format!(
                            "{week_short} {}: {paid_label} {}h, {volunteer_label} {}h, {required_label} {}h",
                            week.week,
                            format_hours(week.paid_hours, 1),
                            format_hours(week.volunteer_hours, 1),
                            format_hours(week.required_hours, 1)
                        );

                        rsx! {
                            div {
                                title: "{tooltip}",
                                style: "flex: 1; height: 100%; position: relative; display: flex; flex-direction: column; justify-content: flex-end; opacity: {bar_opacity}; cursor: pointer; min-width: 4px;",
                                onclick: move |_| {
                                    navigator().push(nav_url.clone());
                                },
                                // Volunteer (top portion of stack)
                                if week.volunteer_hours > 0.0 {
                                    div { style: "height: {vol_pct}%; background: var(--ink-muted); opacity: 0.35;" }
                                }
                                // Paid (bottom portion)
                                if week.paid_hours > 0.0 {
                                    div { style: "height: {paid_pct}%; background: {paid_bg};" }
                                }
                                // Required line: absolute-positioned dashed top border
                                div { style: "position: absolute; left: 0; right: 0; top: {req_top_pct}%; border-top: 1.5px dashed var(--bad); pointer-events: none;" }
                            }
                        }
                    }
                }
            }

            // X-axis: 5 evenly-spaced mono labels
            div {
                class: "font-mono",
                style: "display: flex; justify-content: space-between; margin-top: 6px; font-size: 10px; color: var(--ink-muted);",
                for label_week in axis_label_weeks().iter() {
                    span { "{week_short} {label_week}" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn compute_max_hours_uses_larger_of_bar_or_required() {
        let weeks = vec![
            sample_week(2026, 1, 20.0, 10.0, 35.0),
            sample_week(2026, 2, 30.0, 12.0, 25.0),
        ];
        // Week 2 has bar_total = 42 > all required values
        assert_eq!(compute_max_hours(&weeks), 42.0);
    }

    #[test]
    fn compute_max_hours_uses_required_when_larger() {
        let weeks = vec![sample_week(2026, 1, 5.0, 0.0, 30.0)];
        assert_eq!(compute_max_hours(&weeks), 30.0);
    }

    #[test]
    fn compute_max_hours_floors_at_ten() {
        assert_eq!(compute_max_hours(&[]), 10.0);
        let weeks = vec![sample_week(2026, 1, 0.0, 0.0, 0.0)];
        assert_eq!(compute_max_hours(&weeks), 10.0);
    }

    #[test]
    fn axis_label_weeks_are_five_evenly_spaced() {
        assert_eq!(axis_label_weeks(), [1, 13, 26, 39, 52]);
    }

    #[derive(Props, Clone, PartialEq)]
    struct ViewProps {
        weeks: Rc<[WeeklySummary]>,
        current_year: u32,
        current_week: u8,
        paid_label: String,
        volunteer_label: String,
        required_label: String,
        week_short: String,
    }

    fn render_view(
        weeks: Rc<[WeeklySummary]>,
        current_year: u32,
        current_week: u8,
        week_short: &str,
    ) -> String {
        fn app(p: ViewProps) -> Element {
            rsx! {
                WeeklyOverviewChartView {
                    weeks: p.weeks.clone(),
                    current_year: p.current_year,
                    current_week: p.current_week,
                    paid_label: p.paid_label.clone(),
                    volunteer_label: p.volunteer_label.clone(),
                    required_label: p.required_label.clone(),
                    week_short: p.week_short.clone(),
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(
            app,
            ViewProps {
                weeks,
                current_year,
                current_week,
                paid_label: "Paid".to_string(),
                volunteer_label: "Volunteer".to_string(),
                required_label: "Required Hours".to_string(),
                week_short: week_short.to_string(),
            },
        );
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn chart_uses_token_styles_not_legacy_hex() {
        let weeks: Rc<[WeeklySummary]> = vec![
            sample_week(2026, 1, 20.0, 10.0, 35.0),
            sample_week(2026, 2, 30.0, 12.0, 25.0),
        ]
        .into();
        let html = render_view(weeks, 2026, 12, "W");
        assert!(
            html.contains("var(--accent)"),
            "expected accent token: {html}"
        );
        assert!(
            html.contains("var(--ink-muted)"),
            "expected ink-muted token: {html}"
        );
        assert!(html.contains("var(--bad)"), "expected bad token: {html}");
        // Legacy chart hex must not appear.
        for hex in ["#3B82F6", "#10B981", "#EF4444", "#e5e7eb", "#6b7280"] {
            assert!(!html.contains(hex), "found legacy hex `{hex}` in: {html}");
        }
    }

    #[test]
    fn chart_volunteer_uses_ink_muted_not_good() {
        // The reference uses ink-muted with opacity for volunteer; var(--good) (green) was wrong.
        let weeks: Rc<[WeeklySummary]> = vec![sample_week(2026, 1, 20.0, 10.0, 35.0)].into();
        let html = render_view(weeks, 2026, 1, "W");
        // The volunteer bar div should reference var(--ink-muted) with opacity 0.35.
        assert!(
            html.contains("background: var(--ink-muted); opacity: 0.35"),
            "expected volunteer ink-muted with opacity 0.35: {html}"
        );
        assert!(
            !html.contains("background: var(--good)"),
            "should not use --good for volunteer: {html}"
        );
    }

    #[test]
    fn chart_required_line_uses_bad_token_dashed() {
        let weeks: Rc<[WeeklySummary]> = vec![sample_week(2026, 1, 20.0, 10.0, 35.0)].into();
        let html = render_view(weeks, 2026, 1, "W");
        // Per-bar required line: dashed border-top in --bad
        assert!(
            html.contains("border-top: 1.5px dashed var(--bad)"),
            "expected dashed bad-colored required line: {html}"
        );
    }

    #[test]
    fn chart_current_week_full_opacity_others_dimmed() {
        let weeks: Rc<[WeeklySummary]> = vec![
            sample_week(2026, 16, 20.0, 5.0, 30.0),
            sample_week(2026, 17, 25.0, 5.0, 30.0),
            sample_week(2026, 18, 15.0, 5.0, 30.0),
        ]
        .into();
        let html = render_view(weeks, 2026, 17, "W");
        // Two bars dimmed (16, 18), one at full opacity (17).
        let dim_count = html.matches("opacity: 0.85").count();
        assert_eq!(dim_count, 2, "expected 2 dimmed bars: {html}");
        let full_opacity_count = html.matches("opacity: 1").count();
        // Each bar emits exactly one outer wrapper opacity. The current bar is "opacity: 1".
        assert!(
            full_opacity_count >= 1,
            "expected at least 1 full-opacity bar: {html}"
        );
    }

    #[test]
    fn chart_current_week_uses_accent_others_dimmed_color() {
        let weeks: Rc<[WeeklySummary]> = vec![
            sample_week(2026, 16, 20.0, 5.0, 30.0),
            sample_week(2026, 17, 25.0, 5.0, 30.0),
        ]
        .into();
        let html = render_view(weeks, 2026, 17, "W");
        assert!(
            html.contains("background: var(--accent)"),
            "current week paid bar should use accent: {html}"
        );
        assert!(
            html.contains("#7787e8"),
            "non-current bars should use designed dimmer accent: {html}"
        );
    }

    #[test]
    fn chart_no_dim_when_year_does_not_match() {
        let weeks: Rc<[WeeklySummary]> = vec![
            sample_week(2025, 16, 20.0, 5.0, 30.0),
            sample_week(2025, 17, 25.0, 5.0, 30.0),
        ]
        .into();
        let html = render_view(weeks, 2026, 17, "W");
        let dim_count = html.matches("opacity: 0.85").count();
        assert_eq!(dim_count, 0, "no bars dimmed when year mismatched: {html}");
        // No accent — all bars get the dimmer color.
        assert!(
            html.contains("#7787e8"),
            "expected dimmer color when no current bar: {html}"
        );
    }

    #[test]
    fn chart_x_axis_renders_five_locale_labels() {
        let weeks_vec: Vec<WeeklySummary> = (1..=52u8)
            .map(|w| sample_week(2026, w, 10.0, 5.0, 20.0))
            .collect();
        let weeks: Rc<[WeeklySummary]> = weeks_vec.into();
        let html = render_view(weeks, 2026, 27, "KW");
        for w in [1u8, 13, 26, 39, 52] {
            assert!(
                html.contains(&format!("KW {w}")),
                "expected axis label `KW {w}`: missing"
            );
        }
    }

    #[test]
    fn i18n_week_short_returns_locale_value() {
        use crate::i18n::{generate, Key as I18nKey, Locale};
        let en = generate(Locale::En);
        let de = generate(Locale::De);
        let cs = generate(Locale::Cs);
        assert_eq!(en.t(I18nKey::WeekShort).as_ref(), "W");
        assert_eq!(de.t(I18nKey::WeekShort).as_ref(), "KW");
        assert_eq!(cs.t(I18nKey::WeekShort).as_ref(), "T");
        for i18n in [en, de, cs] {
            assert!(!i18n.t(I18nKey::PreviousYear).is_empty());
            assert!(!i18n.t(I18nKey::NextYear).is_empty());
        }
    }
}
