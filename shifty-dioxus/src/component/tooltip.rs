use dioxus::prelude::*;
use tracing::error;
use web_sys::window;

use crate::service::tooltip::{TooltipAction, TOOLTIP};

#[component]
pub fn TooltipBase() -> Element {
    let tooltip = TOOLTIP.read().clone();
    let tooltip_service = use_coroutine_handle::<TooltipAction>();
    let width = window().unwrap().inner_width().unwrap().as_f64().unwrap();

    use_effect({
        move || {
            let tooltip = TOOLTIP.read().clone();
            if let Some(tooltip) = &tooltip {
                let tooltip_base = window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .get_element_by_id("tooltip-base");

                if let Some(tooltip_base) = tooltip_base {
                    let tooltip_width = tooltip_base.client_width() as f64;
                    let x = if tooltip.x + tooltip_width > width {
                        width - tooltip_width - 10.0 // Add 10px padding from edge
                    } else {
                        tooltip.x
                    };

                    if let Err(_) = tooltip_base
                        .set_attribute("style", &format!("top: {}px; left: {}px", tooltip.y, x))
                    {
                        error!("Failed to set tooltip position");
                    }
                }
            }
        }
    });

    if let Some(tooltip) = tooltip.clone() {
        rsx! {
            div {
                class: "absolute inset-0 z-40",
                onclick: {
                    to_owned![tooltip_service];
                    move |_| tooltip_service.send(TooltipAction::HideTooltip)
                },
                div {
                    class: "absolute z-50 bg-white border border-gray-300 shadow-lg rounded-md p-3 max-w-xs",
                    id: "tooltip-base",
                    style: "pointer-events: none;",
                    "{tooltip.content}"
                }
            }
        }
    } else {
        VNode::empty()
    }
}
