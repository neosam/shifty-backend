use dioxus::prelude::*;

use crate::{
    base_types::ImStr,
    component::{base_components::*, TopBar},
    service::{self, UserManagementAction},
    state::shiftplan::SalesPerson,
};

#[derive(Clone, PartialEq, Props)]
pub struct SalesPersonDetailsProps {
    pub sales_person_id: String,
}

#[component]
pub fn SalesPersonDetails(props: SalesPersonDetailsProps) -> Element {
    let user_management_service = use_coroutine_handle::<UserManagementAction>();
    let user_management = service::USER_MANAGEMENT_STORE.read().clone();

    use_effect(move || {
        if props.sales_person_id.is_empty() {
            user_management_service.send(UserManagementAction::CreateNewSalesPerson);
        } else {
            user_management_service.send(UserManagementAction::LoadSalesPerson(
                uuid::Uuid::parse_str(&props.sales_person_id).unwrap(),
            ));
        }
    });

    rsx! {
        TopBar {}

        div { class: "m-4",
            div {
                h1 { class: "text-2xl font-bold", "Sales Person Details" }
                if let Some(sales_person) = user_management.sales_person {
                    Form {
                        FormPair { label: "Name".into(),
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
                                }
                            }
                        }
                        FormPair { label: "Shiftplan color".into(),
                            div { class: "flex items-center flex-row",
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
                                    }
                                }
                                div {
                                    class: "w-6 h-6 ml-2 block",
                                    style: format!("background-color: {}", sales_person.sales_person.background_color.clone())
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
                                }
                            }
                        }
                        FormPair { label: "Connected user".into(),
                            if let Some(user_id) = sales_person.user_id {
                                div {
                                    TextInput {
                                        on_change: {
                                            to_owned![user_management_service];
                                            move |value: ImStr| {
                                                user_management_service
                                                    .send(UserManagementAction::UpdateSalesPersonUser(value));
                                            }
                                        },
                                        value: user_id.clone().into()
                                    }
                                    Button {
                                        on_click: {
                                            to_owned![user_management_service];
                                            move |_| {
                                                user_management_service.send(UserManagementAction::RemoveSalesPersonUser);
                                            }
                                        },
                                        "🗑"
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
                        FormGroup {
                            Button {
                                on_click: {
                                    to_owned![user_management_service];
                                    move |_| {
                                        user_management_service.send(UserManagementAction::SaveSalesPerson);
                                    }
                                },
                                "Save"
                            }
                        }
                    }
                } else {
                    p { "Loading" }
                }
            }
        }
    }
}
