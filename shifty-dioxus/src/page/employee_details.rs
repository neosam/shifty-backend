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
    state::employee::ExtraHours,
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
    OpenEditExtraHours(ExtraHours),
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
    let mut editing_extra_hours = use_signal(|| None::<ExtraHours>);

    // Mirror the route-driven `employee_id` into a signal so the coroutine
    // below reads the CURRENTLY displayed employee at dispatch time. The
    // `use_coroutine` closure captures its environment once on first mount and
    // is never re-run, so a plain `let employee_id` capture would freeze to the
    // first-loaded employee in the master/detail layout (where this component
    // stays mounted across employee switches). Reading through a signal avoids
    // that stale-capture bug (otherwise a newly created contract would be
    // assigned to the first-opened employee instead of the displayed one).
    //
    // This signal doubles as the load gate: whenever the route prop changes we
    // forward the new id and (re)trigger the employee-data load. The
    // `loaded_once` flag distinguishes "never loaded" from "id changed".
    let mut current_employee_id = use_signal(|| employee_id);
    let mut loaded_once = use_signal(|| false);

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
                        employee_work_details_service.send(
                            EmployeeWorkDetailsAction::NewWorkingHours(*current_employee_id.peek()),
                        );
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
                        editing_extra_hours.set(None);
                        show_extra_hours_dialog.set(true);
                    }
                    EmployeeDetailsAction::OpenEditExtraHours(entry) => {
                        editing_extra_hours.set(Some(entry));
                        show_extra_hours_dialog.set(true);
                    }
                    EmployeeDetailsAction::CloseExtraHours => {
                        show_extra_hours_dialog.set(false);
                        editing_extra_hours.set(None);
                    }
                    EmployeeDetailsAction::ExtraHoursSaved => {
                        show_extra_hours_dialog.set(false);
                        editing_extra_hours.set(None);
                        employee_service.send(EmployeeAction::Refresh);
                    }
                }
            }
        },
    );

    // Sync the route-driven `employee_id` prop into `current_employee_id` so
    // that switching between sales persons (which keeps the same component
    // mounted but with a different prop) reliably retriggers the load AND keeps
    // the coroutine's view of "which employee is displayed" current. `use_effect`
    // only re-runs on reactive-state changes, and a plain `let` capture is not
    // reactive — so we forward prop changes into the signal here.
    if !*loaded_once.peek() || *current_employee_id.peek() != employee_id {
        loaded_once.set(true);
        current_employee_id.set(employee_id);
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

        if *show_extra_hours_dialog.read() {
            ExtraHoursModal {
                open: true,
                sales_person_id: employee_id,
                editing: editing_extra_hours.read().clone(),
                on_saved: move |_| cr.send(EmployeeDetailsAction::ExtraHoursSaved),
                on_cancel: move |_| cr.send(EmployeeDetailsAction::CloseExtraHours),
            }
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
                // NAV-01 Link 2 (D-26-06): HR EmployeeDetails → AbsencesFor(:id)
                Btn {
                    variant: BtnVariant::Ghost,
                    on_click: move |_| {
                        nav.push(Route::AbsencesFor {
                            employee_id: employee_id.to_string(),
                        });
                    },
                    "{i18n.t(Key::NavToEmployeeAbsences)}"
                }
                EmployeeView {
                    onupdate: move |_| cr.send(EmployeeDetailsAction::Update),
                    show_vacation: true,
                    show_delete_employee_work_details: true,
                    on_extra_hour_delete: move |id| cr.send(EmployeeDetailsAction::DeleteExtraHour(id)),
                    on_extra_hour_edit: move |entry: ExtraHours| cr.send(EmployeeDetailsAction::OpenEditExtraHours(entry)),
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
    use dioxus::prelude::*;
    use std::cell::RefCell;
    use uuid::Uuid;

    thread_local! {
        /// Captures, on every render of the inner Detail component, what an
        /// event handler dispatched THROUGH THE SIGNAL would read at that point.
        /// This mirrors what `NewEmployeeWorkDetails` sends as the sales_person_id.
        static SIGNAL_READ: RefCell<Option<Uuid>> = const { RefCell::new(None) };
        /// What a plain (frozen) first-mount capture would have read — kept for
        /// contrast so the assertion documents the bug it guards against.
        static FROZEN_CAPTURE: RefCell<Option<Uuid>> = const { RefCell::new(None) };
    }

    /// Drives the harness prop reactively to simulate switching the displayed
    /// employee while the Detail component stays mounted (master/detail layout).
    static DISPLAYED_ID: GlobalSignal<Uuid> = Signal::global(|| Uuid::from_u128(0x1111));

    /// Regression test for "Working Hours dem falschen Mitarbeiter zugeordnet".
    ///
    /// Root cause: in the master/detail layout `EmployeeDetails` stays mounted
    /// while the route prop `employee_id` changes. The `use_coroutine` closure
    /// captures its environment ONCE on first mount and never re-runs, so a
    /// plain `let employee_id` capture freezes to the first-opened employee.
    /// `NewEmployeeWorkDetails` then created the contract for that stale id.
    ///
    /// The fix mirrors the prop into a signal (`current_employee_id`) that is
    /// re-synced on every render and read via `.peek()` at dispatch time.
    ///
    /// This test reproduces the mount-stable prop switch and asserts that the
    /// signal-read tracks the CURRENT prop (the fix), while the frozen capture
    /// stays on the first id (the bug).
    #[test]
    fn signal_mirror_tracks_current_employee_while_frozen_capture_is_stale() {
        let first = Uuid::from_u128(0x1111);
        let second = Uuid::from_u128(0x2222);

        #[derive(Props, Clone, PartialEq)]
        struct DetailProps {
            employee_id: Uuid,
        }

        #[allow(non_snake_case)]
        fn Detail(props: DetailProps) -> Element {
            let employee_id = props.employee_id;

            // The fix: prop mirrored into a signal, re-synced every render.
            let mut current_employee_id = use_signal(|| employee_id);
            let mut loaded_once = use_signal(|| false);

            // Simulate the (buggy) first-mount frozen capture: record the prop
            // value as seen on the very first render only.
            if !*loaded_once.peek() {
                FROZEN_CAPTURE.with(|f| *f.borrow_mut() = Some(employee_id));
            }

            if !*loaded_once.peek() || *current_employee_id.peek() != employee_id {
                loaded_once.set(true);
                current_employee_id.set(employee_id);
            }

            // What `NewEmployeeWorkDetails` would dispatch now (reads signal).
            SIGNAL_READ.with(|s| *s.borrow_mut() = Some(*current_employee_id.peek()));

            rsx! {}
        }

        #[allow(non_snake_case)]
        fn Root() -> Element {
            // Reading the reactive GlobalSignal each render lets the test flip
            // the displayed employee while Detail stays mounted, and Dioxus
            // re-renders Detail with the new prop.
            let displayed = *DISPLAYED_ID.read();
            rsx! {
                Detail { employee_id: displayed }
            }
        }

        let mut vdom = VirtualDom::new(Root);
        vdom.rebuild_in_place();

        // After first mount: both read the first employee.
        assert_eq!(SIGNAL_READ.with(|s| *s.borrow()), Some(first));
        assert_eq!(FROZEN_CAPTURE.with(|f| *f.borrow()), Some(first));

        // Switch displayed employee A -> B (Detail stays mounted). Mutating the
        // GlobalSignal must happen inside the VirtualDom runtime.
        vdom.in_runtime(|| {
            *DISPLAYED_ID.write() = second;
        });
        vdom.render_immediate(&mut dioxus_core::NoOpMutations);

        // The fix: signal-read now reflects the CURRENT employee.
        assert_eq!(
            SIGNAL_READ.with(|s| *s.borrow()),
            Some(second),
            "signal-mirrored read must follow the displayed employee after a mount-stable prop switch"
        );
        // The bug it guards against: a frozen first-mount capture stays stale.
        assert_eq!(
            FROZEN_CAPTURE.with(|f| *f.borrow()),
            Some(first),
            "frozen first-mount capture stays on the first id — this is exactly the stale-capture bug the fix avoids"
        );
    }

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
