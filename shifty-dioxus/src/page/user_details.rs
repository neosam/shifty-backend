use dioxus::prelude::*;

use crate::component::TopBar;

#[derive(Clone, PartialEq, Props)]
pub struct UserDetailsProps {
    pub user_id: String,
}

#[component]
pub fn UserDetails(props: UserDetailsProps) -> Element {
    rsx! {
        TopBar {}

        div { class: "m-4",
            div {
                h1 { class: "text-2xl font-bold", "User Details {props.user_id}" }
                p { class: "mt-8 mb-8", "This is a page that shows the details of a user." }
                button { class: "bg-red-500 hover:bg-red-700 text-white font-bold py-2 px-4 rounded",
                    "Delete"
                }
            }
        }
    }
}
