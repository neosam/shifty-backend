use crate::{
    component::{EmployeeShort, TopBar},
    js, loader,
    router::Route,
    service::CONFIG,
};
use dioxus::prelude::*;

#[component]
pub fn Employees() -> Element {
    let year = use_signal(|| 2024);
    let week_until = if *year.read() == js::get_current_year() {
        js::get_current_week()
    } else {
        52
    };
    let config = CONFIG.read().clone();
    let employees =
        use_resource(move || loader::load_employees(config.to_owned(), *year.read(), week_until));

    rsx! {
        TopBar {}

        div { class: "ml-1 mr-1 pt-4 md:m-8",
            match &*employees.read_unchecked() {
                Some(Ok(employee)) => {
                    let mut employee = employee.iter().cloned().collect::<Vec<_>>();
                    employee.sort_by(|a, b| a.sales_person.name.cmp(&b.sales_person.name));
                    rsx! {
                        for employee in employee.iter() {
                            Link {
                                to: Route::EmployeeDetails {
                                    employee_id: employee.sales_person.id.to_string()
                                },
                                EmployeeShort {
                                    employee: employee.clone()
                                }
                            }
                        }
                    }
                },
                Some(Err(err)) => {
                    rsx! { "Error while loading employees: {err}" }
                },
                None => {
                    rsx! { "Loading employees." }
                }
            }
        }
    }
}
