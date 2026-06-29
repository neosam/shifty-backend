use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{error::ShiftyError, loader, state::shiftplan::BookingConflict};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
    week_guard::{is_current_selection, SELECTED_WEEK},
};

pub static BOOKING_CONFLICTS_STORE: GlobalSignal<Rc<[BookingConflict]>> =
    Signal::global(|| Rc::new([]));

pub enum BookingConflictAction {
    LoadWeek(u32, u8),
}

async fn load_booking_conflict_week(year: u32, week: u8) -> Result<(), ShiftyError> {
    let booking_conflicts =
        loader::load_bookings_conflicts_for_week(CONFIG.read().clone(), year, week).await?;
    // D-30-01 / SC3: only write the store if the week we loaded for is still the
    // selected week.  A result that arrives after a week-switch is silently dropped
    // (no store write, no error, no log).
    if is_current_selection((year, week), *SELECTED_WEEK.read()) {
        *BOOKING_CONFLICTS_STORE.write() = booking_conflicts;
    }
    Ok(())
}

pub async fn booking_conflicts_service(mut rx: UnboundedReceiver<BookingConflictAction>) {
    while let Some(action) = rx.next().await {
        match match action {
            BookingConflictAction::LoadWeek(year, week) => {
                load_booking_conflict_week(year, week).await
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
