use dioxus::prelude::*;
use dioxus_elements::u;

use crate::{
    component::base_components::*,
    component::TopBar,
    service::{UserManagementAction, USER_MANAGEMENT_STORE},
};

#[derive(Clone, PartialEq, Props)]
pub struct UserDetailsProps {
    pub user_id: String,
}

#[component]
pub fn UserDetails(props: UserDetailsProps) -> Element {
    let user_management_service = use_coroutine_handle::<UserManagementAction>();
    let user_management = USER_MANAGEMENT_STORE.read().clone();

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

        div { class: "m-4",
            div {
                h1 { class: "text-2xl font-bold", "User Details {props.user_id}" }
                p { class: "mt-8 mb-8", "This is a page that shows the details of a user." }

                h2 { class: "text-xl font-bold", "Role Assignments" }

                for role_assignment in user_management.role_assignements.iter() {
                    FormPair { label: role_assignment.role.clone().as_str().into(),
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
                            }
                        }
                    }
                }

                button { class: "bg-red-500 hover:bg-red-700 text-white font-bold py-2 px-4 rounded",
                    "Delete"
                }
            }
        }
    }
}
