use std::rc::Rc;

use crate::auth::Auth;
use crate::component::TopBar;
use crate::page::NotAuthenticated;
use crate::{api, state, state::AuthInfo};
use crate::{i18n, router::Route};
use dioxus::prelude::*;
use futures_util::StreamExt;
use js_sys::wasm_bindgen::closure::Closure;
use js_sys::wasm_bindgen::JsValue;
use js_sys::{Array, Reflect};
use tracing::info;
use web_sys::wasm_bindgen::JsCast;

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
