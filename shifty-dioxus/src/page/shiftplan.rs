use std::rc::Rc;

use dioxus::prelude::*;
use uuid::timestamp::context;

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

    let shift_plan_context = {
        let config = config.clone();
        use_resource(move || {
            loader::load_shift_plan(
                config.to_owned(),
                *week.to_owned().read(),
                *year.to_owned().read(),
            )
        })
    };
    let current_sales_person_resource =
        use_resource(move || loader::load_current_sales_person(config.to_owned()));

    let current_sales_person: Rc<str> = match &*current_sales_person_resource.read_unchecked() {
        Some(Ok(Some(sales_person))) => sales_person.name.clone(),
        Some(Ok(None)) => "No sales person".into(),
        Some(Err(err)) => format!("Error while loading sales person: {err}").into(),
        None => "Loading sales person...".into(),
    };

    rsx! {
        TopBar {}

        h2 {
            class: "m-4 text-lg",
            "Week: {week.read()} Year: {year.read()} - from {date_str} - {current_sales_person}"
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
