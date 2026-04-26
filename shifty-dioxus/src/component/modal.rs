use dioxus::prelude::*;

#[derive(Clone, Props, PartialEq)]
pub struct ModalProps {
    pub children: Element,
}

/// **Legacy** — kept for unmigrated call sites. New code SHOULD use
/// [`crate::component::dialog::Dialog`], which uses design tokens, supports
/// four layout variants (`Center` / `Sheet` / `Bottom` / `Auto`), header
/// and footer slots, body scroll lock, and standard dismissal paths
/// (backdrop click, ESC, X button).
///
/// This component will be removed once all call sites are migrated as
/// part of redesign changes 05–09 (see `openspec/changes/REDESIGN_PLAN.md`).
#[component]
pub fn Modal(props: ModalProps) -> Element {
    rsx! {
        div { class: "fixed inset-0 z-50 bg-black bg-opacity-50 flex justify-center items-center md:p-4",
            div { class: "bg-white w-full max-w-3/4 max-h-[90vh] p-8 overflow-y-auto rounded-lg shadow-lg",
                div { class: "", { props.children } }
            }
        }
    }

    /*rsx! {
        div { class: "fixed inset-0 z-50 bg-black bg-opacity-50 flex justify-center items-center p-4",
            div { class: "bg-white w-full h-full md:w-3/4 md:max-h-[calc(100vh-2.5rem)] p-8 overflow-auto rounded",
                div { class: "", { props.children } }
            }
        }
    }*/
    /*rsx! {
        div { class: "fixed inset-0 z-50 bg-black bg-opacity-50 top-0",
            div { class: "relative bg-white w-full h-full md:w-3/4 m-auto p-8 top-0 bottom-0 md:top-10 md:bottom-10 overflow-auto",
                div { class: "", { props.children } }
            }
        }
    }*/
}
