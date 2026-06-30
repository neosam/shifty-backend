use dioxus::prelude::*;

use crate::i18n::Key;
use crate::service::error::{ErrorStore, ERROR_STORE};
use crate::service::i18n::I18N;

/// Global error banner (e.g. a failed API call).
///
/// Styled with the project's `bad` design tokens to match the other banners
/// (rounded, soft background, left accent border) instead of the old flat
/// hard-red bar, and dismissible via the `×` button which clears the global
/// `ERROR_STORE`.
///
/// Pitfall 5: all Tailwind classes are plain string literals (no interpolation),
/// so the content scan keeps them.
#[component]
pub fn ErrorView() -> Element {
    let error = ERROR_STORE.read();
    if let Some(ref error) = error.error {
        let dismiss_label = I18N.read().t(Key::ErrorBannerDismiss).to_string();
        rsx! {
            div {
                class: "border-l-[3px] border-bad bg-bad-soft rounded-md p-3 flex items-start justify-between gap-3",
                role: "alert",
                div { class: "flex items-start gap-2 min-w-0",
                    span { class: "text-bad font-bold leading-none flex-shrink-0", "⚠" }
                    div { class: "text-body text-ink min-w-0", "{error}" }
                }
                button {
                    class: "text-bad hover:text-ink flex-shrink-0 text-lg leading-none font-bold px-1 transition-colors",
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
