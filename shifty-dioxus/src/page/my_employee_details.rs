use std::rc::Rc;

use futures_util::StreamExt;

use crate::{
    component::{EmployeeView, TopBar},
    error::ShiftyError,
    js, loader,
    state::{
        self,
        employee::{Employee, ExtraHours},
    },
};
use dioxus::prelude::*;

pub enum MyEmployeeDetailsAction {
    Update,
}

#[component]
pub fn MyEmployeeDetails() -> Element {
    let year = use_signal(|| 2024);
    let week_until = if *year.read() == js::get_current_year() {
        js::get_current_week()
    } else {
        52
    };
    let config = use_context::<state::Config>();
    let employee_resource: Signal<Option<Result<Employee, ShiftyError>>> = use_signal(|| None);
    let extra_hours_resource: Signal<Option<Result<Rc<[ExtraHours]>, ShiftyError>>> =
        use_signal(|| None);

    let cr = use_coroutine(
        move |mut rx: UnboundedReceiver<MyEmployeeDetailsAction>| async move {
            to_owned![employee_resource, extra_hours_resource];
            if let Ok(Some(sales_person)) = loader::load_current_sales_person(config.clone()).await
            {
                *employee_resource.write() = Some(
                    loader::load_employee_details(
                        config.to_owned(),
                        *year.read(),
                        week_until,
                        sales_person.id,
                    )
                    .await,
                );
                *extra_hours_resource.write() = Some(
                    loader::load_extra_hours_per_year(
                        config.to_owned(),
                        *year.read(),
                        sales_person.id,
                    )
                    .await,
                )
            }
            while let Some(action) = rx.next().await {
                match action {
                    MyEmployeeDetailsAction::Update => {
                        *employee_resource.write() = if let Ok(Some(sales_person)) =
                            loader::load_current_sales_person(config.clone()).await
                        {
                            let employee = loader::load_employee_details(
                                config.to_owned(),
                                *year.read(),
                                week_until,
                                sales_person.id,
                            )
                            .await;
                            Some(employee)
                        } else {
                            None
                        };
                    }
                }
            }
        },
    );

    rsx! {
        TopBar {}

        div {
            class: "ml-1 mr-1 pt-4 md:m-8",
            match (&*employee_resource.read_unchecked(), &*extra_hours_resource.read_unchecked()) {
                (Some(Ok(employee)), Some(Ok(extra_hours))) => {
                    rsx! {
                        EmployeeView {
                            employee: employee.clone(),
                            extra_hours: extra_hours.clone(),
                            onupdate: move |_| cr.send(MyEmployeeDetailsAction::Update),
                        }
                    }
                },
                (Some(Err(err)), _) => {
                    rsx! { "Error while loading employee: {err}" }
                },
                (_, Some(Err(err))) => {
                    rsx! { "Error while loading extra hours: {err}" }
                },
                (None, None) => {
                    rsx! { "Loading..." }
                }
                (None, _) => {
                    rsx! { "Loading employee." }
                }
                (_, None) => {
                    rsx! { "Loading extra hours." }
                }
            }
        }
    }
}
