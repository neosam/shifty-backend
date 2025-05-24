use dioxus::prelude::*;

use crate::service::error::ERROR_STORE;

#[component]
pub fn ErrorView() -> Element {
    let error = ERROR_STORE.read();
    if let Some(ref error) = error.error {
        rsx!{
            div {
                class: "error-view",
                div {
                    class: "error-message",
                    "{error}"
                }
            }
        }
    } else {
        rsx!()
    }

}
