use std::rc::Rc;

use futures_util::StreamExt;
use uuid::Uuid;

use crate::{
    api,
    component::{EmployeeView, TopBar},
    error::{result_handler, ShiftyError},
    js, loader,
    state::{
        self,
        employee::{Employee, ExtraHours},
    },
};
use dioxus::prelude::*;

pub enum MyEmployeeDetailsAction {
    Update,
    DeleteExtraHour(Uuid),
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
                        if let Ok(Some(sales_person)) =
                            loader::load_current_sales_person(config.clone()).await
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
                    }
                    MyEmployeeDetailsAction::DeleteExtraHour(extra_hour_id) => {
                        result_handler(
                            api::delete_extra_hour(config.to_owned(), extra_hour_id)
                                .await
                                .map_err(|err| err.into()),
                        );
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
                            on_extra_hour_delete: move |uuid| cr.send(MyEmployeeDetailsAction::DeleteExtraHour(uuid)),
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
