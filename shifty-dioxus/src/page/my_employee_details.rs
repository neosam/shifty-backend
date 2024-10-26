use std::rc::Rc;

use futures_util::StreamExt;
use uuid::Uuid;

use crate::{
    api,
    component::{
        employee_work_details_form::EmployeeWorkDetailsFormType, EmployeeView,
        EmployeeWorkDetailsForm, Modal, TopBar,
    },
    error::{result_handler, ShiftyError},
    js, loader,
    service::{EmployeeWorkDetailsAction, CONFIG, EMPLOYEE_WORK_DETAILS_STORE},
    state::employee::{Employee, ExtraHours},
};
use dioxus::prelude::*;

pub enum MyEmployeeDetailsAction {
    Update,
    DeleteExtraHour(Uuid),
    FullYear,
    UntilNow,
    OpenEmployeeWorkDetails(Uuid),
    CloseEmployeeWorkDetailsDialog,
}

#[component]
pub fn MyEmployeeDetails() -> Element {
    let year = use_signal(|| 2024);
    let week_until = if *year.read() == js::get_current_year() {
        js::get_current_week()
    } else {
        52
    };
    let week_until = use_signal(|| week_until);
    let config = CONFIG.read().clone();
    let employee_resource: Signal<Option<Result<Employee, ShiftyError>>> = use_signal(|| None);
    let extra_hours_resource: Signal<Option<Result<Rc<[ExtraHours]>, ShiftyError>>> =
        use_signal(|| None);
    let employee_work_details_service = use_coroutine_handle::<EmployeeWorkDetailsAction>();
    let employee_work_details_list = EMPLOYEE_WORK_DETAILS_STORE
        .read()
        .employee_work_details
        .clone();
    let mut show_add_employee_work_details_dialog = use_signal(|| false);

    let cr = use_coroutine(
        move |mut rx: UnboundedReceiver<MyEmployeeDetailsAction>| async move {
            to_owned![employee_resource, extra_hours_resource, week_until];
            if let Ok(Some(sales_person)) = loader::load_current_sales_person(config.clone()).await
            {
                *employee_resource.write() = Some(
                    loader::load_employee_details(
                        config.to_owned(),
                        *year.read(),
                        *week_until.read(),
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
                );
                employee_work_details_service
                    .send(EmployeeWorkDetailsAction::LoadForEmployee(sales_person.id));
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
                                    *week_until.read(),
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
                    MyEmployeeDetailsAction::FullYear => {
                        if let Ok(Some(sales_person)) =
                            loader::load_current_sales_person(config.clone()).await
                        {
                            *employee_resource.write() = Some(
                                loader::load_employee_details(
                                    config.to_owned(),
                                    *year.read(),
                                    53,
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
                            );
                            *week_until.write() = 53u8;
                        }
                    }
                    MyEmployeeDetailsAction::UntilNow => {
                        if let Ok(Some(sales_person)) =
                            loader::load_current_sales_person(config.clone()).await
                        {
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
                            );
                            *week_until.write() = week;
                        }
                    }
                    MyEmployeeDetailsAction::OpenEmployeeWorkDetails(id) => {
                        employee_work_details_service.send(EmployeeWorkDetailsAction::Load(id));
                        *show_add_employee_work_details_dialog.write() = true;
                    }
                    MyEmployeeDetailsAction::CloseEmployeeWorkDetailsDialog => {
                        *show_add_employee_work_details_dialog.write() = false;
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
                        if *show_add_employee_work_details_dialog.read() {
                            Modal {
                                EmployeeWorkDetailsForm {
                                    employee_work_details_form_type: EmployeeWorkDetailsFormType::ReadOnly,
                                    on_cancel: move |_| cr.send(MyEmployeeDetailsAction::CloseEmployeeWorkDetailsDialog),
                                }
                            }
                        }
                        EmployeeView {
                            employee: employee.clone(),
                            extra_hours: extra_hours.clone(),
                            employee_work_details_list,
                            show_delete_employee_work_details: false,
                            onupdate: move |_| cr.send(MyEmployeeDetailsAction::Update),
                            on_extra_hour_delete: move |uuid| cr.send(MyEmployeeDetailsAction::DeleteExtraHour(uuid)),
                            on_full_year: move |_| cr.send(MyEmployeeDetailsAction::FullYear),
                            on_until_now: move |_| cr.send(MyEmployeeDetailsAction::UntilNow),
                            on_employee_work_details_clicked: move |id| cr.send(MyEmployeeDetailsAction::OpenEmployeeWorkDetails(id)),
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
