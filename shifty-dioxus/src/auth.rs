use dioxus::prelude::*;

use crate::service::AUTH;

#[derive(PartialEq, Clone, Props)]
pub struct AuthProps {
    authenticated: Element,
    unauthenticated: Element,
}

#[component]
pub fn Auth(props: AuthProps) -> Element {
    let auth = AUTH.read().clone();

    match (auth.auth_info, auth.loading_done) {
        (Some(_auth_info), true) => props.authenticated,
        (None, true) => props.unauthenticated,
        (_, false) => {
            rsx! {
                div { "Fetching auth information..." }
            }
        }
    }
}
