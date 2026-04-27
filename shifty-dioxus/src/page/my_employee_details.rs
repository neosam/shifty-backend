use futures_util::StreamExt;
use uuid::Uuid;

use crate::{
    component::{
        employee_work_details_form::EmployeeWorkDetailsFormType, error_view::ErrorView,
        ContractModal, EmployeeView, ExtraHoursModal, TopBar,
    },
    service::{
        config::CONFIG,
        employee::{EmployeeAction, EMPLOYEE_STORE},
        employee_work_details::EmployeeWorkDetailsAction,
    },
};
use dioxus::prelude::*;

pub enum MyEmployeeDetailsAction {
    Update,
    DeleteExtraHour(Uuid),
    OpenEmployeeWorkDetails(Uuid),
    CloseEmployeeWorkDetailsDialog,
    OpenExtraHours,
    CloseExtraHours,
    ExtraHoursSaved,
}

#[component]
pub fn MyEmployeeDetails() -> Element {
    let employee_work_details_service = use_coroutine_handle::<EmployeeWorkDetailsAction>();
    let mut show_contract_dialog = use_signal(|| false);
    let mut show_extra_hours_dialog = use_signal(|| false);
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
                        show_contract_dialog.set(true);
                    }
                    MyEmployeeDetailsAction::CloseEmployeeWorkDetailsDialog => {
                        show_contract_dialog.set(false);
                    }
                    MyEmployeeDetailsAction::OpenExtraHours => {
                        show_extra_hours_dialog.set(true);
                    }
                    MyEmployeeDetailsAction::CloseExtraHours => {
                        show_extra_hours_dialog.set(false);
                    }
                    MyEmployeeDetailsAction::ExtraHoursSaved => {
                        show_extra_hours_dialog.set(false);
                        employee_service.send(EmployeeAction::Refresh);
                    }
                }
            }
        },
    );

    let sales_person_id = EMPLOYEE_STORE.read().employee.sales_person.id;

    rsx! {
        TopBar {}

        ErrorView {}

        ContractModal {
            open: *show_contract_dialog.read(),
            form_type: EmployeeWorkDetailsFormType::ReadOnly,
            on_save: move |_| {},
            on_cancel: move |_| cr.send(MyEmployeeDetailsAction::CloseEmployeeWorkDetailsDialog),
        }

        ExtraHoursModal {
            open: *show_extra_hours_dialog.read(),
            sales_person_id,
            on_saved: move |_| cr.send(MyEmployeeDetailsAction::ExtraHoursSaved),
            on_cancel: move |_| cr.send(MyEmployeeDetailsAction::CloseExtraHours),
        }

        div { class: "ml-1 mr-1 pt-4 md:m-8",
            EmployeeView {
                show_delete_employee_work_details: false,
                show_vacation: config.show_vacation,
                onupdate: move |_| cr.send(MyEmployeeDetailsAction::Update),
                on_extra_hour_delete: move |uuid| cr.send(MyEmployeeDetailsAction::DeleteExtraHour(uuid)),
                on_custom_delete: move |_uuid| cr.send(MyEmployeeDetailsAction::Update),
                on_employee_work_details_clicked: move |id| cr.send(MyEmployeeDetailsAction::OpenEmployeeWorkDetails(id)),
                on_open_extra_hours: Some(EventHandler::new(move |_| cr.send(MyEmployeeDetailsAction::OpenExtraHours))),
            }
        }
    }
}
