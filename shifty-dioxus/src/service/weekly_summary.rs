use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{error::ShiftyError, loader, state::weekly_overview::WeeklySummary};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

pub static WEEKLY_SUMMARY_STORE: GlobalSignal<Rc<[WeeklySummary]>> = Signal::global(|| Rc::new([]));

pub enum WeeklySummaryAction {
    LoadYear(u32),
}

async fn load_weekly_summary_year(year: u32) -> Result<(), ShiftyError> {
    let weekly_summary = loader::load_weekly_summary_for_year(CONFIG.read().clone(), year).await?;
    *WEEKLY_SUMMARY_STORE.write() = weekly_summary;
    Ok(())
}

pub async fn weekly_summary_service(mut rx: UnboundedReceiver<WeeklySummaryAction>) {
    while let Some(action) = rx.next().await {
        match match action {
            WeeklySummaryAction::LoadYear(year) => load_weekly_summary_year(year).await,
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
