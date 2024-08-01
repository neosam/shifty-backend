use dioxus::prelude::*;
use futures_util::StreamExt;
use time::macros::format_description;
use tracing::info;
use uuid::Uuid;

use crate::{
    api,
    error::result_handler,
    i18n::{self, Key},
    js,
    service::CONFIG,
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

    let config = CONFIG.read().clone();
    let sales_person_id = props.sales_person_id;

    let i18n = use_context::<i18n::I18n<Key, i18n::Locale>>();
    let form_title = i18n.t(Key::AddExtraHoursFormTitle);
    let category_str = i18n.t(Key::Category);
    let amount_of_hours_str = i18n.t(Key::AmountOfHours);
    let description_str = i18n.t(Key::Description);
    let when_str = i18n.t(Key::When);
    let submit_str = i18n.t(Key::Submit);
    let cancel_str = i18n.t(Key::Cancel);
    let extra_work_str = i18n.t(Key::CategoryExtraWork);
    let vacation_str = i18n.t(Key::CategoryVacation);
    let sick_leave_str = i18n.t(Key::CategorySickLeave);
    let holidays_str = i18n.t(Key::CategoryHolidays);

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
            h1 { class: "text-2xl font-bold", "{form_title}" }

            div { class: "flex flex-col md:flex-row md:border-b-2 border-gray-300 border-dashed mb-1",
                label { class: "block mt-4 mr-4 grow", "{category_str}" }
                select {
                    class: "block mt-2 pl-2 pr-2 w-full md:w-1/2",
                    value: "{category.read().identifier()}",
                    onchange: move |event| {
                        let value = event.data.value();
                        *category.write() = WorkingHoursCategory::from_identifier(&value);
                    },
                    option { value: "extra_work", "{extra_work_str}" }
                    option { value: "holiday", "{holidays_str}" }
                    option { value: "sick_leave", "{sick_leave_str}" }
                    option { value: "vacation", "{vacation_str}" }
                }
            }

            div { class: "flex flex-col md:flex-row md:border-b-2 border-gray-300 border-dashed mb-1",
                label { class: "block mt-4 mr-4 grow", "{amount_of_hours_str}" }
                input {
                    class: "block mt-2 pl-2 pr-2 border border-black w-full md:w-1/2",
                    value: "{amount.read()}",
                    onchange: move |event| {
                        let value = event.data.value().parse::<f32>().unwrap_or(0.0);
                        *amount.write() = value;
                    },
                    "type": "number",
                    "step": "0.001"
                }
            }

            div { class: "flex flex-col md:flex-row md:border-b-2 border-gray-300 border-dashed mb-1",
                label { class: "block mt-4 mr-4 grow", "{description_str}" }
                input {
                    class: "block mt-2 pl-2 pr-2 border border-black w-full md:w-1/2",
                    value: "{description.read()}",
                    onchange: move |event| {
                        let value = event.data.value();
                        *description.write() = value;
                    }
                }
            }

            div { class: "flex flex-col md:flex-row md:border-b-2 border-gray-300 border-dashed mb-1",
                label { class: "block mt-4 mr-4 grow", "{when_str}" }
                input {
                    class: "block mt-2 pl-2 pr-2 border border-black w-full md:w-1/2",
                    value: "{*when.read()}",
                    onchange: move |event| {
                        let value = event.data.value();
                        info!("Setting when to: {value}");
                        *when.write() = value;
                    },
                    "type": "datetime-local"
                }
            }

            div { class: "flex flex-col md:flex-row md:border-b-2 border-gray-300 border-dashed mb-1 mt-8",
                button {
                    class: "block mt-2 pl-2 pr-2 border border-black w-full md:w-1/2",
                    onclick: move |_| props.onabort.call(()),
                    "{cancel_str}"
                }
                button {
                    class: "block mt-2 pl-2 pr-2 border border-black w-full md:w-1/2",
                    prevent_default: "onclick",
                    onclick: move |event| {
                        event.stop_propagation();
                        cr.send(AddExtraHoursFormAction::Submit)
                    },
                    "{submit_str}"
                }
            }
        }
    }
}
