use dioxus::prelude::*;

use crate::{
    base_types::ImStr,
    component::{base_components::*, TopBar},
    router::Route,
    service::{self, user_management::UserManagementAction},
    state::shiftplan::SalesPerson,
};

#[derive(Clone, PartialEq, Props)]
pub struct SalesPersonDetailsProps {
    pub sales_person_id: String,
}

#[component]
pub fn SalesPersonDetails(props: SalesPersonDetailsProps) -> Element {
    let user_management_service = use_coroutine_handle::<UserManagementAction>();
    let nav = navigator();

    use_effect({
        to_owned![props];
        move || {
            if props.sales_person_id.is_empty() {
                user_management_service.send(UserManagementAction::CreateNewSalesPerson);
            } else {
                user_management_service.send(UserManagementAction::LoadSalesPerson(
                    uuid::Uuid::parse_str(&props.sales_person_id).unwrap(),
                ));
            }
            // Clear any previous save success status
            user_management_service.send(UserManagementAction::ClearSaveSuccess);
        }
    });

    // Redirect immediately when save is successful
    use_effect(move || {
        let user_management = service::user_management::USER_MANAGEMENT_STORE.read();
        if user_management.save_success {
            nav.push(Route::UserManagementPage {});
        }
    });

    let user_management = service::user_management::USER_MANAGEMENT_STORE.read();

    rsx! {
        TopBar {}

        div { class: "px-4 py-4 md:px-6 lg:px-8 max-w-3xl mx-auto",
            // Header with back button
            div { class: "flex items-center mb-6",
                button {
                    class: "mr-3 p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded-md transition-colors",
                    onclick: move |_| { nav.push(Route::UserManagementPage {}); },
                    title: "Back to User Management",
                    "← Back"
                }
                div {
                    h1 { class: "text-2xl md:text-3xl font-bold text-gray-800", "Sales Person Details" }
                    if props.sales_person_id.is_empty() {
                        p { class: "text-lg text-gray-600 mt-1", "Create new sales person" }
                    } else {
                        p { class: "text-lg text-gray-600 mt-1", "Edit sales person information" }
                    }
                }
            }
            
            // Success message - this will show briefly before redirect
            if user_management.save_success {
                div { 
                    class: "mb-6 p-4 bg-green-100 border-l-4 border-green-500 text-green-700 rounded-lg",
                    div { class: "flex items-center",
                        div { class: "mr-2", "✅" }
                        div { "Sales person saved successfully!" }
                    }
                }
            }
            
            // Main content card
            div { class: "bg-white rounded-lg shadow-sm border p-4 md:p-6",
                if let Some(sales_person) = &user_management.sales_person {
                    Form {
                        FormPair { label: "Name".into(),
                            div { class: "w-full", style: "min-width: 0;",
                                TextInput {
                                    value: sales_person.sales_person.name.clone().into(),
                                    on_change: {
                                        to_owned![user_management_service, sales_person];
                                        move |name: ImStr| {
                                            user_management_service
                                                .send(
                                                    UserManagementAction::UpdateSalesPerson(SalesPerson {
                                                        name: name.as_rc(),
                                                        ..sales_person.sales_person.clone()
                                                    }),
                                                );
                                        }
                                    },
                                }
                            }
                        }
                        FormPair { label: "Shiftplan color".into(),
                            div { class: "flex items-center flex-row gap-2",
                                div { class: "flex-1", style: "min-width: 0;",
                                    TextInput {
                                        value: sales_person.sales_person.background_color.clone().into(),
                                        on_change: {
                                            to_owned![user_management_service, sales_person];
                                            move |background_color: ImStr| {
                                                user_management_service
                                                    .send(
                                                        UserManagementAction::UpdateSalesPerson(SalesPerson {
                                                            background_color: background_color.as_rc(),
                                                            ..sales_person.sales_person.clone()
                                                        }),
                                                    );
                                            }
                                        },
                                    }
                                }
                                div {
                                    class: "w-8 h-8 flex-shrink-0 border border-gray-300 rounded",
                                    style: format!("background-color: {}", sales_person.sales_person.background_color.clone()),
                                }
                            }
                        }
                        FormPair { label: "Is paid".into(),
                            Checkbox {
                                value: sales_person.sales_person.is_paid,
                                on_change: {
                                    to_owned![user_management_service, sales_person];
                                    move |is_paid: bool| {
                                        user_management_service
                                            .send(
                                                UserManagementAction::UpdateSalesPerson(SalesPerson {
                                                    is_paid,
                                                    ..sales_person.sales_person.clone()
                                                }),
                                            );
                                    }
                                },
                            }
                        }
                        FormPair { label: "Connected user".into(),
                            if let Some(user_id) = &sales_person.user_id {
                                div { class: "flex flex-col sm:flex-row gap-2",
                                    div { class: "flex-1", style: "min-width: 0;",
                                        TextInput {
                                            on_change: {
                                                to_owned![user_management_service];
                                                move |value: ImStr| {
                                                    user_management_service
                                                        .send(UserManagementAction::UpdateSalesPersonUser(value));
                                                }
                                            },
                                            value: user_id.clone().into(),
                                        }
                                    }
                                    div { class: "flex-shrink-0",
                                        Button {
                                            on_click: {
                                                to_owned![user_management_service];
                                                move |_| {
                                                    user_management_service.send(UserManagementAction::RemoveSalesPersonUser);
                                                }
                                            },
                                            "🗑️ Remove"
                                        }
                                    }
                                }
                            } else {
                                Button {
                                    on_click: {
                                        to_owned![user_management_service];
                                        move |_| {
                                            user_management_service
                                                .send(UserManagementAction::UpdateSalesPersonUser("".into()));
                                        }
                                    },
                                    "Connect user"
                                }
                            }
                        }
                        
                        // Action buttons
                        div { class: "flex flex-col sm:flex-row gap-3 pt-6 border-t",
                            button {
                                class: "w-full sm:w-auto bg-blue-600 hover:bg-blue-700 text-white border-2 border-gray-200 p-2 rounded",
                                onclick: move |_| {
                                    user_management_service.send(UserManagementAction::SaveSalesPersonAndNavigate);
                                },
                                "Save Changes"
                            }
                            button {
                                class: "w-full sm:w-auto border-2 border-gray-200 p-2 rounded",
                                onclick: move |_| {
                                    nav.push(Route::UserManagementPage {});
                                },
                                "Cancel"
                            }
                        }
                    }
                } else {
                    div { class: "text-center py-8",
                        div { class: "text-4xl mb-4", "⏳" }
                        p { class: "text-gray-600", "Loading sales person details..." }
                    }
                }
            }
        }
    }
}
