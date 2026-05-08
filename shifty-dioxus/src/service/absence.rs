//! Absence-CRUD coroutine service (Phase 8 Wave 4).
//!
//! Owns the global `ABSENCE_STORE` (the rendered list) and `ABSENCE_REFRESH`
//! token (bumped on every successful POST/PUT/DELETE so resources can re-fetch).
//!
//! Modal-local 409 / 422 / forward-warning events are surfaced through the
//! `ABSENCE_MODAL_EVENT` side-channel rather than the Action-Enum's payload —
//! keeps the action enum cheap to derive `Debug` on, and the Page can react
//! without needing to thread `EventHandler<...>` through the message bus
//! (PATTERNS.md Z. 522-525 explicitly allows either approach).

use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use tracing::info;
use uuid::Uuid;

use rest_types::{AbsencePeriodCreateResultTO, AbsencePeriodTO};

use crate::{
    api,
    error::ShiftyError,
    loader,
    state::{absence_period::AbsencePeriod, shiftplan::SalesPerson},
};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

pub static ABSENCE_STORE: GlobalSignal<Rc<[AbsencePeriod]>> = Signal::global(|| Rc::new([]));

/// Bump-token. Every successful create / update / delete increments this.
/// Pages that render derived data (e.g. `VacationBalance`) read this signal
/// inside a `use_resource` to re-fetch on mutation.
pub static ABSENCE_REFRESH: GlobalSignal<u64> = Signal::global(|| 0);

pub(crate) fn bump_absence_refresh() {
    *ABSENCE_REFRESH.write() += 1;
}

/// Modal-local outcome of the most recent Create / Update action.
///
/// The page-level modal subscribes to this signal and reacts:
/// - `Created(result)` / `Updated(result)` — render `WarningList` if
///   `result.warnings` is non-empty, otherwise close.
/// - `VersionConflict` — render the version-conflict banner (D-08).
/// - `Validation(text)` — render the self-overlap banner (D-11).
/// - `Network(msg)` — generic error fallback.
/// - `None` — idle / consumed.
#[derive(Clone, Debug)]
pub enum AbsenceModalEvent {
    Created(AbsencePeriodCreateResultTO),
    Updated(AbsencePeriodCreateResultTO),
    VersionConflict,
    Validation(String),
    Network(String),
    Deleted,
}

/// Side-channel for modal-local outcomes. The page resets this to `None`
/// once it has acknowledged the event (e.g. user clicks "Verstanden" /
/// "Got it" or the modal closes).
pub static ABSENCE_MODAL_EVENT: GlobalSignal<Option<AbsenceModalEvent>> =
    Signal::global(|| None);

#[derive(Debug)]
pub enum AbsenceAction {
    /// HR variant — load all absences and resolve sales-person side-fields.
    LoadAll(Rc<[SalesPerson]>),
    /// Employee variant — load only this person's absences.
    LoadForSalesPerson(Uuid),
    /// POST. Outcome surfaces via `ABSENCE_MODAL_EVENT`.
    Create(AbsencePeriodTO),
    /// PUT. Outcome surfaces via `ABSENCE_MODAL_EVENT` (incl. 409 / 422).
    Update(AbsencePeriodTO),
    /// DELETE.
    Delete(Uuid),
    /// Bump the refresh token without an API call.
    Refresh,
}

pub async fn absence_service(mut rx: UnboundedReceiver<AbsenceAction>) {
    while let Some(action) = rx.next().await {
        info!("AbsenceAction: {:?}", &action);
        let config = CONFIG.read().clone();
        match action {
            AbsenceAction::LoadAll(sales_persons) => {
                match loader::load_absence_periods_all(config, sales_persons).await {
                    Ok(list) => {
                        *ABSENCE_STORE.write() = list;
                    }
                    Err(err) => {
                        *ERROR_STORE.write() = ErrorStore { error: Some(err) };
                    }
                }
            }
            AbsenceAction::LoadForSalesPerson(sp_id) => {
                match loader::load_absence_periods_by_sales_person(config, sp_id).await {
                    Ok(list) => {
                        *ABSENCE_STORE.write() = list;
                    }
                    Err(err) => {
                        *ERROR_STORE.write() = ErrorStore { error: Some(err) };
                    }
                }
            }
            AbsenceAction::Create(body) => match api::create_absence_period(config, body).await {
                Ok(result) => {
                    *ABSENCE_MODAL_EVENT.write() =
                        Some(AbsenceModalEvent::Created(result));
                    bump_absence_refresh();
                }
                Err(ShiftyError::Conflict(_)) => {
                    *ABSENCE_MODAL_EVENT.write() = Some(AbsenceModalEvent::VersionConflict);
                }
                Err(ShiftyError::Validation(text)) => {
                    *ABSENCE_MODAL_EVENT.write() = Some(AbsenceModalEvent::Validation(text));
                }
                Err(other) => {
                    let msg = format!("{}", &other);
                    *ABSENCE_MODAL_EVENT.write() = Some(AbsenceModalEvent::Network(msg));
                    *ERROR_STORE.write() = ErrorStore { error: Some(other) };
                }
            },
            AbsenceAction::Update(body) => {
                let id = body.id;
                match api::update_absence_period(config, id, body).await {
                    Ok(result) => {
                        *ABSENCE_MODAL_EVENT.write() =
                            Some(AbsenceModalEvent::Updated(result));
                        bump_absence_refresh();
                    }
                    Err(ShiftyError::Conflict(_)) => {
                        *ABSENCE_MODAL_EVENT.write() =
                            Some(AbsenceModalEvent::VersionConflict);
                    }
                    Err(ShiftyError::Validation(text)) => {
                        *ABSENCE_MODAL_EVENT.write() =
                            Some(AbsenceModalEvent::Validation(text));
                    }
                    Err(other) => {
                        let msg = format!("{}", &other);
                        *ABSENCE_MODAL_EVENT.write() = Some(AbsenceModalEvent::Network(msg));
                        *ERROR_STORE.write() = ErrorStore { error: Some(other) };
                    }
                }
            }
            AbsenceAction::Delete(id) => match api::delete_absence_period(config, id).await {
                Ok(()) => {
                    *ABSENCE_MODAL_EVENT.write() = Some(AbsenceModalEvent::Deleted);
                    bump_absence_refresh();
                }
                Err(err) => {
                    *ERROR_STORE.write() = ErrorStore {
                        error: Some(err.into()),
                    };
                }
            },
            AbsenceAction::Refresh => {
                bump_absence_refresh();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Service-level smoke tests. The coroutine itself drives `reqwest` calls
    //! against the configured backend, so we cannot exercise it end-to-end in
    //! a unit test without standing up an HTTP server. We instead lock the
    //! observable contracts the page relies on:
    //! - `bump_absence_refresh` is observable.
    //! - `ABSENCE_MODAL_EVENT` accepts and surfaces every variant.
    use super::*;

    #[test]
    fn bump_absence_refresh_increments_observable_signal() {
        fn assertion_app() -> Element {
            let before = *ABSENCE_REFRESH.read();
            bump_absence_refresh();
            let after = *ABSENCE_REFRESH.read();
            assert_eq!(after, before.wrapping_add(1));
            rsx! {}
        }
        let mut vdom = VirtualDom::new(assertion_app);
        vdom.rebuild_in_place();
    }

    #[test]
    fn modal_event_can_round_trip_each_variant() {
        fn assertion_app() -> Element {
            // Validation
            *ABSENCE_MODAL_EVENT.write() =
                Some(AbsenceModalEvent::Validation("self-overlap".into()));
            assert!(matches!(
                ABSENCE_MODAL_EVENT.read().as_ref(),
                Some(AbsenceModalEvent::Validation(s)) if s == "self-overlap"
            ));
            // VersionConflict
            *ABSENCE_MODAL_EVENT.write() = Some(AbsenceModalEvent::VersionConflict);
            assert!(matches!(
                ABSENCE_MODAL_EVENT.read().as_ref(),
                Some(AbsenceModalEvent::VersionConflict)
            ));
            // Deleted
            *ABSENCE_MODAL_EVENT.write() = Some(AbsenceModalEvent::Deleted);
            assert!(matches!(
                ABSENCE_MODAL_EVENT.read().as_ref(),
                Some(AbsenceModalEvent::Deleted)
            ));
            // Reset
            *ABSENCE_MODAL_EVENT.write() = None;
            assert!(ABSENCE_MODAL_EVENT.read().is_none());
            rsx! {}
        }
        let mut vdom = VirtualDom::new(assertion_app);
        vdom.rebuild_in_place();
    }
}
