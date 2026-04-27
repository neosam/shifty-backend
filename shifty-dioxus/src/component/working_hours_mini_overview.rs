use std::rc::Rc;

use dioxus::prelude::*;
use uuid::Uuid;

use crate::base_types::format_hours;
use crate::state::employee_work_details::WorkingHoursMini;

#[derive(PartialEq, Clone, Props)]
pub struct WorkingHoursMiniOverviewProps {
    pub working_hours: Rc<[WorkingHoursMini]>,
    #[props(!optional)]
    pub selected_sales_person_id: Option<Uuid>,
    #[props(default = false)]
    pub show_balance: bool,

    pub on_dbl_click: EventHandler<Uuid>,
}

/// Returns the percent-of-target for the progress bar, capped at 100.
pub(crate) fn progress_bar_percent(actual: f32, target: f32) -> f32 {
    if target <= 0.0 {
        return 0.0;
    }
    let pct = actual / target * 100.0;
    pct.clamp(0.0, 100.0)
}

/// Returns the token class string for the progress-bar fill segment.
pub(crate) fn progress_bar_class(actual: f32, target: f32) -> &'static str {
    if actual < target {
        "bg-warn"
    } else {
        "bg-good"
    }
}

/// Returns the token class for the hours-line text color.
pub(crate) fn hours_text_class(actual: f32, target: f32) -> &'static str {
    if actual < target {
        "text-warn"
    } else {
        "text-good"
    }
}

#[component]
pub fn WorkingHoursMiniOverview(props: WorkingHoursMiniOverviewProps) -> Element {
    let mut working_hours: Vec<WorkingHoursMini> = props.working_hours.iter().cloned().collect();
    working_hours.sort_by(|a, b| a.sales_person_name.cmp(&b.sales_person_name));
    rsx! {
        div {
            class: "grid gap-2 select-none",
            style: "grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));",
            for working_hour in working_hours.iter() {
                {
                    let sales_person_id = working_hour.sales_person_id;
                    let actual = working_hour.actual_hours;
                    let target = working_hour.dynamic_hours;
                    let actual_hours_str = format_hours(actual, 1);
                    let dynamic_hours_str = format_hours(target, 1);
                    let balance_hours_str = format_hours(working_hour.balance_hours, 1);
                    let show_balance = props.show_balance;
                    let is_selected = Some(sales_person_id) == props.selected_sales_person_id;
                    let card_class = if is_selected {
                        "flex items-center gap-[10px] bg-accent-soft border border-accent rounded-md cursor-pointer"
                    } else {
                        "flex items-center gap-[10px] bg-surface border border-border rounded-md cursor-pointer hover:bg-surface-alt"
                    };
                    let bar_class = progress_bar_class(actual, target);
                    let hours_class = hours_text_class(actual, target);
                    let pct = progress_bar_percent(actual, target);
                    let bg_color = working_hour.background_color.clone();
                    rsx! {
                        div {
                            class: "{card_class}",
                            style: "padding: 10px 12px;",
                            ondoubleclick: move |_| props.on_dbl_click.call(sales_person_id),
                            span {
                                class: "rounded-full flex-shrink-0",
                                style: "width: 28px; height: 28px; background-color: {bg_color};",
                                "aria-hidden": "true",
                            }
                            div { class: "flex-1 min-w-0",
                                div { class: "text-xs font-medium text-ink truncate",
                                    "{working_hour.sales_person_name}"
                                }
                                div {
                                    class: "font-mono tabular-nums {hours_class}",
                                    style: "font-size: 10px;",
                                    "{actual_hours_str} / {dynamic_hours_str}h"
                                    if show_balance {
                                        span { class: "text-ink-muted ml-1", "({balance_hours_str})" }
                                    }
                                }
                                div {
                                    class: "bg-surface-2 rounded-[2px] mt-1 overflow-hidden",
                                    style: "height: 3px;",
                                    div {
                                        class: "h-full {bar_class}",
                                        style: "width: {pct}%;",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_bar_percent_caps_at_100() {
        assert_eq!(progress_bar_percent(12.0, 8.0), 100.0);
    }

    #[test]
    fn progress_bar_percent_zero_for_zero_target() {
        assert_eq!(progress_bar_percent(5.0, 0.0), 0.0);
    }

    #[test]
    fn progress_bar_percent_proportional() {
        assert!((progress_bar_percent(4.0, 8.0) - 50.0).abs() < 0.01);
    }

    #[test]
    fn progress_bar_percent_zero_for_zero_actual() {
        assert_eq!(progress_bar_percent(0.0, 8.0), 0.0);
    }

    #[test]
    fn progress_bar_class_under_target() {
        assert_eq!(progress_bar_class(5.0, 8.0), "bg-warn");
    }

    #[test]
    fn progress_bar_class_at_target() {
        assert_eq!(progress_bar_class(8.0, 8.0), "bg-good");
    }

    #[test]
    fn progress_bar_class_over_target() {
        assert_eq!(progress_bar_class(10.0, 8.0), "bg-good");
    }

    #[test]
    fn hours_text_class_under_target_is_warn() {
        assert_eq!(hours_text_class(5.0, 8.0), "text-warn");
    }

    #[test]
    fn hours_text_class_at_target_is_good() {
        assert_eq!(hours_text_class(8.0, 8.0), "text-good");
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    fn make_row(name: &str, color: &str, actual: f32, target: f32) -> WorkingHoursMini {
        WorkingHoursMini {
            sales_person_id: Uuid::nil(),
            sales_person_name: name.into(),
            expected_hours: target,
            dynamic_hours: target,
            actual_hours: actual,
            balance_hours: 0.0,
            background_color: color.into(),
        }
    }

    #[test]
    fn mini_overview_uses_auto_fit_grid() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([make_row("A", "#abc", 5.0, 8.0)].to_vec()),
                    selected_sales_person_id: None,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("repeat(auto-fit, minmax(180px, 1fr))"),
            "missing auto-fit grid: {html}"
        );
    }

    #[test]
    fn mini_overview_card_renders_color_dot() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([make_row("Alex", "#dbe0ff", 5.0, 8.0)].to_vec()),
                    selected_sales_person_id: None,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("background-color: #dbe0ff"),
            "missing color: {html}"
        );
        assert!(
            html.contains("rounded-full"),
            "missing rounded-full: {html}"
        );
        assert!(html.contains("28px"), "missing 28px size: {html}");
    }

    #[test]
    fn mini_overview_progress_bar_warn_color() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([make_row("A", "#abc", 5.0, 8.0)].to_vec()),
                    selected_sales_person_id: None,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("bg-warn"), "missing bg-warn: {html}");
    }

    #[test]
    fn mini_overview_progress_bar_good_color() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([make_row("A", "#abc", 8.0, 8.0)].to_vec()),
                    selected_sales_person_id: None,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("bg-good"), "missing bg-good: {html}");
    }

    #[test]
    fn mini_overview_selected_card_carries_accent() {
        const TEST_ID: Uuid = Uuid::from_u128(0x1234_5678_1234_5678_1234_5678_1234_5678);
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([WorkingHoursMini {
                        sales_person_id: TEST_ID,
                        sales_person_name: "Selected".into(),
                        expected_hours: 8.0,
                        dynamic_hours: 8.0,
                        actual_hours: 5.0,
                        balance_hours: 0.0,
                        background_color: "#abc".into(),
                    }].to_vec()),
                    selected_sales_person_id: Some(TEST_ID),
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("bg-accent-soft"),
            "missing bg-accent-soft: {html}"
        );
        assert!(
            html.contains("border-accent"),
            "missing border-accent: {html}"
        );
    }

    #[test]
    fn mini_overview_show_balance_renders_parenthesized() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([WorkingHoursMini {
                        sales_person_id: Uuid::nil(),
                        sales_person_name: "A".into(),
                        expected_hours: 8.0,
                        dynamic_hours: 8.0,
                        actual_hours: 5.0,
                        balance_hours: -2.5,
                        background_color: "#abc".into(),
                    }].to_vec()),
                    selected_sales_person_id: None,
                    show_balance: true,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("(-2.5)") || html.contains("(-2,5)"),
            "missing parenthesized balance: {html}"
        );
    }

    #[test]
    fn mini_overview_no_legacy_color_classes() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([make_row("A", "#abc", 5.0, 8.0)].to_vec()),
                    selected_sales_person_id: None,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        for forbidden in [
            "text-red-800",
            "text-green-800",
            "text-gray-600",
            "bg-gray-200",
        ] {
            assert!(
                !html.contains(forbidden),
                "unexpected legacy class `{}`: {}",
                forbidden,
                html
            );
        }
    }

    #[test]
    fn working_hours_mini_overview_no_legacy_classes_in_source() {
        let source = include_str!("working_hours_mini_overview.rs");
        // Strip the test module so the legacy-class assertions inside `assert!`
        // strings don't trigger the sweep.
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for forbidden in [
            "bg-gray-",
            "bg-white",
            "text-gray-",
            "text-blue-",
            "text-red-",
            "text-green-",
            "bg-blue-",
            "bg-green-",
            "bg-red-",
            "border-gray-",
            "border-black",
        ] {
            assert!(
                !production.contains(forbidden),
                "non-test source contains legacy class `{}`",
                forbidden
            );
        }
    }
}
