use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShiftyError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}
