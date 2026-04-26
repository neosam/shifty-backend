//! `EmployeesShell` — the master/detail composition layer for the
//! employees pages.
//!
//! On viewports above the 720 px breakpoint, the shell renders the
//! employee list on the left and the routed page content on the right.
//! Below the breakpoint, it shows either the list (when no employee is
//! selected via the `EmployeeDetails` route) or the children only.
//!
//! The active employee id is read from the route, so the list's
//! highlight stays in sync with the URL without explicit prop wiring.

use dioxus::prelude::*;
use uuid::Uuid;

use crate::component::atoms::use_media_query;
use crate::component::employees_list::EmployeesList;
use crate::router::Route;

#[derive(Props, Clone, PartialEq)]
pub struct EmployeesShellProps {
    pub children: Element,
}

#[component]
pub fn EmployeesShell(props: EmployeesShellProps) -> Element {
    let route = use_route::<Route>();
    let active_id = match &route {
        Route::EmployeeDetails { employee_id } => Uuid::parse_str(employee_id).ok(),
        _ => None,
    };

    let is_mobile = *use_media_query("(max-width: 720px)").read();

    if is_mobile {
        if active_id.is_some() {
            rsx! {
                div { class: "w-full", { props.children } }
            }
        } else {
            rsx! {
                EmployeesList { active_id: active_id }
            }
        }
    } else {
        rsx! {
            div { class: "flex w-full",
                aside {
                    class: "w-[280px] shrink-0 border-r border-border bg-surface md:w-[320px] lg:w-[360px] overflow-y-auto",
                    style: "max-height: calc(100vh - 56px);",
                    EmployeesList { active_id: active_id }
                }
                main { class: "flex-1 min-w-0", { props.children } }
            }
        }
    }
}
