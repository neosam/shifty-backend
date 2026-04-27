use futures_util::StreamExt;

use crate::{
    base_types::ImStr,
    component::{
        atoms::{use_media_query, Btn, BtnVariant},
        employee_work_details_form::EmployeeWorkDetailsFormType,
        error_view::ErrorView,
        ContractModal, EmployeeView, EmployeesShell, ExtraHoursModal, TopBar,
    },
    i18n::Key,
    router::Route,
    service::{
        employee::EmployeeAction, employee_work_details::EmployeeWorkDetailsAction, i18n::I18N,
    },
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
    OpenExtraHours,
    CloseExtraHours,
    ExtraHoursSaved,
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

    let mut show_contract_dialog = use_signal(|| false);
    let mut contract_dialog_type = use_signal(|| EmployeeWorkDetailsFormType::New);
    let mut show_extra_hours_dialog = use_signal(|| false);

    let employee_service = use_coroutine_handle::<EmployeeAction>();
    let employee_work_details_service = use_coroutine_handle::<EmployeeWorkDetailsAction>();
    let i18n = I18N.read().clone();
    let nav = use_navigator();

    let cr = use_coroutine(
        move |mut rx: UnboundedReceiver<EmployeeDetailsAction>| async move {
            while let Some(action) = rx.next().await {
                match action {
                    EmployeeDetailsAction::Update => {
                        employee_service.send(EmployeeAction::Refresh);
                    }
                    EmployeeDetailsAction::DeleteExtraHour(extra_hour_id) => {
                        employee_service.send(EmployeeAction::DeleteExtraHours(extra_hour_id));
                    }
                    EmployeeDetailsAction::EmployeeWorkDetailsDialogVisibility(visible) => {
                        show_contract_dialog.set(visible);
                    }
                    EmployeeDetailsAction::NewEmployeeWorkDetails => {
                        employee_work_details_service
                            .send(EmployeeWorkDetailsAction::NewWorkingHours(employee_id));
                        contract_dialog_type.set(EmployeeWorkDetailsFormType::New);
                        show_contract_dialog.set(true);
                    }
                    EmployeeDetailsAction::OpenEmployeeWorkDetails(id) => {
                        employee_work_details_service.send(EmployeeWorkDetailsAction::Load(id));
                        contract_dialog_type.set(EmployeeWorkDetailsFormType::Edit);
                        show_contract_dialog.set(true);
                    }
                    EmployeeDetailsAction::EmployeeWorkDetailsSaved => {
                        show_contract_dialog.set(false);
                    }
                    EmployeeDetailsAction::OpenExtraHours => {
                        show_extra_hours_dialog.set(true);
                    }
                    EmployeeDetailsAction::CloseExtraHours => {
                        show_extra_hours_dialog.set(false);
                    }
                    EmployeeDetailsAction::ExtraHoursSaved => {
                        show_extra_hours_dialog.set(false);
                        employee_service.send(EmployeeAction::Refresh);
                    }
                }
            }
        },
    );

    // Sync the route-driven `employee_id` prop into a signal so that switching
    // between sales persons (which keeps the same component mounted but with a
    // different prop) reliably retriggers the load. `use_effect` only re-runs
    // on reactive-state changes, and a plain `let` capture is not reactive —
    // so we forward prop changes into a signal here.
    let mut last_loaded_id = use_signal(|| None::<Uuid>);
    if last_loaded_id.peek().as_ref() != Some(&employee_id) {
        last_loaded_id.set(Some(employee_id));
        employee_service.send(EmployeeAction::LoadEmployeeDataUntilNow {
            sales_person_id: employee_id,
        });
    }

    let is_mobile = *use_media_query("(max-width: 720px)").read();
    let back_label = ImStr::from(i18n.t(Key::BackToList).as_ref());

    rsx! {
        TopBar {}
        ErrorView {}

        ContractModal {
            open: *show_contract_dialog.read(),
            form_type: *contract_dialog_type.read(),
            on_save: move |_| {
                let kind = *contract_dialog_type.read();
                match kind {
                    EmployeeWorkDetailsFormType::New => {
                        employee_work_details_service.send(EmployeeWorkDetailsAction::Save);
                    }
                    EmployeeWorkDetailsFormType::Edit => {
                        employee_work_details_service.send(EmployeeWorkDetailsAction::Update);
                    }
                    EmployeeWorkDetailsFormType::ReadOnly => {}
                }
                cr.send(EmployeeDetailsAction::EmployeeWorkDetailsSaved);
            },
            on_cancel: move |_| cr.send(EmployeeDetailsAction::EmployeeWorkDetailsDialogVisibility(false)),
        }

        ExtraHoursModal {
            open: *show_extra_hours_dialog.read(),
            sales_person_id: employee_id,
            on_saved: move |_| cr.send(EmployeeDetailsAction::ExtraHoursSaved),
            on_cancel: move |_| cr.send(EmployeeDetailsAction::CloseExtraHours),
        }

        EmployeesShell {
            div { class: "px-4 py-4 md:px-8 md:py-6 flex flex-col gap-4",
                if is_mobile {
                    Btn {
                        variant: BtnVariant::Ghost,
                        icon: Some(ImStr::from("‹")),
                        on_click: move |_| {
                            nav.push(Route::Employees {});
                        },
                        "{back_label}"
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
                    on_open_extra_hours: Some(EventHandler::new(move |_| cr.send(EmployeeDetailsAction::OpenExtraHours))),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("employee_details.rs");
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
}
