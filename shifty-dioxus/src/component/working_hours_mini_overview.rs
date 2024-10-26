use std::rc::Rc;

use dioxus::prelude::*;
use uuid::Uuid;

use crate::state::employee_work_details::WorkingHoursMini;

#[derive(PartialEq, Clone, Props)]
pub struct WorkingHoursMiniOverviewProps {
    pub working_hours: Rc<[WorkingHoursMini]>,
    #[props(!optional)]
    pub selected_sales_person_id: Option<Uuid>,

    pub on_dbl_click: EventHandler<Uuid>,
}

#[component]
pub fn WorkingHoursMiniOverview(props: WorkingHoursMiniOverviewProps) -> Element {
    let mut working_hours: Vec<WorkingHoursMini> = props.working_hours.iter().cloned().collect();
    working_hours.sort_by(|a, b| a.sales_person_name.cmp(&b.sales_person_name));
    rsx! {
        div { class: "select-none flex flex-col max-w-96",
            for working_hour in working_hours.iter() {
                {
                    let sales_person_id = working_hour.sales_person_id.clone();
                    let actual_hours = format!("{:.1}", working_hour.actual_hours);
                    let expected_hours = format!("{:.1}", working_hour.expected_hours);
                    rsx! { div {
                        class: format!("flex cusor-pointer border-b border-gray-200 border-dashed p-1 {}",
                            if Some(sales_person_id) == props.selected_sales_person_id {
                                "bg-gray-200"
                            } else {
                                ""
                            }),
                        ondoubleclick: move |_| props.on_dbl_click.call(sales_person_id),
                        div { class: "flex-1", "{working_hour.sales_person_name}" }
                        div {
                            class: format!("flex flex-row {}", if working_hour.actual_hours < working_hour.expected_hours {
                                "text-red-800"
                            } else {
                                "text-green-800"
                            }),
                            div { class: "flex-1", {actual_hours} }
                            div { class: "flex-1", "/" }
                            div { class: "flex-1", {expected_hours} }
                        }
                    } }
                }
            }
        }
    }
}
