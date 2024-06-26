use dioxus::prelude::*;
use futures_util::StreamExt;
use time::macros::format_description;
use uuid::Uuid;

use crate::{
    api,
    error::result_handler,
    js,
    state::{employee::WorkingHoursCategory, Config},
};

pub enum AddExtraHoursFormAction {
    Submit,
}

#[derive(Clone, PartialEq, Props)]
pub struct AddExtraHoursFormProps {
    pub sales_person_id: Uuid,
    pub onabort: EventHandler<()>,
    pub onsaved: EventHandler<()>,
}

#[component]
pub fn AddExtraHoursForm(props: AddExtraHoursFormProps) -> Element {
    let format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    let mut category = use_signal(|| WorkingHoursCategory::ExtraWork("".into()));
    let mut amount = use_signal(|| 0.0);
    let mut description = use_signal(|| "".to_string());
    let mut when = use_signal(|| js::current_datetime().format(&format).unwrap());

    let config = use_context::<Config>();
    let sales_person_id = props.sales_person_id;

    let cr = use_coroutine(
        move |mut rx: UnboundedReceiver<AddExtraHoursFormAction>| async move {
            to_owned![category, amount, description, when, config];
            while let Some(action) = rx.next().await {
                match action {
                    AddExtraHoursFormAction::Submit => {
                        let category = (*category.read()).clone();
                        let amount = *amount.read();
                        let description = (*description.read()).clone();
                        let when = (*when.read()).clone();

                        result_handler(
                            api::add_extra_hour(
                                config.to_owned(),
                                sales_person_id,
                                amount,
                                (&category).into(),
                                description,
                                when,
                            )
                            .await,
                        );

                        props.onsaved.call(());
                    }
                }
            }
        },
    );

    rsx! {
        form {
            h1 {
                class: "text-2xl font-bold",
                "Add extra hours"
            }

            div {
                class: "flex flex-col md:flex-row md:border-b-2 border-gray-300 border-dashed mb-1",
                label {
                    class: "block mt-4 mr-4 grow",
                    "Category"
                }
                select {
                    class: "block mt-2 pl-2 pr-2 w-full md:w-1/2",
                    value: "{category.read().identifier()}",
                    onchange: move |event| {
                        let value = event.data.value();
                        *category.write() = WorkingHoursCategory::from_identifier(&value);
                    },
                    option {
                        value: "extra_work",
                        "Extra Work"
                    }
                    option {
                        value: "holiday",
                        "Holiday"
                    }
                    option {
                        value: "sick_leave",
                        "Sick"
                    }
                    option {
                        value: "vacation",
                        "Vacation"
                    }
                }
            }

            div {
                class: "flex flex-col md:flex-row md:border-b-2 border-gray-300 border-dashed mb-1",
                label {
                    class: "block mt-4 mr-4 grow",
                    "Amount of hours"
                }
                input {
                    class: "block mt-2 pl-2 pr-2 border border-black w-full md:w-1/2",
                    value: "{amount.read()}",
                    onchange: move |event| {
                        let value = event.data.value().parse::<f32>().unwrap_or(0.0);
                        *amount.write() = value;
                    },
                    "type": "number",
                }
            }

            div {
                class: "flex flex-col md:flex-row md:border-b-2 border-gray-300 border-dashed mb-1",
                label {
                    class: "block mt-4 mr-4 grow",
                    "Description"
                }
                input {
                    class: "block mt-2 pl-2 pr-2 border border-black w-full md:w-1/2",
                    value: "{description.read()}",
                    onchange: move |event| {
                        let value = event.data.value();
                        *description.write() = value;
                    },
                }
            }

            div {
                class: "flex flex-col md:flex-row md:border-b-2 border-gray-300 border-dashed mb-1",
                label {
                    class: "block mt-4 mr-4 grow",
                    "When"
                }
                input {
                    class: "block mt-2 pl-2 pr-2 border border-black w-full md:w-1/2",
                    value: "{*when.read()}",
                    onchange: move |event| {
                        let value = event.data.value();
                        *when.write() = value;
                    },
                    "type": "datetime-local",
                }
            }

            div {
                class: "flex flex-col md:flex-row md:border-b-2 border-gray-300 border-dashed mb-1 mt-8",
                button {
                    class: "block mt-2 pl-2 pr-2 border border-black w-full md:w-1/2",
                    onclick: move |_| props.onabort.call(()),
                    "Abort"
                }
                button {
                    class: "block mt-2 pl-2 pr-2 border border-black w-full md:w-1/2",
                    onclick: move |_| cr.send(AddExtraHoursFormAction::Submit),
                    "Submit"
                }
            }
        }
    }
}
