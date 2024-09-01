use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use uuid::Uuid;

use crate::base_types::ImStr;
use crate::state::shiftplan::SalesPerson;
use crate::state::User;
use crate::{
    api,
    error::ShiftyError,
    i18n::{self, I18nType},
    loader,
    state::{
        dropdown::{Dropdown, DropdownEntry},
        working_hours::WorkingHoursMini,
        AuthInfo, Config,
    },
};

pub async fn load_auth_info() {
    if CONFIG.read().backend.is_empty() {
        return;
    }
    let auth_info = api::fetch_auth_info(CONFIG.read().backend.clone()).await;

    match auth_info {
        Ok(Some(auth_info)) => {
            *AUTH.write() = AuthStore {
                auth_info: Some(auth_info),
                loading_done: true,
            };
        }
        Ok(None) => {
            *AUTH.write() = AuthStore {
                auth_info: None,
                loading_done: true,
            };
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
            *AUTH.write() = AuthStore {
                auth_info: None,
                loading_done: true,
            };
        }
    }
}
pub async fn load_config() {
    let config = api::load_config().await;
    match config {
        Ok(config) => {
            *CONFIG.write() = config;
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
        }
    }
    *CONFIG.write() = api::load_config().await.unwrap();
    load_auth_info().await;
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct ErrorStore {
    pub error: Option<ShiftyError>,
}
pub static ERROR_STORE: GlobalSignal<ErrorStore> = Signal::global(|| ErrorStore::default());

#[allow(dead_code)]
pub enum ErrorAction {
    SetError(ShiftyError),
}

#[allow(dead_code)]
pub async fn error_service(mut rx: UnboundedReceiver<ErrorAction>) {
    while let Some(action) = rx.next().await {
        match action {
            ErrorAction::SetError(error) => {
                *ERROR_STORE.write() = ErrorStore { error: Some(error) };
            }
        }
    }
}

pub static DROPDOWN: GlobalSignal<Option<Dropdown>> = Signal::global(|| None);

pub enum DropdownAction {
    CloseDropdown,
    ToggleDropdown(f64, f64, Rc<[DropdownEntry]>),
}

pub async fn open_dropdown(x: f64, y: f64, entries: Rc<[DropdownEntry]>) {
    *DROPDOWN.write() = Some(Dropdown { x, y, entries });
}
pub async fn close_dropdown() {
    *DROPDOWN.write() = None;
}
pub async fn toggle_dropdown(x: f64, y: f64, entries: Rc<[DropdownEntry]>) {
    if DROPDOWN.read().is_some() {
        close_dropdown().await;
    } else {
        open_dropdown(x, y, entries).await;
    }
}

pub async fn dropdown_service(mut rx: UnboundedReceiver<DropdownAction>) {
    while let Some(action) = rx.next().await {
        match action {
            DropdownAction::CloseDropdown => close_dropdown().await,
            DropdownAction::ToggleDropdown(x, y, entries) => toggle_dropdown(x, y, entries).await,
        }
    }
}

// Config service
pub static CONFIG: GlobalSignal<Config> = Signal::global(|| Config::default());
#[allow(dead_code)]
pub enum ConfigAction {
    LoadConfig,
}
pub async fn config_service(mut rx: UnboundedReceiver<ConfigAction>) {
    load_config().await;

    while let Some(action) = rx.next().await {
        match action {
            ConfigAction::LoadConfig => {
                load_config().await;
            }
        }
    }
}

pub static I18N: GlobalSignal<I18nType> = Signal::global(|| i18n::generate(i18n::Locale::En));

pub async fn i18n_service(_rx: UnboundedReceiver<()>) {
    let set_browser_language = || async {
        let language = web_sys::window()
            .map(|w| w.navigator())
            .and_then(|n| n.language())
            .map(|locale| locale[..2].to_string())
            .unwrap_or_else(|| "en".to_string());
        let i18n = i18n::generate(i18n::Locale::from_str(&language));
        *I18N.write() = i18n;
    };

    set_browser_language().await;
}

#[derive(Default, Clone, Eq, PartialEq)]
pub struct AuthStore {
    pub auth_info: Option<AuthInfo>,
    pub loading_done: bool,
}

pub static AUTH: GlobalSignal<AuthStore> = Signal::global(|| AuthStore::default());

#[allow(dead_code)]
pub async fn auth_service(_rx: UnboundedReceiver<()>) {
    load_auth_info().await;
}

pub static WORKING_HOURS_MINI: GlobalSignal<Rc<[WorkingHoursMini]>> = Signal::global(|| [].into());
pub enum WorkingHoursMiniAction {
    // Load working hours for a specific week (year, week)
    LoadWorkingHoursMini(u32, u8),
}

pub async fn working_hours_mini_service(mut rx: UnboundedReceiver<WorkingHoursMiniAction>) {
    while let Some(action) = rx.next().await {
        match action {
            WorkingHoursMiniAction::LoadWorkingHoursMini(year, week) => {
                let working_hours =
                    loader::load_working_hours_minified_for_week(CONFIG.read().clone(), year, week)
                        .await;
                match working_hours {
                    Ok(working_hours) => {
                        *WORKING_HOURS_MINI.write() = working_hours;
                    }
                    Err(err) => {
                        *ERROR_STORE.write() = ErrorStore {
                            error: Some(err.into()),
                        };
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct SelectedSalesPerson {
    pub sales_person: SalesPerson,
    pub user_id: Option<ImStr>,
}
impl SelectedSalesPerson {
    pub fn new(sales_person: SalesPerson) -> Self {
        Self {
            sales_person,
            user_id: None,
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
}
pub static USER_MANAGEMENT_STORE: GlobalSignal<UserManagementStore> =
    Signal::global(|| UserManagementStore::default());

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

pub async fn save_sales_person() {
    let selected_sales_person = USER_MANAGEMENT_STORE.read().sales_person.clone();
    let loaded_sales_person = USER_MANAGEMENT_STORE.read().loaded_sales_person.clone();
    if let (Some(selected_sales_person), Some(loaded_sales_person)) =
        (selected_sales_person, loaded_sales_person)
    {
        if selected_sales_person != loaded_sales_person {
            match loader::save_sales_person(
                CONFIG.read().clone(),
                selected_sales_person.sales_person.clone(),
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    *ERROR_STORE.write() = ErrorStore {
                        error: Some(err.into()),
                    };
                }
            }
            match (
                selected_sales_person.user_id.clone(),
                loaded_sales_person.user_id.clone(),
            ) {
                (Some(new_user_id), Some(old_user_id)) => {
                    if new_user_id != old_user_id {
                        match loader::save_user_for_sales_person(
                            CONFIG.read().clone(),
                            selected_sales_person.sales_person.id,
                            new_user_id,
                        )
                        .await
                        {
                            Ok(_) => {}
                            Err(err) => {
                                *ERROR_STORE.write() = ErrorStore {
                                    error: Some(err.into()),
                                };
                            }
                        }
                    }
                }
                (Some(user_id), None) => {
                    match loader::save_user_for_sales_person(
                        CONFIG.read().clone(),
                        selected_sales_person.sales_person.id,
                        user_id,
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(err) => {
                            *ERROR_STORE.write() = ErrorStore {
                                error: Some(err.into()),
                            };
                        }
                    }
                }
                (None, Some(_)) => {
                    match loader::remove_user_from_sales_person(
                        CONFIG.read().clone(),
                        selected_sales_person.sales_person.id,
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(err) => {
                            *ERROR_STORE.write() = ErrorStore {
                                error: Some(err.into()),
                            };
                        }
                    }
                }
                _ => {}
            }
        }
    }
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

pub enum UserManagementAction {
    LoadAllUsers,
    LoadAllSalesPersons,
    LoadSalesPerson(Uuid),
    UpdateSalesPerson(SalesPerson),
    UpdateSalesPersonUser(ImStr),
    RemoveSalesPersonUser,
    SaveSalesPerson,
    CreateNewSalesPerson,
    LoadUserRoleAssignments(ImStr),
    AssignUserToRole(ImStr, ImStr),
    RemoveUserFromRole(ImStr, ImStr),
}

pub async fn user_management_service(mut rx: UnboundedReceiver<UserManagementAction>) {
    while let Some(action) = rx.next().await {
        match match action {
            UserManagementAction::LoadAllUsers => {
                load_all_users().await;
                Ok(())
            }
            UserManagementAction::LoadAllSalesPersons => {
                load_all_sales_persons().await;
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
            UserManagementAction::SaveSalesPerson => {
                save_sales_person().await;
                Ok(())
            }
            UserManagementAction::CreateNewSalesPerson => {
                let new_sales_person = SalesPerson {
                    id: Uuid::nil(),
                    name: "Name".into(),
                    background_color: "#FFF".into(),
                    is_paid: false,
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
