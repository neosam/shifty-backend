use crate::auth::Auth;
use crate::component::{Footer, TopBar};
use crate::page::NotAuthenticated;
use crate::service::CONFIG;
use crate::{api, service};
use crate::{i18n, router::Route};
use dioxus::prelude::*;
use web_sys::window;

pub fn App() -> Element {
    let mut config_service = use_coroutine(service::config_service);
    let language = web_sys::window()
        .map(|w| w.navigator())
        .and_then(|n| n.language())
        .map(|locale| locale[..2].to_string())
        .unwrap_or_else(|| "en".to_string());
    use_context_provider(|| i18n::generate(i18n::Locale::from_str(&language)));
    use_coroutine(service::dropdown_service);
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
            div { class: "flex flex-col",
                Auth {
                    authenticated: {
                        eval(
                            format!(
                                "window.oidcLoginKeepAliveURL = '{}/authenticate';",
                                config.backend.clone(),
                            )
                                .as_str(),
                        );
                        rsx! {
                            div { Router::<Route> {} }
                        }
                    },
                    unauthenticated: rsx! {
                        TopBar {}
                        NotAuthenticated {}
                    }
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
