use dioxus::prelude::*;

use crate::{
    api,
    service::{AUTH, CONFIG},
    state,
};

#[derive(PartialEq, Clone, Props)]
pub struct AuthProps {
    authenticated: Element,
    unauthenticated: Element,
}

#[component]
pub fn Auth(props: AuthProps) -> Element {
    let auth = AUTH.read().clone();

    match (auth.auth_info, auth.loading_done) {
        (Some(auth_info), true) => props.authenticated,
        (None, true) => props.unauthenticated,
        (_, false) => {
            rsx! {
                div { "Fetching auth information..." }
            }
        }
    }
}
