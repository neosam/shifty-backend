use std::rc::Rc;

use dioxus::prelude::*;

use crate::{i18n::Key, service::i18n::I18N, state::weekly_overview::WeeklySummary};

const CHART_HEIGHT: f32 = 250.0;
const PADDING_LEFT: f32 = 45.0;
const PADDING_RIGHT: f32 = 10.0;
const PADDING_TOP: f32 = 10.0;
const PADDING_BOTTOM: f32 = 30.0;
const MIN_BAR_STEP: f32 = 22.0;
const BAR_WIDTH_RATIO: f32 = 0.7;
const LEGEND_HEIGHT: f32 = 30.0;

fn ceil_to_multiple(value: f32, multiple: f32) -> f32 {
    (value / multiple).ceil() * multiple
}

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
    let ceiled = ceil_to_multiple(max_val, 10.0);
    if ceiled < 10.0 {
        10.0
    } else {
        ceiled
    }
}

fn grid_lines(max_hours: f32) -> Vec<f32> {
    let step = if max_hours <= 50.0 { 10.0 } else { 20.0 };
    let mut lines = Vec::new();
    let mut v = 0.0;
    while v <= max_hours {
        lines.push(v);
        v += step;
    }
    lines
}

fn y_pos(hours: f32, max_hours: f32) -> f32 {
    PADDING_TOP + CHART_HEIGHT - (hours / max_hours) * CHART_HEIGHT
}

#[component]
pub fn WeeklyOverviewChart(weeks: Rc<[WeeklySummary]>) -> Element {
    let i18n = I18N.read().clone();
    let paid_label = i18n.t(Key::Paid);
    let volunteer_label = i18n.t(Key::Volunteer);
    let required_label = i18n.t(Key::ChartRequiredHours);

    if weeks.is_empty() {
        return rsx! {};
    }

    let num_weeks = weeks.len() as f32;
    let bar_step = MIN_BAR_STEP;
    let chart_width = num_weeks * bar_step;
    let svg_width = PADDING_LEFT + chart_width + PADDING_RIGHT;
    let svg_height = PADDING_TOP + CHART_HEIGHT + PADDING_BOTTOM + LEGEND_HEIGHT;

    let max_hours = compute_max_hours(&weeks);
    let grid = grid_lines(max_hours);

    let bar_width = bar_step * BAR_WIDTH_RATIO;

    let required_points: String = weeks
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let x = PADDING_LEFT + (i as f32) * bar_step + bar_step / 2.0;
            let y = y_pos(w.required_hours, max_hours);
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");

    let view_box = format!("0 0 {svg_width:.0} {svg_height:.0}");
    let svg_width_str = format!("{svg_width:.0}");

    rsx! {
        div { class: "overflow-x-auto mb-4 mt-4",
            svg {
                view_box: "{view_box}",
                width: "{svg_width_str}",
                height: "{svg_height:.0}",
                style: "min-width: {svg_width_str}px",

                // Grid lines and Y-axis labels
                for hours in grid.iter() {
                    {
                        let y = y_pos(*hours, max_hours);
                        let label = format!("{:.0}", hours);
                        rsx! {
                            line {
                                x1: "{PADDING_LEFT:.0}",
                                y1: "{y:.1}",
                                x2: "{svg_width - PADDING_RIGHT:.0}",
                                y2: "{y:.1}",
                                stroke: "#e5e7eb",
                                stroke_width: "1",
                            }
                            text {
                                x: "{PADDING_LEFT - 5.0:.0}",
                                y: "{y + 3.0:.1}",
                                text_anchor: "end",
                                font_size: "9",
                                fill: "#6b7280",
                                "{label}"
                            }
                        }
                    }
                }

                // Stacked bars
                for (i, week) in weeks.iter().enumerate() {
                    {
                        let x = PADDING_LEFT + (i as f32) * bar_step + (bar_step - bar_width) / 2.0;
                        let paid_h = (week.paid_hours / max_hours) * CHART_HEIGHT;
                        let vol_h = (week.volunteer_hours / max_hours) * CHART_HEIGHT;
                        let total_h = paid_h + vol_h;
                        let paid_y = PADDING_TOP + CHART_HEIGHT - paid_h;
                        let vol_y = paid_y - vol_h;
                        let tooltip = format!(
                            "W{}: {paid_label} {:.1}h, {volunteer_label} {:.1}h, {required_label} {:.1}h",
                            week.week, week.paid_hours, week.volunteer_hours, week.required_hours
                        );

                        rsx! {
                            // Paid hours (bottom)
                            if week.paid_hours > 0.0 {
                                rect {
                                    x: "{x:.1}",
                                    y: "{paid_y:.1}",
                                    width: "{bar_width:.1}",
                                    height: "{paid_h:.1}",
                                    fill: "#3B82F6",
                                }
                            }
                            // Volunteer hours (top)
                            if week.volunteer_hours > 0.0 {
                                rect {
                                    x: "{x:.1}",
                                    y: "{vol_y:.1}",
                                    width: "{bar_width:.1}",
                                    height: "{vol_h:.1}",
                                    fill: "#10B981",
                                }
                            }
                            // Invisible hover rect covering the full bar for tooltip + navigation
                            {
                                let nav_url = format!("/shiftplan/{}/{}", week.year, week.week);
                                rsx! {
                                    rect {
                                        x: "{x:.1}",
                                        y: "{PADDING_TOP + CHART_HEIGHT - total_h:.1}",
                                        width: "{bar_width:.1}",
                                        height: "{total_h.max(1.0):.1}",
                                        fill: "transparent",
                                        style: "cursor: pointer",
                                        onclick: move |_| {
                                            navigator().push(nav_url.clone());
                                        },
                                        title { "{tooltip}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Required hours polyline
                polyline {
                    points: "{required_points}",
                    fill: "none",
                    stroke: "#EF4444",
                    stroke_width: "1.5",
                }

                // X-axis week labels
                for (i, week) in weeks.iter().enumerate() {
                    {
                        let x = PADDING_LEFT + (i as f32) * bar_step + bar_step / 2.0;
                        let y = PADDING_TOP + CHART_HEIGHT + 12.0;
                        let label = format!("{}", week.week);
                        // Only show every Nth label to avoid clutter
                        let show_label = weeks.len() <= 26 || week.week % 2 == 1;
                        rsx! {
                            if show_label {
                                text {
                                    x: "{x:.1}",
                                    y: "{y:.1}",
                                    text_anchor: "middle",
                                    font_size: "7",
                                    fill: "#6b7280",
                                    "{label}"
                                }
                            }
                        }
                    }
                }

                // Legend
                {
                    let legend_y = PADDING_TOP + CHART_HEIGHT + PADDING_BOTTOM + 5.0;
                    let box_size = 8.0;
                    rsx! {
                        // Paid
                        rect {
                            x: "{PADDING_LEFT:.0}",
                            y: "{legend_y:.0}",
                            width: "{box_size:.0}",
                            height: "{box_size:.0}",
                            fill: "#3B82F6",
                        }
                        text {
                            x: "{PADDING_LEFT + box_size + 3.0:.0}",
                            y: "{legend_y + box_size - 1.0:.0}",
                            font_size: "9",
                            fill: "#374151",
                            "{paid_label}"
                        }
                        // Volunteer
                        rect {
                            x: "{PADDING_LEFT + 70.0:.0}",
                            y: "{legend_y:.0}",
                            width: "{box_size:.0}",
                            height: "{box_size:.0}",
                            fill: "#10B981",
                        }
                        text {
                            x: "{PADDING_LEFT + 70.0 + box_size + 3.0:.0}",
                            y: "{legend_y + box_size - 1.0:.0}",
                            font_size: "9",
                            fill: "#374151",
                            "{volunteer_label}"
                        }
                        // Required line
                        line {
                            x1: "{PADDING_LEFT + 160.0:.0}",
                            y1: "{legend_y + box_size / 2.0:.0}",
                            x2: "{PADDING_LEFT + 175.0:.0}",
                            y2: "{legend_y + box_size / 2.0:.0}",
                            stroke: "#EF4444",
                            stroke_width: "1.5",
                        }
                        text {
                            x: "{PADDING_LEFT + 178.0:.0}",
                            y: "{legend_y + box_size - 1.0:.0}",
                            font_size: "9",
                            fill: "#374151",
                            "{required_label}"
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

    #[test]
    fn test_ceil_to_multiple() {
        assert_eq!(ceil_to_multiple(0.0, 10.0), 0.0);
        assert_eq!(ceil_to_multiple(1.0, 10.0), 10.0);
        assert_eq!(ceil_to_multiple(10.0, 10.0), 10.0);
        assert_eq!(ceil_to_multiple(11.0, 10.0), 20.0);
        assert_eq!(ceil_to_multiple(37.0, 10.0), 40.0);
        assert_eq!(ceil_to_multiple(40.0, 10.0), 40.0);
    }

    #[test]
    fn test_compute_max_hours_uses_larger_of_bar_or_required() {
        let weeks = vec![
            WeeklySummary {
                week: 1,
                year: 2026,
                available_hours: 0.0,
                required_hours: 35.0,
                paid_hours: 20.0,
                volunteer_hours: 10.0,
                monday_available_hours: 0.0,
                tuesday_available_hours: 0.0,
                wednesday_available_hours: 0.0,
                thursday_available_hours: 0.0,
                friday_available_hours: 0.0,
                saturday_available_hours: 0.0,
                sunday_available_hours: 0.0,
                sales_person_absences: vec![],
            },
            WeeklySummary {
                week: 2,
                year: 2026,
                available_hours: 0.0,
                required_hours: 25.0,
                paid_hours: 30.0,
                volunteer_hours: 12.0,
                monday_available_hours: 0.0,
                tuesday_available_hours: 0.0,
                wednesday_available_hours: 0.0,
                thursday_available_hours: 0.0,
                friday_available_hours: 0.0,
                saturday_available_hours: 0.0,
                sunday_available_hours: 0.0,
                sales_person_absences: vec![],
            },
        ];
        // Week 2 has bar_total = 42, which is largest => ceil to 50
        assert_eq!(compute_max_hours(&weeks), 50.0);
    }

    #[test]
    fn test_compute_max_hours_empty() {
        assert_eq!(compute_max_hours(&[]), 10.0);
    }

    #[test]
    fn test_grid_lines_small() {
        let lines = grid_lines(40.0);
        assert_eq!(lines, vec![0.0, 10.0, 20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_grid_lines_large() {
        let lines = grid_lines(100.0);
        assert_eq!(lines, vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
    }

    #[test]
    fn test_y_pos() {
        let max = 40.0;
        // 0 hours should be at bottom
        assert_eq!(y_pos(0.0, max), PADDING_TOP + CHART_HEIGHT);
        // max hours should be at top
        assert_eq!(y_pos(max, max), PADDING_TOP);
        // half should be in the middle
        assert_eq!(y_pos(20.0, max), PADDING_TOP + CHART_HEIGHT / 2.0);
    }
}
