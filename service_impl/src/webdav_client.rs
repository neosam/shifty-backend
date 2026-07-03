//! WebDAV client wrapper (Phase 48 EXP-01 + EXP-03 retry).
//!
//! Pure client: takes bytes + folder + filename, uploads to Nextcloud via
//! WebDAV Basic Auth. In-run retry with exponential backoff (2s/4s/8s per
//! Q7 in CONTEXT). No DAO, no config-service dep — the scheduler in 48-04
//! owns the config and wires it in.
//!
//! ## Trust boundaries (see 48-03-PLAN.md threat model)
//! - T-48-08 (token leak): custom `Debug` impl skips `app_token` field.
//! - T-48-09 (TLS spoofing): reqwest built with `rustls-tls`; no
//!   `danger_accept_invalid_certs` allowed.
//! - T-48-10 (DoS): retry capped at 3 attempts, per-request timeout 30s.
//! - T-48-SC (supply chain): `reqwest_dav` + `wiremock` deps human-verified
//!   during discuss-phase Q2 decision (see 48-CONTEXT.md).

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use base64::Engine;
use mockall::automock;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Method, StatusCode};

/// Default in-run retry backoff (production): 2s → 4s → 8s.
///
/// The array length also acts as the maximum attempt count (3 attempts).
pub const DEFAULT_RETRY_DELAYS: [Duration; 3] = [
    Duration::from_secs(2),
    Duration::from_secs(4),
    Duration::from_secs(8),
];

/// Per-request timeout (T-48-10 mitigation).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Errors surfaced by the WebDAV client.
///
/// The scheduler in 48-04 maps these to `pdf_export_config.last_error_message`.
#[derive(Debug, thiserror::Error)]
pub enum WebDavError {
    #[error("transient webdav error after {attempts} attempts: {reason}")]
    Transient {
        attempts: usize,
        reason: Arc<str>,
    },

    #[error("permanent webdav error: {status} {body}")]
    Permanent { status: u16, body: Arc<str> },

    #[error("http/io error: {0}")]
    Io(#[from] reqwest::Error),
}

/// Trait abstraction over WebDAV upload (Phase 48 Plan 04).
///
/// Extracted so the scheduler in [`crate::pdf_export_scheduler`] can be tested
/// with a mock upload without spinning up wiremock. The real implementation
/// is [`WebDavClient`]; tests inject `MockWebDavUpload`.
#[automock]
#[async_trait]
pub trait WebDavUpload: Send + Sync + 'static {
    /// Upload `bytes` to `folder/filename` under the WebDAV base URL.
    async fn upload_file(
        &self,
        folder: &str,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<(), WebDavError>;
}

/// Classification of an HTTP response for retry decisions.
///
/// Pure fn over `reqwest::StatusCode` + `on_mkcol` flag. Extracted so it can
/// be unit-tested independently of the network layer (see `classify_*` tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Classification {
    /// 2xx — request succeeded.
    Success,
    /// 405 Method Not Allowed on MKCOL — Nextcloud says "folder already
    /// exists". Treated as success for MKCOL only.
    MkcolExisting,
    /// 5xx / timeout / IO — worth retrying.
    Transient,
    /// 4xx (except MKCOL 405) — no retry, surface immediately.
    Permanent,
}

fn classify(status: StatusCode, on_mkcol: bool) -> Classification {
    if status.is_success() {
        Classification::Success
    } else if on_mkcol && status == StatusCode::METHOD_NOT_ALLOWED {
        Classification::MkcolExisting
    } else if status.is_server_error() {
        Classification::Transient
    } else {
        Classification::Permanent
    }
}

/// Thin WebDAV client around `reqwest`.
///
/// One instance per WebDAV target. Cheap to clone (all state is `Arc`d).
/// The custom `Debug` impl (below) intentionally omits `app_token` so that
/// accidental `dbg!(&client)` / `tracing::debug!(?client)` calls cannot
/// leak the Nextcloud credential (T-48-08).
#[derive(Clone)]
pub struct WebDavClient {
    client: reqwest::Client,
    base_url: Arc<str>,
    retry_delays: Arc<[Duration]>,
}

impl fmt::Debug for WebDavClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebDavClient")
            .field("base_url", &self.base_url)
            .field("retry_delays", &self.retry_delays)
            // client + credentials intentionally omitted (T-48-08)
            .finish_non_exhaustive()
    }
}

impl WebDavClient {
    /// Build a WebDAV client with production retry delays (2s/4s/8s).
    ///
    /// `base_url` is the WebDAV endpoint (e.g.
    /// `https://cloud.example.com/remote.php/dav/files/user`). It should NOT
    /// end with `/`; a trailing slash is stripped defensively.
    pub fn new(
        base_url: impl Into<Arc<str>>,
        user: &str,
        app_token: &str,
    ) -> Result<Self, WebDavError> {
        Self::new_with_delays(base_url, user, app_token, DEFAULT_RETRY_DELAYS)
    }

    /// Test-friendly constructor accepting custom retry delays.
    ///
    /// Tests pass `[Duration::from_millis(10); 3]` to keep the suite fast.
    pub fn new_with_delays(
        base_url: impl Into<Arc<str>>,
        user: &str,
        app_token: &str,
        delays: impl Into<Arc<[Duration]>>,
    ) -> Result<Self, WebDavError> {
        let auth = format!("{user}:{app_token}");
        let encoded = base64::engine::general_purpose::STANDARD.encode(auth.as_bytes());
        let header_value = format!("Basic {encoded}");
        let mut header_value = HeaderValue::from_str(&header_value).map_err(|_| {
            // Invalid header can only happen if user/token contain non-printable
            // ASCII, which is not expected. We surface it as Permanent 400 so
            // the scheduler can persist a meaningful last_error_message.
            WebDavError::Permanent {
                status: 400,
                body: Arc::from("invalid credentials produced non-ASCII auth header"),
            }
        })?;
        header_value.set_sensitive(true);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, header_value);

        // TLS backend: rustls-tls (T-48-09). NO danger_accept_invalid_certs —
        // any change here MUST be reviewed against the phase threat model.
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(REQUEST_TIMEOUT)
            .https_only(false) // allow http:// so wiremock tests work; production URLs are https://
            .build()?;

        let base_url = strip_trailing_slash(base_url.into());

        Ok(Self {
            client,
            base_url,
            retry_delays: delays.into(),
        })
    }

    /// Upload a file to `folder/filename` under the WebDAV base URL.
    ///
    /// Sequence per attempt:
    /// 1. MKCOL on `folder` (405 = "already exists", treated as success).
    /// 2. PUT `folder/filename` with the given bytes as body (overwrite).
    ///
    /// Retries the whole MKCOL+PUT sequence on transient errors according to
    /// `retry_delays`. Permanent errors (4xx except MKCOL-405) short-circuit
    /// immediately without retry.
    pub async fn upload_file(
        &self,
        folder: &str,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<(), WebDavError> {
        let folder = folder.trim_matches('/');
        let filename = filename.trim_matches('/');
        let attempts = self.retry_delays.len().max(1);

        let mut last_transient_reason: Arc<str> = Arc::from("unknown");
        for attempt in 0..attempts {
            match self.one_upload_pass(folder, filename, &bytes).await {
                Ok(()) => return Ok(()),
                Err(PassOutcome::Permanent { status, body }) => {
                    return Err(WebDavError::Permanent { status, body });
                }
                Err(PassOutcome::Transient { reason }) => {
                    last_transient_reason = reason;
                    // Sleep before the *next* attempt, unless this was the last one.
                    if attempt + 1 < attempts {
                        tokio::time::sleep(self.retry_delays[attempt]).await;
                    }
                }
            }
        }

        Err(WebDavError::Transient {
            attempts,
            reason: last_transient_reason,
        })
    }

    async fn one_upload_pass(
        &self,
        folder: &str,
        filename: &str,
        bytes: &[u8],
    ) -> Result<(), PassOutcome> {
        // 1. MKCOL — idempotent folder create.
        let mkcol_url = format!("{}/{}", self.base_url, folder);
        let mkcol_method = Method::from_bytes(b"MKCOL").expect("MKCOL is a valid HTTP method name");
        let mkcol_resp = match self.client.request(mkcol_method, &mkcol_url).send().await {
            Ok(r) => r,
            Err(e) => {
                return Err(PassOutcome::Transient {
                    reason: Arc::from(format!("MKCOL network error: {e}")),
                });
            }
        };

        match classify(mkcol_resp.status(), true) {
            Classification::Success | Classification::MkcolExisting => {
                // proceed to PUT
            }
            Classification::Transient => {
                let status = mkcol_resp.status().as_u16();
                let body = mkcol_resp.text().await.unwrap_or_default();
                return Err(PassOutcome::Transient {
                    reason: Arc::from(format!("MKCOL {status}: {body}")),
                });
            }
            Classification::Permanent => {
                let status = mkcol_resp.status().as_u16();
                let body = mkcol_resp.text().await.unwrap_or_default();
                return Err(PassOutcome::Permanent {
                    status,
                    body: Arc::from(body),
                });
            }
        }

        // 2. PUT — upload bytes (overwrite).
        let put_url = format!("{}/{}/{}", self.base_url, folder, filename);
        let put_resp = match self
            .client
            .put(&put_url)
            .body(bytes.to_vec())
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return Err(PassOutcome::Transient {
                    reason: Arc::from(format!("PUT network error: {e}")),
                });
            }
        };

        match classify(put_resp.status(), false) {
            Classification::Success => Ok(()),
            Classification::MkcolExisting => {
                // Unreachable: on_mkcol=false so 405 → Permanent. Keep as
                // defensive Permanent to satisfy exhaustiveness.
                let status = put_resp.status().as_u16();
                let body = put_resp.text().await.unwrap_or_default();
                Err(PassOutcome::Permanent {
                    status,
                    body: Arc::from(body),
                })
            }
            Classification::Transient => {
                let status = put_resp.status().as_u16();
                let body = put_resp.text().await.unwrap_or_default();
                Err(PassOutcome::Transient {
                    reason: Arc::from(format!("PUT {status}: {body}")),
                })
            }
            Classification::Permanent => {
                let status = put_resp.status().as_u16();
                let body = put_resp.text().await.unwrap_or_default();
                Err(PassOutcome::Permanent {
                    status,
                    body: Arc::from(body),
                })
            }
        }
    }
}

#[async_trait]
impl WebDavUpload for WebDavClient {
    async fn upload_file(
        &self,
        folder: &str,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<(), WebDavError> {
        // Delegate to the inherent implementation. Kept as a thin wrapper so
        // both `client.upload_file(...)` and `<dyn WebDavUpload>::upload_file(...)`
        // stay valid callers.
        WebDavClient::upload_file(self, folder, filename, bytes).await
    }
}

/// Outcome of a single MKCOL+PUT pass — kept private, mapped to
/// `WebDavError` by the retry loop.
enum PassOutcome {
    Transient { reason: Arc<str> },
    Permanent { status: u16, body: Arc<str> },
}

fn strip_trailing_slash(url: Arc<str>) -> Arc<str> {
    if let Some(stripped) = url.strip_suffix('/') {
        Arc::from(stripped)
    } else {
        url
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Base64 of `testuser:token` — verifies the client sends the exact
    /// Basic-Auth header expected by Nextcloud.
    const AUTH_HEADER: &str = "Basic dGVzdHVzZXI6dG9rZW4=";

    fn fast_delays() -> [Duration; 3] {
        [Duration::from_millis(10); 3]
    }

    fn make_client(base_url: String) -> WebDavClient {
        WebDavClient::new_with_delays(base_url, "testuser", "token", fast_delays())
            .expect("build test client")
    }

    // ---- Test A: happy path ---------------------------------------------
    #[tokio::test]
    async fn put_success_returns_ok() {
        let server = MockServer::start().await;

        Mock::given(method("MKCOL"))
            .and(path("/Schichtplaene"))
            .and(header("authorization", AUTH_HEADER))
            .respond_with(ResponseTemplate::new(201))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/Schichtplaene/foo.pdf"))
            .and(header("authorization", AUTH_HEADER))
            .respond_with(ResponseTemplate::new(201))
            .expect(1)
            .mount(&server)
            .await;

        let client = make_client(server.uri());
        client
            .upload_file("Schichtplaene", "foo.pdf", b"hello".to_vec())
            .await
            .expect("upload succeeds");
        // Mock `.expect(1)` panics on drop if not exactly 1 request seen.
    }

    // ---- Test B: MKCOL 405 is treated as success -------------------------
    #[tokio::test]
    async fn mkcol_folder_exists_treated_as_success() {
        let server = MockServer::start().await;

        Mock::given(method("MKCOL"))
            .and(path("/Schichtplaene"))
            .respond_with(ResponseTemplate::new(405))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/Schichtplaene/foo.pdf"))
            .respond_with(ResponseTemplate::new(201))
            .expect(1)
            .mount(&server)
            .await;

        let client = make_client(server.uri());
        client
            .upload_file("Schichtplaene", "foo.pdf", b"hi".to_vec())
            .await
            .expect("mkcol 405 must not fail");
    }

    // ---- Test C: MKCOL 201 then PUT 201 ----------------------------------
    #[tokio::test]
    async fn mkcol_created_then_put_success() {
        let server = MockServer::start().await;

        Mock::given(method("MKCOL"))
            .and(path("/Schichtplaene"))
            .respond_with(ResponseTemplate::new(201))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/Schichtplaene/foo.pdf"))
            .respond_with(ResponseTemplate::new(201))
            .expect(1)
            .mount(&server)
            .await;

        let client = make_client(server.uri());
        client
            .upload_file("Schichtplaene", "foo.pdf", b"body".to_vec())
            .await
            .expect("mkcol 201 + put 201 must succeed");
    }

    // ---- Test D: transient 503 twice, then success on third attempt -----
    #[tokio::test]
    async fn transient_5xx_retries_and_succeeds() {
        let server = MockServer::start().await;

        // MKCOL always succeeds (201). Mounted with no expect() so any count
        // is fine (each retry pass re-runs MKCOL).
        Mock::given(method("MKCOL"))
            .and(path("/Schichtplaene"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&server)
            .await;

        // First two PUTs fail transiently, third succeeds.
        Mock::given(method("PUT"))
            .and(path("/Schichtplaene/foo.pdf"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(2)
            .expect(2)
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/Schichtplaene/foo.pdf"))
            .respond_with(ResponseTemplate::new(201))
            .expect(1)
            .mount(&server)
            .await;

        let client = make_client(server.uri());
        client
            .upload_file("Schichtplaene", "foo.pdf", b"body".to_vec())
            .await
            .expect("must succeed after retries");
    }

    // ---- Test E: permanent 4xx → no retry --------------------------------
    #[tokio::test]
    async fn permanent_4xx_no_retry() {
        let server = MockServer::start().await;

        Mock::given(method("MKCOL"))
            .and(path("/Schichtplaene"))
            .respond_with(ResponseTemplate::new(201))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/Schichtplaene/foo.pdf"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1) // exactly one attempt, no retry on 4xx
            .mount(&server)
            .await;

        let client = make_client(server.uri());
        let err = client
            .upload_file("Schichtplaene", "foo.pdf", b"body".to_vec())
            .await
            .expect_err("401 must surface as error");
        match err {
            WebDavError::Permanent { status, .. } => assert_eq!(status, 401),
            other => panic!("expected Permanent 401, got {other:?}"),
        }
    }

    // ---- Test F: transient exhausted after 3 tries ----------------------
    #[tokio::test]
    async fn transient_exhausted_returns_error() {
        let server = MockServer::start().await;

        Mock::given(method("MKCOL"))
            .and(path("/Schichtplaene"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/Schichtplaene/foo.pdf"))
            .respond_with(ResponseTemplate::new(503))
            .expect(3) // exactly 3 attempts
            .mount(&server)
            .await;

        let client = make_client(server.uri());
        let err = client
            .upload_file("Schichtplaene", "foo.pdf", b"body".to_vec())
            .await
            .expect_err("must fail after 3 attempts");
        match err {
            WebDavError::Transient { attempts, .. } => assert_eq!(attempts, 3),
            other => panic!("expected Transient after 3 attempts, got {other:?}"),
        }
    }

    // ---- Debug-impl must not leak the app_token (T-48-08) ---------------
    #[test]
    fn debug_impl_does_not_leak_app_token() {
        let client = WebDavClient::new_with_delays(
            "https://cloud.example.com/remote.php/dav/files/testuser",
            "testuser",
            "super-secret-token-abc123",
            fast_delays(),
        )
        .expect("build client");

        let dbg = format!("{client:?}");
        assert!(
            !dbg.contains("super-secret-token-abc123"),
            "Debug output leaked token: {dbg}"
        );
        assert!(
            !dbg.contains("token"),
            "Debug output should not include a token field name: {dbg}"
        );
    }

    // ---- classify() unit tests -------------------------------------------
    #[test]
    fn classify_2xx_is_success() {
        assert_eq!(classify(StatusCode::OK, false), Classification::Success);
        assert_eq!(classify(StatusCode::CREATED, true), Classification::Success);
        assert_eq!(
            classify(StatusCode::NO_CONTENT, false),
            Classification::Success
        );
    }

    #[test]
    fn classify_405_on_mkcol_is_mkcol_existing() {
        assert_eq!(
            classify(StatusCode::METHOD_NOT_ALLOWED, true),
            Classification::MkcolExisting
        );
    }

    #[test]
    fn classify_405_off_mkcol_is_permanent() {
        assert_eq!(
            classify(StatusCode::METHOD_NOT_ALLOWED, false),
            Classification::Permanent
        );
    }

    #[test]
    fn classify_5xx_is_transient() {
        assert_eq!(
            classify(StatusCode::INTERNAL_SERVER_ERROR, false),
            Classification::Transient
        );
        assert_eq!(
            classify(StatusCode::SERVICE_UNAVAILABLE, true),
            Classification::Transient
        );
        assert_eq!(
            classify(StatusCode::GATEWAY_TIMEOUT, false),
            Classification::Transient
        );
    }

    #[test]
    fn classify_401_403_404_are_permanent() {
        assert_eq!(
            classify(StatusCode::UNAUTHORIZED, false),
            Classification::Permanent
        );
        assert_eq!(
            classify(StatusCode::FORBIDDEN, false),
            Classification::Permanent
        );
        assert_eq!(
            classify(StatusCode::NOT_FOUND, false),
            Classification::Permanent
        );
    }
}
