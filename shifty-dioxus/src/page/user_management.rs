use crate::{
    component::{base_components::Button, TopBar},
    router::Route,
    service::{UserManagementAction, USER_MANAGEMENT_STORE},
};
use dioxus::prelude::*;

#[component]
pub fn UserManagementPage() -> Element {
    let user_management_service = use_coroutine_handle::<UserManagementAction>();
    let user_management = USER_MANAGEMENT_STORE.read().clone();

    use_effect(move || {
        user_management_service.send(UserManagementAction::LoadAllUsers);
        user_management_service.send(UserManagementAction::LoadAllSalesPersons);
    });

    rsx! {
        TopBar {}

        div { class: "ml-1 mr-1 pt-4 md:m-8",
            h1 { class: "text-2xl font-bold mb-4", "User Management (beta)" }

            p { class: "text-red-500 mb-4",
                "User management is somehow working but not completed yet.  Please double check if changes you make are actually active."
            }

            div { class: "flex justify-between gap-4",
                div { class: "grow",
                    h2 { class: "text-xl font-bold mb-2", "Users" }

                    ul { class: "flex flex-col",
                        for user in user_management.users.iter() {
                            Link {
                                to: Route::UserDetails {
                                    user_id: user.username.to_string(),
                                },
                                li { class: "center p-2 border-b ", "{user.username}" }
                            }
                        }
                    }
                }
                div { class: "grow",
                    h2 { class: "text-xl font-bold mb-2", "Sales person" }

                    ul { class: "flex flex-col",
                        for sales_person in user_management.sales_persons.iter() {
                            Link {
                                to: Route::SalesPersonDetails {
                                    sales_person_id: sales_person.id.to_string(),
                                },
                                li { class: "center p-2 border-b ", "{sales_person.name}" }
                            }
                        }
                        Link {
                            to: Route::SalesPersonDetails {
                                sales_person_id: "".to_string(),
                            },
                            Button { "Create new sales person" }
                        }
                    }
                }
            }
        }
    }
}
