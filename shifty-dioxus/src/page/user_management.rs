use crate::{
    base_types::ImStr,
    component::{
        base_components::{Button, TextInput},
        TopBar,
    },
    i18n::Key,
    router::Route,
    service::{
        user_management::{UserManagementAction, USER_MANAGEMENT_STORE},
        i18n::I18N,
    },
};
use dioxus::prelude::*;

#[component]
pub fn UserManagementPage() -> Element {
    let user_management_service = use_coroutine_handle::<UserManagementAction>();
    let user_management = USER_MANAGEMENT_STORE.read().clone();
    let add_user_value: Signal<ImStr> = use_signal(|| "".into());
    let i18n = I18N.read().clone();

    use_effect(move || {
        user_management_service.send(UserManagementAction::LoadAllUsers);
        user_management_service.send(UserManagementAction::LoadAllSalesPersons);
    });

    rsx! {
        TopBar {}

        div { class: "px-4 py-4 md:px-6 lg:px-8 max-w-5xl mx-auto",
            h1 { class: "text-2xl md:text-3xl font-bold mb-6 text-center md:text-left", "{i18n.t(Key::UserManagement)}" }

            // Mobile-first responsive layout: stack vertically on mobile, side by side on desktop
            div { class: "flex flex-col lg:flex-row gap-4 lg:gap-6",
                
                // Users Section
                div { class: "flex-1 bg-white rounded-lg shadow-sm border p-4 md:p-6",
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-xl font-bold text-gray-800", "{i18n.t(Key::Users)}" }
                        span { class: "text-sm text-gray-500 bg-gray-100 px-2 py-1 rounded", 
                            {
                                i18n.t(Key::UsersCount).replace("{count}", &user_management.users.len().to_string())
                            } 
                        }
                    }

                    // Users List
                    if user_management.users.is_empty() {
                        div { class: "text-center py-8 text-gray-500",
                            div { class: "text-4xl mb-2", "👥" }
                            p { "{i18n.t(Key::NoUsersFound)}" }
                            p { class: "text-sm", "{i18n.t(Key::AddFirstUserBelow)}" }
                        }
                    } else {
                        ul { class: "space-y-2 mb-4",
                            for user in user_management.users.iter() {
                                li { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors",
                                    Link {
                                        to: Route::UserDetails {
                                            user_id: user.username.to_string(),
                                        },
                                        class: "flex-1 font-medium text-blue-600 hover:text-blue-800 hover:underline truncate mr-3",
                                        "{user.username}"
                                    }
                                    button {
                                        class: "flex-shrink-0 p-2 text-red-600 hover:bg-red-100 rounded-md transition-colors",
                                        onclick: {
                                            to_owned![user_management_service, user];
                                            move |_| {
                                                user_management_service
                                                    .send(UserManagementAction::DeleteUser(user.username.clone()));
                                            }
                                        },
                                        title: "{i18n.t(Key::DeleteUser)}",
                                        "🗑️"
                                    }
                                }
                            }
                        }
                    }

                    // Add User Form
                    div { class: "border-t pt-4",
                        h3 { class: "text-sm font-semibold text-gray-700 mb-3", "{i18n.t(Key::AddNewUser)}" }
                        div { class: "flex flex-col sm:flex-row gap-2",
                            div { class: "flex-1",
                                div { class: "w-full", style: "min-width: 0;",
                                    TextInput {
                                        value: add_user_value.read().clone(),
                                        on_change: {
                                            to_owned![add_user_value];
                                            move |value: ImStr| {
                                                *add_user_value.write() = value;
                                            }
                                        },
                                    }
                                }
                            }
                            Button {
                                on_click: {
                                    to_owned![user_management_service, add_user_value];
                                    move |_| {
                                        if !add_user_value.read().as_str().trim().is_empty() {
                                            user_management_service
                                                .send(UserManagementAction::AddUser(add_user_value.read().clone()));
                                            *add_user_value.write() = "".into();
                                        }
                                    }
                                },
                                "{i18n.t(Key::CreateUser)}"
                            }
                        }
                    }
                }

                // Sales Persons Section
                div { class: "flex-1 bg-white rounded-lg shadow-sm border p-4 md:p-6",
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-xl font-bold text-gray-800", "{i18n.t(Key::SalesPersons)}" }
                        span { class: "text-sm text-gray-500 bg-gray-100 px-2 py-1 rounded", 
                            {
                                i18n.t(Key::SalesPersonsCount).replace("{count}", &user_management.sales_persons.len().to_string())
                            } 
                        }
                    }

                    // Sales Persons List
                    if user_management.sales_persons.is_empty() {
                        div { class: "text-center py-8 text-gray-500 mb-4",
                            div { class: "text-4xl mb-2", "👤" }
                            p { "{i18n.t(Key::NoSalesPersonsFound)}" }
                            p { class: "text-sm", "{i18n.t(Key::CreateFirstSalesPersonBelow)}" }
                        }
                    } else {
                        ul { class: "space-y-2 mb-4",
                            for sales_person in user_management.sales_persons.iter() {
                                Link {
                                    to: Route::SalesPersonDetails {
                                        sales_person_id: sales_person.id.to_string(),
                                    },
                                    li { class: "flex items-center p-3 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors",
                                        div { 
                                            class: "w-4 h-4 rounded-full mr-3 flex-shrink-0",
                                            style: "background-color: {sales_person.background_color}",
                                        }
                                        span { class: "font-medium text-blue-600 hover:text-blue-800 truncate", 
                                            "{sales_person.name}" 
                                        }
                                        div { class: "ml-auto flex items-center gap-2 flex-shrink-0",
                                            if sales_person.is_paid {
                                                span { class: "text-xs bg-green-100 text-green-800 px-2 py-1 rounded-full", 
                                                    "💰" 
                                                }
                                            }
                                            if sales_person.inactive {
                                                span { class: "text-xs bg-red-100 text-red-800 px-2 py-1 rounded-full", 
                                                    "{i18n.t(Key::Inactive)}" 
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Add Sales Person Button
                    div { class: "border-t pt-4",
                        Link {
                            to: Route::SalesPersonDetails {
                                sales_person_id: "".to_string(),
                            },
                            div { class: "w-full sm:w-auto",
                                Button { 
                                    "{i18n.t(Key::CreateNewSalesPerson)}" 
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

