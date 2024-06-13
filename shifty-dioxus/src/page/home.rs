use dioxus::prelude::*;

use crate::component::TopBar;
use crate::router::Route;
use crate::state;

#[component]
pub fn Home() -> Element {
    let mut count = use_signal(|| 0);
    let backend = use_context::<state::Config>().backend.clone();
    let nav = navigator();
    nav.push(Route::ShiftPlan {});

    rsx! {
        TopBar {}
        div {
            h1 { "Welcome to Shifty!" }
            p { "" }
        }
    }
}
