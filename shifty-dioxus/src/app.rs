use crate::auth::Auth;
use crate::not_authenticated::NotAuthenticated;
use crate::top_bar::TopBar;
use crate::{api, AuthInfo};
use crate::{i18n, router::Route, state::State};
use dioxus::prelude::*;

pub fn App() -> Element {
    let config = use_resource(|| api::load_config());
    match &*config.read_unchecked() {
        Some(Ok(config)) => {
            use_context_provider(|| {
                Signal::new(State {
                    config: config.clone(),
                    i18n: i18n::generate(i18n::Locale::De).into(),
                    auth_info: AuthInfo::default(),
                })
            });
            rsx! {
                /* Router::<Route> {} */
                Auth {
                    authenticated: rsx! {
                        div {
                            Router::<Route> {}
                        }
                    },
                    unauthenticated: rsx! {
                        TopBar {}
                        NotAuthenticated {}
                    }
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
