use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use tracing::info;
use uuid::Uuid;

use crate::{
    api,
    error::ShiftyError,
    js, loader,
    state::{
        employee::{Employee, ExtraHours, CustomExtraHoursDefinition},
        shiftplan::SalesPerson,
    },
};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

#[derive(Clone, PartialEq)]
pub struct EmployeeStore {
    pub year: u32,
    pub until_week: u8,

    pub employee: Employee,
    pub extra_hours: Rc<[ExtraHours]>,
    pub custom_extra_hours_definitions: Rc<[CustomExtraHoursDefinition]>,
}

pub static EMPLOYEE_STORE: GlobalSignal<EmployeeStore> = Signal::global(|| EmployeeStore {
    year: 0,
    until_week: 0,
    employee: Employee {
        sales_person: SalesPerson::default(),
        working_hours_by_week: Rc::new([]),
        working_hours_by_month: Rc::new([]),
        overall_working_hours: 0.0,
        expected_working_hours: 0.0,
        balance: 0.0,
        carryover_balance: 0.0,
        shiftplan_hours: 0.0,
        extra_work_hours: 0.0,
        vacation_hours: 0.0,
        sick_leave_hours: 0.0,
        holiday_hours: 0.0,
        vacation_days: 0.0,
        vacation_entitlement: 0.0,
        vacation_carryover: 0,
        custom_extra_hours: [].into(),
    },
    extra_hours: Rc::new([]),
    custom_extra_hours_definitions: Rc::new([]),
});

#[derive(Debug)]
pub enum EmployeeAction {
    LoadEmployeeDataUntilNow { sales_person_id: Uuid },
    LoadCurrentEmployeeDataUntilNow,
    Refresh,
    DeleteExtraHours(Uuid),
    DeleteCustomExtraHour(Uuid),
    FullYear,
    UntilNow,
    NextYear,
    PrevYear,
}

pub async fn load_employee_data(
    sales_person_id: Uuid,
    year: u32,
    until_week: u8,
) -> Result<(), ShiftyError> {
    let employee =
        loader::load_employee_details(CONFIG.read().clone(), year, until_week, sales_person_id)
            .await?;
    let extra_hours =
        loader::load_extra_hours_per_year(CONFIG.read().clone(), year, sales_person_id).await?;
    let custom_extra_hours_definitions = match api::get_custom_extra_hours_by_sales_person(CONFIG.read().clone(), sales_person_id).await {
        Ok(hours) => {
            let definitions: Rc<[CustomExtraHoursDefinition]> = hours
                .iter()
                .map(|h| h.into())
                .collect();
            definitions
        }
        Err(e) => {
            info!("Failed to load custom extra hours definitions: {}", e);
            Rc::new([])
        }
    };
    super::employee_work_details::load_employee_work_details(sales_person_id).await?;
    *EMPLOYEE_STORE.write() = EmployeeStore {
        employee,
        extra_hours,
        custom_extra_hours_definitions,
        year,
        until_week,
    };
    Ok(())
}

pub async fn load_current_employee_data() -> Result<(), ShiftyError> {
    if let Some(sales_person) = loader::load_current_sales_person(CONFIG.read().clone()).await? {
        let year = js::get_current_year();
        let until_week = js::get_current_week();
        load_employee_data(sales_person.id, year, until_week).await?;
    }
    Ok(())
}

pub async fn refresh_employee_data() -> Result<(), ShiftyError> {
    let sales_person_id = EMPLOYEE_STORE.read().employee.sales_person.id;
    let year = EMPLOYEE_STORE.read().year;
    let until_week = EMPLOYEE_STORE.read().until_week;
    load_employee_data(sales_person_id, year, until_week).await
}

pub async fn delete_extra_hours(extra_hours_id: Uuid) -> Result<(), ShiftyError> {
    api::delete_extra_hour(CONFIG.read().clone(), extra_hours_id).await?;
    Ok(())
}

pub async fn delete_custom_extra_hours(custom_extra_hours_id: Uuid) -> Result<(), ShiftyError> {
    api::delete_custom_extra_hours(CONFIG.read().clone(), custom_extra_hours_id).await?;
    Ok(())
}

pub async fn employee_service(mut rx: UnboundedReceiver<EmployeeAction>) {
    while let Some(action) = rx.next().await {
        info!("EmployeeAction: {:?}", &action);
        match match action {
            EmployeeAction::LoadEmployeeDataUntilNow { sales_person_id } => {
                let year: u32 = js::get_current_year();
                let until_week = js::get_current_week();
                load_employee_data(sales_person_id, year, until_week).await
            }
            EmployeeAction::LoadCurrentEmployeeDataUntilNow => load_current_employee_data().await,
            EmployeeAction::Refresh => refresh_employee_data().await,
            EmployeeAction::DeleteExtraHours(extra_hours_id) => {
                delete_extra_hours(extra_hours_id).await
            }
            EmployeeAction::DeleteCustomExtraHour(extra_hours_id) => {
                delete_custom_extra_hours(extra_hours_id).await
            }
            EmployeeAction::FullYear => {
                let sales_person_id: Uuid = EMPLOYEE_STORE.read().employee.sales_person.id;
                let year = EMPLOYEE_STORE.read().year;
                let until_week = 54;
                load_employee_data(sales_person_id, year, until_week).await
            }
            EmployeeAction::UntilNow => {
                let sales_person_id = EMPLOYEE_STORE.read().employee.sales_person.id;
                let year = EMPLOYEE_STORE.read().year;
                if year == js::get_current_year() {
                    let until_week = js::get_current_week();
                    load_employee_data(sales_person_id, year, until_week).await
                } else {
                    load_employee_data(
                        sales_person_id,
                        year,
                        time::util::weeks_in_year(year as i32),
                    )
                    .await
                }
            }
            EmployeeAction::NextYear => {
                let sales_person_id = EMPLOYEE_STORE.read().employee.sales_person.id;
                let year = EMPLOYEE_STORE.read().year + 1;
                let until_week = if year == js::get_current_year() {
                    js::get_current_week()
                } else {
                    54
                };
                load_employee_data(sales_person_id, year, until_week).await
            }
            EmployeeAction::PrevYear => {
                let sales_person_id = EMPLOYEE_STORE.read().employee.sales_person.id;
                let year = EMPLOYEE_STORE.read().year - 1;
                let until_week = if year == js::get_current_year() {
                    js::get_current_week()
                } else {
                    54
                };
                load_employee_data(sales_person_id, year, until_week).await
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
