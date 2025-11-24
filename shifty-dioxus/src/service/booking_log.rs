use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{loader, state::booking_log::BookingLog};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

pub static BOOKING_LOG_STORE: GlobalSignal<Rc<[BookingLog]>> = Signal::global(|| [].into());

pub enum BookingLogAction {
    LoadBookingLog(u32, u8),
}

pub async fn booking_log_service(mut rx: UnboundedReceiver<BookingLogAction>) {
    while let Some(action) = rx.next().await {
        match action {
            BookingLogAction::LoadBookingLog(year, week) => {
                let booking_log =
                    loader::load_booking_log(CONFIG.read().clone(), year, week).await;
                match booking_log {
                    Ok(booking_log) => {
                        *BOOKING_LOG_STORE.write() = booking_log;
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
