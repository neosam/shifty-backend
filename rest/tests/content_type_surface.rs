//! Phase 46 (Plan 46-03) — REST Content-Type surface drift-guard.
//!
//! Purpose (HYG-05)
//! ----------------
//! For every REST operation registered in the utoipa/OpenAPI doc, assert that
//! each declared 2xx response either
//!
//!   (a) declares at least one content-type from the whitelist below, or
//!   (b) is a spec-conform empty-body 204, or
//!   (c) is on the explicit `KNOWN_NO_BODY_2XX` grandfather list (pre-existing
//!       handlers that return 200/201 without a declared body — flagged for
//!       later cleanup outside HYG-05 scope; adding a NEW such handler still
//!       fails the test).
//!
//! Any content-type key that is neither `application/json` nor `text/plain`
//! fails the test hard with a structured offender report — this catches
//! silent drift when a new handler forgets `content_type = "application/json"`
//! or accidentally introduces an XML/HTML/PDF variant we did not plan for.
//!
//! Strategy (D-46-03)
//! ------------------
//! This is pure OpenAPI reflection — no live server, no DB fixture, no auth
//! mock. We trust the utoipa `#[utoipa::path(responses(...))]` declaration to
//! be the contract that frontend and external consumers see. Runtime
//! header-mismatch between contract and handler is a separate concern covered
//! by the existing domain-level integration tests.
//!
//! Rationale for this choice: a live-roundtrip approach via
//! `tower::ServiceExt::oneshot` would need per-handler DB seeding, auth
//! bootstrapping and request-body construction — cost is out of proportion
//! with the "did you set the Content-Type?" question. The pattern is already
//! established in the sibling `openapi_surface.rs` test (Phase 8 Plan 08-03).
//!
//! Drift-detection scope
//! ---------------------
//! * Whitelist is intentionally minimal (2 entries). Adding a new
//!   content-type — even a legitimate one like `text/csv` — requires an
//!   explicit whitelist edit with the handler reference. This forces the
//!   choice to be visible in code review.
//! * The `KNOWN_NO_BODY_2XX` list documents the 13 currently-shipping
//!   endpoints that return 200/201 without a body. Cleaning them up is
//!   tracked outside HYG-05 (either promote to 204 or add a real response
//!   DTO). New handlers may NOT join this list without explicit approval.
//! * Coverage-sanity check guards against silent iterator-emptying, e.g. if
//!   a future utoipa version renames `paths.paths` or `responses.responses`
//!   and the reflection loop returns zero — the whitelist test would then
//!   trivially pass and hide the drift.
//!
//! Determinism
//! -----------
//! Reflection reads `IndexMap`/`BTreeMap` in stable order. Three consecutive
//! runs MUST pass green.

use rest::ApiDoc;
use utoipa::openapi::path::{Operation, PathItem};
use utoipa::openapi::RefOr;
use utoipa::OpenApi;

/// Content-types allowed on any declared 2xx response. See D-46-03 for
/// rationale. Extending this list is a deliberate act — each entry must
/// document which handler(s) rely on it.
const ALLOWED_CONTENT_TYPES: &[&str] = &[
    // Default for the whole REST surface — every JSON-returning handler.
    "application/json",
    // Plain-text report endpoints:
    //   - rest/src/block_report.rs:24    GET /block-report/...  (rendered report as text)
    //   - rest/src/billing_period.rs:214 POST /billing-period/.../custom-report
    //   - rest/src/toggle.rs             GET /toggle/{name}/enabled  (bool as text)
    //   - rest/src/toggle.rs             GET /toggle/{name}/value    (raw string value)
    //   - rest/src/sales_person.rs       GET /sales-person/{id}/user (username as text)
    "text/plain",
];

/// Endpoints known to declare a 2xx response (200 or 201) without a response
/// body content-type. These are pre-existing shape drifts — the handler
/// either returns nothing (should be 204) or the OpenAPI annotation is
/// incomplete (should declare a body DTO).
///
/// Cleaning these up is out of scope for HYG-05 (test-layer only); each entry
/// is a candidate for a follow-up hygiene plan. Adding a NEW handler in this
/// shape must be blocked by removing the offending endpoint from this list
/// once fixed. New entries require reviewer approval — the test intentionally
/// makes drift visible in a PR.
///
/// Format: `(METHOD, path, status)`.
const KNOWN_NO_BODY_2XX: &[(&str, &str, &str)] = &[
    // Permission catalog — bulk-write handlers return 200 without a body.
    ("POST", "/permission/role", "200"),
    ("DELETE", "/permission/role", "200"),
    ("POST", "/permission/role-privilege/", "201"),
    ("DELETE", "/permission/role-privilege/", "200"),
    ("POST", "/permission/user", "201"),
    ("POST", "/permission/user-role", "201"),
    ("DELETE", "/permission/user-role", "200"),
    ("DELETE", "/permission/user/", "200"),
    // Sales-person <-> shiftplans bulk assignment.
    ("PUT", "/sales-person-shiftplan/{id}/shiftplans", "200"),
    // Shiftplan-catalog delete returns 200 with no body (should be 204).
    ("DELETE", "/shiftplan-catalog/{id}", "200"),
    // Toggle catalog create endpoints return 201 without echoing the created entity.
    ("POST", "/toggle", "201"),
    ("POST", "/toggle-group", "201"),
    ("POST", "/toggle-group/{group}/toggle/{toggle}", "201"),
];

/// Sanity threshold for coverage: current REST surface has 120 operations.
/// If the reflection loop suddenly reports fewer than this we assume utoipa
/// API changed shape (e.g. field renamed, iterator returns empty) and fail
/// loud rather than trivially passing the whitelist test.
const MIN_OPERATIONS_EXPECTED: usize = 40;

fn operations_of(item: &PathItem) -> Vec<(&'static str, &Operation)> {
    let mut ops = Vec::new();
    if let Some(op) = item.get.as_ref() {
        ops.push(("GET", op));
    }
    if let Some(op) = item.put.as_ref() {
        ops.push(("PUT", op));
    }
    if let Some(op) = item.post.as_ref() {
        ops.push(("POST", op));
    }
    if let Some(op) = item.delete.as_ref() {
        ops.push(("DELETE", op));
    }
    if let Some(op) = item.options.as_ref() {
        ops.push(("OPTIONS", op));
    }
    if let Some(op) = item.head.as_ref() {
        ops.push(("HEAD", op));
    }
    if let Some(op) = item.patch.as_ref() {
        ops.push(("PATCH", op));
    }
    if let Some(op) = item.trace.as_ref() {
        ops.push(("TRACE", op));
    }
    ops
}

fn is_grandfathered(method: &str, path: &str, status: &str) -> bool {
    KNOWN_NO_BODY_2XX
        .iter()
        .any(|(m, p, s)| *m == method && *p == path && *s == status)
}

#[test]
fn every_response_declares_known_content_type() {
    let openapi = ApiDoc::openapi();

    let mut offenders: Vec<String> = Vec::new();

    for (path, item) in openapi.paths.paths.iter() {
        for (method, op) in operations_of(item) {
            for (status, ref_or_resp) in op.responses.responses.iter() {
                // Only 2xx responses are contract-relevant for content-type drift.
                if !status.starts_with('2') {
                    continue;
                }

                let resp = match ref_or_resp {
                    RefOr::T(r) => r,
                    RefOr::Ref(_) => continue,
                };

                if resp.content.is_empty() {
                    // 204 legally has no body — spec-conform empty response.
                    if status == "204" {
                        continue;
                    }
                    // Pre-existing drift — grandfathered.
                    if is_grandfathered(method, path, status) {
                        continue;
                    }
                    offenders.push(format!(
                        "{method} {path} status={status}: no content-type declared \
                         (add `content_type = \"application/json\"` to the utoipa \
                         responses(...) entry, or make the handler return 204 for a \
                         truly empty body)"
                    ));
                    continue;
                }

                for content_type in resp.content.keys() {
                    if !ALLOWED_CONTENT_TYPES.contains(&content_type.as_str()) {
                        offenders.push(format!(
                            "{method} {path} status={status}: unknown content-type \
                             `{content_type}` (allowed: {ALLOWED_CONTENT_TYPES:?}). \
                             Either fix the handler declaration or extend \
                             ALLOWED_CONTENT_TYPES with the concrete handler reference."
                        ));
                    }
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "REST content-type surface drift detected ({n} offender(s)):\n{list}",
        n = offenders.len(),
        list = offenders.join("\n"),
    );
}

#[test]
fn content_type_surface_covers_all_openapi_operations() {
    let openapi = ApiDoc::openapi();

    let total_operations: usize = openapi
        .paths
        .paths
        .values()
        .map(|item| operations_of(item).len())
        .sum();

    assert!(
        total_operations >= MIN_OPERATIONS_EXPECTED,
        "Coverage sanity failed: iterated only {total_operations} operations, \
         expected >= {MIN_OPERATIONS_EXPECTED}. Either the REST surface shrank \
         dramatically or utoipa reflection is silently returning nothing. \
         Re-verify `ApiDoc::openapi().paths.paths` API before trusting the \
         whitelist test above.",
    );
}
