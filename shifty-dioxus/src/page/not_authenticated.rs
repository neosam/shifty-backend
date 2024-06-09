use dioxus::prelude::*;

#[component]
pub fn NotAuthenticated() -> Element {
    rsx! {
        div {
            class: "flex place-content-center mt-16",
            div {
                h1 {
                    class: "text-6xl font-bold",
                    "Welcome to Shifty!"
                }
                p {
                    class: "mt-8",
                    "Please log in."
                }
            }
        }
    }
}
