//! `EmployeeWeeklyHistogram` — an SVG histogram of working hours per ISO
//! week, scaled to fill the available width regardless of how many weeks
//! are passed in (typically a full year).
//!
//! Each bar's height encodes the week's `overall_hours`. A dashed reference
//! line indicates the contract's `expected_hours_per_week`. Bars below the
//! line render in `var(--warn)`; bars at or above render in `var(--accent)`.
//! The current week is always X-axis-labeled. Clicking a bar emits the
//! `(year, week)` tuple via `on_select`.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::base_types::ImStr;
use crate::i18n::Key;
use crate::service::i18n::I18N;
use crate::state::employee::WorkingHours;

const SVG_WIDTH: f32 = 340.0;
const SVG_HEIGHT: f32 = 120.0;
const BAR_AREA_HEIGHT: f32 = 90.0;
const BAR_GAP: f32 = 1.0;

/// Returns the maximum Y for vertical scaling. Considers each week's own
/// `overall_hours` and `expected_hours`, with a 1.0 floor that prevents
/// divide-by-zero when every value is zero.
pub(crate) fn compute_max_y(weeks: &[WorkingHours]) -> f32 {
    weeks
        .iter()
        .flat_map(|w| [w.overall_hours, w.expected_hours])
        .fold(0.0f32, f32::max)
        .max(1.0)
}

/// Returns the Y coordinate for a given hours value, anchored so larger
/// values sit higher in the SVG.
pub(crate) fn bar_y(value: f32, max_y: f32) -> f32 {
    BAR_AREA_HEIGHT - (value / max_y) * BAR_AREA_HEIGHT
}

/// Returns the CSS variable token for a bar's fill color: `warn` when
/// below the expected line, `accent` otherwise.
pub(crate) fn bar_color_token(value: f32, expected: f32) -> &'static str {
    if value < expected {
        "var(--warn)"
    } else {
        "var(--accent)"
    }
}

/// Returns the (year, ISO week) pair for a `WorkingHours` entry, derived
/// from its `from` date.
fn week_year_week(week: &WorkingHours) -> (u32, u8) {
    let (iso_year, iso_week, _) = week.from.to_iso_week_date();
    (iso_year as u32, iso_week)
}

#[derive(Props, Clone, PartialEq)]
pub struct EmployeeWeeklyHistogramProps {
    pub weeks: Rc<[WorkingHours]>,
    pub current_year: u32,
    pub current_week: u8,
    #[props(!optional, default = None)]
    pub selected_week: Option<(u32, u8)>,
    pub on_select: EventHandler<(u32, u8)>,
}

#[component]
pub fn EmployeeWeeklyHistogram(props: EmployeeWeeklyHistogramProps) -> Element {
    let i18n = I18N.read().clone();
    let week_short: ImStr = ImStr::from(i18n.t(Key::WeekShort).as_ref());

    rsx! {
        EmployeeWeeklyHistogramView {
            weeks: props.weeks,
            current_year: props.current_year,
            current_week: props.current_week,
            selected_week: props.selected_week,
            on_select: props.on_select,
            week_short,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct EmployeeWeeklyHistogramViewProps {
    weeks: Rc<[WorkingHours]>,
    current_year: u32,
    current_week: u8,
    #[props(!optional, default = None)]
    selected_week: Option<(u32, u8)>,
    on_select: EventHandler<(u32, u8)>,
    week_short: ImStr,
}

#[component]
fn EmployeeWeeklyHistogramView(props: EmployeeWeeklyHistogramViewProps) -> Element {
    let weeks = props.weeks.clone();
    let max_y = compute_max_y(&weeks);
    let count = weeks.len() as f32;
    let bar_width = if count > 0.0 {
        ((SVG_WIDTH - BAR_GAP * (count + 1.0)) / count).max(1.0)
    } else {
        1.0
    };

    let any_positive_expected = weeks.iter().any(|w| w.expected_hours > 0.0);
    let polyline_points: Option<String> = if any_positive_expected {
        let slot_width = bar_width + BAR_GAP;
        let pts = weeks
            .iter()
            .enumerate()
            .map(|(i, week)| {
                let y = bar_y(week.expected_hours, max_y);
                let x_left = (i as f32) * slot_width;
                let x_right = ((i + 1) as f32) * slot_width;
                format!("{x_left},{y} {x_right},{y}")
            })
            .collect::<Vec<_>>()
            .join(" ");
        Some(pts)
    } else {
        None
    };

    let has_selection = props.selected_week.is_some();
    let week_short = props.week_short.clone();

    rsx! {
        svg {
            view_box: "0 0 {SVG_WIDTH} {SVG_HEIGHT}",
            preserve_aspect_ratio: "none",
            width: "100%",
            height: "120",
            // Stepped reference polyline of per-week expected_hours
            if let Some(points) = polyline_points {
                polyline {
                    points: "{points}",
                    stroke_dasharray: "4 3",
                    stroke_width: "1.5",
                    style: "stroke: var(--ink-muted); fill: none;",
                }
            }
            for (i, week) in weeks.iter().enumerate() {
                {
                    let (year, week_num) = week_year_week(week);
                    let is_selected = props.selected_week == Some((year, week_num));
                    let is_current = (year, week_num) == (props.current_year, props.current_week);
                    let x = BAR_GAP + (i as f32) * (bar_width + BAR_GAP);
                    let y = bar_y(week.overall_hours, max_y);
                    let height = (BAR_AREA_HEIGHT - y).max(0.0);
                    let color_token = bar_color_token(week.overall_hours, week.expected_hours);
                    let group_style = if has_selection && !is_selected {
                        String::from("opacity: 0.85; cursor: pointer;")
                    } else {
                        String::from("cursor: pointer;")
                    };
                    let rect_style = format!("fill: {color_token}");
                    let show_label =
                        (week_num as usize - 1) % 4 == 0 || week_num == 52 || is_current;
                    let label_x = x + bar_width / 2.0;
                    let label_text = format!("{} {}", week_short, week_num);
                    let on_select = props.on_select;
                    rsx! {
                        g {
                            key: "{i}",
                            style: "{group_style}",
                            onclick: move |_| on_select.call((year, week_num)),
                            rect {
                                x: "{x}",
                                y: "{y}",
                                width: "{bar_width}",
                                height: "{height}",
                                style: "{rect_style}",
                            }
                            if show_label {
                                text {
                                    x: "{label_x}",
                                    y: "{SVG_HEIGHT - 5.0}",
                                    text_anchor: "middle",
                                    font_family: "ui-monospace, SFMono-Regular, Menlo, monospace",
                                    font_size: "9",
                                    style: "fill: var(--ink-muted)",
                                    "{label_text}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use time::macros::date;

    fn make_week(from: time::Date, overall: f32, expected: f32) -> WorkingHours {
        WorkingHours {
            from,
            to: from,
            expected_hours: expected,
            overall_hours: overall,
            balance: 0.0,
            shiftplan_hours: 0.0,
            extra_work_hours: 0.0,
            vacation_hours: 0.0,
            vacation_days: 0.0,
            sick_leave_hours: 0.0,
            holiday_hours: 0.0,
            unpaid_leave_hours: 0.0,
            volunteer_hours: 0.0,
            days: Rc::from([]),
        }
    }

    #[test]
    fn compute_max_y_uses_largest_of_overall_or_per_week_expected() {
        let w = vec![
            make_week(date!(2026 - 03 - 02), 10.0, 20.0),
            make_week(date!(2026 - 03 - 09), 25.0, 20.0),
        ];
        assert_eq!(compute_max_y(&w), 25.0);

        let w2 = vec![make_week(date!(2026 - 03 - 02), 10.0, 30.0)];
        assert_eq!(compute_max_y(&w2), 30.0);
    }

    #[test]
    fn compute_max_y_floors_to_one_when_all_zero() {
        let w = vec![make_week(date!(2026 - 03 - 02), 0.0, 0.0)];
        assert_eq!(compute_max_y(&w), 1.0);
    }

    #[test]
    fn bar_y_proportional_to_value_over_max() {
        // value 0 → bottom (y == BAR_AREA_HEIGHT)
        assert!((bar_y(0.0, 30.0) - 90.0).abs() < 0.001);
        // value == max → top (y == 0)
        assert!(bar_y(30.0, 30.0).abs() < 0.001);
        // value == half max → middle
        assert!((bar_y(15.0, 30.0) - 45.0).abs() < 0.001);
    }

    #[test]
    fn bar_color_token_returns_warn_below_expected() {
        assert_eq!(bar_color_token(15.0, 20.0), "var(--warn)");
    }

    #[test]
    fn bar_color_token_returns_accent_at_or_above_expected() {
        assert_eq!(bar_color_token(20.0, 20.0), "var(--accent)");
        assert_eq!(bar_color_token(25.0, 20.0), "var(--accent)");
    }

    #[derive(Props, Clone, PartialEq)]
    struct ViewProps {
        weeks: Rc<[WorkingHours]>,
        current_year: u32,
        current_week: u8,
        selected: Option<(u32, u8)>,
        week_short: ImStr,
    }

    fn render_view(p: ViewProps) -> String {
        fn app(p: ViewProps) -> Element {
            rsx! {
                EmployeeWeeklyHistogramView {
                    weeks: p.weeks.clone(),
                    current_year: p.current_year,
                    current_week: p.current_week,
                    selected_week: p.selected,
                    on_select: |_| {},
                    week_short: p.week_short.clone(),
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(app, p);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn ssr_renders_one_rect_per_week() {
        let weeks: Rc<[WorkingHours]> = (1..=17u8)
            .map(|i| {
                make_week(
                    time::Date::from_iso_week_date(2026, i, time::Weekday::Monday).unwrap(),
                    10.0,
                    0.0,
                )
            })
            .collect();
        let html = render_view(ViewProps {
            weeks,
            current_year: 2026,
            current_week: 1,
            selected: None,
            week_short: ImStr::from("KW"),
        });
        let rect_count = html.matches("<rect").count();
        assert_eq!(
            rect_count, 17,
            "expected 17 rects, got {rect_count}: {html}"
        );
    }

    #[test]
    fn ssr_below_per_week_expected_uses_warn_token() {
        let weeks: Rc<[WorkingHours]> = vec![make_week(date!(2026 - 03 - 02), 15.0, 20.0)].into();
        let html = render_view(ViewProps {
            weeks,
            current_year: 2026,
            current_week: 10,
            selected: None,
            week_short: ImStr::from("KW"),
        });
        assert!(
            html.contains("fill: var(--warn)"),
            "expected warn token in: {html}"
        );
        assert!(
            !html.contains("fill: var(--accent)"),
            "should not use accent: {html}"
        );
    }

    #[test]
    fn ssr_at_or_above_per_week_expected_uses_accent_token() {
        let weeks: Rc<[WorkingHours]> = vec![make_week(date!(2026 - 03 - 02), 30.0, 20.0)].into();
        let html = render_view(ViewProps {
            weeks,
            current_year: 2026,
            current_week: 10,
            selected: None,
            week_short: ImStr::from("KW"),
        });
        assert!(
            html.contains("fill: var(--accent)"),
            "expected accent token in: {html}"
        );
        assert!(
            !html.contains("fill: var(--warn)"),
            "should not use warn: {html}"
        );
    }

    #[test]
    fn ssr_dashed_polyline_present_when_any_week_has_positive_expected() {
        let weeks: Rc<[WorkingHours]> = vec![make_week(date!(2026 - 03 - 02), 30.0, 20.0)].into();
        let html = render_view(ViewProps {
            weeks,
            current_year: 2026,
            current_week: 10,
            selected: None,
            week_short: ImStr::from("KW"),
        });
        assert!(html.contains("<polyline"), "expected polyline: {html}");
        assert!(
            html.contains("stroke-dasharray=\"4 3\""),
            "expected dashed line: {html}"
        );
        assert!(
            html.contains("stroke: var(--ink-muted)"),
            "expected ink-muted stroke: {html}"
        );
        assert!(
            html.contains("fill: none"),
            "polyline must not be filled: {html}"
        );
    }

    #[test]
    fn ssr_polyline_steps_at_contract_change() {
        // Weeks 1-2 expected = 20h, weeks 3-4 expected = 30h.
        let weeks: Rc<[WorkingHours]> = (1..=4u8)
            .map(|i| {
                let expected = if i <= 2 { 20.0 } else { 30.0 };
                make_week(
                    time::Date::from_iso_week_date(2026, i, time::Weekday::Monday).unwrap(),
                    0.0,
                    expected,
                )
            })
            .collect();
        let max_y = compute_max_y(&weeks);
        let html = render_view(ViewProps {
            weeks: weeks.clone(),
            current_year: 2026,
            current_week: 1,
            selected: None,
            week_short: ImStr::from("KW"),
        });
        assert!(html.contains("<polyline"), "expected polyline: {html}");
        let y_20 = bar_y(20.0, max_y);
        let y_30 = bar_y(30.0, max_y);
        // y_20 and y_30 must differ for this to be a real step.
        assert!(
            (y_20 - y_30).abs() > f32::EPSILON,
            "y values must differ for step"
        );
        let needle_20 = format!(",{y_20} ");
        let needle_30 = format!(",{y_30} ");
        assert!(
            html.contains(&needle_20),
            "polyline must contain a y at 20h ({y_20}): {html}"
        );
        assert!(
            html.contains(&needle_30),
            "polyline must contain a y at 30h ({y_30}): {html}"
        );
    }

    #[test]
    fn ssr_polyline_drops_to_baseline_on_zero_expected_week() {
        // Three weeks: expected = 20, 0, 20.
        let weeks: Rc<[WorkingHours]> = vec![
            make_week(
                time::Date::from_iso_week_date(2026, 1, time::Weekday::Monday).unwrap(),
                0.0,
                20.0,
            ),
            make_week(
                time::Date::from_iso_week_date(2026, 2, time::Weekday::Monday).unwrap(),
                0.0,
                0.0,
            ),
            make_week(
                time::Date::from_iso_week_date(2026, 3, time::Weekday::Monday).unwrap(),
                0.0,
                20.0,
            ),
        ]
        .into();
        let max_y = compute_max_y(&weeks);
        let html = render_view(ViewProps {
            weeks: weeks.clone(),
            current_year: 2026,
            current_week: 1,
            selected: None,
            week_short: ImStr::from("KW"),
        });
        let baseline_y = bar_y(0.0, max_y);
        // Polyline must reach the chart baseline for the zero-expected week.
        assert!(html.contains("<polyline"), "expected polyline: {html}");
        let needle = format!(",{baseline_y} ");
        assert!(
            html.contains(&needle) || html.contains(&format!(",{baseline_y}\"")),
            "polyline must reach baseline ({baseline_y}) for zero week: {html}"
        );
    }

    #[test]
    fn ssr_per_week_expected_drives_color_independently() {
        // Two weeks, both with overall = 22.
        // Week A expected = 20 → accent (22 >= 20)
        // Week B expected = 30 → warn (22 < 30)
        let weeks: Rc<[WorkingHours]> = vec![
            make_week(
                time::Date::from_iso_week_date(2026, 1, time::Weekday::Monday).unwrap(),
                22.0,
                20.0,
            ),
            make_week(
                time::Date::from_iso_week_date(2026, 2, time::Weekday::Monday).unwrap(),
                22.0,
                30.0,
            ),
        ]
        .into();
        let html = render_view(ViewProps {
            weeks,
            current_year: 2026,
            current_week: 1,
            selected: None,
            week_short: ImStr::from("KW"),
        });
        assert!(
            html.contains("fill: var(--accent)"),
            "expected accent token (week 1, 22 >= 20): {html}"
        );
        assert!(
            html.contains("fill: var(--warn)"),
            "expected warn token (week 2, 22 < 30): {html}"
        );
    }

    #[test]
    fn ssr_no_polyline_when_all_weeks_have_zero_expected() {
        let weeks: Rc<[WorkingHours]> = (1..=4u8)
            .map(|i| {
                make_week(
                    time::Date::from_iso_week_date(2026, i, time::Weekday::Monday).unwrap(),
                    10.0,
                    0.0,
                )
            })
            .collect();
        let html = render_view(ViewProps {
            weeks,
            current_year: 2026,
            current_week: 1,
            selected: None,
            week_short: ImStr::from("KW"),
        });
        assert!(
            !html.contains("<polyline"),
            "polyline must be omitted when no week has positive expected: {html}"
        );
    }

    #[test]
    fn ssr_selected_bar_full_opacity_others_dimmed() {
        // Two weeks, one selected — non-selected should carry opacity 0.85.
        let week1 = make_week(
            time::Date::from_iso_week_date(2026, 17, time::Weekday::Monday).unwrap(),
            10.0,
            0.0,
        );
        let week2 = make_week(
            time::Date::from_iso_week_date(2026, 18, time::Weekday::Monday).unwrap(),
            10.0,
            0.0,
        );
        let weeks: Rc<[WorkingHours]> = vec![week1, week2].into();
        let html = render_view(ViewProps {
            weeks,
            current_year: 2026,
            current_week: 17,
            selected: Some((2026, 17)),
            week_short: ImStr::from("KW"),
        });
        // At least one bar carries opacity 0.85 (the non-selected week 18).
        assert!(
            html.contains("opacity: 0.85"),
            "expected opacity dimming: {html}"
        );
        // The selected bar's group should not carry opacity 0.85 — verify
        // there is at least one group without it.
        let dimmed = html.matches("opacity: 0.85").count();
        let groups = html.matches("<g ").count();
        assert!(dimmed < groups, "all groups dimmed: {html}");
    }

    #[test]
    fn ssr_no_dimming_when_no_selection() {
        let weeks: Rc<[WorkingHours]> = (17..=18u8)
            .map(|i| {
                make_week(
                    time::Date::from_iso_week_date(2026, i, time::Weekday::Monday).unwrap(),
                    10.0,
                    0.0,
                )
            })
            .collect();
        let html = render_view(ViewProps {
            weeks,
            current_year: 2026,
            current_week: 17,
            selected: None,
            week_short: ImStr::from("KW"),
        });
        assert!(
            !html.contains("opacity: 0.85"),
            "should not dim without selection: {html}"
        );
    }

    #[test]
    fn ssr_current_week_is_always_labeled() {
        // Week 27 normally would not match the `(week-1) % 4 == 0` rule
        // (27-1=26, 26%4=2). Yet, when current_week=27, it should render.
        let weeks: Rc<[WorkingHours]> = vec![make_week(
            time::Date::from_iso_week_date(2026, 27, time::Weekday::Monday).unwrap(),
            10.0,
            0.0,
        )]
        .into();
        let html = render_view(ViewProps {
            weeks,
            current_year: 2026,
            current_week: 27,
            selected: None,
            week_short: ImStr::from("KW"),
        });
        assert!(html.contains("KW 27"), "current-week label missing: {html}");
    }

    #[test]
    fn ssr_label_cadence_every_fourth_week() {
        // Weeks 1, 5, 9 should label; weeks 2, 3, 4, 6 should not.
        let weeks: Rc<[WorkingHours]> = (1..=9u8)
            .map(|i| {
                make_week(
                    time::Date::from_iso_week_date(2026, i, time::Weekday::Monday).unwrap(),
                    10.0,
                    0.0,
                )
            })
            .collect();
        let html = render_view(ViewProps {
            weeks,
            current_year: 2025, // mismatch so current-week rule doesn't fire
            current_week: 1,
            selected: None,
            week_short: ImStr::from("KW"),
        });
        for include in ["KW 1", "KW 5", "KW 9"] {
            assert!(html.contains(include), "missing label `{include}`: {html}");
        }
        for omit in ["KW 2", "KW 3", "KW 4", "KW 6", "KW 7", "KW 8"] {
            assert!(
                !html.contains(omit),
                "unexpected label `{omit}` rendered: {html}"
            );
        }
    }

    #[test]
    fn no_hex_color_literals_in_source() {
        let src = include_str!("employee_weekly_histogram.rs");
        // Strip the test module so test helpers can use any literal.
        let test_module_start = src
            .find("#[cfg(test)]")
            .expect("test module marker missing");
        let prefix = &src[..test_module_start];
        // Look for `#XXXXXX` and `#XXX` color hex patterns.
        let bytes = prefix.as_bytes();
        for i in 0..bytes.len() {
            if bytes[i] == b'#' {
                // Check for 6 or 3 hex digits following the `#`.
                let is_hex = |b: u8| {
                    (b'0'..=b'9').contains(&b)
                        || (b'a'..=b'f').contains(&b)
                        || (b'A'..=b'F').contains(&b)
                };
                if i + 6 < bytes.len()
                    && is_hex(bytes[i + 1])
                    && is_hex(bytes[i + 2])
                    && is_hex(bytes[i + 3])
                    && is_hex(bytes[i + 4])
                    && is_hex(bytes[i + 5])
                    && is_hex(bytes[i + 6])
                {
                    panic!("found 6-char hex literal at offset {i}");
                }
                if i + 3 < bytes.len()
                    && is_hex(bytes[i + 1])
                    && is_hex(bytes[i + 2])
                    && is_hex(bytes[i + 3])
                    // Avoid false positives like markdown or JSON pointers
                    && !is_hex(bytes.get(i + 4).copied().unwrap_or(b' '))
                {
                    // Allow #abc to slip if it happens to align — but our
                    // current source uses `var(--token)` for all colors.
                    // No 3-char hex should be present.
                    panic!("found 3-char hex literal at offset {i}");
                }
            }
        }
    }
}
