use std::rc::Rc;

use dioxus::prelude::*;
use uuid::Uuid;

use crate::base_types::format_hours;
use crate::i18n::Key;
use crate::service::i18n::I18N;
use crate::service::ui_prefs::WorkingHoursLayout;
use crate::state::employee_work_details::WorkingHoursMini;

#[derive(PartialEq, Clone, Props)]
pub struct WorkingHoursMiniOverviewProps {
    pub working_hours: Rc<[WorkingHoursMini]>,
    #[props(!optional)]
    pub selected_sales_person_id: Option<Uuid>,
    #[props(default = false)]
    pub show_balance: bool,
    #[props(default = WorkingHoursLayout::Cards)]
    pub layout: WorkingHoursLayout,

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

/// Formats the difference `actual - target` as a signed string with one decimal
/// and an `h` suffix. Non-negative values are prefixed with `+`.
pub(crate) fn signed_hours_diff(actual: f32, target: f32) -> String {
    let diff = actual - target;
    let body = format_hours(diff, 1);
    if body.starts_with('-') || body.starts_with('+') {
        format!("{body}h")
    } else {
        format!("+{body}h")
    }
}

#[component]
pub fn WorkingHoursMiniOverview(props: WorkingHoursMiniOverviewProps) -> Element {
    let mut working_hours: Vec<WorkingHoursMini> = props.working_hours.iter().cloned().collect();
    working_hours.sort_by(|a, b| a.sales_person_name.cmp(&b.sales_person_name));

    match props.layout {
        WorkingHoursLayout::Cards => rsx! {
            CardsLayout {
                rows: working_hours,
                selected_sales_person_id: props.selected_sales_person_id,
                show_balance: props.show_balance,
                on_dbl_click: props.on_dbl_click,
            }
        },
        WorkingHoursLayout::Table => rsx! {
            TableLayout {
                rows: working_hours,
                selected_sales_person_id: props.selected_sales_person_id,
                on_dbl_click: props.on_dbl_click,
            }
        },
    }
}

#[derive(PartialEq, Clone, Props)]
struct LayoutInnerProps {
    rows: Vec<WorkingHoursMini>,
    #[props(!optional)]
    selected_sales_person_id: Option<Uuid>,
    #[props(default = false)]
    show_balance: bool,
    on_dbl_click: EventHandler<Uuid>,
}

#[component]
fn CardsLayout(props: LayoutInnerProps) -> Element {
    rsx! {
        div {
            class: "grid gap-2 select-none",
            style: "grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));",
            for working_hour in props.rows.iter() {
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
                                div { class: "text-small font-medium text-ink truncate",
                                    "{working_hour.sales_person_name}"
                                }
                                div {
                                    class: "font-mono tabular-nums text-micro {hours_class}",
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

#[component]
fn TableLayout(props: LayoutInnerProps) -> Element {
    let i18n = I18N.read().clone();
    let total_actual: f32 = props.rows.iter().map(|r| r.actual_hours).sum();
    let total_target: f32 = props.rows.iter().map(|r| r.dynamic_hours).sum();
    let total_actual_str = format_hours(total_actual, 1);
    let total_target_str = format_hours(total_target, 1);

    rsx! {
        div {
            class: "bg-surface border border-border rounded-lg overflow-hidden select-none",
            style: "overflow-x: auto;",
            table {
                class: "w-full border-collapse text-body",
                thead {
                    tr {
                        class: "bg-surface-alt text-left",
                        th {
                            class: "px-[14px] py-2 text-micro font-bold text-ink-muted uppercase",
                            "{i18n.t(Key::WorkingHoursTableEmployee)}"
                        }
                        th {
                            class: "px-[14px] py-2 text-micro font-bold text-ink-muted uppercase text-right",
                            "{i18n.t(Key::WorkingHoursTableActual)}"
                        }
                        th {
                            class: "px-[14px] py-2 text-micro font-bold text-ink-muted uppercase text-right",
                            "{i18n.t(Key::WorkingHoursTableTarget)}"
                        }
                        th {
                            class: "px-[14px] py-2 text-micro font-bold text-ink-muted uppercase",
                            "{i18n.t(Key::WorkingHoursTableUtilization)}"
                        }
                        th {
                            class: "px-[14px] py-2 text-micro font-bold text-ink-muted uppercase text-right",
                            "{i18n.t(Key::Balance)}"
                        }
                    }
                }
                tbody {
                    for working_hour in props.rows.iter() {
                        {
                            let sales_person_id = working_hour.sales_person_id;
                            let actual = working_hour.actual_hours;
                            let target = working_hour.dynamic_hours;
                            let balance = working_hour.balance_hours;
                            let actual_str = format_hours(actual, 1);
                            let target_str = format_hours(target, 1);
                            let balance_str = signed_hours_diff(balance, 0.0);
                            let balance_class = hours_text_class(balance, 0.0);
                            let bar_class = progress_bar_class(actual, target);
                            let pct = progress_bar_percent(actual, target);
                            let pct_int = pct.round() as i32;
                            let bg_color = working_hour.background_color.clone();
                            let is_selected = Some(sales_person_id) == props.selected_sales_person_id;
                            let row_class = if is_selected {
                                "border-t border-border bg-accent-soft cursor-pointer"
                            } else {
                                "border-t border-border cursor-pointer hover:bg-surface-alt"
                            };
                            rsx! {
                                tr {
                                    class: "{row_class}",
                                    ondoubleclick: move |_| props.on_dbl_click.call(sales_person_id),
                                    td {
                                        class: "px-[14px] py-2",
                                        div {
                                            class: "flex items-center gap-[10px]",
                                            span {
                                                class: "rounded-full flex-shrink-0",
                                                style: "width: 22px; height: 22px; background-color: {bg_color};",
                                                "aria-hidden": "true",
                                            }
                                            span { class: "font-medium text-ink truncate",
                                                "{working_hour.sales_person_name}"
                                            }
                                        }
                                    }
                                    td {
                                        class: "font-mono tabular-nums px-[14px] py-2 text-right",
                                        "{actual_str}h"
                                    }
                                    td {
                                        class: "font-mono tabular-nums px-[14px] py-2 text-right text-ink-muted",
                                        "{target_str}h"
                                    }
                                    td {
                                        class: "px-[14px] py-2",
                                        style: "min-width: 140px;",
                                        div {
                                            class: "flex items-center gap-2",
                                            div {
                                                class: "flex-1 bg-surface-2 rounded-[3px] overflow-hidden",
                                                style: "height: 6px;",
                                                div {
                                                    class: "h-full {bar_class}",
                                                    style: "width: {pct}%;",
                                                }
                                            }
                                            span {
                                                class: "font-mono tabular-nums text-small font-normal text-ink-muted text-right",
                                                style: "min-width: 38px;",
                                                "{pct_int}%"
                                            }
                                        }
                                    }
                                    td {
                                        class: "font-mono tabular-nums px-[14px] py-2 text-right font-semibold {balance_class}",
                                        "{balance_str}"
                                    }
                                }
                            }
                        }
                    }
                    tr {
                        class: "border-t border-border-strong bg-surface-alt",
                        td {
                            class: "px-[14px] py-2 font-semibold text-ink-muted",
                            "{i18n.t(Key::WorkingHoursTableTotal)}"
                        }
                        td {
                            class: "font-mono tabular-nums px-[14px] py-2 text-right font-semibold",
                            "{total_actual_str}h"
                        }
                        td {
                            class: "font-mono tabular-nums px-[14px] py-2 text-right font-semibold text-ink-muted",
                            "{total_target_str}h"
                        }
                        td { class: "px-[14px] py-2" }
                        td { class: "px-[14px] py-2" }
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

    #[test]
    fn signed_hours_diff_positive_carries_plus() {
        assert_eq!(signed_hours_diff(22.0, 20.0), "+2.0h");
    }

    #[test]
    fn signed_hours_diff_negative_carries_minus() {
        assert_eq!(signed_hours_diff(15.0, 20.0), "-5.0h");
    }

    #[test]
    fn signed_hours_diff_exact_target_is_plus_zero() {
        assert_eq!(signed_hours_diff(20.0, 20.0), "+0.0h");
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
                    layout: WorkingHoursLayout::Cards,
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
                    layout: WorkingHoursLayout::Cards,
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
                    layout: WorkingHoursLayout::Cards,
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
                    layout: WorkingHoursLayout::Cards,
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
                    layout: WorkingHoursLayout::Cards,
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
                    layout: WorkingHoursLayout::Cards,
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
                    layout: WorkingHoursLayout::Cards,
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

    // ----- Table layout SSR tests -----

    #[test]
    fn table_layout_renders_table_with_thead_tbody_and_total_row() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([
                        make_row("Alice", "#abc", 5.0, 10.0),
                        make_row("Bob", "#def", 8.0, 8.0),
                        make_row("Charlie", "#ghi", 12.0, 10.0),
                    ].to_vec()),
                    selected_sales_person_id: None,
                    layout: WorkingHoursLayout::Table,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("<table"), "missing <table>: {html}");
        assert!(html.contains("<thead"), "missing <thead>: {html}");
        assert!(html.contains("<tbody"), "missing <tbody>: {html}");
        // Three body rows + 1 total row + 1 header row = 5 <tr> total.
        let tr_count = html.matches("<tr").count();
        assert_eq!(
            tr_count, 5,
            "expected 5 <tr> elements, got {tr_count}: {html}"
        );
        // English default i18n: "Total"
        assert!(html.contains("Total"), "missing Total footer label: {html}");
    }

    #[test]
    fn table_layout_balance_positive_uses_text_good() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([WorkingHoursMini {
                        sales_person_id: Uuid::nil(),
                        sales_person_name: "A".into(),
                        expected_hours: 20.0,
                        dynamic_hours: 20.0,
                        actual_hours: 22.0,
                        balance_hours: 3.5,
                        background_color: "#abc".into(),
                    }].to_vec()),
                    selected_sales_person_id: None,
                    layout: WorkingHoursLayout::Table,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("+3.5h"), "missing +3.5h balance: {html}");
        assert!(html.contains("text-good"), "missing text-good: {html}");
    }

    #[test]
    fn table_layout_balance_negative_uses_text_warn() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([WorkingHoursMini {
                        sales_person_id: Uuid::nil(),
                        sales_person_name: "A".into(),
                        expected_hours: 20.0,
                        dynamic_hours: 20.0,
                        actual_hours: 15.0,
                        balance_hours: -7.0,
                        background_color: "#abc".into(),
                    }].to_vec()),
                    selected_sales_person_id: None,
                    layout: WorkingHoursLayout::Table,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("-7.0h"), "missing -7.0h balance: {html}");
        assert!(html.contains("text-warn"), "missing text-warn: {html}");
    }

    /// The balance column must reflect `balance_hours`, not `actual - target`.
    /// Why: a positive workload diff for the current week can coexist with a
    /// negative carry-over balance (or vice versa). Replacing the column was
    /// the whole point of this change.
    #[test]
    fn table_layout_balance_independent_of_actual_target_diff() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    // Positive week diff (+2h) but a negative balance overall.
                    working_hours: Rc::from([WorkingHoursMini {
                        sales_person_id: Uuid::nil(),
                        sales_person_name: "A".into(),
                        expected_hours: 20.0,
                        dynamic_hours: 20.0,
                        actual_hours: 22.0,
                        balance_hours: -4.0,
                        background_color: "#abc".into(),
                    }].to_vec()),
                    selected_sales_person_id: None,
                    layout: WorkingHoursLayout::Table,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("-4.0h"),
            "balance column must show balance_hours, not actual-target: {html}"
        );
        assert!(
            !html.contains("+2.0h"),
            "balance column must NOT show actual-target diff: {html}"
        );
    }

    #[test]
    fn table_layout_utilization_renders_progress_bar_and_percent() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([make_row("A", "#abc", 5.0, 10.0)].to_vec()),
                    selected_sales_person_id: None,
                    layout: WorkingHoursLayout::Table,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("width: 50%"), "missing width: 50%: {html}");
        assert!(html.contains("50%"), "missing 50% label: {html}");
    }

    /// Footer aggregates actual and target hours, but deliberately does NOT
    /// aggregate balance — a sum of individual hour-account balances has no
    /// meaningful interpretation.
    #[test]
    fn table_layout_footer_aggregates_actual_and_target_but_not_balance() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([
                        WorkingHoursMini {
                            sales_person_id: Uuid::nil(),
                            sales_person_name: "A".into(),
                            expected_hours: 10.0,
                            dynamic_hours: 10.0,
                            actual_hours: 5.0,
                            balance_hours: -2.5,
                            background_color: "#abc".into(),
                        },
                        WorkingHoursMini {
                            sales_person_id: Uuid::nil(),
                            sales_person_name: "B".into(),
                            expected_hours: 10.0,
                            dynamic_hours: 10.0,
                            actual_hours: 12.0,
                            balance_hours: 4.5,
                            background_color: "#def".into(),
                        },
                    ].to_vec()),
                    selected_sales_person_id: None,
                    layout: WorkingHoursLayout::Table,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        // Per-row balances rendered in body cells.
        assert!(html.contains("-2.5h"), "missing row balance -2.5h: {html}");
        assert!(html.contains("+4.5h"), "missing row balance +4.5h: {html}");
        // Footer aggregates actual and target only.
        assert!(html.contains("17.0h"), "missing actual total 17.0h: {html}");
        assert!(html.contains("20.0h"), "missing target total 20.0h: {html}");
        // Sum of balance_hours would be -2.5 + 4.5 = +2.0; must not appear.
        assert!(
            !html.contains("+2.0h"),
            "balance must not be aggregated in footer: {html}"
        );
    }

    #[test]
    fn rows_render_alphabetical_in_both_layouts() {
        fn cards() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([
                        make_row("Charlie", "#abc", 5.0, 10.0),
                        make_row("Alice", "#def", 5.0, 10.0),
                        make_row("Bob", "#ghi", 5.0, 10.0),
                    ].to_vec()),
                    selected_sales_person_id: None,
                    layout: WorkingHoursLayout::Cards,
                    on_dbl_click: |_| {},
                }
            }
        }
        fn table() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([
                        make_row("Charlie", "#abc", 5.0, 10.0),
                        make_row("Alice", "#def", 5.0, 10.0),
                        make_row("Bob", "#ghi", 5.0, 10.0),
                    ].to_vec()),
                    selected_sales_person_id: None,
                    layout: WorkingHoursLayout::Table,
                    on_dbl_click: |_| {},
                }
            }
        }
        for html in [render(cards), render(table)] {
            let alice = html.find("Alice").expect("Alice missing");
            let bob = html.find("Bob").expect("Bob missing");
            let charlie = html.find("Charlie").expect("Charlie missing");
            assert!(alice < bob, "Alice should appear before Bob: {html}");
            assert!(bob < charlie, "Bob should appear before Charlie: {html}");
        }
    }

    #[test]
    fn table_layout_selected_row_carries_accent() {
        const TEST_ID: Uuid = Uuid::from_u128(0xaaaa_bbbb_cccc_dddd_eeee_ffff_0000_1111);
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
                    layout: WorkingHoursLayout::Table,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("bg-accent-soft"),
            "missing bg-accent-soft on selected row: {html}"
        );
    }

    /// The balance column belongs at the right edge — utilization (the
    /// progress bar) is the at-a-glance signal, balance is the deep-detail
    /// follow-up. Verified at the rendered-HTML level so we catch a
    /// reordering regression even if the source structure changes.
    #[test]
    fn table_layout_balance_column_is_right_of_utilization() {
        fn app() -> Element {
            rsx! {
                WorkingHoursMiniOverview {
                    working_hours: Rc::from([WorkingHoursMini {
                        sales_person_id: Uuid::nil(),
                        sales_person_name: "A".into(),
                        expected_hours: 10.0,
                        dynamic_hours: 10.0,
                        actual_hours: 5.0,
                        balance_hours: 1.5,
                        background_color: "#abc".into(),
                    }].to_vec()),
                    selected_sales_person_id: None,
                    layout: WorkingHoursLayout::Table,
                    on_dbl_click: |_| {},
                }
            }
        }
        let html = render(app);
        let utilization = html.find("Utilization").expect("Utilization header missing");
        let balance = html.find("Balance").expect("Balance header missing");
        assert!(
            utilization < balance,
            "Balance header must appear after Utilization in the rendered table: {html}"
        );
    }

    /// The trailing column is the locked anchor for the "Stundenkonto"
    /// (balance) label. Reading at the source level keeps the assertion robust
    /// against locale switches in test setup.
    #[test]
    fn table_layout_uses_balance_key_for_trailing_column_header() {
        let source = include_str!("working_hours_mini_overview.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        let table_block = production
            .split_once("fn TableLayout")
            .map(|(_, after)| after)
            .expect("TableLayout function not found in source");
        assert!(
            table_block.contains("Key::Balance"),
            "TableLayout source must reference Key::Balance for the column header"
        );
        assert!(
            !table_block.contains("Key::WorkingHoursTableDifference"),
            "TableLayout source must no longer reference the legacy difference key"
        );
    }

    #[test]
    fn table_layout_wires_ondoubleclick_in_source() {
        // Dioxus SSR does not emit event-handler attributes into the rendered
        // HTML, so we verify the wiring at the source level: the production
        // `TableLayout` body must contain `ondoubleclick:` on its row element.
        let source = include_str!("working_hours_mini_overview.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        let table_block = production
            .split_once("fn TableLayout")
            .map(|(_, after)| after)
            .expect("TableLayout function not found in source");
        assert!(
            table_block.contains("ondoubleclick:"),
            "TableLayout source must wire ondoubleclick on a row"
        );
        assert!(
            table_block.contains("on_dbl_click.call"),
            "TableLayout source must call on_dbl_click handler"
        );
    }
}
