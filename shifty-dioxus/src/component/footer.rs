use dioxus::prelude::*;

use crate::api;

#[component]
pub fn Footer() -> Element {
    let version = env!("CARGO_PKG_VERSION");
    let config = use_context::<crate::state::config::Config>();
    let version_resource = use_resource(move || api::get_version(config.clone()));
    rsx! {
        footer {
            // class: "fixed bottom-0 right-0 text-xs text-gray-500 p-2",
            class: "text-xs text-gray-500 p-2 flex flex-row",
            div {
                "Shifty Frontend {version} |"
            }
            match &*version_resource.read_unchecked() {
                Some(Ok(version)) => {
                    rsx! { div {
                        "Backend: {version}"
                    } }
                },
                Some(Err(err)) => {
                    rsx! { div {
                        "Error while loading version: {err}"
                    } }
                },
                None => {
                    rsx! { div {
                        "Loading backend version."
                    } }
                }
            }
        }
    }
}
