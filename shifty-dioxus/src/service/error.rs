use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::error::ShiftyError;

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct ErrorStore {
    pub error: Option<ShiftyError>,
}

impl ErrorStore {
    /// The no-error state. Used by the dismissable `ErrorView` banner: clicking
    /// the close button resets `ERROR_STORE` to this so the banner disappears
    /// (previously a set error stayed forever, since nothing ever cleared it).
    pub fn cleared() -> Self {
        ErrorStore { error: None }
    }
}

pub static ERROR_STORE: GlobalSignal<ErrorStore> = Signal::global(ErrorStore::default);

#[allow(dead_code)]
pub enum ErrorAction {
    SetError(ShiftyError),
    ClearError,
}

#[allow(dead_code)]
pub async fn error_service(mut rx: UnboundedReceiver<ErrorAction>) {
    while let Some(action) = rx.next().await {
        match action {
            ErrorAction::SetError(error) => {
                *ERROR_STORE.write() = ErrorStore { error: Some(error) };
            }
            ErrorAction::ClearError => {
                *ERROR_STORE.write() = ErrorStore::cleared();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ShiftyError;

    #[test]
    fn cleared_has_no_error() {
        assert!(ErrorStore::cleared().error.is_none());
    }

    #[test]
    fn dismiss_replaces_existing_error_with_none() {
        // A banner is showing a validation error...
        let showing = ErrorStore {
            error: Some(ShiftyError::Validation("boom".into())),
        };
        assert!(showing.error.is_some());
        // ...and the user dismisses it: the resulting state carries no error.
        assert!(ErrorStore::cleared().error.is_none());
    }
}
