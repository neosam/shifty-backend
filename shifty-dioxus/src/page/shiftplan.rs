use dioxus::prelude::*;

use crate::component::TopBar;
use crate::component::WeekView;
use crate::js;
use crate::loader;
use crate::state;

#[component]
pub fn ShiftPlan() -> Element {
    let config = use_context::<state::Config>();
    let week = use_signal(|| js::get_current_week());
    let year = use_signal(|| js::get_current_year());
    let date =
        time::Date::from_iso_week_date(*year.read() as i32, *week.read(), time::Weekday::Monday)
            .unwrap();
    let formatter = time::format_description::parse("[day].[month].[year]").unwrap();
    let date_str = date.format(&formatter).unwrap().to_string();

    let shift_plan_context = use_resource(move || {
        loader::load_shift_plan(
            config.to_owned(),
            *week.to_owned().read(),
            *year.to_owned().read(),
        )
    });
    rsx! {
        TopBar {}

        h2 {
            class: "m-4 text-lg",
            "Week: {week.read()} Year: {year.read()} - from {date_str}"
        }

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
