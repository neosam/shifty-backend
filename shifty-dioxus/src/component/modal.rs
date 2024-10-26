use dioxus::prelude::*;

#[derive(Clone, Props, PartialEq)]
pub struct ModalProps {
    pub children: Element,
}

#[component]
pub fn Modal(props: ModalProps) -> Element {
    rsx! {
        div { class: "fixed inset-0 z-10 bg-black bg-opacity-50 flex justify-center items-center md:p-4",
            div { class: "bg-white w-full max-w-3/4 max-h-[90vh] p-8 overflow-y-auto rounded-lg shadow-lg",
                div { class: "", { props.children } }
            }
        }
    }

    /*rsx! {
        div { class: "fixed inset-0 z-10 bg-black bg-opacity-50 flex justify-center items-center p-4",
            div { class: "bg-white w-full h-full md:w-3/4 md:max-h-[calc(100vh-2.5rem)] p-8 overflow-auto rounded",
                div { class: "", { props.children } }
            }
        }
    }*/
    /*rsx! {
        div { class: "fixed inset-0 z-10 bg-black bg-opacity-50 top-0",
            div { class: "relative bg-white w-full h-full md:w-3/4 m-auto p-8 top-0 bottom-0 md:top-10 md:bottom-10 overflow-auto",
                div { class: "", { props.children } }
            }
        }
    }*/
}
