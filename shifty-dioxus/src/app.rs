use std::rc::Rc;

use crate::auth::Auth;
use crate::component::TopBar;
use crate::page::NotAuthenticated;
use crate::{api, state, state::AuthInfo};
use crate::{i18n, router::Route};
use dioxus::prelude::*;
use futures_util::StreamExt;
use tracing::info;

pub enum AppAction {
    SetConfig(state::Config),
}

pub fn App() -> Element {
    let config_resource = use_resource(|| api::load_config());
    use_context_provider(|| i18n::generate(i18n::Locale::De));
    match &*config_resource.read_unchecked() {
        Some(Ok(config)) => {
            use_context_provider(|| config.clone());
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
