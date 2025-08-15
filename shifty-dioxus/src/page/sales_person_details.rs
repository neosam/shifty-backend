use dioxus::prelude::*;

use crate::{
    base_types::ImStr,
    component::{base_components::*, TopBar},
    i18n::Key,
    router::Route,
    service::{self, user_management::UserManagementAction, i18n::I18N},
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
    let i18n = I18N.read().clone();

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
                    title: "{i18n.t(Key::BackToUserManagement)}",
                    "← {i18n.t(Key::BackToUserManagement)}"
                }
                div {
                    h1 { class: "text-2xl md:text-3xl font-bold text-gray-800", "{i18n.t(Key::SalesPersonDetails)}" }
                    if props.sales_person_id.is_empty() {
                        p { class: "text-lg text-gray-600 mt-1", "{i18n.t(Key::CreateNewSalesPersonTitle)}" }
                    } else {
                        p { class: "text-lg text-gray-600 mt-1", "{i18n.t(Key::EditSalesPersonInformation)}" }
                    }
                }
            }
            
            // Success message - this will show briefly before redirect
            if user_management.save_success {
                div { 
                    class: "mb-6 p-4 bg-green-100 border-l-4 border-green-500 text-green-700 rounded-lg",
                    div { class: "flex items-center",
                        div { class: "mr-2", "✅" }
                        div { "{i18n.t(Key::SalesPersonSavedSuccessfully)}" }
                    }
                }
            }
            
            // Main content card
            div { class: "bg-white rounded-lg shadow-sm border p-4 md:p-6",
                if let Some(sales_person) = &user_management.sales_person {
                    Form {
                        // Basic Information Section
                        div { class: "mb-6",
                            h2 { class: "text-lg font-semibold text-gray-800 mb-4 pb-2 border-b border-gray-200", 
                                "{i18n.t(Key::BasicInformation)}" 
                            }
                            
                            FormPair { label: i18n.t(Key::Name).into(),
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
                            
                            FormPair { label: i18n.t(Key::ShiftplanColor).into(),
                                div { class: "flex items-center gap-3",
                                    div {
                                        class: "w-6 h-6 border border-gray-300 rounded flex-shrink-0",
                                        style: format!("background-color: {}", sales_person.sales_person.background_color.clone()),
                                        title: "{i18n.t(Key::ColorPreview)}",
                                    }
                                    div { class: "flex-1",
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
                                }
                            }
                        }
                        
                        // Settings Section
                        div { class: "mb-6",
                            h2 { class: "text-lg font-semibold text-gray-800 mb-4 pb-2 border-b border-gray-200", 
                                "{i18n.t(Key::Settings)}" 
                            }
                            
                            div { class: "border-b-2 border-gray-200 border-dashed p-2",
                                Checkbox {
                                    value: Some(sales_person.sales_person.is_paid),
                                    on_change: Some({
                                        to_owned![user_management_service, sales_person];
                                        EventHandler::new(move |is_paid: bool| {
                                            user_management_service
                                                .send(
                                                    UserManagementAction::UpdateSalesPerson(SalesPerson {
                                                        is_paid,
                                                        ..sales_person.sales_person.clone()
                                                    }),
                                                );
                                        })
                                    }),
                                    "{i18n.t(Key::ThisPersonReceivesPayment)}"
                                }
                            }
                            
                            div { class: "border-b-2 border-gray-200 border-dashed p-2",
                                Checkbox {
                                    value: Some(sales_person.sales_person.inactive),
                                    on_change: Some({
                                        to_owned![user_management_service, sales_person];
                                        EventHandler::new(move |inactive: bool| {
                                            user_management_service
                                                .send(
                                                    UserManagementAction::UpdateSalesPerson(SalesPerson {
                                                        inactive,
                                                        ..sales_person.sales_person.clone()
                                                    }),
                                                );
                                        })
                                    }),
                                    "{i18n.t(Key::ThisPersonIsInactive)}"
                                }
                            }
                            
                            FormPair { label: i18n.t(Key::UserAccount).into(),
                                if let Some(user_id) = &sales_person.user_id {
                                    div { class: "flex gap-2",
                                        Button {
                                            on_click: move |_| {
                                                user_management_service.send(UserManagementAction::RemoveSalesPersonUser);
                                            },
                                            "🗑️"
                                        }
                                        div { class: "flex-1",
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
                                    }
                                } else {
                                    Button {
                                        on_click: move |_| {
                                            user_management_service
                                                .send(UserManagementAction::UpdateSalesPersonUser("".into()));
                                        },
                                        "{i18n.t(Key::ConnectUserAccount)}"
                                    }
                                }
                            }
                        }
                        
                        // Action buttons section
                        div { class: "pt-6 border-t border-gray-200",
                            div { class: "flex flex-col sm:flex-row gap-3 justify-end",
                                Button {
                                    on_click: move |_| {
                                        nav.push(Route::UserManagementPage {});
                                    },
                                    "{i18n.t(Key::Cancel)}"
                                }
                                Button {
                                    primary: true,
                                    on_click: move |_| {
                                        user_management_service.send(UserManagementAction::SaveSalesPersonAndNavigate);
                                    },
                                    "{i18n.t(Key::SaveChanges)}"
                                }
                            }
                        }
                    }
                } else {
                    div { class: "text-center py-12",
                        div { class: "text-6xl mb-4 text-gray-300", "⏳" }
                        p { class: "text-lg text-gray-500", "{i18n.t(Key::LoadingSalesPersonDetails)}" }
                        div { class: "mt-4 animate-pulse",
                            div { class: "h-2 bg-gray-200 rounded w-24 mx-auto" }
                        }
                    }
                }
            }
        }
    }
}
