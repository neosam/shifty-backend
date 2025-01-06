use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use uuid::Uuid;

use crate::{
    api,
    error::ShiftyError,
    loader,
    state::{employee_work_details::EmployeeWorkDetails, shiftplan::SalesPerson},
};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

#[derive(Clone, PartialEq)]
pub struct EmployeeWorkDetailsStore {
    pub employee_work_details: Rc<[EmployeeWorkDetails]>,
    pub selected_employee_work_details: EmployeeWorkDetails,
    pub sales_person_id: Uuid,

    pub selected_sales_person: SalesPerson,
}

impl Default for EmployeeWorkDetailsStore {
    fn default() -> Self {
        Self {
            employee_work_details: Rc::new([]),
            selected_employee_work_details: EmployeeWorkDetails::blank_standard(Uuid::nil()),
            sales_person_id: Uuid::nil(),
            selected_sales_person: SalesPerson::default(),
        }
    }
}

pub static EMPLOYEE_WORK_DETAILS_STORE: GlobalSignal<EmployeeWorkDetailsStore> =
    Signal::global(|| EmployeeWorkDetailsStore::default());

async fn load_sales_person_in_employee_work_details(
    sales_person_id: Uuid,
) -> Result<(), ShiftyError> {
    let sales_person = loader::load_sales_person(CONFIG.read().clone(), sales_person_id).await?;
    EMPLOYEE_WORK_DETAILS_STORE.write().selected_sales_person = sales_person;
    Ok(())
}

pub async fn load_employee_work_details(employee_id: Uuid) -> Result<(), ShiftyError> {
    let employee_work_details =
        loader::load_employee_work_details(CONFIG.read().clone(), employee_id).await?;
    EMPLOYEE_WORK_DETAILS_STORE.write().sales_person_id = employee_id;
    EMPLOYEE_WORK_DETAILS_STORE.write().employee_work_details = employee_work_details;
    Ok(())
}

pub async fn reload_employee_work_details() -> Result<(), ShiftyError> {
    let sales_person_id = EMPLOYEE_WORK_DETAILS_STORE.read().sales_person_id;
    load_employee_work_details(sales_person_id).await?;
    super::employee::refresh_employee_data().await?;
    Ok(())
}

async fn new_employee_work_details_for_sales_person(
    sales_person_id: Uuid,
) -> Result<(), ShiftyError> {
    load_sales_person_in_employee_work_details(sales_person_id).await?;
    (*EMPLOYEE_WORK_DETAILS_STORE.write()).selected_employee_work_details =
        EmployeeWorkDetails::blank_standard(sales_person_id);
    Ok(())
}

async fn delete_employee_work_details(employee_work_details_id: Uuid) -> Result<(), ShiftyError> {
    api::delete_employee_work_details(CONFIG.read().clone(), employee_work_details_id).await?;
    reload_employee_work_details().await?;
    Ok(())
}

async fn save_employee_work_details(
    employee_work_details: EmployeeWorkDetails,
) -> Result<(), ShiftyError> {
    loader::save_new_employee_work_details(CONFIG.read().clone(), employee_work_details).await?;
    reload_employee_work_details().await?;
    Ok(())
}

async fn update_employee_work_details(
    employee_work_details: EmployeeWorkDetails,
) -> Result<(), ShiftyError> {
    loader::update_employee_work_details(CONFIG.read().clone(), employee_work_details).await?;
    reload_employee_work_details().await?;
    Ok(())
}

async fn find_and_activate_employee_work_details(
    employee_work_details_id: Uuid,
) -> Result<(), ShiftyError> {
    let employee_work_details_list = EMPLOYEE_WORK_DETAILS_STORE
        .read()
        .employee_work_details
        .clone();
    let employee_work_details = employee_work_details_list
        .iter()
        .find(|details| details.id == employee_work_details_id)
        .to_owned();
    if let Some(employee_work_details) = employee_work_details {
        load_sales_person_in_employee_work_details(employee_work_details.sales_person_id).await?;
        EMPLOYEE_WORK_DETAILS_STORE
            .write()
            .selected_employee_work_details = employee_work_details.to_owned();
    }
    Ok(())
}

pub enum EmployeeWorkDetailsAction {
    NewWorkingHours(Uuid),
    UpdateWorkingHours(EmployeeWorkDetails),
    Save,
    Update,
    Delete(Uuid),
    Load(Uuid),
}

pub async fn employee_work_details_service(mut rx: UnboundedReceiver<EmployeeWorkDetailsAction>) {
    while let Some(action) = rx.next().await {
        match match action {
            EmployeeWorkDetailsAction::NewWorkingHours(sales_person_id) => {
                new_employee_work_details_for_sales_person(sales_person_id).await
            }
            EmployeeWorkDetailsAction::UpdateWorkingHours(working_hours) => {
                dbg!(&working_hours);
                EMPLOYEE_WORK_DETAILS_STORE
                    .write()
                    .selected_employee_work_details = working_hours;
                Ok(())
            }
            EmployeeWorkDetailsAction::Save => {
                let employee_work_details = EMPLOYEE_WORK_DETAILS_STORE
                    .read()
                    .selected_employee_work_details
                    .clone();
                save_employee_work_details(employee_work_details).await
            }
            EmployeeWorkDetailsAction::Update => {
                let employee_work_details = EMPLOYEE_WORK_DETAILS_STORE
                    .read()
                    .selected_employee_work_details
                    .clone();
                update_employee_work_details(employee_work_details).await
            }
            EmployeeWorkDetailsAction::Delete(id) => delete_employee_work_details(id).await,
            EmployeeWorkDetailsAction::Load(employee_work_details_id) => {
                find_and_activate_employee_work_details(employee_work_details_id).await
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
