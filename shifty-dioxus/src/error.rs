use std::error::Error as _;
use std::fmt::Write as _;

use reqwest::StatusCode;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum ShiftyError {
    #[error("reqwest error: {}", format_reqwest_error(.0))]
    Reqwest(#[from] reqwest::Error),

    #[error("Time ComponentRange error: {0}")]
    TimeComponentRange(#[from] time::error::ComponentRange),

    /// HTTP 409 Conflict — typically optimistic-lock failure on a versioned PUT.
    /// The wrapped string is the user-facing message (already translated).
    #[error("{0}")]
    Conflict(String),

    /// HTTP 422 Validation error — e.g. self-overlap of absence periods (D-11).
    /// The wrapped string is the raw response body from the backend; callers
    /// (modal-side) typically render an i18n-translated banner and discard the
    /// raw text.
    #[error("{0}")]
    Validation(String),

    /// BUG-02 (v2.2): The `/user-invitation/invitation/user/{name}` endpoint
    /// returned a body that failed to deserialize into
    /// `Rc<[InvitationResponse]>`. Prior to Phase 44 this was silently
    /// swallowed to `Ok(Rc::new([]))`, hiding schema drift as an "empty list".
    ///
    /// The wrapped string is the `serde_json` error message concatenated with
    /// the first 200 characters of the response body — enough context to
    /// diagnose the parse failure without leaking secrets (invitation JSON
    /// only carries id/username/status/redeemed_at/token/invitation_link).
    /// Intentionally NOT `#[from] serde_json::Error` — the call site adds
    /// the body-head snippet, so conversion is explicit.
    #[error("invitation parse error: {0}")]
    InvitationParse(String),
}

/// Walks the `reqwest::Error` source chain and joins each level with ` → `.
/// `reqwest::Error::Display` only prints the top message ("builder error",
/// "decode error", …) and hides the actual cause (URL parse error, serde error,
/// hyper IO error, …). This helper exposes the full chain so a single log line
/// is enough to diagnose the failure.
fn format_reqwest_error(err: &reqwest::Error) -> String {
    let mut out = err.to_string();
    if let Some(status) = err.status() {
        let _ = write!(out, " [status={}]", status.as_u16());
    }
    if let Some(url) = err.url() {
        let _ = write!(out, " [url={url}]");
    }
    let mut source: Option<&dyn std::error::Error> = err.source();
    while let Some(s) = source {
        let _ = write!(out, " → {s}");
        source = s.source();
    }
    out
}

/// Logs a `ShiftyError` to the browser console (DevTools) via `tracing::error!`.
/// `dioxus_logger` routes `tracing` events to `console.{error,warn,info,…}`,
/// so this call surfaces in the browser DevTools — unlike `eprintln!`, which
/// writes to stderr and is invisible in WASM.
///
/// Service coroutines that store the error in `ERROR_STORE` should also call
/// this helper so the failure is visible in the console, not just in the
/// `<ErrorView>` overlay (which only shows the top-level Display).
pub fn log_shifty_error(err: &ShiftyError) {
    error!("ShiftyError: {err}");
}

pub fn error_handler(e: ShiftyError) {
    log_shifty_error(&e);
    if let ShiftyError::Reqwest(ref re) = e {
        if let Some(StatusCode::UNAUTHORIZED) = re.status() {
            let _ = web_sys::window().expect("no window").location().reload();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reqwest_display_includes_source_chain_for_invalid_url() {
        // reqwest produces a "builder error" wrapping a `url::ParseError`
        // when given a relative URL. Without the source-chain walk, the
        // top-level Display is just "builder error" — useless for
        // diagnosing the actual cause. With the walker, the URL parse
        // error must appear after the ` → ` separator.
        let req_err = reqwest::Client::new()
            .post("/relative-path")
            .build()
            .expect_err("relative URL must be a builder error");
        let shifty: ShiftyError = req_err.into();
        let rendered = shifty.to_string();
        assert!(
            rendered.contains("reqwest error:"),
            "missing prefix in {rendered:?}",
        );
        assert!(
            rendered.contains(" → "),
            "source chain not surfaced — display still hides the cause: {rendered:?}",
        );
        // Sanity: the inner `url::ParseError` for a relative URL is a
        // "relative URL without a base" message — must appear in the chain,
        // not just be hidden behind the top-level "builder error" label.
        assert!(
            rendered.to_lowercase().contains("relative")
                || rendered.to_lowercase().contains("base"),
            "expected url-parse cause in chain, got {rendered:?}",
        );
    }
}

