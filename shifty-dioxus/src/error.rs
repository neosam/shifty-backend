use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShiftyError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Time ComponentRange error: {0}")]
    TimeComponentRange(#[from] time::error::ComponentRange),

    /// HTTP 409 Conflict — typically optimistic-lock failure on a versioned PUT.
    /// The wrapped string is the user-facing message (already translated).
    #[error("{0}")]
    Conflict(String),
}

pub fn error_handler(e: ShiftyError) {
    match e {
        ShiftyError::Reqwest(e) => {
            eprintln!("Error: {}", e);
            if let Some(StatusCode::UNAUTHORIZED) = e.status() {
                let _ = web_sys::window().expect("no window").location().reload();
            }
        }
        ShiftyError::TimeComponentRange(e) => {
            eprintln!("Error: {}", e);
        }
        ShiftyError::Conflict(msg) => {
            eprintln!("Conflict: {}", msg);
        }
    }
}

pub fn result_handler<T>(res: Result<T, ShiftyError>) -> Option<T> {
    match res {
        Ok(t) => Some(t),
        Err(e) => {
            error_handler(e);
            None
        }
    }
}
