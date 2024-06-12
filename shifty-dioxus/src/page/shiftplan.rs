use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use tracing::info;
use uuid::timestamp::context;
use uuid::Uuid;

use crate::component::TopBar;
use crate::component::WeekView;
use crate::js;
use crate::loader;
use crate::state;

pub enum ShiftPlanAction {
    AddUserToSlot {
        slot_id: Uuid,
        sales_person_id: Uuid,
        week: u8,
        year: u32,
    },
}

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

    let mut shift_plan_context = {
        let config = config.clone();
        use_resource(move || {
            loader::load_shift_plan(
                config.to_owned(),
                *week.to_owned().read(),
                *year.to_owned().read(),
            )
        })
    };
    let current_sales_person_resource = {
        let config = config.clone();
        use_resource(move || loader::load_current_sales_person(config.to_owned()))
    };

    let (current_sales_person, current_sales_person_id): (Rc<str>, Option<Uuid>) =
        match &*current_sales_person_resource.read_unchecked() {
            Some(Ok(Some(sales_person))) => (sales_person.name.clone(), Some(sales_person.id)),
            Some(Ok(None)) => ("No sales person".into(), None),
            Some(Err(err)) => (
                format!("Error while loading sales person: {err}").into(),
                None,
            ),
            None => ("Loading sales person...".into(), None),
        };

    let cr = use_coroutine(|mut rx: UnboundedReceiver<ShiftPlanAction>| async move {
        while let Some(action) = rx.next().await {
            match action {
                ShiftPlanAction::AddUserToSlot {
                    slot_id,
                    sales_person_id,
                    week,
                    year,
                } => {
                    info!("Registering user to slot");
                    loader::register_user_to_slot(
                        config.to_owned(),
                        slot_id,
                        sales_person_id,
                        week,
                        year,
                    )
                    .await;
                    shift_plan_context.restart();
                }
            }
        }
    });

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
                    WeekView {
                        shiftplan_data: shift_plan.clone(),
                        add_event: move |slot: state::Slot| {
                            info!("Register to slot");
                            cr.send(ShiftPlanAction::AddUserToSlot {
                                slot_id: slot.id,
                                sales_person_id: current_sales_person_id.unwrap(),
                                week: *week.read(),
                                year: *year.read(),
                            });
                            //loader::register_user_to_slot(config.to_owned(), slot.id, sales_id.unwrap(), *week.read(), *year.read());
                        }
                    }
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
