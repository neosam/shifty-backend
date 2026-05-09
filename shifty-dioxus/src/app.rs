use crate::auth::Auth;
use crate::component::dropdown_base::DropdownBase;
use crate::component::tooltip::TooltipBase;
use crate::component::{Footer, TopBar};
use crate::page::NotAuthenticated;
use crate::router::Route;
use crate::service;
use crate::service::config::CONFIG;
use dioxus::prelude::*;
use web_sys::window;

pub fn App() -> Element {
    use_coroutine(service::config::config_service);
    use_coroutine(service::theme::theme_service);
    use_coroutine(service::dropdown::dropdown_service);
    use_coroutine(service::tooltip::tooltip_service);
    use_coroutine(service::i18n::i18n_service);
    use_coroutine(service::working_hours_mini::working_hours_mini_service);
    use_coroutine(service::user_management::user_management_service);
    use_coroutine(service::booking_conflict::booking_conflicts_service);
    use_coroutine(service::booking_log::booking_log_service);
    use_coroutine(service::weekly_summary::weekly_summary_service);
    use_coroutine(service::employee_work_details::employee_work_details_service);
    use_coroutine(service::employee::employee_service);
    use_coroutine(service::slot_edit::slot_edit_service);
    use_coroutine(service::billing_period::billing_period_service);
    use_coroutine(service::absence::absence_service);
    use_coroutine(service::cutover::cutover_service);
    use_coroutine(service::vacation_balance::vacation_balance_service);
    // Plan 08-07 Gap-Closure (Task 3): Feature-Flag-Service muss laufen, bevor
    // das TopBar (Task 4) den `absence_range_source_active`-Flag liest.
    let feature_flag_handle =
        use_coroutine(service::feature_flag::feature_flag_service);
    // Direkt nach Service-Konstruktion einen einmaligen Load für den
    // Cutover-Flag triggern. `use_effect`-mit-leerer-Dep wäre eine
    // Alternative; ein direkter `send` reicht hier, weil das Service nach
    // dem ersten Polling-Tick alle pending Actions abarbeitet.
    feature_flag_handle.send(
        service::feature_flag::FeatureFlagAction::LoadAbsenceRangeSourceActive,
    );
    let config = CONFIG.read();
    if !config.backend.is_empty() {
        let title = config.application_title.clone();
        let is_prod = config.is_prod;
        let env_short_description = config.env_short_description.clone();
        use_effect(move || {
            let window = window().unwrap();
            let document = window.document().unwrap();
            if is_prod {
                document.set_title(title.as_ref());
            } else {
                document.set_title(format!("{} ({})", title, env_short_description).as_str());
            }
        });

        rsx! {
            document::Stylesheet { href: asset!("./assets/tailwind.css") }
            div { class: "flex flex-col",
                DropdownBase {}
                TooltipBase {}
                Auth {
                    authenticated: rsx! {
                        Router::<Route> {}
                    },
                    unauthenticated: rsx! {
                        TopBar {}
                        NotAuthenticated {}
                    },
                }
                Footer {}
            }
        }
    } else {
        rsx! {
            div { "Loading application configuration." }
        }
    }
}
