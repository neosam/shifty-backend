use dioxus::prelude::*;

use crate::component::TopBar;
use crate::component::WeekView;
use crate::loader;

#[component]
pub fn ShiftPlan() -> Element {
    let shift_plan_context = use_resource(|| loader::load_shift_plan());
    rsx! {
        TopBar {}

        {match &*shift_plan_context.read_unchecked() {
            Some(Ok(shift_plan)) => {
                rsx! {div {
                    class: "m-4",
                    WeekView {shiftplan_data: shift_plan.clone()}
                }}
            }
            Some(Err(err)) => {
                rsx!{div {
                    class: "m-4",
                    "Error while loading shift plan: {err}"
                }}
            }
            _ => {
                rsx!{div {
                    class: "m-4",
                    "Loading shift plan..."
                }}
            }
        }}
    }
}
