use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use tracing::info;
use uuid::Uuid;

use rest_types::ExtraHoursTO;

use crate::{
    api,
    error::ShiftyError,
    i18n::Key,
    js, loader,
    state::{
        employee::{CustomExtraHoursDefinition, Employee, ExtraHours},
        shiftplan::SalesPerson,
    },
};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
    i18n::I18N,
};

#[derive(Clone, PartialEq)]
pub struct EmployeeStore {
    pub year: u32,
    pub until_week: u8,

    pub employee: Employee,
    pub extra_hours: Rc<[ExtraHours]>,
    pub custom_extra_hours_definitions: Rc<[CustomExtraHoursDefinition]>,
}

/// Bumped from `refresh_employee_data` after every successful per-employee
/// data reload. The employees sidebar (`EmployeesList`) reads this inside its
/// `use_resource` closure so the cached list re-fetches whenever a mutation
/// changes a sidebar-visible aggregate (balance, target hours).
///
/// Contract: bump only from inside this service module. Outside callers should
/// trigger the refresh via the existing `EmployeeAction::Refresh` /
/// `refresh_employee_data` paths, which already ride along here.
pub static EMPLOYEES_LIST_REFRESH: GlobalSignal<u64> = Signal::global(|| 0);

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
        unpaid_leave_hours: 0.0,
        volunteer_hours: 0.0,
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
    UpdateExtraHours(ExtraHoursTO),
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
    let custom_extra_hours_definitions =
        match api::get_custom_extra_hours_by_sales_person(CONFIG.read().clone(), sales_person_id)
            .await
        {
            Ok(hours) => {
                let definitions: Rc<[CustomExtraHoursDefinition]> =
                    hours.iter().map(|h| h.into()).collect();
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
    let result = load_employee_data(sales_person_id, year, until_week).await;
    if result.is_ok() {
        bump_employees_list_refresh();
    }
    result
}

/// Increment the sidebar refresh token. Internal helper — the only call site
/// is `refresh_employee_data`. Exposed only for unit tests that lock the
/// observable-bump contract.
pub(crate) fn bump_employees_list_refresh() {
    *EMPLOYEES_LIST_REFRESH.write() += 1;
}

pub async fn delete_extra_hours(extra_hours_id: Uuid) -> Result<(), ShiftyError> {
    api::delete_extra_hour(CONFIG.read().clone(), extra_hours_id).await?;
    Ok(())
}

pub async fn update_extra_hours(extra_hours: ExtraHoursTO) -> Result<(), ShiftyError> {
    api::update_extra_hour(CONFIG.read().clone(), extra_hours).await?;
    Ok(())
}

pub async fn delete_custom_extra_hours(custom_extra_hours_id: Uuid) -> Result<(), ShiftyError> {
    api::delete_custom_extra_hours(CONFIG.read().clone(), custom_extra_hours_id).await?;
    Ok(())
}

/// Build the `ExtraHoursTO` payload that the modal hands to the service for
/// an update. Pulls identity (`id`, `sales_person_id`, `version`) from the
/// `editing` snapshot and the user-editable fields from the form state.
///
/// Pure function so the modal-side construction logic can be unit-tested
/// without spinning up a Dioxus VirtualDom or stubbing the network layer.
pub fn build_update_payload(
    editing: &ExtraHours,
    amount: f32,
    category: rest_types::ExtraHoursCategoryTO,
    description: std::sync::Arc<str>,
    date_time: time::PrimitiveDateTime,
) -> ExtraHoursTO {
    ExtraHoursTO {
        id: editing.id,
        sales_person_id: editing.sales_person_id,
        amount,
        category,
        description,
        date_time,
        created: None,
        deleted: None,
        version: editing.version,
    }
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
            EmployeeAction::UpdateExtraHours(extra_hours) => {
                match update_extra_hours(extra_hours).await {
                    Ok(()) => refresh_employee_data().await,
                    Err(ShiftyError::Conflict(_)) => {
                        let message = I18N
                            .read()
                            .t(Key::ExtraHoursConflictNotice)
                            .as_ref()
                            .to_string();
                        let refresh_result = refresh_employee_data().await;
                        *ERROR_STORE.write() = ErrorStore {
                            error: Some(ShiftyError::Conflict(message)),
                        };
                        refresh_result
                    }
                    Err(other) => Err(other),
                }
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

#[cfg(test)]
mod tests {
    //! Service-level unit tests for `EmployeeAction`.
    //!
    //! The async coroutine handler in `employee_service` reaches into the
    //! global `CONFIG` / `EMPLOYEE_STORE` / `ERROR_STORE` signals and issues
    //! real `reqwest` calls via `api::*`. There is no mock-API layer in this
    //! crate today, so we cannot drive the `UpdateExtraHours` arm end-to-end
    //! from a unit test without standing up an HTTP server.
    //!
    //! The behavior the user cares about is split as follows:
    //! - The `UpdateExtraHours` payload construction is covered here via
    //!   `build_update_payload` (pure function).
    //! - The dispatch wiring (modal sends `UpdateExtraHours(to)`) is covered
    //!   by the modal SSR tests in `component::extra_hours_modal::tests`,
    //!   which build the payload and route it through the modal's submit
    //!   handler. They do not run the network call but verify the dialog
    //!   renders in edit mode, prefills the editing snapshot, and exposes
    //!   the right form surface.
    //! - The 409 → refresh + ExtraHoursConflictNotice mapping is small enough
    //!   (one match arm) that it is covered by inspection; integration with
    //!   the live backend (which now ships `extra-hours-update`) is verified
    //!   by the manual exercise in tasks.md §9.5.
    use super::*;
    use rest_types::ExtraHoursCategoryTO;
    use std::rc::Rc;
    use std::sync::Arc;
    use time::macros::datetime;
    use uuid::Uuid;

    #[test]
    fn bump_employees_list_refresh_increments_observable_signal() {
        // Locks the contract: the bump helper produces a value change observable
        // outside the service module. `GlobalSignal` requires a Dioxus runtime
        // to be active, so we drive the read + bump from inside a VirtualDom
        // — the same harness the SSR component tests use.
        fn assertion_app() -> Element {
            let before = *EMPLOYEES_LIST_REFRESH.read();
            bump_employees_list_refresh();
            let after = *EMPLOYEES_LIST_REFRESH.read();
            assert_eq!(after, before.wrapping_add(1));
            rsx! {}
        }
        let mut vdom = VirtualDom::new(assertion_app);
        vdom.rebuild_in_place();
    }

    #[test]
    fn build_update_payload_carries_identity_from_editing_snapshot() {
        let editing = ExtraHours {
            id: Uuid::from_u128(0xaaaa),
            sales_person_id: Uuid::from_u128(0xbbbb),
            amount: 1.0,
            category: crate::state::employee::WorkingHoursCategory::ExtraWork("-".into()),
            description: Rc::from("old"),
            date_time: datetime!(2026-01-01 09:00:00),
            version: Uuid::from_u128(0xcccc),
        };

        let payload = build_update_payload(
            &editing,
            7.5,
            ExtraHoursCategoryTO::Holiday,
            Arc::from("new note"),
            datetime!(2026-04-15 10:30:00),
        );

        assert_eq!(payload.id, editing.id, "id must be the logical id");
        assert_eq!(payload.sales_person_id, editing.sales_person_id);
        assert_eq!(
            payload.version, editing.version,
            "version must be carried so optimistic-lock works"
        );
        assert_eq!(payload.amount, 7.5);
        assert!(matches!(payload.category, ExtraHoursCategoryTO::Holiday));
        assert_eq!(payload.description.as_ref(), "new note");
        assert_eq!(payload.date_time, datetime!(2026-04-15 10:30:00));
        assert!(
            payload.created.is_none(),
            "created is server-assigned on insert"
        );
        assert!(payload.deleted.is_none());
    }
}
