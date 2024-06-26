use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    let version = env!("CARGO_PKG_VERSION");
    rsx! {
        footer {
            // class: "fixed bottom-0 right-0 text-xs text-gray-500 p-2",
            class: "text-xs text-gray-500 p-2",
            div {
                "Shifty {version}"
            }
        }
    }
}
