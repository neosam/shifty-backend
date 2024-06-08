use dioxus::prelude::*;

use crate::top_bar::TopBar;
use crate::week_view::WeekView;

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
