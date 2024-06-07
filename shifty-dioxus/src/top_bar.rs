use dioxus::prelude::*;

use crate::{router::Route, state::State};

#[component]
pub fn TopBar() -> Element {
    let mut state = use_context::<Signal<State>>();

    rsx! {
        div {
            class: "flex bg-gray-800 text-white p-4 items-center",
            h1 {
                class: "text-2xl font-bold",
                "Shifty"
            }

            nav {
                class: "flex grow ml-4 justify-between",
                ul {
                    class: "flex space-x-4",
                    if (state.read().auth_info.has_privilege("sales")) {
                        li {
                            Link {
                                to: Route::Home {},
                                "Shift plan"
                            }
                        }
                    }
                    if (state.read().auth_info.has_privilege("hr")) {
                        li {
                            Link {
                                to: Route::Home {},
                                "Reports"
                            }
                        }
                    }
                    li {

                    }
                }
                ul {
                    li {
                        class: "flex",
                        if state.read().auth_info.authenticated {
                            a {
                                href: "/logout",
                                "Logout {state.read().auth_info.user}"
                            }
                        } else {
                            a {
                                href: "/authenticate",
                                "Login"
                            }
                        }
                    }
                }
            }
        }
    }
}
