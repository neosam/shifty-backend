use crate::component::{EmployeesShell, TopBar};
use dioxus::prelude::*;

#[component]
pub fn Employees() -> Element {
    rsx! {
        TopBar {}
        EmployeesShell {
            div { class: "hidden md:flex h-full items-center justify-center text-ink-muted text-body p-6",
                "Wähle einen Mitarbeiter aus der Liste"
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
        let src = include_str!("employees.rs");
        let test_module_start = src
            .find("#[cfg(test)]")
            .expect("test module marker missing");
        let prefix = &src[..test_module_start];
        for forbidden in ["BillingPeriod", "BILLING_PERIOD", "Modal", "billing_period"] {
            assert!(
                !prefix.contains(forbidden),
                "billing-period reference `{forbidden}` still in employees.rs"
            );
        }
    }
}
