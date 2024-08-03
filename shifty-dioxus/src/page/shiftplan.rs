use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use tracing::info;
use uuid::Uuid;

use crate::base_types::ImStr;
use crate::component::dropdown_base::DropdownTrigger;
use crate::component::TopBar;
use crate::component::WeekView;
use crate::error::result_handler;
use crate::i18n::{self, Key};
use crate::js;
use crate::loader;
use crate::service;
use crate::service::AUTH;
use crate::service::CONFIG;
use crate::service::I18N;
use crate::state;
use crate::state::sales_person_available::SalesPersonUnavailable;
use crate::state::shiftplan::SalesPerson;
use crate::state::Config;
use crate::state::Weekday;

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
    },
    NextWeek,
    PreviousWeek,
    UpdateSalesPerson(Uuid),
    CopyFromPreviousWeek,
    ToggleAvailability(Weekday),
}

#[component]
pub fn ShiftPlan() -> Element {
    let config = CONFIG.read().clone();
    let i18n = I18N.read().clone();
    let auth_info = AUTH.read().clone();
    let is_shiftplanner = auth_info
        .map(|auth_info| auth_info.has_privilege("shiftplanner"))
        .unwrap_or(false);
    let dropdown_cr = use_coroutine_handle::<service::DropdownAction>();

    let mut week = use_signal(|| js::get_current_week());
    let year = use_signal(|| js::get_current_year());
    let date =
        time::Date::from_iso_week_date(*year.read() as i32, *week.read(), time::Weekday::Monday)
            .unwrap();
    let formatter = time::format_description::parse("[day].[month]").unwrap();
    let date_str = date.format(&formatter).unwrap().to_string();

    let calendar_week_str = i18n.t_m(
        Key::ShiftplanCalendarWeek,
        [
            ("week", week.read().to_string().as_str()),
            ("year", &year.read().to_string().as_str()),
            ("date", date_str.as_str()),
        ]
        .into(),
    );
    let take_last_week_str: ImStr = i18n.t(Key::ShiftplanTakeLastWeek).into();
    let edit_as_str = i18n.t(Key::ShiftplanEditAs);
    let you_are_str = i18n.t(Key::ShiftplanYouAre);

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
    let sales_persons_resource = {
        let config = config.clone();
        use_resource(move || loader::load_sales_persons(config.to_owned()))
    };

    let mut current_sales_person: Signal<Option<SalesPerson>> = use_signal(|| None);
    let unavailable_days: Signal<Rc<[SalesPersonUnavailable]>> = use_signal(|| [].into());

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

    let cr = use_coroutine({
        to_owned![year, week, current_sales_person, unavailable_days, config];
        move |mut rx: UnboundedReceiver<ShiftPlanAction>| async move {
            let sales_person = loader::load_current_sales_person(config.to_owned())
                .await
                .ok()
                .flatten();
            *current_sales_person.write() = sales_person.clone();

            let mut reload_unavailable_days = {
                to_owned![current_sales_person, unavailable_days];
                move |config: Config| async move {
                    if let Some(sales_person) = &*current_sales_person.read() {
                        let result = result_handler(
                            loader::load_unavailable_sales_person_days_for_week(
                                config.clone(),
                                sales_person.id,
                                *year.read(),
                                *week.read(),
                            )
                            .await,
                        )
                        .unwrap_or(Rc::new([]));
                        *unavailable_days.write() = result;
                    }
                }
            };
            reload_unavailable_days(config.clone()).await;

            //if let Some(sales_person) = sales_person {
            //    let unavailable_days = result_handler(
            //        loader::load_unavailable_sales_person_days_for_week(
            //            config.clone(),
            //            sales_person.id,
            //            *year.read(),
            //            *week.read(),
            //        )
            //        .await,
            //    )
            //    .unwrap_or(Rc::new([]))
            //    .iter()
            //    .map(|unavailable_day| unavailable_day.day_of_week)
            //    .collect::<Rc<[Weekday]>>();
            //    *discouraged_weekdays.write() = unavailable_days;
            //};

            while let Some(action) = rx.next().await {
                match action {
                    ShiftPlanAction::AddUserToSlot {
                        slot_id,
                        sales_person_id,
                        week,
                        year,
                    } => {
                        info!("Registering user to slot");
                        result_handler(
                            loader::register_user_to_slot(
                                config.to_owned(),
                                slot_id,
                                sales_person_id,
                                week,
                                year,
                            )
                            .await,
                        );
                        shift_plan_context.restart();
                    }
                    ShiftPlanAction::RemoveUserFromSlot {
                        slot_id,
                        sales_person_id,
                    } => {
                        info!("Removing user from slot");
                        if let Some(Ok(shift_plan)) = &*shift_plan_context.read_unchecked() {
                            result_handler(
                                loader::remove_user_from_slot(
                                    config.to_owned(),
                                    slot_id,
                                    sales_person_id,
                                    shift_plan.clone(),
                                )
                                .await,
                            );
                        }
                        shift_plan_context.restart();
                    }
                    ShiftPlanAction::NextWeek => {
                        info!("Next week");
                        let current_week = *week.read();
                        week.set(current_week + 1);
                        shift_plan_context.restart();
                        reload_unavailable_days(config.clone()).await;
                    }
                    ShiftPlanAction::PreviousWeek => {
                        info!("Previous week");
                        let current_week = *week.read();
                        week.set(current_week - 1);
                        shift_plan_context.restart();
                        reload_unavailable_days(config.clone()).await;
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
                        reload_unavailable_days(config.clone()).await;
                    }
                    ShiftPlanAction::CopyFromPreviousWeek => {
                        result_handler(
                            loader::copy_from_previous_week(
                                config.to_owned(),
                                *week.read(),
                                *year.read(),
                            )
                            .await,
                        );
                        shift_plan_context.restart();
                    }
                    ShiftPlanAction::ToggleAvailability(weekday) => {
                        if let Some(available_day) = unavailable_days
                            .read()
                            .iter()
                            .find(|unavailable_day| unavailable_day.day_of_week == weekday)
                        {
                            result_handler(
                                loader::delete_unavailable_sales_person_day(
                                    config.to_owned(),
                                    available_day.id,
                                )
                                .await,
                            );
                        } else if let Some(sales_person) = current_sales_person.read().as_ref() {
                            result_handler(
                                loader::create_unavailable_sales_person_day(
                                    config.to_owned(),
                                    sales_person.id,
                                    *year.read(),
                                    *week.read(),
                                    weekday,
                                )
                                .await,
                            );
                        }
                        reload_unavailable_days(config.clone()).await;
                    }
                }
            }
        }
    });

    rsx! {
        TopBar {}

        DropdownTrigger {
            entries: [
                ("Entry 1", Box::new(|| info!("Entry 1 clicked"))).into(),
                ("Entry 2", Box::new(|| info!("Entry 2 clicked"))).into(),
            ]
                .into(),
            "I am a dropdown"
        }
        div { class: "flex flex-col md:flex-row md:items-center md:justify-between",
            div { class: "m-4 text-lg flex align-center justify-center width-full",
                button {
                    onclick: move |_| cr.send(ShiftPlanAction::PreviousWeek),
                    class: "border-2 border-solid border-black mr-2 pt-2 pb-2 pl-4 pr-4 text-xl font-bold print:hidden",
                    "<"
                }
                div { class: "pt-2", "{calendar_week_str}" }
                button {
                    onclick: move |_| cr.send(ShiftPlanAction::NextWeek),
                    class: "border-2 border-solid border-black mr-2 ml-2 pt-2 pb-2 pl-4 pr-4 text-xl font-bold print:hidden",
                    ">"
                }
            }
            div { class: "flex flex-row ml-4 mr-4 border-t-2 border-solid border-black pt-4 items-center justify-between text-right md:justify-right md:border-t-none md:border-t-0 md:mt-4 md:mb-4 md:pt-0 md:gap-4 print:hidden",
                if is_shiftplanner {
                    DropdownTrigger {
                        entries: [
                            (
                                take_last_week_str,
                                Box::new(move || cr.send(ShiftPlanAction::CopyFromPreviousWeek)),
                            )
                                .into(),
                        ]
                            .into(),
                        button { class: "border-2 border-solid border-black pt-2 pb-2 pl-4 pr-4 text-xl font-bold",
                            "..."
                        }
                    }
                }
                match &*sales_persons_resource.read_unchecked() {
                    Some(Ok(sales_persons)) => {
                        if is_shiftplanner {
                            rsx!{div {
                                class: "align-center",
                                "{edit_as_str}"
                                select {
                                    class: "bg-slate-200 p-1 rounded-md ml-2",
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
                                    "{you_are_str}"
                                    span {
                                        class: "bg-slate-200 p-1 rounded-md",
                                        "{current_sales_person.name}"
                                    }
                                }}
                            } else {
                                rsx!{ "" }
                            }
                        }
                    }
                    Some(Err(err)) => {
                        rsx!{div {
                            "Error while loading sales persons: {err}"
                        }}
                    }
                    _ => {
                        rsx!{div {
                            "Loading sales persons..."
                        }}
                    }
                }
            }
        }

        {match &*shift_plan_context.read_unchecked() {
            Some(Ok(shift_plan)) => {
                rsx! {div {
                    class: "m-4",
                    WeekView {
                        shiftplan_data: shift_plan.clone(),
                        date_of_monday: date,
                        highlight_item_id: current_sales_person.read().as_ref().map(|sp| sp.id),
                        discourage_weekdays: unavailable_days.read().iter().map(|unavailable_day| unavailable_day.day_of_week).collect(),
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
                                });
                            };
                        },
                        item_clicked: move |sales_person_id: Uuid| {
                            if is_shiftplanner {
                                cr.send(ShiftPlanAction::UpdateSalesPerson(sales_person_id));
                            }
                        },
                        title_double_clicked: move |weekday: Weekday| {
                            if is_shiftplanner {
                                cr.send(ShiftPlanAction::ToggleAvailability(weekday));
                            }
                        },
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
