//! Impersonation service — global store, coroutine, and pure status-mapping helper.
//!
//! ## Why a full client reload on Start and Stop (D-32-06 / IMP-04)
//!
//! `current_sales_person` in `shiftplan.rs` is a **component-local** Dioxus
//! signal; there is no global handle that would let this service reach in and
//! refresh it without a broad, out-of-scope refactor across many components.
//! A full `window.location.reload()` is the strongest no-stale-state guarantee:
//! it re-initialises *every* store from scratch via the post-change session.
//!
//! On remount the app fires `LoadStatus` (D-32-05) as its first init call, so
//! the banner and the IMPERSONATE_STORE always reflect the live backend state —
//! no additional synchronisation logic required.  This makes Start and Stop
//! symmetrical: both are a single API call followed by a full page reload.

use dioxus::prelude::*;
use futures_util::StreamExt;
use rest_types::ImpersonateTO;

use crate::{api, base_types::ImStr, error::ShiftyError};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

// ─── Store ────────────────────────────────────────────────────────────────────

/// Reactive store that tracks the current impersonation state.
///
/// `loaded` starts `false` so consumers (banner, person list) can distinguish
/// "not yet fetched" from "fetched and not impersonating".
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImpersonateStore {
    pub impersonating: bool,
    /// The raw `user_id` / username copied verbatim from `ImpersonateTO`
    /// (D-32-03: no name lookup, no change to the DTO).
    pub user_id: Option<ImStr>,
    /// `true` once the first successful `LoadStatus` response has been
    /// processed; also `true` after a non-admin 403 (mapped to cleared state).
    pub loaded: bool,
}

pub static IMPERSONATE_STORE: GlobalSignal<ImpersonateStore> =
    GlobalSignal::new(ImpersonateStore::default);

// ─── Pure status mapping (unit-tested) ───────────────────────────────────────

/// Maps a backend `ImpersonateTO` directly into an `ImpersonateStore`.
///
/// Pure function — no side-effects, no I/O, no DOM access — safe to unit-test
/// without a WASM runtime.  D-32-03: `user_id` is copied through unchanged;
/// no friendly-name lookup is performed.
pub fn status_from_to(to: ImpersonateTO) -> ImpersonateStore {
    ImpersonateStore {
        impersonating: to.impersonating,
        // Arc<str> → &str → ImStr (the cheapest allocation-free conversion path)
        user_id: to.user_id.as_deref().map(ImStr::from),
        loaded: true,
    }
}

// ─── Action enum ─────────────────────────────────────────────────────────────

pub enum ImpersonateAction {
    /// Load the current impersonation status from the backend (D-32-05).
    /// Called at app-mount so the banner survives a hard page reload.
    LoadStatus,
    /// Start impersonating the given user (D-32-06 / SC1).
    /// On success triggers a full client reload so the impersonated view and
    /// banner re-initialise from the GET-status init on remount.
    Start(ImStr),
    /// Stop the current impersonation session (D-32-06 / IMP-04 / SC4).
    /// On success triggers a full client reload so every user-bound store —
    /// including the component-local `current_sales_person` in `shiftplan.rs`
    /// — is re-initialised for the real admin with no stale impersonated state.
    Stop,
}

// ─── Service coroutine ────────────────────────────────────────────────────────

pub async fn impersonate_service(mut rx: UnboundedReceiver<ImpersonateAction>) {
    while let Some(action) = rx.next().await {
        match action {
            ImpersonateAction::LoadStatus => {
                // D-32-05: load status so the banner survives a hard reload.
                //
                // A 403 (FORBIDDEN) is the normal non-admin path — the server
                // rejects the GET because the caller lacks the admin privilege.
                // We map this to a cleared, loaded store so non-admins see
                // nothing and no error banner appears.  Only genuine transport
                // errors (network failure, 5xx, etc.) go to ERROR_STORE.
                match api::get_impersonate_status(CONFIG.read().clone()).await {
                    Ok(to) => {
                        *IMPERSONATE_STORE.write() = status_from_to(to);
                    }
                    Err(err) => {
                        let is_forbidden =
                            err.status() == Some(reqwest::StatusCode::FORBIDDEN);
                        if is_forbidden {
                            // Non-admin path — not impersonating, no error.
                            *IMPERSONATE_STORE.write() = ImpersonateStore {
                                impersonating: false,
                                user_id: None,
                                loaded: true,
                            };
                        } else {
                            *ERROR_STORE.write() = ErrorStore {
                                error: Some(ShiftyError::from(err)),
                            };
                        }
                    }
                }
            }

            ImpersonateAction::Start(user_id) => {
                // D-32-06 / SC1: start impersonation then reload the whole page.
                // The reload re-fires LoadStatus on remount — the single source
                // of truth for banner visibility.  Mirrors the reload pattern
                // in src/error.rs:66.
                match api::start_impersonate(CONFIG.read().clone(), user_id).await {
                    Ok(_) => {
                        let _ = web_sys::window().expect("no window").location().reload();
                    }
                    Err(err) => {
                        *ERROR_STORE.write() = ErrorStore {
                            error: Some(ShiftyError::from(err)),
                        };
                    }
                }
            }

            ImpersonateAction::Stop => {
                // D-32-06 / IMP-04 / SC4: stop impersonation then reload.
                // The reload re-initialises every user-bound store — including
                // the component-local `current_sales_person` in shiftplan.rs,
                // which has no global handle and cannot be targeted individually
                // without a broad out-of-scope refactor.  Mirrors error.rs:66.
                match api::stop_impersonate(CONFIG.read().clone()).await {
                    Ok(_) => {
                        let _ = web_sys::window().expect("no window").location().reload();
                    }
                    Err(err) => {
                        *ERROR_STORE.write() = ErrorStore {
                            error: Some(ShiftyError::from(err)),
                        };
                    }
                }
            }
        }
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn make_to(impersonating: bool, user_id: Option<&str>) -> ImpersonateTO {
        ImpersonateTO {
            impersonating,
            user_id: user_id.map(Arc::from),
        }
    }

    /// Impersonating=true with a user_id: store reflects both flags + loaded=true.
    #[test]
    fn status_from_to_impersonating_with_user() {
        let to = make_to(true, Some("alex"));
        let store = status_from_to(to);
        assert!(store.impersonating, "should be impersonating");
        assert_eq!(
            store.user_id,
            Some(ImStr::from("alex")),
            "user_id should match"
        );
        assert!(store.loaded, "should be marked loaded");
    }

    /// Impersonating=false + None: cleared store, loaded=true.
    #[test]
    fn status_from_to_not_impersonating() {
        let to = make_to(false, None);
        let store = status_from_to(to);
        assert!(!store.impersonating, "should not be impersonating");
        assert!(store.user_id.is_none(), "user_id should be None");
        assert!(store.loaded, "should be marked loaded");
    }

    /// Defensive: impersonating=true but user_id=None — we carry both flags
    /// through unchanged (D-32-03: no mutation of the backend's output).
    #[test]
    fn status_from_to_defensive_impersonating_without_user() {
        let to = make_to(true, None);
        let store = status_from_to(to);
        assert!(store.impersonating, "impersonating flag should be preserved");
        assert!(store.user_id.is_none(), "user_id should remain None");
        assert!(store.loaded, "should be marked loaded");
    }
}
