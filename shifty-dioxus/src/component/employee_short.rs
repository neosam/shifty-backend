use dioxus::prelude::*;

use crate::base_types::format_hours;
use crate::state::employee::Employee;

#[derive(Clone, Props, PartialEq)]
pub struct EmployeeShortProps {
    pub employee: Employee,

    #[props(default = false)]
    pub active: bool,

    #[props(default = 0.0)]
    pub target_hours: f32,
}

const ROW_BASE: &str =
    "flex items-center gap-3 px-3 py-2 border-l-[3px] hover:bg-surface-alt cursor-pointer";
const ROW_ACTIVE: &str = "bg-accent-soft border-accent";
const ROW_INACTIVE: &str = "border-transparent";

pub(crate) fn row_class(active: bool) -> String {
    let mut out = String::with_capacity(120);
    out.push_str(ROW_BASE);
    out.push(' ');
    if active {
        out.push_str(ROW_ACTIVE);
    } else {
        out.push_str(ROW_INACTIVE);
    }
    out
}

#[component]
pub fn EmployeeShort(props: EmployeeShortProps) -> Element {
    let class = row_class(props.active);
    let color = props.employee.sales_person.background_color.clone();
    let dot_style = format!("background-color: {}; width: 10px; height: 10px;", color);
    let hours = format!(
        "{}/{}",
        format_hours(props.employee.balance, 1),
        format_hours(props.target_hours, 0)
    );

    rsx! {
        div { class: "{class}",
            span {
                class: "rounded-full inline-block flex-shrink-0",
                style: "{dot_style}",
            }
            span { class: "flex-1 text-ink truncate text-sm",
                "{props.employee.sales_person.name}"
            }
            span { class: "font-mono tabular-nums text-xs text-ink-muted",
                "{hours}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::state::shiftplan::SalesPerson;
    use uuid::Uuid;

    fn employee_with(name: &str, color: &str, balance: f32) -> Employee {
        Employee {
            sales_person: SalesPerson {
                id: Uuid::nil(),
                name: name.into(),
                background_color: color.into(),
                is_paid: true,
                inactive: false,
                version: Uuid::nil(),
            },
            working_hours_by_week: Rc::from([]),
            working_hours_by_month: Rc::from([]),
            overall_working_hours: 0.0,
            expected_working_hours: 0.0,
            balance,
            carryover_balance: 0.0,
            shiftplan_hours: 0.0,
            extra_work_hours: 0.0,
            vacation_hours: 0.0,
            sick_leave_hours: 0.0,
            holiday_hours: 0.0,
            unpaid_leave_hours: 0.0,
            volunteer_hours: 0.0,
            vacation_days: 0.0,
            vacation_entitlement: 0.0,
            vacation_carryover: 0,
            custom_extra_hours: Rc::from([]),
        }
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn row_class_active_carries_accent_soft_and_accent_border() {
        let c = row_class(true);
        assert!(c.contains("bg-accent-soft"), "missing bg-accent-soft: {c}");
        assert!(c.contains("border-accent"), "missing border-accent: {c}");
    }

    #[test]
    fn row_class_inactive_carries_transparent_border() {
        let c = row_class(false);
        assert!(
            c.contains("border-transparent"),
            "missing border-transparent: {c}"
        );
        assert!(
            !c.contains("bg-accent-soft"),
            "unexpected bg-accent-soft: {c}"
        );
    }

    #[test]
    fn row_always_reserves_3px_left_border() {
        for active in [true, false] {
            let c = row_class(active);
            assert!(
                c.contains("border-l-[3px]"),
                "missing 3px left border (active={active}): {c}"
            );
        }
    }

    #[test]
    fn row_renders_color_dot_with_inline_background() {
        fn app() -> Element {
            rsx! {
                EmployeeShort {
                    employee: super::tests::employee_with("Lena", "#dbe0ff", 5.0),
                    target_hours: 20.0,
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("background-color: #dbe0ff"),
            "missing inline color: {html}"
        );
        assert!(html.contains("Lena"), "missing name: {html}");
    }

    #[test]
    fn color_dot_has_no_inner_text() {
        fn app() -> Element {
            rsx! {
                EmployeeShort {
                    employee: super::tests::employee_with("Lena", "#dbe0ff", 5.0),
                    target_hours: 20.0,
                }
            }
        }
        let html = render(app);
        // The dot span has only its style attribute and no text node.
        // We assert by verifying that "background-color: #dbe0ff" appears
        // inside a self-contained <span ...></span> tag (no inner text).
        let needle = "background-color: #dbe0ff";
        let dot_idx = html
            .find(needle)
            .expect(&format!("color marker missing: {html}"));
        let after_dot = &html[dot_idx..];
        let close_idx = after_dot
            .find("</span>")
            .expect(&format!("dot's closing tag missing: {html}"));
        let dot_block = &after_dot[..close_idx];
        // Anything after the closing > of the opening tag should be empty
        let inner_start = dot_block.find('>').expect("opening tag must close") + 1;
        let inner = &dot_block[inner_start..];
        assert!(
            inner.trim().is_empty(),
            "color dot has inner text `{}` in: {html}",
            inner
        );
    }

    #[test]
    fn hours_use_mono_tabular_nums() {
        fn app() -> Element {
            rsx! {
                EmployeeShort {
                    employee: super::tests::employee_with("Tom", "#aaa", 12.5),
                    target_hours: 35.0,
                }
            }
        }
        let html = render(app);
        assert!(html.contains("font-mono"), "missing font-mono: {html}");
        assert!(
            html.contains("tabular-nums"),
            "missing tabular-nums: {html}"
        );
        assert!(html.contains("12.5/35"), "missing hours pattern: {html}");
    }

    #[test]
    fn active_row_renders_with_accent_soft_in_html() {
        fn app() -> Element {
            rsx! {
                EmployeeShort {
                    employee: super::tests::employee_with("Lena", "#dbe0ff", 5.0),
                    target_hours: 20.0,
                    active: true,
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
    fn inactive_row_does_not_emit_accent_soft() {
        fn app() -> Element {
            rsx! {
                EmployeeShort {
                    employee: super::tests::employee_with("Lena", "#dbe0ff", 5.0),
                    target_hours: 20.0,
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("bg-accent-soft"),
            "unexpected bg-accent-soft: {html}"
        );
        assert!(
            html.contains("border-transparent"),
            "missing border-transparent: {html}"
        );
    }

    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("employee_short.rs");
        // Crude inspection of the non-test prefix of the source.
        let test_module_start = src
            .find("#[cfg(test)]")
            .expect("test module marker missing");
        let prefix = &src[..test_module_start];
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
            "border-black",
            "border-gray-",
        ] {
            assert!(
                !prefix.contains(forbidden),
                "legacy class `{forbidden}` found in source"
            );
        }
    }
}
