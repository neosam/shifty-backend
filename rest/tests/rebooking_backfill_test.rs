//! Phase 56 (56-04) — F4 Backfill REST surface + contract test.
//!
//! Scope
//! -----
//! The behavioral core of the backfill engine (dry-run writes nothing, apply
//! persists `AutoCronBackfill` batches, HR-Admin gate, year-boundary walk, DoS
//! upper-bound clamp) is exhaustively covered by the SERVICE layer in Plan 02
//! (`service_impl/src/test/rebooking_backfill.rs`, 7 tests). The REST layer is
//! a thin delegate — its own logic is (a) from-week parsing/validation, (b)
//! pagination slicing, (c) DTO mapping, and (d) route/OpenAPI registration.
//!
//! Parsing/validation is unit-tested in-crate (`rest/src/rebooking.rs` tests
//! module: `parse_iso_week_*`). This integration file asserts the OpenAPI
//! CONTRACT the frontend and external consumers depend on: the endpoint exists
//! at the expected nested path, declares the documented status codes (200 /
//! 400 / 403), and the request/response DTOs are registered as schemas.
//!
//! This mirrors the established codebase pattern (`openapi_surface.rs`,
//! `content_type_surface.rs`) which deliberately avoids a live-app DB+auth
//! oneshot harness — that would require per-handler DB seeding and mocking the
//! full `RestStateDef` (~45 associated services) for coverage the service
//! layer already owns. See `content_type_surface.rs` for the rationale.

use rest::ApiDoc;
use utoipa::OpenApi;

/// The F4 backfill endpoint must be registered at the `/admin/rebooking`
/// nest with the `/backfill` sub-path. Disappearance signals the nest was
/// unwired from `ApiDoc`'s `nest(...)` block or the utoipa path changed.
#[test]
fn backfill_path_registered_under_admin_rebooking() {
    let openapi = ApiDoc::openapi();
    let actual: Vec<&str> = openapi.paths.paths.keys().map(String::as_str).collect();

    assert!(
        actual.contains(&"/admin/rebooking/backfill"),
        "OpenAPI surface drift: '/admin/rebooking/backfill' missing.\n\
         actual paths ({n} total): {actual:#?}",
        n = actual.len(),
    );
}

/// The backfill request/response DTOs must appear in `components.schemas` so
/// the generated client sees the contract.
#[test]
fn backfill_dtos_registered_as_schemas() {
    let openapi = ApiDoc::openapi();
    let components = openapi
        .components
        .as_ref()
        .expect("ApiDoc::openapi() must populate components");
    let actual: Vec<&str> = components.schemas.keys().map(String::as_str).collect();

    for expected in ["BackfillRequestTO", "BackfillPreviewTO", "BackfillPreviewItemTO"] {
        assert!(
            actual.contains(&expected),
            "OpenAPI surface drift: schema '{expected}' missing.\n\
             actual schemas ({n} total): {actual:#?}",
            n = actual.len(),
        );
    }
}

/// The backfill operation must declare the documented status codes: 200
/// (preview / apply result), 400 (malformed from-week), 403 (non-HR-Admin).
/// This pins the security-relevant contract (T-56-10 / T-56-11): a consumer
/// must be told that a bad from-week is a 400 and a non-admin caller is a 403.
#[test]
fn backfill_operation_declares_200_400_403() {
    let openapi = ApiDoc::openapi();
    let item = openapi
        .paths
        .paths
        .get("/admin/rebooking/backfill")
        .expect("/admin/rebooking/backfill path must exist");

    let post = item
        .post
        .as_ref()
        .expect("/admin/rebooking/backfill must have a POST operation");

    for code in ["200", "400", "403"] {
        assert!(
            post.responses.responses.contains_key(code),
            "backfill POST must declare a '{code}' response; declared: {:#?}",
            post.responses.responses.keys().collect::<Vec<_>>(),
        );
    }
}
