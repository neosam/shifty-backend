use std::rc::Rc;

use dioxus::prelude::*;
use rest_types::ShiftplanTO;
use uuid::Uuid;

#[derive(Clone, PartialEq, Props)]
pub struct ShiftplanTabBarProps {
    pub shiftplans: Rc<[ShiftplanTO]>,
    pub selected_id: Option<Uuid>,
    pub on_select: EventHandler<Uuid>,
}

#[component]
pub fn ShiftplanTabBar(props: ShiftplanTabBarProps) -> Element {
    rsx! {
        div { class: "flex border-b border-gray-300 mb-2",
            for shiftplan in props.shiftplans.iter() {
                {
                    let is_active = props.selected_id == Some(shiftplan.id);
                    let id = shiftplan.id;
                    rsx! {
                        button {
                            class: if is_active {
                                "px-4 py-2 text-sm font-medium border-b-2 border-blue-500 text-blue-600"
                            } else {
                                "px-4 py-2 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                            },
                            onclick: move |_| props.on_select.call(id),
                            "{shiftplan.name}"
                        }
                    }
                }
            }
        }
    }
}
