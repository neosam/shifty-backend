use crate::auth::Auth;
use crate::component::dropdown_base::DropdownBase;
use crate::component::tooltip::TooltipBase;
use crate::component::{Footer, ImpersonationBanner, TopBar};
use crate::page::NotAuthenticated;
use crate::router::Route;
use crate::service;
use crate::service::config::CONFIG;
use crate::service::impersonate::ImpersonateAction;
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
    use_coroutine(service::vacation_balance::vacation_balance_service);
    // D-32-05 / SC2: register the impersonate service; LoadStatus is sent on
    // first mount (inside the if-block below) so the amber banner re-appears
    // automatically after a hard page reload.
    let impersonate_init = use_coroutine(service::impersonate::impersonate_service);
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
        // D-32-05 / SC2: fire LoadStatus once on app mount so the impersonation
        // banner survives a hard page reload (the GET returns the live backend
        // state and updates IMPERSONATE_STORE accordingly).
        use_effect(move || {
            impersonate_init.send(ImpersonateAction::LoadStatus);
        });

        rsx! {
            document::Stylesheet { href: asset!("./assets/tailwind.css") }
            div { class: "flex flex-col",
                DropdownBase {}
                TooltipBase {}
                Auth {
                    authenticated: rsx! {
                        // D-32-04 / SC1: non-closable amber banner mounted ABOVE the
                        // router outlet so it appears on every route.
                        ImpersonationBanner {}
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
