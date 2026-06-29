use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{error::ShiftyError, loader, state::weekly_overview::WeeklySummary};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
    week_guard::{is_current_selection, SELECTED_WEEK},
};

#[derive(Clone, Debug)]
pub struct WeeklySummaryStore {
    pub weekly_summary: Rc<[WeeklySummary]>,
    pub data_loaded: bool,
    /// The `(year, week)` that the currently stored weekly summary was loaded for.
    /// `None` means the store holds year-view data (D-30-04) or has not yet been
    /// populated with a week load.  Used by the render-guard in `shiftplan.rs` to
    /// ensure the summary cards never display data from a non-selected week.
    pub loaded_week: Option<(u32, u8)>,
}
pub static WEEKLY_SUMMARY_STORE: GlobalSignal<WeeklySummaryStore> =
    GlobalSignal::new(|| WeeklySummaryStore {
        weekly_summary: Rc::new([]),
        data_loaded: false,
        loaded_week: None,
    });

pub enum WeeklySummaryAction {
    LoadYear(u32),
    LoadWeek(u32, u8),
}

async fn load_weekly_summary_year(year: u32) -> Result<(), ShiftyError> {
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = false;
    let weekly_summary = loader::load_weekly_summary_for_year(CONFIG.read().clone(), year).await?;
    // D-30-04: year data must never satisfy the week render-guard — stamp loaded_week = None
    // so summary cards (which compare loaded_week against the selected week) stay in the
    // empty/loading state while a year view is stored here.
    {
        let mut store = WEEKLY_SUMMARY_STORE.write();
        store.weekly_summary = weekly_summary;
        store.loaded_week = None;
        store.data_loaded = true;
    }
    Ok(())
}

async fn load_summary_for_week(year: u32, week: u8) -> Result<(), ShiftyError> {
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = false;
    let weekly_summary = loader::load_summary_for_week(CONFIG.read().clone(), year, week).await?;
    // D-30-01 / SC3: only write the store if the week we loaded for is still the
    // selected week.  A result that arrives after a week-switch is silently dropped
    // (no store write, no error, no log).
    let selected = *SELECTED_WEEK.read();
    if is_current_selection((year, week), selected) {
        let mut store = WEEKLY_SUMMARY_STORE.write();
        store.weekly_summary = Rc::new([weekly_summary]);
        store.loaded_week = Some((year, week));
        store.data_loaded = true;
    }
    Ok(())
}

pub async fn weekly_summary_service(mut rx: UnboundedReceiver<WeeklySummaryAction>) {
    while let Some(action) = rx.next().await {
        match match action {
            WeeklySummaryAction::LoadYear(year) => load_weekly_summary_year(year).await,
            WeeklySummaryAction::LoadWeek(year, week) => load_summary_for_week(year, week).await,
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
