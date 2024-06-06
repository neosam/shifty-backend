use crate::api;
use crate::{i18n, router::Route, state::State};
use dioxus::prelude::*;

pub fn App() -> Element {
    let config = use_resource(|| api::load_config());
    match &*config.read_unchecked() {
        Some(Ok(config)) => {
            use_context_provider(|| State {
                config: config.clone(),
                i18n: i18n::generate(i18n::Locale::De).into(),
            });
            rsx! {
                Router::<Route> {}
            }
        }
        Some(Err(err)) => {
            rsx! {
                div { "Error while loading technical configuration: {err}" }
            }
        }
        None => {
            rsx! {
                div { "Loading technical configuration..." }
            }
        }
    }
}
