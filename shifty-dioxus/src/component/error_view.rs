use dioxus::prelude::*;

use crate::i18n::Key;
use crate::service::error::{ErrorStore, ERROR_STORE};
use crate::service::i18n::I18N;

#[component]
pub fn ErrorView() -> Element {
    let error = ERROR_STORE.read();
    if let Some(ref error) = error.error {
        let dismiss_label = I18N.read().t(Key::ErrorBannerDismiss).to_string();
        rsx! {
            div {
                class: "error-view",
                div {
                    class: "error-message",
                    "{error}"
                }
                button {
                    class: "error-dismiss",
                    r#type: "button",
                    "aria-label": "{dismiss_label}",
                    title: "{dismiss_label}",
                    // Reset the global error so the banner disappears. Writing the
                    // GlobalSignal directly mirrors how errors are set across the
                    // services (the ErrorAction coroutine is unused here).
                    onclick: move |_| {
                        *ERROR_STORE.write() = ErrorStore::cleared();
                    },
                    "×"
                }
            }
        }
    } else {
        rsx!()
    }
}
