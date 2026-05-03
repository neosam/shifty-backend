//! Phase 4 OpenAPI snapshot lock (D-Phase4-11).
//!
//! Wave 2 (Plan 04-06): the cutover endpoints + DTOs are wired, so the snapshot
//! is now ready to be locked. First run produces `.snap.new`; the human reviews
//! the diff (no global `cargo insta` install per MEMORY.md), then renames
//! `.snap.new → .snap` (or runs `cargo insta accept` if locally installed).
//! Subsequent runs are pin-checks — any API surface drift fails the build.

use rest::ApiDoc;
use utoipa::OpenApi;

#[test]
fn openapi_snapshot_locks_full_api_surface() {
    let openapi = ApiDoc::openapi();
    insta::with_settings!({ sort_maps => true }, {
        insta::assert_json_snapshot!(openapi);
    });
}
