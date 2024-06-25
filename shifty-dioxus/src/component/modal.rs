use dioxus::prelude::*;

#[derive(Clone, Props, PartialEq)]
pub struct ModalProps {
    pub children: Element,
}

#[component]
pub fn Modal(props: ModalProps) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-10 bg-black bg-opacity-50",
            div {
                class: "bg-white w-full md:w-3/4 m-auto mt-20 p-8",
                { props.children }
            }
        }
    }
}
