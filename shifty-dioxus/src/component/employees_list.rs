//! `EmployeesList` — searchable employee list rendered in the master/detail
//! shell. Reads the active employee id from props (the shell derives it
//! from the route) and passes it through `EmployeeShort` for highlight
//! rendering.

use dioxus::prelude::*;
use uuid::Uuid;

use crate::component::EmployeeShort;
use crate::i18n::Key;
use crate::js;
use crate::loader;
use crate::router::Route;
use crate::service::{config::CONFIG, i18n::I18N};
use crate::state::employee::Employee;

#[derive(Props, Clone, PartialEq)]
pub struct EmployeesListProps {
    #[props(!optional, default = None)]
    pub active_id: Option<Uuid>,
}

const SEARCH_INPUT_CLASSES: &str =
    "h-[34px] px-[10px] border border-border-strong rounded-md bg-surface text-ink text-body w-full min-w-0 form-input";

pub(crate) fn matches_search(name: &str, term: &str) -> bool {
    if term.is_empty() {
        return true;
    }
    name.to_lowercase().contains(&term.to_lowercase())
}

pub(crate) fn target_hours_for(employee: &Employee) -> f32 {
    employee
        .working_hours_by_week
        .iter()
        .last()
        .map(|w| w.expected_hours)
        .unwrap_or(0.0)
}

#[component]
pub fn EmployeesList(props: EmployeesListProps) -> Element {
    let i18n = I18N.read().clone();
    let year = use_signal(|| js::get_current_year());
    let week_until = if *year.read() == js::get_current_year() {
        js::get_current_week()
    } else {
        52
    };
    let config = CONFIG.read().clone();
    let employees =
        use_resource(move || loader::load_employees(config.to_owned(), *year.read(), week_until));

    let mut search = use_signal(String::new);

    let placeholder = i18n.t(Key::SearchPlaceholder);
    let heading = i18n.t(Key::Employees);

    rsx! {
        div { class: "flex flex-col gap-3 p-3",
            h2 { class: "text-micro font-bold text-ink-muted uppercase",
                "{heading}"
            }
            input {
                class: "{SEARCH_INPUT_CLASSES}",
                r#type: "text",
                placeholder: "{placeholder}",
                value: "{search.read()}",
                oninput: move |evt| search.set(evt.value()),
            }
            div { class: "flex flex-col",
                match &*employees.read_unchecked() {
                    Some(Ok(list)) => {
                        let term = search.read().clone();
                        let mut filtered: Vec<Employee> = list
                            .iter()
                            .filter(|e| !e.sales_person.inactive)
                            .filter(|e| matches_search(&e.sales_person.name, &term))
                            .cloned()
                            .collect();
                        filtered.sort_by(|a, b| a.sales_person.name.cmp(&b.sales_person.name));
                        rsx! {
                            for employee in filtered.into_iter() {
                                {
                                    let id = employee.sales_person.id;
                                    let active = props.active_id == Some(id);
                                    let target = target_hours_for(&employee);
                                    rsx! {
                                        Link {
                                            to: Route::EmployeeDetails {
                                                employee_id: id.to_string(),
                                            },
                                            EmployeeShort {
                                                employee,
                                                active,
                                                target_hours: target,
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(err)) => rsx! {
                        div { class: "text-bad text-body px-3 py-2",
                            "Error: {err}"
                        }
                    },
                    None => rsx! {
                        div { class: "text-ink-muted text-body px-3 py-2",
                            "Loading…"
                        }
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_search_empty_term_matches_all() {
        assert!(matches_search("Lena", ""));
        assert!(matches_search("", ""));
    }

    #[test]
    fn matches_search_is_case_insensitive() {
        assert!(matches_search("Lena Müller", "lena"));
        assert!(matches_search("lena müller", "LENA"));
        assert!(matches_search("Tom", "TOM"));
    }

    #[test]
    fn matches_search_substring_match() {
        assert!(matches_search("Lena Müller", "müller"));
        assert!(matches_search("Lena Müller", "ena"));
        assert!(!matches_search("Tom", "Lena"));
    }

    #[test]
    fn target_hours_for_returns_zero_when_no_weeks() {
        use crate::state::shiftplan::SalesPerson;
        use std::rc::Rc;
        let employee = Employee {
            sales_person: SalesPerson::default(),
            working_hours_by_week: Rc::from([]),
            working_hours_by_month: Rc::from([]),
            overall_working_hours: 0.0,
            expected_working_hours: 0.0,
            balance: 0.0,
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
        };
        assert_eq!(target_hours_for(&employee), 0.0);
    }

    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("employees_list.rs");
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
