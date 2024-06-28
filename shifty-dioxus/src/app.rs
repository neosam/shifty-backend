use crate::api;
use crate::auth::Auth;
use crate::component::{Footer, TopBar};
use crate::page::NotAuthenticated;
use crate::{i18n, router::Route};
use dioxus::prelude::*;

pub fn App() -> Element {
    let config_resource = use_resource(|| api::load_config());
    let language = web_sys::window()
        .map(|w| w.navigator())
        .and_then(|n| n.language())
        .map(|locale| locale[..2].to_string())
        .unwrap_or_else(|| "en".to_string());
    use_context_provider(|| i18n::generate(i18n::Locale::from_str(&language)));
    match &*config_resource.read_unchecked() {
        Some(Ok(config)) => {
            use_context_provider(|| config.clone());
            rsx! {
                /* Router::<Route> {} */
                div {
                    class: "flex flex-col",
                    Auth {
                        authenticated: {
                            eval(
                                format!("window.oidcLoginKeepAliveURL = '{}/authenticate';", config.backend.clone()).as_str()
                            );
                            rsx! { div {
                                Router::<Route> {}
                            }}
                        },
                        unauthenticated: rsx! {
                            TopBar {}
                            NotAuthenticated {}
                        }
                    }
                    Footer {}
                }
            }
        }
        Some(Err(err)) => {
            rsx! {
                div { "Error while loading technical configuration: {err}" }
            }
        }
        None => {
            rsx! {
                div { "Loading application configuration." }
            }
        }
    }
}
