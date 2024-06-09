use dioxus::prelude::*;

use crate::component::TopBar;
use crate::component::WeekView;

#[component]
pub fn ShiftPlan() -> Element {
    rsx! {
        TopBar {}
        div {
            class: "m-4",
            WeekView {}
        }
    }
}
