use dioxus::prelude::*;

use crate::{
    router::Route,
    state::{auth_info, config},
};

#[component]
pub fn TopBar() -> Element {
    let auth_info = try_use_context::<crate::state::AuthInfo>();
    let config = use_context::<config::Config>();
    let backend_url = config.backend.clone();
    let show_shiftplan = if let Some(ref auth_info) = auth_info {
        auth_info.has_privilege("sales") || auth_info.has_privilege("shiftplanner")
    } else {
        false
    };
    let show_reports = if let Some(ref auth_info) = auth_info {
        auth_info.has_privilege("hr")
    } else {
        false
    };

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
                    if show_shiftplan {
                        li {
                            Link {
                                to: Route::ShiftPlan {},
                                "Shift plan"
                            }
                        }
                    }
                    if show_reports {
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
                        if let Some(auth_info) = auth_info {
                            a {
                                href: "{backend_url}/logout",
                                "Logout {auth_info.user}"
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
