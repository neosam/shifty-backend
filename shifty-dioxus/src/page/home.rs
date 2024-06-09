use dioxus::prelude::*;

use crate::component::TopBar;
use crate::state::State;

#[component]
pub fn Home() -> Element {
    let mut count = use_signal(|| 0);
    let state = use_context::<Signal<State>>();
    let backend = state.read().config.backend.clone();

    rsx! {
        TopBar {}
        div {
            h1 { "High-Five counter: {count} {backend}" }
            button { onclick: move |_| count += 1, "Up high!" }
            button { onclick: move |_| count -= 1, "Down low!" }
        }
    }
}
