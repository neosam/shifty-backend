use crate::component::{EmployeesShell, RebookingSuggestionModal, TopBar};
use crate::loader::load_rebooking_suggestions_pending;
use crate::service::config::CONFIG;
use crate::service::employee::{EmployeeAction, EMPLOYEES_LIST_REFRESH};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn Employees() -> Element {
    // Phase 55 (HR-ALERT-01, D-55-02, D-55-07): der Banner-Klick oeffnet das
    // Suggestion-Modal fuer die uebergebene batch_id. Das Modal laedt die
    // konkrete Suggestion via `load_rebooking_suggestions_pending` (Loader
    // aus Plan 55-04) und filtert im FE per `find` — kein neuer Endpoint
    // (siehe Plan 55-05 action-Doku).
    let mut open_suggestion_batch_id: Signal<Option<Uuid>> = use_signal(|| None);
    let employee_service = use_coroutine_handle::<EmployeeAction>();

    // Suggestion-Resource: laedt alle pending Suggestions und filtert per
    // batch_id. `open_suggestion_batch_id` als Signal ist die einzige
    // Dependency — schliessen des Modals setzt es auf `None` und die
    // Resource loest sich auf.
    let suggestion_resource = use_resource(move || {
        let selected = *open_suggestion_batch_id.read();
        async move {
            let batch_id = selected?;
            let config = CONFIG.read().clone();
            match load_rebooking_suggestions_pending(config).await {
                Ok(list) => list.iter().find(|s| s.batch_id == batch_id).cloned(),
                Err(_) => None,
            }
        }
    });

    let on_banner_click = move |batch_id: Uuid| {
        open_suggestion_batch_id.set(Some(batch_id));
    };

    let on_close = move |_| {
        open_suggestion_batch_id.set(None);
    };

    let on_after_action = move |_| {
        // D-55-07: nach Approve/Reject die Employees-Liste refreshen — der
        // Backend-Flag `has_pending_rebooking` wird `false`, das Banner
        // verschwindet automatisch beim naechsten Render.
        open_suggestion_batch_id.set(None);
        employee_service.send(EmployeeAction::Refresh);
        // Explizit den Sidebar-Refresh-Token bumpen, damit `use_resource`
        // in `EmployeesList` neu lauft (die Coroutine bumpt ihn ebenfalls,
        // aber diese Zeile ist eine klare, redundant-sichere Signalisierung
        // fuer den Fall, dass der Refresh-Pfad in Zukunft geaendert wird).
        *EMPLOYEES_LIST_REFRESH.write() += 1;
    };

    rsx! {
        TopBar {}
        EmployeesShell {
            on_banner_click: on_banner_click,
            div { class: "hidden md:flex h-full items-center justify-center text-ink-muted text-body p-6",
                "Wähle einen Mitarbeiter aus der Liste"
            }
        }
        // Suggestion-Overlay: erscheint sobald eine batch_id gewaehlt ist
        // UND die Resource ein Suggestion-Objekt geliefert hat. Wenn die
        // Suggestion nicht mehr existiert (batch zwischenzeitlich approved
        // oder rejected), schliessen wir und triggern einen Refresh.
        if let Some(Some(suggestion)) = &*suggestion_resource.read_unchecked() {
            RebookingSuggestionModal {
                suggestion: suggestion.clone(),
                on_close: on_close,
                on_after_action: on_after_action,
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

    #[test]
    fn no_billing_period_content_in_source() {
        // Phase 55 (HR-ALERT-01, D-55-02) hat die Verbotsliste praezisiert:
        // frueher stand `"Modal"` in der Liste, das aber legitime
        // Rebooking-Modal-Referenzen (RebookingSuggestionModal) blockieren
        // wuerde. Wir behalten die BillingPeriod-Guards und ersetzen den
        // breiten `"Modal"`-Guard durch die spezifische Kombination
        // `"BillingPeriodModal"` — die einzige Konstruktions-Form, die als
        // Regression tatsaechlich BillingPeriod-Teile in diese Seite
        // zurueckbringen wuerde.
        let src = include_str!("employees.rs");
        let test_module_start = src
            .find("#[cfg(test)]")
            .expect("test module marker missing");
        let prefix = &src[..test_module_start];
        for forbidden in [
            "BillingPeriod",
            "BILLING_PERIOD",
            "BillingPeriodModal",
            "billing_period",
        ] {
            assert!(
                !prefix.contains(forbidden),
                "billing-period reference `{forbidden}` still in employees.rs"
            );
        }
    }
}
