use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::error::ShiftyError;

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct ErrorStore {
    pub error: Option<ShiftyError>,
}
pub static ERROR_STORE: GlobalSignal<ErrorStore> = Signal::global(|| ErrorStore::default());

#[allow(dead_code)]
pub enum ErrorAction {
    SetError(ShiftyError),
}

#[allow(dead_code)]
pub async fn error_service(mut rx: UnboundedReceiver<ErrorAction>) {
    while let Some(action) = rx.next().await {
        match action {
            ErrorAction::SetError(error) => {
                *ERROR_STORE.write() = ErrorStore { error: Some(error) };
            }
        }
    }
}
