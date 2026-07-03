use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{loader, state::employee_work_details::WorkingHoursMini};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
    week_guard::{is_current_selection, SELECTED_WEEK},
};

pub static WORKING_HOURS_MINI: GlobalSignal<Rc<[WorkingHoursMini]>> = Signal::global(|| [].into());
pub enum WorkingHoursMiniAction {
    // Load working hours for a specific week (year, week, fetch_balance)
    LoadWorkingHoursMini(u32, u8, bool),
}

pub async fn working_hours_mini_service(mut rx: UnboundedReceiver<WorkingHoursMiniAction>) {
    while let Some(action) = rx.next().await {
        match action {
            WorkingHoursMiniAction::LoadWorkingHoursMini(year, week, fetch_balance) => {
                let working_hours = loader::load_working_hours_minified_for_week(
                    CONFIG.read().clone(),
                    year,
                    week,
                    fetch_balance,
                )
                .await;
                match working_hours {
                    Ok(working_hours) => {
                        // SC2/D-30-02: the mini working-hours bars are a fourth summary
                        // loader under the shiftplan dispatched on every week switch.
                        // Drop the result if the user has already navigated to another
                        // week while this request was in flight (silent drop, SC3).
                        if is_current_selection((year, week), *SELECTED_WEEK.read()) {
                            *WORKING_HOURS_MINI.write() = working_hours;
                        }
                    }
                    Err(err) => {
                        *ERROR_STORE.write() = ErrorStore {
                            error: Some(err),
                        };
                    }
                }
            }
        }
    }
}
