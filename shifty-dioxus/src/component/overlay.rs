use dioxus::prelude::*;

// Props
#[derive(Clone, Debug, PartialEq, Props)]
pub struct OverlayProps {
    pub children: Element,
}

// Component which shows a overlay with a black background and 50% opacity and the children in the middle
#[component]
pub fn Overlay(props: OverlayProps) -> Element {
    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 z-50", {props.children} }
    }
}
