use dioxus::prelude::*;

use crate::api;
use crate::router::Route;
use crate::state::State;

#[component]
pub fn Home() -> Element {
    let mut count = use_signal(|| 0);
    let state = use_context::<State>();
    let backend = state.config.backend;

    let auth_info = {
        let backend = backend.clone();
        use_resource(move || api::fetch_auth_info(backend.clone()))
    };

    rsx! {
        Link {
            to: Route::Blog {
                id: count()
            },
            "Go to blog"
        }
        div {
            h1 { "High-Five counter: {count} {backend}" }
            {
                /*match &*auth_info.read_unchecked() {
                    Some(Ok(auth_info)) => {
                            rsx! { div { "Logged in as {auth_info.user}" } }
                    }
                    Some(Err(err)) => {
                            rsx! { div { "Error while fetching username: {err}" } }
                    }
                    None => {
                            rsx! { div { "Fetching username..." } }
                    }
                }*/
            }
            button { onclick: move |_| count += 1, "Up high!" }
            button { onclick: move |_| count -= 1, "Down low!" }
        }
    }
}
