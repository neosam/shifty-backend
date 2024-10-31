use dioxus::prelude::*;

use crate::{
    i18n::Key,
    loader,
    router::Route,
    service::{AUTH, CONFIG, I18N},
};

#[component]
pub fn TopBar() -> Element {
    let i18n = I18N.read().clone();
    let auth_info = AUTH.read().auth_info.clone();
    let config = CONFIG.read().clone();
    let show_my_time = config.show_my_time.unwrap_or(true);
    let backend_url = config.backend.clone();
    let non_production_warning_str = i18n.t(Key::NonProdWarning);
    let non_production_warning_detail_str = i18n.t(Key::NonProdWarningDetails);
    let employee = {
        let config = config.clone();
        use_resource(move || loader::load_current_sales_person(config.to_owned()))
    };
    let mut visible = use_signal(|| false);
    let show_shiftplan = if let Some(ref auth_info) = auth_info {
        auth_info.has_privilege("sales") || auth_info.has_privilege("shiftplanner")
    } else {
        false
    };
    let show_weekly_overview = if let Some(ref auth_info) = auth_info {
        auth_info.has_privilege("shiftplanner") || auth_info.has_privilege("sales")
    } else {
        false
    };
    let show_reports = if let Some(ref auth_info) = auth_info {
        auth_info.has_privilege("hr")
    } else {
        false
    };
    let is_paid = if let Some(Ok(Some(employee))) = &*employee.read_unchecked() {
        employee.is_paid
    } else {
        false
    };
    let show_user_management = if let Some(ref auth_info) = auth_info {
        auth_info.has_privilege("admin")
    } else {
        false
    };

    rsx! {
        div { class: "flex bg-gray-800 text-white p-4 md:p-0 items-center print:hidden",
            button {
                class: "md:hidden pr-6 pl-4 text-xl",
                onclick: move |_| {
                    let visibility = *visible.read();
                    visible.set(!visibility)
                },
                "☰"
            }

            h1 { class: "text-2xl font-bold ml-2",
                "Shifty"
                if !config.is_prod {
                    span { class: "ml-2 text-sm", "{config.env_short_description}" }
                }
            }

            nav {
                class: "hidden bg-gray-800 md:pl-0 p-4 md:grow md:ml-4 md:justify-between md:flex",
                style: if *visible.read() {
                    "display: flex; flex-direction: column; position: absolute; left: 0px; top: 64px;"
                } else {
                    ""
                },
                ul { class: "flex flex-col md:flex-row space-y-4 md:space-y-0 md:space-x-4 ml-1",
                    if show_shiftplan {
                        li {
                            Link { to: Route::ShiftPlan {}, {i18n.t(Key::Shiftplan)} }
                        }
                    }
                    if is_paid && !show_reports && show_my_time {
                        li {
                            Link { to: Route::MyEmployeeDetails {}, {i18n.t(Key::MyTime)} }
                        }
                    }
                    if show_weekly_overview {
                        li {
                            Link { to: Route::WeeklyOverview {}, {i18n.t(Key::YearOverview)} }
                        }
                    }
                    if show_reports {
                        li {
                            Link { to: Route::Employees {}, {i18n.t(Key::Employees)} }
                        }
                    }
                    if show_user_management {
                        li {
                            Link { to: Route::UserManagementPage {}, {"User Management (beta)"} }
                        }
                    }
                    if auth_info.is_some() {
                        div { class: "mb-6 md:mb-0" }
                    }
                }
                ul { class: "ml-1",
                    li { class: "flex",
                        if let Some(auth_info) = auth_info {
                            a { href: "{backend_url}/logout", "Logout {auth_info.user}" }
                        } else {
                            a { href: "/authenticate", "Login" }
                        }
                    }
                }
            }
        }
        if !config.is_prod {
            div { class: "bg-yellow-200 text-yellow-800 pl-4 p-1 print:hidden",
                div {
                    class: "font-bold",
                    title: "{non_production_warning_detail_str}",
                    {non_production_warning_str}
                }
            }
        }
        div {
        }
    }
}
