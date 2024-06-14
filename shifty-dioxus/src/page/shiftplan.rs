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
use crate::state::shiftplan::SalesPerson;

pub enum ShiftPlanAction {
    AddUserToSlot {
        slot_id: Uuid,
        sales_person_id: Uuid,
        week: u8,
        year: u32,
    },
    RemoveUserFromSlot {
        slot_id: Uuid,
        sales_person_id: Uuid,
        week: u8,
        year: u32,
    },
    NextWeek,
    PreviousWeek,
    UpdateSalesPerson(Uuid),
    CopyFromPreviousWeek,
}

#[component]
pub fn ShiftPlan() -> Element {
    let config = use_context::<state::Config>();
    let auth_info = use_context::<state::AuthInfo>();
    let is_shiftplanner = auth_info.has_privilege("shiftplanner");

    let mut week = use_signal(|| js::get_current_week());
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
    let sales_persons_resource = {
        let config = config.clone();
        use_resource(move || loader::load_sales_persons(config.to_owned()))
    };

    let mut current_sales_person: Signal<Option<SalesPerson>> = use_signal(|| None);

    //let (current_sales_person, current_sales_person_id): (Rc<str>, Option<Uuid>) =
    //    match &*current_sales_person_resource.read_unchecked() {
    //        Some(Ok(Some(sales_person))) => (sales_person.name.clone(), Some(sales_person.id)),
    //        Some(Ok(None)) => ("No sales person".into(), None),
    //        Some(Err(err)) => (
    //            format!("Error while loading sales person: {err}").into(),
    //            None,
    //        ),
    //        None => ("Loading sales person...".into(), None),
    //    };

    let cr = use_coroutine(|mut rx: UnboundedReceiver<ShiftPlanAction>| async move {
        let sales_person = loader::load_current_sales_person(config.to_owned())
            .await
            .ok()
            .flatten();
        *current_sales_person.write() = sales_person;

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
                ShiftPlanAction::RemoveUserFromSlot {
                    slot_id,
                    sales_person_id,
                    week,
                    year,
                } => {
                    info!("Removing user from slot");
                    if let Some(Ok(shift_plan)) = &*shift_plan_context.read_unchecked() {
                        loader::remove_user_from_slot(
                            config.to_owned(),
                            slot_id,
                            sales_person_id,
                            shift_plan.clone(),
                        )
                        .await;
                    }
                    shift_plan_context.restart();
                }
                ShiftPlanAction::NextWeek => {
                    info!("Next week");
                    let current_week = *week.read();
                    week.set(current_week + 1);
                    shift_plan_context.restart();
                }
                ShiftPlanAction::PreviousWeek => {
                    info!("Previous week");
                    let current_week = *week.read();
                    week.set(current_week - 1);
                    shift_plan_context.restart();
                }
                ShiftPlanAction::UpdateSalesPerson(uuid) => {
                    info!("Update sales person");
                    if let Some(Ok(sales_persons)) = &*sales_persons_resource.read_unchecked() {
                        let new_sales_person =
                            sales_persons.iter().find(|sp| sp.id == uuid).cloned();
                        if let Some(new_sales_person) = new_sales_person {
                            *current_sales_person.write() = Some(new_sales_person);
                        }
                    }
                }
                ShiftPlanAction::CopyFromPreviousWeek => {
                    loader::copy_from_previous_week(config.to_owned(), *week.read(), *year.read())
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
            button {
                onclick: move |_| cr.send(ShiftPlanAction::PreviousWeek),
                class: "border-2 border-solid border-black mr-2 p-2",
                "<"
            }
            "Week: {week.read()} Year: {year.read()} - from {date_str}"
            button {
                onclick: move |_| cr.send(ShiftPlanAction::NextWeek),
                class: "border-2 border-solid border-black mr-2 ml-2 p-2",
                ">"
            }
            if is_shiftplanner {
                button {
                    onclick: move |_| cr.send(ShiftPlanAction::CopyFromPreviousWeek),
                    class: "border-2 border-solid border-black mr-2 ml-2 p-2",
                    "Take last week"
                }
            }
            match &*sales_persons_resource.read_unchecked() {
                Some(Ok(sales_persons)) => {
                    if is_shiftplanner {
                        rsx!{div {
                            class: "m-4",
                            "Sales person:"
                            select {
                                onchange: move |event| {
                                    info!("Event: {:?}", event);
                                    let value = event.data.value().parse::<Uuid>().unwrap();
                                    cr.send(ShiftPlanAction::UpdateSalesPerson(value));
                                },
                                value: current_sales_person.read().as_ref().map(|sp| sp.id.to_string()),
                                for sales_person in sales_persons {
                                    if let Some(ref current_sales_person) = *current_sales_person.read() {
                                        option {
                                            value: sales_person.id.to_string(),
                                            selected: sales_person.id == current_sales_person.id,
                                            {sales_person.name.clone()}
                                        }
                                    }
                                }
                            }
                        }}
                    } else {
                        if let Some( ref current_sales_person) = *current_sales_person.read() {
                            rsx!{div {
                                class: "m-4",
                                "You are {current_sales_person.name}"
                            }}
                        } else {
                            rsx!{ "" }
                        }
                    }
                }
                Some(Err(err)) => {
                    rsx!{div {
                        class: "m-4",
                        "Error while loading sales persons: {err}"
                    }}
                }
                _ => {
                    rsx!{div {
                        class: "m-4",
                        "Loading sales persons..."
                    }}
                }
            }
        }


        {match &*shift_plan_context.read_unchecked() {
            Some(Ok(shift_plan)) => {
                rsx! {div {
                    class: "m-4",
                    WeekView {
                        shiftplan_data: shift_plan.clone(),
                        add_event: move |slot: state::Slot| {
                            to_owned![current_sales_person];
                            info!("Register to slot");
                            if let Some(ref current_sales_person) = *current_sales_person.read() {
                                cr.send(ShiftPlanAction::AddUserToSlot {
                                    slot_id: slot.id,
                                    sales_person_id: current_sales_person.id,
                                    week: *week.read(),
                                    year: *year.read(),
                                });
                            };
                        },
                        remove_event: move |slot: state::Slot| {
                            to_owned![current_sales_person];
                            info!("Register to slot");
                            if let Some(ref current_sales_person) = *current_sales_person.read() {
                                cr.send(ShiftPlanAction::RemoveUserFromSlot {
                                    slot_id: slot.id,
                                    sales_person_id: current_sales_person.id,
                                    week: *week.read(),
                                    year: *year.read(),
                                });
                            };
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
