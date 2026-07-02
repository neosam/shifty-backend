//! KW-Status store + coroutine.
//!
//! Fresh-fetch flow (D-39-06, T-39-05): the store never applies an optimistic
//! update. After a successful `Set` (server-side PUT) the coroutine performs a
//! fresh `Load` (GET) and only then writes the confirmed server value into
//! [`WEEK_STATUS_STORE`]. A failed `Set` leaves the store untouched and raises a
//! translated `WeekStatusSetError` banner via `ERROR_STORE`.

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{
    api,
    error::ShiftyError,
    i18n::Key,
    state::week_status::WeekStatus,
};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
    i18n::I18N,
};

#[derive(Clone, Default)]
pub struct WeekStatusStore {
    /// The confirmed server value. `WeekStatus::Unset` is the default and also
    /// what a missing row / missing server value maps to (D-39-04).
    pub status: WeekStatus,
}

pub static WEEK_STATUS_STORE: GlobalSignal<WeekStatusStore> =
    Signal::global(WeekStatusStore::default);

pub enum WeekStatusAction {
    Load { year: u32, week: u8 },
    Set { year: u32, week: u8, status: WeekStatus },
}

/// GET the current status and write it into the store. `None` (no row / 404)
/// maps to `WeekStatus::Unset`. On error the store keeps its last value.
async fn load_week_status(year: u32, week: u8) -> Result<(), ShiftyError> {
    let config = CONFIG.read().clone();
    let status = api::get_week_status(config, year, week)
        .await?
        .map(|to| WeekStatus::from(&to))
        .unwrap_or(WeekStatus::Unset);
    let mut store = WEEK_STATUS_STORE.write();
    store.status = status;
    Ok(())
}

pub async fn week_status_service(mut rx: UnboundedReceiver<WeekStatusAction>) {
    while let Some(action) = rx.next().await {
        match action {
            WeekStatusAction::Load { year, week } => {
                if let Err(err) = load_week_status(year, week).await {
                    *ERROR_STORE.write() = ErrorStore {
                        error: Some(err),
                    };
                }
            }
            WeekStatusAction::Set { year, week, status } => {
                let config = CONFIG.read().clone();
                match api::set_week_status(config, year, week, status).await {
                    // Fresh-fetch: only the server roundtrip updates the store.
                    Ok(()) => {
                        if let Err(err) = load_week_status(year, week).await {
                            *ERROR_STORE.write() = ErrorStore {
                                error: Some(err),
                            };
                        }
                    }
                    // No optimistic update — the store stays on its last value.
                    Err(err) => {
                        crate::error::log_shifty_error(&ShiftyError::from(err));
                        let message = I18N
                            .read()
                            .t(Key::WeekStatusSetError)
                            .as_ref()
                            .to_string();
                        *ERROR_STORE.write() = ErrorStore {
                            error: Some(ShiftyError::Validation(message)),
                        };
                    }
                }
            }
        }
    }
}
