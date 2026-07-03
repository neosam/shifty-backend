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
use crate::service::{config::CONFIG, employee::EMPLOYEES_LIST_REFRESH, i18n::I18N};
use crate::state::employee::Employee;

/// D-03: pure filter predicate — extracted for unit-testability.
/// Returns true if the employee should be shown given current filter state.
/// Rules:
///   - inactive employees are always hidden (independent of show_all)
///   - when show_all=false, only paid employees are shown
///   - when show_all=true, paid AND unpaid non-inactive employees are shown
///   - search term filter applies in all cases
pub(crate) fn employee_visible(e: &Employee, show_all: bool, term: &str) -> bool {
    !e.sales_person.inactive
        && (show_all || e.sales_person.is_paid)
        && matches_search(&e.sales_person.name, term)
}

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
    let year = use_signal(js::get_current_year);
    let week_until = if *year.read() == js::get_current_year() {
        js::get_current_week()
    } else {
        52
    };
    let config = CONFIG.read().clone();
    let config2 = config.clone();
    let employees = use_resource(move || {
        // Subscribe to the sidebar refresh token so any mutation that bumps
        // it (via `refresh_employee_data` in the employee service) re-runs
        // this resource and the cached list stays in sync with the detail
        // view's aggregates.
        let _refresh_token = *EMPLOYEES_LIST_REFRESH.read();
        loader::load_employees(config.to_owned(), *year.read(), week_until)
    });

    // D-03: Second resource call — loads ALL sales persons (incl. is_paid=false)
    // for merging in show_all-mode. The loader filters to !is_paid && !inactive,
    // so the result is disjoint from the paid-list above (no dedup needed).
    let unpaid_employees = use_resource(move || {
        loader::load_unpaid_volunteer_employees(config2.to_owned())
    });

    // D-03: Default false = only paid employees shown; true = also show unpaid volunteers.
    let mut show_all = use_signal(|| false);

    let mut search = use_signal(String::new);

    let placeholder = i18n.t(Key::SearchPlaceholder);
    let heading = i18n.t(Key::Employees);
    let show_all_label = i18n.t(Key::EmployeesShowAll);

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
            // D-03: show_all toggle — reveals unpaid non-inactive volunteers
            label { class: "flex items-center gap-2 text-body text-ink",
                input {
                    r#type: "checkbox",
                    class: "rounded border-border accent-accent",
                    checked: *show_all.read(),
                    onchange: move |event| show_all.set(event.checked()),
                }
                span { "{show_all_label}" }
            }
            div { class: "flex flex-col",
                match (&*employees.read_unchecked(), &*unpaid_employees.read_unchecked()) {
                    (Some(Ok(list)), unpaid_result) => {
                        let term = search.read().clone();
                        let show_all_val = *show_all.read();

                        // Merge unpaid dummies into the list when show_all is active.
                        // The paid list (from GET /report) is paid-only by backend design;
                        // unpaid dummies come from GET /sales-person filtered to !is_paid && !inactive.
                        // The two sets are disjoint — no deduplication needed.
                        let mut combined: Vec<Employee> = list.iter().cloned().collect();
                        if show_all_val {
                            if let Some(Ok(unpaid_list)) = unpaid_result {
                                combined.extend(unpaid_list.iter().cloned());
                            }
                        }

                        let mut filtered: Vec<Employee> = combined
                            .iter()
                            .filter(|e| employee_visible(e, show_all_val, &term))
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
                    (Some(Err(err)), _) => rsx! {
                        div { class: "text-bad text-body px-3 py-2",
                            "Error: {err}"
                        }
                    },
                    (None, _) => rsx! {
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

    // D-03 filter semantic tests — pin show_all / is_paid / inactive behaviour.

    fn make_employee(is_paid: bool, inactive: bool, name: &str) -> Employee {
        use crate::state::shiftplan::SalesPerson;
        use std::rc::Rc;
        use uuid::Uuid;
        Employee {
            sales_person: SalesPerson {
                id: Uuid::new_v4(),
                name: name.into(),
                background_color: "#fff".into(),
                is_paid,
                inactive,
                version: Uuid::nil(),
            },
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
        }
    }

    #[test]
    fn filter_default_hides_unpaid() {
        // show_all=false must hide is_paid=false persons regardless of search
        let unpaid = make_employee(false, false, "Freiwillige");
        assert!(
            !employee_visible(&unpaid, false, ""),
            "unpaid person must not appear when show_all=false"
        );
        let paid = make_employee(true, false, "Bezahlt");
        assert!(
            employee_visible(&paid, false, ""),
            "paid person must appear when show_all=false"
        );
    }

    #[test]
    fn filter_show_all_reveals_unpaid() {
        // show_all=true must reveal is_paid=false && !inactive persons
        let unpaid = make_employee(false, false, "Freiwillige");
        assert!(
            employee_visible(&unpaid, true, ""),
            "unpaid active person must appear when show_all=true"
        );
    }

    #[test]
    fn filter_inactive_always_hidden() {
        // inactive persons must never appear, regardless of show_all or is_paid
        let inactive_paid = make_employee(true, true, "InactivePaid");
        let inactive_unpaid = make_employee(false, true, "InactiveUnpaid");
        assert!(
            !employee_visible(&inactive_paid, false, ""),
            "inactive paid person must not appear (show_all=false)"
        );
        assert!(
            !employee_visible(&inactive_paid, true, ""),
            "inactive paid person must not appear (show_all=true)"
        );
        assert!(
            !employee_visible(&inactive_unpaid, false, ""),
            "inactive unpaid person must not appear (show_all=false)"
        );
        assert!(
            !employee_visible(&inactive_unpaid, true, ""),
            "inactive unpaid person must not appear (show_all=true)"
        );
    }
}
