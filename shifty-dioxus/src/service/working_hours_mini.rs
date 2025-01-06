use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{loader, state::employee_work_details::WorkingHoursMini};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

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
