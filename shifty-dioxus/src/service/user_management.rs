use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;
use futures::future::join_all;
use futures_util::StreamExt;
use rest_types::InvitationResponse;
use uuid::Uuid;

use rest_types::ShiftplanTO;

use crate::{
    base_types::ImStr,
    error::ShiftyError,
    loader,
    state::{shiftplan::SalesPerson, ShiftplanAssignment, User},
};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

#[derive(Clone, PartialEq)]
pub struct SelectedSalesPerson {
    pub sales_person: SalesPerson,
    pub user_id: Option<ImStr>,
    pub shiftplan_assignments: Vec<ShiftplanAssignment>,
}
impl SelectedSalesPerson {
    pub fn new(sales_person: SalesPerson) -> Self {
        Self {
            sales_person,
            user_id: None,
            shiftplan_assignments: Vec::new(),
        }
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct RoleAssignment {
    pub role: ImStr,
    pub assigned: bool,
}

#[derive(Default, Clone, PartialEq)]
pub struct UserManagementStore {
    pub users: Rc<[User]>,
    pub sales_persons: Rc<[SalesPerson]>,
    pub sales_person: Option<SelectedSalesPerson>,
    pub loaded_sales_person: Option<SelectedSalesPerson>,
    pub role_assignements: Rc<[RoleAssignment]>,
    pub user_invitations: Rc<[InvitationResponse]>,
    pub save_success: bool,
    pub shiftplan_catalog: Rc<[ShiftplanTO]>,

    /// Maps a `SalesPerson` id to the linked user login (if any).
    /// Populated by `LoadAllSalesPersonUserLinks`.
    pub sales_person_user_links: Rc<HashMap<Uuid, Option<ImStr>>>,

    /// Maps a user login to the linked `SalesPerson` (if any).
    /// Populated by `LoadAllUserSalesPersonLinks`.
    pub user_sales_person_links: Rc<HashMap<ImStr, Option<SalesPerson>>>,

    /// Maps a user login to the assigned roles list.
    /// Populated by `LoadAllUserRoles`.
    pub user_role_assignments: Rc<HashMap<ImStr, Rc<[ImStr]>>>,
}
pub static USER_MANAGEMENT_STORE: GlobalSignal<UserManagementStore> =
    Signal::global(|| UserManagementStore::default());

pub async fn load_shiftplan_catalog() {
    let config = CONFIG.read().clone();
    match crate::api::get_all_shiftplans(config).await {
        Ok(catalog) => {
            USER_MANAGEMENT_STORE.write().shiftplan_catalog = catalog;
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
        }
    }
}

pub async fn load_shiftplan_assignments(sales_person_id: Uuid) {
    let config = CONFIG.read().clone();
    match crate::api::get_shiftplan_assignments(config, sales_person_id).await {
        Ok(assignments) => {
            let mut store = USER_MANAGEMENT_STORE.write();
            if let Some(sp) = store.sales_person.as_mut() {
                sp.shiftplan_assignments = assignments.clone();
            }
            if let Some(sp) = store.loaded_sales_person.as_mut() {
                sp.shiftplan_assignments = assignments;
            }
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
        }
    }
}

pub async fn load_all_users() {
    let users = loader::load_all_users(CONFIG.read().clone()).await;
    match users {
        Ok(users) => {
            USER_MANAGEMENT_STORE.write().users = users.into();
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
        }
    }
}

pub async fn load_all_sales_persons() {
    let sales_persons = loader::load_sales_persons(CONFIG.read().clone()).await;
    match sales_persons {
        Ok(sales_persons) => {
            USER_MANAGEMENT_STORE.write().sales_persons = sales_persons.into();
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
        }
    }
}

/// Pure split: `Ok` entries land in the map, `Err` entries land in the error
/// vec. Caller is responsible for forwarding errors to `ERROR_STORE`. Kept
/// as a free function so it can be unit-tested without touching globals.
fn partition_results<K, V>(
    results: Vec<(K, Result<V, ShiftyError>)>,
) -> (HashMap<K, V>, Vec<ShiftyError>)
where
    K: std::hash::Hash + Eq,
{
    let mut map: HashMap<K, V> = HashMap::with_capacity(results.len());
    let mut errors: Vec<ShiftyError> = Vec::new();
    for (key, res) in results {
        match res {
            Ok(value) => {
                map.insert(key, value);
            }
            Err(err) => errors.push(err),
        }
    }
    (map, errors)
}

fn report_errors(errors: Vec<ShiftyError>) {
    for err in errors {
        *ERROR_STORE.write() = ErrorStore {
            error: Some(err.into()),
        };
    }
}

pub async fn load_all_sales_person_user_links() {
    let sales_persons = USER_MANAGEMENT_STORE.read().sales_persons.clone();
    let config = CONFIG.read().clone();

    let fetches = sales_persons.iter().map(|sp| {
        let cfg = config.clone();
        let id = sp.id;
        async move { (id, loader::load_user_for_sales_person(cfg, id).await) }
    });
    let results = join_all(fetches).await;

    let (map, errors) = partition_results(results);
    report_errors(errors);
    USER_MANAGEMENT_STORE.write().sales_person_user_links = Rc::new(map);
}

pub async fn load_all_user_sales_person_links() {
    let users = USER_MANAGEMENT_STORE.read().users.clone();
    let config = CONFIG.read().clone();

    let fetches = users.iter().map(|u| {
        let cfg = config.clone();
        let username = u.username.clone();
        async move {
            let res = loader::load_sales_person_by_user(cfg, username.clone()).await;
            (username, res)
        }
    });
    let results = join_all(fetches).await;

    let (map, errors) = partition_results(results);
    report_errors(errors);
    USER_MANAGEMENT_STORE.write().user_sales_person_links = Rc::new(map);
}

pub async fn load_all_user_roles() {
    let users = USER_MANAGEMENT_STORE.read().users.clone();
    let config = CONFIG.read().clone();

    let fetches = users.iter().map(|u| {
        let cfg = config.clone();
        let username = u.username.clone();
        async move {
            let res = loader::load_roles_from_user(cfg, username.clone()).await;
            (username, res)
        }
    });
    let results = join_all(fetches).await;

    let (map, errors) = partition_results(results);
    report_errors(errors);
    USER_MANAGEMENT_STORE.write().user_role_assignments = Rc::new(map);
}

pub async fn load_sales_person(sales_person_id: Uuid) {
    let sales_person = loader::load_sales_person(CONFIG.read().clone(), sales_person_id).await;
    match sales_person {
        Ok(sales_person) => {
            USER_MANAGEMENT_STORE.write().sales_person =
                Some(SelectedSalesPerson::new(sales_person.clone()));
            USER_MANAGEMENT_STORE.write().loaded_sales_person =
                Some(SelectedSalesPerson::new(sales_person));
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
        }
    }

    let user = loader::load_user_for_sales_person(CONFIG.read().clone(), sales_person_id).await;
    match user {
        Ok(user) => {
            if let Some(user) = user {
                USER_MANAGEMENT_STORE
                    .write()
                    .sales_person
                    .as_mut()
                    .unwrap()
                    .user_id = Some(user.clone());
                USER_MANAGEMENT_STORE
                    .write()
                    .loaded_sales_person
                    .as_mut()
                    .unwrap()
                    .user_id = Some(user);
            }
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
        }
    }
}

pub async fn save_sales_person() -> Result<(), ShiftyError> {
    let selected_sales_person = USER_MANAGEMENT_STORE.read().sales_person.clone();
    let loaded_sales_person = USER_MANAGEMENT_STORE.read().loaded_sales_person.clone();
    if let (Some(selected_sales_person), Some(loaded_sales_person)) =
        (selected_sales_person, loaded_sales_person)
    {
        if selected_sales_person != loaded_sales_person {
            let saved_id = loader::save_sales_person(
                CONFIG.read().clone(),
                selected_sales_person.sales_person.clone(),
            )
            .await?;

            match (
                selected_sales_person.user_id.clone(),
                loaded_sales_person.user_id.clone(),
            ) {
                (Some(new_user_id), Some(old_user_id)) => {
                    if new_user_id != old_user_id {
                        loader::save_user_for_sales_person(
                            CONFIG.read().clone(),
                            saved_id,
                            new_user_id,
                        )
                        .await?;
                    }
                }
                (Some(user_id), None) => {
                    loader::save_user_for_sales_person(CONFIG.read().clone(), saved_id, user_id)
                        .await?;
                }
                (None, Some(_)) => {
                    loader::remove_user_from_sales_person(CONFIG.read().clone(), saved_id).await?;
                }
                _ => {}
            }

            // Save shiftplan assignments
            if selected_sales_person.shiftplan_assignments
                != loaded_sales_person.shiftplan_assignments
            {
                crate::api::set_shiftplan_assignments(
                    CONFIG.read().clone(),
                    saved_id,
                    &selected_sales_person.shiftplan_assignments,
                )
                .await?;
            }
        }
    }
    Ok(())
}

pub async fn load_role_assignments(user: ImStr) -> Result<(), ShiftyError> {
    let config = CONFIG.read().clone();
    let roles = loader::load_all_roles(config.clone()).await?;
    let user_roles = loader::load_roles_from_user(config.clone(), user).await?;
    let mut role_assignments = roles
        .iter()
        .map(|role| RoleAssignment {
            role: role.clone(),
            assigned: user_roles.contains(&role),
        })
        .collect::<Vec<_>>();
    role_assignments.sort_by_key(|role| role.role.clone());
    USER_MANAGEMENT_STORE.write().role_assignements = role_assignments.into();
    Ok(())
}

pub async fn assign_user_to_role(user: ImStr, role: ImStr) -> Result<(), ShiftyError> {
    loader::add_user_to_role(CONFIG.read().clone(), user, role).await?;
    Ok(())
}

pub async fn remove_user_from_role(user: ImStr, role: ImStr) -> Result<(), ShiftyError> {
    loader::remove_user_from_role(CONFIG.read().clone(), user, role).await?;
    Ok(())
}

pub async fn add_user(user: ImStr) -> Result<(), ShiftyError> {
    loader::add_user(CONFIG.read().clone(), user).await?;
    Ok(())
}

pub async fn delete_user(user: ImStr) -> Result<(), ShiftyError> {
    // First check if the user is connected to a sales person
    let sales_person =
        loader::load_sales_person_by_user(CONFIG.read().clone(), user.clone()).await?;

    // If connected to a sales person, remove the connection first
    if let Some(sales_person) = sales_person {
        loader::remove_user_from_sales_person(CONFIG.read().clone(), sales_person.id).await?;
    }

    // Now proceed with deleting the user
    loader::delete_user(CONFIG.read().clone(), user).await?;
    Ok(())
}

pub async fn load_user_invitations(username: ImStr) {
    let invitations = loader::load_user_invitations(CONFIG.read().clone(), username).await;
    match invitations {
        Ok(invitations) => {
            USER_MANAGEMENT_STORE.write().user_invitations = invitations;
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
        }
    }
}

pub async fn generate_user_invitation(
    username: ImStr,
    expiration_hours: Option<i64>,
) -> Result<(), ShiftyError> {
    loader::generate_invitation(CONFIG.read().clone(), username.clone(), expiration_hours).await?;
    // Reload invitations to show the new one
    load_user_invitations(username).await;
    Ok(())
}

pub async fn revoke_user_invitation(
    invitation_id: Uuid,
    username: ImStr,
) -> Result<(), ShiftyError> {
    loader::revoke_invitation(CONFIG.read().clone(), invitation_id).await?;
    // Reload invitations to update the list
    load_user_invitations(username).await;
    Ok(())
}

pub async fn revoke_user_invitation_session(
    invitation_id: Uuid,
    username: ImStr,
) -> Result<(), ShiftyError> {
    loader::revoke_invitation_session(CONFIG.read().clone(), invitation_id).await?;
    // Reload invitations to update the status
    load_user_invitations(username).await;
    Ok(())
}

pub enum UserManagementAction {
    LoadAllUsers,
    LoadAllSalesPersons,
    LoadSalesPerson(Uuid),
    UpdateSalesPerson(SalesPerson),
    UpdateSalesPersonUser(ImStr),
    RemoveSalesPersonUser,
    SaveSalesPerson,
    SaveSalesPersonAndNavigate,
    ClearSaveSuccess,
    CreateNewSalesPerson,
    LoadUserRoleAssignments(ImStr),
    AssignUserToRole(ImStr, ImStr),
    RemoveUserFromRole(ImStr, ImStr),
    AddUser(ImStr),
    DeleteUser(ImStr),
    LoadUserInvitations(ImStr),
    GenerateInvitation(ImStr, Option<i64>),
    RevokeInvitation(Uuid),
    RevokeInvitationSession(Uuid),
    LoadShiftplanCatalog,
    LoadShiftplanAssignments(Uuid),
    UpdateShiftplanAssignments(Vec<ShiftplanAssignment>),
    LoadAllSalesPersonUserLinks,
    LoadAllUserSalesPersonLinks,
    LoadAllUserRoles,
}

pub async fn user_management_service(mut rx: UnboundedReceiver<UserManagementAction>) {
    while let Some(action) = rx.next().await {
        match match action {
            UserManagementAction::LoadAllUsers => {
                load_all_users().await;
                load_all_user_sales_person_links().await;
                load_all_user_roles().await;
                Ok(())
            }
            UserManagementAction::LoadAllSalesPersons => {
                load_all_sales_persons().await;
                load_all_sales_person_user_links().await;
                Ok(())
            }
            UserManagementAction::LoadSalesPerson(sales_person_id) => {
                load_sales_person(sales_person_id).await;
                Ok(())
            }
            UserManagementAction::UpdateSalesPerson(sales_person) => {
                if USER_MANAGEMENT_STORE.read().sales_person.is_none() {
                    USER_MANAGEMENT_STORE.write().sales_person =
                        Some(SelectedSalesPerson::new(sales_person));
                } else {
                    USER_MANAGEMENT_STORE
                        .write()
                        .sales_person
                        .as_mut()
                        .unwrap()
                        .sales_person = sales_person;
                }
                Ok(())
            }
            UserManagementAction::UpdateSalesPersonUser(user_id) => {
                USER_MANAGEMENT_STORE
                    .write()
                    .sales_person
                    .as_mut()
                    .unwrap()
                    .user_id = Some(user_id);
                Ok(())
            }
            UserManagementAction::RemoveSalesPersonUser => {
                USER_MANAGEMENT_STORE
                    .write()
                    .sales_person
                    .as_mut()
                    .unwrap()
                    .user_id = None;
                Ok(())
            }
            UserManagementAction::SaveSalesPerson => match save_sales_person().await {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            },
            UserManagementAction::SaveSalesPersonAndNavigate => {
                match save_sales_person().await {
                    Ok(_) => {
                        // Refresh the sales persons list after successful save
                        load_all_sales_persons().await;
                        USER_MANAGEMENT_STORE.write().save_success = true;
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            }
            UserManagementAction::ClearSaveSuccess => {
                USER_MANAGEMENT_STORE.write().save_success = false;
                Ok(())
            }
            UserManagementAction::CreateNewSalesPerson => {
                let new_sales_person = SalesPerson {
                    id: Uuid::nil(),
                    name: "Name".into(),
                    background_color: "#FFF".into(),
                    is_paid: false,
                    inactive: false,
                    version: Uuid::nil(),
                };
                USER_MANAGEMENT_STORE.write().sales_person =
                    Some(SelectedSalesPerson::new(new_sales_person.clone()));
                USER_MANAGEMENT_STORE.write().loaded_sales_person =
                    Some(SelectedSalesPerson::new(new_sales_person));
                Ok(())
            }
            UserManagementAction::LoadUserRoleAssignments(user) => {
                load_role_assignments(user).await
            }
            UserManagementAction::AssignUserToRole(user, role) => {
                assign_user_to_role(user, role).await
            }
            UserManagementAction::RemoveUserFromRole(user, role) => {
                remove_user_from_role(user, role).await
            }
            UserManagementAction::AddUser(user) => match add_user(user).await {
                Ok(()) => {
                    load_all_users().await;
                    Ok(())
                }
                Err(err) => Err(err),
            },
            UserManagementAction::DeleteUser(user) => match delete_user(user).await {
                Ok(()) => {
                    load_all_users().await;
                    Ok(())
                }
                Err(err) => Err(err),
            },
            UserManagementAction::LoadUserInvitations(username) => {
                load_user_invitations(username).await;
                Ok(())
            }
            UserManagementAction::GenerateInvitation(username, expiration_hours) => {
                generate_user_invitation(username, expiration_hours).await
            }
            UserManagementAction::RevokeInvitation(invitation_id) => {
                // We need to get the username from somewhere to reload invitations
                // For now, we'll get it from the first invitation in the store
                let username = USER_MANAGEMENT_STORE
                    .read()
                    .user_invitations
                    .first()
                    .map(|inv| inv.username.clone().into())
                    .unwrap_or_else(|| "".into());
                revoke_user_invitation(invitation_id, username).await
            }
            UserManagementAction::RevokeInvitationSession(invitation_id) => {
                // We need to get the username from somewhere to reload invitations
                // For now, we'll get it from the first invitation in the store
                let username = USER_MANAGEMENT_STORE
                    .read()
                    .user_invitations
                    .first()
                    .map(|inv| inv.username.clone().into())
                    .unwrap_or_else(|| "".into());
                revoke_user_invitation_session(invitation_id, username).await
            }
            UserManagementAction::LoadShiftplanCatalog => {
                load_shiftplan_catalog().await;
                Ok(())
            }
            UserManagementAction::LoadShiftplanAssignments(sales_person_id) => {
                load_shiftplan_assignments(sales_person_id).await;
                Ok(())
            }
            UserManagementAction::UpdateShiftplanAssignments(assignments) => {
                if let Some(sp) = USER_MANAGEMENT_STORE.write().sales_person.as_mut() {
                    sp.shiftplan_assignments = assignments;
                }
                Ok(())
            }
            UserManagementAction::LoadAllSalesPersonUserLinks => {
                load_all_sales_person_user_links().await;
                Ok(())
            }
            UserManagementAction::LoadAllUserSalesPersonLinks => {
                load_all_user_sales_person_links().await;
                Ok(())
            }
            UserManagementAction::LoadAllUserRoles => {
                load_all_user_roles().await;
                Ok(())
            }
        } {
            Ok(_) => {}
            Err(err) => {
                *ERROR_STORE.write() = ErrorStore {
                    error: Some(err.into()),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_error() -> ShiftyError {
        // ComponentRange is constructible via an out-of-range time; cheaper
        // than fabricating a reqwest::Error.
        time::Time::from_hms(99, 0, 0).unwrap_err().into()
    }

    #[test]
    fn partition_results_empty_yields_empty_map() {
        let input: Vec<(Uuid, Result<Option<ImStr>, ShiftyError>)> = Vec::new();
        let (map, errors) = partition_results(input);
        assert!(map.is_empty());
        assert!(errors.is_empty());
    }

    #[test]
    fn partition_results_keeps_ok_entries() {
        let id_a = Uuid::new_v4();
        let id_b = Uuid::new_v4();
        let input: Vec<(Uuid, Result<Option<ImStr>, ShiftyError>)> =
            vec![(id_a, Ok(Some(ImStr::from("alex")))), (id_b, Ok(None))];
        let (map, errors) = partition_results(input);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&id_a), Some(&Some(ImStr::from("alex"))));
        assert_eq!(map.get(&id_b), Some(&None));
        assert!(errors.is_empty());
    }

    #[test]
    fn partition_results_skips_err_entries_but_keeps_others() {
        let id_a = Uuid::new_v4();
        let id_b = Uuid::new_v4();
        let id_c = Uuid::new_v4();
        let input: Vec<(Uuid, Result<Option<ImStr>, ShiftyError>)> = vec![
            (id_a, Ok(Some(ImStr::from("alex")))),
            (id_b, Err(sample_error())),
            (id_c, Ok(None)),
        ];
        let (map, errors) = partition_results(input);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key(&id_a));
        assert!(!map.contains_key(&id_b));
        assert!(map.contains_key(&id_c));
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn partition_results_keys_match_input() {
        let id_a = Uuid::new_v4();
        let id_b = Uuid::new_v4();
        let input: Vec<(Uuid, Result<u32, ShiftyError>)> = vec![(id_a, Ok(1)), (id_b, Ok(2))];
        let (map, _) = partition_results(input);
        let keys: std::collections::HashSet<Uuid> = map.keys().copied().collect();
        let expected: std::collections::HashSet<Uuid> = [id_a, id_b].into_iter().collect();
        assert_eq!(keys, expected);
    }

    #[test]
    fn partition_results_supports_imstr_keys() {
        let input: Vec<(ImStr, Result<Rc<[ImStr]>, ShiftyError>)> = vec![
            (ImStr::from("alex"), Ok(vec![ImStr::from("admin")].into())),
            (ImStr::from("bob"), Err(sample_error())),
        ];
        let (map, errors) = partition_results(input);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key(&ImStr::from("alex")));
        assert!(!map.contains_key(&ImStr::from("bob")));
        assert_eq!(errors.len(), 1);
    }
}
