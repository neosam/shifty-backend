use dioxus::prelude::*;

use crate::{api, state::State};

#[derive(PartialEq, Clone, Props)]
pub struct AuthProps {
    authenticated: Element,
    unauthenticated: Element,
}

#[component]
pub fn Auth(props: AuthProps) -> Element {
    let mut state = use_context::<Signal<State>>();
    let backend = state.read().config.backend.clone();

    let auth_info = {
        let backend = backend.clone();
        use_resource(move || api::fetch_auth_info(backend.clone()))
    };

    match &*auth_info.read_unchecked() {
        Some(Ok(Some(auth_info))) => {
            if !state.read().auth_info.authenticated {
                state.write().auth_info = auth_info.clone();
            }
            {
                props.authenticated
            }
        }
        Some(Ok(None)) => props.unauthenticated,
        Some(Err(err)) => {
            rsx! { div { "Error while fetching username: {err}" } }
        }
        None => {
            rsx! { div { "Fetching auth information..." } }
        }
    }
}
