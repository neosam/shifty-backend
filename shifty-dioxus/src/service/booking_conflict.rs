use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{error::ShiftyError, loader, state::shiftplan::BookingConflict};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

pub static BOOKING_CONFLICTS_STORE: GlobalSignal<Rc<[BookingConflict]>> =
    Signal::global(|| Rc::new([]));

pub enum BookingConflictAction {
    LoadWeek(u32, u8),
}

async fn load_booking_conflict_week(year: u32, week: u8) -> Result<(), ShiftyError> {
    let booking_conflicts =
        loader::load_bookings_conflicts_for_week(CONFIG.read().clone(), year, week).await?;
    *BOOKING_CONFLICTS_STORE.write() = booking_conflicts;
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
