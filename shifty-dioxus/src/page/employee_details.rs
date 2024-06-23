use std::rc::Rc;

use crate::{
    component::{EmployeeShort, EmployeeView, TopBar},
    js, loader,
    state::{
        self,
        employee::{self, Employee},
    },
};
use dioxus::prelude::*;
use uuid::Uuid;

#[derive(Clone, PartialEq, Props)]
pub struct EmployeeDetailsProps {
    pub employee_id: String,
}

#[component]
pub fn EmployeeDetails(props: EmployeeDetailsProps) -> Element {
    let year = use_signal(|| 2024);
    let week_until = if *year.read() == js::get_current_year() {
        js::get_current_week()
    } else {
        52
    };
    let config = use_context::<state::Config>();
    let employee_id = match Uuid::parse_str(&props.employee_id) {
        Ok(employee_id) => employee_id,
        Err(err) => {
            return rsx! { "Invalid employee id: {err}" };
        }
    };
    let employee = use_resource(move || {
        loader::load_employee_details(config.to_owned(), *year.read(), week_until, employee_id)
    });

    rsx! {
        TopBar {}

        div {
            class: "ml-1 mr-1 pt-4 md:m-8",
            match &*employee.read_unchecked() {
                Some(Ok(employee)) => {
                    rsx! {
                        EmployeeView {
                            employee: employee.clone()
                        }
                    }
                },
                Some(Err(err)) => {
                    rsx! { "Error while loading employee: {err}" }
                },
                None => {
                    rsx! { "Loading employee." }
                }
            }
        }
    }
}
