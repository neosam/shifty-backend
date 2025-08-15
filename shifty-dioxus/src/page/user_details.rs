use dioxus::prelude::*;

use crate::{
    component::base_components::*,
    component::TopBar,
    i18n::Key,
    router::Route,
    service::{
        user_management::{UserManagementAction, USER_MANAGEMENT_STORE},
        i18n::I18N,
    },
};

#[derive(Clone, PartialEq, Props)]
pub struct UserDetailsProps {
    pub user_id: String,
}

#[component]
pub fn UserDetails(props: UserDetailsProps) -> Element {
    let user_management_service = use_coroutine_handle::<UserManagementAction>();
    let user_management = USER_MANAGEMENT_STORE.read().clone();
    let nav = navigator();
    let i18n = I18N.read().clone();

    use_effect({
        to_owned![user_management_service, props];
        move || {
            user_management_service.send(UserManagementAction::LoadUserRoleAssignments(
                props.user_id.to_owned().into(),
            ));
        }
    });

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
                    h1 { class: "text-2xl md:text-3xl font-bold text-gray-800", "{i18n.t(Key::UserDetails)}" }
                    p { class: "text-lg text-gray-600 mt-1", "{props.user_id}" }
                }
            }

            // Main content card
            div { class: "bg-white rounded-lg shadow-sm border p-4 md:p-6",
                div { class: "mb-6",
                    p { class: "text-gray-600", "{i18n.t(Key::ManageRolesAndPermissions)}" }
                }

                // Role Assignments Section
                div {
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-xl font-bold text-gray-800", "{i18n.t(Key::RoleAssignments)}" }
                        span { class: "text-sm text-gray-500 bg-gray-100 px-2 py-1 rounded", 
                            {
                                let assigned = user_management.role_assignements.iter().filter(|r| r.assigned).count();
                                let total = user_management.role_assignements.len();
                                i18n.t(Key::RolesCount).replace("{assigned}", &assigned.to_string()).replace("{total}", &total.to_string())
                            } 
                        }
                    }

                    if user_management.role_assignements.is_empty() {
                        div { class: "text-center py-8 text-gray-500",
                            div { class: "text-4xl mb-2", "🔐" }
                            p { "{i18n.t(Key::NoRolesAvailable)}" }
                            p { class: "text-sm", "{i18n.t(Key::ContactAdministratorForRoles)}" }
                        }
                    } else {
                        div { class: "space-y-3",
                            for role_assignment in user_management.role_assignements.iter() {
                                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors",
                                    div { class: "flex items-center flex-1",
                                        Checkbox {
                                            value: role_assignment.assigned,
                                            on_change: {
                                                to_owned![user_management_service, role_assignment, props];
                                                move |assigned: bool| {
                                                    if assigned {
                                                        user_management_service
                                                            .send(
                                                                UserManagementAction::AssignUserToRole(
                                                                    props.user_id.to_owned().into(),
                                                                    role_assignment.role.clone().into(),
                                                                ),
                                                            );
                                                    } else {
                                                        user_management_service
                                                            .send(
                                                                UserManagementAction::RemoveUserFromRole(
                                                                    props.user_id.to_owned().into(),
                                                                    role_assignment.role.clone().into(),
                                                                ),
                                                            );
                                                    }
                                                }
                                            },
                                        }
                                        span { 
                                            class: if role_assignment.assigned { 
                                                "ml-3 font-medium text-gray-800" 
                                            } else { 
                                                "ml-3 text-gray-600" 
                                            },
                                            "{role_assignment.role}" 
                                        }
                                    }
                                    if role_assignment.assigned {
                                        span { class: "text-xs bg-green-100 text-green-800 px-2 py-1 rounded-full", 
                                            "{i18n.t(Key::Active)}" 
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
