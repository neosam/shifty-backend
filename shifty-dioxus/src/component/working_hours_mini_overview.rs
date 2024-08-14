use std::rc::Rc;

use dioxus::prelude::*;
use uuid::Uuid;

use crate::state::working_hours::WorkingHoursMini;

#[derive(PartialEq, Clone, Props)]
pub struct WorkingHoursMiniOverviewProps {
    pub working_hours: Rc<[WorkingHoursMini]>,

    pub on_dbl_click: EventHandler<Uuid>,
}

#[component]
pub fn WorkingHoursMiniOverview(props: WorkingHoursMiniOverviewProps) -> Element {
    let working_hours = props.working_hours.clone();
    rsx! {
        div { class: "select-none flex flex-col max-w-96",
            for working_hour in working_hours.iter() {
                {
                    let sales_person_id = working_hour.sales_person_id.clone();
                    rsx! { div {
                        class: "flex cusor-pointer border-b border-gray-200 border-dashed pt-2",
                        ondoubleclick: move |_| props.on_dbl_click.call(sales_person_id),
                        div { class: "flex-1", "{working_hour.sales_person_name}" }
                        div { class: "flex-1", "{working_hour.actual_hours.to_string()}/{working_hour.expected_hours.to_string()}" }
                    } }
                }
            }
        }
    }
}
