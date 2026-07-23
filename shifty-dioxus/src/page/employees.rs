//! Employee-Overview-Landing (HR-Path).
//!
//! Phase 55 (F5, HR-ALERT-01, D-55-01/07): Auf der Overview-Landing rendert pro
//! `ShortEmployeeReportTO` mit backend-gesetztem `has_pending_rebooking == true`
//! ein klickbarer `RebookingAlertBanner`. Klick loest `open_rebooking_suggestion`
//! (POST-Claim, Modell B) aus und oeffnet das `RebookingSuggestionModal` mit der
//! zurueckgelieferten Vorschau. Approve/Reject refetchen die Overview (Banner
//! verschwindet, sobald der Predicate `false` ist).
//!
//! **Fat Backend, Thin Client (bindend):** Sichtbarkeit + IST/DANN-Werte sind
//! backend-computed; hier kein FE-Rechnen, kein FE-Rollen-Check. Die Sichtbarkeit
//! haengt allein am DTO-Flag (Non-HR liefert es API-redigiert als `false`).

use dioxus::prelude::*;
use uuid::Uuid;

use crate::base_types::ImStr;
use crate::component::{
    EmployeesShell, RebookingAlertBanner, RebookingSuggestionModal, TopBar,
};
use crate::error::ShiftyError;
use crate::js;
use crate::loader;
use crate::service::config::CONFIG;
use crate::service::employee::EMPLOYEES_LIST_REFRESH;
use crate::state::employee::Employee;
use crate::state::rebooking::RebookingSuggestion;

#[component]
pub fn Employees() -> Element {
    // Overview-Report-Resource (Praezedenz EmployeesList): traegt pro SalesPerson
    // das backend-berechnete `has_pending_rebooking`-Flag. Subscribed auf den
    // Sidebar-Refresh-Token, damit ein Approve/Reject die Liste re-fetcht.
    let year = use_signal(js::get_current_year);
    let week_until = if *year.read() == js::get_current_year() {
        js::get_current_week()
    } else {
        52
    };
    let config = CONFIG.read().clone();
    let mut reports = use_resource(move || {
        let _refresh_token = *EMPLOYEES_LIST_REFRESH.read();
        loader::load_employees(config.to_owned(), *year.read(), week_until)
    });

    // Suggestion-Modal-State: offene Vorschau + Person-Name + in-flight/error.
    let mut modal_open = use_signal(|| false);
    let mut active_suggestion = use_signal(|| None::<RebookingSuggestion>);
    let mut active_person = use_signal(|| ImStr::from(""));
    let mut in_flight = use_signal(|| false);
    let mut had_error = use_signal(|| false);

    // Banner-Klick (F5-Modell B): POST-Claim → Suggestion. Bei 409 (entschiedener
    // Slot) das Modal NICHT oeffnen, sondern Overview refetchen (Banner weg).
    let open_suggestion = move |(sales_person_id, person_name): (Uuid, ImStr)| {
        active_person.set(person_name);
        had_error.set(false);
        in_flight.set(true);
        modal_open.set(true);
        spawn(async move {
            let config = CONFIG.read().clone();
            let cur_year = js::get_current_year();
            let cur_week = js::get_current_week();
            match loader::open_rebooking_suggestion(config, sales_person_id, cur_year, cur_week)
                .await
            {
                Ok(suggestion) => {
                    active_suggestion.set(Some(suggestion));
                    in_flight.set(false);
                }
                Err(ShiftyError::Conflict(_)) => {
                    // Slot inzwischen entschieden → kein Modal, Overview refetchen.
                    modal_open.set(false);
                    in_flight.set(false);
                    reports.restart();
                }
                Err(_) => {
                    // Netzwerk-/sonstiger Fehler: Empty-State + generischer Fehler.
                    active_suggestion.set(None);
                    had_error.set(true);
                    in_flight.set(false);
                }
            }
        });
    };

    let close_modal = move |_| {
        modal_open.set(false);
        active_suggestion.set(None);
        had_error.set(false);
        in_flight.set(false);
    };

    // Approve/Reject: Loader dispatchen; bei Erfolg Modal schliessen + Overview
    // refetchen (Banner verschwindet). Bei 409 (Race) generischer Fehler, danach
    // Modal schliessen + refetch (D-55-07 / Pitfall 12).
    let approve = move |suggestion: RebookingSuggestion| {
        in_flight.set(true);
        had_error.set(false);
        spawn(async move {
            let config = CONFIG.read().clone();
            match loader::approve_rebooking_suggestion(
                config,
                suggestion.batch_id,
                suggestion.proposed_hours,
            )
            .await
            {
                Ok(()) => {
                    in_flight.set(false);
                    modal_open.set(false);
                    active_suggestion.set(None);
                    reports.restart();
                }
                Err(_) => {
                    // Race verloren / Fehler: inline generisch, dann refetchen.
                    had_error.set(true);
                    in_flight.set(false);
                    reports.restart();
                }
            }
        });
    };

    let reject = move |suggestion: RebookingSuggestion| {
        in_flight.set(true);
        had_error.set(false);
        spawn(async move {
            let config = CONFIG.read().clone();
            match loader::reject_rebooking_suggestion(config, suggestion.batch_id).await {
                Ok(()) => {
                    in_flight.set(false);
                    modal_open.set(false);
                    active_suggestion.set(None);
                    reports.restart();
                }
                Err(_) => {
                    had_error.set(true);
                    in_flight.set(false);
                    reports.restart();
                }
            }
        });
    };

    // Nur SalesPersons mit backend-gesetztem Alert-Flag (natuerliche Report-Reihenfolge).
    let pending: Vec<Employee> = match &*reports.read_unchecked() {
        Some(Ok(list)) => list
            .iter()
            .filter(|e| e.has_pending_rebooking)
            .cloned()
            .collect(),
        _ => Vec::new(),
    };

    rsx! {
        TopBar {}

        RebookingSuggestionModal {
            open: *modal_open.read(),
            person_name: active_person.read().clone(),
            suggestion: *active_suggestion.read(),
            in_flight: *in_flight.read(),
            had_error: *had_error.read(),
            on_approve: approve,
            on_reject: reject,
            on_close: close_modal,
        }

        EmployeesShell {
            if pending.is_empty() {
                div { class: "hidden md:flex h-full items-center justify-center text-ink-muted text-body p-6",
                    "Wähle einen Mitarbeiter aus der Liste"
                }
            } else {
                // HR-Alert-Banner-Stack (F5): pro betroffenem SalesPerson eine Zeile.
                div { class: "flex flex-col gap-3 p-4 md:p-6",
                    for employee in pending.into_iter() {
                        {
                            let sp_id = employee.sales_person.id;
                            let person_name: ImStr = ImStr::from(employee.sales_person.name.clone());
                            let banner_name = person_name.clone();
                            let pending_id = employee.pending_rebooking_id;
                            let mut open_suggestion = open_suggestion;
                            rsx! {
                                RebookingAlertBanner {
                                    has_pending_rebooking: employee.has_pending_rebooking,
                                    person_name: banner_name,
                                    balance: employee.balance,
                                    voluntary: employee.volunteer_hours,
                                    pending_rebooking_id: pending_id,
                                    on_open: move |_id: Option<Uuid>| {
                                        open_suggestion((sp_id, person_name.clone()));
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("employees.rs");
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

    /// Guard: the HR-alert banner must NOT gate on a FE role-check — visibility
    /// is driven solely by the backend `has_pending_rebooking` DTO flag
    /// (T-55-16 mitigation, D-55-01). A `has_privilege(` call in the banner path
    /// would reintroduce FE-side authority.
    #[test]
    fn banner_visibility_has_no_fe_role_check() {
        let src = include_str!("employees.rs");
        let test_module_start = src
            .find("#[cfg(test)]")
            .expect("test module marker missing");
        let prefix = &src[..test_module_start];
        assert!(
            !prefix.contains("has_privilege"),
            "banner visibility must depend on has_pending_rebooking, not a FE role-check"
        );
        assert!(
            prefix.contains("has_pending_rebooking"),
            "banner path must gate on the backend has_pending_rebooking flag"
        );
    }
}
