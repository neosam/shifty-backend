use dioxus::prelude::*;
use rest_types::InvitationStatus;

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
    let mut expiration_hours = use_signal(|| "24".to_string());
    let copied_invitation_id = use_signal(|| None);

    use_effect({
        to_owned![user_management_service, props];
        move || {
            user_management_service.send(UserManagementAction::LoadUserRoleAssignments(
                props.user_id.to_owned().into(),
            ));
            user_management_service.send(UserManagementAction::LoadUserInvitations(
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

                // User Invitations Section
                div { class: "mt-8 border-t pt-6",
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-xl font-bold text-gray-800", "{i18n.t(Key::UserInvitations)}" }
                        span { class: "text-sm text-gray-500 bg-gray-100 px-2 py-1 rounded", 
                            {
                                i18n.t(Key::InvitationsCount).replace("{count}", &user_management.user_invitations.len().to_string())
                            } 
                        }
                    }

                    if user_management.user_invitations.is_empty() {
                        div { class: "text-center py-8 text-gray-500",
                            div { class: "text-4xl mb-2", "✉️" }
                            p { "{i18n.t(Key::NoInvitationsFound)}" }
                            p { class: "text-sm", "{i18n.t(Key::GenerateFirstInvitation)}" }
                        }
                    } else {
                        div { class: "space-y-3 mb-4",
                            for invitation in user_management.user_invitations.iter() {
                                div { class: "p-4 bg-gray-50 rounded-lg",
                                    div { class: "flex items-start justify-between",
                                        div { class: "flex-1",
                                            div { class: "flex items-center gap-2 mb-2",
                                                span { 
                                                    class: match invitation.status {
                                                        InvitationStatus::Valid => "text-xs bg-green-100 text-green-800 px-2 py-1 rounded-full",
                                                        InvitationStatus::Expired => "text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded-full",
                                                        InvitationStatus::Redeemed => "text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded-full",
                                                    },
                                                    {
                                                        match invitation.status {
                                                            InvitationStatus::Valid => i18n.t(Key::Valid),
                                                            InvitationStatus::Expired => i18n.t(Key::Expired),
                                                            InvitationStatus::Redeemed => i18n.t(Key::Redeemed),
                                                        }
                                                    }
                                                }
                                                if let Some(redeemed_at) = invitation.redeemed_at {
                                                    span { class: "text-xs text-gray-500",
                                                        "{i18n.t(Key::Redeemed)}: {redeemed_at}"
                                                    }
                                                }
                                            }
                                            div { class: "flex items-center gap-2",
                                                input { 
                                                    class: "flex-1 px-3 py-2 text-sm bg-white border border-gray-300 rounded-md font-mono text-gray-600",
                                                    readonly: true,
                                                    value: "{invitation.invitation_link}",
                                                }
                                                button {
                                                    class: "px-3 py-2 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                                                    onclick: {
                                                        to_owned![invitation, copied_invitation_id];
                                                        move |_| {
                                                            let link = invitation.invitation_link.clone();
                                                            let id = invitation.id;
                                                            spawn(async move {
                                                                if let Ok(_) = crate::js::copy_to_clipboard(&link).await {
                                                                    copied_invitation_id.set(Some(id));
                                                                    // Reset after 2 seconds
                                                                    gloo_timers::future::sleep(std::time::Duration::from_secs(2)).await;
                                                                    copied_invitation_id.set(None);
                                                                }
                                                            });
                                                        }
                                                    },
                                                    if copied_invitation_id.read().as_ref() == Some(&invitation.id) {
                                                        "{i18n.t(Key::InvitationCopied)}"
                                                    } else {
                                                        "{i18n.t(Key::CopyToClipboard)}"
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "flex gap-2 ml-4",
                                            if invitation.status == InvitationStatus::Valid {
                                                button {
                                                    class: "px-3 py-2 text-sm bg-red-600 text-white rounded-md hover:bg-red-700 transition-colors",
                                                    onclick: {
                                                        to_owned![user_management_service, invitation];
                                                        move |_| {
                                                            user_management_service.send(UserManagementAction::RevokeInvitation(invitation.id));
                                                        }
                                                    },
                                                    "{i18n.t(Key::RevokeInvitation)}"
                                                }
                                            }
                                            if invitation.status == InvitationStatus::Redeemed {
                                                button {
                                                    class: "px-3 py-2 text-sm bg-orange-600 text-white rounded-md hover:bg-orange-700 transition-colors",
                                                    onclick: {
                                                        to_owned![user_management_service, invitation];
                                                        move |_| {
                                                            user_management_service.send(UserManagementAction::RevokeInvitationSession(invitation.id));
                                                        }
                                                    },
                                                    "{i18n.t(Key::RevokeSession)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Generate New Invitation
                    div { class: "border-t pt-4",
                        h3 { class: "text-sm font-semibold text-gray-700 mb-3", "{i18n.t(Key::GenerateNewInvitation)}" }
                        div { class: "flex flex-col sm:flex-row gap-2",
                            div { class: "flex-1",
                                label { 
                                    class: "block text-sm font-medium text-gray-700 mb-1", 
                                    "{i18n.t(Key::ExpirationHours)}" 
                                }
                                input {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    r#type: "number",
                                    value: "{expiration_hours.read()}",
                                    required: true,
                                    min: "1",
                                    oninput: move |event| {
                                        expiration_hours.set(event.value());
                                    },
                                }
                            }
                            Button {
                                on_click: {
                                    to_owned![user_management_service, props, expiration_hours];
                                    move |_| {
                                        let hours = expiration_hours.read().parse::<i64>().ok();
                                        user_management_service.send(
                                            UserManagementAction::GenerateInvitation(
                                                props.user_id.to_owned().into(),
                                                hours,
                                            )
                                        );
                                        expiration_hours.set("24".to_string());
                                    }
                                },
                                "{i18n.t(Key::GenerateInvitation)}"
                            }
                        }
                    }
                }
            }
        }
    }
}
