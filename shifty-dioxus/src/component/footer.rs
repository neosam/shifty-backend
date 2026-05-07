use dioxus::prelude::*;

use crate::{api, service::config::CONFIG};

#[component]
pub fn Footer() -> Element {
    let version = env!("CARGO_PKG_VERSION");
    let config = CONFIG.read().clone();
    let version_resource = use_resource(move || api::get_version(config.clone()));
    rsx! {
        footer {
            class: "text-small text-ink-muted p-2 flex flex-row",
            div { "Shifty Frontend {version} |" }
            match &*version_resource.read_unchecked() {
                Some(Ok(version)) => {
                    rsx! {
                        div { "Backend: {version}" }
                    }
                }
                Some(Err(err)) => {
                    rsx! {
                        div { "Error while loading version: {err}" }
                    }
                }
                None => {
                    rsx! {
                        div { "Loading backend version." }
                    }
                }
            }
        }
    }
}
