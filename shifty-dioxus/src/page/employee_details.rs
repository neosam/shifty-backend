use futures_util::StreamExt;

use crate::{
    base_types::ImStr,
    component::{
        atoms::{use_media_query, Btn, BtnVariant},
        employee_work_details_form::EmployeeWorkDetailsFormType,
        error_view::ErrorView,
        ContractModal, EmployeeView, EmployeesShell, ExtraHoursModal, ManualRebookingModal, TopBar,
    },
    error::ShiftyError,
    i18n::Key,
    js,
    loader,
    router::Route,
    service::{
        auth::AUTH, config::CONFIG, employee::EmployeeAction, employee::EMPLOYEE_STORE,
        employee_work_details::EmployeeWorkDetailsAction, i18n::I18N,
    },
    state::employee::ExtraHours,
    state::rebooking::RebookingDirection,
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

    // Phase 55 (F3, REB-MANUAL-03, D-55-06): Manual-Rebooking-Modal-State.
    let mut show_manual_rebooking = use_signal(|| false);
    let mut manual_rebooking_slot_taken = use_signal(|| false);
    let mut manual_rebooking_in_flight = use_signal(|| false);

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

    // Phase 55 (F3, D-55-06): HR-gate + Person-Name/Default-KW fuer das
    // Manual-Rebooking-Modal. Der Trigger sitzt in der Action-Zone der Seite
    // (NICHT in der Voluntary-Stats-Zeile) und ist nur im HR-Report-Kontext
    // sichtbar. Kein FE-Rechnen — die Umbuchung schreibt das Backend.
    let is_hr = AUTH
        .read()
        .auth_info
        .as_ref()
        .map(|a| a.has_privilege("hr"))
        .unwrap_or(false);
    let person_name: ImStr = ImStr::from(EMPLOYEE_STORE.read().employee.sales_person.name.clone());
    let manual_trigger_label = ImStr::from(i18n.t(Key::RebookingManualTrigger).as_ref());
    let default_year = js::get_current_year();
    let default_week = js::get_current_week();

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

        // Phase 55 (F3, REB-MANUAL-03, D-55-05/06): Manual-Rebooking-Modal.
        // Submit -> submit_manual_rebooking; bei Erfolg schliessen + Report-Refetch,
        // bei 409 (belegter Slot) bleibt es offen (slot_taken_error inline).
        ManualRebookingModal {
            open: *show_manual_rebooking.read(),
            person_name: person_name.clone(),
            default_year,
            default_week,
            slot_taken_error: *manual_rebooking_slot_taken.read(),
            in_flight: *manual_rebooking_in_flight.read(),
            on_submit: move |(direction, hours, iso_year, iso_week): (RebookingDirection, f32, u32, u8)| {
                manual_rebooking_slot_taken.set(false);
                manual_rebooking_in_flight.set(true);
                let sp_id = *current_employee_id.peek();
                spawn(async move {
                    let config = CONFIG.read().clone();
                    match loader::submit_manual_rebooking(
                        config,
                        sp_id,
                        direction,
                        hours,
                        iso_year,
                        iso_week,
                    )
                    .await
                    {
                        Ok(()) => {
                            manual_rebooking_in_flight.set(false);
                            show_manual_rebooking.set(false);
                            // Report-Refetch (bestehender Loader-Pfad) — Umbuchung als Batch sichtbar.
                            employee_service.send(EmployeeAction::Refresh);
                        }
                        Err(ShiftyError::Conflict(_)) => {
                            // Belegter Slot (D-55-05): Modal bleibt offen, inline-Fehler.
                            manual_rebooking_in_flight.set(false);
                            manual_rebooking_slot_taken.set(true);
                        }
                        Err(_) => {
                            // Sonstiger Fehler: Modal offen lassen, in-flight aufheben
                            // (ErrorView surft den globalen Fehler ohnehin).
                            manual_rebooking_in_flight.set(false);
                        }
                    }
                });
            },
            on_cancel: move |_| {
                show_manual_rebooking.set(false);
                manual_rebooking_slot_taken.set(false);
                manual_rebooking_in_flight.set(false);
            },
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
                // Phase 55 (F3, D-55-06): Action-Zone der Seite — HR-gated F3-Trigger.
                // NICHT in der Voluntary-Stats-Zeile (die bleibt reine Lese-Anzeige).
                if is_hr {
                    div { class: "flex justify-end",
                        Btn {
                            variant: BtnVariant::Secondary,
                            on_click: move |_| {
                                manual_rebooking_slot_taken.set(false);
                                manual_rebooking_in_flight.set(false);
                                show_manual_rebooking.set(true);
                            },
                            "{manual_trigger_label}"
                        }
                    }
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
                    // NAV-01 Link 2 (D-26-06): HR EmployeeDetails → AbsencesFor(:id)
                    on_nav_to_absences: Some(EventHandler::new(move |_| {
                        nav.push(Route::AbsencesFor {
                            employee_id: employee_id.to_string(),
                        });
                    })),
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
