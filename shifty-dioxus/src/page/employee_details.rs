use std::rc::Rc;

use futures_util::StreamExt;

use crate::{
    api,
    component::{EmployeeView, TopBar},
    error::{result_handler, ShiftyError},
    js, loader,
    service::CONFIG,
    state::employee::{Employee, ExtraHours},
};
use dioxus::prelude::*;
use uuid::Uuid;

pub enum EmployeeDetailsAction {
    Update,
    DeleteExtraHour(Uuid),
    FullYear,
    UntilNow,
}

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
    let week_until = use_signal(|| week_until);
    let config = CONFIG.read().clone();
    let employee_id = match Uuid::parse_str(&props.employee_id) {
        Ok(employee_id) => employee_id,
        Err(err) => {
            return rsx! { "Invalid employee id: {err}" };
        }
    };
    let employee_resource: Signal<Option<Result<Employee, ShiftyError>>> = use_signal(|| None);
    let extra_hours_resource: Signal<Option<Result<Rc<[ExtraHours]>, ShiftyError>>> =
        use_signal(|| None);

    let cr = use_coroutine(
        move |mut rx: UnboundedReceiver<EmployeeDetailsAction>| async move {
            to_owned![employee_resource, extra_hours_resource, week_until];
            *employee_resource.write() = Some(
                loader::load_employee_details(
                    config.to_owned(),
                    *year.read(),
                    *week_until.read(),
                    employee_id,
                )
                .await,
            );
            *extra_hours_resource.write() = Some(
                loader::load_extra_hours_per_year(config.to_owned(), *year.read(), employee_id)
                    .await,
            );
            while let Some(action) = rx.next().await {
                match action {
                    EmployeeDetailsAction::Update => {
                        *employee_resource.write() = Some(
                            loader::load_employee_details(
                                config.to_owned(),
                                *year.read(),
                                *week_until.read(),
                                employee_id,
                            )
                            .await,
                        );
                        *extra_hours_resource.write() = Some(
                            loader::load_extra_hours_per_year(
                                config.to_owned(),
                                *year.read(),
                                employee_id,
                            )
                            .await,
                        );
                    }
                    EmployeeDetailsAction::DeleteExtraHour(extra_hour_id) => {
                        result_handler(
                            api::delete_extra_hour(config.to_owned(), extra_hour_id)
                                .await
                                .map_err(|err| err.into()),
                        );
                    }
                    EmployeeDetailsAction::FullYear => {
                        *employee_resource.write() = Some(
                            loader::load_employee_details(
                                config.to_owned(),
                                *year.read(),
                                53,
                                employee_id,
                            )
                            .await,
                        );
                        *extra_hours_resource.write() = Some(
                            loader::load_extra_hours_per_year(
                                config.to_owned(),
                                *year.read(),
                                employee_id,
                            )
                            .await,
                        );
                        *week_until.write() = 53u8;
                    }
                    EmployeeDetailsAction::UntilNow => {
                        let week = if *year.read() == js::get_current_year() {
                            js::get_current_week()
                        } else {
                            53
                        };
                        *employee_resource.write() = Some(
                            loader::load_employee_details(
                                config.to_owned(),
                                *year.read(),
                                week,
                                employee_id,
                            )
                            .await,
                        );
                        *extra_hours_resource.write() = Some(
                            loader::load_extra_hours_per_year(
                                config.to_owned(),
                                *year.read(),
                                employee_id,
                            )
                            .await,
                        );
                        *week_until.write() = week;
                    }
                }
            }
        },
    );

    rsx! {
        TopBar {}

        div { class: "ml-1 mr-1 pt-4 md:m-8",
            match (&*employee_resource.read_unchecked(), &*extra_hours_resource.read_unchecked()) {
                (Some(Ok(employee)), Some(Ok(extra_hours))) => {
                    rsx! {
                        EmployeeView {
                            employee: employee.clone(),
                            extra_hours: extra_hours.clone(),
                            onupdate: move |_| cr.send(EmployeeDetailsAction::Update),
                            on_extra_hour_delete: move |id| cr.send(EmployeeDetailsAction::DeleteExtraHour(id)),
                            on_full_year: move |_| cr.send(EmployeeDetailsAction::FullYear),
                            on_until_now: move |_| cr.send(EmployeeDetailsAction::UntilNow),
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
