use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{error::ShiftyError, loader, state::weekly_overview::WeeklySummary};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

#[derive(Clone, Debug)]
pub struct WeeklySummaryStore {
    pub weekly_summary: Rc<[WeeklySummary]>,
    pub data_loaded: bool,
}
pub static WEEKLY_SUMMARY_STORE: GlobalSignal<WeeklySummaryStore> =
    GlobalSignal::new(|| WeeklySummaryStore {
        weekly_summary: Rc::new([]),
        data_loaded: false,
    });

pub enum WeeklySummaryAction {
    LoadYear(u32),
    LoadWeek(u32, u8),
}

async fn load_weekly_summary_year(year: u32) -> Result<(), ShiftyError> {
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = false;
    let weekly_summary = loader::load_weekly_summary_for_year(CONFIG.read().clone(), year).await?;
    (*WEEKLY_SUMMARY_STORE.write()).weekly_summary = weekly_summary;
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = true;
    Ok(())
}

async fn load_summary_for_week(year: u32, week: u8) -> Result<(), ShiftyError> {
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = false;
    let weekly_summary = loader::load_summary_for_week(CONFIG.read().clone(), year, week).await?;
    (*WEEKLY_SUMMARY_STORE.write()).weekly_summary = Rc::new([weekly_summary]);
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = true;
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
