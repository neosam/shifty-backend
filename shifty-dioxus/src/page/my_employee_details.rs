use futures_util::StreamExt;
use uuid::Uuid;

use crate::{
    component::{
        employee_work_details_form::EmployeeWorkDetailsFormType, EmployeeView,
        EmployeeWorkDetailsForm, Modal, TopBar,
    },
    service::{EmployeeAction, EmployeeWorkDetailsAction, CONFIG},
};
use dioxus::prelude::*;

pub enum MyEmployeeDetailsAction {
    Update,
    DeleteExtraHour(Uuid),
    OpenEmployeeWorkDetails(Uuid),
    CloseEmployeeWorkDetailsDialog,
}

#[component]
pub fn MyEmployeeDetails() -> Element {
    let employee_work_details_service = use_coroutine_handle::<EmployeeWorkDetailsAction>();
    let mut show_add_employee_work_details_dialog = use_signal(|| false);
    let config = CONFIG.read().clone();

    let employee_service = use_coroutine_handle::<EmployeeAction>();

    let cr = use_coroutine(
        move |mut rx: UnboundedReceiver<MyEmployeeDetailsAction>| async move {
            employee_service.send(EmployeeAction::LoadCurrentEmployeeDataUntilNow);
            while let Some(action) = rx.next().await {
                match action {
                    MyEmployeeDetailsAction::Update => {
                        employee_service.send(EmployeeAction::Refresh);
                    }
                    MyEmployeeDetailsAction::DeleteExtraHour(extra_hour_id) => {
                        employee_service.send(EmployeeAction::DeleteExtraHours(extra_hour_id));
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
            if *show_add_employee_work_details_dialog.read() {
                Modal {
                    EmployeeWorkDetailsForm {
                        employee_work_details_form_type: EmployeeWorkDetailsFormType::ReadOnly,
                        on_cancel: move |_| cr.send(MyEmployeeDetailsAction::CloseEmployeeWorkDetailsDialog)
                    }
                }
            }
            EmployeeView {
                show_delete_employee_work_details: false,
                show_vacation: config.show_vacation,
                onupdate: move |_| cr.send(MyEmployeeDetailsAction::Update),
                on_extra_hour_delete: move |uuid| cr.send(MyEmployeeDetailsAction::DeleteExtraHour(uuid)),
                on_employee_work_details_clicked: move |id| cr.send(MyEmployeeDetailsAction::OpenEmployeeWorkDetails(id))
            }
        }
    }
}
