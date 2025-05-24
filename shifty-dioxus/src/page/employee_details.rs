use futures_util::StreamExt;

use crate::{
    component::{
        employee_work_details_form::EmployeeWorkDetailsFormType, error_view::ErrorView, EmployeeView, EmployeeWorkDetailsForm, Modal, TopBar
    },
    service::{employee::EmployeeAction, employee_work_details::EmployeeWorkDetailsAction},
};
use dioxus::prelude::*;
use uuid::Uuid;

pub enum EmployeeDetailsAction {
    Update,
    DeleteExtraHour(Uuid),
    EmployeeWorkDetailsDialogVisibility(bool),
    NewEmployeeWorkDetails,
    OpenEmployeeWorkDetails(Uuid),
    EmployeeWorkDetailsSaved,
}

#[derive(Clone, PartialEq, Props)]
pub struct EmployeeDetailsProps {
    pub employee_id: String,
}

#[component]
pub fn EmployeeDetails(props: EmployeeDetailsProps) -> Element {
    let employee_id = match Uuid::parse_str(&props.employee_id) {
        Ok(employee_id) => employee_id,
        Err(err) => {
            return rsx! { "Invalid employee id: {err}" };
        }
    };
    let show_add_employee_work_details_dialog = use_signal(|| false);
    let mut employee_work_details_dialog_type = use_signal(|| EmployeeWorkDetailsFormType::New);
    let employee_service = use_coroutine_handle::<EmployeeAction>();

    let employee_work_details_service = use_coroutine_handle::<EmployeeWorkDetailsAction>();

    let cr = use_coroutine(
        move |mut rx: UnboundedReceiver<EmployeeDetailsAction>| async move {
            to_owned![show_add_employee_work_details_dialog];
            while let Some(action) = rx.next().await {
                match action {
                    EmployeeDetailsAction::Update => {
                        employee_service.send(EmployeeAction::Refresh);
                    }
                    EmployeeDetailsAction::DeleteExtraHour(extra_hour_id) => {
                        employee_service.send(EmployeeAction::DeleteExtraHours(extra_hour_id));
                    }
                    EmployeeDetailsAction::EmployeeWorkDetailsDialogVisibility(visible) => {
                        *show_add_employee_work_details_dialog.write() = visible;
                    }
                    EmployeeDetailsAction::NewEmployeeWorkDetails => {
                        employee_work_details_service
                            .send(EmployeeWorkDetailsAction::NewWorkingHours(employee_id));
                        *show_add_employee_work_details_dialog.write() = true;
                        *employee_work_details_dialog_type.write() =
                            EmployeeWorkDetailsFormType::New;
                    }
                    EmployeeDetailsAction::OpenEmployeeWorkDetails(id) => {
                        employee_work_details_service.send(EmployeeWorkDetailsAction::Load(id));
                        *show_add_employee_work_details_dialog.write() = true;
                        *employee_work_details_dialog_type.write() =
                            EmployeeWorkDetailsFormType::Edit;
                    }
                    EmployeeDetailsAction::EmployeeWorkDetailsSaved => {
                        *show_add_employee_work_details_dialog.write() = false;
                    }
                }
            }
        },
    );
    use_effect(move || {
        employee_service.send(EmployeeAction::LoadEmployeeDataUntilNow {
            sales_person_id: employee_id,
        })
    });

    rsx! {
        TopBar {}
        ErrorView {}

        div { class: "ml-1 mr-1 pt-4 md:m-8",
            if *show_add_employee_work_details_dialog.read() {
                Modal {
                    EmployeeWorkDetailsForm {
                        employee_work_details_form_type: *employee_work_details_dialog_type.read(),
                        on_save: move |_| cr.send(EmployeeDetailsAction::EmployeeWorkDetailsSaved),
                        on_cancel: move |_| cr.send(EmployeeDetailsAction::EmployeeWorkDetailsDialogVisibility(false)),
                    }
                }
            }
            EmployeeView {
                onupdate: move |_| cr.send(EmployeeDetailsAction::Update),
                show_vacation: true,
                show_delete_employee_work_details: true,
                on_extra_hour_delete: move |id| cr.send(EmployeeDetailsAction::DeleteExtraHour(id)),
                on_custom_delete: move |_id| cr.send(EmployeeDetailsAction::Update),
                on_add_employee_work_details: move |_| cr.send(EmployeeDetailsAction::NewEmployeeWorkDetails),
                on_employee_work_details_clicked: move |id| cr.send(EmployeeDetailsAction::OpenEmployeeWorkDetails(id)),
                on_delete_employee_work_details_clicked: move |_id| cr.send(EmployeeDetailsAction::Update),
            }
        }
    }
}
